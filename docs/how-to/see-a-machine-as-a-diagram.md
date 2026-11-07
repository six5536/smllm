---
type: HowTo
id: how-to-see-a-machine-as-a-diagram
title: See a machine as a diagram
description: Print the states and transitions of a machine as text or as a Mermaid diagram, open it in your browser, or write it to an HTML page or a PNG picture.
order: "70"
---

# See a machine as a diagram

When you review a machine, or show it to someone who doesn't read YAML, draw it with `smllm graph`. The command prints the states and transitions as text or as a Mermaid diagram, opens an interactive diagram in your browser, or writes the diagram to a file.

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

1. Print the machine as text. Each state lists its transitions, with the event, the target, and the actions:

   ```console
   $ smllm graph fix
   fix (initial TRIAGE)
     TRIAGE [entry point]
       reproduced → FIX (setRef)
     FIX
       fixed → DONE
     DONE [final]
   ```

   Without an ID, `smllm graph` prints every machine of the config.

2. To put the diagram in a Markdown file, such as a pull request or a README, print it as Mermaid:

   ```console
   $ smllm graph fix --mermaid
   ---
   title: fix
   ---
   stateDiagram-v2
       [*] --> TRIAGE
       TRIAGE --> FIX: reproduced
       FIX --> DONE: fixed
       DONE --> [*]
   ```

   GitHub and GitLab draw a fenced code block of language `mermaid` as a diagram.

3. To explore the machine, open it in your browser:

   ```sh
   smllm graph fix --open
   ```

   The page shows the diagram. Select a state or a transition to read its prompts, commands, guards, and events. A note beside the diagram lists the events that every state but a final one offers, `yield`, `pause`, and `unmatched`, and the `resume` of the fallback state. Select the note to read their guidance. smllm opens the page with the default browser, or with the program that the variable `BROWSER` names.

4. To share the interactive diagram, write it to an HTML page, which needs only a browser:

   ```console
   $ smllm graph fix --html fix.html
   wrote fix.html
   ```

5. To put the diagram in a slide or a document, write it as a PNG picture:

   ```console
   $ smllm graph fix --png fix.png
   wrote fix.png
   ```

   When you name no ID and the config has several machines, each machine goes to a file of its own: `smllm graph --png diagram.png` writes `diagram-ID.png` for each machine `ID`, such as `diagram-fix.png`.

## See also

- [The command][sokf:reference-cli]
- [Commands and options][sokf:reference-commands]
- [Your first state machine][sokf:tutorial-first-machine]

<!-- sokf:links -->

[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-commands]: /docs/reference/commands.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
