//! JSON Schema of the state machine format, derived from the source types
//! (FMT-7).

use crate::source::MachineFile;

/// The JSON Schema, pretty-printed.
// sokf-implements: FMT-7_AC-1
pub fn json_schema() -> String {
    let schema = schemars::schema_for!(MachineFile);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}
