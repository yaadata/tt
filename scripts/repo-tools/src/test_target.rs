use crate::project_root;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub(crate) enum TestTarget {
    Directory(PathBuf),
    File(PathBuf),
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

    pub(crate) fn module_filter(&self) -> Result<Option<String>, String> {
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

        let resolved = candidate
            .canonicalize()
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()))?;
        let lib_root = project_root()
            .join("lib")
            .canonicalize()
            .map_err(|error| format!("failed to resolve lib/: {error}"))?;
        let relative = resolved
            .strip_prefix(&lib_root)
            .map_err(|_| format!("{} must be beneath lib/", self.name()))?;

        self.module_filter_from(relative)
    }

    fn module_filter_from(&self, relative: &Path) -> Result<Option<String>, String> {
        if matches!(self, Self::File(_)) && relative.extension() != Some(OsStr::new("rs")) {
            return Err("file must have a .rs extension".to_owned());
        }

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
    use super::{module_segments, TestTarget};
    use std::path::{Path, PathBuf};

    #[test]
    fn directory_maps_to_module_prefix() {
        assert_eq!(
            TestTarget::Directory(PathBuf::from("lib/framework/golang/operations")).module_filter(),
            Ok(Some("framework::golang::operations".to_owned()))
        );
    }

    #[test]
    fn source_file_maps_to_module_prefix() {
        assert_eq!(
            TestTarget::File(PathBuf::from(
                "lib/framework/golang/operations/get_build_tags.rs"
            ))
            .module_filter(),
            Ok(Some(
                "framework::golang::operations::get_build_tags".to_owned()
            ))
        );
    }

    #[test]
    fn mod_file_maps_to_parent_module() {
        assert_eq!(
            TestTarget::File(PathBuf::from("lib/framework/golang/mod.rs")).module_filter(),
            Ok(Some("framework::golang".to_owned()))
        );
    }

    #[test]
    fn crate_root_maps_to_all_tests() {
        assert_eq!(
            TestTarget::File(PathBuf::from("lib/lib.rs")).module_filter(),
            Ok(None)
        );
    }

    #[test]
    fn invalid_module_segment_is_rejected() {
        assert_eq!(
            module_segments(Path::new("not-a-module"), "directory"),
            Err("directory path contains a non-module segment".to_owned())
        );
    }
}
