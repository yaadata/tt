use tree_sitter::{Node, Point, Tree};

use crate::core::types::CursorPosition;

pub(crate) fn node_text(node: Node, src: &str) -> String {
    src[node.start_byte()..node.end_byte()].to_string()
}

pub(crate) fn nearest_source_file_position(node: Option<Node>) -> Option<Point> {
    match node {
        Some(node) => {
            if node.grammar_name().to_string().eq("source_file") {
                return Some(node.start_position());
            }
            nearest_source_file_position(node.parent())
        }
        _ => None,
    }
}

pub(crate) fn position_to_nearest_point(tree: &Tree, position: CursorPosition) -> Option<Node> {
    let mut walker = tree.walk();
    let child_index = walker.goto_first_child_for_point(position.to_point());
    if let Some(child_index) = child_index {
        walker.goto_descendant(child_index);
        return Some(walker.node());
    }

    None
}
