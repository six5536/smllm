---
type: HowTo
id: how-to-build-a-status-line-row
title: Build a status line row of your own
description: Read the JSON fields of `smllm statusline` and print a row of your own in place of the default.
order: "20"
---

# Build a status line row of your own

When the default row shows more or less than you want, build a row of your own from the JSON fields of `smllm statusline`. You read the fields with `jq` and print the row with `printf`.

## Before you begin

- A status line script that Claude Code runs, as in [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line].
- `jq` is installed.

## Steps

1. Read the fields of the session into a variable, in your status line script:

   ```sh
   s=$(printf '%s' "$input" | smllm statusline --json)
   ```

   `$input` is the status JSON that your script read from stdin. [The status line row][sokf:reference-statusline] lists each field.

2. Print the row from the fields. This row shows only the state, in bold yellow:

   ```sh
   state=$(jq -r '.state // empty' <<<"$s")
   if [ -n "$state" ]; then printf '\n\033[1;33m%s\033[0m' "$state"; fi
   ```

   This one puts the state and the instance on the same row as your other fields, and shows nothing when the session is idle:

   ```sh
   where=$(jq -r 'select(.state) | "\(.state) \(.instance.label)"' <<<"$s")
   printf '%s%s' "$my_other_fields" "${where:+ | $where}"
   ```

   This one shows a warning colour while the agent has yielded:

   ```sh
   if [ "$(jq -r '.yielded' <<<"$s")" = true ]; then
     printf '\n\033[43;30m waiting for you \033[0m'
   fi
   ```

## See also

- [The status line row][sokf:reference-statusline]

<!-- sokf:links -->

[sokf:how-to-show-smllm-in-the-status-line]: /docs/how-to/show-smllm-in-the-status-line.md
[sokf:reference-statusline]: /docs/reference/statusline.md
