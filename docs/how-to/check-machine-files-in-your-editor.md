---
type: HowTo
id: how-to-check-machine-files-in-your-editor
title: Check machine files in your editor
description: Point your editor at the JSON Schema of the machine format, so that it completes the keys of a machine file and marks a wrong key as you type.
order: "80"
---

# Check machine files in your editor

When you write a machine file, let your editor check it as you type, before you run `smllm validate`. smllm describes the machine format as a JSON Schema. An editor that reads JSON Schema for YAML then completes each key, shows its description, and marks a key that the format doesn't have.

## Before you begin

- An editor that runs the YAML language server, such as Visual Studio Code with the YAML extension by Red Hat.

## Steps

1. Start each machine from `smllm new`. Its template begins with a comment that tells the YAML language server where the schema is:

   ```console
   $ smllm new review
   # yaml-language-server: $schema=https://raw.githubusercontent.com/six5536/smllm/main/schema/smllm.schema.json
   # An XState v5 machine config, in the subset that smllm reads. smllm's own data is under `meta`.
   id: review
   description: What this state machine is for.
   initial: START
   meta:
     smllm: 1
     instance:
       kind: task
       ref:
         param: taskId
         description: "The task's id."
     events:
       done:
         description: Fire when the work that this state asks for is finished.
   states:
     START:
       meta: { entryPoint: true }
       entry: { type: prompt, params: { text: "Describe what the agent should do here." } }
       on:
         done: DONE
     DONE:
       type: final
       entry: { type: prompt, params: { text: "Summarise what you did." } }
   ```

   The comment points at the schema of the latest smllm. For a machine file that you wrote without the template, copy the first line into it.

2. To check against the schema of the smllm that you have installed, and to work offline, write the schema into the project:

   ```sh
   smllm info schema > .smllm/smllm.schema.json
   ```

   Then point the comment of each machine file at that file, by a path relative to the machine file:

   ```yaml
   # yaml-language-server: $schema=./smllm.schema.json
   ```

   Run the command again after you upgrade smllm, because each version prints the schema of its own format.

3. To check every machine file without a comment in each, map the schema to the files' names in your editor's settings. In Visual Studio Code, add this to `.vscode/settings.json` in the project:

   ```json
   { "yaml.schemas": { "./.smllm/smllm.schema.json": "**/*.smllm.yaml" } }
   ```

   The key is the schema's path, relative to the folder that you opened, or a URL, such as the URL of step 1. The value is the pattern of the files that the schema checks.

4. Open a machine file, and type a key under `states.START`. The editor offers the keys of a state, such as `entry`, `on`, and `type`, and marks a misspelt key, such as `entri`.

## Troubleshooting

- The editor marks nothing: the comment of the file comes before `yaml.schemas`, so a comment with a wrong path turns the check off. Check the path of the comment, or remove the comment to use the setting.
- The editor accepts a file that `smllm validate` rejects: the schema describes the keys of the file and their types, and `smllm validate` checks more, such as that each target names a state. Run `smllm validate` before an agent runs the machine.

## See also

- [The machine file][sokf:reference-machine-file]
- [The rules of smllm validate][sokf:reference-validate-rules]

<!-- sokf:links -->

[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
