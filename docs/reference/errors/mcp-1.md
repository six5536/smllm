---
type: ErrorReference
id: error-mcp-1
title: 'MCP-1: Param types'
description: The finding that smllm validate reports when an event's param has a type other than string and json.
code: MCP-1
---

# MCP-1: Param types

A finding of `MCP-1` means that a param of an event has a type that smllm doesn't take. A param is a string, the default, or a JSON value, with `type: json`. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: <path>: param <param> has type <type>, but a param is a string or JSON. Use type: string, with enum or pattern to limit its values, or type: json for a JSON value (MCP-1)
```

## Cause

A param of `meta.events.<EVENT>.params.properties` sets `type` to a value other than `string` and `json`, such as `number`, `integer`, or `boolean`.

## Fix

Choose the type by what the param holds:

- For a value that the agent gives as text, such as a number or a name, set `type: string`, or remove `type`. To limit its values, add `pattern` or `enum`.
- For a structured value, set `type: json`, and add `schema` to check it.

## Example

The param `bugId` of this machine is a number:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    reproduced:
      description: Fire after a test reproduces the bug.
      params:
        type: object
        properties:
          bugId: { type: number, description: The number of the bug. }
        required: [bugId]
states:
  TRIAGE:
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports the type:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:11: error: meta.events.reproduced.params.properties.bugId: param bugId has type number, but a param is a string or JSON. Use type: string, with enum or pattern to limit its values, or type: json for a JSON value (MCP-1)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed param is a string of digits:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    reproduced:
      description: Fire after a test reproduces the bug.
      params:
        type: object
        properties:
          bugId: { type: string, pattern: "^[0-9]+$", description: The number of the bug. }
        required: [bugId]
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
