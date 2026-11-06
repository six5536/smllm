---
type: Schema
id: schema-reference
title: Reference schema
description: A page where a reader looks up facts, such as the forms of the status line row or the rules of smllm validate.
---

<!-- prettier-ignore -->
````yaml
line-limit: 800

guidance: >
  A reference page lists facts for a reader to look up. It is complete, follows a fixed
  order, and tells no story. A generated page names its generator. A change to a generated
  page goes into the input of its generator, not into the page, because the generator
  rewrites the page. The findings of `smllm validate` have pages of their own type.

root: /docs/reference

frontmatter:
  type:
    const: Reference
  id:
    template: 'reference-${:slug}'
  title:
    example: The status line row
  description:
    guidance: The description says what the reader looks up on the page, in one sentence.
    example: The forms of the row that `smllm statusline` prints, its options, and its JSON fields.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the page in the reference, smallest first.
    example: '10'
  generator:
    optional: true
    enum: [info-rules, info-schema, man-markdown]
    guidance: The generator names the command that writes the body of the page, when a command writes it.
    example: info-rules

sections:
  - heading: { from: title }
    level: 1
    subheadings-open: true
    blocks-ordered: false
    blocks:
      - kind: prose
        repeat: true
        guidance: The opening says what the page lists, and the order of the list.
        example: This page lists the two forms of the row that `smllm statusline` prints, then its options and its JSON fields.
      - kind: table
        optional: true
        columns: [Code, Rule, What validate checks]
        guidance: >
          The table of the rules page lists each rule with its code, its name, and what it
          checks. Any other table sits under a subheading, whose body is open.
        example: |
          | Code | Rule | What validate checks |
          | ---- | ---- | -------------------- |
          | `FMT-6` | Findings | smllm can read each file that the config names. |
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block shows a form or an output as smllm prints it.
        example: |
          smllm dev › WORK (visit 3) · issue GH-123
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: A list gives facts that have one dimension, such as the meaning of each part of a form.
        example: '`MACHINE` is the state machine, and `STATE` is its current state.'
````
