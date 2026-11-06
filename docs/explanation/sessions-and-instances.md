---
type: Explanation
id: explanation-sessions-and-instances
title: Sessions and instances
description: What a session and an instance are, why smllm keeps the work apart from the conversation, and how several sessions share the work.
order: "20"
---

# Sessions and instances

What is the difference between a session and an instance, and why does smllm keep the two apart? A session is one conversation of the agent, and an instance is one piece of work, such as an issue, that moves through a state machine. smllm keeps them apart so that the work outlasts the conversation: a later conversation continues the issue in the state where the last one left it.

- [A session is a conversation](#a-session-is-a-conversation)
- [An instance is a piece of work](#an-instance-is-a-piece-of-work)
- [A ref names the work](#a-ref-names-the-work)
- [Work that outlasts a session](#work-that-outlasts-a-session)
- [Work put aside](#work-put-aside)
- [Several sessions](#several-sessions)

## A session is a conversation

When a conversation of the harness starts, the session start hook gives it an smllm session with a key, such as `sm-nqvfvw`. smllm records which conversation of the harness the key belongs to, so every later hook of the same conversation finds the same session. A conversation that the harness resumes or compacts keeps its session, and the hook shows the agent where the session is.

Pi, OpenCode, and Kilo Code take text only with a prompt, so their agent gets the session with the first prompt. GitHub Copilot and the `agents` integration give the agent no key: the agent fires `enter` without one, and smllm then makes the session.

The key appears in the header of every block, and the agent passes it on every call of the smllm tool. Only the main agent calls the tool, and it never passes the key to a subagent, because the session belongs to the main agent's conversation.

A session records its harness, its working folder, its configs, the instance that it holds, and whether the agent yielded. It holds at most one instance at a time. A session that holds no instance is in idle, where the agent sees the machines that it can enter. smllm keeps the sessions in your user's state folder, with an audit of every call that drove each session. On Linux and macOS the folder is `~/.local/state/smllm/` unless you set `$XDG_STATE_HOME`, and on Windows it is `%APPDATA%\smllm\`.

## An instance is a piece of work

An instance is one run of a state machine over one piece of work. `enter` starts it, with a generated ID such as `i-k0xzjt`, in the machine's initial state or in an entry point. Each event moves it, and a final state completes it.

An instance has a status: `active` while a session holds it, `paused` or `interrupted` after the agent puts it aside, and `completed` after it reaches a final state. It counts the visits to each state, and its history records each event, with the session that fired it, the states before and after, the params, and the results of the guards.

smllm keeps the instances in a `state/` folder beside the config of their machine, which is `.smllm/state/` for the project's config, and git ignores the folder. The instances belong to the config, and the sessions belong to you. `smllm instance list` lists the instances, and `smllm instance show` shows one with its history.

## A ref names the work

A ref is the work's own name outside smllm, such as the issue `GH-12`. The machine names the param that carries the ref in `meta.instance.ref`, and it can give the ref a pattern. The ref is set once, by an `enter` that names a new ref, or by a `setRef` action of a transition. No two instances of a machine have the same ref.

After the ref is set, smllm shows the instance by its ref instead of its ID: in the header, in the idle list, and in the status line. The agent and you know the issue by its number, so `enter` with the number finds the instance in any session, and `smllm instance show` finds it by the number too.

## Work that outlasts a session

A conversation ends, but its instance stays in its state, on disk. A new conversation gets a new session, which starts in idle. The agent continues the work by firing `enter` with the ref or the ID, and smllm returns the instance to its saved state, with the state's instructions.

The idle list names the paused instances, so the agent sees the work that it paused in each new session. An instance that another session still holds isn't in that list, and the agent continues it with its ref. A completed instance stays on disk too, and `enter` with an entry point as its `state` reopens it, with its visits and its history.

## Work put aside

The agent puts an instance aside in two ways:

- `pause` sets the instance aside to continue later, and returns the session to idle. The idle list shows the 10 most recently updated paused instances, and `listPaused` lists them all. `enter` with the ref continues one.
- `unmatched` handles a request that fits none of the state's events. When the machine has a fallback state, the instance moves there and stays active, and `resume` returns it to the state that it left. Otherwise the instance becomes interrupted, the session goes to idle, and `resume` from idle returns to it.

A session has one interrupted instance at most. When `unmatched` interrupts a second instance, the first one becomes paused, so that no work is lost.

## Several sessions

You can run several conversations at once, in one project or in several. Each has its own session and its own key, and each holds its own instance.

Two sessions never hold the same instance. When the agent of one session fires `enter` for an instance that another session holds, the instance moves to the new session, and the new session's block notes `Took over from session KEY`. The next event or stop of the old session returns it to idle, with the error that the instance moved to the new session.

Each write of an instance carries a version, and smllm refuses a write that another session made stale. When two sessions change one instance at the same moment, the second change is refused, and its session returns to idle. Neither session overwrites the other's work.

`smllm session list` lists the sessions, with the instance that each holds.

## See also

- [The agent's text][sokf:reference-agent-text]
- [The command][sokf:reference-cli]
- [What happens when the agent stops][sokf:explanation-what-happens-when-the-agent-stops]
- [Your first state machine][sokf:tutorial-first-machine]

<!-- sokf:links -->

[sokf:explanation-what-happens-when-the-agent-stops]: /docs/explanation/what-happens-when-the-agent-stops.md
[sokf:reference-agent-text]: /docs/reference/agent-text.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
