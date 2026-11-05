//! smllm-format: read, check and compile smllm state machine files.
//!
//! A state machine file is an XState v5 machine config written in YAML, with
//! smllm's data under `meta` (see [`source::MachineFile`]). [`load_machine`]
//! parses and validates one, collecting [`Finding`]s, and lowers it to the
//! [`smllm_core`] model; [`load_machine_from`] reads its files through an
//! [`InstructionSource`], such as [`FileSource`] or [`MemorySource`];
//! [`load_configs`] combines user and project `config.toml`s;
//! [`json_schema`] is the format's JSON Schema; [`compile`] emits the model
//! as JSON for wasm hosts.

#![warn(missing_docs)]
// This crate's clippy.toml bans Path::display; the tests may print a path
// as the system does.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod bundle;
mod compile;
mod finding;
mod load;
mod locate;
mod lower;
mod path_text;
pub mod rules;
mod schema;
mod shape;
pub mod source;
mod sources;
mod template;

pub use compile::compile;
pub use finding::{Finding, Findings, Severity};
pub use load::{
    ConfigFile, Loaded, MachineSource, Mode, Origin, load_configs, load_machine, load_machine_from,
};
pub use locate::locate;
pub use path_text::{path_text, system_path_text};
pub use schema::json_schema;
pub use smllm_core::host::InstructionSource;
pub use sources::{FileSource, MemorySource};
pub use template::{config_template, machine_template};
