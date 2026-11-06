---
type: ErrorReference
id: error-cli-11
title: 'CLI-11: Command guards and actions'
description: The findings that smllm validate reports when a command guard or action has an empty or malformed run, uses $SMLLM_* variables without a shell, or has a time limit of 0.
code: CLI-11
---

# CLI-11: Command guards and actions

A finding of `CLI-11` means that a guard or an action of `type: command` can't run as written. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: <path>.params.run: run must be a string (shell) or a list of strings (no shell) (CLI-11)
<file>:<line>: error: <path>: command run is empty (CLI-11)
<file>:<line>: error: <path>: command run is an empty list (CLI-11)
<file>:<line>: error: <path>: timeoutSecs must be at least 1 (CLI-11)
<file>:<line>: warning: <path>: a list-form run gets no shell, so nothing expands $SMLLM_*. Use the string form, or read the environment in a script (CLI-11)
```

## Cause

Each message has its own cause, and `smllm validate` lists the last, a warning, only with `--warnings`:

- `run` is neither a string nor a list of strings, such as a number or a list that holds a number.
- `run` is an empty string, or a string of spaces.
- `run` is an empty list.
- `timeoutSecs` is 0.
- `run` is a list, and one of its items contains `$SMLLM_`. smllm runs a list without a shell, so the program gets the text `$SMLLM_REF` itself, not the value of the variable.

## Fix

Fix each message in the order of the causes:

- Write `run` as a string, which a shell runs, or as a list of strings: the program, then its arguments.
- Give `run` the command to run.
- Give `run` at least the program to run.
- Set `timeoutSecs` to 1 or more, or remove it for the default of 60 seconds.
- Write `run` as a string, so that the shell expands the variables, or run a script that reads them from its environment. On Windows, `cmd` writes a variable as `%SMLLM_REF%`.

## Example

The command of `DONE` passes the ref of the instance in a list, where no shell expands `$SMLLM_REF`:

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
    entry: { type: command, params: { run: [git, commit, -am, "Fix $SMLLM_REF"] } }
```

`smllm validate --warnings` reports the list:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:11: warning: states.DONE.entry: a list-form run gets no shell, so nothing expands $SMLLM_*. Use the string form, or read the environment in a script (CLI-11)
0 errors, 1 warning, 0 info
```

The fixed command is a string, which a shell runs:

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
    entry: { type: command, params: { run: 'git commit -am "Fix $SMLLM_REF"' } }
```

`smllm validate` reports no finding:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
