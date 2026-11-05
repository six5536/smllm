//! The audit files: one file of JSON lines for each session, in
//! `audit/` of the user state folder (STORE-5).

use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use agent_harness_adapter_core::fs as fs_kit;
use sha2::{Digest, Sha256};
use smllm_core::host::HostError;
use smllm_core::record::AuditEntry;

use super::{other, safe};

/// `audit/<KEY>.jsonl` under the user state folder `user`.
pub(crate) fn path(user: &Path, key: &str) -> PathBuf {
    user.join("audit").join(format!("{}.jsonl", safe(key)))
}

/// `sha256:` and the hex of the SHA-256 of `text`'s UTF-8 bytes.
pub(crate) fn sha256(text: &str) -> String {
    let mut out = String::from("sha256:");
    for b in Sha256::digest(text.as_bytes()) {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The line of `entry`: its fields, then `reply`, the hash of the reply's
/// text, and a newline.
pub(crate) fn line(entry: &AuditEntry, reply: &str) -> Result<String, HostError> {
    let mut value = serde_json::to_value(entry).map_err(other)?;
    if let Some(fields) = value.as_object_mut() {
        fields.insert("reply".to_string(), sha256(reply).into());
    }
    let mut line = serde_json::to_string(&value).map_err(other)?;
    line.push('\n');
    Ok(line)
}

/// Append `line` to the file at `path` whole. The file is opened to
/// append, so each write lands at its end, and the line is written while
/// the file is locked, so that a write that the system splits, or one on a
/// file system without atomic appends, never interleaves with another
/// process's line.
// sokf-implements: STORE-5_AC-2
pub(crate) fn append(path: &Path, line: &str) -> Result<(), HostError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(other)?;
    }
    // Windows locks only a file opened to read or write, and `append`
    // alone opens it for neither.
    let mut f = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .map_err(other)?;
    f.lock().map_err(other)?;
    let written = f.write_all(line.as_bytes()).map_err(other);
    let _ = f.unlock();
    written
}

/// The lines of the audit at `path`; none when it has no file.
pub(crate) fn read(path: &Path) -> Result<Option<Vec<String>>, HostError> {
    let text = fs_kit::read_text(path).map_err(other)?;
    Ok(text.map(|t| {
        t.lines()
            .filter(|l| !l.trim().is_empty())
            .map(ToString::to_string)
            .collect()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Eight writers, each with its own open file as each process has, append
    // lines of very different sizes, up to 256 KiB, to one file at once:
    // every line is read back whole, and none is lost.
    // sokf-tests: STORE-5_AC-2
    #[test]
    fn writers_at_the_same_moment_never_interleave_their_lines() {
        let d = crate::test_support::temp_dir("audit-race");
        let path = path(&d, "sm-race");
        std::thread::scope(|s| {
            for w in 0..8u8 {
                let path = &path;
                s.spawn(move || {
                    for i in 0..40usize {
                        let len = [10, 5_000, 70_000, 262_144][i % 4];
                        let entry = AuditEntry {
                            session: "sm-race".into(),
                            call: "fire".into(),
                            trace: vec![char::from(b'a' + w).to_string().repeat(len)],
                            ..AuditEntry::default()
                        };
                        append(path, &line(&entry, "text").unwrap()).unwrap();
                    }
                });
            }
        });
        let lines = read(&path).unwrap().unwrap();
        assert_eq!(lines.len(), 8 * 40);
        for l in &lines {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            let t = v["trace"][0].as_str().unwrap();
            assert!(t.bytes().all(|b| b == t.as_bytes()[0]), "a mixed line");
            assert_eq!(v["reply"], sha256("text"));
        }
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_hash_names_its_algorithm() {
        // FIPS 180-2's example "abc".
        assert_eq!(
            sha256("abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256(""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
