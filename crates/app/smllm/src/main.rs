//! smllm CLI entry point.
// Under the nightly coverage job (cargo-llvm-cov sets `coverage_nightly`), enable
// the attribute used to exclude genuinely untestable glue from coverage. Inert on
// the stable toolchain used for normal builds and tests.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(missing_docs)]
// The wasm's size rules of clippy.toml cover only the crates the wasm links.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod cache;
mod cli;
mod commands;
mod error;
mod host;
mod load_cache;
mod man;
mod output;
mod paths;
mod runtime;
mod store;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use crate::cli::{
    Cli, Command, CompletionsArgs, HarnessCommand, InfoCommand, InstanceCommand, SessionCommand,
};
use crate::commands::{config, graph, harness, hook, mcp, state, statusline};
use crate::error::Result;

fn main() -> ExitCode {
    agent_harness_adapter_core::cli::finish(run())
}

fn run() -> Result<u8> {
    let cli = Cli::parse();
    if cli.version {
        // `name x.y.z`, the near-universal CLI convention, so pasted output is
        // self-identifying in bug reports.
        output::text(&format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))?;
        return Ok(output::EXIT_OK);
    }
    let Some(command) = cli.command else {
        Cli::command().print_help().ok();
        return Ok(output::EXIT_OK);
    };
    let explicit = cli.config.as_deref();
    match command {
        Command::Init(a) => config::init(&a),
        Command::New(a) => config::new(&a, explicit),
        Command::Validate(a) => config::validate(&a, explicit),
        Command::Fire(a) => state::fire(&a, explicit),
        Command::Session(SessionCommand::List(a)) => state::session_list(&a),
        Command::Session(SessionCommand::Show(a)) => state::session_show(&a),
        Command::Instance(InstanceCommand::List(a)) => state::instance_list(&a, explicit),
        Command::Instance(InstanceCommand::Show(a)) => state::instance_show(&a, explicit),
        Command::Harness(HarnessCommand::Install(a)) => harness::run(&a, true, explicit),
        Command::Harness(HarnessCommand::Status(a)) => harness::run(&a, false, explicit),
        Command::Harness(HarnessCommand::Hook(a)) => hook::hook(&a),
        Command::Graph(a) => graph::graph(&a, explicit),
        Command::Info(InfoCommand::Schema) => config::schema(),
        Command::Info(InfoCommand::Rules) => config::rules(),
        Command::Compile(a) => config::compile_cmd(&a),
        Command::Statusline(a) => statusline::statusline(&a),
        Command::Mcp => mcp::serve(explicit),
        Command::Completions(a) => completions(&a),
        Command::Man => man::render().map_err(output::stdout_err).and_then(|b| {
            agent_harness_adapter_core::cli::write_stdout(&b).map_err(output::stdout_err)?;
            Ok(output::EXIT_OK)
        }),
    }
}

// sokf-implements: CLI-10_AC-1
/// Write a shell completion script for `shell` to stdout.
fn completions(args: &CompletionsArgs) -> Result<u8> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    // Rendered into a buffer rather than straight to stdout: `generate`
    // `expect()`s its writes, so a closed pipe would abort the process
    // (`panic = "abort"`) instead of surfacing an error we can classify.
    let mut buf = Vec::new();
    clap_complete::generate(args.shell, &mut cmd, name, &mut buf);
    agent_harness_adapter_core::cli::write_stdout(&buf).map_err(output::stdout_err)?;
    Ok(output::EXIT_OK)
}

/// Helpers for the unit tests. They live in this file, not in a file of
/// their own, because the public export leaves out the test-only files and
/// keeps this one.
#[cfg(test)]
mod test_support {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    /// A fresh, empty temporary directory, unique per call and process.
    pub(crate) fn temp_dir(name: &str) -> PathBuf {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("smllm-{name}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
