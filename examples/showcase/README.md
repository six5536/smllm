# Showcase example

This example puts every feature of smllm's machine format into one machine, so that you can look
up how a feature is written. Its workflow is small on purpose: a draft is written, checked,
reviewed, and approved or sent back. The comments of `showcase.smllm.yaml` explain each feature
where it appears.

The machine shows these features:

- An instance kind and a ref with a pattern, recorded with `setRef`.
- Events declared once, with string params, an `enum`, and descriptions for each state.
- Reworded guidance for the built-in events `yield` and `unmatched`.
- Shared actions that run before or after the entry of the states that they name.
- Prompts from inline text and from files in subfolders.
- A self transition that re-enters its state.
- An eventless state whose `command` and `visits` guards pick the next state.
- An action that runs a command as an argument list.
- Entry points, a fallback state, a state without transitions, and a final state.

The example has these files:

- `showcase.smllm.yaml` is the machine.
- `instructions/draft.md` and `shared/review-checklist.md` hold its prompts.
- `config.toml` lists the machine, and sets the text that smllm shows when no machine runs.

To check the machine and see its graph, run these commands from the root of the repository:

```sh
smllm --config examples/showcase/config.toml validate
smllm --config examples/showcase/config.toml graph showcase
```
