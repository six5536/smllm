//! What a host supplies: the core has no IO, time or randomness (ENG-5).

use crate::model::Value;
use crate::prelude::*;
use crate::record::{AuditEntry, HistoryEntry, Instance, Session, Status};
use crate::utils::{SmallMap, merge_sort_by};

/// A store failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// Another writer changed the instance first (AGT-10).
    Conflict,
    /// Anything else, as a message.
    Other(String),
}

impl core::fmt::Display for HostError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HostError::Conflict => f.write_str("another session changed the instance"),
            HostError::Other(m) => f.write_str(m),
        }
    }
}

/// Sessions, bindings, instances, history and, when the host keeps one, the
/// audit.
pub trait Store {
    /// The session with `key`.
    fn session(&mut self, key: &str) -> Result<Option<Session>, HostError>;
    /// Save a session.
    fn put_session(&mut self, session: &Session) -> Result<(), HostError>;
    /// The key bound to a harness session id.
    fn binding(&mut self, harness: &str, host_session: &str) -> Result<Option<String>, HostError>;
    /// Bind a harness session id to a key.
    fn put_binding(
        &mut self,
        harness: &str,
        host_session: &str,
        key: &str,
    ) -> Result<(), HostError>;
    /// The instance `id` of `machine`.
    fn instance(&mut self, machine: &str, id: &str) -> Result<Option<Instance>, HostError>;
    /// Every instance of `machine`.
    fn instances(&mut self, machine: &str) -> Result<Vec<Instance>, HostError>;
    /// The instances of `machine` with `status`. The default filters
    /// [`Store::instances`]; a store that keeps completed instances apart
    /// answers without reading them (AGT-10 keeps them forever).
    fn instances_with(
        &mut self,
        machine: &str,
        status: Status,
    ) -> Result<Vec<Instance>, HostError> {
        let mut all = self.instances(machine)?;
        all.retain(|i| i.status == status);
        Ok(all)
    }
    /// The instance of `machine` whose ref is `r#ref` (AGT-10). The default
    /// searches [`Store::instances`]; a store may keep an index.
    fn instance_by_ref(
        &mut self,
        machine: &str,
        r#ref: &str,
    ) -> Result<Option<Instance>, HostError> {
        Ok(self
            .instances(machine)?
            .into_iter()
            .find(|i| i.r#ref.as_deref() == Some(r#ref)))
    }
    /// How many instances of `machine` have `status` (the idle list's
    /// "…and K more", the status line). The default counts
    /// [`Store::instances_with`]; a store may count without reading them.
    fn count(&mut self, machine: &str, status: Status) -> Result<usize, HostError> {
        Ok(self.instances_with(machine, status)?.len())
    }
    /// The `limit` most recently updated instances of `machine` with
    /// `status`, newest first, ties by label (AGT-9). The default sorts
    /// [`Store::instances_with`]; a store may read only the newest.
    fn recent(
        &mut self,
        machine: &str,
        status: Status,
        limit: usize,
    ) -> Result<Vec<Instance>, HostError> {
        let mut all = self.instances_with(machine, status)?;
        newest_first(&mut all);
        all.truncate(limit);
        Ok(all)
    }
    /// Save an instance whose `version` was bumped by one from the stored copy
    /// (or is 1 for a new one); anything else is [`HostError::Conflict`].
    fn put_instance(&mut self, instance: &Instance) -> Result<(), HostError>;
    /// Append a history entry.
    fn append_history(
        &mut self,
        machine: &str,
        id: &str,
        entry: &HistoryEntry,
    ) -> Result<(), HostError>;
    // sokf-implements: ENG-11_AC-7
    /// Whether the host keeps an audit (ENG-11). The default keeps none, and
    /// the engine then builds no entry.
    fn wants_audit(&self) -> bool {
        false
    }
    /// Keep one audit entry, with `reply`, the text that the call returned
    /// (empty for a stop that lets the agent stop). The host keeps what it
    /// likes of the text, such as `sha256:` and the hex of the SHA-256 of
    /// its UTF-8 bytes. The default drops both.
    fn append_audit(&mut self, entry: &AuditEntry, reply: &str) -> Result<(), HostError> {
        let _ = (entry, reply);
        Ok(())
    }
}

/// Order instances most recently updated first, ties by machine, then
/// label: the idle list's order across machines (AGT-9), and within one
/// machine the order of [`Store::recent`].
pub fn newest_first(instances: &mut Vec<Instance>) {
    merge_sort_by(instances, |a, b| {
        b.updated
            .cmp(&a.updated)
            .then_with(|| a.machine.cmp(&b.machine))
            .then_with(|| a.label().cmp(b.label()))
    });
}

/// A guard or action invocation handed to the host.
pub struct Call<'a> {
    /// Machine id.
    pub machine: &'a str,
    /// `type`.
    pub kind: &'a str,
    /// `params`.
    pub params: &'a SmallMap<Value>,
    /// Environment (ENG-4, CLI-11).
    pub env: &'a [(String, String)],
    /// The session's working dir (CLI-11).
    pub cwd: &'a str,
}

/// How a host guard or action went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Guard: true. Action: succeeded.
    pub ok: bool,
    /// Exit status, reason and output tail, for the trace (AGT-1).
    pub detail: String,
}

/// Host guard kinds (`command`).
pub trait Guard {
    /// Whether this host evaluates `kind` (ENG-4).
    fn supports(&self, kind: &str) -> bool;
    /// Evaluate a guard.
    fn check(&mut self, call: &Call<'_>) -> Outcome;
}

/// Host action kinds (`command`).
pub trait Action {
    /// Whether this host runs `kind` (ENG-4).
    fn supports(&self, kind: &str) -> bool;
    /// Run an action.
    fn run(&mut self, call: &Call<'_>) -> Outcome;
}

/// Reads the text of a file: a prompt file when the engine shows it (ENG-4),
/// and, in `smllm-format`, every file of a machine when it loads (FMT-11).
pub trait InstructionSource {
    /// The text of `file`; `Ok(None)` when it does not exist.
    fn read(&self, file: &str) -> Result<Option<String>, String>;
}

/// Matches param `pattern`s (JSON Schema / ECMA-262 regex).
pub trait Matcher {
    /// Whether `value` matches `pattern`.
    fn is_match(&self, pattern: &str, value: &str) -> Result<bool, String>;

    /// The errors of the JSON text `value` against the JSON Schema `schema`,
    /// each with the JSON path of the value that fails: empty when the value
    /// passes, and `None` when the host checks no schema, which is the
    /// default (AGT-3).
    fn check_json(&self, schema: &str, value: &str) -> Option<Vec<String>> {
        let _ = (schema, value);
        None
    }

    /// Whether [`Matcher::check_json`] checks a schema, without a check. A
    /// host that checks overrides both.
    fn checks_json(&self) -> bool {
        false
    }
}

/// Wall-clock time.
pub trait Clock {
    /// Unix time, ms.
    fn now_ms(&self) -> u64;
}

/// Randomness for keys and ids.
pub trait Ids {
    /// 64 random bits.
    fn random(&mut self) -> u64;
}

/// Everything a host supplies, borrowed for one engine call.
pub struct Host<'a> {
    /// Storage.
    pub store: &'a mut dyn Store,
    /// Host guards.
    pub guards: &'a mut dyn Guard,
    /// Host actions.
    pub actions: &'a mut dyn Action,
    /// Prompt files.
    pub source: &'a dyn InstructionSource,
    /// Patterns.
    pub matcher: &'a dyn Matcher,
    /// Time.
    pub clock: &'a dyn Clock,
    /// Randomness.
    pub ids: &'a mut dyn Ids,
}
