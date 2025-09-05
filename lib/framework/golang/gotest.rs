use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use crate::core::errors::FrameworkError;
use crate::core::metadata::RunnableMeta;
use crate::core::types::Command;
use crate::core::types::CursorPosition;
use crate::core::types::Runnable;
use crate::core::types::Target;
use crate::core::{
    enums::Capability,
    traits::{Framework, FrameworkProvider},
    types::CapabilityDetails,
};
use crate::framework::golang::treesitter::operations::extract_gotest_subtests;
use crate::framework::golang::treesitter::queries::gotest_file_test_methods;
use crate::framework::golang::treesitter::queries::gotest_test_function;

use crate::core::enums::Language as crate_language;
use crate::framework::golang::treesitter::operations::parse_tree;
use crate::treesitter::node as crate_treesitter_node;
use tree_sitter::{Language, Node, Query, QueryCursor};

pub struct GotestProvider;

static FILE_SUFFIX: &str = "_test.go";

static SEARCH_STRATEGIES: LazyLock<HashSet<CapabilityDetails>> = LazyLock::new(|| {
    let mut res = HashSet::with_capacity(3);
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::Nearest,
        description: "Test Nearest".to_string(),
    });
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::Method,
        description: "Test Function".to_string(),
    });
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::File,
        description: "Test File".to_string(),
    });

    res
});

impl FrameworkProvider for GotestProvider {
    fn create(&self) -> Box<dyn Framework> {
        Box::new(GotestProvider::new())
    }

    fn name(&self) -> &'static str {
        "GoTest"
    }

    fn language(&self) -> crate_language {
        crate_language::Golang
    }

    fn capability(&self) -> Capability {
        Capability::TestRunner
    }
}

impl Framework for GotestProvider {
    fn detect(&self, target: &Target) -> bool {
        if target.category != self.capability() {
            return false;
        }

        target.buffer.filepath.to_string().ends_with(FILE_SUFFIX)
    }

    fn generate_command(&self, runnable: Runnable) -> Command {
        let mut cmd = Command {
            command: "go".to_string(),
            args: vec!["test".to_string(), "-v".to_string()],
        };

        cmd.args.push(runnable.filepath);
        if let Some(meta) = runnable.meta.get_meta() {
            if !meta.build_tags.is_empty() {
                cmd.args
                    .push(format!("-tags={}", meta.build_tags.join(",")));
            }
        }
        cmd
    }

    fn runnables(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError> {
        /*
         * Goals
         *   - Search set to nearest, return singular test.
         *       If the test contains a subtest, check if that subtest contains the cursory position
         *   - Search set to method, return singular top level test
         *   - Search set to file, return all test names in a file
         *   -
         * */
        let tree = parse_tree::op::execute(target.buffer.content)?;
        let mut walker = tree.walk();
        walker.goto_first_child_for_point(target.buffer.position.to_point());
        let walker_node = walker.node();
        match target.search_strategy {
            crate::core::enums::Search::File => {
                let parent_runnables = self.get_all_test_methods(walker_node, target);
                if parent_runnables.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found no tests in this file".to_string(),
                    ));
                }
                let parent_runnables = parent_runnables.unwrap();
                let mut res: Vec<Runnable> = vec![];
                for parent in parent_runnables.into_iter() {
                    let subtests = extract_gotest_subtests::op::execute(
                        walker_node,
                        parent.to_owned(),
                        target,
                    );
                    if let Some(sub) = subtests {
                        res.extend(sub);
                    } else {
                        res.push(parent);
                    }
                }
                Ok(res)
            }
            crate::core::enums::Search::Method => {
                let res = self.get_single_test_method(walker_node, target);
                if res.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }
                Ok(vec![res.unwrap()])
            }
            crate::core::enums::Search::Nearest => {
                let parent_runnable = self.get_single_test_method(walker_node, target);
                if parent_runnable.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }

                let parent_runnable = parent_runnable.unwrap();
                let subtests = extract_gotest_subtests::op::execute(
                    walker_node,
                    parent_runnable.to_owned(),
                    target,
                );
                if subtests.is_none() {
                    return Ok(vec![parent_runnable]);
                }

                Ok(subtests.unwrap())
            }
        }
    }

    fn capabilities(&self) -> &HashSet<CapabilityDetails> {
        LazyLock::force(&SEARCH_STRATEGIES)
    }

    fn search_for_capability(&self, description: &str) -> Option<&CapabilityDetails> {
        let c = LazyLock::force(&SEARCH_STRATEGIES);
        c.iter().find(|s| s.description == description).clone()
    }
}

impl GotestProvider {
    pub fn new() -> Self {
        Self {}
    }

    fn get_test_function_query(&self) -> Option<Query> {
        let query_pattern = gotest_test_function::query();
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), &query_pattern).ok()?;
        Some(query)
    }

    fn get_single_test_method(&self, node: Node, target: &Target) -> Option<Runnable> {
        let current_node_position = node.start_position();
        let query = self.get_test_function_query()?;
        let content = target.buffer.content;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        for node_matched in query_matches.into_iter() {
            let function_node = node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_function_index)
                .map(|c| c.node)
                .next();

            if function_node.is_none() {
                continue;
            }

            let function_node = function_node.unwrap();

            for m in node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_name_index)
            {
                if m.node.start_position().row <= current_node_position.row
                    && m.node.end_position().row >= current_node_position.row
                {
                    return Some(Runnable {
                        name: crate_treesitter_node::node_text(m.node, content),
                        filepath: target.buffer.filepath.to_string(),
                        range: Range {
                            start: CursorPosition::from_point(function_node.start_position()),
                            end: CursorPosition::from_point(function_node.end_position()),
                        },
                        meta: RunnableMeta::default_golang(),
                    });
                }
            }
        }

        None
    }

    fn get_all_test_methods(&self, node: Node, target: &Target) -> Option<Vec<Runnable>> {
        let content = target.buffer.content;
        let query_pattern = &gotest_file_test_methods::query();
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query_pattern).ok()?;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());

        let mut parent_runnables: Vec<Runnable> = vec![];
        for node_matched in query_matches {
            let function_node = node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_function_index)
                .map(|c| c.node)
                .next();

            if function_node.is_none() {
                continue;
            }

            let function_node = function_node.unwrap();

            for m in node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_name_index)
            {
                parent_runnables.push(Runnable {
                    name: crate_treesitter_node::node_text(m.node, content),
                    filepath: target.buffer.filepath.to_string(),
                    range: Range {
                        start: CursorPosition::from_point(function_node.start_position()),
                        end: CursorPosition::from_point(function_node.end_position()),
                    },
                    meta: RunnableMeta::default_golang(),
                });
            }
        }

        if parent_runnables.is_empty() {
            None
        } else {
            Some(parent_runnables)
        }
    }
}
