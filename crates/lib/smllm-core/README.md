# smllm-core

smllm-core is the engine of [smllm](https://crates.io/crates/smllm). smllm runs state machines
that drive the turn loop of a large language model (LLM) agent. This README describes what the crate contains and
where to find its API documentation.

The crate is `no_std` and needs `alloc`. It has no IO, time, or randomness of its own. A host
supplies them through the traits in `smllm_core::host`: `Store`, `Guard`, `Action`,
`InstructionSource`, `Matcher`, `Clock`, and `Ids`. The same engine runs in the `smllm` CLI and,
through `smllm-wasm`, in a browser.

The crate has these parts:

- The methods `Engine::bind`, `view`, `menu`, `fire`, `stop`, and `prompt_submitted` make up the
  whole protocol.
- `model` holds the lowered state machine. `smllm-format` builds the model from YAML, and the
  `serde` feature deserialises the JSON output of `smllm compile`.
- `record` holds sessions, instances, and history entries.
- Every reply is `<smllm>` text for the agent, together with where the session ended up.

If you want the tool rather than the library, install [`smllm`](https://crates.io/crates/smllm).
The API isn't stable yet. The [API documentation](https://docs.rs/smllm-core) describes each
item.

## Licence

smllm-core is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.
