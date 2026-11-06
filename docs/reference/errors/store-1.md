---
type: ErrorReference
id: error-store-1
title: 'STORE-1: The instance files'
description: The findings that smllm validate reports when it can't read an instance file or a ref marker of a machine.
code: STORE-1
---

# STORE-1: The instance files

A finding of `STORE-1` means that smllm can't read a file in which it saves the instances of a machine. smllm skips such a file everywhere else, so `smllm validate` is where you learn of it. The findings are warnings. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>: warning: can't read the instance file of <machine>: <reason>. smllm skips it, so fix or delete the file (STORE-1)
<file>: warning: can't read the ref marker of <machine>: <reason>. smllm finds these refs more slowly without the marker, and rewrites the marker when it next sets one of them. Don't delete the marker, because smllm reads a missing marker as no such ref (STORE-1)
```

## Cause

smllm keeps each instance of a machine in a JSON file in the machine's state folder, which is `.smllm/state/<MACHINE>` by default. The ref markers, in the folder's `refs` folder, map the ref of each instance to its ID. `smllm validate` lists the warnings only with `--warnings`. Each message has its own cause:

- An instance file isn't JSON, or isn't an instance that smllm reads. An edit by hand that went wrong is the usual cause, or a file from a version of smllm that saves instances in another form.
- A ref marker isn't JSON, or smllm can't read it.

## Fix

Fix each message in the order of the causes:

- Correct the file, or delete it. Deleting it deletes the instance.
- Correct the marker, but don't delete it. smllm rewrites the marker the next time it sets a ref that the marker holds.

## Example

An edit by hand cut this paused instance of the machine `fix` short:

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

**`.smllm/state/fix/paused/i-5z4x7n.json`**

```json
{
  "id": "i-5z4x7n",
  "machine": "fix",
  "ref": null,
  "state": "FIX",
  "status": "paused",
  "holder": null,
  "version": 3,
  "visits": {
    "FIX": 1,
    "TRIAGE": 1
  },
```

`smllm validate --warnings` reports the file:

```console
$ smllm validate --warnings
.smllm/state/fix/paused/i-5z4x7n.json: warning: can't read the instance file of fix: it isn't valid JSON: EOF while parsing a value at line 13 column 0. smllm skips it, so fix or delete the file (STORE-1)
0 errors, 1 warning, 0 info
```

The fixed file is whole again:

**`.smllm/state/fix/paused/i-5z4x7n.json`**

```json
{
  "id": "i-5z4x7n",
  "machine": "fix",
  "ref": null,
  "state": "FIX",
  "status": "paused",
  "holder": null,
  "version": 3,
  "visits": {
    "FIX": 1,
    "TRIAGE": 1
  },
  "resumeState": null,
  "created": 1793928806977,
  "updated": 1793928816175
}
```

`smllm validate` reports no finding:

```console
$ smllm validate --warnings
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
