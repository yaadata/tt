use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Point;
use tree_sitter::Tree;
use tree_sitter::TreeCursor;

use crate::core::errors::FrameworkError;
use crate::core::types::CursorPosition;
use crate::core::types::Target;
use crate::core::types::TestMethod;
use crate::core::{
    enums::{Langauge, ToolCategory},
    traits::{Framework, FrameworkProvider},
};

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
            false;
        }

        target.buf.filepath.to_string().ends_with(FILE_SUFFIX)
    }

    fn generate_command(&self, content: &str) -> String {
        "go test some".to_string()
    }

    fn find_test_methods(&self, target: &Target) -> Result<Vec<TestMethod>, FrameworkError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Error loading Go parser");
        if let Some(tree) = parser.parse(target.clone().buffer.content, None) {
            let mut walker = tree.walk();
            walker.goto_first_child_for_point(target.clone().buffer.position.as_ts_point());
            let method = detect_test(
                Some(walker.node()),
                target.buffer.content.to_string().clone(),
            );
        }
        Err(FrameworkError::UnknownError("unknown".to_string()))
    }
}

fn detect_test(node: Option<Node>, content: String) -> Option<TestMethod> {
    match node {
        Some(n) => {
            let parent = get_parent(Some(n));
            if n.is_named() && n.grammar_name().eq("function_declaration") {
                let func_name = get_func_name(Some(n), content.clone());
                match (func_name, parent) {
                    (Some(name), Some(par)) => {
                        if name.starts_with("Test") && par.grammar_name() == "source_file" {
                            return Some(TestMethod {
                                name: name.clone(),
                                filepath: "".to_string(),
                                meta: crate::core::metadata::DetectedTestMeta::Golang {
                                    build_tags: Vec::new(),
                                    point: CursorPosition::default(),
                                },
                            });
                        }

                        return detect_test(Some(par), content);
                    }
                    _ => return detect_test(parent, content),
                }
            } else {
                let res = detect_test(parent, content);
                return res;
            }
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

            return Some(node);
        }
        _ => None,
    }
}

fn get_func_name(node: Option<Node>, content: String) -> Option<String> {
    match node {
        Some(n) => {
            if n.grammar_name().to_string().starts_with("identifier") {
                return Some(content.as_str()[n.start_byte()..n.end_byte()].to_string());
            } else {
                return get_func_name(n.next_sibling(), content);
            }
        }
        _ => None,
    }
}
