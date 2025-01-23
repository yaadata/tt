use std::collections::HashMap;

use super::traits::{FrameworkProvider, TestFramework};

pub struct FrameworkRegistry {
    providers: HashMap<&str, Box<dyn FrameworkProvider>>,
}

impl FrameworkRegistry {
    pub fn register(&mut self, provider: Box<dyn FrameworkProvider>) {
        self.providers.insert(provider.name(), provider)
    }

    pub fn get_framework(&self, name: &str) -> Option<Box<dyn TestFramework>> {
        self.providers.get(name).map(|p| p.create())
    }
}
