---
type: HowTo
id: how-to-show-smllm-in-the-status-line
title: Show smllm in your status line
description: Add a row to Claude Code's status line that shows the state machine, the state, and the instance of the session.
order: "10"
---

# Show smllm in your status line

When you want to see where an agent's work is without asking it, show smllm in Claude Code's status line. The row shows the state machine, the state, and the instance of the session, such as `smllm dev › WORK (visit 3) · issue GH-123`.

To let Claude make the change, ask it to "add smllm to my status line". The `smllm-statusline` skill makes the change and shows it to you first. `smllm harness install claude` installs the skill, and so does `smllm harness install all`, unless you add `--without statusline`. The Claude Code plugin also ships the skill. To make the change by hand, follow these steps.

## Steps

1. If you have no status line script yet, create `~/.claude/statusline-command.sh`:

   ```sh
   #!/usr/bin/env bash
   input=$(cat)
   printf '%s' "$input" | smllm statusline
   ```

   Then point Claude Code at the script in `~/.claude/settings.json`, and skip the next step:

   ```json
   { "statusLine": { "type": "command", "command": "bash ~/.claude/statusline-command.sh" } }
   ```

2. If you have a script, add this line at the end of it, after its other output:

   ```sh
   if row=$(printf '%s' "$input" | smllm statusline 2>/dev/null) && [ -n "$row" ]; then printf '\n%s' "$row"; fi
   ```

   `$input` is the status JSON that your script read from stdin (`input=$(cat)`). Claude Code runs one `statusLine` command, and each line that the command prints is a row.

3. Check the setup:

   ```sh
   smllm harness status claude
   ```

   The command reminds you when your status line doesn't call `smllm statusline`.

## Troubleshooting

- The whole status line goes blank: Claude Code blanks it when the script exits with a non-zero status. Keep the `if` form of the line in step 2, because a trailing `[ -n "$row" ] && printf …` exits with a non-zero status whenever smllm has nothing to show.
- The row doesn't appear: smllm prints nothing when the project has no smllm config, or before Claude Code's session start hook has bound a session to smllm. The row appears as soon as Claude Code starts a session in a project with a config.
- `smllm harness install` didn't add the row: it never changes the `statusLine` setting, because the setting is yours, and a setting at project level would override the status lines of your collaborators.

## See also

- [The status line row][sokf:reference-statusline]
- [Build a status line row of your own][sokf:how-to-build-a-status-line-row]

<!-- sokf:links -->

[sokf:how-to-build-a-status-line-row]: /docs/how-to/build-a-status-line-row.md
[sokf:reference-statusline]: /docs/reference/statusline.md
