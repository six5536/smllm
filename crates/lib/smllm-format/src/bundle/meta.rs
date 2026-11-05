//! The check of a schema against the meta-schema of its draft (FMT-10).
//!
//! The meta-schemas are copies of those that json-schema.org publishes for
//! draft-07 and 2020-12, in `meta/`. The check is a small validator that
//! knows only the keywords that those meta-schemas use, because it checks
//! nothing else: `$ref`, `$dynamicRef`, `type`, `enum`, `const`,
//! `properties`, `additionalProperties`, `patternProperties`,
//! `propertyNames`, `items`, `minItems`, `uniqueItems`, `minimum`,
//! `exclusiveMinimum`, `pattern`, `required`, `allOf`, `anyOf`, `oneOf`, and
//! `not`. It treats `format` as an annotation, as both drafts do by default.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::{Map, Value};

/// A draft of JSON Schema that smllm reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Draft {
    /// Draft-07.
    D7,
    /// Draft 2020-12.
    D2020,
}

impl Draft {
    /// The draft that a `$schema` URI names, if smllm reads it.
    pub(crate) fn of_uri(uri: &str) -> Option<Draft> {
        let uri = uri.strip_suffix('#').unwrap_or(uri);
        let uri = uri
            .strip_prefix("https://")
            .or_else(|| uri.strip_prefix("http://"))?;
        match uri {
            "json-schema.org/draft-07/schema" => Some(Draft::D7),
            "json-schema.org/draft/2020-12/schema" => Some(Draft::D2020),
            _ => None,
        }
    }

    /// The draft's `$schema` URI, as its meta-schema writes it.
    pub(crate) fn uri(self) -> &'static str {
        match self {
            Draft::D7 => "http://json-schema.org/draft-07/schema#",
            Draft::D2020 => "https://json-schema.org/draft/2020-12/schema",
        }
    }

    /// The draft's name in a finding.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Draft::D7 => "draft-07",
            Draft::D2020 => "draft 2020-12",
        }
    }

    /// The id of the draft's meta-schema, without a fragment.
    fn id(self) -> &'static str {
        self.uri().trim_end_matches('#')
    }
}

/// The meta-schemas by id, read once.
fn metas() -> &'static HashMap<String, Value> {
    static METAS: OnceLock<HashMap<String, Value>> = OnceLock::new();
    METAS.get_or_init(|| {
        const D2020: &str = "https://json-schema.org/draft/2020-12/";
        let files: [(&str, &str); 9] = [
            (
                "http://json-schema.org/draft-07/schema",
                include_str!("meta/draft-07.json"),
            ),
            ("schema", include_str!("meta/2020-12/schema.json")),
            ("meta/core", include_str!("meta/2020-12/core.json")),
            (
                "meta/applicator",
                include_str!("meta/2020-12/applicator.json"),
            ),
            (
                "meta/unevaluated",
                include_str!("meta/2020-12/unevaluated.json"),
            ),
            (
                "meta/validation",
                include_str!("meta/2020-12/validation.json"),
            ),
            (
                "meta/meta-data",
                include_str!("meta/2020-12/meta-data.json"),
            ),
            (
                "meta/format-annotation",
                include_str!("meta/2020-12/format-annotation.json"),
            ),
            ("meta/content", include_str!("meta/2020-12/content.json")),
        ];
        let mut out = HashMap::new();
        for (id, text) in files {
            let id = if id.starts_with("http") {
                id.to_string()
            } else {
                format!("{D2020}{id}")
            };
            // The files are part of the crate, and a test reads each one.
            let value = serde_json::from_str(text).unwrap_or(Value::Bool(true));
            out.insert(id, value);
        }
        out
    })
}

/// Check `schema` against the meta-schema of `draft`: `Err` holds the first
/// problem, with the place in `schema` where it is.
pub(crate) fn check(draft: Draft, schema: &Value) -> Result<(), String> {
    let root = draft.id();
    let Some(meta) = metas().get(root) else {
        return Ok(());
    };
    let v = Validator { root };
    v.eval(meta, root, schema, "")
}

struct Validator {
    /// The meta-schema that the check started from: the outermost dynamic
    /// scope, where a `$dynamicRef` lands.
    root: &'static str,
}

impl Validator {
    /// Check `inst`, at the pointer `at`, against `schema`, a part of the
    /// meta-schema `base`.
    fn eval(&self, schema: &Value, base: &str, inst: &Value, at: &str) -> Result<(), String> {
        let s = match schema {
            Value::Bool(true) => return Ok(()),
            Value::Bool(false) => return Err(fail(at, "the value isn't allowed here")),
            Value::Object(s) => s,
            _ => return Ok(()),
        };
        if let Some(Value::String(r)) = s.get("$ref") {
            let (doc, target) = resolve(base, r)?;
            self.eval(target, doc, inst, at)?;
        }
        if let Some(Value::String(r)) = s.get("$dynamicRef") {
            let anchor = r.strip_prefix('#').unwrap_or(r);
            let root = metas().get(self.root);
            let dynamic =
                root.filter(|m| m.get("$dynamicAnchor").and_then(Value::as_str) == Some(anchor));
            match dynamic {
                Some(root) => self.eval(root, self.root, inst, at)?,
                None => {
                    let (doc, target) = resolve(base, r)?;
                    self.eval(target, doc, inst, at)?;
                }
            }
        }
        if let Some(t) = s.get("type") {
            type_check(t, inst, at)?;
        }
        if let Some(Value::Array(values)) = s.get("enum")
            && !values.contains(inst)
        {
            let list: Vec<String> = values.iter().map(Value::to_string).collect();
            return Err(fail(at, &format!("expected one of: {}", list.join(", "))));
        }
        if let Some(c) = s.get("const")
            && c != inst
        {
            return Err(fail(at, &format!("expected {c}")));
        }
        for key in ["allOf", "anyOf", "oneOf"] {
            if let Some(Value::Array(all)) = s.get(key) {
                self.combine(key, all, base, inst, at)?;
            }
        }
        if let Some(not) = s.get("not")
            && self.eval(not, base, inst, at).is_ok()
        {
            return Err(fail(at, "the value isn't allowed here"));
        }
        match inst {
            Value::Object(m) => self.object(s, base, m, at)?,
            Value::Array(items) => self.array(s, base, items, at)?,
            Value::String(text) => {
                if let Some(Value::String(p)) = s.get("pattern")
                    && regex::Regex::new(p).is_ok_and(|re| !re.is_match(text))
                {
                    return Err(fail(at, &format!("expected a string that matches {p}")));
                }
            }
            Value::Number(n) => {
                let n = n.as_f64().unwrap_or(0.0);
                if let Some(min) = s.get("minimum").and_then(Value::as_f64)
                    && n < min
                {
                    return Err(fail(at, &format!("expected at least {min}")));
                }
                if let Some(min) = s.get("exclusiveMinimum").and_then(Value::as_f64)
                    && n <= min
                {
                    return Err(fail(at, &format!("expected more than {min}")));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn combine(
        &self,
        key: &str,
        all: &[Value],
        base: &str,
        inst: &Value,
        at: &str,
    ) -> Result<(), String> {
        if key == "allOf" {
            for s in all {
                self.eval(s, base, inst, at)?;
            }
            return Ok(());
        }
        let mut first = None;
        let mut passed = 0;
        for s in all {
            match self.eval(s, base, inst, at) {
                Ok(()) => passed += 1,
                Err(e) => {
                    first.get_or_insert(e);
                }
            }
        }
        match (key, passed) {
            ("anyOf", 0) | ("oneOf", 0) => {
                Err(first.unwrap_or_else(|| fail(at, "the value isn't allowed here")))
            }
            ("oneOf", n) if n > 1 => Err(fail(at, "the value matches more than one form")),
            _ => Ok(()),
        }
    }

    fn object(
        &self,
        s: &Map<String, Value>,
        base: &str,
        m: &Map<String, Value>,
        at: &str,
    ) -> Result<(), String> {
        if let Some(Value::Array(required)) = s.get("required") {
            for r in required.iter().filter_map(Value::as_str) {
                if !m.contains_key(r) {
                    return Err(fail(at, &format!("expected the key {r}")));
                }
            }
        }
        let props = s.get("properties").and_then(Value::as_object);
        let patterns = s.get("patternProperties").and_then(Value::as_object);
        for (k, v) in m {
            let here = format!("{at}/{}", escape(k));
            if let Some(names) = s.get("propertyNames") {
                self.eval(names, base, &Value::String(k.clone()), &here)?;
            }
            let mut matched = false;
            if let Some(p) = props.and_then(|p| p.get(k)) {
                matched = true;
                self.eval(p, base, v, &here)?;
            }
            for (pat, p) in patterns.into_iter().flatten() {
                if regex::Regex::new(pat).is_ok_and(|re| re.is_match(k)) {
                    matched = true;
                    self.eval(p, base, v, &here)?;
                }
            }
            if !matched && let Some(extra) = s.get("additionalProperties") {
                self.eval(extra, base, v, &here)?;
            }
        }
        Ok(())
    }

    fn array(
        &self,
        s: &Map<String, Value>,
        base: &str,
        items: &[Value],
        at: &str,
    ) -> Result<(), String> {
        if let Some(min) = s.get("minItems").and_then(Value::as_u64)
            && (items.len() as u64) < min
        {
            return Err(fail(at, &format!("expected at least {min} items")));
        }
        if s.get("uniqueItems") == Some(&Value::Bool(true)) {
            for (i, a) in items.iter().enumerate() {
                if items[..i].contains(a) {
                    return Err(fail(at, &format!("expected unique items, but {a} repeats")));
                }
            }
        }
        match s.get("items") {
            // Draft-07's tuple form: a schema for each place.
            Some(Value::Array(each)) => {
                for (i, (schema, item)) in each.iter().zip(items).enumerate() {
                    self.eval(schema, base, item, &format!("{at}/{i}"))?;
                }
            }
            Some(schema) => {
                for (i, item) in items.iter().enumerate() {
                    self.eval(schema, base, item, &format!("{at}/{i}"))?;
                }
            }
            None => {}
        }
        Ok(())
    }
}

/// The meta-schema that `r` names from `base`, and the part of it that the
/// fragment points to.
fn resolve(base: &str, r: &str) -> Result<(&'static str, &'static Value), String> {
    let (uri, fragment) = r.split_once('#').unwrap_or((r, ""));
    let id = if uri.is_empty() {
        base.to_string()
    } else if uri.contains("://") {
        uri.to_string()
    } else {
        // Relative to the folder of `base`.
        let folder = base.rsplit_once('/').map_or(base, |(f, _)| f);
        format!("{folder}/{uri}")
    };
    let Some((doc, meta)) = metas().get_key_value(id.as_str()) else {
        return Err(format!(
            "the meta-schema refers to {r}, which smllm doesn't hold"
        ));
    };
    let target = if fragment.is_empty() {
        Some(meta)
    } else {
        meta.pointer(fragment)
    };
    target
        .map(|t| (doc.as_str(), t))
        .ok_or_else(|| format!("the meta-schema refers to {r}, which smllm doesn't hold"))
}

fn type_check(t: &Value, inst: &Value, at: &str) -> Result<(), String> {
    let names: Vec<&str> = match t {
        Value::String(n) => vec![n.as_str()],
        Value::Array(ns) => ns.iter().filter_map(Value::as_str).collect(),
        _ => return Ok(()),
    };
    let is = |n: &str| match n {
        "object" => inst.is_object(),
        "array" => inst.is_array(),
        "string" => inst.is_string(),
        "boolean" => inst.is_boolean(),
        "null" => inst.is_null(),
        "number" => inst.is_number(),
        "integer" => inst.as_f64().is_some_and(|f| f.fract() == 0.0),
        _ => true,
    };
    if names.iter().any(|n| is(n)) {
        Ok(())
    } else {
        Err(fail(
            at,
            &format!("expected {}, found {}", names.join(" or "), kind(inst)),
        ))
    }
}

/// The JSON type of `v`, as JSON Schema names it.
fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// A key as a JSON pointer writes it.
fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn fail(at: &str, what: &str) -> String {
    if at.is_empty() {
        format!("at the top: {what}")
    } else {
        format!("at {at}: {what}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn every_meta_schema_parses() {
        let files = [
            include_str!("meta/draft-07.json"),
            include_str!("meta/2020-12/schema.json"),
            include_str!("meta/2020-12/core.json"),
            include_str!("meta/2020-12/applicator.json"),
            include_str!("meta/2020-12/unevaluated.json"),
            include_str!("meta/2020-12/validation.json"),
            include_str!("meta/2020-12/meta-data.json"),
            include_str!("meta/2020-12/format-annotation.json"),
            include_str!("meta/2020-12/content.json"),
        ];
        for f in files {
            assert!(serde_json::from_str::<Value>(f).unwrap().is_object());
        }
        // Each meta-schema is valid against its own draft.
        for d in [Draft::D7, Draft::D2020] {
            check(d, metas().get(d.id()).unwrap()).unwrap();
        }
    }

    #[test]
    fn drafts_are_named_by_their_uris() {
        assert_eq!(
            Draft::of_uri("http://json-schema.org/draft-07/schema#"),
            Some(Draft::D7)
        );
        assert_eq!(
            Draft::of_uri("https://json-schema.org/draft-07/schema"),
            Some(Draft::D7)
        );
        assert_eq!(
            Draft::of_uri("https://json-schema.org/draft/2020-12/schema"),
            Some(Draft::D2020)
        );
        assert_eq!(
            Draft::of_uri("http://json-schema.org/draft-04/schema#"),
            None
        );
        assert_eq!(Draft::of_uri("json-schema.org/draft-07/schema"), None);
        assert_eq!(Draft::D7.name(), "draft-07");
    }

    #[test]
    fn a_good_schema_passes_in_both_drafts() {
        let s = json!({
            "type": "object",
            "properties": {
                "steps": { "type": "array", "items": { "$ref": "#/$defs/step" }, "minItems": 1 },
                "kind": { "enum": ["a", "b"] },
                "n": { "type": ["integer", "null"], "minimum": 0 }
            },
            "required": ["steps"],
            "additionalProperties": false,
            "$defs": { "step": { "type": "string", "pattern": "^[a-z]+$" } },
            "definitions": { "old": true },
            "dependencies": { "a": ["b"] },
            "items": [true, { "const": 1 }]
        });
        check(Draft::D7, &s).unwrap();
        check(
            Draft::D2020,
            &json!({"$defs": {"x": {"type": "string"}}, "prefixItems": [true]}),
        )
        .unwrap();
        check(Draft::D2020, &json!(true)).unwrap();
    }

    #[test]
    fn a_bad_schema_fails_at_its_place() {
        let bad = json!({ "properties": { "a": { "type": "strin" } } });
        let e = check(Draft::D2020, &bad).unwrap_err();
        assert!(e.starts_with("at /properties/a/type: "), "{e}");
        let e = check(Draft::D7, &bad).unwrap_err();
        assert!(e.starts_with("at /properties/a/type: "), "{e}");
        let e = check(Draft::D2020, &json!({ "minItems": -1 })).unwrap_err();
        assert_eq!(e, "at /minItems: expected at least 0");
        let e = check(Draft::D2020, &json!({ "required": ["a", "a"] })).unwrap_err();
        assert!(e.contains("unique"), "{e}");
        let e = check(Draft::D2020, &json!(3)).unwrap_err();
        assert_eq!(e, "at the top: expected object or boolean, found number");
        let e = check(Draft::D7, &json!({ "multipleOf": 0 })).unwrap_err();
        assert_eq!(e, "at /multipleOf: expected more than 0");
        let e = check(Draft::D2020, &json!({ "$anchor": "1x" })).unwrap_err();
        assert!(e.contains("matches"), "{e}");
    }
}
