---
type: HowTo
id: how-to-pause-and-resume-work
title: Pause and resume work
description: Put an instance aside, find it in the idle list, and pick it up again later, in the same session or in another.
order: "30"
---

# Pause and resume work

When an urgent task interrupts the work of an agent, pause the instance that it works on, and resume the instance when the urgent task is done. smllm keeps the paused instance at its state, so the agent, in this session or in another, continues where it stopped.

## Steps

The steps use this config and the machine of [Your first state machine][sokf:tutorial-first-machine]:

**`.smllm/config.toml`**

```toml
[machines]
files = ["fix.smllm.yaml"]
```

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

1. Start work on a bug. The agent enters the machine with the bug's id, as it does when you ask it to fix `BUG-7`:

   ```console
   $ smllm fire --session sm-nqvfvw enter --param stateMachine=fix --param bugId=BUG-7
   <smllm>
   session sm-nqvfvw · fix › TRIAGE · bug BUG-7
   Arrived by: enter (new bug)
   Params: stateMachine = "fix", bugId = "BUG-7"
   <instructions>
   Write a test that reproduces the bug, then fire reproduced with the bug's id.
   </instructions>
   </smllm>
   ```

   `sm-nqvfvw` is the key of the session. Your key differs.

2. To pause the bug, ask the agent to put it aside. The agent fires `pause`, and smllm returns the session to idle:

   ```console
   $ smllm fire --session sm-nqvfvw pause
   <smllm>
   session sm-nqvfvw · idle
   Paused bug BUG-7 at TRIAGE.
   State machines:
   - fix: Fix a bug, from a test that reproduces it to the fix.
       bug id param bugId (pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.
       starts at: TRIAGE; entry points: TRIAGE
   Paused:
   - bug BUG-7 (fix) at TRIAGE
   Fire one event: smllm({ session: "sm-nqvfvw", event, params })
   <events>
   - enter: Start a new instance of a state machine, or continue one.
       stateMachine (required, one of: fix): The state machine to enter.
       bugId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
       state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
   </events>
   </smllm>
   ```

   The idle list shows the bug under `Paused:`, so the agent sees it each time the session is idle. The session is free for the urgent task.

3. List the instances of the project, to see the paused bug from the shell:

   ```console
   $ smllm instance list
   fix  BUG-7 (i-ex2bq1)  TRIAGE  paused
   ```

   Each line shows the machine, the instance's ref and its ID, its state, and its status. To read the history of the bug, run `smllm instance show` with its ref or its ID:

   ```console
   $ smllm instance show BUG-7
   fix BUG-7 (i-ex2bq1)
   state: TRIAGE  status: paused  holder: -
   visits: TRIAGE=1
   history:
     2026-01-05 09:01 UTC  sm-nqvfvw  enter  → TRIAGE
         bugId = BUG-7
         stateMachine = fix
     2026-01-05 09:02 UTC  sm-nqvfvw  pause  TRIAGE →
         Paused bug BUG-7 at TRIAGE.
   ```

4. To resume the bug, ask the agent to continue it. The agent fires `enter` with the bug's id, and the bug continues at its saved state:

   ```console
   $ smllm fire --session sm-nqvfvw enter --param stateMachine=fix --param bugId=BUG-7
   <smllm>
   session sm-nqvfvw · fix › TRIAGE (visit 2) · bug BUG-7
   Arrived by: enter
   Params: stateMachine = "fix", bugId = "BUG-7"
   <instructions>
   Write a test that reproduces the bug, then fire reproduced with the bug's id.
   </instructions>
   </smllm>
   ```

5. To pick the bug up in another session, such as a new Claude Code session the next day, pause it in this session first:

   ```console
   $ smllm fire --session sm-nqvfvw pause --json
   {"ok":true,"session":"sm-nqvfvw","location":{"machine":null,"state":null,"instance":null,"ref":null},"text":"<smllm>\nsession sm-nqvfvw · idle\nPaused bug BUG-7 at TRIAGE.\nState machines:\n- fix: Fix a bug, from a test that reproduces it to the fix.\n    bug id param bugId (pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.\n    starts at: TRIAGE; entry points: TRIAGE\nPaused:\n- bug BUG-7 (fix) at TRIAGE\nFire one event: smllm({ session: \"sm-nqvfvw\", event, params })\n<events>\n- enter: Start a new instance of a state machine, or continue one.\n    stateMachine (required, one of: fix): The state machine to enter.\n    bugId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.\n    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.\n</events>\n</smllm>\n"}
   ```

   Then the agent of the other session enters the bug, as in step 4. From the shell, `smllm fire enter` without `--session` starts a new session, as a second agent would:

   ```console
   $ smllm fire enter --param stateMachine=fix --param bugId=BUG-7
   <smllm>
   session sm-6yv68n · fix › TRIAGE (visit 3) · bug BUG-7
   Arrived by: enter
   Params: stateMachine = "fix", bugId = "BUG-7"
   <instructions>
   Write a test that reproduces the bug, then fire reproduced with the bug's id.
   </instructions>
   </smllm>
   ```

   A session that enters a bug that another session still holds takes the bug over, and the reply names the session that held it. Pause first, so that the first agent doesn't go on working on the bug.

6. To reopen a bug that is complete, enter it at one of its entry points. Finish the bug first:

   ```console
   $ smllm fire --session sm-6yv68n reproduced --param bugId=BUG-7 --json
   {"ok":true,"session":"sm-6yv68n","location":{"machine":"fix","state":"FIX","instance":"i-ex2bq1","ref":"BUG-7"},"text":"<smllm>\nsession sm-6yv68n · fix › FIX · bug BUG-7\nArrived by: reproduced from TRIAGE\nParams: bugId = \"BUG-7\"\n<instructions>\nChange the code until the test passes, then fire fixed.\n</instructions>\n</smllm>\n"}
   $ smllm fire --session sm-6yv68n fixed --json
   {"ok":true,"session":"sm-6yv68n","location":{"machine":null,"state":null,"instance":null,"ref":null},"text":"<smllm>\nsession sm-6yv68n · fix › DONE · bug BUG-7\nArrived by: fixed from FIX\n<instructions>\nSummarise the cause of the bug and the fix.\n</instructions>\nCompleted bug BUG-7. You are now in idle.\nState machines:\n- fix: Fix a bug, from a test that reproduces it to the fix.\n    bug id param bugId (pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.\n    starts at: TRIAGE; entry points: TRIAGE\nFire one event: smllm({ session: \"sm-6yv68n\", event, params })\n<events>\n- enter: Start a new instance of a state machine, or continue one.\n    stateMachine (required, one of: fix): The state machine to enter.\n    bugId (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.\n    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.\n</events>\n</smllm>\n"}
   $ smllm instance list
   fix  BUG-7 (i-ex2bq1)  DONE  completed
   ```

   Then enter the bug with `state`, the entry point to reopen it at:

   ```console
   $ smllm fire --session sm-6yv68n enter --param stateMachine=fix --param bugId=BUG-7 --param state=TRIAGE
   <smllm>
   session sm-6yv68n · fix › TRIAGE (visit 4) · bug BUG-7
   Arrived by: enter (reopened from DONE)
   Params: stateMachine = "fix", bugId = "BUG-7", state = "TRIAGE"
   Reopened bug BUG-7 at TRIAGE.
   <instructions>
   Write a test that reproduces the bug, then fire reproduced with the bug's id.
   </instructions>
   </smllm>
   ```

   The bug keeps its history and its visits, and `TRIAGE` counts its fourth visit, because each `enter` that continues or reopens the bug is a visit of its state.

## Troubleshooting

- smllm refuses `enter` with `bug BUG-7 is completed. To reopen it, fire enter with state: TRIAGE`: the bug is complete, and `enter` without `state` doesn't reopen it. Add `state` with an entry point that the error names.
- `pause` isn't offered: the session is idle, and holds no instance to pause. Run `smllm instance list` to see which instances are paused.

## See also

- [The command][sokf:reference-cli]
- [Commands and options][sokf:reference-commands]
- [Name work by your issue id][sokf:how-to-name-work-by-your-issue-id]
- [Sessions and instances][sokf:explanation-sessions-and-instances]

<!-- sokf:links -->

[sokf:explanation-sessions-and-instances]: /docs/explanation/sessions-and-instances.md
[sokf:how-to-name-work-by-your-issue-id]: /docs/how-to/name-work-by-your-issue-id.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-commands]: /docs/reference/commands.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
