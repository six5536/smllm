//! The events that every state of a machine offers, `yield`, `pause`, and
//! `unmatched`, and the fallback state's `resume` and its `unmatched`, which
//! returns to idle. The picture lists them once for the machine, in a note
//! beside it (CLI-8). The engine's
//! [`builtin_offers`] gives their guidance and params.

use smllm_core::model::Machine;
use smllm_core::{BuiltinsAt, Offer, builtin_offers};

/// A built-in event that the note lists.
pub(crate) struct Shared {
    /// The event, its guidance, and its params, as the agent sees them.
    pub offer: Offer,
    /// Whether only a host fires the event.
    pub host_only: bool,
}

/// The fallback state and the built-in events that it offers apart from
/// every other state.
pub(crate) struct Fallback {
    /// The fallback state.
    pub state: String,
    /// `resume`, which the state offers during a detour.
    pub resume: Shared,
    /// `unmatched`, which in this state returns to idle.
    pub unmatched: Shared,
}

/// The events that a machine's states share.
pub(crate) struct Events {
    /// `yield`, `pause`, and `unmatched`, which every state but a final one
    /// offers. Empty when every state is final.
    pub every: Vec<Shared>,
    /// The fallback state, if the machine has one that isn't final.
    pub fallback: Option<Fallback>,
}

/// The shared events of `m`.
// sokf-implements: CLI-8_AC-12
pub(crate) fn events(m: &Machine) -> Events {
    let subject = format!("the {}", m.instance.kind);
    let shared = |offer: Offer| Shared {
        host_only: m.host_only(&offer.name),
        offer,
    };
    let every = if m.states.iter().all(|s| s.is_final) {
        Vec::new()
    } else {
        let here = BuiltinsAt {
            here: Some("the state"),
            in_fallback: false,
            subject: &subject,
            back: None,
        };
        builtin_offers(m, &here).into_iter().map(shared).collect()
    };
    let fallback = m.fallback().filter(|f| !f.is_final).and_then(|f| {
        let there = BuiltinsAt {
            here: Some(&f.name),
            in_fallback: true,
            subject: &subject,
            back: Some("the state that the detour left"),
        };
        let mut offers = builtin_offers(m, &there).into_iter();
        let resume = offers.find(|o| o.name == "resume")?;
        let unmatched = offers.find(|o| o.name == "unmatched")?;
        Some(Fallback {
            state: f.name.clone(),
            resume: shared(resume),
            unmatched: shared(unmatched),
        })
    });
    Events { every, fallback }
}

#[cfg(test)]
mod tests {
    use smllm_core::SmallMap;
    use smllm_core::model::{EventDef, InstanceSpec, State};

    use super::*;

    fn machine(states: Vec<State>) -> Machine {
        Machine {
            id: "m".into(),
            description: None,
            initial: "A".into(),
            instance: InstanceSpec::default(),
            events: SmallMap::new(),
            shared: Vec::new(),
            states,
        }
    }

    // sokf-tests: CLI-8_AC-12
    #[test]
    fn every_state_but_a_final_one_and_the_fallback() {
        let state = |name: &str| State {
            name: name.into(),
            ..State::default()
        };
        let mut m = machine(vec![
            state("A"),
            State {
                fallback: true,
                ..state("F")
            },
        ]);
        m.events.insert(
            "pause",
            EventDef {
                description: Some("Park it.".into()),
                host_only: true,
                ..EventDef::default()
            },
        );
        let e = events(&m);
        let names: Vec<&str> = e.every.iter().map(|s| s.offer.name.as_str()).collect();
        assert_eq!(names, ["yield", "pause", "unmatched"]);
        assert_eq!(e.every[1].offer.description.as_deref(), Some("Park it."));
        assert!(e.every[1].host_only && !e.every[0].host_only);
        assert_eq!(e.every[0].offer.params[0].name, "note");
        let detour = e.every[2].offer.description.as_deref().unwrap();
        assert!(detour.contains("handle it in F"), "{detour}");
        let f = e.fallback.unwrap();
        assert_eq!(
            (f.state.as_str(), f.resume.offer.name.as_str()),
            ("F", "resume")
        );
        let idle = f.unmatched.offer.description.unwrap();
        assert!(idle.contains("from idle"), "{idle}");

        let done = machine(vec![State {
            is_final: true,
            ..state("A")
        }]);
        let e = events(&done);
        assert!(e.every.is_empty() && e.fallback.is_none());
    }
}
