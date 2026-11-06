# Extractor builder example

This example is a state machine that a host drives for a model without tools. It helps you write
such a machine, and the [smllm-wasm README](../../packages/smllm-wasm/README.md#host-driven-machines)
shows the host's loop over it.

For a bank statement in a format that the host hasn't seen, the model writes an extractor: a JSON
file of rules that turns the lines of the statement into items. The host runs the extractor and
checks the items, and the model tries again with the errors, up to five attempts. The model has
no tools, so a statement that holds text written to steer the model can at worst cause a bad
extractor, which the checks catch.

The example has these files:

- `extractor-build.smllm.yaml` is the machine. In `READ`, the model answers with `extractor` or
  `pages`, and the host can fire `timed_out`. In `CHECKING`, which invokes the host's work
  `runChecks`, only the host fires: `checks_passed` goes to `ACCEPTED`, and `checks_failed` goes
  back to `READ`, or to `FAILED` after five visits of `READ`.
- `prompts/read.md` holds the instructions of `READ`.
- `schemas/extractor.json` and `schemas/rule.json` are the JSON Schema of the param `json` of
  `extractor`. The loader bundles both files into the compiled machine.
- `config.toml` lists the machine, for `smllm validate` and `smllm compile`.

To check the machine and compile it for the wasm package, run these commands from the root of the
repository:

```sh
smllm validate examples/extractor/config.toml
smllm compile examples/extractor/config.toml -o machines.json
```
