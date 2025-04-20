use std::collections::HashSet;

use super::enums::Search;
use super::{
    enums::{Language, SearchDescriptor, ToolCategory},
    errors::FrameworkError,
    types::{Runnable, Target},
};

pub trait Framework {
    fn detect(&self, target: &Target) -> bool;
    fn runnables(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError>;
    fn generate_command(&self, runnable: Runnable) -> String;
    fn search_strategies(&self) -> &HashSet<SearchDescriptor>;
    fn search_strategy_by_description(&self, description: &str) -> Option<Search>;
}

pub trait FrameworkProvider {
    fn create(&self) -> Box<dyn Framework>;
    fn name(&self) -> &'static str;
    fn language(&self) -> Language;
    fn category(&self) -> ToolCategory;
}
