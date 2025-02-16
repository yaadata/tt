use tree_sitter::Point;

use super::{enums, metadata::DetectedTestMeta};

pub struct TestMethod {
    pub name: String,
    pub filepath: String,
    pub meta: DetectedTestMeta,
}

pub struct Buffer<'a> {
    pub content: &'a str,
    pub filepath: String,
    pub position: CursorPosition,
}

#[derive(Default, Clone)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}
pub(crate) const fn cursor_position(row: usize, col: usize) -> CursorPosition {
    CursorPosition { row, col }
}

impl CursorPosition {
    pub(crate) fn to_point(&self) -> Point {
        Point::new(self.row, self.col)
    }
}

pub struct Target<'a> {
    pub category: enums::ToolCategory,
    pub buffer: Buffer<'a>,
}
