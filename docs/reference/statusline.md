---
type: Reference
id: reference-statusline
title: The status line row
description: The forms of the row that `smllm statusline` prints, its options, its JSON fields, and its use from other harnesses.
order: "30"
---

# The status line row

This page lists the forms of the row that `smllm statusline` prints, then its options, its JSON fields, and its use from harnesses other than Claude Code.

`smllm statusline` prints where the current session is, as one row for your harness's status line. The command reads Claude Code's status JSON on stdin and finds the smllm session bound to its `session_id`. When there is nothing to show, because there is no smllm config or smllm hasn't bound a session yet, it prints nothing. The command only reads, takes a few milliseconds, and always exits with `0`, so it can't blank the rest of your status line.

- [The default row](#the-default-row)
- [Options](#options)
- [The JSON fields](#the-json-fields)
- [Other harnesses](#other-harnesses)

## The default row

The row takes one of two forms:

| Where        | Row                                                          |
| ------------ | ------------------------------------------------------------ |
| In a machine | `smllm MACHINE › STATE[ (visit N)] · KIND LABEL[ · yielded]` |
| In idle      | `smllm idle[ · N interrupted][ · N paused]`                  |

These rows show each form:

```
smllm dev › WORK (visit 3) · issue GH-123
smllm dev › REVIEW · issue GH-123 · yielded
smllm idle · 1 interrupted · 2 paused
```

The parts mean this:

- `MACHINE` is the state machine, and `STATE` is its current state.
- `(visit N)` appears from the second visit to a state, and `N` is the number of the visit.
- `KIND LABEL` is the instance, for example `issue GH-123`. The label is the instance's ref
  after the ref is set, and its ID before.
- `N interrupted` and `N paused` give the number of interrupted and paused instances.
- `yielded` means that the agent fired `yield`, so it can stop until your next prompt.

The row has these colours: `smllm` dim, the machine cyan, the state bold, the instance magenta,
the notes dim, and `yielded` yellow. Colour is on by default, because a status line command never
runs in a terminal that it could check. To turn colour off, set `NO_COLOR=1` or pass
`--color never`.

## Options

The command takes these options:

```
smllm statusline [--session KEY] [--json] [--color auto|always|never]
```

- `--session KEY` shows the session with the key `KEY` instead of reading stdin.
- `--json` prints the fields described in [the JSON fields](#the-json-fields) instead of the row.
- `--color` takes `always`, `never`, or `auto`. `auto` means always, unless `NO_COLOR` is set.

## The JSON fields

`smllm statusline --json` prints one object, or `{}` when there is nothing to show. Every key is
always present, and its value is `null` when it doesn't apply.

| Field         | Type           | Meaning                                              |
| ------------- | -------------- | ---------------------------------------------------- |
| `session`     | string         | The smllm session key (`sm-nqvfvw`)                  |
| `idle`        | bool           | True when the session holds no instance              |
| `machine`     | string \| null | The state machine of the held instance               |
| `state`       | string \| null | The held instance's current state                    |
| `visit`       | number \| null | The number of times the state has been entered       |
| `yielded`     | bool           | True when the agent fired `yield` after your last prompt |
| `instance`    | object \| null | The held instance                                    |
| `interrupted` | object \| null | The instance that `unmatched` put aside              |
| `paused`      | number         | The number of paused instances of the configured state machines |

`instance` and `interrupted` are each an object `{machine, kind, id, ref, label, status}`:

- `kind` is what the machine calls an instance (`meta.instance.kind`).
- `ref` is `null` until it is set.
- `label` is the ref, or the ID when there is no ref.
- `status` is `active`, `interrupted`, `paused`, or `completed`.

A later release can add fields. Before version 1.0, a field is renamed or removed only in a minor
release.

The sample in this section comes from a project with this config and this machine:

**`.smllm/config.toml`**

```toml
[machines]
files = ["dev.smllm.yaml"]
```

**`.smllm/dev.smllm.yaml`**

```yaml
id: dev
description: Work on an issue, then review the change.
initial: WORK
meta:
  smllm: 1
  instance:
    kind: issue
    ref: { param: issueId, pattern: "^GH-[0-9]+$", description: "The issue's id, such as GH-123." }
  events:
    done:
      description: Fire after the change is made.
    changes:
      description: Fire when the review asks for changes.
    approved:
      description: Fire when the review approves the change.
states:
  WORK:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Make the change, then fire done." } }
    on:
      done: REVIEW
  REVIEW:
    entry: { type: prompt, params: { text: "Review the change, then fire approved or changes." } }
    on:
      changes: WORK
      approved: DONE
  DONE:
    type: final
```

The session paused the issues `GH-121` and `GH-122`. Then it entered `GH-123`, and the review
sent the issue back to `WORK` twice, so the session is at its third visit to `WORK`, and
`smllm statusline --json` prints this object for it:

<!-- example:
  fire enter stateMachine=dev issueId=GH-121
  fire pause
  fire enter stateMachine=dev issueId=GH-122
  fire pause
  fire enter stateMachine=dev issueId=GH-123
  fire done
  fire changes
  fire done
  fire changes
  statusline
-->
```json
{
  "session": "sm-nqvfvw",
  "idle": false,
  "machine": "dev",
  "state": "WORK",
  "visit": 3,
  "yielded": false,
  "instance": { "machine": "dev", "kind": "issue", "id": "i-n7d6tx", "ref": "GH-123", "label": "GH-123", "status": "active" },
  "interrupted": null,
  "paused": 2
}
```

`sm-nqvfvw` is the key of the session, and `i-n7d6tx` is the instance's ID. Your key and your
instance's ID differ.

## Other harnesses

The command needs only a session key. `smllm statusline --session KEY` works from any status bar,
prompt, or tmux line that knows the key. Without `--session`, the command reads stdin as JSON with
a `session_id` that the Claude Code hooks have bound. A host that uses `@six5536/smllm-wasm` gets
the same fields from `engine.status(key)`.
