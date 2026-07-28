mod task;
mod test_target;

use clap::Parser;
use std::path::Path;
use std::process::{self, Command};
use task::Task;
use test_target::{TestSelection, TestTarget};

#[derive(Parser)]
#[command(about = "Repository automation for testing-tools")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

fn main() {
    match run(Cli::parse().task) {
        Ok(exit_code) => process::exit(exit_code),
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(2);
        }
    }
}

fn run(task: Task) -> Result<i32, String> {
    let selection = match task {
        Task::Test => None,
        Task::TestDir { directory } => Some(TestTarget::Directory(directory).selection()?),
        Task::TestFile { file } => Some(TestTarget::File(file).selection()?),
    };

    run_nextest(selection.as_ref())
}

pub(crate) fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo-tools must be located at scripts/repo-tools")
}

fn run_nextest(selection: Option<&TestSelection>) -> Result<i32, String> {
    let mut command = Command::new("mise");
    command
        .args([
            "exec",
            "--",
            "cargo",
            "nextest",
            "run",
            "--cargo-quiet",
            "--failure-output=immediate",
            "--success-output=never",
            "--status-level=pass",
            "--no-tests=fail",
        ])
        .current_dir(project_root())
        .env("RUSTFLAGS", "-Awarnings")
        .env("RUST_BACKTRACE", "1");

    if let Some(selection) = selection {
        selection.configure_command(&mut command);
    } else {
        command.arg("--lib");
    }

    let status = command
        .status()
        .map_err(|error| format!("failed to run cargo nextest: {error}"))?;
    Ok(status.code().unwrap_or(1))
}
