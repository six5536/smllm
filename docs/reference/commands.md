---
type: Reference
id: reference-commands
title: Commands and options
description: Each command of smllm, with its arguments and its options, as the command's own help gives them.
order: "15"
generator: man-markdown
---

# Commands and options

This page lists each command of `smllm`, with its arguments and its options, as `smllm COMMAND --help` prints them.

The commands:

- [`smllm`](#smllm)
- [`smllm init`](#smllm-init)
- [`smllm new`](#smllm-new)
- [`smllm validate`](#smllm-validate)
- [`smllm fire`](#smllm-fire)
- [`smllm session`](#smllm-session)
- [`smllm session list`](#smllm-session-list)
- [`smllm session show`](#smllm-session-show)
- [`smllm instance`](#smllm-instance)
- [`smllm instance list`](#smllm-instance-list)
- [`smllm instance show`](#smllm-instance-show)
- [`smllm harness`](#smllm-harness)
- [`smllm harness install`](#smllm-harness-install)
- [`smllm harness status`](#smllm-harness-status)
- [`smllm graph`](#smllm-graph)
- [`smllm info`](#smllm-info)
- [`smllm info schema`](#smllm-info-schema)
- [`smllm info rules`](#smllm-info-rules)
- [`smllm compile`](#smllm-compile)
- [`smllm statusline`](#smllm-statusline)
- [`smllm completions`](#smllm-completions)

## `smllm`

State machines for LLM agents: declarative XState-style machines that drive an agent's turn loop through its hooks and one MCP tool.

**Usage:** `smllm [OPTIONS] [COMMAND]`

Exit codes: 0 success, 1 errors found (a config error, a rejected event, or a failed hook), 2 a usage or internal error.

Documentation: <https://github.com/six5536/smllm>

### Subcommands

- `init` — Create .smllm/config.toml in this directory, or the user config.
- `new` — Print a template state machine, or write it to `<ID>.smllm.yaml`.
- `validate` — Check the configs, their state machines, and the saved instances.
- `fire` — Fire an event from the shell, as the smllm tool does.
- `session` — List sessions, or show where one is.
- `instance` — List instances, or show one with its history.
- `harness` — Install or check a harness integration (Claude Code, Codex, Cursor, and others).
- `graph` — Print the states and transitions of the state machines.
- `info` — Print reference information.
- `compile` — Compile a config or state machine to compact JSON, for smllm-wasm hosts.
- `statusline` — Print where a session is, as a status line row or as JSON.
- `completions` — Print a shell completion script to stdout.

### Options

- `--config <FILE>` — Use only this config file (SMLLM_CONFIG also sets it).
- `-V`, `--version` — Print the version and exit.

## `smllm init`

Create .smllm/config.toml in this directory, or the user config

**Usage:** `smllm init [OPTIONS]`

### Options

- `--user` — Create the user config (~/.config/smllm/config.toml) instead.
- `--json` — Print JSON.

## `smllm new`

Print a template state machine, or write it to `<ID>.smllm.yaml`

**Usage:** `smllm new [OPTIONS] <ID>`

### Arguments

- `<ID>` — The state machine id.

### Options

- `--dir <DIR>` — The directory to write it to (default: the directory of the config).
- `--write` — Write the file and list it in the config, instead of printing it.
- `--json` — Print JSON.

## `smllm validate`

Check the configs, their state machines, and the saved instances

**Usage:** `smllm validate [OPTIONS] [PATHS]...`

### Arguments

- `<PATHS>` — Config (.toml) or state machine (.yaml) files (default: the configs found).

### Options

- `--json` — Print JSON.
- `--warnings` — List the warnings too.
- `--info` — List the info findings too.

## `smllm fire`

Fire an event from the shell, as the smllm tool does

**Usage:** `smllm fire [OPTIONS] <EVENT>`

### Arguments

- `<EVENT>` — The event to fire.

### Options

- `--session <KEY>` — The session key (omit it only to start a session with `enter`).
- `--param <KEY=VALUE>` — A param of the event (repeatable).
- `--json` — Print JSON.

## `smllm session`

List sessions, or show where one is

**Usage:** `smllm session <COMMAND>`

### Subcommands

- `list` — List sessions, the most recently active first.
- `show` — Show where a session is, as the smllm tool does with no event, or with --audit every call that drove it.

## `smllm session list`

List sessions, the most recently active first

**Usage:** `smllm session list [OPTIONS]`

### Options

- `--json` — Print JSON.

## `smllm session show`

Show where a session is, as the smllm tool does with no event, or with --audit every call that drove it

**Usage:** `smllm session show [OPTIONS] <KEY>`

### Arguments

- `<KEY>` — The session key.

### Options

- `--audit` — Print the session's audit: one line for each call, or with --json the audit file's JSON lines.
- `--json` — Print JSON.

## `smllm instance`

List instances, or show one with its history

**Usage:** `smllm instance <COMMAND>`

### Subcommands

- `list` — List instances of the configured state machines.
- `show` — Show an instance, by id or ref, with its history.

## `smllm instance list`

List instances of the configured state machines

**Usage:** `smllm instance list [OPTIONS]`

### Options

- `--json` — Print JSON.

## `smllm instance show`

Show an instance, by id or ref, with its history

**Usage:** `smllm instance show [OPTIONS] <KEY>`

### Arguments

- `<KEY>` — The session key, or instance id or ref.

### Options

- `--json` — Print JSON.

## `smllm harness`

Install or check a harness integration (Claude Code, Codex, Cursor, and others)

**Usage:** `smllm harness <COMMAND>`

### Subcommands

- `install` — Install or update a harness integration.
- `status` — Show the state of each part of a harness integration.

## `smllm harness install`

Install or update a harness integration

**Usage:** `smllm harness install [OPTIONS] <HARNESS>...`

### Arguments

- `<HARNESS>` — The harnesses: claude, codex, factory, gemini, copilot, cursor, pi, opencode, kilo, qwen, devin, agents, or all.

### Options

- `--scope <S>` — The scope: project, user, or local (local for Claude Code and Copilot only).

  Default value: `project`
- `--without <PART>` — Leave out a part, such as instructions, hooks, mcp, permissions, or statusline (repeatable).
- `--force` — Overwrite parts edited by hand.
- `--json` — Print JSON.

## `smllm harness status`

Show the state of each part of a harness integration

**Usage:** `smllm harness status [OPTIONS] <HARNESS>...`

### Arguments

- `<HARNESS>` — The harnesses: claude, codex, factory, gemini, copilot, cursor, pi, opencode, kilo, qwen, devin, agents, or all.

### Options

- `--scope <S>` — The scope: project, user, or local (local for Claude Code and Copilot only).

  Default value: `project`
- `--without <PART>` — Leave out a part, such as instructions, hooks, mcp, permissions, or statusline (repeatable).
- `--force` — Overwrite parts edited by hand.
- `--json` — Print JSON.

## `smllm graph`

Print the states and transitions of the state machines

**Usage:** `smllm graph [OPTIONS] [ID]`

### Arguments

- `<ID>` — A state machine id (default: all of them).

### Options

- `--json` — Print JSON.
- `--mermaid` — Print a Mermaid stateDiagram-v2.
- `--open` — Open the interactive diagram in the default web browser (BROWSER sets the program).
- `--html <FILE>` — Write the interactive diagram to FILE, an HTML page that needs only a browser.
- `--png <FILE>` — Write the diagram as a PNG picture to FILE, or for several machines to STEM-ID.EXT beside it.

## `smllm info`

Print reference information

**Usage:** `smllm info <COMMAND>`

### Subcommands

- `schema` — Print the JSON Schema of the state machine format.
- `rules` — Print the rules that validate checks, with the code that each finding ends with, as Markdown.

## `smllm info schema`

Print the JSON Schema of the state machine format

**Usage:** `smllm info schema`

## `smllm info rules`

Print the rules that validate checks, with the code that each finding ends with, as Markdown

**Usage:** `smllm info rules`

## `smllm compile`

Compile a config or state machine to compact JSON, for smllm-wasm hosts

**Usage:** `smllm compile [OPTIONS] <FILE>`

### Arguments

- `<FILE>` — A config.toml or a state machine file.

### Options

- `-o`, `--out <OUT>` — The file to write to, instead of stdout.

## `smllm statusline`

Print where a session is, as a status line row or as JSON.

The statusline command reads Claude Code's status JSON on stdin and shows the session bound to its `session_id`. It prints nothing when there is nothing to show. It always exits 0, because a non-zero exit blanks Claude Code's whole status line.

**Usage:** `smllm statusline [OPTIONS]`

### Options

- `--session <KEY>` — The session key (default: the session bound to the `session_id` of the status JSON on stdin).
- `--json` — Print JSON (`{}` when there is nothing to show).
- `--color <WHEN>` — When to colour the row (default: auto).

  Possible values:
  - `auto`:
    Colour, unless NO_COLOR is set and not empty
  - `always`:
    Colour
  - `never`:
    No colour

## `smllm completions`

Print a shell completion script to stdout

**Usage:** `smllm completions <SHELL>`

### Arguments

- `<SHELL>` — The shell to print a completion script for.

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`

