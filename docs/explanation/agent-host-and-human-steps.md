---
type: Explanation
id: explanation-agent-host-and-human-steps
title: Agent, host, and human steps
description: Who moves a state machine from state to state, the agent, smllm's own commands, the host, or a person, and how one machine mixes them.
order: "40"
---

# Agent, host, and human steps

Who moves a state machine from state to state? The agent fires most events, but not every step of the work is the agent's to take. A machine can run commands that the agent can't skip, leave an event to the program that hosts smllm, and wait for a person through `yield`.

- [Steps that the agent takes](#steps-that-the-agent-takes)
- [Steps that smllm runs](#steps-that-smllm-runs)
- [Steps that the host takes](#steps-that-the-host-takes)
- [Steps that a person takes](#steps-that-a-person-takes)
- [Machines that mix them](#machines-that-mix-them)

## Steps that the agent takes

An event belongs to the agent unless the machine says otherwise: its `source` in `meta.events` is `agent`, the default. The agent fires it with the smllm tool, or with `smllm fire` in a harness without the tool. smllm accepts the event only when the state offers it and its params are valid, and the session's audit records the call with the source `agent`.

You can fire the same events yourself with `smllm fire` from a shell, as [Your first state machine][sokf:tutorial-first-machine] does. smllm treats your call as the agent's, because `smllm fire` makes the same call as the smllm tool.

## Steps that smllm runs

A guard or an action of the type `command` runs a command, such as the project's tests. smllm runs it as part of the transition, so the agent can't skip it, and the agent can't change its result.

- A `command` guard chooses the transition. When the agent fires the event, smllm takes the first transition whose guard passes, and a guard passes when its command exits with `0`. The agent can say that the tests pass, but the machine moves on only when they do. The block that the agent gets shows each guard's result.
- A `command` action runs when the instance enters or leaves a state, or takes a transition. An action never stops a transition. A failed action appears in the block as `Action failed`, and smllm skips the commands after it in the same list.
- A state with `always` transitions is left as soon as the instance enters it, by the first transition whose guard passes. A check can then run with no event at all.

Each command gets the session, the machine, the state, the event, the instance, its ref, and the event's params in its environment, in variables such as `SMLLM_REF` and `SMLLM_PARAM_SIZE`. Rule [`CLI-11`][sokf:error-cli-11] says what `smllm validate` checks in a command.

## Steps that the host takes

A host is a program that embeds smllm's engine, such as a web application that runs `@six5536/smllm-wasm`. A machine marks an event with `source: host` when only that program can decide it, for example from the result of its own checks. The host fires the event with `fireAsHost`.

The agent never sees a host event: the events list leaves it out, and smllm refuses it when the agent fires it. Under a harness such as Claude Code, no program fires host events, so an instance of the machine would wait for one that never comes. smllm therefore leaves such a machine out of the idle list under a harness, refuses to start an instance of it there, and `smllm validate` gives the machine an info finding under rule [`FMT-4`][sokf:error-fmt-4].

A state can also name the host's work with XState's `invoke`, such as `invoke: { src: runChecks, input: { strict: true } }`. smllm runs nothing: the host's view of the state lists each `invoke`, and the host runs the work that `src` names and fires its result as a host event. A host that runs several machines then finds its code by the name of the work, not by the names of the states. A state with an `invoke` must handle an event of `source: host`, which rule [`FMT-2`][sokf:error-fmt-2] checks, so that the work's result always has a way back into the machine.

## Steps that a person takes

A person doesn't fire an event through the tool. Instead, the agent waits for the person with `yield`, and the person answers with their next prompt. The agent then fires the event that the answer calls for.

A machine models a decision of a person, such as an approval, as a state of its own. The state's instructions ask the agent to show the work, fire `yield`, and fire `approved` after the person approves. The decision is the person's, and the event is the agent's. The status line shows `yielded` while the agent waits, so that you see that the next step is yours. [What happens when the agent stops][sokf:explanation-what-happens-when-the-agent-stops] explains how `yield` lets the agent stop.

## Machines that mix them

One machine can give each step to the one who should take it. In this machine, the agent plans, a person approves the plan, the tests decide whether the change is done, and the host reviews it:

**`.smllm/config.toml`**

```toml
[machines]
files = ["change.smllm.yaml"]
```

**`.smllm/change.smllm.yaml`**

```yaml
id: change
description: Make a change that the user approves and the host checks.
initial: PLAN
meta:
  smllm: 1
  events:
    planned: { description: Fire after you write the plan. }
    approved: { description: Fire after the user approves the plan. }
    done: { description: Fire after you make the change. }
    checked: { description: "The host fires this after the change passes its review.", source: host }
states:
  PLAN:
    entry: { type: prompt, params: { text: "Write a plan for the change, then fire planned." } }
    on:
      planned: APPROVE
  APPROVE:
    entry: { type: prompt, params: { text: "Show the plan to the user, then fire yield and wait. Fire approved after the user approves the plan." } }
    on:
      approved: WORK
  WORK:
    entry: { type: prompt, params: { text: "Make the change, then fire done." } }
    on:
      done:
        - { target: REVIEW, guard: { type: command, params: { run: "cargo test" } } }
        - { target: WORK, reenter: true }
  REVIEW:
    entry: { type: prompt, params: { text: "Wait while the host reviews the change." } }
    on:
      checked: DONE
  DONE:
    type: final
```

`smllm validate` says that the machine needs a host:

```console
$ smllm validate --info
.smllm/change.smllm.yaml:6: info: meta.events: only the host fires the events checked, so the machine needs a host to drive it. A harness can't start an instance of it, and a host that embeds the engine fires these events (FMT-4)
0 errors, 0 warnings, 1 info
```

Under a harness, and in `smllm fire`, smllm refuses to start it:

```console
$ smllm fire --session sm-nqvfvw enter --param stateMachine=change
<smllm>
session sm-nqvfvw · idle
error: state machine change needs a host to drive it: only the host fires its events checked
No state machines are configured.
Fire one event: smllm({ session: "sm-nqvfvw", event, params })
<events>
- enter: Start a new instance of a state machine, or continue one.
    stateMachine (required): The state machine to enter.
    state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
</events>
</smllm>
[exit 1]
```

Without `REVIEW` and its host event, with `done` leading straight to `DONE`, the same machine runs under any harness, and the agent, the tests, and the person share the work.

## See also

- [The machine file][sokf:reference-machine-file]
- [The agent's text][sokf:reference-agent-text]
- [The rules of smllm validate][sokf:reference-validate-rules]
- [What happens when the agent stops][sokf:explanation-what-happens-when-the-agent-stops]

<!-- sokf:links -->

[sokf:error-cli-11]: /docs/reference/errors/cli-11.md
[sokf:error-fmt-2]: /docs/reference/errors/fmt-2.md
[sokf:error-fmt-4]: /docs/reference/errors/fmt-4.md
[sokf:explanation-what-happens-when-the-agent-stops]: /docs/explanation/what-happens-when-the-agent-stops.md
[sokf:reference-agent-text]: /docs/reference/agent-text.md
[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
