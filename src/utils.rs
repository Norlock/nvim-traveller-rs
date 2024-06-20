use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct NeoUtils;

impl NeoUtils {
    pub fn git_root(path: &Path) -> Option<PathBuf> {
        let dir_path = if path.is_file() {
            path.parent()?.to_string_lossy()
        } else {
            path.to_string_lossy()
        };

        let output = Command::new("git")
            .args(["-C", &dir_path, "rev-parse", "--show-toplevel"])
            .output()
            .ok()?;

        if output.status.success() {
            let out_raw = String::from_utf8_lossy(&output.stdout);
            let out = Self::strip_trailing_newline(&out_raw);

            Some(out.into())
        } else {
            None
        }
    }

    pub fn home_directory() -> PathBuf {
        std::env::var_os("HOME").unwrap().into()
    }

    fn strip_trailing_newline(input: &str) -> &str {
        input
            .strip_suffix("\r\n")
            .or(input.strip_suffix('\n'))
            .unwrap_or(input)
    }
}
