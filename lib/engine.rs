/*
* Engine
* The purpose of engine is to orchestrate calls and actions
*/

use crate::core::registry::FrameworkRegistry;
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
}
