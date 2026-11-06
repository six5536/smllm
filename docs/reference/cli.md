---
type: Reference
id: reference-cli
title: The command
description: The commands of `smllm`, the forms of their output, their exit codes, and the files that smllm reads and writes.
order: "10"
---

# The command

This page lists the commands of `smllm`, then the forms of their output and their exit codes, then the files that smllm reads and writes. [Commands and options][sokf:reference-commands] lists the options of each command, as `smllm COMMAND --help` prints them.

- [Commands](#commands)
- [Output and exit codes](#output-and-exit-codes)
- [Files](#files)

## Commands

smllm has these commands:

- `smllm init [--user]` creates `.smllm/config.toml` in the project, or with `--user` the user config `~/.config/smllm/config.toml`.
- `smllm new ID [--write]` prints a template state machine with the ID `ID`. With `--write`, it writes `ID.smllm.yaml` beside the config and lists it in the config.
- `smllm validate [PATHS]…` checks the configs, their state machines, and the saved instances, or only the files that `PATHS` names. It prints each finding as `file:line: level: message (CODE)`, where `CODE` is the rule that the finding breaks, as [The rules of smllm validate][sokf:reference-validate-rules] lists. `--warnings` and `--info` list the warnings and the info findings too.
- `smllm graph [ID]` prints the states and transitions of a state machine, or of all of them, as text. `--json` prints JSON, and `--mermaid` a Mermaid state diagram. `--open` opens the machine as a diagram in your web browser, where you select a state or a transition to read its prompts, commands, guards, and events. `--html FILE` writes the same page to a file, and `--png FILE` writes the diagram as a picture. With several machines, `--png m.png` writes `m-ID.png` for each machine `ID`.
- `smllm fire --session KEY EVENT [--param k=v]…` fires an event from a shell, as the agent does through the smllm tool.
- `smllm statusline [--json]` prints where a session is, as [The status line row][sokf:reference-statusline] describes.
- `smllm session list` and `smllm session show KEY` list the sessions, or show where one is, as the smllm tool does with no event. `smllm session show KEY --audit [--json]` prints the session's audit: one line for each call that drove the session, accepted or refused, with its time, its source (`agent` or `host`), the call, the state, and the outcome.
- `smllm instance list` and `smllm instance show ID` list the instances of the configured state machines, or show one, by its ID or its ref, with its history.
- `smllm harness install HARNESS…` and `smllm harness status HARNESS…` install smllm into each harness, or show what is installed, as [Harnesses][sokf:reference-harnesses] describes.
- `smllm info schema` prints the JSON Schema of the state machine format, for completion in your editor, and `smllm info rules` prints the rules page.
- `smllm compile FILE` writes a config or a state machine as JSON for `@six5536/smllm-wasm`.
- `smllm completions SHELL` prints a completion script for `bash`, `elvish`, `fish`, `powershell`, or `zsh`.

In these commands, `KEY` is a session key, `EVENT` is the name of an event, and `k=v` gives the param `k` the value `v`. A JSON param reads `v` as JSON text, such as `plan={"steps":[]}`.

## Output and exit codes

smllm exits with one of three codes:

- `0` on success.
- `1` when it finds errors: an error in a config, a rejected event, or a failed hook.
- `2` on a usage error or an internal error.

`smllm statusline` always exits with `0`, so that it never blanks a status line. Errors go to standard error with the prefix `error: `. With `--json`, a command prints exactly one JSON object, except for an audit, which it prints as JSON lines.

## Files

smllm reads and writes these files:

- **Configs**: `.smllm/config.toml` in the project and `~/.config/smllm/config.toml`, which smllm combines. When both define a state machine with the same ID, the project's wins.
- **Instances**: `.smllm/state/` in the project, which git ignores.
- **Sessions**: `~/.local/state/smllm/`.
- **Audits**: `audit/KEY.jsonl` in the folder of the sessions, one for each session, which smllm keeps until you remove it. Each line holds the hash of the reply that the agent was shown, as `sha256:` and the hex of the SHA-256 of the reply's UTF-8 text, not the text itself.
- **Cache**: `~/.cache/smllm/`, a copy of each load of your machines, which is safe to delete.

<!-- sokf:links -->

[sokf:reference-commands]: /docs/reference/commands.md
[sokf:reference-harnesses]: /docs/reference/harnesses.md
[sokf:reference-statusline]: /docs/reference/statusline.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
