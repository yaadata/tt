#[derive(PartialEq, Eq)]
pub enum Langauge {
    Golang,
    Rust,
    Python,
}

#[derive(PartialEq, Eq)]
pub enum ToolCategory {
    Debugger,
    TestRunner,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Search {
    // Nearest - peaking the nearest eligible test
    Nearest,
    // Method - find the name of the parent test method
    Method,
    // File - find all the tests in a file
    File,
}

#[derive(PartialEq, Eq, Hash)]
pub struct SearchDescriptor {
    pub(crate) search: Search,
    pub(crate) description: String,
}
