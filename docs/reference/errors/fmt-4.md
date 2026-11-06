---
type: ErrorReference
id: error-fmt-4
title: 'FMT-4: Event and instance declarations'
description: The findings that smllm validate reports when a declaration of an event, a param, or the instance's ref has a bad name, a bad pattern, or a part that nothing uses.
code: FMT-4
---

# FMT-4: Event and instance declarations

A finding of `FMT-4` means that a declaration under `meta.events` or `meta.instance`, or a state's rewording of a param, has a bad name, a bad pattern, or a part that nothing uses. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: meta.instance.ref.param: ref param "<param>" must contain only letters, digits, _, and - (FMT-4)
<file>:<line>: error: <path>: event name "<event>" must contain only letters, digits, _, and - (FMT-4)
<file>:<line>: error: meta.events.<event>.params.properties.<param>: param name "<param>" must contain only letters, digits, _, and - (FMT-4)
<file>:<line>: error: meta.events.<event>.params: params type must be object, not <type> (FMT-4)
<file>:<line>: error: meta.events.<event>.params.required: required param <param> is not in properties (FMT-4)
<file>:<line>: error: meta.events.<event>.params.properties.<param>: enum is empty, so it allows no value (FMT-4)
<file>:<line>: error: <path>.pattern: invalid pattern "<pattern>": <reason> (FMT-4)
<file>:<line>: warning: meta.events.<event>.params.properties.<param>: enum value "<value>" does not match the pattern, so the param can never take it (FMT-4)
<file>:<line>: warning: meta.events.<event>.params.properties.<param>: <event> sets the ref, so meta.instance.ref.pattern <pattern> applies instead of this pattern. Remove this pattern, or change meta.instance.ref.pattern (FMT-4)
<file>:<line>: warning: states.<state>.meta.paramDescriptions.<event>: <state> has no transition on <event>, so smllm never shows these descriptions (FMT-4)
<file>:<line>: error: states.<state>.meta.paramDescriptions.<event>.<param>: <event> declares no param <param>. Declare params in meta.events, because paramDescriptions only rewords their prompts (FMT-4)
<file>:<line>: warning: meta.events.<event>: event <event> is declared but no state uses it (FMT-4)
<file>:<line>: error: meta.events.<event>.source: `<value>` is not one of: agent, host (FMT-4)
<file>:<line>: info: meta.events: only the host fires the events <events>, so the machine needs a host to drive it. A harness can't start an instance of it, and a host that embeds the engine fires these events (FMT-4)
```

## Cause

`smllm validate` lists a warning only with `--warnings`, and an info finding only with `--info`.

Each message has its own cause:

- The ref param's name has a character other than a letter, a digit, `_`, and `-`.
- An event's name, under `meta.events` or under a state's `on`, has such a character.
- A param's name has such a character.
- An event's `params` has a `type` other than `object`.
- `required` names a param that `properties` doesn't declare.
- A param's `enum` is an empty list.
- A `pattern` isn't a valid regular expression.
- A value of a param's `enum` doesn't match the param's `pattern`.
- An event that sets the ref gives the ref param a `pattern` of its own, and `meta.instance.ref.pattern` gives another. The pattern of `meta.instance.ref` applies.
- A state's `meta.paramDescriptions` rewords the params of an event that the state has no transition on.
- A state's `meta.paramDescriptions` rewords a param that the event doesn't declare.
- `meta.events` declares an event that no state has a transition on.
- An event's `source` is a value other than `agent` and `host`.
- An event has `source: host`. This finding is only information: only a host program can fire the event, so the machine needs a host to drive it.

## Fix

Make the change that the message names:

- Rename the ref param with letters, digits, `_`, and `-`.
- Rename the event in the same way.
- Rename the param in the same way.
- Set the `type` of `params` to `object`.
- Correct the name in `required`, or declare the param under `properties`.
- Add a value to `enum`, or remove `enum`.
- Correct the pattern.
- Correct the value or the pattern.
- Remove the param's `pattern`, or change `meta.instance.ref.pattern`.
- Add a transition on the event to the state, or remove the event from `meta.paramDescriptions`.
- Declare the param under the event in `meta.events`, or remove it from `meta.paramDescriptions`.
- Add a transition on the event to a state, or remove the event.
- Set `source` to `agent` or `host`.
- Nothing needs to change if a host program drives the machine. If the agent should fire the event, set `source` to `agent`, or remove `source`.

## Example

This machine requires a param `issue`, but the param under `properties` is `issueId`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    reproduced:
      params:
        type: object
        required: [issue]
        properties:
          issueId:
            type: string
states:
  TRIAGE:
    on:
      reproduced: DONE
  DONE:
    type: final
```

`smllm validate` reports the required param:

```console
$ smllm validate .smllm/fix.smllm.yaml
.smllm/fix.smllm.yaml:9: error: meta.events.reproduced.params.required: required param issue is not in properties (FMT-4)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed machine requires `issueId`:

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
initial: TRIAGE
meta:
  smllm: 1
  events:
    reproduced:
      params:
        type: object
        required: [issueId]
        properties:
          issueId:
            type: string
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
