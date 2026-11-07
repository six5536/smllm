# Ticket triage example

This example is a state machine that a host drives for a model without tools. It helps you write
such a machine, and the [smllm-wasm README](../../packages/smllm-wasm/README.md#host-driven-machines)
shows the host's loop over it.

For each support ticket, the model answers with a triage: a priority, and one or more labels, each
with the words of the ticket that support it. The host checks the triage, and the model tries
again with the errors, up to five attempts. The model has no tools, so a ticket that holds text
written to steer the model can at worst cause a bad triage, which the checks catch.

The host's checks are its own code, outside the machine. A host might check these things:

- Each label's evidence appears word for word in the ticket, so the model invented no reason.
- A ticket that mentions a charge has the label `billing`.
- An `urgent` ticket has the label `bug` or `account`.

The example has these files:

- `ticket-triage.smllm.yaml` is the machine. In `READ`, the model answers with `triage` or
  `history`, and the host can fire `timed_out`. In `CHECKING`, which invokes the host's work
  `runChecks`, only the host fires: `checks_passed` goes to `ACCEPTED`, and `checks_failed` goes
  back to `READ`, or to `FAILED` after five visits of `READ`.
- `prompts/read.md` holds the instructions of `READ`.
- `schemas/triage.json` and `schemas/label.json` are the JSON Schema of the param `json` of
  `triage`. The loader bundles both files into the compiled machine.
- `config.toml` lists the machine, for `smllm validate` and `smllm compile`.

A valid answer in `READ` looks like this:

```json
{
  "event": "triage",
  "params": {
    "json": {
      "priority": "urgent",
      "labels": [{ "name": "billing", "evidence": "charged twice for March" }]
    }
  }
}
```

To check the machine and compile it for the wasm package, run these commands from the root of the
repository:

```sh
smllm validate examples/ticket-triage/config.toml
smllm compile examples/ticket-triage/config.toml -o machines.json
```
