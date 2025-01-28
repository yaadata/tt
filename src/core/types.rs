use std::usize;

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

#[derive(Default)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}

impl CursorPosition {
    pub(crate) const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub(crate) fn as_ts_point(&self) -> Point {
        Point::new(self.row, self.col)
    }
}

pub struct Target<'a> {
    pub category: enums::ToolCategory,
    pub buffer: Buffer<'a>,
}
