/*
* Engine
* The purpose of engine is to orchestrate calls and actions
*/

use std::fs;
use std::str::FromStr;

use crate::core::enums::Language;
use crate::core::registry::FrameworkRegistry;
use crate::core::types::{Buffer, CapabilityDetails, CursorPosition, Target};
use crate::framework::golang::gotest::GotestProvider;
struct Engine {
    registry: FrameworkRegistry,
}

impl Engine {
    pub fn initialize() -> Self {
        let mut registry = FrameworkRegistry::new();
        let gotest_provider = Box::new(GotestProvider::new());
        registry.register(gotest_provider);
        Self { registry }
    }

    pub fn get_capabilities(&self, filepath: &str) -> Vec<CapabilityDetails> {
        // get file extension from filepath
        let extension = filepath.split('.').last().unwrap_or("");
        let lang = Language::from_str(extension).unwrap_or(Language::Unsupported);
        if lang == Language::Unsupported {
            return vec![];
        }

        let fm = self.registry.get_frameworks_by_category_and_language(
            crate::core::enums::Capability::TestRunner,
            lang,
        );

        fm.into_iter()
            .flat_map(|f| f.capabilities().clone())
            .collect()
    }

    pub fn find_runnables(
        &self,
        filepath: &str,
        description: &str,
        framework_name: &str,
        cursor: CursorPosition,
    ) {
        let framework = self.registry.get_framework(framework_name).unwrap();
        let contents = fs::read_to_string(filepath).unwrap_or("".to_string());
        let cap = framework.search_for_capability(description).unwrap();
        let target = Target::new(
            cap.capability.clone(),
            Buffer::new(contents.as_str(), filepath.to_string(), cursor),
        );

        framework.runnables(&target);
    }
}
