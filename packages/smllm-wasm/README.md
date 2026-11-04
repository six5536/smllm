# smllm-wasm

`@six5536/smllm-wasm` is smllm's engine, `smllm-core`, compiled to WebAssembly. It is for a host
that runs state machines in its own process, such as a browser, a worker, a serverless function,
or Node. The large language model (LLM) can run anywhere. The API is typed: it takes objects and returns
objects, and the package includes TypeScript declarations.

This README helps you run the engine in your own host. It shows a first example, then describes
the host methods, storage, the Node helpers, and when to use the WebAssembly engine instead of the
CLI.

- [Host](#host)
- [Storage](#storage)
- [Run in Node](#run-in-node)
- [WebAssembly or CLI](#webassembly-or-cli)
- [Licence](#licence)

This example loads compiled machines, starts a session, and passes the model's tool calls to the
engine:

```js
import init, { Engine, memoryStorage } from "@six5536/smllm-wasm"; // or from "@six5536/smllm-wasm/bundler"

await init(); // the bundler build needs no init
const compiled = await (await fetch("machines.json")).json(); // `smllm compile`
const engine = new Engine(
  compiled,
  {
    // Every host method is optional; supports / check / run add host guard and
    // action kinds (`command`), none by default.
  },
  memoryStorage(), // or your own Storage (see Storage); optional
);
const { session, text } = engine.bind("web");
// Give the model the same `smllm` tool the MCP server offers, and pass its
// calls through: a malformed call comes back as an answer, never a throw.
const tool = engine.tool(); // { name, description, inputSchema }
const reply = engine.callTool({ session, event: "enter", params: { stateMachine: "dev" } });
// When the model tries to stop (`stopHookActive`: it is already continuing
// because of an earlier block):
const stop = engine.stop(session, stopHookActive);
// stop.decision "block": hold the model, show it stop.text (the events list);
// "runaway": let it stop, show stop.text to the user; "allow": let it stop.
```

The `examples/wasm` folder of the [smllm repository](https://github.com/six5536/smllm) holds a
whole agent loop, in TypeScript.

## Host

A host can supply these methods. Each one is optional, and the engine uses its default when the
host leaves the method out:

- `supports(kind)` says which guard and action kinds the host runs. By default, it supports none.
- `check(kind, params, env, cwd)` checks a guard of a supported kind.
- `run(kind, params, env, cwd)` runs an action of a supported kind. It returns nothing on
  success, and the detail of the failure otherwise. A command's `params.cwd` is relative to the
  compiled config file.
- `read(file)` reads a prompt file. `smllm compile` inlines prompt files, so a host rarely needs
  this method.
- `isMatch(pattern, value)` matches a pattern. By default, it uses `new RegExp`.
- `now()` returns the time. By default, it uses `Date.now`.
- `random()` returns a random number. By default, it uses `crypto.getRandomValues`.
- `history(machine, id, entry)` and `put(kind, key, record)` receive writes. By default, the
  engine drops them.

A host method that throws is a host failure: a failed guard or action, or a rejected pattern. The
reply text shows the failure, and the engine stays usable.

## Storage

The engine works in memory and writes through to storage. It passes each session, binding, and
instance to `put` when it saves them, and each history entry to `history`. Persistence therefore
costs only what changed, never a snapshot for each call.

A `Storage` is `{ load(), put(kind, key, record), history(machine, id, entry) }`. The engine
takes a `Storage` as the constructor's third argument, loads it once, and then hands it every
write.
`memoryStorage()` keeps the state in memory. In a browser, you can write a `Storage` over
IndexedDB, or one that sends the records to a server. `exportState()` and `importState(state)`
move the whole state at once, for backups and moves.

## Run in Node

`@six5536/smllm-wasm/node` provides a host and a storage for Node:

```js
import { nodeFileStorage, nodeHost } from "@six5536/smllm-wasm/node";

const engine = new Engine(
  compiled,
  nodeHost({ configDir: "/path/to/the/compiled/config/folder" }),
  nodeFileStorage("/path/to/state"),
);
```

`nodeHost` runs `command` guards and actions as the smllm CLI does:

- It runs a string `run` through the shell, and a list without one.
- It sets the `SMLLM_*` environment variables.
- It applies a timeout, 60 seconds by default, and the timeout also kills the processes that the
  command started.
- On a failure, it reports the last 400 characters of output.

`nodeFileStorage` keeps one file for each record, written atomically, and keeps history as JSON Lines (JSONL).
Only one process can own a storage folder.

## WebAssembly or CLI

The following table gives the median time for each call. The project's benchmark measured the
times in a Linux container with a local disk. Each column heading gives the size of the store as the
number of paused instances and the number of completed instances.

| Path | Call | 0 / 0 | 100 / 1000 | 1000 / 1000 |
| ---- | ---- | ---: | ---: | ---: |
| wasm, in-process | `fire` enter / pause | 0.06 ms | 0.02 / 0.03 ms | 0.02 / 0.03 ms |
| wasm, in-process | view, stop | 0.02 ms | 0.006 ms | 0.006 ms |
| CLI, `smllm mcp` (one process, over stdio) | tool call | 0.2–0.4 ms | 0.1–0.6 ms | 0.1–2.3 ms |
| CLI, a process per call | `smllm fire`, `statusline` | 1.4–1.7 ms | 1.2–1.8 ms | 1.3–4.4 ms |

Use the WebAssembly engine in these cases:

- In a browser, a worker, or a serverless function, where there is no binary and no shell.
- For a model that answers in milliseconds.
- For guards and actions written in JavaScript.
- For storage of your own.

Use the CLI, from the `@six5536/smllm` npm package, when a Node app needs the CLI's behaviour as
it is. For tool calls, run `smllm mcp`. The CLI shares files between processes with locking, and
provides the status line, `smllm instance list`, and the Claude Code hooks.

The string API underneath, which takes and returns JSON text, is in `@six5536/smllm-wasm/raw` and
`@six5536/smllm-wasm/raw/bundler`.

## Licence

smllm-wasm is licensed under the MIT licence. Its source is in the
[smllm repository](https://github.com/six5536/smllm).
