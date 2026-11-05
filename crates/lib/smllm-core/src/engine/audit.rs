//! The audit (ENG-11): one entry for each call that drives a session, built
//! only when the store wants one.

use crate::engine::{Location, ParamValue, turn::Turn};
use crate::host::HostError;
use crate::prelude::*;
use crate::record::{AuditBinding, AuditEntry};

/// What a call asked for.
pub(crate) struct Ask<'p> {
    /// `agent` or `host`.
    pub source: &'static str,
    /// `bind`, `view`, `menu`, `fire`, or `stop`.
    pub call: &'static str,
    /// The event a `fire` names.
    pub event: Option<&'p str>,
    /// The params as given.
    pub params: &'p [(String, ParamValue)],
    /// The call created the session: the entry holds its binding.
    pub created: bool,
}

impl<'p> Ask<'p> {
    /// A call of the agent's, with no event.
    pub(crate) fn agent(call: &'static str) -> Self {
        Self {
            source: "agent",
            call,
            event: None,
            params: &[],
            created: false,
        }
    }
}

/// Whether to audit this call: ask the store once, and note the instance the
/// session holds before the call changes it.
// sokf-implements: ENG-11_AC-5
pub(crate) fn begin(turn: &mut Turn<'_, '_>) {
    if turn.host.store.wants_audit() {
        turn.audit = true;
        turn.held = turn.session.holding.clone();
    }
}

/// Hand the store the entry of a call that ended with `outcome` at `at`,
/// and the text of its reply (ENG-11).
// sokf-implements: ENG-11_AC-1
// sokf-implements: ENG-11_AC-2
// sokf-implements: ENG-11_AC-3
// sokf-implements: ENG-11_AC-4
pub(crate) fn record(
    turn: &mut Turn<'_, '_>,
    ask: &Ask<'_>,
    outcome: &str,
    at: &Location,
    text: &str,
) -> Result<(), HostError> {
    if !turn.audit {
        return Ok(());
    }
    let (machine, instance) = match &turn.held {
        Some(k) => (Some(k.machine.clone()), Some(k.id.clone())),
        None => (at.machine.clone(), at.instance.clone()),
    };
    let trace = match turn.logged.take() {
        Some(t) => t,
        None => turn.trace_lines(),
    };
    let s = &turn.session;
    let entry = AuditEntry {
        at: turn.now,
        session: s.key.clone(),
        source: ask.source.to_string(),
        call: ask.call.to_string(),
        event: ask.event.map(ToString::to_string),
        machine,
        instance,
        from: turn.before.take(),
        to: at.state.clone(),
        params: ask
            .params
            .iter()
            .map(|(k, v)| {
                let (ParamValue::Str(t) | ParamValue::Json(t) | ParamValue::Text(t)) = v;
                (k.clone(), t.clone())
            })
            .collect(),
        offered: core::mem::take(&mut turn.offered),
        outcome: outcome.to_string(),
        reason: turn.refusal.take(),
        trace,
        binding: ask.created.then(|| AuditBinding {
            harness: s.harness.clone(),
            host_session: s.host_session.clone(),
            cwd: s.cwd.clone(),
            configs: s.configs.clone(),
        }),
    };
    turn.host.store.append_audit(&entry, text)
}

/// `accepted` or `refused`, as the reply's `ok` says.
pub(crate) fn outcome(ok: bool) -> &'static str {
    if ok { "accepted" } else { "refused" }
}
