//! Collects findings for one machine file, with lines from YAML paths.

use std::path::{Path, PathBuf};

use crate::finding::{Finding, Findings, Severity};
use crate::locate::locate;

/// Findings for one file.
pub(crate) struct Checker<'a> {
    pub file: &'a Path,
    pub text: &'a str,
    pub findings: Findings,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(file: &'a Path, text: &'a str) -> Self {
        Self {
            file,
            text,
            findings: Findings::default(),
        }
    }

    pub(crate) fn add(
        &mut self,
        level: Severity,
        path: &YPath<'_>,
        message: String,
        hint: Option<&str>,
        rule: &'static str,
    ) {
        let path = path.segments();
        let segs: Vec<&str> = path.iter().map(String::as_str).collect();
        let line = locate(self.text, &segs);
        let shown = if path.is_empty() {
            None
        } else {
            let mut s = String::new();
            for seg in &path {
                if !seg.starts_with('[') && !s.is_empty() {
                    s.push('.');
                }
                s.push_str(seg);
            }
            Some(s)
        };
        self.findings.push(Finding {
            level,
            file: PathBuf::from(self.file),
            line,
            path: shown,
            message,
            hint: hint.map(ToString::to_string),
            rule,
        });
    }

    pub(crate) fn error(
        &mut self,
        path: &YPath<'_>,
        message: String,
        hint: Option<&str>,
        rule: &'static str,
    ) {
        self.add(Severity::Error, path, message, hint, rule);
    }

    pub(crate) fn warning(
        &mut self,
        path: &YPath<'_>,
        message: String,
        hint: Option<&str>,
        rule: &'static str,
    ) {
        self.add(Severity::Warning, path, message, hint, rule);
    }
}

/// A YAML path as a chain of segments on the stack, each borrowing its
/// parent, so that a walk allocates nothing for a path. Only a finding turns
/// one into strings ([`YPath::segments`]).
#[derive(Clone, Copy)]
pub(crate) struct YPath<'a> {
    up: Option<&'a YPath<'a>>,
    seg: Seg<'a>,
}

#[derive(Clone, Copy)]
enum Seg<'a> {
    Root,
    Key(&'a str),
    Index(usize),
}

impl<'a> YPath<'a> {
    /// The path of the whole document.
    pub(crate) const ROOT: YPath<'static> = YPath {
        up: None,
        seg: Seg::Root,
    };

    /// This path, then the key `k`.
    pub(crate) fn key<'b>(&'b self, k: &'b str) -> YPath<'b> {
        YPath {
            up: Some(self),
            seg: Seg::Key(k),
        }
    }

    /// This path, then the list index `i`.
    pub(crate) fn index(&self, i: usize) -> YPath<'_> {
        YPath {
            up: Some(self),
            seg: Seg::Index(i),
        }
    }

    /// The last key, if the path ends in one.
    pub(crate) fn last(&self) -> Option<&'a str> {
        match self.seg {
            Seg::Key(k) => Some(k),
            _ => None,
        }
    }

    /// The segments from the root, an index written as `[i]`.
    pub(crate) fn segments(&self) -> Vec<String> {
        let mut out = match self.up {
            Some(up) => up.segments(),
            None => Vec::new(),
        };
        match self.seg {
            Seg::Root => {}
            Seg::Key(k) => out.push(k.to_string()),
            Seg::Index(i) => out.push(format!("[{i}]")),
        }
        out
    }
}

/// The [`YPath`] of keys from the root, for use as an argument.
#[macro_export]
#[doc(hidden)]
macro_rules! ypath {
    ($($seg:expr),+ $(,)?) => {
        $crate::lower::YPath::ROOT$(.key(::core::convert::AsRef::<str>::as_ref(&$seg)))+
    };
}
