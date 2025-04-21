use crate::core::metadata::RunnableMeta;

impl RunnableMeta {
    pub(in crate::framework::golang) fn extend_build_tags(&mut self, tags: Vec<String>) {
        match self {
            RunnableMeta::Golang { build_tags, .. } => {
                build_tags.extend(tags);
            }
        }
    }

    pub(in crate::framework::golang) fn get_meta(&self) -> Option<Meta> {
        match self {
            RunnableMeta::Golang {
                package,
                build_tags,
            } => Some(Meta {
                package: package.clone(),
                build_tags: build_tags.clone(),
            }),
            _ => None,
        }
    }
}

pub(in crate::framework::golang) struct Meta {
    pub(in crate::framework::golang) package: String,
    pub(in crate::framework::golang) build_tags: Vec<String>,
}
