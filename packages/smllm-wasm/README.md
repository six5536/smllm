# smllm-wasm

`@six5536/smllm-wasm` is smllm's engine, `smllm-core`, compiled to WebAssembly. It is for a host
that runs state machines in its own process, such as a browser, a worker, a serverless function,
or Node. The large language model (LLM) can run anywhere. The API is typed: it takes objects and returns
objects, and the package includes TypeScript declarations.

This README helps you run the engine in your own host. It shows a first example, then describes
the host methods, storage, the events that only the host fires, machines that the host drives for
a model without tools, the Node helpers, and when to use the WebAssembly engine instead of the CLI.

- [Host](#host)
- [Storage](#storage)
- [Host-only events](#host-only-events)
- [Host-driven machines](#host-driven-machines)
- [Run in Node](#run-in-node)
- [WebAssembly or CLI](#webassembly-or-cli)
- [Licence](#licence)

This example loads compiled machines, starts a session, and passes the model's tool calls to the
engine:

```js
import init, { Engine, memoryStorage } from "@six5536/smllm-wasm"; // or from "@six5536/smllm-wasm/bundler"

await init(); // in a browser or Node.js, while the bundler build needs no init
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

A param takes a string. A param that the machine declares with `type: json` takes any JSON value,
in `callTool`, `fire`, and `fireAsHost`, and the host receives its compact JSON text. A JSON string stays a string, even when its text spells an object. The events list shows
the param as a JSON value, with its JSON Schema.

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
- `checkJson(schema, value)` checks a `json` param's value against its schema, and returns one
  message for each error, such as `at /steps: [] has fewer than 1 item`, or none when the value
  passes. The engine refuses an event whose value has errors. The package has no validator of its
  own, so give it one of yours, such as Ajv's. Without `checkJson`, the engine takes the value
  unchecked, and `unsupported()` names each `json` param with a schema.
- `now()` returns the time. By default, it uses `Date.now`.
- `random()` returns a random number. By default, it uses `crypto.getRandomValues`.
- `history(machine, id, entry)` and `put(kind, key, record)` receive writes. By default, the
  engine drops them.
- `wantsAudit()` and `audit(entry, reply)` keep an audit, as the following section describes.

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

## Audit

An audit records every call that drove a session, accepted or refused: `bind`, `callTool`,
`events`, `fire`, `fireAsHost`, and `stop`. `view`, `status`, and `promptSubmitted` record
nothing. To keep an audit, give the storage or the host `wantsAudit()`, which returns true, and
`audit(entry, reply)`, which receives each `AuditEntry` and the text of the call's reply:

```ts
const storage = {
  ...memoryStorage(),
  wantsAudit: () => true,
  audit(entry: AuditEntry, reply: string) {
    console.log(entry.source, entry.call, entry.event, entry.outcome, entry.reason);
  },
};
```

An entry holds the time, the session, the source (`agent`, or `host` for `fireAsHost`), the call,
the instance with its states before and after, the params, the names of the events that the reply
offered, the outcome with the reason for a refusal, and for a call that created the session its
binding. Keep what you need of the reply's text. A hash names its algorithm, as `sha256:` and the
hex of the SHA-256 of the text's UTF-8 bytes, which is the default that the `smllm` CLI uses.

## Host-only events

A host-only event is an event that only the host fires, such as the result of checks that the host
runs outside the machine. A machine marks it with `source: host` in `meta.events`:

```yaml
meta:
  smllm: 1
  events:
    checksPassed: { source: host }
    checksFailed: { source: host }
states:
  WORK:
    on: { submit: CHECKING }
  CHECKING:
    on: { checksPassed: DONE, checksFailed: WORK }
  DONE: { type: final }
```

The agent never sees a host-only event: the events lists of the tool's replies, of `view`, of
`events`, and of `stop` leave it out. `fire` and `callTool` refuse it with an error that names it
as an event that only the host fires. The host fires it with `fireAsHost`, which takes the same
arguments as `fire` and fires any event that the state offers:

```js
engine.callTool({ session, event: "submit" }); // the agent: WORK → CHECKING
const passed = await runChecks(); // your own checks, which can be async
engine.fireAsHost(session, passed ? "checksPassed" : "checksFailed");
```

Fire an event that the agent chose with `fire` or `callTool`, never with `fireAsHost`, because
`fireAsHost` would then take a host-only event from the agent. Keep `fireAsHost` for the host's
own events.

`source: host` is a security boundary only when the host drives a model without tools. An agent
that runs under a harness, such as Claude Code, usually has a shell, and there the marker
prevents mistakes rather than an attack. A harness has no host to fire the events, so the smllm
CLI refuses to start an instance of such a machine, and `smllm validate` notes that the machine
needs a host to drive it.

## Host-driven machines

A host can drive a machine for a model that has no tools and only answers in text, such as a
model that the host calls through the Messages API with no tools. The host shows the model the
state, reads its answer as one event, runs its own checks, and fires the results. Four methods
serve it, and none of them changes anything:

- `hostView(key)` returns `{ location, model, host, invoke }`: where the session is, the events
  that the model can fire, the events that only the host fires, and the host's work that the
  state invokes. Each event has its `name`, `description`, and `params`, and a JSON param has its
  bundled `schema` as an object. Each `invoke` has its `src`, the name of the work, and its
  `input`, an object of fixed values, if the machine gives one: run the work, then fire its result
  with `fireAsHost`.
- `stateText(key, { schemas })` returns the state's text for the model: its instructions, the
  model's events with their params, and a request to answer with one event as JSON. Each JSON
  param shows its schema, unless `schemas` is `false`.
- `answerSchema(key)` returns a JSON Schema of a valid answer, as an object, for a model that can
  constrain its output, such as the Claude API's structured outputs.
- `checkAnswer(key, text)` reads the model's answer, a bare JSON object or one fenced block with
  nothing outside it. It returns `{ ok: true, event, params }`, whose params are plain
  JavaScript values ready for `fire`, or `{ ok: false, error }`, whose error the host sends back
  to the model as it stands. Every malformed answer gets the same error.

This loop runs a machine from start to end. The machine is an extractor builder for bank
statements, which is in the `examples/extractor` folder of the smllm repository. The model
answers with an extractor, a JSON value with a schema, or asks for more pages. The host checks
the extractor and fires `checks_passed` or `checks_failed`, and the machine gives up after five
attempts:

```js
const engine = new Engine(compiled, {}, memoryStorage());
// A keyless `enter` creates the session and starts the machine in one call.
const { session } = engine.fire(undefined, "enter", { stateMachine: "extractor-build" });
for (;;) {
  const view = engine.hostView(session);
  if (view.location.machine === null) break; // a final state: the session is idle again
  if (view.model.length > 0) {
    let prompt = `${engine.stateText(session)}\n${statement}`;
    for (;;) {
      const answer = engine.checkAnswer(session, await askModel(prompt)); // your adapter
      if (answer.ok) {
        engine.fire(session, answer.event, answer.params); // the model chose the event
        if (answer.event === "extractor") extractor = answer.params.json;
        break;
      }
      prompt = answer.error; // the refusal, as it stands; count it as an attempt
    }
  } else if (view.invoke.length > 0) {
    // The state names its work, so the host finds its code by name, not by the state's.
    for (const { src, input } of view.invoke) await work[src](session, input);
  } else {
    throw new Error(`nothing to do in ${view.location.state}`); // a person's step: wait for them
  }
}
```

`work` maps each `src` to the host's code, which fires the work's result:

```js
const work = {
  async runChecks(session) {
    const errors = await runChecks(extractor); // your checks, which can be async
    if (errors.length === 0) engine.fireAsHost(session, "checks_passed");
    else engine.fireAsHost(session, "checks_failed", { errors: errors.join("\n") });
  },
};
```

The model is never offered a built-in event, such as `yield`, `pause`, or `unmatched`, or a
host-only event, because those belong to whoever drives the session, which here is the host. The
host fires them as it needs them. An outcome that the model can choose, such as "I can't do this"
or "ask the user", is an event that the machine declares.

One machine can mix the model's steps, the host's deterministic steps, and a person's steps. A
state that offers the model events is a step of the model. A state whose events are all the
host's is a step of the host, such as `CHECKING`, or a person's step, such as an approval for
which the host fires the person's decision. A state that offers both lists is a step of the model
that the host can interrupt: the extractor builder's `READ` offers the host `timed_out`, which the
host fires when the model takes too long.

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
