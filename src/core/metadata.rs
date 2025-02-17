use super::types::{cursor_position, CursorPosition};

#[derive(Clone)]
pub(crate) enum RunnableMeta {
    Golang {
        package: String,
        build_tags: Vec<String>,
        point: CursorPosition,
    },
}

impl RunnableMeta {
    pub(crate) fn default_golang() -> Self {
        RunnableMeta::Golang {
            package: String::new(),
            build_tags: Vec::new(),
            point: cursor_position(0, 0),
        }
    }
}
