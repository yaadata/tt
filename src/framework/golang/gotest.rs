use crate::core::errors::FrameworkError;
use crate::core::metadata::DetectedTestMeta;
use crate::core::types::Target;
use crate::core::types::TestMethod;
use crate::core::{
    enums::{Langauge, ToolCategory},
    traits::{Framework, FrameworkProvider},
};
use crate::treesitter::node as crate_treesitter_node;
use tree_sitter::Language;
use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Query;
use tree_sitter::QueryCursor;

use super::common;

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

    fn generate_command(&self, content: &str) -> String {
        "go test some".to_string()
    }

    fn find_test_methods(&self, target: &Target) -> Result<Vec<TestMethod>, FrameworkError> {
        let tree = common::utils::parse_tree(target.buffer.content);
        match tree {
            Ok(tree) => {
                let method = top_level_test_function(
                    crate_treesitter_node::position_to_nearest_point(
                        &tree,
                        target.buffer.position.clone(),
                    ),
                    target.buffer.content.to_string().clone(),
                );
                if method.is_some() {
                    return Ok(vec![method.unwrap()]);
                }

                Err(FrameworkError::NotFoundError(
                    "no test found at the current position".to_string(),
                ))
            }
            Err(e) => Err(e),
        }
    }
}

fn get_parent(node: Option<Node>) -> Option<Node> {
    match node {
        Some(node) => {
            if node.is_extra() {
                return get_parent(Some(node));
            }

            Some(node)
        }
        _ => None,
    }
}

pub(crate) fn top_level_test_function(node: Option<Node>, content: String) -> Option<TestMethod> {
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
                            return Some(TestMethod {
                                name,
                                filepath: "".to_string(),
                                meta: DetectedTestMeta::default_golang(),
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
