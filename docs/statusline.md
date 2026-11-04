# smllm in your status line

This guide helps you show where smllm is in your harness's status line. It explains how to set the
row up in Claude Code, what the default row shows, the command's options and JSON fields, how to
build a row of your own, and how to use the command from other harnesses.

`smllm statusline` prints where the current session is, as one row for your harness's status
line. The row shows the state machine, the state, and the instance:

```
smllm dev › WORK (visit 3) · issue GH-123
smllm dev › REVIEW · issue GH-123 · yielded
smllm idle · 1 interrupted · 2 paused
```

The command reads Claude Code's status JSON on stdin and finds the smllm session bound to its
`session_id`. When there is nothing to show, because there is no smllm config or smllm hasn't
bound a session yet, it prints nothing. The command only reads, takes a few milliseconds, and
always exits with `0`, so it can't blank the rest of your status line.

- [Set it up in Claude Code](#set-it-up-in-claude-code)
- [The default row](#the-default-row)
- [Options](#options)
- [The JSON fields](#the-json-fields)
- [Custom rows](#custom-rows)
- [Other harnesses](#other-harnesses)

## Set it up in Claude Code

To let Claude make the change, ask it to "add smllm to my status line". The `smllm-statusline`
skill makes the change, and shows it to you first. `smllm harness install claude` installs the
skill, and so does `smllm harness install all`. To leave the skill out, add
`--without statusline`. The Claude Code plugin also ships the skill.

To make the change by hand, add a line to your status line script. Claude Code runs one
`statusLine` command, and each line that the command prints is a row. Add this line at the end of
your script, after its other output:

```sh
if row=$(printf '%s' "$input" | smllm statusline 2>/dev/null) && [ -n "$row" ]; then printf '\n%s' "$row"; fi
```

`$input` is the status JSON that your script read from stdin (`input=$(cat)`). Keep the `if`
form, because Claude Code blanks the whole status line when the script exits with a non-zero
status. A trailing `[ -n "$row" ] && printf …` would exit with a non-zero status whenever there is
nothing to show.

If you have no script yet, create `~/.claude/statusline-command.sh`:

```sh
#!/usr/bin/env bash
input=$(cat)
printf '%s' "$input" | smllm statusline
```

Then point Claude Code at the script in `~/.claude/settings.json`:

```json
{ "statusLine": { "type": "command", "command": "bash ~/.claude/statusline-command.sh" } }
```

`smllm harness install` never changes the `statusLine` setting itself. The setting is yours, and
a setting at project level would override the status lines of your collaborators.
`smllm harness status claude` reminds you when your status line doesn't call `smllm statusline`.

## The default row

The row takes one of two forms:

| Where        | Row                                                          |
| ------------ | ------------------------------------------------------------ |
| In a machine | `smllm MACHINE › STATE[ (visit N)] · KIND LABEL[ · yielded]` |
| In idle      | `smllm idle[ · N interrupted][ · N paused]`                  |

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
| `session`     | string         | The smllm session key (`sm-k7f3q2`)                  |
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

This object shows a session in the `WORK` state:

```json
{
  "session": "sm-k7f3q2",
  "idle": false,
  "machine": "dev",
  "state": "WORK",
  "visit": 3,
  "yielded": false,
  "instance": { "machine": "dev", "kind": "issue", "id": "a1b2c3", "ref": "GH-123", "label": "GH-123", "status": "active" },
  "interrupted": null,
  "paused": 2
}
```

## Custom rows

To build a row of your own in place of the default, read `--json` with `jq` and print it with
`printf`.

This row shows only the state, in bold yellow:

```sh
s=$(printf '%s' "$input" | smllm statusline --json)
state=$(jq -r '.state // empty' <<<"$s")
if [ -n "$state" ]; then printf '\n\033[1;33m%s\033[0m' "$state"; fi
```

This one puts the state and the instance on the same row as your other fields, and shows nothing
when the session is idle:

```sh
s=$(printf '%s' "$input" | smllm statusline --json)
where=$(jq -r 'select(.state) | "\(.state) \(.instance.label)"' <<<"$s")
printf '%s%s' "$my_other_fields" "${where:+ | $where}"
```

This one shows a warning colour while the agent has yielded:

```sh
s=$(printf '%s' "$input" | smllm statusline --json)
if [ "$(jq -r '.yielded' <<<"$s")" = true ]; then
  printf '\n\033[43;30m waiting for you \033[0m'
fi
```

## Other harnesses

The command needs only a session key. `smllm statusline --session KEY` works from any status bar,
prompt, or tmux line that knows the key. Without `--session`, the command reads stdin as JSON with
a `session_id` that the Claude Code hooks have bound. A host that uses `@six5536/smllm-wasm` gets
the same fields from `engine.status(key)`.
