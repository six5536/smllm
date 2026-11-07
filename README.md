# smllm

[![crates.io](https://img.shields.io/crates/v/smllm.svg)](https://crates.io/crates/smllm)
[![npm](https://img.shields.io/npm/v/@six5536/smllm.svg)](https://www.npmjs.com/package/@six5536/smllm)
[![docs.rs](https://img.shields.io/docsrs/smllm-core)](https://docs.rs/smllm-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

smllm guides an LLM agent through a workflow. You write the workflow as a state machine in YAML.
smllm shows the agent one state at a time, and the agent moves on by firing one of the state's
events through a single MCP tool. When the agent tries to stop before it has finished, smllm
shows it the events that it can fire instead.

- [Features](#features)
- [Install](#install)
- [Quick start](#quick-start)
- [Documentation](#documentation)
- [Project layout](#project-layout)
- [Development](#development)
- [Licence](#licence)
- [Third-party notices](#third-party-notices)

## Features

- **State machines in YAML**: a strict subset of [XState v5](https://stately.ai/docs/xstate),
  which `smllm validate` checks before an agent runs it.
- **One step at a time**: the agent sees only the instructions of its current state.
- **Stops that ask for a decision**: when the agent tries to stop in a state that isn't final,
  smllm shows it the events that it can fire, `yield` to wait for you, or `pause`.
- **Checks that the agent can't skip**: a guard runs a command, such as a check of the agent's
  work, and its result chooses the next state.
- **Work that outlasts a session**: each piece of work is an instance that you can pause,
  resume, reopen, or hand to another session.
- **Many harnesses**: Claude Code, Codex, Cursor, Gemini CLI, GitHub Copilot, and six more,
  and any agent that reads `AGENTS.md`.
- **A view of the work**: a row in Claude Code's status line, a diagram of each machine in your
  browser, and an audit of every call.
- **An engine to embed**: `@six5536/smllm-wasm` runs the same engine in a browser or Node.js.

## Install

Install smllm from npm, which brings a prebuilt binary for Linux, macOS, and Windows, or from
crates.io:

```sh
npm install -g @six5536/smllm
cargo install smllm
```

Then connect smllm to your harness, the program that runs your agent:

```sh
smllm harness install claude
```

[Harnesses](docs/reference/harnesses.md) lists the other harnesses and what each supports.
Claude Code can also take smllm as a plugin. Use either the plugin or `harness install`, not
both.

## Quick start

These commands create a config, scaffold a state machine, check it, and draw it:

```sh
smllm init                  # .smllm/config.toml
smllm new dev --write       # .smllm/dev.smllm.yaml, listed in the config
smllm validate --warnings   # findings as file:line: level: message (rule)
smllm graph dev --open      # the diagram in your web browser
```

This machine takes an issue from triage to a passing test suite. `submit` leads to `DONE` only
when `cargo test` passes:

**`.smllm/dev.smllm.yaml`**

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

The prompt of `TRIAGE` is a file beside the machine:

**`.smllm/triage.md`**

```markdown
Read the issue, reproduce the bug, and fire `issueCreated` with the issue's number.
```

The config lists the machine:

**`.smllm/config.toml`**

```toml
[machines]
files = ["dev.smllm.yaml"]
```

When the agent tries to stop in `WORK`, it sees this:

<!-- example:
  fire enter stateMachine=dev
  fire issueCreated issueId=GH-123
  hook stop
-->
```
<smllm>
session sm-nqvfvw · dev › WORK · issue GH-123
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- submit
- yield: Stop for now and stay in WORK.
    note (optional): What you are waiting for.
- pause: Put issue GH-123 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

The [`examples/` folder](examples/) has more machines, with JSON params, host events, and
every kind of guard and action.

## Documentation

The documentation is on [its website](https://six5536.github.io/smllm/), and in
[`docs/`](docs/welcome.md):

- [Your first state machine](docs/tutorials/first-machine.md) teaches smllm step by step.
- [Connect an agent](docs/tutorials/connect-an-agent.md) installs smllm into Claude Code and
  follows the agent through a machine.
- [Check the agent's work](docs/tutorials/check-the-agents-work.md) adds a check that the agent
  can't skip.
- [Run the engine in Node.js](docs/tutorials/run-the-engine-in-node.md) drives a machine from
  your own program with `@six5536/smllm-wasm`.
- [Show smllm in your status line](docs/how-to/show-smllm-in-the-status-line.md) sets up the
  status line row.
- [The command](docs/reference/cli.md) lists every command, its exit codes, and the files that
  smllm keeps.
- [The rules of smllm validate](docs/reference/validate-rules.md) says what each rule checks,
  and [`docs/reference/errors/`](docs/reference/errors/) gives the cause and the fix of the
  findings of each code.
- [Why a state machine](docs/explanation/why-a-state-machine.md) explains the idea behind smllm.

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
docs/                         user documentation: tutorials, how-to guides, reference, explanations
examples/                     example state machines, each with a README
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

## Third-party notices

The smllm binary embeds the JSON Schema meta-schemas of draft-07 and draft 2020-12, which json-schema.org
publishes in the [json-schema-spec repository](https://github.com/json-schema-org/json-schema-spec),
to check the schema of each JSON param. They are under the BSD 3-Clause licence:

```text
Copyright (c) 2022 JSON Schema Specification Authors

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

1. Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright
notice, this list of conditions and the following disclaimer in the
documentation and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
contributors may be used to endorse or promote products derived from
this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```
