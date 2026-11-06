//! The validated, lowered state machine model the engine runs. `smllm-format`
//! builds it from YAML; `smllm compile` serialises it for wasm hosts.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::model::{ActionDef, GuardDef};
use crate::prelude::*;
use crate::utils::SmallMap;

/// Everything the engine needs: the state machines plus idle's extra text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Config {
    /// State machines, in config order.
    pub machines: Vec<Machine>,
    /// Prompt actions shown in the idle list (`[idle] on-enter`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub idle: Vec<ActionDef>,
}

impl Config {
    /// The machine with `id`.
    pub fn machine(&self, id: &str) -> Option<&Machine> {
        self.machines.iter().find(|m| m.id == id)
    }
}

/// One state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Machine {
    /// Unique id within the config.
    pub id: String,
    /// Shown in the idle list.
    pub description: Option<String>,
    /// Where a new instance starts.
    pub initial: String,
    /// Instance vocabulary (FMT-4).
    pub instance: InstanceSpec,
    /// Declared event types (FMT-4), including guidance overrides for built-ins.
    pub events: SmallMap<EventDef>,
    /// `meta.sharedActions` (AGT-6).
    pub shared: Vec<SharedAction>,
    /// States, in file order.
    pub states: Vec<State>,
}

impl Machine {
    /// The state named `name`.
    pub fn state(&self, name: &str) -> Option<&State> {
        self.states.iter().find(|s| s.name == name)
    }

    /// The `meta.fallback` state, if any (AGT-7).
    pub fn fallback(&self) -> Option<&State> {
        self.states.iter().find(|s| s.fallback)
    }

    /// States that may be entered directly from idle (`meta.entryPoint`).
    pub fn entry_points(&self) -> impl Iterator<Item = &State> {
        self.states.iter().filter(|s| s.entry_point)
    }

    /// Every state's name, in order.
    pub(crate) fn state_names(&self) -> impl Iterator<Item = &str> {
        self.states.iter().map(|s| s.name.as_str())
    }

    /// Whether `meta.events` marks `event` with `source: host`.
    pub fn host_only(&self, event: &str) -> bool {
        self.events.get(event).is_some_and(|e| e.host_only)
    }

    /// The entry points' names, in order.
    pub(crate) fn entry_point_names(&self) -> impl Iterator<Item = &str> {
        self.entry_points().map(|s| s.name.as_str())
    }
}

/// `meta.instance`: what an instance is called and its ref param (FMT-4).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct InstanceSpec {
    /// The kind of instance, used in all generated text (`issue`).
    pub kind: String,
    /// The ref's param name (`issueId`).
    pub ref_param: String,
    /// The ref param's prompt.
    pub ref_description: Option<String>,
    /// The ref's pattern.
    pub ref_pattern: Option<String>,
}

impl Default for InstanceSpec {
    fn default() -> Self {
        Self {
            kind: "instance".to_string(),
            ref_param: "ref".to_string(),
            ref_description: None,
            ref_pattern: None,
        }
    }
}

/// An event type from `meta.events` (FMT-4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct EventDef {
    /// Default guidance.
    pub description: Option<String>,
    /// Params, in declaration order.
    pub params: Vec<ParamSpec>,
    /// `source: host`: only the host fires the event, through
    /// `Engine::fire_as_host`, and no agent sees it (AGT-2, AGT-3).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "core::ops::Not::not")
    )]
    pub host_only: bool,
}

impl EventDef {
    /// The param named `name`.
    pub fn param(&self, name: &str) -> Option<&ParamSpec> {
        self.params.iter().find(|p| p.name == name)
    }
}

/// One param (JSON Schema subset, FMT-4, FMT-10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct ParamSpec {
    /// Param name.
    pub name: String,
    /// Prompt.
    pub description: Option<String>,
    /// Listed in `required`.
    pub required: bool,
    /// `enum` values, empty when unconstrained.
    pub enum_values: Vec<String>,
    /// `pattern`.
    pub pattern: Option<String>,
    /// `type`: a string, or a JSON value. Left out of the JSON when `string`,
    /// so a machine without a JSON param compiles as before.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "type", default, skip_serializing_if = "ParamKind::is_string")
    )]
    pub kind: ParamKind,
    /// A JSON param's JSON Schema, bundled when the machine loaded, as
    /// compact JSON text: every file that it refers to is a resource under
    /// its `$defs` (FMT-10).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub schema: Option<String>,
}

/// A param's `type` (FMT-10).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum ParamKind {
    /// A string, the default.
    #[default]
    String,
    /// Any JSON value, which the engine hands over as compact JSON text.
    Json,
}

impl ParamKind {
    /// Whether this is [`ParamKind::String`].
    pub fn is_string(&self) -> bool {
        *self == ParamKind::String
    }
}

/// Before or after a state's own `entry`/`exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Position {
    /// Before the state's own actions.
    Before,
    /// After them.
    After,
}

/// One `meta.sharedActions` entry (AGT-6).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct SharedAction {
    /// States it applies to.
    pub states: Vec<String>,
    /// Before or after.
    pub position: Position,
    /// Added to those states' `entry`.
    pub entry: Vec<ActionDef>,
    /// Added to those states' `exit`.
    pub exit: Vec<ActionDef>,
}

/// One state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct State {
    /// State name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// `type: final`.
    pub is_final: bool,
    /// `meta.entryPoint`.
    pub entry_point: bool,
    /// `meta.fallback`.
    pub fallback: bool,
    /// `entry` actions.
    pub entry: Vec<ActionDef>,
    /// `exit` actions.
    pub exit: Vec<ActionDef>,
    /// `on`, in file order.
    pub on: Vec<On>,
    /// `always` (eventless) transitions.
    pub always: Vec<Transition>,
    /// `meta.paramDescriptions`: event → param → prompt (FMT-4).
    pub param_descriptions: SmallMap<SmallMap<String>>,
    /// `invoke`: the host's work while an instance is in this state, in
    /// file order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub invoke: Vec<Invoke>,
}

/// One piece of the host's work that a state names: XState's `invoke`, with
/// only `src` and `input`. The engine runs nothing: the host's view of the
/// state shows it, and the host fires the work's result as an event of
/// `source: host`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Invoke {
    /// The name of the work, by which the host finds its code.
    pub src: String,
    /// The fixed values of the work, as compact JSON text of an object.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub input: Option<String>,
}

impl State {
    /// The transitions for `event`.
    pub fn on(&self, event: &str) -> Option<&On> {
        self.on.iter().find(|o| o.event == event)
    }
}

/// One `on` entry: an event and its guarded transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct On {
    /// Event name.
    pub event: String,
    /// Candidates, first match wins (AGT-5).
    pub transitions: Vec<Transition>,
}

impl On {
    /// Per-state guidance: the first transition `description` (FMT-4).
    pub fn description(&self) -> Option<&str> {
        self.transitions
            .iter()
            .find_map(|t| t.description.as_deref())
    }
}

/// One transition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Transition {
    /// Target state; `None` = targetless (internal).
    pub target: Option<String>,
    /// Guard.
    pub guard: Option<GuardDef>,
    /// Transition actions.
    pub actions: Vec<ActionDef>,
    /// XState `reenter`.
    pub reenter: bool,
    /// Per-state guidance.
    pub description: Option<String>,
}
