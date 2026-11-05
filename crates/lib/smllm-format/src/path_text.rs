//! The text of a path that smllm prints: the same on every platform.

use std::path::{Component, Path};

/// `path` as text, with `/` between its components on every platform, so
/// that the CLI and the format print a path alike everywhere. A component
/// keeps its own text, a `\` in a Unix file name included, and a Windows
/// prefix such as `C:` stays as it is.
// sokf-implements: CLI-15_AC-1
pub fn path_text(path: &Path) -> String {
    let mut out = String::new();
    // Whether the last component was a name, which the next one follows
    // after a `/`; a prefix or the root needs none.
    let mut after_name = false;
    for c in path.components() {
        let part = match c {
            Component::Prefix(p) => {
                out.push_str(&p.as_os_str().to_string_lossy());
                after_name = false;
                continue;
            }
            Component::RootDir => {
                out.push('/');
                after_name = false;
                continue;
            }
            Component::CurDir => ".".into(),
            Component::ParentDir => "..".into(),
            Component::Normal(s) => s.to_string_lossy(),
        };
        if after_name {
            out.push('/');
        }
        out.push_str(&part);
        after_name = true;
    }
    out
}

/// `path` as the system's own text, for a path that smllm passes on to be
/// read again, such as a file to read, a folder that a command runs in, or a
/// session's stored folder, and never prints.
#[expect(
    clippy::disallowed_methods,
    reason = "the system's own text of a path, which smllm passes on and never prints"
)]
pub fn system_path_text(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: CLI-15_AC-1
    #[test]
    fn components_are_joined_with_a_slash() {
        let p: std::path::PathBuf = ["out", "all-dev.png"].iter().collect();
        assert_eq!(path_text(&p), "out/all-dev.png");
        assert_eq!(path_text(Path::new("./a/../b")), "./a/../b");
        assert_eq!(path_text(Path::new("")), "");
    }

    #[cfg(unix)]
    #[test]
    fn a_unix_path_keeps_its_root_and_its_backslashes() {
        assert_eq!(path_text(Path::new("/tmp/a\\b.md")), "/tmp/a\\b.md");
    }

    #[cfg(windows)]
    #[test]
    fn a_windows_path_keeps_its_prefix() {
        assert_eq!(
            path_text(Path::new(r"C:\Users\me\x.md")),
            "C:/Users/me/x.md"
        );
        assert_eq!(path_text(Path::new(r"C:x.md")), "C:x.md");
    }
}
