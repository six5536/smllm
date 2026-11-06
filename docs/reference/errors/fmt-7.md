---
type: ErrorReference
id: error-fmt-7
title: 'FMT-7: Combining configs'
description: The findings that smllm validate reports when it combines the user config and the project config, and a machine of one replaces a machine of the other.
code: FMT-7
---

# FMT-7: Combining configs

A finding of `FMT-7` means that smllm combined the user config, `~/.config/smllm/config.toml`, with the project config, and that a machine of the project replaces a machine of the user config, or might. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>: info: machine <id> here replaces the one in <file> (FMT-7)
<file>: warning: smllm does not use machine <id> in <file> while this file, which replaces it, has errors (FMT-7)
<file>: warning: smllm doesn't load the machine <id> from earlier configs while this file has errors, because this file might replace it (FMT-7)
```

## Cause

`smllm validate` lists a warning only with `--warnings`, and an info finding only with `--info`.

smllm reads the user config first, then the project config. When both configs give a machine with the same ID, the project's machine replaces the user's. Each message has its own cause:

- A machine of the project config has the ID of a machine of the user config, and replaces it. This finding is only information.
- A machine file of the project config has errors, and its `id` is the ID of a machine of the user config. smllm uses neither machine until the file is fixed, because the user's machine isn't the one that the project asks for.
- The project config itself has errors, so smllm can't tell which machines it gives. smllm uses none of the user config's machines until the config is fixed.

## Fix

Fix the errors that `smllm validate` reports in the project's machine file or config. smllm then uses the project's machine in place of the user's. To use the user's machine in the project, remove the project's machine from the project config, or give the project's machine another ID.

## Example

smllm prints the full path of a file in the user config, and this page shortens your home folder to `~`. The user config gives a machine `fix`:

**`~/.config/smllm/config.toml`**

```toml
[machines]
files = ["fix.smllm.yaml"]
```

**`~/.config/smllm/fix.smllm.yaml`**

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

The project gives its own machine `fix`, with a review step, but misspells `final`:

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
      reproduced: REVIEW
  REVIEW:
    on:
      approved: DONE
  DONE:
    type: fnal
```

`smllm validate --warnings` reports the error, and that smllm doesn't use the user's machine in its place:

```console
$ smllm validate --warnings
.smllm/fix.smllm.yaml: warning: smllm does not use machine fix in ~/.config/smllm/fix.smllm.yaml while this file, which replaces it, has errors (FMT-7)
.smllm/fix.smllm.yaml:13: error: states.DONE.type: `fnal` is not one of: atomic, final (FMT-2)
1 error, 1 warning, 0 info
[exit 1]
```

The fixed machine spells `final`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    on:
      reproduced: REVIEW
  REVIEW:
    on:
      approved: DONE
  DONE:
    type: final
```

A run that names the machine file checks the file alone, without the configs, and reports no finding:

```console
$ smllm validate .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

`smllm validate --info` combines the configs, reports no error, and says that the project's machine replaces the user's:

```console
$ smllm validate --info
.smllm/fix.smllm.yaml: info: machine fix here replaces the one in ~/.config/smllm/fix.smllm.yaml (FMT-7)
0 errors, 0 warnings, 1 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
