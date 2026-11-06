---
type: Schema
id: schema-explanation
title: Explanation schema
description: Why smllm works the way it does, for a reader who wants to understand one part of smllm.
---

<!-- prettier-ignore -->
````yaml
line-limit: 400

guidance: >
  An explanation describes why smllm works the way it does. It covers the idea behind one
  part, the choices in its design, and their consequences for the reader. It links to the
  how-to guides for steps. Where it discusses a rule of `smllm validate`, it links the rule's
  code on the rules page instead of restating the rule, because the rules page is the
  public statement of each rule.

root: /docs/explanation

frontmatter:
  type:
    const: Explanation
  id:
    template: 'explanation-${:slug}'
  title:
    example: Why a state machine
  description:
    guidance: The description says what the explanation clarifies, in one sentence.
    example: Why smllm guides an agent with a state machine rather than with a longer prompt, and what the machine can hold the agent to.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the explanation among the explanations, smallest first.
    example: '10'

sections:
  - heading: { from: title }
    level: 1
    subheadings-open: true
    blocks:
      - kind: prose
        guidance: The opening states the question that the explanation answers, and a short answer.
        example: An agent forgets a step of a long prompt, and nothing tells it that it did. A state machine gives the agent one step at a time, and smllm refuses the agent's stop until the step's event is fired.
      - kind: bullet-list
        optional: true
        guidance: The contents list has one item for each level-2 section, in order, and each item links to its section's heading.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[One step at a time](#one-step-at-a-time)'
            - '[A stop that needs an event](#a-stop-that-needs-an-event)'
  - heading: See also
    level: 2
    blocks:
      - kind: bullet-list
        guidance: Each item links one page that applies or lists what the explanation describes.
        each:
          template: '[${}][sokf:${:ref}]'
          example: '[Your first state machine][sokf:tutorial-first-machine]'
````
