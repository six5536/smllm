---
type: ErrorReference
id: error-fmt-3
title: 'FMT-3: Actions and guards'
description: The findings that smllm validate reports when an action or a guard isn't one that smllm knows, or has params that it can't use.
code: FMT-3
---

# FMT-3: Actions and guards

A finding of `FMT-3` means that an action or a guard of the machine isn't one that smllm knows, or that its params are wrong. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: <path>: unknown action `<name>`. Write an action as {type, params}. Only setRef can be a bare name (FMT-3)
<file>:<line>: error: <path>.type: unknown action type `<type>` (expected one of: prompt, command, setRef) (FMT-3)
<file>:<line>: error: <path>: a prompt needs exactly one of: text, file (FMT-3)
<file>:<line>: error: <path>: prompt file <file>: <error>. The path is relative to the file that gives it (FMT-3)
<file>:<line>: error: <path>: setRef runs only in a transition's actions under `on`. Entry, exit, and always have no event to read the ref param from (FMT-3)
<file>:<line>: error: <path>: a guard is a map {type, params}, not a name. Guard types: command { run }, visits { state, atLeast } (FMT-3)
<file>:<line>: error: <path>.type: unknown guard type `<type>` (expected one of: command, visits) (FMT-3)
<file>:<line>: error: <path>: visits counts unknown state <state> (FMT-3)
<file>:<line>: warning: <path>: visits atLeast 0 is always true (FMT-3)
```

## Cause

`smllm validate` lists a warning only with `--warnings`.

Each message has its own cause:

- An action is a bare name other than `setRef`.
- An action's `type` is a value other than `prompt`, `command`, and `setRef`.
- A prompt's params give both `text` and `file`, or neither.
- smllm can't read the file that a prompt names. smllm reads the path from the folder of the file that gives it, which is the machine file, or the config for `[idle] on-enter`.
- A `setRef` action is under `entry`, `exit`, or `always`. Only a transition under `on` has an event that carries the ref param.
- A guard is a name, not a map.
- A guard's `type` is a value other than `command` and `visits`.
- A `visits` guard counts a state that the machine doesn't have.
- A `visits` guard has `atLeast: 0`, which every instance passes.

## Fix

Make the change that the message names:

- Write the action as a map with `type` and `params`.
- Set `type` to `prompt`, `command`, or `setRef`.
- Give the prompt exactly one of `text` and `file`.
- Correct the path, or create the file. The path is relative to the folder of the file that gives it: the machine file, or the config for `[idle] on-enter`.
- Move `setRef` to the actions of a transition under `on`.
- Write the guard as a map with `type` and `params`.
- Set `type` to `command` or `visits`.
- Name a state of the machine in `state`.
- Set `atLeast` to 1 or more, or remove the guard.

## Example

This machine gives a prompt file with a path that starts at the project, not at the machine file:

**`.smllm/prompts/triage.md`**

```markdown
Reproduce the bug, then fire `reproduced`.
```

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    entry:
      type: prompt
      params:
        file: .smllm/prompts/triage.md
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports the prompt file:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:7: error: states.TRIAGE.entry: prompt file .smllm/prompts/triage.md: no such file. The path is relative to the file that gives it (FMT-3)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed machine gives the path from the folder of the machine file:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    entry:
      type: prompt
      params:
        file: prompts/triage.md
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
