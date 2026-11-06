---
type: Tutorial
id: tutorial-connect-an-agent
title: Connect an agent
description: Install smllm into Claude Code, and follow a state machine as the agent sees it, from the start of a session to the end of the work.
order: "20"
---

# Connect an agent

In this tutorial you install smllm into Claude Code for a project with a state machine, and you follow the machine as the agent sees it: the text at the start of a session, the reply to each event, and a stop that smllm blocks. At the end you can connect smllm to a project of your own and read what the agent reads.

## Before you begin

- You have written and checked a machine, as [Your first state machine][sokf:tutorial-first-machine] shows.
- smllm is installed, as the [README](https://github.com/six5536/smllm#install) shows.
- Claude Code is installed.
- You have a new project folder for this tutorial, which is a git repository. A new folder has no smllm files yet, so each run prints what the page shows.

## Step 1: add a machine to the project

The agent can follow only the machines that the project's config lists. Write a config that lists one machine, and the machine itself. The machine guides the agent through a small feature: in `PLAN` the agent writes a plan, in `BUILD` it builds the feature, and `DONE` ends the work.

**`.smllm/config.toml`**

```toml
[machines]
files = ["feature.smllm.yaml"]
```

**`.smllm/feature.smllm.yaml`**

```yaml
id: feature
description: Build a small feature, from a plan to working code.
initial: PLAN
meta:
  smllm: 1
  instance:
    kind: feature
  events:
    planned:
      description: Fire after the user agrees to the plan.
    built:
      description: Fire after the feature works and its tests pass.
states:
  PLAN:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Write a short plan for the feature, and ask the user to agree to it." } }
    on:
      planned: BUILD
  BUILD:
    entry: { type: prompt, params: { text: "Build the feature as the plan says, with a test for it." } }
    on:
      built: DONE
  DONE:
    type: final
    entry: { type: prompt, params: { text: "Tell the user what changed." } }
```

Check the machine before an agent runs it:

```console
$ smllm validate
0 errors, 0 warnings, 0 info
```

## Step 2: install smllm into Claude Code

`smllm harness install claude` writes the files that connect Claude Code to smllm. Run it in the project's folder:

```console
$ smllm harness install claude
claude:
  created CLAUDE.md (instructions)
  created .claude/settings.json (hooks)
  created .mcp.json (mcp)
  created .claude/settings.json (permissions)
  created .claude/skills/smllm-statusline (statusline)
note: smllm is not in your Claude Code status line; ask Claude to add it (skill smllm-statusline)
```

Each line names a part and the file that holds it:

- The instructions in `CLAUDE.md` tell the agent to follow the text in `<smllm>` blocks, and to fire an event when its work is done.
- The hooks in `.claude/settings.json` run smllm when a session starts, when you send a prompt, and when the agent tries to stop.
- `.mcp.json` gives the agent the smllm tool, with which it fires events, and the permission lets the agent call the tool without asking you.
- The skill `smllm-statusline` adds smllm to your status line when you ask Claude to, as [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line] shows.

Commit these files, so that everyone who works on the project gets the same setup. After you upgrade smllm, run the command again: `smllm harness status claude` shows whether each part is current.

```console
$ smllm harness status claude
claude:
  current CLAUDE.md (instructions)
  current .claude/settings.json (hooks)
  current .mcp.json (mcp)
  current .claude/settings.json (permissions)
  current .claude/skills/smllm-statusline (statusline)
note: smllm is not in your Claude Code status line; ask Claude to add it (skill smllm-statusline)
```

## Step 3: start a session

Start Claude Code in the project, and approve the smllm server when Claude Code asks. The session start hook gives the agent a session and the list of machines that it can enter. The agent sees this text before your first prompt:

<!-- example:
  hook session-start
-->
```text
<smllm>
session sm-nqvfvw · idle
State machines:
- feature: Build a small feature, from a plan to working code.
    feature id param ref
    starts at: PLAN; entry points: PLAN
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: feature): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

`sm-nqvfvw` is the key of the session. Your key differs. The agent passes the key on every call of the smllm tool.

## Step 4: ask for the work

Ask Claude for a feature, such as "Add a `--verbose` flag to the build script". The agent picks the machine whose description fits the request, and fires `enter` through the smllm tool. The tool's reply is the text that `smllm fire` prints for the same event, so this tutorial shows each call of the agent as a run of `smllm fire`. You don't run these commands: the agent makes the calls.

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=feature
<smllm>
session sm-nqvfvw · feature › PLAN · feature i-x96snr
Arrived by: enter (new feature)
Params: stateMachine = "feature"
<instructions>
Write a short plan for the feature, and ask the user to agree to it.
</instructions>
</smllm>
```

The agent now writes a plan and asks you to agree to it.

## Step 5: watch a blocked stop

The agent tries to stop, to wait for your answer. Before it stops, the stop hook checks where it is. The agent fired no event, so smllm blocks the stop and gives the agent the events that it can fire:

<!-- example:
  hook stop
-->
```text
<smllm>
session sm-nqvfvw · feature › PLAN · feature i-x96snr
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after the user agrees to the plan.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put feature i-x96snr aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

The agent fires `yield`, because it is waiting for you, and then stops. A second stop without an event is allowed, so that an agent that can't go on doesn't loop.

## Step 6: come back to the work

When Claude Code resumes the conversation, or compacts it, the session start hook runs again for the same session. This time the agent sees where it is, with the state's instructions and its events:

<!-- example:
  fire yield
  hook session-start resume
-->
```text
<smllm>
session sm-nqvfvw · feature › PLAN · feature i-x96snr
<instructions>
Write a short plan for the feature, and ask the user to agree to it.
</instructions>
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- planned: Fire after the user agrees to the plan.
- yield: Stop for now and stay in PLAN.
    note (optional): What you are waiting for.
- pause: Put feature i-x96snr aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

## Step 7: finish the feature

Agree to the plan. The agent fires `planned`, builds the feature, and fires `built`:

```console
$ smllm fire --session sm-nqvfvw planned
<smllm>
session sm-nqvfvw · feature › BUILD · feature i-x96snr
Arrived by: planned from PLAN
<instructions>
Build the feature as the plan says, with a test for it.
</instructions>
</smllm>
$ smllm fire --session sm-nqvfvw built
<smllm>
session sm-nqvfvw · feature › DONE · feature i-x96snr
Arrived by: built from BUILD
<instructions>
Tell the user what changed.
</instructions>
Completed feature i-x96snr. You are now in idle.
State machines:
- feature: Build a small feature, from a plan to working code.
    feature id param ref
    starts at: PLAN; entry points: PLAN
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: feature): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

The two replies show how a machine ends:

- Each reply starts with the state that the event entered, and with how the agent got there.
- `DONE` is a final state, so the feature is complete, and the session returns to idle with the list of machines.

## What's next

- [Check the agent's work][sokf:tutorial-check-the-agents-work]
- [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line]
- [Harnesses][sokf:reference-harnesses]

<!-- sokf:links -->

[sokf:how-to-show-smllm-in-the-status-line]: /docs/how-to/show-smllm-in-the-status-line.md
[sokf:reference-harnesses]: /docs/reference/harnesses.md
[sokf:tutorial-check-the-agents-work]: /docs/tutorials/check-the-agents-work.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
