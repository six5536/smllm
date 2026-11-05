# smllm-format

smllm-format reads, checks, and compiles the state machine files of
[smllm](https://crates.io/crates/smllm). This README describes the file format and the crate's
main functions.

A machine file is an XState v5 machine config, written in YAML as a strict subset, with smllm's
own data under `meta`.

The crate has these main functions:

- `load_machine` parses and validates a file, and lowers it to the `smllm-core` model. It
  collects every finding, whether an error, a warning, or an info finding. Each finding has its
  file, line, YAML path, hint for a fix, and rule ID.
- `load_machine_from` loads a machine as `load_machine` does, but reads the machine file, its
  prompt files, and the JSON Schemas of its JSON params through a source that the host gives:
  `FileSource` reads the file system, and `MemorySource` holds the files in memory, for a host
  that keeps its machines elsewhere, such as in a database. A source implements
  `InstructionSource`, the trait through which the engine reads a prompt file at run time.
- `load_configs` combines the user's and the project's `config.toml` files. When both define the
  same machine ID, the project's wins.
- `json_schema` returns the JSON Schema of the format.
- `compile` returns the model as JSON for hosts of `smllm-wasm`. A JSON param's schema is in it,
  bundled with every file that the schema refers to, so the compiled machine holds no path.

## Licence

smllm-format is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.

## Third-party notices

smllm-format embeds the JSON Schema meta-schemas of draft-07 and draft 2020-12, which json-schema.org
publishes in the [json-schema-spec repository](https://github.com/json-schema-org/json-schema-spec),
to check the schema of each JSON param. The files are in `src/bundle/meta/`, with their `LICENSE`. They are under the BSD 3-Clause licence:

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
