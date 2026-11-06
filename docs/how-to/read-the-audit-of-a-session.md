---
type: HowTo
id: how-to-read-the-audit-of-a-session
title: Read the audit of a session
description: List every call that drove a session, accepted or refused, to see what the agent did and where smllm stopped it.
order: "100"
---

# Read the audit of a session

When you want to know what an agent did in a session, such as why it skipped a step or how often smllm refused an event, read the session's audit. smllm writes one line for each call that drove the session: each event that the agent fired, each stop that the harness asked about, and the session's start, with the outcome of each.

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

1. Let the agent work in a session. In this session, the agent enters the machine, fires `fixed` too early, tries to stop, and then reproduces the bug:

   ```console
   $ smllm fire --session sm-nqvfvw enter --param stateMachine=fix --json
   {"ok":true,"session":"sm-nqvfvw","location":{"machine":"fix","state":"TRIAGE","instance":"i-419405","ref":null},"text":"<smllm>\nsession sm-nqvfvw · fix › TRIAGE · bug i-419405\nArrived by: enter (new bug)\nParams: stateMachine = \"fix\"\n<instructions>\nWrite a test that reproduces the bug, then fire reproduced with the bug's id.\n</instructions>\n</smllm>\n"}
   $ smllm fire --session sm-nqvfvw fixed --json
   {"ok":false,"session":"sm-nqvfvw","location":{"machine":"fix","state":"TRIAGE","instance":"i-419405","ref":null},"text":"<smllm>\nsession sm-nqvfvw · fix › TRIAGE · bug i-419405\nerror: fixed is not offered here (offered: reproduced, yield, pause, unmatched)\nFire one event: smllm({ session: \"sm-nqvfvw\", event, params })\n<events>\n- reproduced: Fire after a test reproduces the bug.\n    bugId (required, pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.\n- yield: Stop for now and stay in TRIAGE.\n    note (optional): What you are waiting for.\n- pause: Put bug i-419405 aside and return to idle.\n- unmatched: The request fits none of these; handle it from idle, then resume.\n</events>\n</smllm>\n"}
   [exit 1]
   ```

   When the agent tries to stop, Claude Code's stop hook asks smllm, and smllm refuses the stop with this text:

   <!-- example:
     hook stop
   -->
   ```text
   <smllm>
   session sm-nqvfvw · fix › TRIAGE · bug i-419405
   Fire one event: smllm({ session: "sm-nqvfvw", event, params })
   <events>
   - reproduced: Fire after a test reproduces the bug.
       bugId (required, pattern: ^BUG-[0-9]+$): The bug's id, such as BUG-7.
   - yield: Stop for now and stay in TRIAGE.
       note (optional): What you are waiting for.
   - pause: Put bug i-419405 aside and return to idle.
   - unmatched: The request fits none of these; handle it from idle, then resume.
   </events>
   </smllm>
   ```

   ```console
   $ smllm fire --session sm-nqvfvw reproduced --param bugId=BUG-7 --json
   {"ok":true,"session":"sm-nqvfvw","location":{"machine":"fix","state":"FIX","instance":"i-419405","ref":"BUG-7"},"text":"<smllm>\nsession sm-nqvfvw · fix › FIX · bug BUG-7\nArrived by: reproduced from TRIAGE\nParams: bugId = \"BUG-7\"\n<instructions>\nChange the code until the test passes, then fire fixed.\n</instructions>\n</smllm>\n"}
   ```

2. Find the key of the session. Each `<smllm>` header shows it, after `session`, and `smllm session list` lists every session, the most recently active first, with its harness, where it is, when it was last active, and its folder. smllm prints the full path of each folder, and this guide shortens your home folder to `~`:

   ```console
   $ smllm session list
   sm-nqvfvw  claude  fix i-419405  last active 2026-01-05 09:04 UTC  ~/project
   ```

3. Print the audit of the session:

   ```console
   $ smllm session show sm-nqvfvw --audit
   2026-01-05 09:00 UTC  agent  bind  idle  accepted
   2026-01-05 09:01 UTC  agent  fire enter  fix i-419405 idle → TRIAGE  accepted
   2026-01-05 09:02 UTC  agent  fire fixed  fix i-419405 TRIAGE  refused: fixed is not offered here (offered: reproduced, yield, pause, unmatched)
   2026-01-05 09:03 UTC  agent  stop  fix i-419405 TRIAGE  block
   2026-01-05 09:04 UTC  agent  fire reproduced  fix i-419405 TRIAGE → FIX  accepted
   ```

   Each line starts with the time of the call, in UTC, and its source: `agent`, or `host` for an event that only the host that embeds smllm can fire. The call follows: `bind` when a harness binds its session to smllm, `fire` and its event, or `stop` when the harness asks whether the agent can stop. Then come the machine, the instance, and the state before the call. When the call moved the session, `→` and the state after the call follow.

   The outcome ends the line: `accepted`, `refused` with the reason, `block` for a stop that smllm refused, or `runaway` for a stop that smllm allowed because the agent tried to stop again without firing an event.

4. To filter the audit, or to keep it with a report, print it as JSON lines, one object for each call:

   ```console
   $ smllm session show sm-nqvfvw --audit --json
   {"at":1767603600000,"session":"sm-nqvfvw","source":"agent","call":"bind","event":null,"machine":null,"instance":null,"from":null,"to":null,"params":{},"offered":["enter"],"outcome":"accepted","reason":null,"trace":[],"binding":{"harness":"claude","hostSession":"cc-1","cwd":"~/project","configs":["~/project/.smllm/config.toml"]},"reply":"sha256:e32bba214c735287af4ff7a2590c135f3c5c643ebd26df19f906428b1dc4cc23"}
   {"at":1767603660000,"session":"sm-nqvfvw","source":"agent","call":"fire","event":"enter","machine":"fix","instance":"i-419405","from":null,"to":"TRIAGE","params":{"stateMachine":"fix"},"offered":[],"outcome":"accepted","reason":null,"trace":[],"binding":null,"reply":"sha256:ea7f7859c16348a748d815d7479e0a07af296be7572d634718dfc4ca8f1f1255"}
   {"at":1767603720000,"session":"sm-nqvfvw","source":"agent","call":"fire","event":"fixed","machine":"fix","instance":"i-419405","from":"TRIAGE","to":"TRIAGE","params":{},"offered":["reproduced","yield","pause","unmatched"],"outcome":"refused","reason":"fixed is not offered here (offered: reproduced, yield, pause, unmatched)","trace":[],"binding":null,"reply":"sha256:4f74289a359e206952a50a14a262ac1d163cbb48796beadda42644f890f8d96b"}
   {"at":1767603780000,"session":"sm-nqvfvw","source":"agent","call":"stop","event":null,"machine":"fix","instance":"i-419405","from":"TRIAGE","to":"TRIAGE","params":{},"offered":["reproduced","yield","pause","unmatched"],"outcome":"block","reason":null,"trace":[],"binding":null,"reply":"sha256:03ef41907cf05fd767716289eed9eb53feeb34c86ad415595a17dc74bfaf40cf"}
   {"at":1767603840000,"session":"sm-nqvfvw","source":"agent","call":"fire","event":"reproduced","machine":"fix","instance":"i-419405","from":"TRIAGE","to":"FIX","params":{"bugId":"BUG-7"},"offered":[],"outcome":"accepted","reason":null,"trace":[],"binding":null,"reply":"sha256:2b22dc5d256dfc28505f76bbfb423df2232dced33f0096a21fa4c86a30508aaa"}
   ```

   Each object has the fields of the line, and more. `at` holds the time in Unix milliseconds, and `params` the params of the call. `offered` lists the events that the reply offered. `trace` records what the call ran, such as the result of each guard.

   The call that created the session also has `binding`: the harness, its session, the folder, and the configs. That call is a `bind`, or a `fire enter` that started a session without a key.

   This command lists only the calls that smllm didn't accept:

   ```sh
   smllm session show sm-nqvfvw --audit --json | jq -c 'select(.outcome != "accepted") | {call, event, from, outcome, reason}'
   ```

   For the session of step 1, it prints:

   ```json
   {"call":"fire","event":"fixed","from":"TRIAGE","outcome":"refused","reason":"fixed is not offered here (offered: reproduced, yield, pause, unmatched)"}
   {"call":"stop","event":null,"from":"TRIAGE","outcome":"block","reason":null}
   ```

## Troubleshooting

- The audit has no line for a reply's text: each line holds `reply`, the hash of the reply that the agent was shown, as `sha256:` and the hex of its SHA-256, not the text. To check that the agent was shown a text, compare the SHA-256 of the text with `reply`.
- The audit takes space: smllm keeps the file of each session, `audit/KEY.jsonl` in the folder of the sessions, until you remove it. On Linux and macOS the folder is `smllm/` in `$XDG_STATE_HOME`, which is `~/.local/state/` by default. On Windows it is `smllm\` in `%APPDATA%`.

## See also

- [The command][sokf:reference-cli]
- [Commands and options][sokf:reference-commands]
- [Drive smllm from a script][sokf:how-to-drive-smllm-from-a-script]
- [What happens when the agent stops][sokf:explanation-what-happens-when-the-agent-stops]

<!-- sokf:links -->

[sokf:explanation-what-happens-when-the-agent-stops]: /docs/explanation/what-happens-when-the-agent-stops.md
[sokf:how-to-drive-smllm-from-a-script]: /docs/how-to/drive-smllm-from-a-script.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-commands]: /docs/reference/commands.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
