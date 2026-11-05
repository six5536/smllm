# smllm

smllm runs state machines for large language model (LLM) coding agents. This README helps you
install smllm from npm and run its first commands. It covers installation, usage, where to find
the full documentation, and the licence.

You describe a workflow in YAML, as a subset of XState v5, and smllm drives the agent through it.
smllm gives the agent each state's instructions. When the agent tries to stop, smllm offers the
events that the agent can fire through one Model Context Protocol (MCP) tool. smllm works with
Claude Code, Codex, Gemini CLI, Cursor, and other agent harnesses.

- [Install](#install)
- [Usage](#usage)
- [Documentation](#documentation)
- [Licence](#licence)
- [Third-party notices](#third-party-notices)

## Install

Install the package globally:

```sh
npm install -g @six5536/smllm
```

This package is a thin launcher. It declares a prebuilt binary for each supported platform as an
`optionalDependency`, and npm installs only the one that matches your machine. A small JavaScript
shim then runs the binary.

The package supports Linux and macOS on `x64` and `arm64`, and Windows on `x64`. On any other
platform, or to build from source, install smllm with the Rust toolchain instead:

```sh
cargo install smllm
```

## Usage

These commands create a config, scaffold a state machine, check it, and connect smllm to an
agent:

```sh
smllm init                          # .smllm/config.toml
smllm new dev --write               # scaffold a state machine
smllm validate                      # findings as file:line: level: message
smllm harness install claude        # hooks + MCP tool for Claude Code (or codex, gemini, cursor, …, all)
```

smllm exits with `0` on success, `1` when it finds errors (config errors, a rejected event, or a
failed hook), and `2` on a usage or internal error.

## Documentation

The [smllm repository on GitHub](https://github.com/six5536/smllm#readme) has the full usage
guide and the notes on development.

## Licence

smllm is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.

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
