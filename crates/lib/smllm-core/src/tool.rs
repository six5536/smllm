//! The agent-facing `smllm` tool: one definition for every host, so the MCP
//! server and the wasm offer the agent the same tool (HOST-13, HOST-12).
// @zen-component: ENG-Engine

use crate::prelude::*;
use crate::render::AGENT_RULES;

/// The tool's name.
pub const TOOL_NAME: &str = "smllm";

/// The tool's input schema, as JSON: params are strings (CFG-16).
pub const TOOL_INPUT_SCHEMA: &str = r#"{"type":"object","properties":{"session":{"type":"string","description":"The session key from the latest <smllm> header, such as sm-k7f3q2. Omit only to start a session with event enter."},"event":{"type":"string","description":"The event to fire. Omit it to see where you are."},"params":{"type":"object","description":"The event's params.","additionalProperties":{"type":"string"}}}}"#;

/// The tool's description: the full agent rules (HOST-12), then how to call.
// @zen-impl: HOST-12_AC-1
pub fn tool_description() -> String {
    format!(
        "{AGENT_RULES}\n\nTo see where you are, call with {{ session }} alone. To fire an event, \
         call with {{ session, event, params }}. params is an object of strings."
    )
}
