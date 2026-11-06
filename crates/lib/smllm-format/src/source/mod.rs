//! The state machine file as written.

mod forms;
mod machine;

pub use forms::{OneOrMany, OrderedMap, Run, StringOr};
pub use machine::{
    ActionSrc, Actions, CommandParams, EventMeta, EventSource, GuardSrc, InstanceMeta, InvokeSrc,
    MachineFile, MachineMeta, ParamsSchema, PositionSrc, PromptParams, PropSchema, RefMeta,
    SchemaSrc, SharedActionSrc, StateMeta, StateNode, StateType, TransitionSrc, Transitions,
    VisitsParams,
};
