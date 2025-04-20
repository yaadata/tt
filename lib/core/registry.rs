use std::collections::HashMap;

use super::{
    enums::{Language, ToolCategory},
    traits::{Framework, FrameworkProvider},
};

pub struct FrameworkRegistry {
    providers: HashMap<String, Box<dyn FrameworkProvider>>,
}

impl FrameworkRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn FrameworkProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get_framework(&self, name: &str) -> Option<Box<dyn Framework>> {
        self.providers.get(name).map(|p| p.create())
    }

    pub fn get_provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn get_frameworks_by_category_and_language(
        &self,
        category: ToolCategory,
        lang: Language,
    ) -> Vec<Box<dyn Framework>> {
        self.providers
            .values()
            .filter(|p| return p.category() == category && p.language() == lang)
            .map(|p| p.create())
            .collect()
    }
}
