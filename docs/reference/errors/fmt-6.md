---
type: ErrorReference
id: error-fmt-6
title: 'FMT-6: Files that smllm reads'
description: The findings that smllm validate reports when it can't read a machine file that the config or the command names.
code: FMT-6
---

# FMT-6: Files that smllm reads

A finding of `FMT-6` means that smllm can't read a machine file that the config or the command line names. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>: error: can't read: no such file (FMT-6)
<file>: error: can't read: <reason> (FMT-6)
```

## Cause

The file doesn't exist, or smllm can't read it. A path in the config's `files` is relative to the folder of the config, so a path that starts at the project names a file that doesn't exist. smllm can't read a path that names a folder, or a file that the user isn't allowed to read.

## Fix

Correct the path in the config's `files`, or in the command, so that it names the machine file. Write a path in `files` from the folder of the config. Where the message names another error, correct the path so that it names a file, or let your user read the file.

## Example

This config names a machine file that doesn't exist, because the machine's file is `fix.smllm.yaml`:

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

**`.smllm/config.toml`**

```toml
[machines]
files = ["fixes.smllm.yaml"]
```

`smllm validate` reports the file that it can't read:

```console
$ smllm validate
.smllm/fixes.smllm.yaml: error: can't read: no such file (FMT-6)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed config names the machine's file:

**`.smllm/config.toml`**

```toml
[machines]
files = ["fix.smllm.yaml"]
```

`smllm validate` reports no finding:

```console
$ smllm validate
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
