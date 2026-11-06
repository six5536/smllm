---
type: HowTo
id: how-to-name-work-by-your-issue-id
title: Name work by your issue id
description: Give each instance of a machine the id of its issue, so that you and the agent find the work by the id that you already use.
order: "40"
---

# Name work by your issue id

When each piece of work has an id in your issue tracker, such as `GH-123`, give the instance that id as its ref. The agent then enters the work by the id, and smllm shows the id in each header and list. You find the instance by the id instead of by its ID, such as `i-ysh871`.

## Steps

The steps use this config and machine. The machine declares the ref in `meta.instance`, where `param` names the param that carries the id. `pattern` sets the form that the id must have, and `description` tells the agent what to pass. `kind` is the word for an instance in the text that the agent reads:

**`.smllm/config.toml`**

```toml
[machines]
files = ["dev.smllm.yaml"]
```

**`.smllm/dev.smllm.yaml`**

```yaml
id: dev
description: Work on one issue, from a plan to a merged pull request.
initial: PLAN
meta:
  smllm: 1
  instance:
    kind: issue
    ref: { param: issue, pattern: "^GH-[0-9]+$", description: "The GitHub issue, such as GH-123." }
  events:
    planned:
      description: Fire after you write the plan.
    merged:
      description: Fire after the pull request is merged.
states:
  PLAN:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Read the issue and write a plan." } }
    on:
      planned: WORK
  WORK:
    entry: { type: prompt, params: { text: "Carry out the plan, and open a pull request." } }
    on:
      merged: DONE
  DONE:
    type: final
```

Write `[0-9]` rather than `\d` in a pattern. In smllm, `\d` matches every Unicode digit, and in a JavaScript host it matches only the ASCII digits.

1. Check the machine:

   ```console
   $ smllm validate
   0 errors, 0 warnings, 0 info
   ```

2. Start a session. The idle list shows the param of the ref, its description, and its pattern, so the agent knows to pass the id:

   <!-- example:
     hook session-start
   -->
   ```text
   <smllm>
   session sm-nqvfvw · idle
   State machines:
   - dev: Work on one issue, from a plan to a merged pull request.
       issue id param issue (pattern: ^GH-[0-9]+$): The GitHub issue, such as GH-123.
       starts at: PLAN; entry points: PLAN
   Fire one event: smllm({ session: "sm-nqvfvw", event, params })
   <events>
   - enter: Start a new instance of a state machine, or continue one.
       stateMachine (required, one of: dev): The state machine to enter.
       issue (optional): The id param that the list of state machines names for the machine: an existing id or ref continues that instance, and a new ref starts one with it. Omit it to start a new instance.
       state (optional): An entry point to enter instead, or any state for an instance whose saved state no longer exists. Omit it to enter a new instance at the initial state, or an existing one at its saved state.
   </events>
   </smllm>
   ```

3. Ask the agent to work on the issue. The agent enters the machine with the issue's id, and the new instance takes the id as its ref:

   ```console
   $ smllm fire --session sm-nqvfvw enter --param stateMachine=dev --param issue=GH-123
   <smllm>
   session sm-nqvfvw · dev › PLAN · issue GH-123
   Arrived by: enter (new issue)
   Params: stateMachine = "dev", issue = "GH-123"
   <instructions>
   Read the issue and write a plan.
   </instructions>
   </smllm>
   ```

   `sm-nqvfvw` is the key of the session. Your key differs.

4. Find the instance by its ref:

   ```console
   $ smllm instance list
   dev  GH-123 (i-ysh871)  PLAN  active  held by sm-nqvfvw
   ```

   `smllm instance show GH-123` shows the instance with its history, and an `enter` with `issue=GH-123` continues it, as [Pause and resume work][sokf:how-to-pause-and-resume-work] shows.

## Troubleshooting

- smllm refuses `enter` with `enter param issue must match ^GH-[0-9]+$ (got "123")`: the id doesn't have the form that `pattern` sets. Pass the whole id, such as `GH-123`, or widen the pattern.
- The id isn't known when the work starts, such as for a bug that has no issue yet: enter without the ref, and name the instance later with a `setRef` action on the transition of an event that carries the ref's param. [Your first state machine][sokf:tutorial-first-machine] names a bug this way.

## See also

- [The machine file][sokf:reference-machine-file]
- [Your first state machine][sokf:tutorial-first-machine]
- [Pause and resume work][sokf:how-to-pause-and-resume-work]
- [Sessions and instances][sokf:explanation-sessions-and-instances]

<!-- sokf:links -->

[sokf:explanation-sessions-and-instances]: /docs/explanation/sessions-and-instances.md
[sokf:how-to-pause-and-resume-work]: /docs/how-to/pause-and-resume-work.md
[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
