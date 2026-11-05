//! Validate a parsed machine file and lower it to the core model (CFG).

use std::collections::{HashMap, HashSet};

use smllm_core::BUILTINS;
use smllm_core::model::{ActionDef, Machine, On, Position, SharedAction, State, Transition};

use crate::finding::Severity;
use crate::lower::actions::{Files, check_fences, default_prompt, lower_actions, lower_guard};
use crate::lower::checker::{Checker, YPath};
use crate::lower::events::{is_name, lower_events, lower_instance, require_ref};
use crate::source::{MachineFile, PositionSrc, StateType, StringOr, Transitions};
use crate::ypath;

/// Lower `file`; `None` when there are errors.
// sokf-implements: FMT-1_AC-1
pub(crate) fn lower(c: &mut Checker<'_>, files: &Files<'_>, file: &MachineFile) -> Option<Machine> {
    if file.meta.smllm != 1 {
        c.error(
            &ypath!["meta", "smllm"],
            format!("unsupported smllm format version {}", file.meta.smllm),
            Some("set meta.smllm: 1, the only version that this smllm reads"),
            "FMT-1",
        );
    }
    if !is_name(&file.id) {
        c.error(
            &ypath!["id"],
            format!(
                "id {:?} must contain only letters, digits, _, and -",
                file.id
            ),
            None,
            "FMT-1",
        );
    }
    if let Some(d) = &file.description {
        check_fences(c, &ypath!["description"], d);
    }
    let names: Vec<&str> = file.states.iter().map(|(n, _)| n).collect();
    if names.is_empty() {
        c.error(
            &ypath!["states"],
            "a state machine needs at least one state".to_string(),
            None,
            "FMT-1",
        );
    }
    if !names.contains(&file.initial.as_str()) {
        c.error(
            &ypath!["initial"],
            format!("initial state {} does not exist", file.initial),
            None,
            "AGT-5",
        );
    }
    let instance = lower_instance(c, file.meta.instance.as_ref());
    let mut events = lower_events(c, files, &file.meta);

    let mut states = Vec::new();
    let mut used_events = HashSet::new();
    let mut set_ref_events = Vec::new();
    let mut fallbacks = Vec::new();
    let states_p = YPath::ROOT.key("states");
    for (name, node) in file.states.iter() {
        let sp = states_p.key(name);
        if name.is_empty() || name.chars().any(char::is_whitespace) {
            c.error(
                &sp,
                format!("state name {name:?} must not be empty or contain spaces"),
                None,
                "FMT-1",
            );
        }
        let meta = node.meta.clone().unwrap_or_default();
        let is_final = node.kind == Some(StateType::Final);
        if let Some(d) = &node.description {
            check_fences(c, &sp.key("description"), d);
        }
        if is_final && (node.on.is_some() || node.always.is_some()) {
            c.error(
                &sp,
                "a final state cannot have on or always".to_string(),
                None,
                "FMT-2",
            );
        }
        if is_final && meta.entry_point {
            c.warning(
                &sp,
                "a final state that is an entry point completes the instance as soon as it is entered".to_string(),
                Some("remove meta.entryPoint, or make the state non-final"),
                "FMT-2",
            );
        }
        if meta.fallback {
            fallbacks.push(name);
            if is_final {
                c.error(
                    &sp,
                    "the fallback state cannot be final".to_string(),
                    None,
                    "FMT-2",
                );
            }
        }
        let mut entry = lower_actions(c, files, &sp.key("entry"), node.entry.as_ref(), false);
        if !entry.iter().any(|a| matches!(a, ActionDef::Prompt(_))) {
            entry.extend(default_prompt(c, files, &sp, name));
        }
        let exit = lower_actions(c, files, &sp.key("exit"), node.exit.as_ref(), false);

        let mut on = Vec::new();
        if let Some(src) = &node.on {
            let on_p = sp.key("on");
            for (event, ts) in src.iter() {
                let ep = on_p.key(event);
                if !is_name(event) {
                    c.error(
                        &ep,
                        format!("event name {event:?} must contain only letters, digits, _, and -"),
                        None,
                        "FMT-4",
                    );
                    continue;
                }
                if BUILTINS.contains(&event) {
                    c.error(
                        &ep,
                        format!(
                            "{event} is a built-in event, so a state cannot handle it under on"
                        ),
                        None,
                        "FMT-2",
                    );
                    continue;
                }
                used_events.insert(event.to_string());
                let transitions = lower_transitions(c, files, &ep, ts, &names, true);
                if sets_ref(&transitions) {
                    set_ref_events.push(event.to_string());
                }
                on.push(On {
                    event: event.to_string(),
                    transitions,
                });
            }
        }
        let always = match &node.always {
            Some(ts) => lower_transitions(c, files, &sp.key("always"), ts, &names, false),
            None => Vec::new(),
        };
        if !always.is_empty() && !on.is_empty() {
            c.warning(
                &sp,
                "smllm leaves a state with always at once, so it never offers the state's on events".to_string(),
                None,
                "AGT-5",
            );
        }

        // Per-state prompts reword declared params, never define them (FMT-4).
        // sokf-implements: FMT-4_AC-3
        let mut param_descriptions = smllm_core::SmallMap::new();
        if let Some(pd) = &meta.param_descriptions {
            let meta_p = sp.key("meta");
            let pd_p = meta_p.key("paramDescriptions");
            for (event, prompts) in pd.iter() {
                let pp = pd_p.key(event);
                if !on.iter().any(|o| o.event == event) {
                    c.warning(
                        &pp,
                        format!("{name} has no transition on {event}, so smllm never shows these descriptions"),
                        None,
                        "FMT-4",
                    );
                }
                let mut m = smllm_core::SmallMap::new();
                // The ref param is implied only on events that set the ref.
                let event_sets_ref = on
                    .iter()
                    .any(|o| o.event == event && sets_ref(&o.transitions));
                for (param, text) in prompts.iter() {
                    let declared = events.get(event).is_some_and(|d| d.param(param).is_some())
                        || (param == instance.ref_param && event_sets_ref);
                    if !declared {
                        c.error(
                            &pp.key(param),
                            format!("{event} declares no param {param}"),
                            Some("declare params in meta.events; paramDescriptions can only reword their prompts"),
                            "FMT-4",
                        );
                    }
                    check_fences(c, &pp.key(param), text);
                    m.insert(param, text.clone());
                }
                param_descriptions.insert(event, m);
            }
        }

        states.push(State {
            name: name.to_string(),
            description: node.description.clone(),
            is_final,
            entry_point: meta.entry_point,
            fallback: meta.fallback,
            entry,
            exit,
            on,
            always,
            param_descriptions,
        });
    }
    for extra in fallbacks.iter().skip(1) {
        c.error(
            &ypath!["states", extra],
            format!(
                "only one state can be the fallback, and {} already is",
                fallbacks[0]
            ),
            None,
            "FMT-2",
        );
    }

    let mut shared = Vec::new();
    let meta_p = YPath::ROOT.key("meta");
    let shared_p = meta_p.key("sharedActions");
    for (i, sa) in file.meta.shared_actions.iter().flatten().enumerate() {
        let p = shared_p.index(i);
        for s in &sa.states {
            if !names.contains(&s.as_str()) {
                c.error(
                    &p,
                    format!("sharedActions lists unknown state {s}"),
                    None,
                    "AGT-6",
                );
            }
        }
        shared.push(SharedAction {
            states: sa.states.clone(),
            position: match sa.position {
                PositionSrc::Before => Position::Before,
                PositionSrc::After => Position::After,
            },
            entry: lower_actions(c, files, &p.key("entry"), sa.entry.as_ref(), false),
            exit: lower_actions(c, files, &p.key("exit"), sa.exit.as_ref(), false),
        });
    }

    for e in &set_ref_events {
        require_ref(c, &mut events, e, &instance);
    }
    for (name, _) in events.iter() {
        if !BUILTINS.contains(&name) && !used_events.contains(name) {
            c.warning(
                &ypath!["meta", "events", name],
                format!("event {name} is declared but no state uses it"),
                None,
                "FMT-4",
            );
        }
    }
    for (name, def) in events.iter() {
        if def.param(&instance.ref_param).is_some()
            && !def.params.is_empty()
            && !set_ref_events.iter().any(|e| e == name)
        {
            c.warning(
                &ypath!["meta", "events", name],
                format!(
                    "{name} has a param with the ref param's name {}, but never sets the ref",
                    instance.ref_param
                ),
                Some("rename the param, or add a setRef action"),
                "FMT-2",
            );
        }
    }

    let machine = Machine {
        id: file.id.clone(),
        description: file.description.clone(),
        initial: file.initial.clone(),
        instance,
        events,
        shared,
        states,
    };
    crate::lower::graph::check(c, &machine);
    if c.findings.count(Severity::Error) > 0 {
        None
    } else {
        Some(machine)
    }
}

/// Lower `on.<event>` or `always`: targets exist; the last candidate has no
/// guard (FMT-2).
// sokf-implements: FMT-2_AC-2
fn lower_transitions(
    c: &mut Checker<'_>,
    files: &Files<'_>,
    path: &YPath<'_>,
    src: &Transitions,
    states: &[&str],
    set_ref_ok: bool,
) -> Vec<Transition> {
    let many = src.0.len() > 1;
    let mut out = Vec::new();
    for (i, t) in src.0.iter().enumerate() {
        let item = path.index(i);
        let p = if many { &item } else { path };
        let lowered = match t {
            StringOr::Str(target) => Transition {
                target: Some(target.clone()),
                ..Transition::default()
            },
            StringOr::Obj(o) => Transition {
                target: o.target.clone(),
                guard: o
                    .guard
                    .as_ref()
                    .and_then(|g| lower_guard(c, files, &p.key("guard"), g, states)),
                actions: lower_actions(c, files, &p.key("actions"), o.actions.as_ref(), set_ref_ok),
                reenter: o.reenter.unwrap_or(false),
                description: o.description.clone(),
            },
        };
        if let Some(tg) = &lowered.target
            && !states.contains(&tg.as_str())
        {
            c.error(
                p,
                format!("target state {tg} does not exist"),
                None,
                "AGT-5",
            );
        }
        if let Some(d) = &lowered.description {
            check_fences(c, p, d);
        }
        if lowered.reenter && lowered.target.is_none() {
            c.warning(
                p,
                "reenter has no effect without a target, because a transition without one never re-enters"
                    .to_string(),
                Some("to re-enter it, add target: <this state>"),
                "AGT-5",
            );
        }
        out.push(lowered);
    }
    if out.last().is_some_and(|t| t.guard.is_some()) {
        let item = path.index(out.len() - 1);
        let p = if many { &item } else { path };
        c.error(
            p,
            "the last transition has a guard, so nothing happens when every guard fails"
                .to_string(),
            Some("end the list with a transition without a guard, such as one back to this state"),
            "FMT-2",
        );
    }
    out
}

/// Reachable states from the initial, entry points and fallback.
pub(crate) fn reachable(m: &Machine) -> HashSet<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut todo: Vec<String> = vec![m.initial.clone()];
    todo.extend(
        m.states
            .iter()
            .filter(|s| s.entry_point || s.fallback)
            .map(|s| s.name.clone()),
    );
    let edges: HashMap<&str, Vec<&str>> = m
        .states
        .iter()
        .map(|s| {
            let ts =
                s.on.iter()
                    .flat_map(|o| o.transitions.iter())
                    .chain(s.always.iter());
            (
                s.name.as_str(),
                ts.filter_map(|t| t.target.as_deref()).collect(),
            )
        })
        .collect();
    while let Some(s) = todo.pop() {
        if !seen.insert(s.clone()) {
            continue;
        }
        for t in edges.get(s.as_str()).into_iter().flatten() {
            todo.push((*t).to_string());
        }
    }
    seen
}
/// Whether any of `transitions` sets the ref.
fn sets_ref(transitions: &[Transition]) -> bool {
    transitions
        .iter()
        .any(|t| t.actions.contains(&ActionDef::SetRef))
}
