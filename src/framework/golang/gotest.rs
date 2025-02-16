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
                let method = detect_test(
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

fn detect_test(node: Option<Node>, content: String) -> Option<TestMethod> {
    match node {
        Some(current_node) => {
            if current_node.is_named()
                && current_node
                    .grammar_name()
                    .to_string()
                    .eq("function_declaration")
            {
                if current_node.grammar_name().to_string().eq("source_file") {
                    return None;
                }

                if let Some(name) =
                    iterate_children_for_function_name(current_node, content.clone())
                {
                    return Some(TestMethod {
                        name,
                        filepath: "".to_string(),
                        meta: DetectedTestMeta::default_golang(),
                    });
                }
            }

            if let Some(parent) = get_parent(Some(current_node)) {
                if parent.is_named() && parent.grammar_name().to_string().eq("function_declaration")
                {
                    if let Some(name) = iterate_children_for_function_name(parent, content.clone())
                    {
                        return Some(TestMethod {
                            name,
                            filepath: "".to_string(),
                            meta: DetectedTestMeta::default_golang(),
                        });
                    }
                }
            }

            detect_test(current_node.parent(), content)
        }
        None => None,
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

fn iterate_children_for_function_name(node: Node, content: String) -> Option<String> {
    let mut qursor = node.clone().walk();
    let children = node.named_children(&mut qursor);
    for child in children {
        if child.is_named() && child.is_named() && child.grammar_name().eq(r#"identifier"#) {
            return Some(crate_treesitter_node::node_text(child, &content));
        }
    }

    None
}

pub(crate) fn detect_test_with_query(node: Option<Node>, content: String) -> Option<TestMethod> {
    match node {
        Some(node) => {
            let current_node_position = node.start_position();
            let source_file_position =
                crate_treesitter_node::nearest_source_file_position(Some(node))?;

            let mut cursor = QueryCursor::new();
            let query_pattern = r#"
            [[((function_declaration name: (identifier) @test_name
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
                let matches = cursor.matches(&q, node, content.as_bytes());
                for node_matched in matches {
                    for m in node_matched.captures {
                        let matched_node_position = m.node.start_position();
                        if matched_node_position.row >= current_node_position.row
                            && matched_node_position.row < source_file_position.row
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
