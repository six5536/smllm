---
type: Tutorial
id: tutorial-run-the-engine-in-node
title: Run the engine in Node.js
description: Compile a state machine to JSON, and drive it from a Node.js script with the engine of `@six5536/smllm-wasm`.
order: "40"
---

# Run the engine in Node.js

In this tutorial you run smllm's engine inside your own program. You compile a state machine to JSON, load it in Node.js with `@six5536/smllm-wasm`, and fire events as an agent does, with no `smllm` process and no harness. At the end you can build a host of your own that gives an agent the smllm tool.

## Before you begin

- You have written and run a machine, as [Your first state machine][sokf:tutorial-first-machine] shows.
- smllm is installed, as the [README](https://github.com/six5536/smllm#install) shows.
- A current long-term support (LTS) release of Node.js is installed.

## Step 1: write the machine

In an empty folder, install the package:

```sh
npm install @six5536/smllm-wasm
```

Write a machine in the folder. It guides an agent through the review of a change: in `READ` the agent reads the change, in `REPORT` it writes its findings, and the event `reported` takes the agent's verdict.

**`review.smllm.yaml`**

```yaml
id: review
description: Review a change and give a verdict.
initial: READ
meta:
  smllm: 1
  instance:
    kind: review
  events:
    read:
      description: Fire after you have read the whole change.
    reported:
      description: Fire after you write your findings, with your verdict.
      params:
        type: object
        properties:
          verdict: { enum: [approve, reject], description: "Your verdict on the change." }
        required: [verdict]
states:
  READ:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Read the change and the code around it." } }
    on:
      read: REPORT
  REPORT:
    entry: { type: prompt, params: { text: "Write your findings, then fire reported with your verdict." } }
    on:
      reported: DONE
  DONE:
    type: final
    entry: { type: prompt, params: { text: "Post your findings for the author." } }
```

Check the machine:

```console
$ smllm validate review.smllm.yaml
0 errors, 0 warnings, 0 info
```

## Step 2: compile the machine

The engine reads compiled machines, not YAML. `smllm compile` checks a machine file, or a config with all its machines, and writes it as JSON. It prints nothing when it succeeds:

```console
$ smllm compile review.smllm.yaml -o review.json
```

Compile the machine again after each change to it, because the engine reads only `review.json`.

## Step 3: drive the machine from Node.js

Write a script that loads `review.json` into an engine, binds a session, and makes the calls that an agent makes through the smllm tool:

**`review.mjs`**

```js
import { readFileSync } from "node:fs";
import init, { Engine } from "@six5536/smllm-wasm";

// Load the engine's WebAssembly once.
await init();

let n = 0;
const engine = new Engine(readFileSync("review.json", "utf8"), {
  // Fixed random numbers give the same keys on every run.
  random: () => ++n,
});

const { session, text } = engine.bind("node");
console.log(text);
console.log(engine.callTool({ session, event: "enter", params: { stateMachine: "review" } }).text);
const stop = engine.stop(session);
console.log(stop.decision);
console.log(stop.text);
console.log(engine.callTool({ session, event: "read" }).text);
console.log(engine.callTool({ session, event: "reported", params: { verdict: "approve" } }).text);
```

The script makes these calls:

- `init` loads the engine's WebAssembly, once for each process.
- `bind` starts a session for the harness `node`, and returns its key and the text that a session start hook gives the agent.
- `callTool` takes the same arguments as the smllm tool, and returns the tool's reply.
- `stop` decides whether the agent can stop, as the stop hook does.

Run the script:

```console
$ node review.mjs
<smllm>
session sm-200000 · idle
State machines:
- review: Review a change and give a verdict.
    review id param ref
    starts at: READ; entry points: READ
Fire one event: smllm({ session: "sm-200000", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: review): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>

<smllm>
session sm-200000 · review › READ · review i-400000
Arrived by: enter (new review)
Params: stateMachine = "review"
<instructions>
Read the change and the code around it.
</instructions>
</smllm>

block
<smllm>
session sm-200000 · review › READ · review i-400000
Fire one event: smllm({ session: "sm-200000", event, params })
<events>
- read: Fire after you have read the whole change.
- yield: Stop for now and stay in READ.
    note (optional): What you are waiting for.
- pause: Put review i-400000 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>

<smllm>
session sm-200000 · review › REPORT · review i-400000
Arrived by: read from READ
<instructions>
Write your findings, then fire reported with your verdict.
</instructions>
</smllm>

<smllm>
session sm-200000 · review › DONE · review i-400000
Arrived by: reported from REPORT
Params: verdict = "approve"
<instructions>
Post your findings for the author.
</instructions>
Completed review i-400000. You are now in idle.
State machines:
- review: Review a change and give a verdict.
    review id param ref
    starts at: READ; entry points: READ
Fire one event: smllm({ session: "sm-200000", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: review): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

The output shows three things about the engine:

- The texts are the same as those of the `smllm` command: the engine is the same.
- The stop is `block`, because the agent is in `READ` and fired no event, so a host shows the agent the events and keeps it working.
- Without `random`, the engine draws random keys, so your keys differ from the page's.

The engine keeps its state in memory. To keep it between runs, pass a storage as the third argument of `Engine`, such as `nodeFileStorage` from `@six5536/smllm-wasm/node`. The [README of `@six5536/smllm-wasm`](https://github.com/six5536/smllm/tree/main/packages/smllm-wasm#readme) describes the whole API: storage, hosts that run `command` guards, and hosts that drive a model without tools.

## What's next

- [Check the agent's work][sokf:tutorial-check-the-agents-work]
- [The machine file][sokf:reference-machine-file]

<!-- sokf:links -->

[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:tutorial-check-the-agents-work]: /docs/tutorials/check-the-agents-work.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
