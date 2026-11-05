//! The parts of a machine that every form of `smllm graph` shows: each
//! state's tags, each transition with its event, and each guard's label
//! (CLI-8).

use smllm_core::model::{GuardDef, Machine, State, Transition, Value};

/// One transition of a machine, in the order that the text form lists them:
/// each state's events in file order, then its `always` transitions.
pub(crate) struct Edge<'m> {
    /// The index of the source state.
    pub from: usize,
    /// The event, `always` for an eventless transition.
    pub event: &'m str,
    /// The transition.
    pub transition: &'m Transition,
    /// The index of the target state, `None` when the transition stays.
    pub to: Option<usize>,
}

/// Every (event, transition) of a state, `always` as event `always`.
pub(crate) fn state_edges(s: &State) -> Vec<(&str, &Transition)> {
    let mut out: Vec<(&str, &Transition)> = Vec::new();
    for o in &s.on {
        out.extend(o.transitions.iter().map(|t| (o.event.as_str(), t)));
    }
    out.extend(s.always.iter().map(|t| ("always", t)));
    out
}

/// Every transition of `m`, with its states as indices.
pub(crate) fn edges(m: &Machine) -> Vec<Edge<'_>> {
    let index = |name: &str| m.states.iter().position(|s| s.name == name);
    let mut out = Vec::new();
    for (from, s) in m.states.iter().enumerate() {
        for (event, t) in state_edges(s) {
            let to = t.target.as_deref().and_then(index).filter(|&to| to != from);
            out.push(Edge {
                from,
                event,
                transition: t,
                to,
            });
        }
    }
    out
}

/// The tags of a state: `entry point`, `fallback`, and `final`.
pub(crate) fn tags(s: &State) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if s.entry_point {
        tags.push("entry point");
    }
    if s.fallback {
        tags.push("fallback");
    }
    if s.is_final {
        tags.push("final");
    }
    tags
}

/// A guard's label, such as `visits WORK ≥ 2` or ``command `cargo test` ``.
pub(crate) fn guard_label(g: &GuardDef) -> String {
    match g {
        GuardDef::Visits { state, at_least } => format!("visits {state} ≥ {at_least}"),
        GuardDef::Host { kind, params } => match params.get("run") {
            Some(Value::Str(s)) => format!("{kind} `{s}`"),
            Some(Value::List(l)) => format!("{kind} [{}]", l.join(" ")),
            _ => kind.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use smllm_core::SmallMap;
    use smllm_core::model::InstanceSpec;

    use super::*;

    #[test]
    fn guard_labels() {
        let v = GuardDef::Visits {
            state: "W".into(),
            at_least: 2,
        };
        assert_eq!(guard_label(&v), "visits W ≥ 2");
        let mut params = SmallMap::new();
        let host = |params: &SmallMap<Value>| GuardDef::Host {
            kind: "command".into(),
            params: params.clone(),
        };
        assert_eq!(guard_label(&host(&params)), "command");
        params.insert("run", Value::List(vec!["a".into(), "b".into()]));
        assert_eq!(guard_label(&host(&params)), "command [a b]");
    }

    #[test]
    fn a_target_that_is_the_source_or_unknown_stays() {
        let t = |target: &str| Transition {
            target: Some(target.into()),
            ..Transition::default()
        };
        let a = State {
            name: "A".into(),
            always: vec![t("A"), t("B"), t("NOPE")],
            ..State::default()
        };
        let b = State {
            name: "B".into(),
            is_final: true,
            ..State::default()
        };
        let m = Machine {
            id: "m".into(),
            description: None,
            initial: "A".into(),
            instance: InstanceSpec::default(),
            events: SmallMap::new(),
            shared: Vec::new(),
            states: vec![a, b],
        };
        let to: Vec<Option<usize>> = edges(&m).iter().map(|e| e.to).collect();
        assert_eq!(to, [None, Some(1), None]);
        assert_eq!(tags(&m.states[1]), ["final"]);
    }
}
