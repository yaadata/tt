use super::types::CursorPosition;

pub(crate) struct MetaGolang {}

pub(crate) enum DetectedTestMeta {
    Golang {
        build_tags: Vec<String>,
        point: CursorPosition,
    },
}
