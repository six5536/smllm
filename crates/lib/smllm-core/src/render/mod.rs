mod block;
mod model;
mod text;

pub(crate) use block::{Block, push_safe};
pub(crate) use model::{ANSWER_FORM, state_text};
pub use text::{AGENT_RULES, format_utc};
pub(crate) use text::{header, idle_header, quote};
