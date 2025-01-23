pub trait TestFramework {
    fn detect(&self, content: &str) -> bool;
    fn find_test_methods(&self, content: &str) -> Vec<_>;
    fn generate_command(&self, content: &str) -> String;
}

pub trait FrameworkProvider {
    fn create(&self) -> Box<dyn TestFramework>;
    fn name(&self) -> &'static str;
}
