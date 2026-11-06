//! Offered events and their params: what the menu shows and what a call is
//! checked against (AGT-2, AGT-2, IDLE).

use smllm_json::ToJson as _;

use crate::host::Matcher;
use crate::model::{Config, EventDef, Machine, ParamKind, ParamSpec, State};
use crate::prelude::*;
use crate::record::Instance;
use crate::utils::{join, join_with};

/// Built-in event names (AGT-2).
/// `listPaused` is offered only in idle, when its list was cut short (AGT-9).
pub const BUILTINS: [&str; 6] = [
    "enter",
    "resume",
    "pause",
    "unmatched",
    "yield",
    "listPaused",
];

/// An offered event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// Event name.
    pub name: String,
    /// Guidance.
    pub description: Option<String>,
    /// Params.
    pub params: Vec<ParamView>,
}

/// A param as shown and checked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParamView {
    /// Name.
    pub name: String,
    /// Required.
    pub required: bool,
    /// Allowed values.
    pub enum_values: Vec<String>,
    /// Pattern.
    pub pattern: Option<String>,
    /// Prompt.
    pub description: Option<String>,
    /// A string, or a JSON value (FMT-10).
    pub kind: ParamKind,
    /// A JSON param's bundled JSON Schema, as compact JSON text.
    pub schema: Option<String>,
}

impl ParamView {
    fn from_spec(spec: &ParamSpec, prompt: Option<&String>) -> Self {
        Self {
            name: spec.name.clone(),
            required: spec.required,
            enum_values: spec.enum_values.clone(),
            pattern: spec.pattern.clone(),
            description: prompt.cloned().or_else(|| spec.description.clone()),
            kind: spec.kind,
            schema: spec.schema.clone(),
        }
    }
}

/// A param's value as a host received it (AGT-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamValue {
    /// A JSON string, such as a tool call sends. A JSON param takes it as
    /// that string, and never reads the JSON that it might hold.
    Str(String),
    /// Any other JSON value, as its JSON text. Only a JSON param takes one.
    Json(String),
    /// Text with no JSON type, such as the `VALUE` of `smllm fire`'s
    /// `KEY=VALUE`. A string param takes it as it is, and a JSON param reads
    /// it as JSON text.
    Text(String),
}

impl From<&str> for ParamValue {
    fn from(s: &str) -> Self {
        ParamValue::Str(s.to_string())
    }
}

impl From<String> for ParamValue {
    fn from(s: String) -> Self {
        ParamValue::Str(s)
    }
}

/// The params of an event whose params are all strings, such as idle's
/// `enter`: a JSON value that isn't a string is an error that names it.
// sokf-implements: AGT-3_AC-8
pub(crate) fn text_params(
    event: &str,
    given: &[(String, ParamValue)],
) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for (name, value) in given {
        match value {
            ParamValue::Str(s) | ParamValue::Text(s) => out.push((name.clone(), s.clone())),
            ParamValue::Json(_) => return Err(not_a_string(event, name)),
        }
    }
    Ok(out)
}

fn not_a_string(event: &str, name: &str) -> String {
    format!("{event} param {name} must be a string")
}

/// Guidance for a built-in: the machine's override, else `default` (AGT-2).
// sokf-implements: AGT-2_AC-3
fn builtin_description(machine: Option<&Machine>, name: &str, default: String) -> Option<String> {
    machine
        .and_then(|m| m.events.get(name))
        .and_then(|e| e.description.clone())
        .or(Some(default))
}

/// Events offered in a machine state: its `on` events, then the built-ins
/// (AGT-2). `state` is `None` when the saved state no longer exists (AGT-11).
// sokf-implements: AGT-2_AC-1
// sokf-implements: AGT-10_AC-8
pub(crate) fn machine_offers(
    machine: &Machine,
    state: Option<&State>,
    inst: &Instance,
) -> Vec<Offer> {
    let mut offers = Vec::new();
    let kind = &machine.instance.kind;
    let label = inst.label();
    if let Some(state) = state {
        if state.is_final {
            return offers;
        }
        let empty = EventDef::default();
        for on in &state.on {
            let def = machine.events.get(&on.event).unwrap_or(&empty);
            let prompts = state.param_descriptions.get(&on.event);
            offers.push(Offer {
                name: on.event.clone(),
                description: on
                    .description()
                    .map(ToString::to_string)
                    .or_else(|| def.description.clone()),
                params: def
                    .params
                    .iter()
                    .map(|p| ParamView::from_spec(p, prompts.and_then(|m| m.get(&p.name))))
                    .collect(),
            });
        }
        if state.fallback
            && let Some(back) = &inst.resume_state
        {
            offers.push(Offer {
                name: "resume".to_string(),
                description: builtin_description(
                    Some(machine),
                    "resume",
                    format!("Return to {back}."),
                ),
                params: Vec::new(),
            });
        }
        offers.push(Offer {
            name: "yield".to_string(),
            description: builtin_description(
                Some(machine),
                "yield",
                format!("Stop for now and stay in {}.", state.name),
            ),
            params: vec![ParamView {
                name: "note".to_string(),
                description: Some("What you are waiting for.".to_string()),
                ..ParamView::default()
            }],
        });
    }
    offers.push(Offer {
        name: "pause".to_string(),
        description: builtin_description(
            Some(machine),
            "pause",
            format!("Put {kind} {label} aside and return to idle."),
        ),
        params: Vec::new(),
    });
    let detour = match machine.fallback() {
        Some(f) if state.is_none_or(|s| s.name != f.name) => {
            format!(
                "The request fits none of these; handle it in {}, then resume.",
                f.name
            )
        }
        _ => "The request fits none of these; handle it from idle, then resume.".to_string(),
    };
    offers.push(Offer {
        name: "unmatched".to_string(),
        description: builtin_description(Some(machine), "unmatched", detour),
        params: Vec::new(),
    });
    offers
}

/// Whether an agent sees and fires `event` of `machine`: every event but one
/// that `meta.events` marks with `source: host`. Every path that shows an
/// agent its events, or takes an event from it, asks this (AGT-2, AGT-3).
pub(crate) fn for_agent(machine: &Machine, event: &str) -> bool {
    !machine.host_only(event)
}

/// [`machine_offers`] as an agent sees them: without the host-only events
/// (AGT-2).
// sokf-implements: AGT-2_AC-4
pub(crate) fn agent_offers(
    machine: &Machine,
    state: Option<&State>,
    inst: &Instance,
) -> Vec<Offer> {
    let mut offers = machine_offers(machine, state, inst);
    offers.retain(|o| for_agent(machine, &o.name));
    offers
}

// sokf-implements: ENG-6_AC-3
/// Whether the agent can start `machine`: under a harness, not a machine
/// that only a host can drive, whose `enter` the engine refuses (ENG-6).
pub(crate) fn startable(machine: &Machine, under_harness: bool) -> bool {
    !under_harness || !machine.events.iter().any(|(_, e)| e.host_only)
}

/// Events offered in idle: `enter`, `resume` when an instance is
/// interrupted, and `listPaused` when the idle list left some of `paused`
/// out (AGT-9).
// sokf-implements: AGT-10_AC-8
pub(crate) fn idle_offers(
    config: &Config,
    under_harness: bool,
    interrupted: Option<(&Machine, &Instance)>,
    paused: Option<usize>,
) -> Vec<Offer> {
    let startable: Vec<&Machine> = config
        .machines
        .iter()
        .filter(|m| startable(m, under_harness))
        .collect();
    let mut ref_params: Vec<&str> = Vec::new();
    for m in &startable {
        if !ref_params.contains(&m.instance.ref_param.as_str()) {
            ref_params.push(&m.instance.ref_param);
        }
    }
    let mut offers = vec![Offer {
        name: "enter".to_string(),
        description: Some("Start a new instance of a state machine, or continue one.".to_string()),
        params: vec![
            ParamView {
                name: "stateMachine".to_string(),
                required: true,
                enum_values: startable.iter().map(|m| m.id.clone()).collect(),
                description: Some("The state machine to enter.".to_string()),
                ..ParamView::default()
            },
            ParamView {
                name: join_with(&mut ref_params.iter().copied(), " | "),
                description: Some(
                    "The id param that the list of state machines names for the machine: an \
                     existing id or ref continues that instance, and a new ref starts one with \
                     it. Omit it to start a new instance."
                        .to_string(),
                ),
                ..ParamView::default()
            },
            ParamView {
                name: "state".to_string(),
                description: Some(
                    "An entry point to enter instead, or any state for an instance whose saved \
                     state no longer exists. Omit it to enter a new instance at the initial \
                     state, or an existing one at its saved state."
                        .to_string(),
                ),
                ..ParamView::default()
            },
        ],
    }];
    if ref_params.is_empty() {
        // No machines, so no id param to name.
        offers[0].params.remove(1);
    }
    if let Some((m, inst)) = interrupted {
        offers.push(Offer {
            name: "resume".to_string(),
            description: builtin_description(
                Some(m),
                "resume",
                format!(
                    "Return to {} {} ({}) at {}.",
                    m.instance.kind,
                    inst.label(),
                    m.id,
                    inst.state
                ),
            ),
            params: Vec::new(),
        });
    }
    if let Some(n) = paused {
        offers.push(Offer {
            name: "listPaused".to_string(),
            description: Some(format!(
                "List all {n} paused instances, most recently updated first."
            )),
            params: Vec::new(),
        });
    }
    offers
}

/// Check `given` against an offer's params (MCP-1, AGT-3). Returns the
/// params in declaration order, or the error message.
// sokf-implements: AGT-3_AC-3
pub(crate) fn check_params(
    offer: &Offer,
    given: &[(String, ParamValue)],
    matcher: &dyn Matcher,
) -> Result<Vec<(String, String)>, String> {
    for (name, _) in given {
        if !offer.params.iter().any(|p| &p.name == name) {
            return Err(if offer.params.is_empty() {
                format!("{} takes no params, but got {name}", offer.name)
            } else {
                let names = join(offer.params.iter().map(|p| p.name.as_str()));
                format!("{} has no param {name} (params: {})", offer.name, names)
            });
        }
    }
    let mut out = Vec::new();
    for p in &offer.params {
        let Some((_, value)) = given.iter().find(|(n, _)| n == &p.name) else {
            if p.required {
                return Err(format!("{} needs param {}", offer.name, p.name));
            }
            continue;
        };
        let value = match (p.kind, value) {
            (ParamKind::String, ParamValue::Str(s) | ParamValue::Text(s)) => {
                check_value(&offer.name, p, s, matcher)?;
                s.clone()
            }
            // sokf-implements: AGT-3_AC-8
            (ParamKind::String, ParamValue::Json(_)) => {
                return Err(not_a_string(&offer.name, &p.name));
            }
            // A JSON string stays a string: the host can then tell it from
            // the object that its text might spell.
            // sokf-implements: AGT-3_AC-9
            (ParamKind::Json, ParamValue::Str(s)) => smllm_json::to_string(s.as_str()),
            (ParamKind::Json, ParamValue::Json(s) | ParamValue::Text(s)) => {
                compact_json(&offer.name, &p.name, s)?
            }
        };
        if p.kind == ParamKind::Json {
            check_schema(&offer.name, p, &value, matcher)?;
        }
        out.push((p.name.clone(), value));
    }
    Ok(out)
}

// sokf-implements: AGT-3_AC-10
/// Check a JSON param's value against its schema through the host, which
/// returns one error for each part that fails, or nothing when it checks no
/// schema (AGT-3).
fn check_schema(
    event: &str,
    p: &ParamView,
    value: &str,
    matcher: &dyn Matcher,
) -> Result<(), String> {
    let Some(schema) = &p.schema else {
        return Ok(());
    };
    match matcher.check_json(schema, value) {
        // One sentence for each error, on the one line of the error: at
        // most `MAX_ERRORS`, each at most `MAX_ERROR` characters, because an
        // error can repeat the whole value.
        Some(errors) if errors.iter().any(|e| !e.trim().is_empty()) => {
            const MAX_ERRORS: usize = 10;
            const MAX_ERROR: usize = 200;
            let errors: Vec<&String> = errors.iter().filter(|e| !e.trim().is_empty()).collect();
            let mut msg = format!("{event} param {} doesn't match its schema", p.name);
            for e in errors.iter().take(MAX_ERRORS) {
                msg.push_str(". ");
                // The name smllm stays lowercase at the start of a sentence.
                let e = e.trim();
                let mut c = e.chars();
                if let Some(first) = c.next() {
                    if e.starts_with("smllm") {
                        msg.push(first);
                    } else {
                        msg.extend(first.to_uppercase());
                    }
                }
                let rest: String = c.by_ref().take(MAX_ERROR - 1).collect();
                msg.push_str(&rest);
                if c.next().is_some() {
                    msg.push('…');
                }
            }
            if errors.len() > MAX_ERRORS {
                msg.push_str(&format!(". And {} more", errors.len() - MAX_ERRORS));
            }
            Err(msg)
        }
        _ => Ok(()),
    }
}

/// A JSON param's value as compact JSON text, or the error that names the
/// param (FMT-10). The host checks the value against the schema.
// sokf-implements: AGT-3_AC-6
// sokf-implements: AGT-3_AC-7
fn compact_json(event: &str, name: &str, text: &str) -> Result<String, String> {
    match smllm_json::parse(text) {
        Ok(v) => {
            let mut out = String::new();
            v.write_json(&mut out);
            Ok(out)
        }
        Err(e) => Err(format!("{event} param {name} must be a JSON value: {e}")),
    }
}

/// Check one value against `enum` and `pattern`.
pub(crate) fn check_value(
    event: &str,
    p: &ParamView,
    value: &str,
    matcher: &dyn Matcher,
) -> Result<(), String> {
    if !p.enum_values.is_empty() && !p.enum_values.iter().any(|v| v == value) {
        return Err(format!(
            "{event} param {} must be one of: {} (got \"{value}\")",
            p.name,
            join(p.enum_values.iter().map(String::as_str))
        ));
    }
    if let Some(pat) = &p.pattern {
        match matcher.is_match(pat, value) {
            Ok(true) => {}
            Ok(false) => {
                return Err(format!(
                    "{event} param {} must match {pat} (got \"{value}\")",
                    p.name
                ));
            }
            Err(e) => {
                return Err(format!(
                    "{event} param {}: invalid pattern {pat}: {e}",
                    p.name
                ));
            }
        }
    }
    Ok(())
}
