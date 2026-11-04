# smllm

[![crates.io](https://img.shields.io/crates/v/smllm.svg)](https://crates.io/crates/smllm)
[![npm](https://img.shields.io/npm/v/@six5536/smllm.svg)](https://www.npmjs.com/package/@six5536/smllm)
[![docs.rs](https://img.shields.io/docsrs/smllm-core)](https://docs.rs/smllm-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

smllm puts a state machine in charge of the turn loop of a large language model (LLM) coding
agent. This README helps you install smllm, connect it to your agents, and write your first state
machine. It lists smllm's features, shows its commands and an example machine, and describes the
layout of the repository.

You describe a workflow as a state machine in YAML, written as a strict subset of an
[XState v5](https://stately.ai/docs/xstate) machine config. When the agent enters a state, smllm
gives it that state's instructions. When the agent tries to stop, smllm shows the events that it
can fire, and the agent picks one with a single Model Context Protocol (MCP) tool call. A state
machine can also choose its next state without the agent. A guarded transition or an eventless
`always` state runs a command such as `cargo test`, and the result chooses the next state. smllm
works with any harness, the program that runs the agent. It plugs into Claude Code,
OpenAI Codex, Factory Droid, Gemini CLI, GitHub Copilot, Cursor, Pi, OpenCode, Kilo Code, Qwen
Code, Devin, and any agent that reads `AGENTS.md`.

- [Features](#features)
- [Install](#install)
- [Usage](#usage)
- [Project layout](#project-layout)
- [Development](#development)
- [Licence](#licence)

## Features

- One MCP tool, `smllm({ session, event?, params? })`. Called with no event, it tells the agent
  where it is.
- Hooks inject the current state when a session starts. They block the agent from stopping, and
  show it the list of events, until it fires an event or `yield`. Where a harness can't inject
  context or block a stop, the agent goes on and uses the tool alone.
- Instances: each piece of work, such as an issue, a document, or a plan, moves through a state
  machine. An instance gets a generated ID and an optional ref, such as `GH-123`, that is set
  once. You can pause an instance, detour from it, resume it, reopen it, or let another session
  take it over.
- Built-in events: `enter`, `resume`, `pause`, `unmatched` (a detour), `yield`, and `listPaused`. The idle list shows the 10 most recently updated paused instances, and `listPaused` lists them all.
- Guards (`command`, `visits`) and actions (`prompt`, `command`, `setRef`), in XState's
  `{type, params}` form. A command receives its input only through environment variables.
- `smllm statusline` shows the current machine, state, and instance as a coloured row in Claude
  Code's status line, or as JSON for a row of your own. To set it up, ask Claude to "add smllm to
  my status line". The [status line guide](docs/statusline.md) has the details.
- `smllm validate` reports each finding as `file:line`, and ends it with the code of the rule that
  it breaks. [The rules of smllm validate](docs/validate-rules.md) says what each rule checks.
- `smllm graph` prints text, JSON, or Mermaid. `smllm info schema` prints a JSON Schema for
  completion in your editor.
- `@six5536/smllm-wasm` runs the engine in a browser or a Node host, from the output of
  `smllm compile`.

## Install

Install smllm from npm or from crates.io:

```sh
npm install -g @six5536/smllm   # prebuilt binary, Linux/macOS/Windows
cargo install smllm             # from crates.io, needs a Rust toolchain
```

Then connect smllm to your agents. The `--scope` flag says where the integration goes: the project
(`--scope project`, the default), your user (`--scope user`), or git-ignored local settings
(`--scope local`, for Claude Code and Copilot only):

```sh
smllm harness install claude          # CLAUDE.md/AGENTS.md block, .mcp.json, hooks, permission,
                                      # status line skill
smllm harness install claude codex    # several at once; AGENTS.md and .mcp.json written once
smllm harness install all             # every harness: claude codex factory gemini copilot cursor pi opencode kilo qwen devin agents
smllm harness status all              # what is installed, part by part
```

Each harness gets as much as it can take of three parts: the instructions block, the hooks
(`smllm harness hook <harness> <event>`), and the `smllm` MCP server. smllm writes each part where
the harness reads it. The report lists what a harness can't take (`unsupported`) and what you
still have to do (`note`), for example trust the folder in Codex or Pi.

The hooks differ by harness:

- Claude Code, Codex, Factory, Gemini CLI, Qwen Code, Devin, and Cursor use their own hooks. The
  context at session start and the blocked stop both work.
- Pi, OpenCode, and Kilo Code have no hooks config. For them, smllm installs a Pi extension or an
  OpenCode or Kilo plugin that runs the same hook command. They receive the session with the
  first prompt. A blocked stop works in OpenCode's terminal user interface (TUI) and server, but
  not in `opencode run`.
- Copilot gets the tool and the instructions. smllm can't show Copilot the session, so its hooks
  stay quiet: the agent starts with `enter`, and stopping isn't blocked.
- `agents`, any other agent that reads `AGENTS.md`, gets only the instructions block, with no
  hooks and no MCP server. The agent fires events with `smllm fire` from the shell.

Claude Code can also take smllm as a plugin:

```sh
claude plugin marketplace add six5536/smllm && claude plugin install smllm@smllm
```

Use either the plugin or `harness install claude`, not both, because with both the hooks and the
MCP server run twice. With the plugin, the tool is named `mcp__plugin_smllm_smllm__smllm`.
`harness install` approves the tool in advance in the project settings. With the plugin, Claude
Code asks for permission the first time instead.

## Usage

These commands create a config, scaffold a state machine, check it, and draw it:

```sh
smllm init                  # .smllm/config.toml
smllm new dev --write       # .smllm/dev.smllm.yaml, listed in the config
smllm validate --warnings   # findings as file:line: level: message (rule)
smllm graph dev --mermaid
```

Here is a state machine. The [`examples/` folder](examples/) has the full showcase.

```yaml
id: dev
initial: TRIAGE
meta:
  smllm: 1
  instance: { kind: issue, ref: { param: issueId, pattern: "^GH-\\d+$" } }
states:
  TRIAGE:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { file: triage.md } }
    on:
      issueCreated: { target: WORK, actions: setRef }
  WORK:
    on:
      submit:
        - guard: { type: command, params: { run: "cargo test --quiet" } }
          target: DONE
        - target: WORK
          reenter: true
  DONE:
    type: final
```

When the agent tries to stop in `WORK`, it sees this:

```
<smllm>
session sm-k7f3q2 · dev › WORK (visit 2) · issue GH-123
Fire one event: smllm({ session: "sm-k7f3q2", event, params })
<events>
- submit
- yield — Stop for now and stay in WORK.
    note (optional): What you are waiting for.
- pause — Put issue GH-123 aside and return to idle.
- unmatched — The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

smllm has these other commands:

- `smllm fire --session KEY EVENT --param k=v` fires an event from a shell, as the tool does.
- `smllm statusline [--json]` prints the status line row, as the
  [status line guide](docs/statusline.md) describes.
- `smllm session list|show` and `smllm instance list|show` list sessions and instances, or show
  one.
- `smllm harness status <harness>…` shows what is installed for each harness.
- `smllm compile` writes the validated model as JSON for `@six5536/smllm-wasm`.
- `smllm completions <shell>` prints a shell completion script.

In these commands, `KEY` is a session key, `EVENT` is the name of an event, and `k=v` gives the
parameter `k` the value `v`. `<harness>` is the name of a harness, such as `claude`, and `<shell>`
is the name of a shell.

smllm exits with `0` on success, `1` when it finds errors (config errors, a rejected event, or a
failed hook), and `2` on a usage or internal error. `smllm statusline` always exits with `0`, so
that it never blanks a status line. Errors go to stderr with the prefix `error: `, and `--json`
prints exactly one object.

smllm reads `.smllm/config.toml` in the project and combines it with
`~/.config/smllm/config.toml`. When both define the same state machine ID, the project's wins.
Instances live in `.smllm/state/`, which is git-ignored. Sessions live in `~/.local/state/smllm/`,
and a copy of each load of your machines, which is safe to delete, lives in `~/.cache/smllm/`.

## Project layout

The repository holds these crates, packages, and folders:

```
crates/lib/smllm-core         the engine: no_std + alloc, host traits for IO (builds for wasm)
crates/lib/smllm-format       YAML/TOML loading, validation, JSON Schema, compile
crates/lib/smllm-json         no_std JSON reader/writer for the wasm build (serde_json-compatible)
crates/lib/smllm-wasm         wasm-bindgen bindings → packages/smllm-wasm (npm)
crates/app/smllm              the CLI: commands, file store, command runner, hooks, MCP server
packages/smllm*               npm launcher + prebuilt-binary packages
plugin/                       the Claude Code plugin (marketplace in .claude-plugin/)
docs/                         user docs (status line)
examples/                     showcase + dev state machines
```

## Development

The project pins its toolchains in `.mise.toml` and `rust-toolchain.toml`, which
[mise](https://mise.jdx.dev/) manages.


These npm scripts build and check the project:

```sh
npm run build           # cargo build --workspace
npm run build:wasm      # packages/smllm-wasm (wasm-bindgen + wasm-opt, size budget)
npm run lint            # cargo clippy --workspace
npm run verify-version  # check that every version in the tree agrees
```

This repository is a read-only mirror. [CONTRIBUTING.md](CONTRIBUTING.md) explains how to report a
bug or ask for a feature, and how to build the packages of a release.


## Licence

smllm is licensed under the MIT licence. The [LICENSE](LICENSE) file has the full text.
