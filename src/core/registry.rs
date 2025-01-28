use std::{any::Any, collections::HashMap};

use super::traits::{Framework, FrameworkProvider};

pub struct FrameworkRegistry {
    providers: HashMap<String, Box<dyn FrameworkProvider>>,
}

impl FrameworkRegistry {
    pub fn register(&mut self, provider: Box<dyn FrameworkProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get_framework(&self, name: &str) -> Option<Box<dyn Framework>> {
        self.providers.get(name).map(|p| p.create())
    }
}
