---
type: Schema
id: schema-error-reference
title: Error reference schema
description: The findings that smllm validate reports under one rule code, with their cause, their fix, and an example.
---

<!-- prettier-ignore -->
````yaml
line-limit: 400

guidance: >
  An error page holds the findings of one rule code of `smllm validate`, such as `FMT-1`,
  because a reader who searches for a code lands on its page. A code covers several findings,
  so the page lists the messages that its findings print. It then gives the cause and the
  fix of each, and shows a finding with four things: the files, a run that reports the
  finding, the fixed file, and a run that reports none. The rules page is the index of the
  error pages, and the site lists them under it, in its order. Each file, each run of
  `smllm`, and each block of agent text is checked, because the example test runs them
  and compares the output with the page.

root: /docs/reference/errors

types:
  rule: '[A-Z]+-[1-9][0-9]*'

frontmatter:
  type:
    const: ErrorReference
  id:
    template: 'error-${:slug}'
    guidance: The id is `error-` and the code in lowercase, such as `error-fmt-1`.
  title:
    template: '${code:rule}: ${}'
    guidance: The title is the code, a colon, and the name of its rule on the rules page.
    example: 'FMT-1: The machine file'
  description:
    guidance: The description says which findings the page lists, in one sentence.
    example: The findings that smllm validate reports when a machine file isn't YAML that smllm reads, or has a key or a value that smllm doesn't know.
  code:
    pattern: '^[A-Z]+-[1-9][0-9]*$'
    guidance: The code is the rule code that the findings of the page print.
    example: FMT-1

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says what the findings of the code mean, and links the code on the rules page.
        example: A finding of `FMT-1` means that the machine file isn't YAML that smllm reads, or that one of its keys or values isn't one that smllm knows. The [rules page][sokf:reference-validate-rules] says what the rule checks.
  - heading: Messages
    level: 2
    blocks:
      - kind: code
        lang: [text]
        guidance: The block shows each message as `smllm validate` prints it, one to a line, with each part that changes in angle brackets.
        example: |
          <file>:<line>: error: meta.smllm: unsupported smllm format version <n> — set meta.smllm: 1, the only version that this smllm reads (FMT-1)
  - heading: Cause
    level: 2
    blocks:
      - kind: prose
        repeat: true
        guidance: This section says what in the machine file or the config causes each message.
        example: The file sets `meta.smllm` to a version other than 1, the only version of the machine format.
      - kind: bullet-list
        optional: true
        guidance: A list gives one cause in each item, where the code has several.
        example: A required key, such as `id`, is missing.
  - heading: Fix
    level: 2
    blocks:
      - kind: prose
        repeat: true
        guidance: This section says what to change in the machine file or the config.
        example: Set `meta.smllm` to `1`.
      - kind: bullet-list
        optional: true
        guidance: A list gives one fix in each item, in the order of the causes.
        example: Add the missing key.
  - heading: Example
    level: 2
    guidance: This section shows one finding of the code, then the fix.
    blocks-ordered: false
    blocks:
      - kind: prose
        repeat: true
        guidance: A caption comes before each file, and a sentence before each run.
        example: '**`.smllm/fix.smllm.yaml`**'
      - kind: code
        repeat: true
        lang: [console]
        guidance: The first run reports the finding, and a run after the fix reports none.
        example: |
          $ smllm validate
          0 errors, 0 warnings, 0 info
      - kind: code
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          meta:
            smllm: 1
````
