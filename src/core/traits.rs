use super::{
    enums::{Langauge, ToolCategory},
    errors::FrameworkError,
    types::{Target, TestMethod},
};

pub trait Framework {
    fn detect(&self, target: &Target) -> bool;
    fn find_test_methods(&self, target: &Target) -> Result<Vec<TestMethod>, FrameworkError>;
    fn generate_command(&self, content: &str) -> String;
}

pub trait FrameworkProvider {
    fn create(&self) -> Box<dyn Framework>;
    fn name(&self) -> &'static str;
    fn language(&self) -> Langauge;
    fn category(&self) -> ToolCategory;
}
