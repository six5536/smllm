//! The calls of a host that drives a model without tools (ENG-7..ENG-10):
//! the state's text, the answer schema, the check of an answer, and the
//! host's view of the offered events. Each reads only.

use crate::engine::answer::{self, Answer};
use crate::engine::machine::{entry_prompts, location, owned};
use crate::engine::offer::{BUILTINS, Offer, for_agent, machine_offers};
use crate::engine::turn::Turn;
use crate::engine::{Engine, Error, Location};
use crate::host::Host;
use crate::model::Machine;
use crate::prelude::*;
use crate::record::{Instance, Session, Status};
use crate::render;

/// The current state as a host that drives the machine sees it: where the
/// session is, and its offered events in two lists (ENG-10). The built-in
/// events are in neither list: they belong to the host, which drives the
/// session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostView {
    /// Where the session is; all `None` in idle.
    pub location: Location,
    /// The events that the model may fire, which the host fires through
    /// [`Engine::fire`] after [`Engine::check_answer`] accepts them.
    pub model: Vec<Offer>,
    /// The events that only the host fires, through
    /// [`Engine::fire_as_host`]: those that the machine marks with
    /// `source: host`.
    pub host: Vec<Offer>,
}

/// The held instance of session `key`, when the session holds one that is
/// still its own.
type Held<'c> = (Session, &'c Machine, Instance);

/// The view of `inst` in its state: the offered events without the
/// built-ins, the model's apart from the host's.
fn view_of(machine: &Machine, inst: &Instance) -> HostView {
    let mut view = HostView {
        location: location(machine, inst),
        ..HostView::default()
    };
    for offer in machine_offers(machine, machine.state(&inst.state), inst) {
        if BUILTINS.contains(&offer.name.as_str()) {
            continue;
        }
        // sokf-implements: ENG-10_AC-2
        if for_agent(machine, &offer.name) {
            view.model.push(offer);
        } else {
            view.host.push(offer);
        }
    }
    view
}

impl Engine {
    fn held_by<'c>(&'c self, host: &mut Host<'_>, key: &str) -> Result<Option<Held<'c>>, Error> {
        let session = host
            .store
            .session(key)?
            .ok_or_else(|| Error::UnknownSession(key.to_string()))?;
        let held = match &session.holding {
            Some(k) => owned(self.config(), host.store, k, Status::Active, key)?,
            None => None,
        };
        Ok(held.map(|(m, i)| (session, m, i)))
    }

    /// The offered events of session `key`'s state, split into the model's
    /// and the host's; none in idle.
    fn split(&self, host: &mut Host<'_>, key: &str) -> Result<HostView, Error> {
        Ok(match self.held_by(host, key)? {
            Some((_, machine, inst)) => view_of(machine, &inst),
            None => HostView::default(),
        })
    }

    /// The current state of session `key`, with its offered events in two
    /// lists: the model's and the host's (ENG-10). A host asks its model
    /// when the first list isn't empty, and fires an event of the second
    /// whenever it has a reason. It reads only.
    // sokf-implements: ENG-10_AC-1
    pub fn host_view(&self, host: &mut Host<'_>, key: &str) -> Result<HostView, Error> {
        self.split(host, key)
    }

    /// The state's text for a model without tools (ENG-7): the state's
    /// instructions, the events that the model may fire with their params,
    /// and a request to answer with one event as JSON. With `schemas`, each
    /// JSON param shows its schema. In idle, or in a state that offers the
    /// model nothing, the text holds only the instructions. It reads only.
    pub fn state_text(
        &self,
        host: &mut Host<'_>,
        key: &str,
        schemas: bool,
    ) -> Result<String, Error> {
        let Some((session, machine, inst)) = self.held_by(host, key)? else {
            return Ok(String::new());
        };
        let offers = view_of(machine, &inst).model;
        let mut turn = Turn::new(self.config(), host, session, "");
        if let Some(state) = machine.state(&inst.state) {
            entry_prompts(&mut turn, machine, state);
        }
        Ok(render::state_text(&turn.prompts, &offers, schemas))
    }

    /// The JSON Schema of a valid answer in session `key`'s state (ENG-8),
    /// as compact JSON text: a `oneOf` with one branch for each event that
    /// the model may fire, with its params and each JSON param's bundled
    /// schema. It reads only.
    pub fn answer_schema(&self, host: &mut Host<'_>, key: &str) -> Result<String, Error> {
        Ok(answer::schema(&self.split(host, key)?.model))
    }

    /// Check a model's text answer in session `key`'s state (ENG-9): the
    /// event and its params, ready for [`Engine::fire`], or one refusal
    /// to send back to the model. It reads only, and fires nothing.
    pub fn check_answer(
        &self,
        host: &mut Host<'_>,
        key: &str,
        text: &str,
    ) -> Result<Result<Answer, String>, Error> {
        let offers = self.split(host, key)?.model;
        Ok(answer::check(text, &offers, host.matcher))
    }
}
