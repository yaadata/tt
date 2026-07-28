use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum Task {
    /// Run all library tests.
    Test,
    /// Run tests beneath a Rust source directory.
    TestDir { directory: PathBuf },
    /// Run tests declared in a Rust source file.
    TestFile { file: PathBuf },
}
