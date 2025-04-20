use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Hash)]
pub enum Language {
    Unsupported,
    #[serde(rename = "go")]
    Golang,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "python")]
    Python,
}

impl Language {
    pub fn aliases(&self) -> Vec<&str> {
        match self {
            Language::Golang => vec!["go", "golang"],
            Language::Rust => vec!["rust", "rs"],
            Language::Python => vec!["python", "py"],
            _ => vec![],
        }
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "go" | "golang" => Ok(Language::Golang),
            "rust" | "rs" => Ok(Language::Rust),
            "python" | "py" => Ok(Language::Python),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub enum Capability {
    #[serde(rename = "debug")]
    Debugger,
    #[serde(rename = "test")]
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
