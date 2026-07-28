use std::env;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::process::{self, Command};

const USAGE: &str = "\
usage:
  cargo run --package tt-repo-tools -- test
  cargo run --package tt-repo-tools -- test-dir <directory>
  cargo run --package tt-repo-tools -- test-file <file>";

#[derive(Clone, Copy)]
enum TargetKind {
    Directory,
    File,
}

fn main() {
    match run() {
        Ok(exit_code) => process::exit(exit_code),
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let mut arguments = env::args().skip(1);
    let command = arguments
        .next()
        .ok_or_else(|| "missing command".to_owned())?;

    let module = match command.as_str() {
        "test" => {
            require_no_more_arguments(arguments)?;
            None
        }
        "test-dir" => {
            let path = require_path_argument(&mut arguments, "directory")?;
            require_no_more_arguments(arguments)?;
            module_for_path(&path, TargetKind::Directory)?
        }
        "test-file" => {
            let path = require_path_argument(&mut arguments, "file")?;
            require_no_more_arguments(arguments)?;
            module_for_path(&path, TargetKind::File)?
        }
        _ => return Err(format!("unknown command: {command}")),
    };

    run_nextest(module.as_deref())
}

fn require_path_argument(
    arguments: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {name} argument"))
}

fn require_no_more_arguments(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    match arguments.next() {
        Some(argument) => Err(format!("unexpected argument: {argument}")),
        None => Ok(()),
    }
}

fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo-tools must be located at scripts/repo-tools")
}

fn module_for_path(path: &Path, kind: TargetKind) -> Result<Option<String>, String> {
    if path.is_absolute() {
        return Err(format!("{} must be repository-relative", target_name(kind)));
    }

    let candidate = project_root().join(path);
    match kind {
        TargetKind::Directory if !candidate.is_dir() => {
            return Err(format!("directory does not exist: {}", path.display()));
        }
        TargetKind::File if !candidate.is_file() => {
            return Err(format!("file does not exist: {}", path.display()));
        }
        _ => {}
    }

    let resolved = candidate
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", path.display()))?;
    let lib_root = project_root()
        .join("lib")
        .canonicalize()
        .map_err(|error| format!("failed to resolve lib/: {error}"))?;
    let relative = resolved
        .strip_prefix(&lib_root)
        .map_err(|_| format!("{} must be beneath lib/", target_name(kind)))?;

    module_from_relative_path(relative, kind)
}

fn target_name(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Directory => "directory",
        TargetKind::File => "file",
    }
}

fn module_from_relative_path(relative: &Path, kind: TargetKind) -> Result<Option<String>, String> {
    if matches!(kind, TargetKind::File) && relative.extension() != Some(OsStr::new("rs")) {
        return Err("file must have a .rs extension".to_owned());
    }

    let module_path = match kind {
        TargetKind::Directory => relative.to_path_buf(),
        TargetKind::File => relative.with_extension(""),
    };
    let mut parts = module_segments(&module_path, kind)?;

    if matches!(kind, TargetKind::File) {
        if parts.as_slice() == ["lib"] {
            parts.clear();
        } else if parts.last().is_some_and(|part| part == "mod") {
            parts.pop();
        }
    }

    Ok((!parts.is_empty()).then(|| parts.join("::")))
}

fn module_segments(path: &Path, kind: TargetKind) -> Result<Vec<String>, String> {
    path.components()
        .map(|component| match component {
            Component::Normal(segment) if is_module_segment(segment) => {
                Ok(segment.to_string_lossy().into_owned())
            }
            _ => Err(format!(
                "{} path contains a non-module segment",
                target_name(kind)
            )),
        })
        .collect()
}

fn is_module_segment(segment: &OsStr) -> bool {
    let Some(segment) = segment.to_str() else {
        return false;
    };
    let mut bytes = segment.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
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

#[cfg(test)]
mod tests {
    use super::{module_from_relative_path, TargetKind};
    use std::path::Path;

    #[test]
    fn directory_maps_to_module_prefix() {
        assert_eq!(
            module_from_relative_path(
                Path::new("framework/golang/operations"),
                TargetKind::Directory
            ),
            Ok(Some("framework::golang::operations".to_owned()))
        );
    }

    #[test]
    fn source_file_maps_to_module_prefix() {
        assert_eq!(
            module_from_relative_path(
                Path::new("framework/golang/operations/get_build_tags.rs"),
                TargetKind::File
            ),
            Ok(Some(
                "framework::golang::operations::get_build_tags".to_owned()
            ))
        );
    }

    #[test]
    fn mod_file_maps_to_parent_module() {
        assert_eq!(
            module_from_relative_path(Path::new("framework/golang/mod.rs"), TargetKind::File),
            Ok(Some("framework::golang".to_owned()))
        );
    }

    #[test]
    fn crate_root_maps_to_all_tests() {
        assert_eq!(
            module_from_relative_path(Path::new("lib.rs"), TargetKind::File),
            Ok(None)
        );
    }

    #[test]
    fn invalid_module_segment_is_rejected() {
        assert_eq!(
            module_from_relative_path(Path::new("not-a-module"), TargetKind::Directory),
            Err("directory path contains a non-module segment".to_owned())
        );
    }
}
