//! What a host supplies: storage, guard/action runners, prompt files, pattern
//! matching, time and randomness (ENG-5).

mod memory;
mod traits;

pub use memory::MemoryStore;
pub use traits::{
    Action, Call, Clock, Guard, Host, HostError, Ids, InstructionSource, Matcher, Outcome, Store,
    newest_first,
};
