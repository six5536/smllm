# Examples

These examples are state machines that you can read, check, and copy into your own projects.
They pick up where the [tutorials](../docs/tutorials/index.md) end. Each folder holds one
machine, a `config.toml` that lists it, and a README that says what the machine does.

The following table lists the examples, what each one shows, and the documentation that goes with
it:

| Example                           | What it shows                                                                                                            | Read with                                                                         |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- |
| [`issue-fix`](issue-fix/)         | A coding agent's workflow, from a GitHub issue to a merged pull request: `command` guards and actions, and a retry limit | [Check the agent's work](../docs/tutorials/check-the-agents-work.md)              |
| [`release`](release/)             | A chain of eventless checks, a retry limit, and a step that waits for a person's answer                                  | [Agent, host, and human steps](../docs/explanation/agent-host-and-human-steps.md) |
| [`ticket-triage`](ticket-triage/) | A machine that a host drives for a model without tools: a JSON param with a schema, host-only events, and `invoke`       | [The smllm-wasm README](../packages/smllm-wasm/README.md#host-driven-machines)    |
| [`wasm`](wasm/)                   | An agent loop in Node.js over `@six5536/smllm-wasm`, with real commands and file storage                                 | [Run the engine in Node.js](../docs/tutorials/run-the-engine-in-node.md)          |
| [`showcase`](showcase/)           | Every feature of the machine format in one machine, for looking up how a feature is written                              | [The machine file](../docs/reference/machine-file.md)                             |

To check an example, run `smllm validate` on its config from the root of the repository:

```sh
smllm validate examples/release/config.toml
```

To see an example's states and transitions, run `smllm graph` with its config:

```sh
smllm --config examples/release/config.toml graph release
```
