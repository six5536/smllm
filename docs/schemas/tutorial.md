---
type: Schema
id: schema-tutorial
title: Tutorial schema
description: A step-by-step lesson in which a reader who is new to smllm builds one thing that works.
---

<!-- prettier-ignore -->
````yaml
line-limit: 600

guidance: >
  A tutorial is a lesson. The reader follows it from the first step to the last and finishes
  with one thing that works. Each step builds on the step before it, and each step ends with
  a run that the reader can compare with their own. The tutorial explains only what each step
  needs, and it links to the explanations for the rest. Each file, each run of
  `smllm`, and each block of agent text is checked, because the example test runs them
  and compares the output with the page. A shell snippet that the reader adapts, such as a
  line of a status line script, isn't run. Each file sits under a
  caption line that holds only its name in bold code. Each run of `smllm` is in a `console`
  block with its output, and a run that exits with a code other than 0 ends with
  `[exit N]`. Each step ends with a run or with the text that the agent sees, so that the
  reader can compare the page with their own screen.

root: /docs/tutorials

frontmatter:
  type:
    const: Tutorial
  id:
    template: 'tutorial-${:slug}'
  title:
    example: Your first state machine
  description:
    guidance: The description says what the reader builds, in one sentence.
    example: Write a state machine that guides an agent through a fix, check it, and run it in Claude Code.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the tutorial among the tutorials, smallest first.
    example: '10'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says what the reader builds, and what they can do at the end.
        example: In this tutorial you write a state machine that guides an agent through a fix, and you watch the agent follow it. At the end you can write a machine for a workflow of your own.
  - heading: Before you begin
    level: 2
    blocks:
      - kind: bullet-list
        guidance: This section lists what the reader needs before the first step.
        example:
          - smllm is installed, as the [README](https://github.com/six5536/smllm#install) shows.
          - Claude Code is installed, in a project that is a git repository.
  - heading:
      template: 'Step ${n:int}: ${}'
      example: 'Step 1: create a config'
    level: 2
    repeat: true
    blocks-ordered: false
    blocks:
      - kind: prose
        repeat: true
        guidance: The prose says what the reader does in the step and why. A caption comes before each file.
        example: smllm reads its state machines from the config of the project, so a project starts with one.
      - kind: code
        optional: true
        repeat: true
        lang: [console]
        guidance: >
          This block holds the run that ends the step and its output, as the example test
          writes them. A step that ends with the text that the agent sees shows that text in
          a `text` block instead, after an example comment that produces it.
        example: |
          $ smllm init
          created .smllm/config.toml
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          id: fix
          initial: TRIAGE
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: The list says what the reader can see in the output.
        example: The first line names the file and the line of the finding, and the code at its end names the rule.
  - heading: What's next
    level: 2
    blocks:
      - kind: bullet-list
        guidance: This section says where the reader can go next, with one link in each item.
        each:
          template: '[${}][sokf:${:ref}]'
          example: '[The rules of smllm validate][sokf:reference-validate-rules]'
````
