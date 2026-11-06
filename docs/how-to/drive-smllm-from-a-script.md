---
type: HowTo
id: how-to-drive-smllm-from-a-script
title: Drive smllm from a script
description: Start a session, fire events, and read where the session is from a shell script or from a harness without MCP, with JSON output and exit codes.
order: "90"
---

# Drive smllm from a script

When an agent runs in a harness that has no MCP, or when a script or a test drives a machine, call `smllm fire` from the shell. `smllm fire` does what the smllm tool does: it fires one event, prints the reply that the agent reads, and exits with a code that says whether smllm accepted the event.

## Before you begin

- `jq` is installed, for the shell snippets that read the JSON.

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

1. Start a session: fire `enter` without `--session`. With `--json`, smllm prints one JSON object. Its field `ok` says whether smllm accepted the event, and `session` holds the session's key. `location` says where the session is, and `text` holds the reply that the agent reads:

   ```console
   $ smllm fire enter --param stateMachine=fix --json
   {"ok":true,"session":"sm-rpnmhe","location":{"machine":"fix","state":"TRIAGE","instance":"i-bzj1bm","ref":null},"text":"<smllm>\nsession sm-rpnmhe · fix › TRIAGE · bug i-bzj1bm\nArrived by: enter (new bug)\nParams: stateMachine = \"fix\"\n<instructions>\nWrite a test that reproduces the bug, then fire reproduced with the bug's id.\n</instructions>\n</smllm>\n"}
   ```

   In a script, keep the key and hand the text to the agent:

   ```bash
   reply=$(smllm fire enter --param stateMachine=fix --json)
   key=$(jq -r .session <<<"$reply")
   jq -r .text <<<"$reply"
   ```

2. Fire each event with the key. Each `--param` gives one param, as `NAME=VALUE`:

   ```console
   $ smllm fire --session sm-rpnmhe reproduced --param bugId=BUG-7 --json
   {"ok":true,"session":"sm-rpnmhe","location":{"machine":"fix","state":"FIX","instance":"i-bzj1bm","ref":"BUG-7"},"text":"<smllm>\nsession sm-rpnmhe · fix › FIX · bug BUG-7\nArrived by: reproduced from TRIAGE\nParams: bugId = \"BUG-7\"\n<instructions>\nChange the code until the test passes, then fire fixed.\n</instructions>\n</smllm>\n"}
   ```

3. Check the exit code of each run. smllm exits with `1` when it refuses the event, and the reply says why and lists the events that the state offers:

   ```console
   $ smllm fire --session sm-rpnmhe reproduced --param bugId=BUG-8
   <smllm>
   session sm-rpnmhe · fix › FIX · bug BUG-7
   error: reproduced is not offered here (offered: fixed, yield, pause, unmatched)
   Fire one event: smllm({ session: "sm-rpnmhe", event, params })
   <events>
   - fixed: Fire after the test passes.
   - yield: Stop for now and stay in FIX.
       note (optional): What you are waiting for.
   - pause: Put bug BUG-7 aside and return to idle.
   - unmatched: The request fits none of these; handle it from idle, then resume.
   </events>
   </smllm>
   [exit 1]
   ```

   smllm exits with `2` on a usage error, such as a key that names no session. It prints the error to standard error, and prints no reply:

   ```console
   $ smllm fire --session sm-zzzzzz fixed
   error: no smllm session sm-zzzzzz: pass the key shown in the latest <smllm> header, or fire enter with no session to start a new one
   [exit 2]
   ```

4. To see where a session is without firing an event, show it. The reply is the one that the smllm tool gives when the agent calls it with no event:

   ```console
   $ smllm session show sm-rpnmhe
   <smllm>
   session sm-rpnmhe · fix › FIX · bug BUG-7
   <instructions>
   Change the code until the test passes, then fire fixed.
   </instructions>
   Fire one event: smllm({ session: "sm-rpnmhe", event, params })
   <events>
   - fixed: Fire after the test passes.
   - yield: Stop for now and stay in FIX.
       note (optional): What you are waiting for.
   - pause: Put bug BUG-7 aside and return to idle.
   - unmatched: The request fits none of these; handle it from idle, then resume.
   </events>
   </smllm>
   ```

   `smllm session show --json` prints the same object as `smllm fire --json`.

5. Put the steps together. This bash script shows each reply and fires the event that you type, with its params, until the session is idle again:

   ```bash
   reply=$(smllm fire enter --param stateMachine=fix --json)
   key=$(jq -r .session <<<"$reply")
   while [ "$(jq -r '.location.state // empty' <<<"$reply")" != "" ]; do
     jq -r .text <<<"$reply"
     read -r -p 'event and params: ' event params
     args=()
     for p in $params; do args+=(--param "$p"); done
     reply=$(smllm fire --session "$key" "$event" "${args[@]}" --json)
   done
   jq -r .text <<<"$reply"
   ```

   `location.state` is `null` when the session is idle. A run that smllm refuses exits with `1` and still prints the reply, so the loop shows why and asks again. In a harness, the agent's reply takes the place of `read`.

## Troubleshooting

- smllm refuses `fire` with `no session given`: only `enter` starts a session. Pass `--session` with the key of an earlier reply for every other event.
- The agent of a harness doesn't see the instructions: `smllm fire` prints the reply, and the harness has to show it to the agent. Give the agent the `text` of the JSON, or the whole output without `--json`.

## See also

- [The command][sokf:reference-cli]
- [Commands and options][sokf:reference-commands]
- [Harnesses][sokf:reference-harnesses]
- [Read the audit of a session][sokf:how-to-read-the-audit-of-a-session]
- [The agent's text][sokf:reference-agent-text]

<!-- sokf:links -->

[sokf:how-to-read-the-audit-of-a-session]: /docs/how-to/read-the-audit-of-a-session.md
[sokf:reference-agent-text]: /docs/reference/agent-text.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-commands]: /docs/reference/commands.md
[sokf:reference-harnesses]: /docs/reference/harnesses.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
