use std::collections::HashSet;

use crate::core::execution::CommandSpec;

use super::{
    enums::{Capability, Language},
    errors::FrameworkError,
    types::{CapabilityDetails, Runnable, Target},
};

pub trait Framework {
    fn detect(&self, target: &Target) -> bool;
    fn runnables(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError>;
    fn generate_command(&self, runnable: Runnable) -> CommandSpec;
    fn capabilities(&self) -> HashSet<CapabilityDetails>;
    fn search_for_capability(&self, description: &str) -> Option<CapabilityDetails>;
}

pub trait FrameworkProvider {
    fn create(&self) -> Box<dyn Framework>;
    fn name(&self) -> &'static str;
    fn language(&self) -> Language;
    fn capability(&self) -> Capability;
}
