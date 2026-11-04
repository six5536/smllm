# smllm-wasm example

This example helps you drive an agent loop with
[`@six5536/smllm-wasm`](../../packages/smllm-wasm). It describes the example's files, and shows how
to run the loop in Node and in a browser.

A scripted model stands in for a real model and answers in milliseconds. The scripted model
answers at once, so the example can show the time that smllm itself takes for each turn.

The example has these files:

- `agent-loop.ts` holds the loop. It uses only web-standard APIs, so it runs in a browser as well
  as in Node. It gives the model `engine.tool()`, passes each tool call to `engine.callTool()`,
  and calls `engine.stop()` when the model wants to stop.
- `node.ts` runs the loop in Node, with `nodeHost` and `nodeFileStorage` from
  `@six5536/smllm-wasm/node`. `nodeHost` runs real `command` guards and actions. The script runs
  the loop twice on one storage folder, and the second run is a new engine that continues the
  paused note. It checks the expected results, and prints smllm's time for each turn, apart from
  the time for storage and commands.
- `note.smllm.yaml` is the state machine: the agent writes a note, a command checks the note, and
  another command files it. `machines.json` is its `smllm compile` output, which
  `npm run build:wasm` regenerates.

To run the example in Node, build the package, then type-check and run `node.ts`:

```sh
npm run build:wasm      # the wasm and the package
npm run test:example    # type-check, then run node.ts
```

To run the loop in a browser, import `init` and `Engine` from `@six5536/smllm-wasm`, and
`await init()`. Then call `runAgent` with a host that has no `command` kinds, or one that supplies
its own.
