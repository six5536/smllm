---
type: Tutorial
id: tutorial-check-the-agents-work
title: Check the agent's work
description: Write a state machine that runs a test before the agent can finish, sends the agent to the user after repeated failures, and records why the test failed.
order: "30"
---

# Check the agent's work

In this tutorial you write a state machine that checks the agent's work with a command, which the agent can't skip. The agent finishes only when the test passes, and after two failed attempts the machine sends it to you for help. At the end you can guard any step of a machine with a check of your own.

## Before you begin

- You have written and run a machine, as [Your first state machine][sokf:tutorial-first-machine] shows.
- smllm is installed, as the [README](https://github.com/six5536/smllm#install) shows.
- You have a project folder that is a git repository.

## Step 1: write the test and the machine

A command guard runs a command, and passes when the command exits with status 0. Any test of your project can be the command. This tutorial uses a test in a shell script, which checks that `greeting.txt` says `hello`. Write the script, and a `greeting.txt` that fails it:

**`test.sh`**

```sh
grep -qx "hello" greeting.txt || { echo "greeting.txt doesn't say hello"; exit 1; }
```

**`greeting.txt`**

```text
hi
```

Write the config, which lists the machine:

**`.smllm/config.toml`**

```toml
[machines]
files = ["fix.smllm.yaml"]
```

The machine has three states. In `WORK` the agent makes the test pass, in `ASK` it asks you what to try, and `DONE` ends the work. The event `fixed` takes the param `cause`, in which the agent says what made the test fail.

**`.smllm/fix.smllm.yaml`**

```yaml
id: fix
description: Make the failing test pass.
initial: WORK
meta:
  smllm: 1
  events:
    fixed:
      description: Fire after you change the code, with the cause of the failure.
      params:
        type: object
        properties:
          cause: { description: "What made the test fail, in one sentence." }
        required: [cause]
    retry:
      description: Fire after the user tells you what to try.
states:
  WORK:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Change the code until sh test.sh passes, then fire fixed." } }
    on:
      fixed:
        - target: DONE
          guard: { type: command, params: { run: "sh test.sh" } }
        - target: ASK
          guard: { type: visits, params: { state: WORK, atLeast: 2 } }
        - target: WORK
          reenter: true
  ASK:
    entry: { type: prompt, params: { text: "The test still fails. Ask the user what to try, then fire retry." } }
    on:
      retry: WORK
  DONE:
    type: final
    entry: { type: prompt, params: { text: "Tell the user the cause of the failure." } }
```

The event `fixed` has three transitions, and smllm takes the first whose guard passes:

- The command guard runs `sh test.sh` in the session's folder, and goes to `DONE` when the test passes.
- The `visits` guard counts how many times the agent has entered `WORK`, and goes to `ASK` from the second visit.
- The last transition has no guard. It leaves `WORK` and enters it again, which counts as a visit.

Check the machine:

```console
$ smllm validate
0 errors, 0 warnings, 0 info
```

## Step 2: start the work

Enter the machine, as the agent does through the smllm tool. Without `--session`, `smllm fire enter` starts a new session, and prints its key in the header:

```console
$ smllm fire enter --param stateMachine=fix
<smllm>
session sm-efajqg · fix › WORK · instance i-2q5chp
Arrived by: enter (new instance)
Params: stateMachine = "fix"
<instructions>
Change the code until sh test.sh passes, then fire fixed.
</instructions>
</smllm>
```

`sm-efajqg` is the key of the new session, and `i-2q5chp` is the instance. Your key and your instance differ: use yours in each of the following runs.

## Step 3: fire an event without its param

The param `cause` is required, so smllm refuses `fixed` without it. The agent stays in `WORK`, and the reply lists the events and their params:

```console
$ smllm fire --session sm-efajqg fixed
<smllm>
session sm-efajqg · fix › WORK · instance i-2q5chp
error: fixed needs param cause
Fire one event: smllm({ session: "sm-efajqg", event, params })
<events>
- fixed: Fire after you change the code, with the cause of the failure.
    cause (required): What made the test fail, in one sentence.
- yield: Stop for now and stay in WORK.
    note (optional): What you are waiting for.
- pause: Put instance i-2q5chp aside and return to idle.
- unmatched: The request fits none of these; handle it from idle, then resume.
</events>
</smllm>
[exit 1]
```

## Step 4: claim a fix that fails the test

Fire `fixed` with a cause, but leave `greeting.txt` as it is. The test fails, so the agent goes back to `WORK`:

```console
$ smllm fire --session sm-efajqg fixed --param "cause=The greeting was too short."
<smllm>
session sm-efajqg · fix › WORK (visit 2) · instance i-2q5chp
Arrived by: fixed from WORK
Params: cause = "The greeting was too short."
Guard: command `sh test.sh` → false (exited 1: greeting.txt doesn't say hello)
Guard: visits WORK ≥ 2 → false (visits: 1)
<instructions>
Change the code until sh test.sh passes, then fire fixed.
</instructions>
</smllm>
```

The reply shows the guards and the visit:

- Each `Guard:` line shows a guard that smllm checked, and its result. A failed command adds the end of its output, so that the agent can see why the test failed.
- `(visit 2)` in the header says that this is the second visit to `WORK`.

## Step 5: fail the test again

Fire `fixed` again without a change. This time the `visits` guard passes, and the machine sends the agent to you:

```console
$ smllm fire --session sm-efajqg fixed --param "cause=The greeting was too short."
<smllm>
session sm-efajqg · fix › ASK · instance i-2q5chp
Arrived by: fixed from WORK
Params: cause = "The greeting was too short."
Guard: command `sh test.sh` → false (exited 1: greeting.txt doesn't say hello)
Guard: visits WORK ≥ 2 → true (visits: 2)
<instructions>
The test still fails. Ask the user what to try, then fire retry.
</instructions>
</smllm>
```

## Step 6: make the test pass

Tell the agent what to try, and fire `retry`, as the agent does after your answer:

```console
$ smllm fire --session sm-efajqg retry
<smllm>
session sm-efajqg · fix › WORK (visit 3) · instance i-2q5chp
Arrived by: retry from ASK
<instructions>
Change the code until sh test.sh passes, then fire fixed.
</instructions>
</smllm>
```

Change `greeting.txt` so that it passes the test:

**`greeting.txt`**

```text
hello
```

Fire `fixed` once more. The test passes, so the agent reaches `DONE`:

```console
$ smllm fire --session sm-efajqg fixed --param "cause=The greeting said hi, not hello."
<smllm>
session sm-efajqg · fix › DONE · instance i-2q5chp
Arrived by: fixed from WORK
Params: cause = "The greeting said hi, not hello."
Guard: command `sh test.sh` → true
<instructions>
Tell the user the cause of the failure.
</instructions>
Completed instance i-2q5chp. You are now in idle.
State machines:
- fix: Make the failing test pass.
    instance id param ref
    starts at: WORK; entry points: WORK
Fire one event: smllm({ session: "sm-efajqg", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required, one of: fix): The state machine to enter.
    ref (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
```

The cause is now in the instance's history. `smllm instance show` prints each event with its time, its params, and its guards:

```console
$ smllm instance show i-2q5chp
fix i-2q5chp (i-2q5chp)
state: DONE  status: completed  holder: -
visits: ASK=1 DONE=1 WORK=3
history:
  2026-01-05 09:01 UTC  sm-efajqg  enter  → WORK
      stateMachine = fix
  2026-01-05 09:04 UTC  sm-efajqg  fixed  WORK → WORK
      cause = The greeting was too short.
      Guard: command `sh test.sh` → false (exited 1: greeting.txt doesn't say hello)
      Guard: visits WORK ≥ 2 → false (visits: 1)
  2026-01-05 09:05 UTC  sm-efajqg  fixed  WORK → ASK
      cause = The greeting was too short.
      Guard: command `sh test.sh` → false (exited 1: greeting.txt doesn't say hello)
      Guard: visits WORK ≥ 2 → true (visits: 2)
  2026-01-05 09:06 UTC  sm-efajqg  retry  ASK → WORK
  2026-01-05 09:07 UTC  sm-efajqg  fixed  WORK → DONE
      cause = The greeting said hi, not hello.
      Guard: command `sh test.sh` → true
```

## What's next

- [The machine file][sokf:reference-machine-file]
- [Run the engine in Node.js][sokf:tutorial-run-the-engine-in-node]
- [Why a state machine][sokf:explanation-why-a-state-machine]

<!-- sokf:links -->

[sokf:explanation-why-a-state-machine]: /docs/explanation/why-a-state-machine.md
[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
[sokf:tutorial-run-the-engine-in-node]: /docs/tutorials/run-the-engine-in-node.md
