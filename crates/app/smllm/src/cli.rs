//! Command-line interface (clap derive), consistent with sokf's (CLI).

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// State machines for LLM agents.
#[derive(Debug, Parser)]
// `version` feeds the man page header and `--help`; the built-in `-V` flag stays
// disabled because we render the version ourselves (see `main::run`).
#[command(name = "smllm", about, long_about = None, version, disable_version_flag = true)]
#[command(
    after_help = "Exit codes: 0 success, 1 errors found (a config error, a rejected event, or a failed hook), 2 a usage or internal error.\n\
    Documentation: https://github.com/six5536/smllm"
)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Use only this config file (SMLLM_CONFIG also sets it).
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Print the version and exit.
    #[arg(short = 'V', long = "version", global = true)]
    pub version: bool,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create .smllm/config.toml in this directory, or the user config.
    Init(InitArgs),
    /// Print a template state machine, or write it to `<ID>.smllm.yaml`.
    New(NewArgs),
    /// Check the configs, their state machines, and the saved instances.
    Validate(ValidateArgs),
    /// Fire an event from the shell, as the smllm tool does.
    Fire(FireArgs),
    /// List sessions, or show where one is.
    #[command(subcommand)]
    Session(SessionCommand),
    /// List instances, or show one with its history.
    #[command(subcommand)]
    Instance(InstanceCommand),
    /// Install or check a harness integration (Claude Code, Codex, Cursor, and others).
    #[command(subcommand)]
    Harness(HarnessCommand),
    /// Print the states and transitions of the state machines.
    Graph(GraphArgs),
    /// Print reference information.
    #[command(subcommand)]
    Info(InfoCommand),
    /// Compile a config or state machine to compact JSON, for smllm-wasm hosts.
    Compile(CompileArgs),
    /// Print where a session is, as a status line row or as JSON.
    ///
    /// The statusline command reads Claude Code's status JSON on stdin and
    /// shows the session bound to its `session_id`. It prints nothing when
    /// there is nothing to show. It always exits 0, because a non-zero exit
    /// blanks Claude Code's whole status line.
    Statusline(StatuslineArgs),
    /// Run the stdio MCP server, which offers the `smllm` tool.
    #[command(hide = true)]
    Mcp,
    /// Print a shell completion script to stdout.
    Completions(CompletionsArgs),
    /// Print the roff man page to stdout, or with `--markdown` the commands
    /// and their options as Markdown.
    ///
    /// The man command is hidden, because it is for packaging: the release
    /// archives include its roff, and the documentation's page of the
    /// commands holds its Markdown.
    #[command(hide = true)]
    Man(ManArgs),
}

/// `init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Create the user config (~/.config/smllm/config.toml) instead.
    #[arg(long)]
    pub user: bool,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `new`.
#[derive(Debug, Args)]
pub struct NewArgs {
    /// The state machine id.
    #[arg(value_name = "ID")]
    pub id: String,
    /// The directory to write it to (default: the directory of the config).
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
    /// Write the file and list it in the config, instead of printing it.
    #[arg(long)]
    pub write: bool,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `validate`.
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Config (.toml) or state machine (.yaml) files (default: the configs found).
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
    /// List the warnings too.
    #[arg(long)]
    pub warnings: bool,
    /// List the info findings too.
    #[arg(long)]
    pub info: bool,
}

/// `fire`.
#[derive(Debug, Args)]
pub struct FireArgs {
    /// The session key (omit it only to start a session with `enter`).
    #[arg(long, value_name = "KEY")]
    pub session: Option<String>,
    /// The event to fire.
    #[arg(value_name = "EVENT")]
    pub event: String,
    /// A param of the event (repeatable).
    #[arg(long = "param", value_name = "KEY=VALUE")]
    pub params: Vec<String>,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `session …`.
#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// List sessions, the most recently active first.
    List(JsonArgs),
    /// Show where a session is, as the smllm tool does with no event, or
    /// with --audit every call that drove it.
    Show(SessionShowArgs),
}

/// `session show KEY [--audit] [--json]`.
#[derive(Debug, Args)]
pub struct SessionShowArgs {
    /// The session key.
    #[arg(value_name = "KEY")]
    pub key: String,
    /// Print the session's audit: one line for each call, or with --json the
    /// audit file's JSON lines.
    #[arg(long)]
    pub audit: bool,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `instance …`.
#[derive(Debug, Subcommand)]
pub enum InstanceCommand {
    /// List instances of the configured state machines.
    List(JsonArgs),
    /// Show an instance, by id or ref, with its history.
    Show(KeyArgs),
}

/// Just `--json`.
#[derive(Debug, Args)]
pub struct JsonArgs {
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// A key or id plus `--json`.
#[derive(Debug, Args)]
pub struct KeyArgs {
    /// The session key, or instance id or ref.
    #[arg(value_name = "KEY")]
    pub key: String,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `harness …`.
#[derive(Debug, Subcommand)]
pub enum HarnessCommand {
    /// Install or update a harness integration.
    Install(HarnessArgs),
    /// Show the state of each part of a harness integration.
    Status(HarnessArgs),
    /// Hook entry point, called by the harness.
    #[command(hide = true)]
    Hook(HookArgs),
}

/// `harness install|status`.
#[derive(Debug, Args)]
pub struct HarnessArgs {
    /// The harnesses: claude, codex, factory, gemini, copilot, cursor, pi,
    /// opencode, kilo, qwen, devin, agents, or all.
    #[arg(value_name = "HARNESS", required = true, num_args = 1..)]
    pub names: Vec<String>,
    /// The scope: project, user, or local (local for Claude Code and Copilot only).
    #[arg(long, default_value = "project", value_name = "S")]
    pub scope: String,
    /// Leave out a part, such as instructions, hooks, mcp, permissions, or statusline (repeatable).
    #[arg(long, value_name = "PART")]
    pub without: Vec<String>,
    /// Overwrite parts edited by hand.
    #[arg(long)]
    pub force: bool,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `harness hook`.
#[derive(Debug, Args)]
pub struct HookArgs {
    /// The harness that runs the hook, such as claude.
    #[arg(value_name = "HARNESS")]
    pub name: String,
    /// The hook event: session-start, prompt-submit, or stop.
    #[arg(value_name = "EVENT")]
    pub hook: String,
}

/// `graph`.
#[derive(Debug, Args)]
pub struct GraphArgs {
    /// A state machine id (default: all of them).
    #[arg(value_name = "ID")]
    pub id: Option<String>,
    /// Print JSON.
    #[arg(long, conflicts_with_all = ["mermaid", "open", "html", "png"])]
    pub json: bool,
    /// Print a Mermaid stateDiagram-v2.
    #[arg(long, conflicts_with_all = ["open", "html", "png"])]
    pub mermaid: bool,
    /// Open the interactive diagram in the default web browser (BROWSER sets the program).
    #[arg(long)]
    pub open: bool,
    /// Write the interactive diagram to FILE, an HTML page that needs only a browser.
    #[arg(long, value_name = "FILE")]
    pub html: Option<PathBuf>,
    /// Write the diagram as a PNG picture to FILE, or for several machines to STEM-ID.EXT beside it.
    #[arg(long, value_name = "FILE")]
    pub png: Option<PathBuf>,
}

/// `info …`.
#[derive(Debug, Subcommand)]
pub enum InfoCommand {
    /// Print the JSON Schema of the state machine format.
    Schema,
    /// Print the rules that validate checks, with the code that each finding ends with, as Markdown.
    Rules,
}

/// `man`.
#[derive(Debug, Args)]
pub struct ManArgs {
    /// Print the commands and their options as Markdown.
    #[arg(long)]
    pub markdown: bool,
}

/// `compile`.
#[derive(Debug, Args)]
pub struct CompileArgs {
    /// A config.toml or a state machine file.
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
    /// The file to write to, instead of stdout.
    #[arg(short = 'o', long = "out", value_name = "OUT")]
    pub out: Option<PathBuf>,
}

/// `statusline`.
#[derive(Debug, Args)]
pub struct StatuslineArgs {
    /// The session key (default: the session bound to the `session_id` of the
    /// status JSON on stdin).
    #[arg(long, value_name = "KEY")]
    pub session: Option<String>,
    /// Print JSON (`{}` when there is nothing to show).
    #[arg(long)]
    pub json: bool,
    /// When to colour the row (default: auto).
    #[arg(long, value_enum, value_name = "WHEN")]
    pub color: Option<ColorChoice>,
}

/// `--color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    /// Colour, unless NO_COLOR is set and not empty.
    Auto,
    /// Colour.
    Always,
    /// No colour.
    Never,
}

/// `completions`.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// The shell to print a completion script for.
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).unwrap()
    }

    #[test]
    fn globals_before_or_after_the_command() {
        assert!(parse(&["smllm", "validate", "-V"]).version);
        let c = parse(&["smllm", "validate", "--config", "x.toml"]);
        assert_eq!(c.config.as_deref(), Some(std::path::Path::new("x.toml")));
        let c = parse(&["smllm", "--config", "x.toml", "validate"]);
        assert!(c.config.is_some());
    }

    #[test]
    fn fire_takes_params_and_an_optional_session() {
        let Some(Command::Fire(f)) = parse(&[
            "smllm",
            "fire",
            "--session",
            "sm-1",
            "reject",
            "--param",
            "reason=x",
            "--param",
            "a=b",
        ])
        .command
        else {
            panic!()
        };
        assert_eq!(f.session.as_deref(), Some("sm-1"));
        assert_eq!(f.params, vec!["reason=x", "a=b"]);
    }

    #[test]
    fn graph_json_and_mermaid_conflict() {
        assert!(Cli::try_parse_from(["smllm", "graph", "--json", "--mermaid"]).is_err());
        for picture in [&["--open"][..], &["--html", "a.html"], &["--png", "a.png"]] {
            for text in ["--json", "--mermaid"] {
                let args = [&["smllm", "graph", text][..], picture].concat();
                assert!(Cli::try_parse_from(args).is_err());
            }
        }
        let Some(Command::Graph(g)) = parse(&[
            "smllm", "graph", "dev", "--open", "--html", "a.html", "--png", "a.png",
        ])
        .command
        else {
            panic!()
        };
        assert!(g.open && g.html.is_some() && g.png.is_some());
        assert!(Cli::try_parse_from(["smllm", "frobnicate"]).is_err());
    }
}
