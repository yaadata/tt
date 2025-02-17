use crate::core::{metadata::RunnableMeta, types::CursorPosition};

impl RunnableMeta {
    pub(in crate::framework::golang) fn set_position(&mut self, cursor: CursorPosition) {
        match self {
            RunnableMeta::Golang { point, .. } => {
                point.row = cursor.row;
                point.col = cursor.col;
            }
        }
    }

    pub(in crate::framework::golang) fn set_build_tags(&mut self, tags: String) {
        match self {
            RunnableMeta::Golang { build_tags, .. } => {
                build_tags.push(tags);
            }
        }
    }

    pub(in crate::framework::golang) fn get_meta(&self) -> Option<Meta> {
        match self {
            RunnableMeta::Golang {
                package,
                build_tags,
                point,
            } => Some(Meta {
                package: package.clone(),
                build_tags: build_tags.clone(),
                point: point.clone(),
            }),
            _ => None,
        }
    }
}

pub(in crate::framework::golang) struct Meta {
    pub(in crate::framework::golang) package: String,
    pub(in crate::framework::golang) build_tags: Vec<String>,
    pub(in crate::framework::golang) point: CursorPosition,
}
