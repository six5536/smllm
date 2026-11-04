//! `harness install|status` for every built-in harness (HARN-1, HARN-2,
//! HARN-2, HARN-1).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use agent_harness_adapter_core::integration::{Hook, McpServer};
use agent_harness_adapter_core::{
    DeclinedStore, ExternalPart, Harness, InstallOptions, Integration, Part, Scope, State,
    TomlDeclined, Tool, harness, install, status,
};
use serde_json::json;

use crate::cli::HarnessArgs;
use crate::commands::{hook, statusline};
use crate::error::Result;
use crate::output::{self, EXIT_OK};
use crate::paths::{self, CONFIG_FILE, PROJECT_DIR};

/// The tiny orientation block for the instructions file each harness reads
/// (D29); the full rules are in the MCP tool description.
// sokf-implements: HARN-2_AC-2
pub const INSTRUCTIONS_BLOCK: &str = "\
## smllm

smllm state machines guide the work in this project. Everything smllm says is fenced in \
`<smllm>…</smllm>`. Follow its `<instructions>`. When the work is done, fire one of its \
`<events>` with the `smllm` MCP tool, and pass the session key from the `<smllm>` header. \
Before you stop to ask the user something, fire `yield`. Only the main agent calls smllm.
";

/// `--all`'s stand-in among the harness names.
pub const ALL: &str = "all";

/// Claude Code's own parts, beside the integration's items: the status line
/// skill, and at user scope the MCP server (part `mcp`, which Claude Code
/// does not render there). With the items' `permissions`, these keep
/// smllm 0.2's part names, so declined parts stored then still apply.
// sokf-implements: HARN-1_AC-5
const STATUSLINE: &str = "statusline";
const MCP_USER: &str = "mcp";

fn server() -> McpServer {
    McpServer::stdio("smllm", "smllm", ["mcp"])
}

/// The `claude` CLI. On Windows `Command` looks only for `claude.exe` on
/// PATH, and an npm install of Claude Code is `claude.cmd`, so look for both.
fn claude() -> Command {
    #[cfg(windows)]
    if let Some(found) = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .flat_map(|dir| ["claude.exe", "claude.cmd"].map(|name| dir.join(name)))
            .find(|p| p.is_file())
    }) {
        return Command::new(found);
    }
    Command::new("claude")
}

/// User-scope MCP registration through `claude mcp`, never by editing
/// `~/.claude.json` (HARN-1); that file is only read to observe it.
// sokf-implements: HARN-1_AC-4
#[derive(Debug)]
struct ClaudeMcpUser {
    claude_json: PathBuf,
}

impl ExternalPart for ClaudeMcpUser {
    fn location(&self) -> String {
        "claude mcp (user)".into()
    }

    fn expected(&self) -> String {
        server().to_json().to_string()
    }

    fn observe(&self) -> agent_harness_adapter_core::Result<Option<String>> {
        let Ok(text) = std::fs::read_to_string(&self.claude_json) else {
            return Ok(None);
        };
        let doc: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        Ok(doc.pointer("/mcpServers/smllm").map(|s| {
            json!({ "command": s.get("command").cloned().unwrap_or_default(), "args": s.get("args").cloned().unwrap_or_default() })
                .to_string()
        }))
    }

    fn write(&self) -> agent_harness_adapter_core::Result<()> {
        let _ = claude()
            .args(["mcp", "remove", "--scope", "user", "smllm"])
            .output();
        let out = claude()
            .args([
                "mcp",
                "add-json",
                "--scope",
                "user",
                "smllm",
                &self.expected(),
            ])
            .output()
            .map_err(|e| {
                agent_harness_adapter_core::Error::Refused(format!("could not run claude: {e}"))
            })?;
        if out.status.success() {
            Ok(())
        } else {
            Err(agent_harness_adapter_core::Error::Refused(format!(
                "claude mcp add-json failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )))
        }
    }
}

/// smllm as a harness-adapter tool.
pub struct Smllm {
    project: PathBuf,
    home: PathBuf,
    user_config: PathBuf,
}

impl Smllm {
    /// The tool for a project: the dir holding the `.smllm/` of `--config` /
    /// SMLLM_CONFIG, else of the nearest project config, else `cwd`.
    pub fn new(cwd: &Path, explicit: Option<&Path>) -> Result<Self> {
        let config = paths::explicit(explicit)
            .map(|p| cwd.join(p))
            .or_else(|| paths::project_config(cwd));
        let project = config
            .and_then(|c| {
                let dir = c.parent()?;
                if dir.file_name().is_some_and(|n| n == PROJECT_DIR) {
                    dir.parent().map(Path::to_path_buf)
                } else {
                    Some(dir.to_path_buf())
                }
            })
            .unwrap_or_else(|| cwd.to_path_buf());
        Ok(Self {
            project,
            home: paths::home_dir()?,
            user_config: paths::user_config_dir()?,
        })
    }

    /// Where Claude Code looks for `statusLine`, in precedence order.
    fn settings(&self, scope: Scope) -> Vec<PathBuf> {
        let user = self.home.join(".claude/settings.json");
        match scope {
            Scope::User => vec![user],
            _ => vec![
                self.project.join(".claude/settings.local.json"),
                self.project.join(".claude/settings.json"),
                user,
            ],
        }
    }
}

impl Tool for Smllm {
    fn name(&self) -> &str {
        "smllm"
    }

    fn harnesses(&self) -> Vec<Arc<dyn Harness>> {
        harness::builtin()
    }

    // sokf-implements: HARN-2_AC-1
    // sokf-implements: HARN-1_AC-2
    fn integration(&self, scope: Scope) -> Integration {
        let mut i = Integration::new()
            .instructions(INSTRUCTIONS_BLOCK)
            .mcp_server(server())
            .allow_mcp_tool("smllm", "smllm");
        for e in hook::EVENTS {
            i = i.hook(Hook::new(e, hook::COMMAND));
        }
        // Claude Code only: the adapter has no neutral item for these.
        if scope != Scope::Local {
            // sokf-implements: HARN-1_AC-7
            // sokf-implements: HARN-1_AC-6
            i = i.part(
                "claude",
                Part::files(
                    STATUSLINE,
                    ".claude/skills/smllm-statusline",
                    vec![("SKILL.md".into(), statusline::SKILL.into())],
                ),
            );
        }
        if scope == Scope::User {
            i = i.part(
                "claude",
                Part::external(
                    MCP_USER,
                    Arc::new(ClaudeMcpUser {
                        claude_json: self.home.join(".claude.json"),
                    }),
                ),
            );
        }
        i
    }

    fn root(&self, scope: Scope) -> agent_harness_adapter_core::Result<PathBuf> {
        Ok(match scope {
            Scope::User => self.home.clone(),
            _ => self.project.clone(),
        })
    }

    fn record_path(&self, scope: Scope) -> agent_harness_adapter_core::Result<PathBuf> {
        Ok(match scope {
            Scope::User => self.user_config.join("harness.toml"),
            Scope::Local => self.project.join(PROJECT_DIR).join("harness.local.toml"),
            _ => self.project.join(PROJECT_DIR).join("harness.toml"),
        })
    }

    fn declined_store(
        &self,
        scope: Scope,
    ) -> agent_harness_adapter_core::Result<Box<dyn DeclinedStore + '_>> {
        Ok(Box::new(match scope {
            Scope::User => TomlDeclined::new(
                self.user_config.join(CONFIG_FILE),
                "~/.config/smllm/config.toml",
            ),
            _ => TomlDeclined::new(
                self.project.join(PROJECT_DIR).join(CONFIG_FILE),
                ".smllm/config.toml",
            ),
        }))
    }
}

/// The harness ids `names` stands for: `all` is every built-in harness with
/// files at `scope`.
// sokf-implements: HARN-1_AC-3
pub fn harness_ids(names: &[String], scope: Scope) -> Vec<String> {
    if names.iter().any(|n| n == ALL) {
        return harness::builtin()
            .iter()
            .filter(|h| h.scopes().contains(&scope))
            .map(|h| h.id().to_string())
            .collect();
    }
    names.to_vec()
}

/// `smllm harness install|status`.
// sokf-implements: HARN-1_AC-2
// sokf-implements: HARN-1_AC-1
pub fn run(args: &HarnessArgs, installing: bool, explicit: Option<&Path>) -> Result<u8> {
    let cwd = paths::cwd()?;
    let tool = Smllm::new(&cwd, explicit)?;
    let scope: Scope = args.scope.parse()?;
    let ids = harness_ids(&args.names, scope);
    let result = if installing {
        let mut opts = InstallOptions::new(&ids, scope).force(args.force);
        if !args.without.is_empty() {
            opts = opts.without(args.without.iter().cloned());
        }
        install(&tool, &opts)?
    } else {
        status(&tool, &ids, scope)?
    };
    if args.json {
        output::json(&result)?;
    } else {
        output::text(&result.to_text())?;
        let edited = result
            .harnesses
            .iter()
            .flat_map(|h| &h.parts)
            .any(|p| p.state == State::Edited && p.action.is_none());
        if edited && !args.force {
            eprintln!(
                "note: parts marked edited were changed by hand, so smllm left them alone; \
                 to overwrite them, run `smllm harness install {} --force`",
                args.names.join(" ")
            );
        }
        let skill = result
            .harness("claude")
            .and_then(|h| h.parts.iter().find(|p| p.part == STATUSLINE));
        if skill.is_some_and(|p| p.state != State::Skipped)
            && let Some(note) = statusline::hint(&tool.settings(scope), &tool.home)
        {
            eprintln!("{note}");
        }
    }
    Ok(EXIT_OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: HARN-2_AC-2
    // sokf-tests: AGT-4_AC-2
    #[test]
    fn agent_facing_rules_are_a_stable_contract() {
        insta::assert_snapshot!("instructions_block", INSTRUCTIONS_BLOCK);
        insta::assert_snapshot!("tool_description", crate::commands::mcp::description());
    }

    // sokf-tests: HARN-1_AC-3
    #[test]
    fn all_is_every_harness_with_the_scope() {
        let all = vec![ALL.to_string()];
        assert_eq!(
            harness_ids(&all, Scope::Project),
            [
                "claude", "codex", "factory", "gemini", "copilot", "cursor", "pi", "opencode",
                "kilo", "qwen", "devin", "agents"
            ]
        );
        assert_eq!(harness_ids(&all, Scope::Local), ["claude", "copilot"]);
        let some = vec!["codex".to_string(), "claude".to_string()];
        assert_eq!(harness_ids(&some, Scope::User), some);
    }
}
