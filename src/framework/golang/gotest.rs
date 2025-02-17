use crate::core::errors::FrameworkError;
use crate::core::metadata::RunnableMeta;
use crate::core::types::Runnable;
use crate::core::types::Target;
use crate::core::{
    enums::{Langauge, ToolCategory},
    traits::{Framework, FrameworkProvider},
};
use crate::treesitter::node as crate_treesitter_node;
use tree_sitter::Language;
use tree_sitter::Node;
use tree_sitter::Query;
use tree_sitter::QueryCursor;

use super::common;
use super::common::utils::get_build_tags;

pub struct GotestProvider;

static FILE_SUFFIX: &str = "_test.go";

impl FrameworkProvider for GotestProvider {
    fn create(&self) -> Box<dyn Framework> {
        Box::new(GotestProvider::new())
    }

    fn name(&self) -> &'static str {
        "gotest"
    }

    fn language(&self) -> Langauge {
        crate::core::enums::Langauge::Golang
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::TestRunner
    }
}

impl GotestProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl Framework for GotestProvider {
    fn detect(&self, target: &Target) -> bool {
        if target.category != self.category() {
            return false;
        }

        target.buffer.filepath.to_string().ends_with(FILE_SUFFIX)
    }

    fn generate_command(&self, runnable: Runnable) -> String {
        "go test some".to_string()
    }

    fn runnable(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError> {
        let tree = common::utils::parse_tree(target.buffer.content);
        match tree {
            Ok(tree) => {
                let build_tags = get_build_tags(tree.root_node(), target.buffer.content);
                let runnable = top_level_test_function(
                    crate_treesitter_node::position_to_nearest_point(
                        &tree,
                        target.buffer.position.clone(),
                    ),
                    target,
                );
                let mut runnables: Vec<Runnable> = vec![];
                if let Some(runnable) = runnable {
                    let mut r = runnable;
                    r.meta.set_position(target.buffer.position.clone());
                    runnables.push(r.clone());
                    if let Some(build_tags) = build_tags {
                        for t in build_tags.into_iter() {
                            let mut r = r.clone();
                            r.meta.set_build_tags(t);
                        }
                    }
                }

                Err(FrameworkError::NotFoundError(
                    "no test found at the current position".to_string(),
                ))
            }
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn top_level_test_function(node: Option<Node>, target: &Target) -> Option<Runnable> {
    match node {
        Some(node) => {
            let current_node_position = node.start_position();
            let query_pattern = r#"
            [[((function_declaration 
                    name: (identifier) @test_name
                    parameters: (parameter_list
                        (parameter_declaration
                                 name: (identifier)
                                 type: (pointer_type
                                     (qualified_type
                                      package: (package_identifier) @_param_package
                                      name: (type_identifier) @_param_name))))
                     ) @testfunc
                  (#contains? @test_name "Test"))]]
            "#;
            let content = target.buffer.content;
            let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query_pattern);
            if let Result::Ok(q) = query {
                let capture_index = q
                    .capture_index_for_name("test_name")
                    .expect("could not find index position of `test_name` capture");
                let mut cursor = QueryCursor::new();
                let query_matches = cursor.matches(&q, node, content.as_bytes());
                for node_matched in query_matches {
                    for m in node_matched
                        .captures
                        .iter()
                        .filter(|c| c.index == capture_index)
                    {
                        if m.node.start_position().row <= current_node_position.row
                            && m.node.end_position().row >= current_node_position.row
                        {
                            let name = crate_treesitter_node::node_text(m.node, &content);
                            return Some(Runnable {
                                name,
                                filepath: "".to_string(),
                                meta: RunnableMeta::default_golang(),
                            });
                        }
                    }
                }
            }

            None
        }
        None => None,
    }
}
