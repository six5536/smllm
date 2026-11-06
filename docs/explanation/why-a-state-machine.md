---
type: Explanation
id: explanation-why-a-state-machine
title: Why a state machine
description: Why smllm guides an agent with a state machine rather than with a longer prompt, and what the machine holds the agent to.
order: "10"
---

# Why a state machine

An agent that follows a long prompt can skip a step, and nothing tells it, or you, that it did. smllm gives the agent one step at a time from a state machine, and when the agent tries to stop before it has fired one of the state's events, smllm shows it those events instead.

- [One step at a time](#one-step-at-a-time)
- [A stop that asks for a decision](#a-stop-that-asks-for-a-decision)
- [Checks that the agent doesn't run](#checks-that-the-agent-doesnt-run)
- [A machine that you can check](#a-machine-that-you-can-check)

## One step at a time

A prompt gives the agent the whole workflow at once, and the agent decides which part applies. A state machine gives the agent only the instructions of its current state. When the agent fires an event, smllm moves it to the next state and shows it that state's instructions, so the agent never holds a step that it isn't on.

Each state lists the events that it accepts, and smllm refuses any other. An agent in `TRIAGE` can't fire `fixed`, because `TRIAGE` offers only `reproduced`, so the agent can't skip from the report to the end.

## A stop that asks for a decision

An agent ends its turn when it decides that the work is done. In a state that isn't final, the harness's stop hook asks smllm, and smllm refuses the first stop and shows the agent the events of its state: an event of the machine, `yield` to wait for you, or `pause` to put the work aside. A stop then becomes a decision that the session records, not a step that the agent forgot. If the agent tries to stop again without firing an event, smllm lets it stop, so that an agent that can't go on doesn't loop, and the session's audit shows the stop.

## Checks that the agent doesn't run

A transition can have a guard. A `command` guard runs a command, such as the project's tests, and the transition happens only when the command succeeds. Otherwise smllm takes the next transition of the event, which can keep the agent in its state. The agent can say that the tests pass, but the machine moves on only when they do.

## A machine that you can check

A machine is a file in the project, reviewed like the code. `smllm validate` checks it before an agent runs it: each target state exists, each state can be reached, and each event that the machine declares is used by a state. A prompt can't be checked in this way, because its steps are prose.

The machine is a strict subset of an XState v5 config, so the tools of XState can draw it, and `smllm graph` draws it too.

## See also

- [Your first state machine][sokf:tutorial-first-machine]
- [The rules of smllm validate][sokf:reference-validate-rules]

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
