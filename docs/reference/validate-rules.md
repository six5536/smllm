---
type: Reference
id: reference-validate-rules
title: The rules of smllm validate
description: Each rule that `smllm validate` checks, under the code that its findings print.
order: "40"
generator: info-rules
---

# The rules of smllm validate

This page lists the rules that `smllm validate` checks. Each finding ends with the code of the rule that it breaks, such as `(FMT-4)`, and this page says what each rule checks. `smllm info rules` prints the same page, so it always matches the smllm that you run.

| Code | Rule | What validate checks |
| ---- | ---- | -------------------- |
| `FMT-1` | The machine file | The file is YAML that parses, sets `meta.smllm: 1`, and has at least one state. Its keys are the known ones, each required key is present, and each value has the expected type. The machine's ID and its state names have the allowed characters, and no other machine in the config has the same ID or one that differs only in case. The file uses only the part of XState v5 that smllm supports: flat atomic and final states, without nested or parallel states, delays, or context, and with an `invoke` of only `src` and `input`. |
| `FMT-2` | States and transitions | A final state has no `on` or `always`, isn't an entry point, and can be reached. A machine without a final state gets an info finding, because its instances never complete. At most one state is the fallback, and the fallback isn't final. A state doesn't handle a built-in event under `on`, a list of guarded transitions ends with one without a guard, and neither the ref param nor any other param has the name of a param of `enter`. An event with the ref param sets the ref. A state with an `invoke` handles an event of `source: host`, which carries the work's result, and each `invoke` names its `src`. |
| `FMT-3` | Actions and guards | Each action is a `{type, params}` object of a known type, with the params that its type needs. A prompt has exactly one of `text` and `file`, a file path is relative to the file that gives it, and `setRef` appears only where an event carries the ref param. Each guard is a `{type, params}` object of a known type, with valid params. A `visits` guard counts a state that exists, and its `atLeast` is more than 0. |
| `FMT-4` | Event and instance declarations | The ref param's name, and each declared event and param name, contain only letters, digits, `_`, and `-`. An event's params are an object of properties, each required param is in `properties`, and each `enum` and `pattern` is valid and allows at least one value. A state rewords only the events that it has a transition on, and only params that the event declares in `meta.events`. Each declared event is used by a state, and each ref pattern can match a ref. An event's `source` is `agent` or `host`, and a machine with an event of `source: host` gets an info finding, because only a host can fire the event, so the machine needs a host to drive it. |
| `FMT-10` | JSON params | A `json` param takes no `enum` or `pattern`, a string param takes no `schema`, and a param that sets the ref is a string. A `json` param's schema, and each file that a relative `$ref` in it names, is a map or a JSON file that smllm can read, declares draft-07, 2020-12, or no draft, and is valid against its draft. A schema without a draft is 2020-12, or the draft of the schema that refers to it. Each `$ref` to another file is a path relative to the schema that holds it, because smllm doesn't fetch a schema from the network. |
| `FMT-6` | Files that smllm reads | smllm can read each file that the config names. |
| `FMT-7` | Combining configs | A config with errors doesn't replace the machines of the configs before it, and validate says which machines it leaves out and which machine replaces which. |
| `AGT-1` | Author text | Author text, such as a prompt or an event's description, contains none of the closing tags `</smllm>`, `</instructions>`, and `</events>` that fence smllm's own text, because the agent couldn't then tell smllm's text from the author's. A finding under this rule is a warning. |
| `AGT-2` | Events list | A built-in event changes only its description, never its params or its source. |
| `AGT-5` | Transitions | The initial state and each transition's target exist, each state can be reached, and a transition that should re-enter its state names it as the target. A state with `always` has no `on` events, because smllm leaves it at once, and no chain of `always` transitions loops back to its start. |
| `AGT-6` | Guards and actions | `meta.sharedActions` lists only states that exist. |
| `MCP-1` | Param types | Each param has `type: string`, the default, or `type: json`. |
| `CLI-3` | Configs and saved instances | smllm can read and parse each config file, and each saved instance is in a state that still exists. |
| `CLI-11` | Command guards and actions | A command's `run` is a non-empty string, run by a shell, or a non-empty list, run without one. A list-form `run` doesn't rely on `$SMLLM_*` variables, because no shell expands them. A command's `timeoutSecs` is at least 1. |
| `STORE-1` | The instance files | smllm can read each instance file and each ref marker of the config's machines. |
