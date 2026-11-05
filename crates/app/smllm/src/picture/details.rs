//! The details of a machine that the page shows for the selected state or
//! transition: its actions with the full text of each prompt, its guards,
//! and the events that leave each state (CLI-8).

use std::path::Path;

use serde_json::{Map, Value, json};
use smllm_core::SmallMap;
use smllm_core::model::{ActionDef, EventDef, GuardDef, Machine, Position, Prompt, State};

use crate::output;
use crate::picture::walk;

/// A param value as JSON.
fn value(v: &smllm_core::model::Value) -> Value {
    use smllm_core::model::Value as V;
    match v {
        V::Str(s) => json!(s),
        V::Int(n) => json!(n),
        V::Bool(b) => json!(b),
        V::List(l) => json!(l),
    }
}

fn params(p: &SmallMap<smllm_core::model::Value>) -> Value {
    let mut m = Map::new();
    for (k, v) in p.iter() {
        m.insert(k.to_string(), value(v));
    }
    Value::Object(m)
}

/// A prompt file's text, `None` when an implied `enter-<STATE>.md` doesn't
/// exist.
fn read(file: &str, implied: bool) -> Option<Value> {
    let shown = output::shown(Path::new(file));
    match std::fs::read_to_string(file) {
        Ok(text) => Some(json!({ "file": shown, "text": text })),
        Err(e) if implied && e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(json!({ "file": shown, "error": format!("cannot read {shown}: {e}") })),
    }
}

/// An action as JSON: a prompt with its full text, a command with its
/// params, or `setRef`. `None` for an implied prompt file that doesn't exist.
fn action(a: &ActionDef) -> Option<Value> {
    Some(match a {
        ActionDef::Prompt(Prompt::Text(t)) => json!({ "type": "prompt", "text": t }),
        ActionDef::Prompt(Prompt::File(f)) => {
            let mut v = read(f, false)?;
            v["type"] = json!("prompt");
            v
        }
        ActionDef::Prompt(Prompt::DefaultFile(f)) => {
            let mut v = read(f, true)?;
            v["type"] = json!("prompt");
            v["implied"] = json!(true);
            v
        }
        ActionDef::SetRef => json!({ "type": "setRef" }),
        ActionDef::Host { kind, params: p } => json!({ "type": kind, "params": params(p) }),
    })
}

fn actions(list: &[ActionDef], from: Option<&str>) -> Vec<Value> {
    let mut out = Vec::new();
    for a in list {
        if let Some(mut v) = action(a) {
            if let Some(from) = from {
                v["from"] = json!(from);
            }
            out.push(v);
        }
    }
    out
}

/// A state's `entry` or `exit` actions as the engine runs them: the shared
/// actions before the state's own, the state's own, then the shared actions
/// after them.
fn state_actions(m: &Machine, s: &State, entry: bool) -> Vec<Value> {
    let shared = |pos: Position| {
        let mut out = Vec::new();
        for (i, sh) in m.shared.iter().enumerate() {
            if sh.position == pos && sh.states.contains(&s.name) {
                let list = if entry { &sh.entry } else { &sh.exit };
                out.extend(actions(list, Some(&format!("sharedActions[{i}]"))));
            }
        }
        out
    };
    let mut out = shared(Position::Before);
    out.extend(actions(if entry { &s.entry } else { &s.exit }, None));
    out.extend(shared(Position::After));
    out
}

fn guard(g: &GuardDef) -> Value {
    match g {
        GuardDef::Visits { state, at_least } => json!({
            "type": "visits",
            "label": walk::guard_label(g),
            "state": state,
            "atLeast": at_least,
        }),
        GuardDef::Host { kind, params: p } => json!({
            "type": kind,
            "label": walk::guard_label(g),
            "params": params(p),
        }),
    }
}

/// The events that leave `s`, each with the guidance and the params that
/// the agent reads, and the indices of its transitions.
fn events(m: &Machine, s: &State, first: usize) -> Vec<Value> {
    let empty = EventDef::default();
    let mut out = Vec::new();
    let mut k = first;
    for on in &s.on {
        let def = m.events.get(&on.event).unwrap_or(&empty);
        let prompts = s.param_descriptions.get(&on.event);
        let ps: Vec<Value> = def
            .params
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "required": p.required,
                    "enum": p.enum_values,
                    "pattern": p.pattern,
                    "description": prompts
                        .and_then(|d| d.get(&p.name))
                        .or(p.description.as_ref()),
                })
            })
            .collect();
        let ts: Vec<usize> = (k..k + on.transitions.len()).collect();
        k += on.transitions.len();
        out.push(json!({
            "event": on.event,
            "guidance": on.description().map(ToString::to_string).or_else(|| def.description.clone()),
            "params": ps,
            "transitions": ts,
        }));
    }
    if !s.always.is_empty() {
        let ts: Vec<usize> = (k..k + s.always.len()).collect();
        out.push(json!({ "event": "always", "guidance": null, "params": [], "transitions": ts }));
    }
    out
}

// sokf-implements: CLI-8_AC-5
// sokf-implements: CLI-8_AC-6
/// The details of `m`: its states and its transitions, each transition in
/// the order of [`walk::edges`], so that index `k` is the key `m{mi}-t{k}`.
pub(crate) fn details(m: &Machine) -> Value {
    let edges = walk::edges(m);
    let mut first = 0;
    let states: Vec<Value> = m
        .states
        .iter()
        .map(|s| {
            let v = json!({
                "name": s.name,
                "description": s.description,
                "tags": walk::tags(s),
                "initial": s.name == m.initial,
                "entry": state_actions(m, s, true),
                "exit": state_actions(m, s, false),
                "events": events(m, s, first),
            });
            first += walk::state_edges(s).len();
            v
        })
        .collect();
    let transitions: Vec<Value> = edges
        .iter()
        .map(|e| {
            let t = e.transition;
            json!({
                "from": m.states[e.from].name,
                "event": e.event,
                "target": t.target,
                "guard": t.guard.as_ref().map(guard),
                "actions": actions(&t.actions, None),
                "reenter": t.reenter,
                "guidance": t.description,
            })
        })
        .collect();
    json!({
        "id": m.id,
        "description": m.description,
        "initial": m.initial,
        "states": states,
        "transitions": transitions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_files_read_or_report() {
        let dir = crate::test_support::temp_dir("details");
        let f = dir.join("p.md");
        std::fs::write(&f, "Do {this} exactly.\n").unwrap();
        let f = f.display().to_string();
        let v = action(&ActionDef::Prompt(Prompt::File(f.clone()))).unwrap();
        assert_eq!(v["text"], "Do {this} exactly.\n");
        let v = action(&ActionDef::Prompt(Prompt::DefaultFile(f))).unwrap();
        assert_eq!(v["implied"], true);
        let gone = dir.join("gone.md").display().to_string();
        assert!(action(&ActionDef::Prompt(Prompt::DefaultFile(gone.clone()))).is_none());
        let v = action(&ActionDef::Prompt(Prompt::File(gone))).unwrap();
        assert!(
            v["error"].as_str().unwrap().starts_with("cannot read "),
            "{v}"
        );
        assert_eq!(action(&ActionDef::SetRef).unwrap()["type"], "setRef");
    }

    #[test]
    fn values_keep_their_types() {
        use smllm_core::model::Value as V;
        let mut p = SmallMap::new();
        p.insert("a", V::Int(3));
        p.insert("b", V::Bool(true));
        p.insert("c", V::List(vec!["x".into()]));
        p.insert("d", V::Str("s".into()));
        assert_eq!(
            params(&p),
            json!({ "a": 3, "b": true, "c": ["x"], "d": "s" })
        );
    }
}
