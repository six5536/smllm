# The rules of smllm validate

This page lists the rules that `smllm validate` checks. Each finding ends with the code of the rule that it breaks, such as `(CFG-7)`, and this page says what each rule checks. `smllm info rules` prints the same page, so it always matches the smllm that you run.

| Code | Rule | What validate checks |
| ---- | ---- | -------------------- |
| `CFG-1` | The machine file | The file is YAML that parses, sets `meta.smllm: 1`, and has at least one state. Its keys are the known ones, each required key is present, and each value has the expected type. The machine's ID and its state names have the allowed characters, and no other machine in the config has the same ID or one that differs only in case. |
| `CFG-2` | The supported part of XState | The file uses only the part of XState v5 that smllm supports: flat atomic and final states, without nested or parallel states, delays, invocations, or context. |
| `CFG-3` | Transitions | The initial state and each transition's target exist, each state can be reached, and a transition that should re-enter its state names it as the target. |
| `CFG-4` | Actions | Each action is a `{type, params}` object of a known type, with the params that its type needs. A prompt has exactly one of `text` and `file`, a file path is relative to the file that gives it, and `setRef` appears only where an event carries the ref param. |
| `CFG-5` | Guards | Each guard is a `{type, params}` object of a known type, with valid params. A `visits` guard counts a state that exists, and its `atLeast` is more than 0. |
| `CFG-6` | The instance's names | The ref param's name contains only letters, digits, `_`, and `-`. |
| `CFG-7` | Event declarations | Each declared event and param has a name that contains only letters, digits, `_`, and `-`. Its params are an object of string properties, each required param is in `properties`, and each `enum` and `pattern` is valid and allows at least one value. |
| `CFG-8` | Rewording for one state | A state rewords only the events that it has a transition on, and only params that the event declares in `meta.events`. |
| `CFG-9` | Using events | Each declared event is used by a state, and each ref pattern can match a ref. |
| `CFG-11` | Shared actions | `meta.sharedActions` lists only states that exist. |
| `CFG-12` | Final states | A final state has no `on` or `always`, isn't an entry point, and can be reached. A machine without a final state gets a warning, because its instances never complete. |
| `CFG-13` | The machine's structure | At most one state is the fallback, and the fallback isn't final. A state doesn't handle a built-in event under `on`, a list of guarded transitions ends with one without a guard, and neither the ref param nor any other param has the name of a param of `enter`. An event with the ref param sets the ref. |
| `CFG-14` | Findings | smllm can read each file that the config names. |
| `CFG-15` | Combining configs | A config with errors doesn't replace the machines of the configs before it, and validate says which machines it leaves out and which machine replaces which. |
| `CFG-16` | String params | Each param has `type: string`. |
| `DEC-2` | Always states | A state with `always` has no `on` events, because smllm leaves it at once, and no chain of `always` transitions loops back to its start. |
| `DEC-4` | Command forms | A command's `run` is a non-empty string, run by a shell, or a non-empty list, run without one. |
| `DEC-5` | Command results | A command's `timeoutSecs` is at least 1. |
| `DEC-9` | Commands without a shell | A list-form `run` doesn't rely on `$SMLLM_*` variables, because no shell expands them. |
| `IDLE-1` | Built-in events | A built-in event changes only its description, never its params. |
| `IDLE-4` | Saved instances | Each saved instance is in a state that still exists. |
| `STO-1` | The instance files | smllm can read each instance file and each ref marker of the config's machines. |
