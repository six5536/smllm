# smllm-json

smllm-json is a small `no_std` JSON reader and writer. It is built for the WebAssembly engine of
[smllm](https://crates.io/crates/smllm), where every kilobyte of the build counts. This README
describes what the crate reads and writes.

The crate has these parts:

- `parse` reads RFC 8259 JSON into a `JsonValue` tree. It follows the grammar strictly, limits
  the nesting depth, and has no floating-point code. Integers are `i64` or `u64`, and smllm-json
  keeps any other number as its text.
- `FromJson` and `Fields` read typed values out of the tree, with serde's rules for struct
  fields. An error names the field that failed, for example
  `machines[0].id: expected a string, found an integer`.
- `ToJson` and `object` write JSON, escaped exactly as serde_json escapes it. For the same shapes,
  the output is byte for byte the same as serde_json's.

The crate has no dependencies.

## Licence

smllm-json is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.
