---
type: Reference
id: reference-agent-text
title: The agent's text
description: The parts of the `<smllm>` text that the agent reads, the built-in events, the idle list, and the text of a reply, a refused event, and a blocked stop.
order: "18"
---

# The agent's text

This page lists what smllm says to the agent: first the parts of a `<smllm>` block in the order in which they appear, then the header, the idle list, the entry block, and the events list, then the built-in events, and last the text of a reply, a refused event, and a blocked stop. The agent reads this text in the hooks of its harness and in the replies of the smllm tool, and `smllm fire` prints the same text in a shell.

- [The example machine](#the-example-machine)
- [The block](#the-block)
- [The header](#the-header)
- [The idle list](#the-idle-list)
- [The entry block](#the-entry-block)
- [The events list](#the-events-list)
- [The built-in events](#the-built-in-events)
- [A refused event](#a-refused-event)
- [A blocked stop](#a-blocked-stop)
- [The replies to a fired event](#the-replies-to-a-fired-event)

## The example machine

The examples of this page come from one session, in this order, with this config and this machine. The machine guides the agent through an issue: in `PLAN` the agent writes a plan, and a `command` guard checks that the plan exists. In `WORK` the agent makes the change, and `DONE` ends the work.

**`.smllm/config.toml`**

```toml
[machines]
files = ["dev.smllm.yaml"]
```

**`.smllm/dev.smllm.yaml`**

```yaml
id: dev
description: Work on an issue, from a plan to a change.
initial: PLAN
meta:
  smllm: 1
  instance:
    kind: issue
    ref: { param: issueId, pattern: "^GH-[0-9]+$", description: "The issue's id, such as GH-12." }
  events:
    planned:
      description: Fire after you write the plan in PLAN.md.
      params:
        type: object
        properties:
          size: { description: "How large the change is.", enum: [small, large] }
        required: [size]
    done:
      description: Fire after you make the change.
states:
  PLAN:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Write a plan for the issue in PLAN.md, then fire planned." } }
    on:
      planned:
        - { target: WORK, guard: { type: command, params: { run: "test -f PLAN.md" } } }
        - { target: PLAN, reenter: true }
  WORK:
    entry: { type: prompt, params: { text: "Make the change that PLAN.md describes, then fire done." } }
    on:
      done: DONE
  DONE:
    type: final
    entry: { type: prompt, params: { text: "Summarise the change for the issue." } }
```

## The block

Each text that smllm gives the agent is one block, fenced in `<smllm>` and `</smllm>`. A block holds these parts, in this order, and leaves out each part that it doesn't need:

1. The [header](#the-header).
2. `Arrived by: HOW`, which says how the instance reached its state, as [the entry block](#the-entry-block) lists.
3. `Params: NAME = "VALUE", …`, the params of the event that the agent fired.
4. The notes, one on each line, such as `Paused issue GH-12 at WORK.`
5. The trace of the guards that smllm checked, and the failures of actions and prompts.
6. `error: MESSAGE`, when smllm refused the event, as [a refused event](#a-refused-event) lists.
7. `<instructions>`, which holds the text of the prompts that the state's actions gave, in the order in which they ran, with a blank line between two prompts.
8. After a final state, `Completed KIND LABEL. You are now in idle.`, then the [idle list](#the-idle-list).
9. The [events list](#the-events-list).

The text of a machine's author goes into `<instructions>` as it is. Every other line goes into the block on one line of its own: a line break shows as `\n`, and a `<` that starts the tag `smllm`, `instructions`, or `events` shows as `&lt;`. A ref or a param value therefore can't close the block. `smllm validate` warns about author text that contains a closing tag, as rule [`AGT-1`][sokf:error-agt-1] says.

## The header

The first line of every block says where the session is. It takes one of three forms:

| Where               | Header                                                    |
| ------------------- | --------------------------------------------------------- |
| In a machine        | `session KEY · MACHINE › STATE[ (visit N)] · KIND LABEL` |
| In idle             | `session KEY · idle`                                      |
| No session was made | `no session · idle`                                       |

The parts mean this:

- `KEY` is the session key, such as `sm-nqvfvw`, which the agent passes on every call of the smllm tool.
- `MACHINE` is the state machine, and `STATE` is the instance's current state.
- `(visit N)` appears from the second visit to a state, and `N` is the number of the visit. Each time the instance enters the state counts as a visit, also when `enter` or `resume` brings it there.
- `KIND` is what the machine calls an instance, from `meta.instance.kind`, and `instance` when the machine doesn't say.
- `LABEL` is the instance's ref, such as `GH-12`, after the ref is set, and its ID, such as `i-k0xzjt`, before.
- `no session · idle` heads the refusal of an `enter` that the agent fired without a session key, because smllm made no session for it.

## The idle list

A session that holds no instance is in idle. The idle list is the block of idle, which the agent sees when its session starts:

<!-- example:
  hook session-start
-->
```text
<smllm>
session sm-nqvfvw · idle
State machines:
- dev: Work on an issue, from a plan to a change.
    issue id param issueId (pattern: ^GH-[0-9]+$): The issue's id, such as GH-12.
    starts at: PLAN; entry points: PLAN
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: dev): The state machine to enter.
    issueId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

After the header, the notes, and any error, the idle list has these lines:

- `<instructions>` holds the text that the config's `[idle]` table gives, when it gives one.
- `State machines:` starts the list of the machines of the config, or `No state machines are configured.` replaces it.
- `- ID: DESCRIPTION` names a machine.
- `KIND id param PARAM (pattern: PATTERN): DESCRIPTION` names the param that holds the machine's ref.
- `starts at: INITIAL; entry points: STATE, …` names the initial state and the states that `enter` can start in.
- `Interrupted: KIND LABEL (MACHINE) at STATE` names the instance that `unmatched` put aside, when there is one.
- `Paused:` starts the list of paused instances, with one `- KIND LABEL (MACHINE) at STATE` line for each. The list shows at most 10, the most recently updated first.
- `…and N more paused: fire listPaused to list them all.` follows a list that leaves some out.
- The events list of idle comes last.

## The entry block

When an event moves the instance to a state, the reply is the state's entry block: the header, `Arrived by`, the params, the notes and the trace, and `<instructions>`. The entry block has no events list:

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=dev --param issueId=GH-12
<smllm>
session sm-nqvfvw · dev › PLAN · issue GH-12
Arrived by: enter (new issue)
Params: stateMachine = "dev", issueId = "GH-12"
<instructions>
Write a plan for the issue in PLAN.md, then fire planned.
</instructions>
</smllm>
```

`Arrived by` takes one of these forms:

- `EVENT from STATE`: the event `EVENT` moved the instance from `STATE`.
- `enter (new KIND)`: `enter` started a new instance.
- `enter`: `enter` continued an instance at its saved state.
- `enter (jump from STATE)`: `enter` with `state` moved an instance from `STATE` to an entry point.
- `enter (reopened from STATE)`: `enter` with `state` reopened a completed instance.
- `enter (saved state STATE no longer exists)`: `enter` with `state` moved an instance whose saved state the machine no longer has.
- `resume`: `resume` returned the session to the instance that it put aside.
- `unmatched from STATE`: `unmatched` moved the instance to the machine's fallback state.

A form ends with `via STATE, …` when the instance passed through states with `always` transitions on its way.

The notes and the trace take these forms:

- `Took over from session KEY (last active TIME)`: the instance belonged to another session, which loses it.
- `Reopened KIND LABEL at STATE.`
- ``Guard: command `RUN` → true``, or ``Guard: command `RUN` → false (DETAIL)``: the result of a `command` guard, where `DETAIL` is the exit status and the end of the command's output.
- `Guard: visits STATE ≥ N → BOOL (visits: COUNT)`: the result of a `visits` guard.
- `Action failed: LIST[INDEX] KIND: DETAIL`: an action failed, and smllm skipped the commands after it in the same list.
- `Prompt failed: FILE: REASON`: smllm couldn't read the file of a prompt.
- `No always transition of STATE matched; stopped there` and `Stopped after 32 always transitions; check for a loop`: the `always` transitions couldn't leave a state.

## The events list

The events list names each event that the agent can fire in the session's current state, with its params. The agent sees it when it calls the smllm tool with no event, when its session starts again, after a refused event, and when smllm blocks a stop. `smllm session show` prints the same block:

```console
$ smllm session show sm-nqvfvw
<smllm>
session sm-nqvfvw · dev › PLAN · issue GH-12
<instructions>
Write a plan for the issue in PLAN.md, then fire planned.
</instructions>
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after you write the plan in PLAN.md.
    size (required, one of: small, large): How large the change is.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put issue GH-12 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

The list has these forms:

- `Fire one event: smllm({ session: "KEY", event, params })` shows the call that fires an event. Without a session, it reads `Fire one event: smllm({ event, params })`.
- `<events>` holds one item for each event: `- NAME: DESCRIPTION`.
- Each param of the event follows its event on a line of its own, indented by four spaces: `NAME (required|optional[, one of: VALUE, …][, pattern: PATTERN][, a JSON value[, schema: SCHEMA]]): DESCRIPTION`.

In a machine state, the list names the state's own events in the order of its `on`, then the built-in events. An event's description comes from its transition's `description`, or else from `meta.events`. A param's description comes from the state's `meta.paramDescriptions`, or else from the param's `description`. The list leaves out each event that the machine marks with `source: host`, because only the host fires it.

## The built-in events

smllm adds its built-in events to every machine:

| Event        | Offered                                         | Params                                     | What it does                                                         |
| ------------ | ----------------------------------------------- | ------------------------------------------ | -------------------------------------------------------------------- |
| `enter`      | In idle                                         | `stateMachine`, the ref param, and `state` | Starts a new instance, or continues, jumps, or reopens one.          |
| `resume`     | In idle with an interrupted instance, and in the fallback state after `unmatched` | None                                       | Returns to the state that `unmatched` left.                          |
| `listPaused` | In idle, when the idle list leaves paused instances out | None                                       | Lists every paused instance, and changes nothing.                    |
| `yield`      | In each state that isn't final                  | `note`, optional                           | Keeps the instance in its state and lets the agent stop.             |
| `pause`      | In each state                                   | None                                       | Puts the instance aside as paused, and returns the session to idle.  |
| `unmatched`  | In each state                                   | None                                       | Moves the instance to the fallback state, or puts it aside as interrupted and returns the session to idle. |

A machine can change the description of a built-in event in `meta.events`, but not its params or its source, as rule [`AGT-2`][sokf:error-agt-2] says. A state can't handle a built-in event under `on`.

## A refused event

When smllm refuses an event, nothing changes, and the reply is the header, `error: MESSAGE`, and the events list. `smllm fire` exits with `1`:

```console
$ smllm fire --session sm-nqvfvw done
<smllm>
session sm-nqvfvw · dev › PLAN · issue GH-12
error: done is not offered here (offered: planned, yield, pause, unmatched)
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after you write the plan in PLAN.md.
    size (required, one of: small, large): How large the change is.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put issue GH-12 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
[exit 1]
$ smllm fire --session sm-nqvfvw planned --param size=huge
<smllm>
session sm-nqvfvw · dev › PLAN · issue GH-12
error: planned param size must be one of: small, large (got "huge")
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after you write the plan in PLAN.md.
    size (required, one of: small, large): How large the change is.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put issue GH-12 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
[exit 1]
```

In a machine state, the message takes one of these forms:

- `EVENT is not offered here (offered: EVENT, …)`
- `EVENT is an event that only the host fires (offered: EVENT, …)`
- `EVENT needs param NAME`
- `EVENT has no param NAME (params: NAME, …)`
- `EVENT takes no params, but got NAME`
- `EVENT param NAME must be one of: VALUE, … (got "VALUE")`
- `EVENT param NAME must match PATTERN (got "VALUE")`
- `EVENT param NAME must be a string`
- `EVENT param NAME must be a JSON value: REASON`
- `EVENT param NAME doesn't match its schema. At PATH: ERROR`, with one sentence for each part of the value that fails, `the root` as the PATH of the whole value, and at most ten sentences, then `And N more`
- `KIND ID already has ref REF, and smllm sets a ref only once`
- `another KIND of MACHINE already has ref REF`
- `the interrupted state STATE no longer exists; leave with one of this state's events, or pause and enter with a state: STATE, …`

In idle, the message takes one of these forms:

- `EVENT is not offered in idle (offered: EVENT, …)`
- `enter needs param stateMachine`
- `enter param stateMachine must be one of: MACHINE, … (got "VALUE")`
- `enter has no param NAME for MACHINE (params: stateMachine, PARAM, state)`
- `enter got param NAME more than once`
- `enter param PARAM must match PATTERN (got "VALUE")`
- `STATE is not an entry point of MACHINE (entry points: STATE, …)`
- `KIND LABEL is completed. To reopen it, fire enter with state: STATE`, or `one of STATE, …` when the machine has several entry points
- `KIND LABEL is completed, and no state is an entry point to reopen it at`
- `state machine MACHINE needs a host to drive it: only the host fires its events EVENT, …`
- `MACHINE has no state STATE`
- `saved state STATE of KIND LABEL no longer exists. Fire enter with state: one of STATE, …`
- `listPaused takes no params, but got NAME`
- `resume takes no params, but got NAME`
- `state machine MACHINE is not configured (its file might have errors); fix the config, then resume`
- `LABEL moved to session KEY`
- `the interrupted instance is no longer interrupted`
- `saved state STATE no longer exists. Fire enter with stateMachine MACHINE, PARAM LABEL, state: one of STATE, …`

When the session's instance has left it, smllm returns the session to idle, and the reply is the idle list with one of these messages:

- `KIND LABEL moved to session KEY; you are in idle`: another session took the instance over.
- `KIND LABEL is STATUS; you are in idle`: the instance is no longer active.
- `another session changed KIND LABEL; you are in idle`: another session wrote the instance first.
- `instance ID no longer exists; you are in idle`

## A blocked stop

When the agent tries to end its turn in a state that isn't final, the stop hook of the harness asks smllm, and smllm blocks the stop. The harness gives the agent this block, which is the header and the events list, without the instructions:

<!-- example:
  hook stop
-->
```text
<smllm>
session sm-nqvfvw · dev › PLAN · issue GH-12
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after you write the plan in PLAN.md.
    size (required, one of: small, large): How large the change is.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put issue GH-12 aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

smllm lets a stop through without a block in these cases:

- The session is in idle.
- The agent fired `yield` after the user's last prompt, and no transition has happened since.
- The agent tries to stop a second time in a row without firing an event. smllm records this stop as `runaway` in the session's audit, and writes the block to standard error instead.
- The harness can't block a stop, as [Harnesses][sokf:reference-harnesses] lists. In GitHub Copilot the session start binds no session, so the stop hook allows every stop without a block. The `agents` integration runs no hooks. In `opencode run` the run ends with the agent's turn, so the agent never reads the block.

[What happens when the agent stops][sokf:explanation-what-happens-when-the-agent-stops] explains these rules.

## The replies to a fired event

A fired event that smllm accepts gets one of these replies:

- A transition gets the entry block of the state that the instance reaches.
- A transition to a final state gets the state's block, `Completed KIND LABEL. You are now in idle.`, and the idle list.
- `yield` gets the header and `Yielded: staying in STATE. You can end your turn.`
- `pause` and `unmatched` without a fallback state get the idle list, with a note that names the instance that the event put aside.
- `listPaused` gets the header of idle, the list of every paused instance, and the events list of idle.

A guard that fails moves the instance along the event's next transition. Here the plan doesn't exist yet, so `planned` re-enters `PLAN`, and the trace shows the guard:

```console
$ smllm fire --session sm-nqvfvw planned --param size=small
<smllm>
session sm-nqvfvw · dev › PLAN (visit 2) · issue GH-12
Arrived by: planned from PLAN
Params: size = "small"
Guard: command `test -f PLAN.md` → false (exited 1)
<instructions>
Write a plan for the issue in PLAN.md, then fire planned.
</instructions>
</smllm>
```

After the agent writes the plan, the guard passes:

**`PLAN.md`**

```markdown
Rename the option `--colour` to `--color`.
```

```console
$ smllm fire --session sm-nqvfvw planned --param size=small
<smllm>
session sm-nqvfvw · dev › WORK · issue GH-12
Arrived by: planned from PLAN
Params: size = "small"
Guard: command `test -f PLAN.md` → true
<instructions>
Make the change that PLAN.md describes, then fire done.
</instructions>
</smllm>
```

These are the replies to `yield`, to `pause`, and to an `enter` that continues the paused issue:

```console
$ smllm fire --session sm-nqvfvw yield --param "note=Waiting for the user to approve the plan."
<smllm>
session sm-nqvfvw · dev › WORK · issue GH-12
Yielded: staying in WORK. You can end your turn.
</smllm>
$ smllm fire --session sm-nqvfvw pause
<smllm>
session sm-nqvfvw · idle
Paused issue GH-12 at WORK.
State machines:
- dev: Work on an issue, from a plan to a change.
    issue id param issueId (pattern: ^GH-[0-9]+$): The issue's id, such as GH-12.
    starts at: PLAN; entry points: PLAN
Paused:
- issue GH-12 (dev) at WORK
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: dev): The state machine to enter.
    issueId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
$ smllm fire --session sm-nqvfvw enter --param stateMachine=dev --param issueId=GH-12
<smllm>
session sm-nqvfvw · dev › WORK (visit 2) · issue GH-12
Arrived by: enter
Params: stateMachine = "dev", issueId = "GH-12"
<instructions>
Make the change that PLAN.md describes, then fire done.
</instructions>
</smllm>
```

This is the reply to the event that reaches the final state:

```console
$ smllm fire --session sm-nqvfvw done
<smllm>
session sm-nqvfvw · dev › DONE · issue GH-12
Arrived by: done from WORK
<instructions>
Summarise the change for the issue.
</instructions>
Completed issue GH-12. You are now in idle.
State machines:
- dev: Work on an issue, from a plan to a change.
    issue id param issueId (pattern: ^GH-[0-9]+$): The issue's id, such as GH-12.
    starts at: PLAN; entry points: PLAN
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: dev): The state machine to enter.
    issueId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

<!-- sokf:links -->

[sokf:error-agt-1]: /docs/reference/errors/agt-1.md
[sokf:error-agt-2]: /docs/reference/errors/agt-2.md
[sokf:explanation-what-happens-when-the-agent-stops]: /docs/explanation/what-happens-when-the-agent-stops.md
[sokf:reference-harnesses]: /docs/reference/harnesses.md
