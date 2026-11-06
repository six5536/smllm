---
type: Reference
id: reference-harnesses
title: Harnesses
description: The harnesses that smllm connects to, what `smllm harness install` writes for each, and the Claude Code plugin.
order: "20"
---

# Harnesses

This page lists what `smllm harness install` writes, then what each harness supports, then the Claude Code plugin. A harness is the program that runs the agent, such as Claude Code or Codex.

- [Install](#install)
- [What each harness supports](#what-each-harness-supports)
- [The Claude Code plugin](#the-claude-code-plugin)

## Install

`smllm harness install` connects smllm to one or more harnesses:

```sh
smllm harness install claude          # CLAUDE.md/AGENTS.md block, .mcp.json, hooks, permission,
                                      # status line skill
smllm harness install claude codex    # several at once; AGENTS.md and .mcp.json written once
smllm harness install all             # every harness: claude codex factory gemini copilot cursor pi opencode kilo qwen devin agents
smllm harness status all              # what is installed, part by part
```

`--scope` says where the integration goes: the project (`--scope project`, the default), your user (`--scope user`), or git-ignored local settings (`--scope local`, for Claude Code and Copilot only).

Each harness gets as much as it can take of three parts: the instructions block, the hooks (`smllm harness hook HARNESS EVENT`), and the `smllm` MCP server. smllm writes each part where the harness reads it. The report lists what a harness can't take (`unsupported`) and what you still have to do (`note`), for example trust the folder in Codex or Pi.

## What each harness supports

The hooks differ by harness:

- **Claude Code, Codex, Factory, Gemini CLI, Qwen Code, Devin, and Cursor**: smllm uses their own hooks. The agent gets the session when it starts, and smllm blocks a stop.
- **Pi, OpenCode, and Kilo Code**: these harnesses have no hooks config, so smllm installs a Pi extension or an OpenCode or Kilo plugin that runs the same hook command. The agent gets the session with the first prompt. A blocked stop works in OpenCode's terminal user interface (TUI) and server, but not in `opencode run`.
- **GitHub Copilot**: Copilot gets the tool and the instructions. smllm can't show Copilot the session, so its hooks stay quiet: the agent starts with `enter`, and smllm doesn't block a stop.
- **`agents`**: any other agent that reads `AGENTS.md` gets only the instructions block, with no hooks and no MCP server. The agent fires events with `smllm fire` from the shell.

## The Claude Code plugin

Claude Code can also take smllm as a plugin:

```sh
claude plugin marketplace add six5536/smllm && claude plugin install smllm@smllm
```

Use either the plugin or `smllm harness install claude`, not both, because with both the hooks and the MCP server run twice. With the plugin, the tool is named `mcp__plugin_smllm_smllm__smllm`. `harness install` approves the tool in advance in the project settings, and with the plugin, Claude Code asks for permission the first time instead.
