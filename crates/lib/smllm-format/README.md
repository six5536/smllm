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
- `load_configs` combines the user's and the project's `config.toml` files. When both define the
  same machine ID, the project's wins.
- `json_schema` returns the JSON Schema of the format.
- `compile` returns the model as JSON for hosts of `smllm-wasm`.

## Licence

smllm-format is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.
