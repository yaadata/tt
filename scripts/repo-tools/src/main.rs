mod task;
mod test_target;

use clap::Parser;
use std::path::Path;
use std::process::{self, Command};
use task::Task;
use test_target::TestTarget;

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
    let module = match task {
        Task::Test => None,
        Task::TestDir { directory } => TestTarget::Directory(directory).module_filter()?,
        Task::TestFile { file } => TestTarget::File(file).module_filter()?,
    };

    run_nextest(module.as_deref())
}

pub(crate) fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo-tools must be located at scripts/repo-tools")
}

fn run_nextest(module: Option<&str>) -> Result<i32, String> {
    let mut command = Command::new("mise");
    command
        .args([
            "exec",
            "--",
            "cargo",
            "nextest",
            "run",
            "--lib",
            "--cargo-quiet",
            "--failure-output=immediate",
            "--success-output=never",
            "--status-level=pass",
            "--no-tests=fail",
        ])
        .current_dir(project_root())
        .env("RUSTFLAGS", "-Awarnings")
        .env("RUST_BACKTRACE", "1");

    if let Some(module) = module {
        command.args(["-E", &format!("test(/^{module}::/)")]);
    }

    let status = command
        .status()
        .map_err(|error| format!("failed to run cargo nextest: {error}"))?;
    Ok(status.code().unwrap_or(1))
}
