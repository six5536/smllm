---
type: Tutorial
id: tutorial-first-machine
title: Your first state machine
description: Write a state machine that guides an agent through a bug fix, check it, and drive it as the agent does.
order: "10"
---

# Your first state machine

In this tutorial you write a state machine that guides an agent through a bug fix, from a test that reproduces the bug to the fix. You check the machine, then fire its events from the shell, as the agent fires them through the smllm tool. At the end you can write a machine for a workflow of your own.

## Before you begin

- smllm is installed, as the [README](https://github.com/six5536/smllm#install) shows.
- Claude Code is installed, and you have run `smllm harness install claude` in your project's folder to connect smllm to it, as the install section of [Harnesses][sokf:reference-harnesses] shows.
- You have a project folder that is a git repository.

## Step 1: create a config

smllm reads its state machines from the config of the project, so a project starts with one. Run this in the project's folder:

```console
$ smllm init
created .smllm/config.toml
```

## Step 2: scaffold a machine

`smllm new` writes a template machine and lists it in the config, so that smllm finds it:

```console
$ smllm new fix --write
created .smllm/fix.smllm.yaml and listed it in .smllm/config.toml
```

## Step 3: write the states and check them

Replace the template with this machine. It has three states. In `TRIAGE` the agent writes a test that reproduces the bug, in `FIX` it makes the test pass, and `DONE` ends the work. Each event moves the agent to the next state, and `setRef` names the instance after the bug.

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
description: Fix a bug, from a test that reproduces it to the fix.
initial: TRIAGE
meta:
  smllm: 1
  instance:
    kind: bug
    ref: { param: bugId, pattern: "^BUG-[0-9]+$", description: "The bug's id, such as BUG-7." }
  events:
    reproduced:
      description: Fire after a test reproduces the bug.
    fixed:
      description: Fire after the test passes.
states:
  TRIAGE:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Write a test that reproduces the bug, then fire reproduced with the bug's id." } }
    on:
      reproduced: { target: FIX, actions: setRef }
  FIX:
    entry: { type: prompt, params: { text: "Change the code until the test passes, then fire fixed." } }
    on:
      fixed: DONE
  DONE:
    type: final
    entry: { type: prompt, params: { text: "Summarise the cause of the bug and the fix." } }
```

`smllm validate` checks every machine of the config before an agent runs it:

```console
$ smllm validate
0 errors, 0 warnings, 0 info
```

A finding names the file and the line of the problem, and ends with the code of the rule that it breaks, such as `(FMT-1)`. [The rules of smllm validate][sokf:reference-validate-rules] says what each rule checks.

## Step 4: start a session

Start Claude Code in the project. smllm gives the agent a session and lists the machines that it can enter. The agent sees this:

<!-- example:
  hook session-start
-->
```text
<smllm>
session sm-nqvfvw · idle
State machines:
- fix: Fix a bug, from a test that reproduces it to the fix.
    bug id param bugId (pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.
    starts at: TRIAGE; entry points: TRIAGE
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: fix): The state machine to enter.
    bugId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

`sm-nqvfvw` is the key of the session. Your key differs.

## Step 5: enter the machine

The agent enters the machine with the event `enter`. Fire it from the shell, with your session's key:

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=fix
<smllm>
session sm-nqvfvw · fix › TRIAGE · bug i-5z4x7n
Arrived by: enter (new bug)
Params: stateMachine = "fix"
<instructions>
Write a test that reproduces the bug, then fire reproduced with the bug's id.
</instructions>
</smllm>
```

The block is what the agent reads: where it is, how it got there, and what the state asks.

## Step 6: try to stop

When the agent tries to stop in `TRIAGE`, smllm refuses, and lists the events that the agent can fire instead:

<!-- example:
  hook stop
-->
```text
<smllm>
session sm-nqvfvw · fix › TRIAGE · bug i-5z4x7n
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- reproduced: Fire after a test reproduces the bug.
    bugId (required, pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.
- yield: Stop for now and stay in TRIAGE.
    note (optional): What you are waiting for.
- pause: Put bug i-5z4x7n aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
```

The agent fires one of these events before it stops: `yield` waits for you, and `pause` puts the bug aside. If the agent tries to stop again without firing one, smllm lets it stop, so that an agent that can't go on doesn't loop.

## Step 7: finish the fix

Fire the two events of the machine, as the agent does after each step of the work:

```console
$ smllm fire --session sm-nqvfvw reproduced --param bugId=BUG-7
<smllm>
session sm-nqvfvw · fix › FIX · bug BUG-7
Arrived by: reproduced from TRIAGE
Params: bugId = "BUG-7"
<instructions>
Change the code until the test passes, then fire fixed.
</instructions>
</smllm>
$ smllm fire --session sm-nqvfvw fixed
<smllm>
session sm-nqvfvw · fix › DONE · bug BUG-7
Arrived by: fixed from FIX
<instructions>
Summarise the cause of the bug and the fix.
</instructions>
Completed bug BUG-7. You are now in idle.
State machines:
- fix: Fix a bug, from a test that reproduces it to the fix.
    bug id param bugId (pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.
    starts at: TRIAGE; entry points: TRIAGE
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: fix): The state machine to enter.
    bugId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

The two replies show how the instance ends:

- The instance takes the name `BUG-7` from `setRef`.
- `DONE` is a final state, so the bug is complete, and the session returns to idle with the list of machines that it can enter.

## What's next

- [Connect an agent][sokf:tutorial-connect-an-agent]
- [Check the agent's work][sokf:tutorial-check-the-agents-work]
- [Why a state machine][sokf:explanation-why-a-state-machine]
- [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line]
- [FMT-1: The machine file][sokf:error-fmt-1]
- [The rules of smllm validate][sokf:reference-validate-rules]

<!-- sokf:links -->

[sokf:error-fmt-1]: /docs/reference/errors/fmt-1.md
[sokf:explanation-why-a-state-machine]: /docs/explanation/why-a-state-machine.md
[sokf:how-to-show-smllm-in-the-status-line]: /docs/how-to/show-smllm-in-the-status-line.md
[sokf:reference-harnesses]: /docs/reference/harnesses.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
[sokf:tutorial-check-the-agents-work]: /docs/tutorials/check-the-agents-work.md
[sokf:tutorial-connect-an-agent]: /docs/tutorials/connect-an-agent.md
