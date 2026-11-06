//! The two sources of a machine's files: the file system and memory. The
//! loader reads a machine file, its prompt files, and its schemas through
//! one [`InstructionSource`], the trait through which the engine reads a
//! prompt file at run time, so a host writes one implementation for both
//! (FMT-11).

use std::collections::HashMap;
use std::path::Path;

use smllm_core::host::InstructionSource;

/// The files on disk: a path is a path of the file system.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSource;

// sokf-implements: FMT-11_AC-1
impl InstructionSource for FileSource {
    fn read(&self, file: &str) -> Result<Option<String>, String> {
        match std::fs::read_to_string(Path::new(file)) {
            Ok(t) => Ok(Some(t)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_reason(&e)),
        }
    }
}

// sokf-implements: FMT-6_AC-3
/// Why a file can't be read, in words, without its path, which the finding
/// names, and without the operating system's error code.
pub fn io_reason(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::InvalidData => "it isn't UTF-8 text".to_string(),
        kind => kind.to_string(),
    }
}

/// Files held in memory, for a host that keeps its machines elsewhere, such
/// as in a database. A path is `/`-separated, and the source reads
/// `machines/../schemas/plan.json` as `schemas/plan.json`, so the loader's
/// relative paths find the files that the host inserted.
#[derive(Debug, Default, Clone)]
pub struct MemorySource {
    files: HashMap<String, String>,
}

impl MemorySource {
    /// An empty source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the file `path` with the text `text`, replacing any file at that
    /// path.
    pub fn insert(&mut self, path: &str, text: impl Into<String>) -> &mut Self {
        self.files.insert(normal(path), text.into());
        self
    }
}

// sokf-implements: FMT-11_AC-1
impl InstructionSource for MemorySource {
    fn read(&self, file: &str) -> Result<Option<String>, String> {
        Ok(self.files.get(&normal(file)).cloned())
    }
}

/// `path` with `/` between its parts, without `.` parts, and with each `..`
/// part taking away the part before it. A leading `/` stays, and a `..`
/// with no part before it stays too.
fn normal(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    let joined = parts.join("/");
    if path.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_read_in_their_normal_form() {
        assert_eq!(normal("a/./b/../c.json"), "a/c.json");
        assert_eq!(normal("a/../../b.json"), "../b.json");
        assert_eq!(normal("/x//y\\z.md"), "/x/y/z.md");
        let mut s = MemorySource::new();
        s.insert("machines/../schemas/plan.json", "{}");
        assert_eq!(s.read("schemas/plan.json").unwrap().as_deref(), Some("{}"));
        assert_eq!(
            s.read("./schemas/plan.json").unwrap().as_deref(),
            Some("{}")
        );
        assert_eq!(s.read("schemas/other.json").unwrap(), None);
    }

    #[test]
    fn the_file_system_reads_none_for_a_missing_file() {
        let dir = std::env::temp_dir().join(format!("smllm-source-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.md");
        std::fs::write(&file, "text").unwrap();
        let read = |p: &Path| FileSource.read(&p.display().to_string());
        assert_eq!(read(&file).unwrap().as_deref(), Some("text"));
        assert_eq!(read(&dir.join("missing.md")).unwrap(), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
