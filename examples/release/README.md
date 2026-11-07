# Release example

This example is a machine that releases one version of a Rust project. It shows a chain of
checks that the agent can't skip, a limit on retries, and a step that waits for a person.

The machine works in these steps:

1. In `NOTES`, the agent adds the version's section to `CHANGELOG.md` and fires `noted`. A
   `command` guard checks that the section's heading exists.
2. `FORMAT`, `LINT`, and `TEST` are eventless states, which the machine passes through on its
   own. Each runs one check, with `cargo fmt`, `cargo clippy`, and `cargo test`, and moves to the
   next check when it passes. The first check that fails moves the agent to `FIX`.
3. In `FIX`, the agent fixes the failure and fires `fixed`, and the checks run again from the
   start. After three visits of `FIX`, the machine moves to `STUCK`, where the agent asks the
   user how to go on.
4. In `SIGN_OFF`, the agent shows the user the changelog and asks whether to release. The agent
   fires the user's answer: `approved` goes on, and `declined` goes back to `NOTES`.
5. `PUBLISH` tags the version and pushes the tag. If either fails, the machine moves to `STUCK`.

The example has these files:

- `release.smllm.yaml` is the machine.
- `prompts/notes.md` and `prompts/sign-off.md` hold the instructions of `NOTES` and `SIGN_OFF`.
- `config.toml` lists the machine.

To check the machine and see its graph, run these commands from the root of the repository:

```sh
smllm validate examples/release/config.toml
smllm --config examples/release/config.toml graph release
```
