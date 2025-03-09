use super::{
    enums::{Langauge, ToolCategory},
    errors::FrameworkError,
    types::{Runnable, Target},
};

pub trait Framework {
    fn detect(&self, target: &Target) -> bool;
    fn runnables(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError>;
    fn generate_command(&self, runnable: Runnable) -> String;
}

pub trait FrameworkProvider {
    fn create(&self) -> Box<dyn Framework>;
    fn name(&self) -> &'static str;
    fn language(&self) -> Langauge;
    fn category(&self) -> ToolCategory;
}
