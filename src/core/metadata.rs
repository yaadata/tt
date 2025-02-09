use super::types::{cursor_position, CursorPosition};

pub(crate) enum DetectedTestMeta {
    Golang {
        package: String,
        build_tags: Vec<String>,
        point: CursorPosition,
    },
}

impl DetectedTestMeta {
    pub(crate) fn default_golang() -> Self {
        DetectedTestMeta::Golang {
            package: String::new(),
            build_tags: Vec::new(),
            point: cursor_position(0, 0),
        }
    }
}
