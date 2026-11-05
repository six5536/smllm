//! One audit entry per call that drives a session, handed to a store that
//! wants an audit (ENG-11).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::utils::SmallMap;

/// An audit entry: what one call asked for and how the engine answered.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AuditEntry {
    /// When, unix ms.
    pub at: u64,
    /// Session key.
    pub session: String,
    /// Who made the call: `agent`, or `host` for `fire_as_host`.
    pub source: String,
    /// The call: `bind`, `view` (the tool with no event), `menu`, `fire`,
    /// or `stop`.
    pub call: String,
    /// The event a `fire` named.
    #[cfg_attr(feature = "serde", serde(default))]
    pub event: Option<String>,
    /// The machine of the instance that the call concerns.
    #[cfg_attr(feature = "serde", serde(default))]
    pub machine: Option<String>,
    /// That instance's id.
    #[cfg_attr(feature = "serde", serde(default))]
    pub instance: Option<String>,
    /// Its state before the call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub from: Option<String>,
    /// The state where the session is after the call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub to: Option<String>,
    /// The params as the call gave them, each as its text (a JSON value's
    /// compact JSON), a repeated name's last value.
    #[cfg_attr(feature = "serde", serde(default))]
    pub params: SmallMap<String>,
    /// The names of the events that the reply offered.
    #[cfg_attr(feature = "serde", serde(default))]
    pub offered: Vec<String>,
    /// `accepted` or `refused`; for `stop`, `allow`, `block`, or `runaway`.
    pub outcome: String,
    /// Why the engine refused the call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub reason: Option<String>,
    /// Notes, guard results, states passed through, failed actions.
    #[cfg_attr(feature = "serde", serde(default))]
    pub trace: Vec<String>,
    /// The new session's binding, on the call that created the session.
    #[cfg_attr(feature = "serde", serde(default))]
    pub binding: Option<AuditBinding>,
}

/// How a new session is bound: what [`crate::Bind`] gave it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AuditBinding {
    /// The harness (`claude`, `none`).
    pub harness: String,
    /// The harness's own session id, when it has one.
    #[cfg_attr(feature = "serde", serde(default))]
    pub host_session: Option<String>,
    /// The working folder.
    pub cwd: String,
    /// The config files.
    #[cfg_attr(feature = "serde", serde(default))]
    pub configs: Vec<String>,
}
