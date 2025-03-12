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

#[derive(Clone, PartialEq, Eq)]
pub enum Search {
    // Nearest - peaking the nearest eligible test
    Nearest,
    // Method - find the name of the parent test method
    Method,
    // File - find all the tests in a file
    File,
}
