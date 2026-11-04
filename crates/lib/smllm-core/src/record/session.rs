//! A session: one harness conversation, keyed by an smllm session key.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::record::InstanceKey;

/// A session record (STORE-1, STORE-2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Session {
    /// The session key (`sm-k7f3q2`).
    pub key: String,
    /// The harness that bound it (`claude`, `none`).
    pub harness: String,
    /// The harness's own session id, when it has one.
    #[cfg_attr(feature = "serde", serde(default))]
    pub host_session: Option<String>,
    /// Working directory at bind: guard/action commands run here (CLI-11).
    pub cwd: String,
    /// The config files bound at creation (STORE-2); opaque to the core.
    #[cfg_attr(feature = "serde", serde(default))]
    pub configs: Vec<String>,
    /// The instance this session is working on; `None` = idle.
    #[cfg_attr(feature = "serde", serde(default))]
    // sokf-implements: AGT-10_AC-1
    pub holding: Option<InstanceKey>,
    /// The instance put aside by `unmatched` (IDLE: detour).
    #[cfg_attr(feature = "serde", serde(default))]
    pub interrupted: Option<InstanceKey>,
    /// The agent fired `yield` since the last user prompt (AGT-8, AGT-8).
    #[cfg_attr(feature = "serde", serde(default))]
    pub yielded: bool,
    /// The stop hook blocked, and the agent has fired no event since
    /// (AGT-8): only then does a harness continuation let the agent stop.
    #[cfg_attr(feature = "serde", serde(default))]
    pub blocked: bool,
    /// Created, unix ms.
    pub created: u64,
    /// Last call, unix ms.
    pub last_active: u64,
}

impl Session {
    /// The stop hook lets the agent stop at once: it yielded, or it holds
    /// no instance (AGT-8, AGT-8).
    pub fn may_stop(&self) -> bool {
        self.yielded || self.holding.is_none()
    }
}
