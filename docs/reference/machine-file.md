---
type: Reference
id: reference-machine-file
title: The machine file
description: Each key of a machine file, with its type, whether it is required, and what it means.
order: "16"
generator: info-schema
---

# The machine file

This page lists each key of a machine file, `*.smllm.yaml`, as the JSON Schema that `smllm info schema` prints describes it. Each section is one object of the file, named by its path from the top of the file, in the order that you meet them. In a path, `<STATE>` stands for the name of a state, `<EVENT>` for the name of an event, `<PARAM>` for the name of a param, and `[]` for an item of a list. Actions, guards, and transitions, which several keys take, have sections of their own. A key without _yes_ under Required can be left out.

## The machine

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `id` | string | yes | The machine's id, unique within the config: letters, digits, `_`, and `-`. |
| `description` | string |  | The machine's description, which the idle list shows. |
| `initial` | string | yes | The state in which a new instance starts. |
| `meta` | object | yes | smllm's data for the machine. |
| `states` | map of objects | yes | The machine's states. smllm does not support nested `states`. |

## `meta`

smllm's data for the machine.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `smllm` | integer | yes | The smllm format version, which is 1. |
| `instance` | object |  | What the machine calls an instance, and the instance's ref. |
| `events` | map of objects |  | The machine's events, each declared once. A built-in event can set only its `description`. |
| `sharedActions` | list of objects |  | Actions added to the `entry` or `exit` actions of several states. |

## `states.<STATE>`

One state.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `description` | string |  | The state's description. |
| `type` | `atomic` or `final` |  | The state's type. A `final` state completes the instance. `atomic`: An ordinary state, the default. `final`: A state that completes the instance. |
| `meta` | object |  | smllm's data for the state. |
| `entry` | [action](#actions), or a list of them |  | The actions that run when the state is entered. Without a prompt among them, smllm shows the prompt file `enter-<STATE>.md`, if it exists. The action `setRef` can be written as its bare name. |
| `exit` | [action](#actions), or a list of them |  | The actions that run when the state is left. The action `setRef` can be written as its bare name. |
| `on` | map of ([transition](#transitions), or a list of them) |  | The transitions for each event. A transition can be written as the name of its target state. |
| `always` | [transition](#transitions), or a list of them |  | Transitions without an event. smllm leaves a state with `always` at once. A transition can be written as the name of its target state. |
| `invoke` | object, or a list of them |  | The host's work while an instance is in this state. The state handles an event of `source: host`, through which the host fires the work's result. |

## `meta.instance`

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `kind` | string |  | The word for an instance in the text that smllm writes, such as `issue`. The default is `instance`. |
| `ref` | object |  | The instance's ref: its id outside smllm, such as an issue number. |

## `meta.events.<EVENT>`

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `description` | string |  | The event's guidance for the agent, unless a transition gives its own. |
| `params` | object |  | The event's params: a JSON Schema object of string and JSON properties. |
| `source` | `agent` or `host` |  | Who fires the event: `agent`, the default, or `host`. Only the host that embeds the engine fires a `host` event, and the agent never sees it. |

## `meta.sharedActions[]`

One `meta.sharedActions` entry.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `states` | list of strings | yes | The states that the actions are added to. |
| `position` | `before` or `after` | yes | Whether the actions run before or after the state's own `entry` or `exit` actions. `before`: Before the state's own actions. `after`: After the state's own actions. |
| `entry` | [action](#actions), or a list of them |  | Actions added to each state's `entry`. The action `setRef` can be written as its bare name. |
| `exit` | [action](#actions), or a list of them |  | Actions added to each state's `exit`. The action `setRef` can be written as its bare name. |

## `states.<STATE>.meta`

smllm's data for a state.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `entryPoint` | boolean |  | Whether the agent can enter this state directly from idle. Default: `false`. |
| `fallback` | boolean |  | Whether the `unmatched` event enters this state. At most one state of a machine can be the fallback. Default: `false`. |
| `paramDescriptions` | map of map of strings |  | Prompts for params in this state, by event and then by param. |

## Actions

These keys take actions: `states.<STATE>.entry`, `states.<STATE>.exit`, `meta.sharedActions[].entry`, `meta.sharedActions[].exit`, `<transition>.actions`.

An action, written as `{type, params}`.

### `type: prompt`

Shows text to the agent.

It needs `params`, an object with these keys:

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `text` | string |  | The text itself. |
| `file` | string |  | A file of text, relative to the machine file, which smllm reads each time it shows the prompt. |

### `type: command`

Runs a command.

It needs `params`, an object with these keys:

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `run` | string or list of strings | yes | A string that a shell runs, or a list of a program and its arguments, run without a shell. String: A string that `sh -c` runs, or `cmd /C` on Windows. List of strings: A program and its arguments, run without a shell. |
| `timeoutSecs` | integer |  | The time limit in seconds. The default is 60. |
| `cwd` | string |  | The working directory, relative to the machine file. The default is the session's working directory. |

### `type: setRef`

Sets the instance's ref from the event's ref param.

It takes no `params`.

## Transitions

These keys take transitions: `states.<STATE>.on.<EVENT>`, `states.<STATE>.always`.

A transition object.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `target` | string |  | The target state. A transition without a target stays in the state without re-entering it. |
| `guard` | [guard](#guards) |  | A condition that must pass for the transition to be taken. |
| `actions` | [action](#actions), or a list of them |  | The actions that run when the transition is taken. The action `setRef` can be written as its bare name. |
| `reenter` | boolean |  | Whether a transition to its own state leaves and re-enters it, as in XState. |
| `description` | string |  | Guidance for the event in this state. |

## `states.<STATE>.invoke`

The host's work while an instance is in its state: XState's `invoke`, with only `src` and `input`. The host runs the work and fires its result as an event of `source: host`.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `src` | string | yes | The name of the work, by which the host finds its code. |
| `input` | object |  | Fixed values that the host passes to the work. |

## `meta.instance.ref`

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `param` | string |  | The name of the ref's param, which `enter` and `setRef` use. The default is `ref`. |
| `description` | string |  | The prompt that asks for the ref. |
| `pattern` | string |  | A pattern that the ref must match, in the syntax of a param's `pattern`. |

## `meta.events.<EVENT>.params`

A subset of a JSON Schema of `type: object`.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `type` | string | yes | Always `object`. |
| `properties` | map of objects |  | The params, by name. |
| `required` | list of strings |  | The names of the required params. |

## Guards

These keys take guards: `<transition>.guard`.

A guard, written as `{type, params}`.

### `type: command`

Passes when the command exits with status 0.

It needs `params`, an object with these keys:

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `run` | string or list of strings | yes | A string that a shell runs, or a list of a program and its arguments, run without a shell. String: A string that `sh -c` runs, or `cmd /C` on Windows. List of strings: A program and its arguments, run without a shell. |
| `timeoutSecs` | integer |  | The time limit in seconds. The default is 60. |
| `cwd` | string |  | The working directory, relative to the machine file. The default is the session's working directory. |

### `type: visits`

Passes when `state` has been entered at least `atLeast` times.

It needs `params`, an object with these keys:

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `state` | string | yes | The state whose entries are counted. |
| `atLeast` | integer | yes | The least number of entries, counting the current one. |

## `meta.events.<EVENT>.params.properties.<PARAM>`

One param's schema.

| Key | Type | Required | Description |
| --- | ---- | -------- | ----------- |
| `type` | `string` or `json` |  | `string`, the default, or `json`, for a param that takes any JSON value. |
| `description` | string |  | The prompt for the param. |
| `enum` | list of strings |  | The allowed values of a string param. |
| `pattern` | string |  | A pattern that a string param's value must match, in Rust `regex` syntax: the common subset of JSON Schema's ECMA-262 patterns, without lookaround or backreferences. In smllm, `\d`, `\w`, and `\s` match Unicode, and in a JavaScript host they match only ASCII. To match ASCII digits in both, write `[0-9]`. |
| `schema` | string or object |  | The JSON Schema of a `json` param's value, draft-07 or 2020-12: a path relative to the machine file, or the schema itself as a map. String: A JSON file, relative to the machine file. The path can lead outside the machine's folder. Object: The schema itself. |
