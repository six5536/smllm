---
type: ErrorReference
id: error-agt-2
title: 'AGT-2: Events list'
description: The findings that smllm validate reports when a machine changes the params or the source of a built-in event.
code: AGT-2
---

# AGT-2: Events list

A finding of `AGT-2` means that the machine changes a built-in event, such as `yield` or `pause`, in a way that smllm doesn't allow. A machine can reword the description of a built-in event, but smllm sets its params and who fires it. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: meta.events.<event>: built-in event <event> can change only its description, not its params (AGT-2)
<file>:<line>: error: meta.events.<event>: built-in event <event> can change only its description, not its source (AGT-2)
```

## Cause

An entry of `meta.events` has the name of a built-in event, and sets `params` or `source`.

## Fix

Remove `params` and `source` from the entry, and keep its `description`. Where the agent needs an event with other params, declare an event of your own, with a name of its own.

## Example

This machine gives the built-in event `yield` a param:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    yield:
      description: Stop and wait for the bug's reporter to answer.
      params:
        type: object
        properties:
          question: { type: string, description: What you asked the reporter. }
states:
  TRIAGE:
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports the params:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:6: error: meta.events.yield: built-in event yield can change only its description, not its params (AGT-2)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed machine rewords only the description of `yield`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    yield:
      description: Stop and wait for the bug's reporter to answer.
states:
  TRIAGE:
    on:
      reproduced: DONE
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
