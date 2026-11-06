---
type: Schema
id: schema-front-page
title: Front page schema
description: The page that the site opens on, which says who each section is for and where each reader starts.
---

<!-- prettier-ignore -->
````yaml
line-limit: 100

guidance: >
  The front page is the first page of the site. It says what smllm is in a sentence or two,
  then gives each kind of reader the page to start from, then lists the sections of the
  documentation. It holds no rule and no example, because each of those has a page of its
  own.

frontmatter:
  type:
    const: FrontPage
  id:
    const: welcome
  title:
    example: smllm
  description:
    guidance: The description says what the documentation covers, in one sentence.
    example: The documentation of smllm, which guides an LLM agent through a workflow with a state machine.

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says what smllm is and what the documentation helps the reader do.
        example: smllm guides an LLM agent through a workflow with a state machine. This documentation teaches you to write a machine, connect it to your agent, and look up each rule and finding.
  - heading: Where to start
    level: 2
    blocks:
      - kind: bullet-list
        guidance: Each item names one kind of reader in bold, then links the page where that reader starts.
        each:
          template: '**${}**: ${}[${}][sokf:${:ref}]${}'
          example:
            - '**You write state machines**: start with [Your first state machine][sokf:tutorial-first-machine].'
            - '**You want to see where an agent is**: read [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line].'
  - heading: The sections
    level: 2
    blocks:
      - kind: bullet-list
        guidance: Each item names one section of the documentation in bold and says what a reader finds there.
        each:
          template: '**${}**: ${}'
          example:
            - '**Tutorials**: lessons that build one working thing, step by step.'
            - '**How-to guides**: one task each, from start to finish.'
            - '**Reference**: the facts to look up, such as each rule of `smllm validate` and the findings under it.'
            - '**Explanations**: why smllm works the way it does.'
````
