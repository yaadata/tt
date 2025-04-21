#[derive(Clone, Debug)]
pub(crate) enum RunnableMeta {
    Golang {
        package: String,
        build_tags: Vec<String>,
    },
}

impl RunnableMeta {
    pub(crate) fn default_golang() -> Self {
        RunnableMeta::Golang {
            package: String::new(),
            build_tags: Vec::new(),
        }
    }
}
