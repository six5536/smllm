//! The state machine file as written: an XState v5 machine config in YAML,
//! smllm data under `meta` (CFG-1, CFG-2). The JSON Schema is derived from
//! these types (CFG-15).
// @zen-component: CFG-Source

use schemars::JsonSchema;
use serde::Deserialize;

use crate::source::{OneOrMany, OrderedMap, Run, StringOr};

/// A state machine file (`<id>.smllm.yaml`).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(
    title = "smllm state machine",
    description = "An XState v5 machine config in YAML, limited to the parts that smllm supports. smllm's own data is under `meta`."
)]
pub struct MachineFile {
    /// The machine's id, unique within the config: letters, digits, `_`, and `-`.
    pub id: String,
    /// The machine's description, which the idle list shows.
    #[serde(default)]
    pub description: Option<String>,
    /// The state in which a new instance starts.
    pub initial: String,
    /// smllm's data for the machine.
    pub meta: MachineMeta,
    /// The machine's states. smllm does not support nested `states`.
    pub states: OrderedMap<StateNode>,
}

/// smllm's data for the machine.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MachineMeta {
    /// The smllm format version, which is 1.
    pub smllm: u32,
    /// What the machine calls an instance, and the instance's ref.
    #[serde(default)]
    pub instance: Option<InstanceMeta>,
    /// The machine's events, each declared once. A built-in event can set only its `description`.
    #[serde(default)]
    pub events: Option<OrderedMap<EventMeta>>,
    /// Actions added to the `entry` or `exit` actions of several states.
    #[serde(default)]
    pub shared_actions: Option<Vec<SharedActionSrc>>,
}

/// `meta.instance`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InstanceMeta {
    /// The word for an instance in the text that smllm writes, such as `issue`. The default is `instance`.
    #[serde(default)]
    pub kind: Option<String>,
    /// The instance's ref: its id outside smllm, such as an issue number.
    #[serde(default, rename = "ref")]
    pub r#ref: Option<RefMeta>,
}

/// `meta.instance.ref`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RefMeta {
    /// The name of the ref's param, which `enter` and `setRef` use. The default is `ref`.
    #[serde(default)]
    pub param: Option<String>,
    /// The prompt that asks for the ref.
    #[serde(default)]
    pub description: Option<String>,
    /// A pattern that the ref must match, in the syntax of a param's `pattern`.
    #[serde(default)]
    pub pattern: Option<String>,
}

/// `meta.events.<name>`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EventMeta {
    /// The event's guidance for the agent, unless a transition gives its own.
    #[serde(default)]
    pub description: Option<String>,
    /// The event's params: a JSON Schema object of string properties.
    #[serde(default)]
    pub params: Option<ParamsSchema>,
}

/// A subset of a JSON Schema of `type: object`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ParamsSchema {
    /// Always `object`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The params, by name.
    #[serde(default)]
    pub properties: OrderedMap<PropSchema>,
    /// The names of the required params.
    #[serde(default)]
    pub required: Vec<String>,
}

/// One param's schema.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PropSchema {
    /// Always `string`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The prompt for the param.
    #[serde(default)]
    pub description: Option<String>,
    /// The allowed values.
    #[serde(default, rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    /// A pattern that the value must match, in Rust `regex` syntax: the common
    /// subset of JSON Schema's ECMA-262 patterns, without lookaround or
    /// backreferences. In smllm, `\d`, `\w`, and `\s` match Unicode, and in a
    /// JavaScript host they match only ASCII. To match ASCII digits in both, write
    /// `[0-9]`.
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Before or after a state's own actions.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PositionSrc {
    /// Before the state's own actions.
    Before,
    /// After the state's own actions.
    After,
}

/// One `meta.sharedActions` entry.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SharedActionSrc {
    /// The states that the actions are added to.
    pub states: Vec<String>,
    /// Whether the actions run before or after the state's own `entry` or `exit` actions.
    pub position: PositionSrc,
    /// Actions added to each state's `entry`.
    #[serde(default)]
    pub entry: Option<Actions>,
    /// Actions added to each state's `exit`.
    #[serde(default)]
    pub exit: Option<Actions>,
}

/// State `type`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StateType {
    /// An ordinary state, the default.
    Atomic,
    /// A state that completes the instance.
    Final,
}

/// One state.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StateNode {
    /// The state's description.
    #[serde(default)]
    pub description: Option<String>,
    /// The state's type. A `final` state completes the instance.
    #[serde(default, rename = "type")]
    pub kind: Option<StateType>,
    /// smllm's data for the state.
    #[serde(default)]
    pub meta: Option<StateMeta>,
    /// The actions that run when the state is entered. Without a prompt among
    /// them, smllm shows the prompt file `enter-<STATE>.md`, if it exists.
    #[serde(default)]
    pub entry: Option<Actions>,
    /// The actions that run when the state is left.
    #[serde(default)]
    pub exit: Option<Actions>,
    /// The transitions for each event.
    #[serde(default)]
    pub on: Option<OrderedMap<Transitions>>,
    /// Transitions without an event. smllm leaves a state with `always` at once.
    #[serde(default)]
    pub always: Option<Transitions>,
}

/// smllm's data for a state.
// @zen-impl: CFG-10_AC-1
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StateMeta {
    /// Whether the agent can enter this state directly from idle.
    #[serde(default)]
    pub entry_point: bool,
    /// Whether the `unmatched` event enters this state. At most one state of a machine can be the fallback.
    #[serde(default)]
    pub fallback: bool,
    /// Prompts for params in this state, by event and then by param.
    #[serde(default)]
    pub param_descriptions: Option<OrderedMap<OrderedMap<String>>>,
}

/// One transition or a guarded list (first match wins).
pub type Transitions = OneOrMany<StringOr<TransitionSrc>>;

/// A transition object.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TransitionSrc {
    /// The target state. A transition without a target stays in the state without re-entering it.
    #[serde(default)]
    pub target: Option<String>,
    /// A condition that must pass for the transition to be taken.
    #[serde(default)]
    pub guard: Option<GuardSrc>,
    /// The actions that run when the transition is taken.
    #[serde(default)]
    pub actions: Option<Actions>,
    /// Whether a transition to its own state leaves and re-enters it, as in XState.
    #[serde(default)]
    pub reenter: Option<bool>,
    /// Guidance for the event in this state.
    #[serde(default)]
    pub description: Option<String>,
}

/// One action or a list.
pub type Actions = OneOrMany<StringOr<ActionSrc>>;

/// An action, written as `{type, params}`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "params",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum ActionSrc {
    /// Shows text to the agent.
    Prompt(PromptParams),
    /// Runs a command.
    Command(CommandParams),
    /// Sets the instance's ref from the event's ref param.
    SetRef,
}

/// The params of `prompt`, which takes exactly one of `text` and `file`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PromptParams {
    /// The text itself.
    #[serde(default)]
    pub text: Option<String>,
    /// A file of text, relative to the machine file, which smllm reads each time it shows the prompt.
    #[serde(default)]
    pub file: Option<String>,
}

/// The params of `command`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CommandParams {
    /// A string that a shell runs, or a list of a program and its arguments, run without a shell.
    pub run: Run,
    /// The time limit in seconds. The default is 60.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// The working directory, relative to the machine file. The default is the session's working directory.
    #[serde(default)]
    pub cwd: Option<String>,
}

/// A guard, written as `{type, params}`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "params",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum GuardSrc {
    /// Passes when the command exits with status 0.
    Command(CommandParams),
    /// Passes when `state` has been entered at least `atLeast` times.
    Visits(VisitsParams),
}

/// `visits` params.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VisitsParams {
    /// The state whose entries are counted.
    pub state: String,
    /// The least number of entries, counting the current one.
    pub at_least: u32,
}
