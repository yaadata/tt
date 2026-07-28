use crate::project_root;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub(crate) enum TestTarget {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Eq, PartialEq)]
enum CargoTarget {
    Lib,
    Bin(String),
    Test(String),
    Example(String),
    Bench(String),
}

pub(crate) struct TestSelection {
    package: String,
    target: CargoTarget,
    module: Option<String>,
}

impl TestSelection {
    pub(crate) fn configure_command(&self, command: &mut Command) {
        command.args(["--package", &self.package]);
        match &self.target {
            CargoTarget::Lib => {
                command.arg("--lib");
            }
            CargoTarget::Bin(name) => {
                command.args(["--bin", name]);
            }
            CargoTarget::Test(name) => {
                command.args(["--test", name]);
            }
            CargoTarget::Example(name) => {
                command.args(["--example", name]);
            }
            CargoTarget::Bench(name) => {
                command.args(["--bench", name]);
            }
        }

        if let Some(module) = &self.module {
            command.arg("-E").arg(format!("test(/^{module}::/)"));
        }
    }
}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    targets: Vec<CargoMetadataTarget>,
}

#[derive(Deserialize)]
struct CargoMetadataTarget {
    name: String,
    kind: Vec<String>,
    src_path: PathBuf,
}

impl TestTarget {
    fn path(&self) -> &Path {
        match self {
            Self::Directory(path) | Self::File(path) => path,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Directory(_) => "directory",
            Self::File(_) => "file",
        }
    }

    pub(crate) fn selection(&self) -> Result<TestSelection, String> {
        let path = self.path();
        if path.is_absolute() {
            return Err(format!("{} must be repository-relative", self.name()));
        }

        let candidate = project_root().join(path);
        let exists = match self {
            Self::Directory(_) => candidate.is_dir(),
            Self::File(_) => candidate.is_file(),
        };
        if !exists {
            return Err(format!(
                "{} does not exist: {}",
                self.name(),
                path.display()
            ));
        }
        if matches!(self, Self::File(_)) && candidate.extension() != Some(OsStr::new("rs")) {
            return Err("file must have a .rs extension".to_owned());
        }

        let resolved = candidate
            .canonicalize()
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()))?;
        let cargo_target = cargo_target_for(&resolved, self.name())?;
        let relative = if resolved == cargo_target.src_path {
            Path::new("")
        } else {
            resolved
                .strip_prefix(&cargo_target.source_root)
                .map_err(|_| format!("{} must be beneath a Cargo target", self.name()))?
        };

        Ok(TestSelection {
            package: cargo_target.package,
            target: cargo_target.target,
            module: self.module_filter_from(relative)?,
        })
    }

    fn module_filter_from(&self, relative: &Path) -> Result<Option<String>, String> {
        let module_path = match self {
            Self::Directory(_) => relative.to_path_buf(),
            Self::File(_) => relative.with_extension(""),
        };
        let mut parts = module_segments(&module_path, self.name())?;

        if matches!(self, Self::File(_)) {
            if parts.as_slice() == ["lib"] {
                parts.clear();
            } else if parts.last().is_some_and(|part| part == "mod") {
                parts.pop();
            }
        }

        Ok((!parts.is_empty()).then(|| parts.join("::")))
    }
}

struct CargoTargetMatch {
    package: String,
    target: CargoTarget,
    src_path: PathBuf,
    source_root: PathBuf,
    score: (bool, usize),
}

fn cargo_target_for(path: &Path, target_name: &str) -> Result<CargoTargetMatch, String> {
    let metadata = cargo_metadata()?;
    metadata
        .packages
        .into_iter()
        .flat_map(|package| {
            package.targets.into_iter().filter_map(move |target| {
                let cargo_target = CargoTarget::from_metadata(&target)?;
                let src_path = target.src_path.canonicalize().ok()?;
                let source_root = src_path.parent()?.to_path_buf();
                let is_target_root = path == src_path;
                (is_target_root || path.starts_with(&source_root)).then(|| CargoTargetMatch {
                    package: package.name.clone(),
                    target: cargo_target,
                    src_path,
                    score: (is_target_root, source_root.components().count()),
                    source_root,
                })
            })
        })
        .max_by_key(|candidate| candidate.score)
        .ok_or_else(|| format!("{target_name} must be beneath a Cargo target"))
}

impl CargoTarget {
    fn from_metadata(target: &CargoMetadataTarget) -> Option<Self> {
        if target.kind.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro"
            )
        }) {
            Some(Self::Lib)
        } else if target.kind.iter().any(|kind| kind == "bin") {
            Some(Self::Bin(target.name.clone()))
        } else if target.kind.iter().any(|kind| kind == "test") {
            Some(Self::Test(target.name.clone()))
        } else if target.kind.iter().any(|kind| kind == "example") {
            Some(Self::Example(target.name.clone()))
        } else if target.kind.iter().any(|kind| kind == "bench") {
            Some(Self::Bench(target.name.clone()))
        } else {
            None
        }
    }
}

fn cargo_metadata() -> Result<CargoMetadata, String> {
    let output = Command::new("mise")
        .args([
            "exec",
            "--",
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version=1",
        ])
        .current_dir(project_root())
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse cargo metadata: {error}"))
}

fn module_segments(path: &Path, target_name: &str) -> Result<Vec<String>, String> {
    path.components()
        .map(|component| match component {
            Component::Normal(segment) if is_module_segment(segment) => {
                Ok(segment.to_string_lossy().into_owned())
            }
            _ => Err(format!("{target_name} path contains a non-module segment")),
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

#[cfg(test)]
mod tests {
    use super::{module_segments, CargoTarget, TestTarget};
    use std::path::{Path, PathBuf};

    fn module_filter(target: TestTarget) -> Result<Option<String>, String> {
        target.selection().map(|selection| selection.module)
    }

    #[test]
    fn directory_maps_to_module_prefix() {
        assert_eq!(
            module_filter(TestTarget::Directory(PathBuf::from(
                "lib/framework/golang/operations"
            ))),
            Ok(Some("framework::golang::operations".to_owned()))
        );
    }

    #[test]
    fn source_file_maps_to_module_prefix() {
        assert_eq!(
            module_filter(TestTarget::File(PathBuf::from(
                "lib/framework/golang/operations/get_build_tags.rs"
            ))),
            Ok(Some(
                "framework::golang::operations::get_build_tags".to_owned()
            ))
        );
    }

    #[test]
    fn mod_file_maps_to_parent_module() {
        assert_eq!(
            module_filter(TestTarget::File(PathBuf::from(
                "lib/framework/golang/mod.rs"
            ))),
            Ok(Some("framework::golang".to_owned()))
        );
    }

    #[test]
    fn crate_root_maps_to_all_tests() {
        assert_eq!(
            module_filter(TestTarget::File(PathBuf::from("lib/lib.rs"))),
            Ok(None)
        );
    }

    #[test]
    fn source_file_outside_lib_maps_to_its_cargo_target() {
        let selection = TestTarget::File(PathBuf::from("scripts/repo-tools/src/test_target.rs"))
            .selection()
            .expect("repo-tools source file should resolve");

        assert_eq!(selection.package, "repo-tools");
        assert_eq!(selection.target, CargoTarget::Bin("repo-tools".to_owned()));
        assert_eq!(selection.module, Some("test_target".to_owned()));
    }

    #[test]
    fn invalid_module_segment_is_rejected() {
        assert_eq!(
            module_segments(Path::new("not-a-module"), "directory"),
            Err("directory path contains a non-module segment".to_owned())
        );
    }
}
