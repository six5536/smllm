//! The agent-facing `smllm` tool: one definition for every host, so the MCP
//! server and the wasm offer the agent the same tool (WASM-1, AGT-4).

use crate::prelude::*;
use crate::render::AGENT_RULES;

/// The tool's name.
pub const TOOL_NAME: &str = "smllm";

/// The tool's input schema, as JSON: params are strings (MCP-1).
pub const TOOL_INPUT_SCHEMA: &str = r#"{"type":"object","properties":{"session":{"type":"string","description":"The session key from the latest <smllm> header, such as sm-k7f3q2. Omit only to start a session with event enter."},"event":{"type":"string","description":"The event to fire. Omit it to see where you are."},"params":{"type":"object","description":"The event's params.","additionalProperties":{"type":"string"}}}}"#;

/// The tool's description: the full agent rules (AGT-4), then how to call.
// sokf-implements: AGT-4_AC-2
pub fn tool_description() -> String {
    format!(
        "{AGENT_RULES}\n\nTo see where you are, call with {{ session }} alone. To fire an event, \
         call with {{ session, event, params }}. params is an object of strings."
    )
}
