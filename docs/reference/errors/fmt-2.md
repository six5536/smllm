---
type: ErrorReference
id: error-fmt-2
title: 'FMT-2: States and transitions'
description: The findings that smllm validate reports when a state, a transition, or the ref param breaks a rule of how a machine moves from state to state.
code: FMT-2
---

# FMT-2: States and transitions

A finding of `FMT-2` means that a state or a transition breaks a rule of how the machine moves from state to state, or that a param takes a name that smllm keeps for itself. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: states.<state>.type: `<type>` is not one of: atomic, final (FMT-2)
<file>:<line>: error: states.<state>: a final state cannot have on or always (FMT-2)
<file>:<line>: warning: states.<state>: a final state that is an entry point completes the instance as soon as it is entered. Remove meta.entryPoint, or make the state non-final (FMT-2)
<file>:<line>: warning: states.<state>: state <state> is unreachable. Add a transition to it, or set meta.entryPoint: true (FMT-2)
<file>:<line>: info: states: no final state: instances of this machine never complete. If instances should complete, add a state with type: final (FMT-2)
<file>:<line>: error: states.<state>: the fallback state cannot be final (FMT-2)
<file>:<line>: error: states.<state>: only one state can be the fallback, and <state> already is (FMT-2)
<file>:<line>: error: states.<state>.on.<event>: <event> is a built-in event, so a state cannot handle it under on (FMT-2)
<file>:<line>: error: <path>: the last transition has a guard, so nothing happens when every guard fails. End the list with a transition without a guard, such as one back to this state (FMT-2)
<file>:<line>: error: meta.instance.ref.param: ref param <param> has the name of one of enter's own params. Choose another name, such as issueId (FMT-2)
<file>:<line>: warning: meta.events.<event>.params.properties.<param>: param <param> has the name of one of enter's own params (FMT-2)
<file>:<line>: warning: meta.events.<event>: <event> has a param with the ref param's name <param>, but never sets the ref. Rename the param, or add a setRef action (FMT-2)
<file>:<line>: error: states.<state>.invoke: the state invokes the host's work, but handles no event of source: host to carry its result. Declare the result as an event of source: host in meta.events, and handle it in the state's on (FMT-2)
<file>:<line>: error: states.<state>.invoke: a final state invokes no work, because the instance completes when it enters the state. Remove the invoke, or make the state atomic (FMT-2)
<file>:<line>: error: <path>.src: src is empty. Give src the name of the host's work (FMT-2)
```

## Cause

`smllm validate` lists a warning only with `--warnings`, and an info finding only with `--info`.

Each message has its own cause:

- A state's `type` is a value other than `atomic` and `final`.
- A final state has `on` or `always`. A final state completes the instance, so no transition leaves it.
- A final state sets `meta.entryPoint: true`, so an agent who enters it completes the instance at once.
- No transition leads to a final state, and the state isn't an entry point.
- The machine has no final state, so its instances never complete. This finding is only information, because a machine can run for ever on purpose.
- A final state sets `meta.fallback: true`.
- More than one state sets `meta.fallback: true`.
- A state handles a built-in event, such as `yield` or `pause`, under `on`. smllm handles the built-in events `enter`, `resume`, `pause`, `unmatched`, `yield`, and `listPaused` itself.
- The last transition of a list has a `guard`. When every guard fails, the event matches no transition.
- `meta.instance.ref.param` is `stateMachine` or `state`, the names of the params of `enter`.
- An event declares a param named `stateMachine` or `state`.
- An event declares a param with the name of the ref param, but no transition on the event has a `setRef` action, so the param never sets the ref.
- A state has an `invoke`, but handles no event of `source: host` under `on`, so the host can't fire the result of the work.
- A final state has an `invoke`.
- An `invoke` has an empty `src`.

## Fix

Make the change that the message names:

- Set `type` to `atomic` or `final`, or leave it out for an atomic state.
- Remove `on` and `always` from the final state, or remove `type: final`.
- Remove `meta.entryPoint` from the final state, or make the state atomic.
- Add a transition to the final state, or remove the state.
- Add a state with `type: final` if the instances complete.
- Move `meta.fallback` to an atomic state.
- Keep `meta.fallback: true` on one state only.
- Rename the event, or remove it from `on`.
- End the list with a transition that has no guard, such as one back to the same state.
- Give the ref param another name, such as `issueId`.
- Rename the param.
- Rename the param, or add a `setRef` action to a transition on the event.
- Declare an event with `source: host` for the work's result, and handle it in the state's `on`.
- Remove the `invoke` from the final state, or make the state atomic.
- Give `src` the name of the work.

## Example

This machine runs the tests before it completes, but the only transition on `fixed` has a guard:

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
      fixed:
        - target: DONE
          guard:
            type: command
            params:
              run: cargo test
  DONE:
    type: final
```

`smllm validate` reports the guard:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:11: error: states.FIX.on.fixed: the last transition has a guard, so nothing happens when every guard fails. End the list with a transition without a guard, such as one back to this state (FMT-2)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed machine stays in `FIX` when the tests fail:

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
      fixed:
        - target: DONE
          guard:
            type: command
            params:
              run: cargo test
        - target: FIX
  DONE:
    type: final
```

`smllm validate` reports no finding:

```console
$ smllm validate .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
