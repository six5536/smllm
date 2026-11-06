---
type: HowTo
id: how-to-use-your-machines-in-every-project
title: Use your machines in every project
description: Keep state machines in your user config, so that every project on your computer can run them.
order: "50"
---

# Use your machines in every project

When you use the same workflow in many projects, such as a review that you do on every pull request, put its machine in your user config. smllm combines the user config with the config of each project, so every session on your computer can enter the machine, and a project needs no copy of it.

## Steps

1. Create the user config:

   ```console
   $ smllm init --user
   created ~/.config/smllm/config.toml
   ```

   The user config is `~/.config/smllm/config.toml` on Linux and macOS, and `%APPDATA%\smllm\config.toml` on Windows. smllm prints the full path, and this guide shortens your home folder to `~`. The following steps use the path of Linux and macOS.

2. Write a template machine beside the user config, and list it there:

   ```console
   $ smllm new review --write --config ~/.config/smllm/config.toml
   created ~/.config/smllm/review.smllm.yaml and listed it in ~/.config/smllm/config.toml
   ```

   `--config` makes `smllm new` write `review.smllm.yaml` beside that config, `~/.config/smllm/review.smllm.yaml`, and list it in the config's `[machines]` table. Replace the template with your machine, as in [Your first state machine][sokf:tutorial-first-machine]. This guide uses a short review machine:

   **`~/.config/smllm/review.smllm.yaml`**

   ```yaml
   id: review
   initial: READ
   meta:
     smllm: 1
   states:
     READ:
       on:
         read: COMMENT
     COMMENT:
       on:
         commented: DONE
     DONE:
       type: final
   ```

   The command lists the machine in the user config:

   **`~/.config/smllm/config.toml`**

   ```toml
   [machines]
   files = ["review.smllm.yaml"]
   ```

3. Check the machine from any project:

   ```console
   $ smllm validate
   0 errors, 0 warnings, 0 info
   $ smllm graph review
   review (initial READ)
     READ
       read → COMMENT
     COMMENT
       commented → DONE
     DONE [final]
   ```

   Both commands read the user config as well as the project's, and so does every session. A folder without a config of its own runs the machines of the user config alone.

4. To change the machine for one project, define a machine with the same ID in the project's config. Create the project's config and a template machine `review`:

   ```console
   $ smllm init
   created .smllm/config.toml
   $ smllm new review --write
   created .smllm/review.smllm.yaml and listed it in .smllm/config.toml
   ```

   Replace the template with the project's machine. This one adds a step that runs the tests:

   **`.smllm/review.smllm.yaml`**

   ```yaml
   id: review
   initial: READ
   meta:
     smllm: 1
   states:
     READ:
       on:
         read: TEST
     TEST:
       on:
         passed: COMMENT
     COMMENT:
       on:
         commented: DONE
     DONE:
       type: final
   ```

   The project's machine replaces the user's, and `smllm validate --info` reports which one it replaces:

   ```console
   $ smllm validate --info
   .smllm/review.smllm.yaml: info: machine review here replaces the one in ~/.config/smllm/review.smllm.yaml (FMT-7)
   0 errors, 0 warnings, 1 info
   ```

## Troubleshooting

- An instance of a user machine shows in `smllm instance list` of every project: smllm keeps the instances of each config beside that config, in its `state` folder, so the instances of a user machine are in `~/.config/smllm/state/`. To keep them elsewhere, set `dir` in the `[state]` table of the user config, relative to the config file.
- A machine of the user config doesn't appear: the project defines a machine with the same ID, which replaces it. Run `smllm validate --info` to see the replacement.

## See also

- [The command][sokf:reference-cli]
- [Commands and options][sokf:reference-commands]
- [Your first state machine][sokf:tutorial-first-machine]

<!-- sokf:links -->

[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-commands]: /docs/reference/commands.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
