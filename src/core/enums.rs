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
    Nearest,
    InMethod,
    InFile,
    InDirectory,
    InProject,
}
