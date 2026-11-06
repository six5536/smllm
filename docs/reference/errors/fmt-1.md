---
type: ErrorReference
id: error-fmt-1
title: 'FMT-1: The machine file'
description: The findings that smllm validate reports when a machine file isn't YAML that smllm reads, or has a key or a value that smllm doesn't know.
code: FMT-1
---

# FMT-1: The machine file

A finding of `FMT-1` means that the machine file isn't YAML that smllm reads, or that one of its keys or values isn't one that smllm knows. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: <YAML error> (FMT-1)
<file>: error: missing key `<key>` (FMT-1)
<file>:<line>: error: <path>: missing key `<key>` (FMT-1)
<file>:<line>: error: <path>: unknown key `<key>` (expected one of: <keys>) (FMT-1)
<file>:<line>: error: <path>: expected <type>, found <type> (FMT-1)
<file>:<line>: error: <path>: expected <type>, found nothing (FMT-1)
<file>:<line>: error: meta.smllm: unsupported smllm format version <n>. Set meta.smllm: 1, the only version that this smllm reads (FMT-1)
<file>:<line>: error: states: a state machine needs at least one state (FMT-1)
<file>:<line>: error: id: id "<id>" must contain only letters, digits, _, and - (FMT-1)
<file>:<line>: error: states.<state>: state name "<state>" must not be empty or contain spaces (FMT-1)
<file>: error: two machines in this config have id <id> (FMT-1)
<file>: error: machine id <id> differs from <id> only in case, so they would share a state directory on macOS and Windows (FMT-1)
<file>:<line>: error: states.<state>.type: `type: <type>` is not supported. smllm supports only flat atomic and final states (FMT-1)
<file>:<line>: error: <path>: `<key>` is not supported. smllm supports only part of XState: flat atomic and final states, without delays or context, and an invoke of only src and input (FMT-1)
```

## Cause

The first `missing key` form is a key of the whole file, such as `id`, and the second is a key of a part of it, such as `smllm` under `meta`. A finding of a wrong type ends with `found nothing` when the key has no value, such as a state written as `TRIAGE:` alone.

Each message has one of these causes:

- The YAML doesn't parse.
- A required key is missing, such as `id`, or `smllm` under `meta`.
- A key has a name that smllm doesn't know, or a value of a type that smllm doesn't expect, or no value.
- `meta.smllm` isn't 1.
- `states` is empty.
- The machine's ID has a character other than a letter, a digit, `_`, and `-`.
- A state's name is empty or has a space.
- Two machines of a config have the same ID, or IDs that differ only in case.
- A state sets `type` to `parallel`, `history`, or `compound`, which smllm leaves out.
- The file uses a key of XState that smllm leaves out: `states` or `initial` inside a state, for a nested state, `after`, `context`, `output`, `tags`, `history`, `onDone`, `types`, or `version`.

## Fix

Correct the key or the value that the message names. Set `meta.smllm` to `1`, give the machine at least one state, and give each machine of a config an ID of its own. Keep each state flat: a state that needs steps of its own becomes several states of the machine.

## Example

This machine sets a format version that smllm doesn't read:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 2
states:
  TRIAGE:
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports the version:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:4: error: meta.smllm: unsupported smllm format version 2. Set meta.smllm: 1, the only version that this smllm reads (FMT-1)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed machine sets version 1:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
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
