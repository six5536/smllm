---
type: ErrorReference
id: error-cli-3
title: 'CLI-3: Configs and saved instances'
description: The findings that smllm validate reports when a saved instance is in a state that its machine no longer has, or when smllm can't read a config file.
code: CLI-3
---

# CLI-3: Configs and saved instances

A finding of `CLI-3` means that a saved instance is in a state that its machine no longer has, or that smllm can't read a config file. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>: warning: saved instance <instance> is in state <state>, which no longer exists. To repair it, fire enter with its ref and a state (CLI-3)
<file>: error: can't read: <reason> (CLI-3)
<file>:<line>: error: <TOML error> (CLI-3)
```

## Cause

Each message has its own cause, and `smllm validate` lists the first, a warning, only with `--warnings`:

- smllm saved an instance in a state, and an edit of the machine file then renamed or removed that state. The message names the machine file.
- smllm can't read a config file, such as a file named with `--config` that doesn't exist.
- A config file isn't TOML that smllm reads, or it has a key or a value that smllm doesn't know.

## Fix

Fix each message in the order of the causes:

- Enter the instance again with the event `enter`, and give its ref and the state to continue in, as `state`. The state doesn't have to be an entry point. Where you renamed the state by mistake, rename it back instead.
- Correct the path of the config file, or make the file readable.
- Correct the line that the message names.

## Example

This machine has the states `TRIAGE`, `FIX`, and `DONE`:

**`.smllm/config.toml`**

```toml
[machines]
files = ["fix.smllm.yaml"]
```

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    on:
      reproduced: FIX
  FIX:
    on:
      fixed: DONE
  DONE:
    type: final
```

An agent enters the machine and fires `reproduced`, so the instance is in `FIX`, then puts the instance aside:

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=fix
<smllm>
session sm-nqvfvw · fix › TRIAGE · instance i-5z4x7n
Arrived by: enter (new instance)
Params: stateMachine = "fix"
</smllm>
$ smllm fire --session sm-nqvfvw reproduced
<smllm>
session sm-nqvfvw · fix › FIX · instance i-5z4x7n
Arrived by: reproduced from TRIAGE
</smllm>
$ smllm fire --session sm-nqvfvw pause
<smllm>
session sm-nqvfvw · idle
Paused instance i-5z4x7n at FIX.
State machines:
- fix
    instance id param ref
    starts at: TRIAGE
Paused:
- instance i-5z4x7n (fix) at FIX
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: fix): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

The machine then renames `FIX` to `REPAIR`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    on:
      reproduced: REPAIR
  REPAIR:
    on:
      fixed: DONE
  DONE:
    type: final
```

`smllm validate --warnings` reports the saved instance:

```console
$ smllm validate --warnings
.smllm/fix.smllm.yaml: warning: saved instance i-5z4x7n is in state FIX, which no longer exists. To repair it, fire enter with its ref and a state (CLI-3)
0 errors, 1 warning, 0 info
```

Entering the instance with `state=REPAIR` moves it to the new state:

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=fix --param ref=i-5z4x7n --param state=REPAIR
<smllm>
session sm-nqvfvw · fix › REPAIR · instance i-5z4x7n
Arrived by: enter (saved state FIX no longer exists)
Params: stateMachine = "fix", ref = "i-5z4x7n", state = "REPAIR"
</smllm>
```

`smllm validate` reports no finding:

```console
$ smllm validate --warnings
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
