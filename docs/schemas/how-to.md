---
type: Schema
id: schema-how-to
title: How-to guide schema
description: One task with smllm, shown from start to finish, for a reader who knows what they want to do.
---

<!-- prettier-ignore -->
````yaml
line-limit: 400

guidance: >
  A how-to guide shows one task from start to finish, for a reader who already knows what
  they want to do. The guide states only the rules that the task needs, and it links to the
  reference and the explanations for the rest. Each file, each run of
  `smllm`, and each block of agent text is checked, because the example test runs them
  and compares the output with the page. A shell snippet that the reader adapts, such as a
  line of a status line script, isn't run. Each file sits under a caption line
  that holds only its name in bold code. Each run of `smllm` is in a `console` block with
  its output, and a run that exits with a code other than 0 ends with `[exit N]`.

root: /docs/how-to

frontmatter:
  type:
    const: HowTo
  id:
    template: 'how-to-${:slug}'
  title:
    pattern: '^[A-Z][a-z]+ [^.]+$'
    guidance: The title names the task, starting with a verb, in the words that a reader searches for.
    example: Show smllm in your status line
  description:
    guidance: The description says what the reader achieves, in one sentence.
    example: Add a row to Claude Code's status line that shows the state machine, the state, and the instance of the session.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the guide among the how-to guides, smallest first.
    example: '20'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says when the reader needs to do this task, and what they have when they finish.
        example: When you want to see where an agent's work is without asking it, show smllm in Claude Code's status line. The row shows the state machine, the state, and the instance of the session.
  - heading: Before you begin
    level: 2
    optional: true
    blocks:
      - kind: bullet-list
        guidance: Each item names one thing that the reader needs before the first step, with a link to the page that explains it.
        example: A project with a state machine, as in [Your first state machine][sokf:tutorial-first-machine].
  - heading: Steps
    level: 2
    blocks-ordered: false
    blocks:
      - kind: numbered-list
        repeat: true
        guidance: >
          Each item holds one action. A file or a run that belongs to a step follows the item,
          or sits inside the item, indented to the item's text.
        each:
          example: 'If you have a script, add this line at the end of it, after its other output:'
          blocks:
            - kind: group
              optional: true
              repeat: true
              guidance: Each code block of a step is followed by the prose that explains it, where it needs any.
              blocks:
                - kind: code
                  lang: any
                  example: |
                    input=$(cat)
                - kind: prose
                  optional: true
                  repeat: true
                  example: '`$input` is the status JSON that your script read from stdin.'
      - kind: prose
        optional: true
        repeat: true
        guidance: A caption is the name of a file in bold code, or a sentence that says what a run shows.
        example: '**`.smllm/fix.smllm.yaml`**'
      - kind: code
        optional: true
        repeat: true
        lang: [console]
        guidance: This block holds a run of `smllm` and its output, as the example test writes them.
        example: |
          $ smllm validate
          0 errors, 0 warnings, 0 info
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          { "statusLine": { "type": "command", "command": "bash ~/.claude/statusline-command.sh" } }
  - heading: Troubleshooting
    level: 2
    optional: true
    blocks:
      - kind: bullet-list
        guidance: Each item gives a problem that the reader might meet during this task, and its fix.
        example: The status line goes blank when the script exits with a non-zero status. Keep the `if` form of the line, which exits with `0` when smllm prints nothing.
  - heading: See also
    level: 2
    blocks:
      - kind: bullet-list
        guidance: Each item links one reference page or explanation that relates to the task.
        each:
          template: '[${}][sokf:${:ref}]'
          example: '[The status line row][sokf:reference-statusline]'
````
