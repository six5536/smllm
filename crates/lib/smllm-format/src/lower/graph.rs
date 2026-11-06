//! Whole-machine checks: reachability, final states, `always` loops
//! (FMT-2, FMT-2, AGT-5).

use std::collections::BTreeMap;

use smllm_core::model::Machine;

use crate::finding::Severity;
use crate::lower::checker::Checker;
use crate::lower::machine::reachable;
use crate::ypath;

// sokf-implements: FMT-2_AC-1
// sokf-implements: FMT-2_AC-3
pub(crate) fn check(c: &mut Checker<'_>, m: &Machine) {
    if m.states.is_empty() || m.state(&m.initial).is_none() {
        return;
    }
    let seen = reachable(m);
    for s in &m.states {
        if !seen.contains(&s.name) {
            c.warning(
                &ypath!["states", s.name],
                format!("state {} is unreachable", s.name),
                Some("add a transition to it, or set meta.entryPoint: true"),
                if s.is_final { "FMT-2" } else { "AGT-5" },
            );
        }
    }
    if !m.states.iter().any(|s| s.is_final) {
        c.add(
            Severity::Info,
            &ypath!["states"],
            "no final state: instances of this machine never complete".to_string(),
            Some("if instances should complete, add a state with type: final"),
            "FMT-2",
        );
    }
    // A cycle made only of `always` edges would never rest (AGT-5).
    let always: BTreeMap<&str, Vec<&str>> = m
        .states
        .iter()
        .filter(|s| !s.always.is_empty())
        .map(|s| {
            (
                s.name.as_str(),
                // A targetless `always` stays put and fires again: a self-loop.
                s.always
                    .iter()
                    .map(|t| t.target.as_deref().unwrap_or(s.name.as_str()))
                    .collect(),
            )
        })
        .collect();
    let mut reported = Vec::new();
    for start in always.keys() {
        let mut stack = vec![(*start, vec![*start])];
        while let Some((node, path)) = stack.pop() {
            for next in always.get(node).into_iter().flatten() {
                if next == start {
                    let mut cycle = path.clone();
                    cycle.sort_unstable();
                    if !reported.contains(&cycle) {
                        c.error(
                            &ypath!["states", start, "always"],
                            format!("always transitions loop: {} → {start}", path.join(" → ")),
                            Some(
                                "smllm leaves a state with always at once, so the loop might never stop. Break it with an event",
                            ),
                            "AGT-5",
                        );
                        reported.push(cycle);
                    }
                } else if always.contains_key(next) && !path.contains(next) {
                    let mut p = path.clone();
                    p.push(next);
                    stack.push((next, p));
                }
            }
        }
    }
}
