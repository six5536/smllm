---
type: Explanation
id: explanation-what-happens-when-the-agent-stops
title: What happens when the agent stops
description: Why smllm blocks the agent's stop in a state that isn't final, which stops it lets through, and what changes in a harness that can't block a stop.
order: "30"
---

# What happens when the agent stops

What does smllm do when the agent tries to end its turn? In a state that isn't final, smllm blocks the stop and shows the agent the events of its state, so that the agent fires one that says what happens to the work: it goes on, waits for you, or puts the work aside. smllm lets the stop through in idle, after `yield`, and when the agent tries to stop a second time in a row without firing an event.

- [The stop hook](#the-stop-hook)
- [A blocked stop asks for a decision](#a-blocked-stop-asks-for-a-decision)
- [Yield and pause](#yield-and-pause)
- [Idle stops freely](#idle-stops-freely)
- [A second stop goes through](#a-second-stop-goes-through)
- [Harnesses that can't block a stop](#harnesses-that-cant-block-a-stop)

## The stop hook

The harness runs its stop hook each time the agent tries to end its turn. The hook asks smllm about the agent's session, and smllm answers in one of two ways. It lets the stop through, and the turn ends. Or it blocks the stop with a `<smllm>` block, and the harness gives the block to the agent and keeps the agent going.

The block of a blocked stop is the header and the events list of the state, without the state's instructions. [The agent's text][sokf:reference-agent-text#a-blocked-stop] shows one.

## A blocked stop asks for a decision

An agent ends its turn when its model stops calling tools. That point often comes too early: the agent leaves out a step, or it stops to report what it did before the work is finished. smllm blocks the stop because the machine's state, not the agent's text, records whether the work is finished.

The block names the events that the agent can fire. They are the state's own events, which move the work on, and the built-in events `yield`, `pause`, and `unmatched`. Each one is a decision about the work, and smllm records each in the instance's history and in the session's audit. A stop is then a choice that you can read later, and not a step that the agent skipped without a trace.

## Yield and pause

Two of the built-in events let the agent stop:

- `yield` keeps the instance in its state, and lets the agent stop until your next prompt. The agent fires it to ask you a question, or to wait for something that only you can do. Its optional `note` says what the agent waits for. The status line shows `yielded`, so that you see that the agent waits for you. Your next prompt, or the next transition, ends the yield, and smllm then blocks a stop again.
- `pause` puts the instance aside and returns the session to idle, where the agent stops freely. The agent fires it when the work has to wait longer, for example for a review. Any later session continues the work with `enter`, as [Sessions and instances][sokf:explanation-sessions-and-instances] explains.

`unmatched` also returns the session to idle when the machine has no fallback state, so that the agent can handle a request that fits none of the events.

## Idle stops freely

A session in idle holds no instance, so it has no work to finish, and smllm lets every stop through. A session also reaches idle after its instance enters a final state. The final state's instructions, such as a request for a summary, are the last text that the agent gets for the work, and the agent then stops when it likes.

smllm also lets the agent stop when the machine of the session's instance is no longer in the config, for example while its file has an error. The instance keeps its saved state, and the agent goes on after the file is fixed.

## A second stop goes through

When the agent tries to stop again straight after a blocked stop, without firing an event, smllm lets the stop through. The harness tells the hook that the agent is going on only because of the earlier block, and the session records that no event was fired since. An agent that can't go on, for example because it is stuck, would otherwise loop between the stop and the block without end.

smllm records this stop as `runaway` in the session's audit, which `smllm session show KEY --audit` prints, and writes the events list to standard error. The instance stays in its state. Your next prompt clears the record of the block, so that the next stop is blocked again.

## Harnesses that can't block a stop

Some harnesses can't block a stop, as [Harnesses][sokf:reference-harnesses] lists, and each lets every stop through for its own reason:

- GitHub Copilot runs the hooks, but its session start hook can't give the agent a session, so smllm binds none. The stop hook finds no session, and allows the stop without a word.
- The `agents` integration installs no hooks, so no hook runs at a stop.
- In `opencode run`, smllm blocks the stop as it does in OpenCode's TUI, but the run ends with the agent's turn, so the agent never reads the events list.

The machine still holds the agent to its states and its events. The agent learns to fire `yield` before it stops only from the instructions block that `smllm harness install` writes.

## See also

- [The agent's text][sokf:reference-agent-text]
- [Harnesses][sokf:reference-harnesses]
- [Why a state machine][sokf:explanation-why-a-state-machine]
- [Sessions and instances][sokf:explanation-sessions-and-instances]

<!-- sokf:links -->

[sokf:explanation-sessions-and-instances]: /docs/explanation/sessions-and-instances.md
[sokf:explanation-why-a-state-machine]: /docs/explanation/why-a-state-machine.md
[sokf:reference-agent-text]: /docs/reference/agent-text.md
[sokf:reference-agent-text#a-blocked-stop]: /docs/reference/agent-text.md#a-blocked-stop
[sokf:reference-harnesses]: /docs/reference/harnesses.md
