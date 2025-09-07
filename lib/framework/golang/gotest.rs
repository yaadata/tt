use std::collections::HashSet;

use crate::core::enums::Language as crate_language;
use crate::core::errors::FrameworkError;
use crate::core::types::Command;
use crate::core::types::Runnable;
use crate::core::types::Target;
use crate::core::{
    enums::Capability,
    traits::{Framework, FrameworkProvider},
    types::CapabilityDetails,
};
use crate::framework::golang::operations::detect_gotest_file;
use crate::framework::golang::operations::gotest_get_file_tests;
use crate::framework::golang::operations::gotest_get_subtests;
use crate::framework::golang::operations::gotest_get_test;
use crate::framework::golang::operations::parse_tree;

pub struct GotestProvider {
    search_capabilities: HashSet<CapabilityDetails>,
}

static FILE_SUFFIX: &str = "_test.go";

impl GotestProvider {
    pub fn new() -> Self {
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
        Self {
            search_capabilities: res,
        }
    }
}

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
        let tree = parse_tree::op::execute(target.buffer.content);
        if tree.is_err() {
            return false;
        }
        let tree = tree.unwrap();
        if !detect_gotest_file::op::execute(tree.root_node(), target.buffer.content) {
            return false;
        }
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
                let parent_runnables = gotest_get_file_tests::op::execute(walker_node, target);
                if parent_runnables.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found no tests in this file".to_string(),
                    ));
                }
                let parent_runnables = parent_runnables.unwrap();
                let mut res: Vec<Runnable> = vec![];
                for parent in parent_runnables.into_iter() {
                    let subtests =
                        gotest_get_subtests::op::execute(walker_node, parent.to_owned(), target);
                    if let Some(sub) = subtests {
                        res.extend(sub);
                    } else {
                        res.push(parent);
                    }
                }
                Ok(res)
            }
            crate::core::enums::Search::Method => {
                let res = gotest_get_test::op::execute(walker_node, target);
                if res.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }
                Ok(vec![res.unwrap()])
            }
            crate::core::enums::Search::Nearest => {
                let parent_runnable = gotest_get_test::op::execute(walker_node, target);
                if parent_runnable.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }

                let parent_runnable = parent_runnable.unwrap();
                let subtests = gotest_get_subtests::op::execute(
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

    fn capabilities(&self) -> HashSet<CapabilityDetails> {
        self.search_capabilities.clone()
    }

    fn search_for_capability(&self, description: &str) -> Option<CapabilityDetails> {
        let capabilities = self.capabilities();
        capabilities
            .iter()
            .find(|&s| s.description == description)
            .cloned()
    }
}
