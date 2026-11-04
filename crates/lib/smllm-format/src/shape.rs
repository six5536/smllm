//! Structural pre-pass: walk the YAML as a generic tree against the format's
//! shape and report EVERY unknown key, wrong type and unsupported XState
//! feature, each with its YAML path and line (FMT-1, FMT-6). serde stops at
//! the first problem; this pass does not. A key whose value is null (`key:`
//! or `key: ~`) counts as absent, as in the JSON Schema; [`normalise`]
//! removes it before serde reads the document.

use serde_json::{Map, Value};

use crate::lower::{Checker, YPath as P};
use crate::source::{
    CommandParams, EventMeta, InstanceMeta, MachineFile, MachineMeta, ParamsSchema, PromptParams,
    PropSchema, RefMeta, SharedActionSrc, StateMeta, StateNode, TransitionSrc, VisitsParams,
};

/// A source type's keys, in the order of its JSON Schema. A test checks each
/// table against the schema that the serde types derive, so this walk and the
/// parser can never disagree on a key, and no load builds the schema.
struct Keys {
    allowed: &'static [&'static str],
    required: &'static [&'static str],
}

/// The [`Keys`] of a source type.
trait SourceKeys {
    const KEYS: Keys;
}

macro_rules! source_keys {
    ($($t:ty => [$($a:literal),*], [$($r:literal),*];)*) => {$(
        impl SourceKeys for $t {
            const KEYS: Keys = Keys { allowed: &[$($a),*], required: &[$($r),*] };
        }
    )*};
}

source_keys! {
    CommandParams => ["run", "timeoutSecs", "cwd"], ["run"];
    EventMeta => ["description", "params"], [];
    InstanceMeta => ["kind", "ref"], [];
    MachineFile => ["id", "description", "initial", "meta", "states"], ["id", "initial", "meta", "states"];
    MachineMeta => ["smllm", "instance", "events", "sharedActions"], ["smllm"];
    ParamsSchema => ["type", "properties", "required"], ["type"];
    PromptParams => ["text", "file"], [];
    PropSchema => ["type", "description", "enum", "pattern"], ["type"];
    RefMeta => ["param", "description", "pattern"], [];
    SharedActionSrc => ["states", "position", "entry", "exit"], ["states", "position"];
    StateMeta => ["entryPoint", "fallback", "paramDescriptions"], [];
    StateNode => ["description", "type", "meta", "entry", "exit", "on", "always"], [];
    TransitionSrc => ["target", "guard", "actions", "reenter", "description"], [];
    VisitsParams => ["state", "atLeast"], ["state", "atLeast"];
}

/// The [`Keys`] of a source type.
macro_rules! keys {
    ($t:ty) => {
        &<$t as SourceKeys>::KEYS
    };
}

/// XState keys smllm v1 does not support: an error with a v1 hint.
// sokf-implements: FMT-1_AC-2
const UNSUPPORTED: [&str; 11] = [
    "states", "initial", "invoke", "after", "context", "output", "tags", "history", "onDone",
    "types", "version",
];

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "nothing",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "a list",
        Value::Object(_) => "a map",
    }
}

struct Walk<'c, 'a> {
    c: &'c mut Checker<'a>,
}

impl Walk<'_, '_> {
    fn err(&mut self, path: &P<'_>, msg: String, hint: Option<&str>, rule: &'static str) {
        self.c.error(path, msg, hint, rule);
    }

    fn wrong(&mut self, path: &P<'_>, want: &str, got: &Value) {
        self.err(
            path,
            format!("expected {want}, found {}", kind(got)),
            None,
            "FMT-1",
        );
    }

    /// A map with `allowed` keys; `required` must be present. Returns it for
    /// the caller to walk its values.
    fn map<'v, S: AsRef<str>>(
        &mut self,
        path: &P<'_>,
        v: &'v Value,
        allowed: &[S],
        required: &[S],
    ) -> Option<&'v Map<String, Value>> {
        let Value::Object(m) = v else {
            self.wrong(path, "a map", v);
            return None;
        };
        for k in m.keys() {
            if allowed.iter().any(|a| a.as_ref() == k) {
                continue;
            }
            if UNSUPPORTED.contains(&k.as_str()) {
                self.err(
                    &path.key(k),
                    format!("`{k}` is not supported"),
                    Some("smllm supports only part of XState: flat atomic and final states, without delays, invocations, or context"),
                    "FMT-1",
                );
            } else {
                self.err(
                    &path.key(k),
                    format!(
                        "unknown key `{k}` (expected one of: {})",
                        allowed
                            .iter()
                            .map(AsRef::as_ref)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    None,
                    "FMT-1",
                );
            }
        }
        for r in required {
            let r = r.as_ref();
            if m.get(r).is_none_or(Value::is_null) {
                self.err(path, format!("missing key `{r}`"), None, "FMT-1");
            }
        }
        Some(m)
    }

    fn string(&mut self, path: &P<'_>, v: Option<&Value>) {
        if let Some(v) = present(v)
            && !v.is_string()
        {
            self.wrong(path, "a string", v);
        }
    }

    fn strings(&mut self, path: &P<'_>, v: Option<&Value>) {
        match present(v) {
            None => {}
            Some(Value::Array(items)) => {
                for (i, it) in items.iter().enumerate() {
                    self.string(&path.index(i), Some(it));
                }
            }
            Some(v) => self.wrong(path, "a list of strings", v),
        }
    }

    fn boolean(&mut self, path: &P<'_>, v: Option<&Value>) {
        if let Some(v) = present(v)
            && !v.is_boolean()
        {
            self.wrong(path, "true or false", v);
        }
    }

    fn uint(&mut self, path: &P<'_>, v: Option<&Value>) {
        if let Some(v) = present(v)
            && !v.is_u64()
        {
            self.wrong(path, "a whole number", v);
        }
    }

    fn one_of(&mut self, path: &P<'_>, v: Option<&Value>, values: &[&str], rule: &'static str) {
        match present(v) {
            None => {}
            Some(Value::String(s)) if values.contains(&s.as_str()) => {}
            Some(Value::String(s)) => self.err(
                path,
                format!("`{s}` is not one of: {}", values.join(", ")),
                None,
                rule,
            ),
            Some(v) => self.wrong(path, "a string", v),
        }
    }

    fn each<'b>(&mut self, path: &'b P<'b>, v: Option<&'b Value>) -> Vec<(P<'b>, &'b Value)> {
        match present(v) {
            None => Vec::new(),
            Some(Value::Object(m)) => m.iter().map(|(k, v)| (path.key(k), v)).collect(),
            Some(v) => {
                self.wrong(path, "a map", v);
                Vec::new()
            }
        }
    }

    fn machine(&mut self, v: &Value) {
        let root = P::ROOT;
        let k = keys!(MachineFile);
        let Some(m) = self.map(&root, v, k.allowed, k.required) else {
            return;
        };
        for k in ["id", "description", "initial"] {
            self.string(&root.key(k), m.get(k));
        }
        if let Some(meta) = present(m.get("meta")) {
            self.meta(meta);
        }
        for (p, s) in self.each(&root.key("states"), m.get("states")) {
            self.state(&p, s);
        }
    }

    fn meta(&mut self, v: &Value) {
        let p = P::ROOT.key("meta");
        let k = keys!(MachineMeta);
        let Some(m) = self.map(&p, v, k.allowed, k.required) else {
            return;
        };
        self.uint(&p.key("smllm"), m.get("smllm"));
        if let Some(i) = present(m.get("instance")) {
            let ip = p.key("instance");
            let k = keys!(InstanceMeta);
            if let Some(im) = self.map(&ip, i, k.allowed, k.required) {
                self.string(&ip.key("kind"), im.get("kind"));
                if let Some(r) = present(im.get("ref")) {
                    let rp = ip.key("ref");
                    let k = keys!(RefMeta);
                    if let Some(rm) = self.map(&rp, r, k.allowed, k.required) {
                        for k in ["param", "description", "pattern"] {
                            self.string(&rp.key(k), rm.get(k));
                        }
                    }
                }
            }
        }
        for (ep, e) in self.each(&p.key("events"), m.get("events")) {
            let k = keys!(EventMeta);
            if let Some(em) = self.map(&ep, e, k.allowed, k.required) {
                self.string(&ep.key("description"), em.get("description"));
                if let Some(ps) = present(em.get("params")) {
                    self.params(&ep.key("params"), ps);
                }
            }
        }
        match present(m.get("sharedActions")) {
            None => {}
            Some(Value::Array(items)) => {
                for (i, it) in items.iter().enumerate() {
                    let sa = p.key("sharedActions");
                    let sp = sa.index(i);
                    let k = keys!(SharedActionSrc);
                    if let Some(sm) = self.map(&sp, it, k.allowed, k.required) {
                        self.strings(&sp.key("states"), sm.get("states"));
                        self.one_of(
                            &sp.key("position"),
                            sm.get("position"),
                            &["before", "after"],
                            "AGT-6",
                        );
                        for k in ["entry", "exit"] {
                            self.actions(&sp.key(k), sm.get(k));
                        }
                    }
                }
            }
            Some(v) => self.wrong(&p.key("sharedActions"), "a list", v),
        }
    }

    fn params(&mut self, p: &P<'_>, v: &Value) {
        let k = keys!(ParamsSchema);
        let Some(m) = self.map(p, v, k.allowed, k.required) else {
            return;
        };
        self.string(&p.key("type"), m.get("type"));
        self.strings(&p.key("required"), m.get("required"));
        for (pp, prop) in self.each(&p.key("properties"), m.get("properties")) {
            let k = keys!(PropSchema);
            if let Some(pm) = self.map(&pp, prop, k.allowed, k.required) {
                for k in ["type", "description", "pattern"] {
                    self.string(&pp.key(k), pm.get(k));
                }
                self.strings(&pp.key("enum"), pm.get("enum"));
            }
        }
    }

    fn state(&mut self, p: &P<'_>, v: &Value) {
        let k = keys!(StateNode);
        let Some(m) = self.map(p, v, k.allowed, k.required) else {
            return;
        };
        self.string(&p.key("description"), m.get("description"));
        match m.get("type") {
            Some(Value::String(t)) if matches!(t.as_str(), "parallel" | "history" | "compound") => {
                self.err(
                    &p.key("type"),
                    format!("`type: {t}` is not supported"),
                    Some("smllm supports only flat atomic and final states"),
                    "FMT-1",
                )
            }
            other => self.one_of(&p.key("type"), other, &["atomic", "final"], "FMT-2"),
        }
        if let Some(meta) = present(m.get("meta")) {
            let mp = p.key("meta");
            let k = keys!(StateMeta);
            if let Some(mm) = self.map(&mp, meta, k.allowed, k.required) {
                self.boolean(&mp.key("entryPoint"), mm.get("entryPoint"));
                self.boolean(&mp.key("fallback"), mm.get("fallback"));
                for (ep, prompts) in
                    self.each(&mp.key("paramDescriptions"), mm.get("paramDescriptions"))
                {
                    for (pp, text) in self.each(&ep, Some(prompts)) {
                        self.string(&pp, Some(text));
                    }
                }
            }
        }
        for k in ["entry", "exit"] {
            self.actions(&p.key(k), m.get(k));
        }
        for (ep, ts) in self.each(&p.key("on"), m.get("on")) {
            self.transitions(&ep, ts);
        }
        if let Some(ts) = present(m.get("always")) {
            self.transitions(&p.key("always"), ts);
        }
    }

    fn transitions(&mut self, p: &P<'_>, v: &Value) {
        match v {
            Value::Array(items) if items.len() == 1 => self.transition(p, &items[0]),
            Value::Array(items) => {
                for (i, it) in items.iter().enumerate() {
                    self.transition(&p.index(i), it);
                }
            }
            v => self.transition(p, v),
        }
    }

    fn transition(&mut self, p: &P<'_>, v: &Value) {
        if v.is_string() {
            return;
        }
        let k = keys!(TransitionSrc);
        let Some(m) = self.map(p, v, k.allowed, k.required) else {
            return;
        };
        self.string(&p.key("target"), m.get("target"));
        self.string(&p.key("description"), m.get("description"));
        self.boolean(&p.key("reenter"), m.get("reenter"));
        self.actions(&p.key("actions"), m.get("actions"));
        if let Some(g) = present(m.get("guard")) {
            self.guard(&p.key("guard"), g);
        }
    }

    fn actions(&mut self, p: &P<'_>, v: Option<&Value>) {
        match present(v) {
            None => {}
            Some(Value::Array(items)) if items.len() == 1 => self.action(p, &items[0]),
            Some(Value::Array(items)) => {
                for (i, it) in items.iter().enumerate() {
                    self.action(&p.index(i), it);
                }
            }
            Some(v) => self.action(p, v),
        }
    }

    fn action(&mut self, p: &P<'_>, v: &Value) {
        if let Value::String(s) = v {
            if s != "setRef" {
                self.err(
                    p,
                    format!("unknown action `{s}`"),
                    Some("write an action as {type, params}; only setRef can be a bare name"),
                    "FMT-3",
                );
            }
            return;
        }
        let Some(m) = self.map(p, v, &["type", "params"], &["type"]) else {
            return;
        };
        let params = present(m.get("params"));
        let pp = p.key("params");
        match m.get("type").and_then(Value::as_str) {
            Some("prompt") => {
                let k = keys!(PromptParams);
                self.keys(&pp, params, k.allowed, k.required, |w, k, v| {
                    w.string(k, Some(v))
                });
                let given = ["text", "file"]
                    .iter()
                    .filter(|k| present(params.and_then(|m| m.get(**k))).is_some())
                    .count();
                if given != 1 && params.is_none_or(Value::is_object) {
                    self.err(
                        &pp,
                        "a prompt needs exactly one of: text, file".to_string(),
                        None,
                        "FMT-3",
                    );
                }
            }
            Some("command") => self.command(&pp, params),
            Some("setRef") => self.keys::<&str>(&pp, params, &[], &[], |_, _, _| {}),
            Some(t) => self.err(
                &p.key("type"),
                format!("unknown action type `{t}` (expected one of: prompt, command, setRef)"),
                None,
                "FMT-3",
            ),
            None => self.wrong(
                &p.key("type"),
                "a string",
                m.get("type").unwrap_or(&Value::Null),
            ),
        }
    }

    fn guard(&mut self, p: &P<'_>, v: &Value) {
        if v.is_string() {
            self.err(
                p,
                "a guard is a map {type, params}, not a name".to_string(),
                Some("guard types: command { run }, visits { state, atLeast }"),
                "FMT-3",
            );
            return;
        }
        let Some(m) = self.map(p, v, &["type", "params"], &["type", "params"]) else {
            return;
        };
        let params = present(m.get("params"));
        let pp = p.key("params");
        match m.get("type").and_then(Value::as_str) {
            Some("command") => self.command(&pp, params),
            Some("visits") => self.keys(
                &pp,
                params,
                keys!(VisitsParams).allowed,
                keys!(VisitsParams).required,
                |w, k, v| {
                    if k.last().is_some_and(|l| l == "atLeast") {
                        w.uint(k, Some(v));
                    } else {
                        w.string(k, Some(v));
                    }
                },
            ),
            Some(t) => self.err(
                &p.key("type"),
                format!("unknown guard type `{t}` (expected one of: command, visits)"),
                None,
                "FMT-3",
            ),
            None => self.wrong(
                &p.key("type"),
                "a string",
                m.get("type").unwrap_or(&Value::Null),
            ),
        }
    }

    fn command(&mut self, p: &P<'_>, params: Option<&Value>) {
        let k = keys!(CommandParams);
        self.keys(p, params, k.allowed, k.required, |w, k, v| match k.last() {
            Some("run") => match v {
                Value::String(_) => {}
                Value::Array(items) if items.iter().all(Value::is_string) => {}
                _ => w.err(
                    k,
                    "run must be a string (shell) or a list of strings (no shell)".to_string(),
                    None,
                    "CLI-11",
                ),
            },
            Some("timeoutSecs") => w.uint(k, Some(v)),
            _ => w.string(k, Some(v)),
        });
    }

    /// `params`: a map with `allowed` keys (absent = empty when nothing is
    /// required), each value checked by `check`.
    fn keys<S: AsRef<str>>(
        &mut self,
        p: &P<'_>,
        params: Option<&Value>,
        allowed: &[S],
        required: &[S],
        check: impl Fn(&mut Self, &P<'_>, &Value),
    ) {
        let empty = Value::Object(Map::new());
        let Some(m) = self.map(p, params.unwrap_or(&empty), allowed, required) else {
            return;
        };
        for (k, v) in m {
            if allowed.iter().any(|a| a.as_ref() == k) && !v.is_null() {
                check(self, &p.key(k), v);
            }
        }
    }
}

/// `v`, unless it is absent or null (the same to the format).
fn present(v: Option<&Value>) -> Option<&Value> {
    v.filter(|v| !v.is_null())
}

/// Check `doc`'s shape; findings go to `c`.
pub(crate) fn check(c: &mut Checker<'_>, doc: &Value) {
    Walk { c }.machine(doc);
}

/// Null values → absent keys, and `{type: setRef, params: {}}` →
/// `{type: setRef}` (serde's unit variant takes no content).
pub(crate) fn normalise(doc: &mut Value) {
    match doc {
        Value::Object(m) => {
            m.retain(|_, v| !v.is_null());
            if m.get("type").and_then(Value::as_str) == Some("setRef")
                && m.get("params")
                    .is_some_and(|p| p.as_object().is_some_and(Map::is_empty))
            {
                m.remove("params");
            }
            for v in m.values_mut() {
                normalise(v);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalise),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;
    use serde_json::Value;

    use super::*;

    /// `T`'s keys as its definition in the machine file's schema gives them,
    /// or its own schema for a type that the schema inlines.
    fn schema_keys<T: JsonSchema>() -> (Vec<String>, Vec<String>) {
        let root = schemars::schema_for!(MachineFile);
        let own = schemars::schema_for!(T);
        let v = match root
            .get("$defs")
            .and_then(|d| d.get(T::schema_name().as_ref()))
        {
            Some(v) => v,
            None if T::schema_name() == MachineFile::schema_name() => root.as_value(),
            None => own.as_value(),
        };
        let names = |key: &str| -> Vec<String> {
            match v.get(key) {
                Some(Value::Object(m)) => m.keys().cloned().collect(),
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect(),
                _ => Vec::new(),
            }
        };
        (names("properties"), names("required"))
    }

    // sokf-tests: FMT-1_AC-1
    #[test]
    fn key_tables_match_the_schema() {
        macro_rules! check {
            ($($t:ty),*) => {$(
                let k = keys!($t);
                let owned = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                assert_eq!(
                    (owned(k.allowed), owned(k.required)),
                    schema_keys::<$t>(),
                    "the key table of {} differs from its JSON Schema",
                    stringify!($t)
                );
            )*};
        }
        check!(
            CommandParams,
            EventMeta,
            InstanceMeta,
            MachineFile,
            MachineMeta,
            ParamsSchema,
            PromptParams,
            PropSchema,
            RefMeta,
            SharedActionSrc,
            StateMeta,
            StateNode,
            TransitionSrc,
            VisitsParams
        );
    }
}
