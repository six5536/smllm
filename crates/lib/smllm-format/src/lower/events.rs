//! `meta.instance` and `meta.events` (FMT-4, FMT-10, MCP-1, AGT-2).

use smllm_core::model::{EventDef, InstanceSpec, ParamKind, ParamSpec};
use smllm_core::{BUILTINS, SmallMap};

use crate::bundle;
use crate::finding::Severity;
use crate::lower::actions::{Files, check_fences};
use crate::lower::checker::{Checker, YPath};
use crate::source::{EventSource, InstanceMeta, MachineMeta, ParamsSchema, PropSchema};
use crate::ypath;

/// Names the `enter` event uses itself (FMT-2).
const ENTER_PARAMS: [&str; 2] = ["stateMachine", "state"];

/// The compiled pattern, or an error finding when it is not a valid regex.
pub(crate) fn valid_pattern(
    c: &mut Checker<'_>,
    path: &YPath<'_>,
    pattern: &str,
) -> Option<regex::Regex> {
    match regex::Regex::new(pattern) {
        Ok(re) => Some(re),
        Err(e) => {
            let first = e.to_string().lines().last().unwrap_or_default().to_string();
            c.error(
                path,
                format!("invalid pattern {pattern}: {first}"),
                None,
                "FMT-4",
            );
            None
        }
    }
}

/// `meta.instance` → [`InstanceSpec`] (FMT-4).
// sokf-implements: FMT-4_AC-1
pub(crate) fn lower_instance(c: &mut Checker<'_>, src: Option<&InstanceMeta>) -> InstanceSpec {
    let mut spec = InstanceSpec::default();
    let Some(src) = src else { return spec };
    if let Some(n) = &src.kind {
        check_fences(c, &ypath!["meta", "instance", "kind"], n);
        spec.kind = n.clone();
    }
    if let Some(r) = &src.r#ref {
        if let Some(p) = &r.param {
            spec.ref_param = p.clone();
        }
        if let Some(d) = &r.description {
            check_fences(c, &ypath!["meta", "instance", "ref", "description"], d);
        }
        spec.ref_description = r.description.clone();
        if let Some(pat) = &r.pattern
            && valid_pattern(c, &ypath!["meta", "instance", "ref", "pattern"], pat).is_some()
        {
            spec.ref_pattern = Some(pat.clone());
        }
    }
    if ENTER_PARAMS.contains(&spec.ref_param.as_str()) {
        c.error(
            &ypath!["meta", "instance", "ref", "param"],
            format!(
                "ref param {} has the name of one of enter's own params",
                spec.ref_param
            ),
            Some("choose another name, such as issueId"),
            "FMT-2",
        );
    }
    if !is_name(&spec.ref_param) {
        c.error(
            &ypath!["meta", "instance", "ref", "param"],
            format!(
                "ref param {:?} must contain only letters, digits, _, and -",
                spec.ref_param
            ),
            None,
            "FMT-4",
        );
    }
    spec
}

pub(crate) fn is_name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// `meta.events` → event defs (FMT-4); built-ins may set only guidance (AGT-2).
// sokf-implements: FMT-4_AC-2
// sokf-implements: AGT-2_AC-3
pub(crate) fn lower_events(
    c: &mut Checker<'_>,
    files: &Files<'_>,
    meta: &MachineMeta,
) -> SmallMap<EventDef> {
    let mut out = SmallMap::new();
    let Some(events) = &meta.events else {
        return out;
    };
    let meta_p = YPath::ROOT.key("meta");
    let events_p = meta_p.key("events");
    for (name, ev) in events.iter() {
        let path = events_p.key(name);
        if !is_name(name) {
            c.error(
                &path,
                format!("event name {name:?} must contain only letters, digits, _, and -"),
                None,
                "FMT-4",
            );
        }
        if let Some(d) = &ev.description {
            check_fences(c, &path, d);
        }
        if BUILTINS.contains(&name) && ev.params.is_some() {
            c.error(
                &path,
                format!("built-in event {name} can change only its description, not its params"),
                None,
                "AGT-2",
            );
        }
        let host_only = ev.source == Some(EventSource::Host);
        if BUILTINS.contains(&name) && ev.source.is_some() {
            c.error(
                &path,
                format!("built-in event {name} can change only its description, not its source"),
                None,
                "AGT-2",
            );
        }
        let params = match &ev.params {
            Some(schema) if !BUILTINS.contains(&name) => lower_params(c, files, &path, schema),
            _ => Vec::new(),
        };
        out.insert(
            name,
            EventDef {
                description: ev.description.clone(),
                params,
                host_only: host_only && !BUILTINS.contains(&name),
            },
        );
    }
    host_note(c, &out);
    out
}

/// A machine with host-only events needs a host to drive it (FMT-4).
// sokf-implements: FMT-4_AC-5
fn host_note(c: &mut Checker<'_>, events: &SmallMap<EventDef>) {
    let hosts: Vec<&str> = events
        .iter()
        .filter(|(_, e)| e.host_only)
        .map(|(n, _)| n)
        .collect();
    if hosts.is_empty() {
        return;
    }
    c.add(
        Severity::Info,
        &ypath!["meta", "events"],
        format!(
            "only the host fires the events {}, so the machine needs a host to drive it",
            hosts.join(", ")
        ),
        Some("a harness can't start an instance; a host fires the events through fire_as_host"),
        "FMT-4",
    );
}

fn lower_params(
    c: &mut Checker<'_>,
    files: &Files<'_>,
    path: &YPath<'_>,
    schema: &ParamsSchema,
) -> Vec<ParamSpec> {
    let p = path.key("params");
    let props = p.key("properties");
    if schema.kind != "object" {
        c.error(
            &p,
            format!("params type must be object, not {}", schema.kind),
            None,
            "FMT-4",
        );
    }
    let mut out = Vec::new();
    for (name, prop) in schema.properties.iter() {
        let pp = props.key(name);
        if !is_name(name) {
            c.error(
                &pp,
                format!("param name {name:?} must contain only letters, digits, _, and -"),
                None,
                "FMT-4",
            );
        }
        if ENTER_PARAMS.contains(&name) {
            c.warning(
                &pp,
                format!("param {name} has the name of one of enter's own params"),
                None,
                "FMT-2",
            );
        }
        let enum_values = prop.enum_values.clone().unwrap_or_default();
        if prop.enum_values.as_ref().is_some_and(Vec::is_empty) {
            c.error(
                &pp,
                "enum is empty, so it allows no value".to_string(),
                None,
                "FMT-4",
            );
        }
        if let Some(d) = &prop.description {
            check_fences(c, &pp, d);
        }
        let pattern = prop.pattern.as_ref().filter(|pat| {
            let Some(re) = valid_pattern(c, &pp.key("pattern"), pat) else {
                return false;
            };
            for v in enum_values.iter().filter(|v| !re.is_match(v)) {
                c.warning(
                    &pp,
                    format!(
                        "enum value {v:?} does not match the pattern, so the param can never take it"
                    ),
                    None,
                    "FMT-4",
                );
            }
            true
        });
        let (kind, json_schema) = lower_kind(c, files, &pp, name, prop);
        out.push(ParamSpec {
            name: name.to_string(),
            description: prop.description.clone(),
            required: schema.required.iter().any(|r| r == name),
            enum_values,
            pattern: pattern.cloned(),
            kind,
            schema: json_schema,
        });
    }
    for r in &schema.required {
        if !schema.properties.contains_key(r) {
            c.error(
                &p.key("required"),
                format!("required param {r} is not in properties"),
                None,
                "FMT-4",
            );
        }
    }
    out
}

/// A param's `type`, and a `json` param's schema, bundled as compact JSON
/// text (FMT-10).
// sokf-implements: FMT-10_AC-1
// sokf-implements: FMT-10_AC-7
// sokf-implements: FMT-10_AC-8
fn lower_kind(
    c: &mut Checker<'_>,
    files: &Files<'_>,
    pp: &YPath<'_>,
    name: &str,
    prop: &PropSchema,
) -> (ParamKind, Option<String>) {
    let kind = prop.kind.as_deref().unwrap_or("string");
    if !matches!(kind, "string" | "json") {
        c.error(
            pp,
            format!("param {name} has type {kind}, but a param is a string or JSON"),
            Some("use type: string, with enum or pattern to limit its values, or type: json for a JSON value"),
            "MCP-1",
        );
    }
    if kind != "json" {
        if prop.schema.is_some() {
            c.error(
                &pp.key("schema"),
                format!("param {name} is a string, so it takes no schema"),
                Some("set type: json for a param that takes a JSON value with a schema"),
                "FMT-10",
            );
        }
        return (ParamKind::String, None);
    }
    for (key, given) in [
        ("enum", prop.enum_values.is_some()),
        ("pattern", prop.pattern.is_some()),
    ] {
        if given {
            c.error(
                &pp.key(key),
                format!("param {name} takes a JSON value, so it takes no {key}"),
                Some("give the constraint in the param's schema"),
                "FMT-10",
            );
        }
    }
    let Some(src) = &prop.schema else {
        return (ParamKind::Json, None);
    };
    let at = pp.key("schema");
    match bundle::bundle(files, src) {
        Ok(v) => {
            let text = v.to_string();
            check_fences(c, &at, &text);
            (ParamKind::Json, Some(text))
        }
        Err(p) => {
            c.error(&at, p.message, p.hint, "FMT-10");
            (ParamKind::Json, None)
        }
    }
}

/// A `setRef` transition makes the ref param required on its event (FMT-4).
// sokf-implements: FMT-4_AC-4
pub(crate) fn require_ref(
    c: &mut Checker<'_>,
    events: &mut SmallMap<EventDef>,
    event: &str,
    spec: &InstanceSpec,
) {
    let mut def = events.remove(event).unwrap_or_default();
    match def.params.iter_mut().find(|p| p.name == spec.ref_param) {
        Some(p) => {
            if p.kind == ParamKind::Json {
                c.error(
                    &ypath![
                        "meta",
                        "events",
                        event,
                        "params",
                        "properties",
                        spec.ref_param
                    ],
                    format!(
                        "{event} sets the ref, so its param {} must be a string",
                        spec.ref_param
                    ),
                    Some("remove type: json, or give the ref another param"),
                    "FMT-10",
                );
            }
            p.required = true;
            // The instance's ref pattern always applies to the ref it sets.
            if let Some(pat) = &spec.ref_pattern {
                if p.pattern.as_ref().is_some_and(|own| own != pat) {
                    c.warning(
                        &ypath!["meta", "events", event, "params", "properties", spec.ref_param],
                        format!(
                            "{event} sets the ref, so meta.instance.ref.pattern {pat} applies instead of this pattern"
                        ),
                        Some("remove this pattern, or change meta.instance.ref.pattern"),
                        "FMT-4",
                    );
                }
                p.pattern = Some(pat.clone());
            }
            if p.description.is_none() {
                p.description = spec.ref_description.clone();
            }
        }
        None => def.params.insert(
            0,
            ParamSpec {
                name: spec.ref_param.clone(),
                description: spec.ref_description.clone(),
                required: true,
                pattern: spec.ref_pattern.clone(),
                ..ParamSpec::default()
            },
        ),
    }
    events.insert(event, def);
}
