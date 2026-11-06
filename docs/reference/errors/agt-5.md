---
type: ErrorReference
id: error-agt-5
title: 'AGT-5: Transitions'
description: The findings that smllm validate reports when a transition leads to a state that doesn't exist, a state can't be reached, or always transitions can't settle.
code: AGT-5
---

# AGT-5: Transitions

A finding of `AGT-5` means that a transition of the machine leads nowhere, that a state can't be reached, or that a state with `always` can't settle. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: initial: initial state <state> does not exist (AGT-5)
<file>:<line>: error: <path>: target state <state> does not exist (AGT-5)
<file>:<line>: error: states.<state>.always: always transitions loop: <states>. smllm leaves a state with always at once, so the loop might never stop. Break it with an event (AGT-5)
<file>:<line>: warning: states.<state>: state <state> is unreachable. Add a transition to it, or set meta.entryPoint: true (AGT-5)
<file>:<line>: warning: <path>: reenter has no effect without a target, because a transition without one never re-enters. To re-enter it, add target: <state> (AGT-5)
<file>:<line>: warning: states.<state>: smllm leaves a state with always at once, so it never offers the state's on events (AGT-5)
```

## Cause

Each message has its own cause, and `smllm validate` lists the last three, the warnings, only with `--warnings`:

- `initial` names a state that the machine doesn't have.
- A transition's `target` names a state that the machine doesn't have. A misspelt or renamed state is the usual cause.
- States with `always` transitions lead back to the state where they started, so smllm would pass through them forever.
- No transition leads to a state, and the state isn't an entry point, so no instance can ever be in it.
- A transition sets `reenter: true` but has no `target`. A transition without a target leaves the instance where it is, and never re-enters the state.
- A state has both `always` and `on`. smllm leaves the state at once, so the agent never gets the chance to fire its events.

## Fix

Fix each message in the order of the causes:

- Set `initial` to the name of a state.
- Correct the target to the name of a state, or add the state.
- Give one of the states of the loop an event that the agent fires, in place of its `always` transition.
- Add a transition to the state, or mark the state with `meta: { entryPoint: true }` if the agent enters it directly. Delete a state that nothing uses.
- Set `target` to the state itself, so that the transition re-enters it and runs its exit and entry actions again.
- Move the events of `on` to a state that the agent waits in, or remove the `always` transitions.

## Example

The event `reproduced` of this machine leads to `FIXING`, but the state is called `FIX`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    on:
      reproduced: FIXING
  FIX:
    on:
      fixed: DONE
  DONE:
    type: final
```

`smllm validate --warnings` reports the missing state. It also warns that nothing leads to `FIX`, and so to `DONE`:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:8: error: states.TRIAGE.on.reproduced: target state FIXING does not exist (AGT-5)
.smllm/fix.smllm.yaml:9: warning: states.FIX: state FIX is unreachable. Add a transition to it, or set meta.entryPoint: true (AGT-5)
.smllm/fix.smllm.yaml:12: warning: states.DONE: state DONE is unreachable. Add a transition to it, or set meta.entryPoint: true (FMT-2)
1 error, 2 warnings, 0 info
[exit 1]
```

The warning for `DONE` ends with `(FMT-2)`, not `(AGT-5)`, because `DONE` is a final state. A final state that no instance can reach breaks the rule for final states, so smllm reports it under [FMT-2][sokf:error-fmt-2].

The fixed machine leads to `FIX`:

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

`smllm validate` reports no finding:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:error-fmt-2]: /docs/reference/errors/fmt-2.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
