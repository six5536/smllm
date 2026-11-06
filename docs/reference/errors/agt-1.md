---
type: ErrorReference
id: error-agt-1
title: 'AGT-1: Author text'
description: The finding that smllm validate reports when the text of a machine contains one of the closing tags that fence smllm's text for the agent.
code: AGT-1
---

# AGT-1: Author text

A finding of `AGT-1` means that text of the machine, such as a prompt or an event's description, contains one of the closing tags `</smllm>`, `</instructions>`, and `</events>`. smllm fences the text that it gives the agent with these tags, so a tag in your text hides from the agent where smllm's text ends. The finding is a warning. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: warning: <path>: text contains `<tag>`, which smllm uses to fence agent text. Reword it so the agent can tell smllm's text from yours (AGT-1)
```

## Cause

A prompt, an event's description, a transition's description, or another text that smllm passes to the agent contains `</smllm>`, `</instructions>`, or `</events>`. The usual cause is a prompt that quotes smllm's own text. `smllm validate` lists the warning only with `--warnings`.

## Fix

Reword the text without the tag. To refer to smllm's text, name it in words, such as "the instructions".

## Example

The prompt of `TRIAGE` quotes the tag that ends the instructions:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    entry: { type: prompt, params: { text: "Write a test that reproduces the bug. Fire reproduced when you reach </instructions>." } }
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate --warnings` reports the tag:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:7: warning: states.TRIAGE.entry: text contains `</instructions>`, which smllm uses to fence agent text. Reword it so the agent can tell smllm's text from yours (AGT-1)
0 errors, 1 warning, 0 info
```

The fixed prompt says what to do in words:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
states:
  TRIAGE:
    entry: { type: prompt, params: { text: "Write a test that reproduces the bug, then fire reproduced." } }
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports no finding:

```console
$ smllm validate --warnings .smllm/fix.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
