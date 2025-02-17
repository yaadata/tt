use tree_sitter::Point;

use super::{enums, metadata::RunnableMeta};

#[derive(Clone)]
pub struct Runnable {
    pub name: String,
    pub filepath: String,
    pub meta: RunnableMeta,
}

pub struct Buffer<'a> {
    pub content: &'a str,
    pub filepath: String,
    pub position: CursorPosition,
}

impl Buffer<'_> {
    pub fn new(content: &str, filepath: String, position: CursorPosition) -> Buffer {
        Buffer {
            content,
            filepath,
            position,
        }
    }
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

impl Target<'_> {
    pub fn new(category: enums::ToolCategory, buffer: Buffer) -> Target {
        Target { category, buffer }
    }
}
