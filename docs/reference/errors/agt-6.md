---
type: ErrorReference
id: error-agt-6
title: 'AGT-6: Guards and actions'
description: The findings that smllm validate reports when an entry of meta.sharedActions names a state that doesn't exist, or a position that smllm doesn't know.
code: AGT-6
---

# AGT-6: Guards and actions

A finding of `AGT-6` means that an entry of `meta.sharedActions` names a state that the machine doesn't have, or a position other than `before` and `after`. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: meta.sharedActions[<n>]: sharedActions lists unknown state <state> (AGT-6)
<file>:<line>: error: meta.sharedActions[<n>].position: `<value>` is not one of: before, after (AGT-6)
```

## Cause

Each message has its own cause:

- The `states` list of a shared action names a state that the machine doesn't have. A misspelt or renamed state is the usual cause.
- The `position` of a shared action is neither `before` nor `after`.

## Fix

Fix each message in the order of the causes:

- Correct the name to the name of a state, or remove it from the list.
- Set `position` to `before`, to run the shared actions before the state's own actions, or to `after`, to run them after.

## Example

The shared action of this machine still names `FIXING`, the old name of the state `FIX`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  sharedActions:
    - states: [TRIAGE, FIXING]
      position: after
      entry: { type: prompt, params: { text: "Record what you try in the bug's notes." } }
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

`smllm validate` reports the unknown state:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:6: error: meta.sharedActions[0]: sharedActions lists unknown state FIXING (AGT-6)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed shared action names `FIX`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  sharedActions:
    - states: [TRIAGE, FIX]
      position: after
      entry: { type: prompt, params: { text: "Record what you try in the bug's notes." } }
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
$ smllm validate .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
