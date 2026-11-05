//! A `json` param's schema, read and bundled when the machine loads
//! (FMT-10): the loader reads the schema and each file that its relative
//! `$ref`s name through the machine's source, checks each against the
//! meta-schema of its draft, and bundles every file as a resource with an
//! `$id` under `$defs`, so that the schema needs no file at run time.

use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value};

use crate::bundle::meta::{self, Draft};
use crate::lower::Files;
use crate::source::SchemaSrc;

/// Why a schema can't be bundled: the message, which names the schema or
/// the ref, and a hint.
pub(crate) struct Problem {
    pub message: String,
    pub hint: Option<&'static str>,
}

impl Problem {
    fn new(message: String) -> Self {
        Self {
            message,
            hint: None,
        }
    }
}

/// The scheme of an id that the loader assigns to a file without an
/// absolute `$id` of its own. It holds the file's name, never its path.
const ASSIGNED: &str = "urn:smllm:schema:";

/// One file of a schema, or the inline schema.
struct Doc {
    /// The file, in its normal form; `None` for an inline schema.
    path: Option<PathBuf>,
    /// The schema as a finding names it.
    shown: String,
    value: Value,
    draft: Draft,
}

impl Doc {
    /// The folder that its relative refs start from.
    fn folder<'a>(&'a self, files: &'a Files<'_>) -> &'a Path {
        match &self.path {
            Some(p) => p.parent().unwrap_or(Path::new("")),
            None => files.dir,
        }
    }

    /// The absolute `$id` that the file declares, without an empty fragment.
    fn declared_id(&self) -> Option<&str> {
        let id = self.value.get("$id")?.as_str()?;
        let id = id.strip_suffix('#').unwrap_or(id);
        has_scheme(id).then_some(id)
    }
}

/// A `$ref` to another file of the schema.
struct Link {
    /// The doc that holds the ref.
    doc: usize,
    /// The ref's place in that doc, as a JSON pointer.
    at: String,
    /// The doc that it names.
    target: usize,
    /// Its fragment, `None` without a `#`.
    fragment: Option<String>,
}

/// Read and bundle `src`, whose relative paths start from the machine's
/// folder. Without a ref to another file, the schema is as written;
/// otherwise it is a 2020-12 schema whose `$ref` names the first resource
/// under its `$defs`.
// sokf-implements: FMT-10_AC-2
// sokf-implements: FMT-10_AC-3
// sokf-implements: FMT-10_AC-4
pub(crate) fn bundle(files: &Files<'_>, src: &SchemaSrc) -> Result<Value, Problem> {
    let root = match src {
        SchemaSrc::File(rel) => {
            let path = normal(&files.dir.join(rel));
            let shown = format!("schema {rel}");
            let value = read_json(files, &path, &shown)?
                .ok_or_else(|| Problem::new(format!("the schema file {rel} does not exist")))?;
            Doc {
                draft: draft(&value, Draft::D2020, &shown)?,
                path: Some(path),
                shown,
                value,
            }
        }
        SchemaSrc::Inline(m) => {
            let value = Value::Object(m.clone());
            let shown = "the inline schema".to_string();
            Doc {
                draft: draft(&value, Draft::D2020, &shown)?,
                path: None,
                shown,
                value,
            }
        }
    };
    check(&root)?;
    let mut docs = vec![root];
    let mut links = Vec::new();
    let mut i = 0;
    while i < docs.len() {
        let mut found = Vec::new();
        refs(&docs[i].value, "", &mut found);
        for (at, r) in found {
            let (uri, fragment) = match r.split_once('#') {
                Some((u, f)) => (u, Some(f.to_string())),
                None => (r.as_str(), None),
            };
            if uri.is_empty() {
                continue;
            }
            let target = if has_scheme(uri) {
                by_id(&docs, uri, &r, &docs[i].shown)?
            } else {
                let path = normal(&docs[i].folder(files).join(uri));
                match docs.iter().position(|d| d.path.as_ref() == Some(&path)) {
                    Some(j) => j,
                    None => {
                        let shown = format!("schema {uri}");
                        let value = read_json(files, &path, &shown)?.ok_or_else(|| {
                            Problem::new(format!(
                                "ref {r} in {} names a file that does not exist",
                                docs[i].shown
                            ))
                        })?;
                        let doc = Doc {
                            draft: draft(&value, docs[i].draft, &shown)?,
                            path: Some(path),
                            shown,
                            value,
                        };
                        check(&doc)?;
                        docs.push(doc);
                        docs.len() - 1
                    }
                }
            };
            links.push(Link {
                doc: i,
                at,
                target,
                fragment,
            });
        }
        i += 1;
    }
    if links.is_empty() {
        return Ok(docs.swap_remove(0).value);
    }
    Ok(wrap(docs, &links))
}

/// The bundle: each doc a resource under `$defs`, with its `$id` and its
/// `$schema`, and each ref to another file rewritten to that file's `$id`.
fn wrap(mut docs: Vec<Doc>, links: &[Link]) -> Value {
    let mut keys: Vec<String> = Vec::new();
    for d in &docs {
        let name = match &d.path {
            Some(p) => p
                .file_name()
                .map_or("schema".to_string(), |n| n.to_string_lossy().into_owned()),
            None => "inline".to_string(),
        };
        let mut key = name.clone();
        let mut n = 1;
        while keys.contains(&key) {
            n += 1;
            key = format!("{name}-{n}");
        }
        keys.push(key);
    }
    let ids: Vec<String> = docs
        .iter()
        .zip(&keys)
        .map(|(d, k)| match d.declared_id() {
            Some(id) => id.to_string(),
            None => format!("{ASSIGNED}{k}"),
        })
        .collect();
    for l in links {
        let mut r = ids[l.target].clone();
        if let Some(f) = &l.fragment {
            r.push('#');
            r.push_str(f);
        }
        if let Some(slot) = docs[l.doc].value.pointer_mut(&l.at) {
            *slot = Value::String(r);
        }
    }
    let mut defs = Map::new();
    for ((d, key), id) in docs.drain(..).zip(keys).zip(&ids) {
        let mut res = Map::new();
        res.insert(
            "$schema".to_string(),
            Value::String(d.draft.uri().to_string()),
        );
        res.insert("$id".to_string(), Value::String(id.clone()));
        match d.value {
            Value::Object(m) => {
                for (k, v) in m {
                    if k != "$schema" && k != "$id" {
                        res.insert(k, v);
                    }
                }
            }
            // A boolean schema can't hold an `$id`.
            other => {
                res.insert("allOf".to_string(), Value::Array(vec![other]));
            }
        }
        defs.insert(key, Value::Object(res));
    }
    let mut out = Map::new();
    out.insert(
        "$schema".to_string(),
        Value::String(Draft::D2020.uri().to_string()),
    );
    out.insert("$ref".to_string(), Value::String(ids[0].clone()));
    out.insert("$defs".to_string(), Value::Object(defs));
    Value::Object(out)
}

/// The doc that declares the absolute `$id` `uri`.
// sokf-implements: FMT-10_AC-6
fn by_id(docs: &[Doc], uri: &str, r: &str, shown: &str) -> Result<usize, Problem> {
    let uri = uri.strip_suffix('#').unwrap_or(uri);
    docs.iter()
        .position(|d| d.declared_id() == Some(uri))
        .ok_or_else(|| Problem {
            message: format!("ref {r} in {shown} resolves to no bundled file"),
            hint: Some(
                "smllm doesn't fetch a schema from the network: write the ref as a path relative to the schema that holds it",
            ),
        })
}

/// The JSON of the file at `path`, or `None` when the file doesn't exist.
fn read_json(files: &Files<'_>, path: &Path, shown: &str) -> Result<Option<Value>, Problem> {
    let text = match files.read(path) {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(None),
        Err(e) => return Err(Problem::new(format!("{shown} can't be read: {e}"))),
    };
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| Problem::new(format!("{shown} is not JSON: {e}")))
}

/// The draft that `value` declares in `$schema`, or `default`.
// sokf-implements: FMT-10_AC-5
fn draft(value: &Value, default: Draft, shown: &str) -> Result<Draft, Problem> {
    match value.get("$schema") {
        None => Ok(default),
        Some(Value::String(uri)) => Draft::of_uri(uri).ok_or_else(|| Problem {
            message: format!(
                "{shown} declares the draft {uri}, but smllm reads only draft-07 and 2020-12"
            ),
            hint: Some(
                "set $schema to http://json-schema.org/draft-07/schema# or https://json-schema.org/draft/2020-12/schema",
            ),
        }),
        Some(other) => Err(Problem::new(format!(
            "{shown} has a $schema of {other}, but $schema must be a string"
        ))),
    }
}

/// `doc` against the meta-schema of its draft.
fn check(doc: &Doc) -> Result<(), Problem> {
    meta::check(doc.draft, &doc.value).map_err(|e| {
        Problem::new(format!(
            "{} is not a valid {} schema: {e}",
            doc.shown,
            doc.draft.name()
        ))
    })
}

/// Each `$ref` string in the schema `v`, with its JSON pointer from `at`.
/// The values of `enum`, `const`, `default`, and `examples` are data, not
/// schemas, so a `$ref` key inside them is not a ref.
fn refs(v: &Value, at: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let here = format!("{at}/{}", k.replace('~', "~0").replace('/', "~1"));
                match (k.as_str(), x) {
                    ("$ref", Value::String(r)) => out.push((here, r.clone())),
                    ("enum" | "const" | "default" | "examples", _) => {}
                    (
                        "properties" | "patternProperties" | "$defs" | "definitions"
                        | "dependentSchemas" | "dependencies",
                        Value::Object(schemas),
                    ) => {
                        for (name, s) in schemas {
                            let p =
                                format!("{here}/{}", name.replace('~', "~0").replace('/', "~1"));
                            refs(s, &p, out);
                        }
                    }
                    _ => refs(x, &here, out),
                }
            }
        }
        Value::Array(items) => {
            for (i, x) in items.iter().enumerate() {
                refs(x, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

/// Whether `uri` starts with a scheme, such as `https:` or `urn:`. A path
/// such as `C:\x.json` has a one-letter scheme, so a scheme needs two.
fn has_scheme(uri: &str) -> bool {
    let Some((scheme, _)) = uri.split_once(':') else {
        return false;
    };
    scheme.len() > 1
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// `path` without `.` parts, and with each `..` part taking away the part
/// before it, so that two refs to one file find one doc.
fn normal(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(out.components().next_back(), Some(Component::Normal(_))) =>
            {
                out.pop();
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemes_and_paths() {
        assert!(has_scheme("https://example.com/x.json"));
        assert!(has_scheme("urn:smllm:schema:x"));
        assert!(!has_scheme("x.json"));
        assert!(!has_scheme("C:\\x.json"));
        assert!(!has_scheme("../a:b.json"));
        assert_eq!(normal(Path::new("a/./b/../c.json")), Path::new("a/c.json"));
        assert_eq!(normal(Path::new("../../x.json")), Path::new("../../x.json"));
    }

    #[test]
    fn refs_are_found_only_in_schemas() {
        let v = serde_json::json!({
            "$ref": "a.json",
            "properties": { "enum": { "$ref": "b.json#/x" }, "a/b": { "$ref": "c.json" } },
            "enum": [{ "$ref": "not-a-ref" }],
            "allOf": [{ "$ref": "d.json" }, { "$ref": 3 }]
        });
        let mut out = Vec::new();
        refs(&v, "", &mut out);
        let refs: Vec<(&str, &str)> = out.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        assert_eq!(
            refs,
            [
                ("/$ref", "a.json"),
                ("/properties/enum/$ref", "b.json#/x"),
                ("/properties/a~1b/$ref", "c.json"),
                ("/allOf/0/$ref", "d.json"),
            ]
        );
    }
}
