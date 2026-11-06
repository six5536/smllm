---
type: FrontPage
id: welcome
title: smllm
description: The documentation of smllm, which guides an LLM agent through a workflow with a state machine.
---

# smllm

smllm guides an LLM agent through a workflow with a state machine: the agent sees one state at a time, and moves on only by firing one of the state's events. This documentation teaches you to write a machine, connect it to your agent, and look up each rule and finding of `smllm validate`.

## Where to start

- **You are new to smllm**: install it as the [README](https://github.com/six5536/smllm#install) shows, then follow [Your first state machine][sokf:tutorial-first-machine].
- **You want an agent to follow your machine**: follow [Connect an agent][sokf:tutorial-connect-an-agent], then [Check the agent's work][sokf:tutorial-check-the-agents-work].
- **You run the engine in your own program**: follow [Run the engine in Node.js][sokf:tutorial-run-the-engine-in-node].
- **You want to know why smllm uses a state machine**: read [Why a state machine][sokf:explanation-why-a-state-machine].
- **You want to see where an agent is**: follow [Show smllm in your status line][sokf:how-to-show-smllm-in-the-status-line].
- **`smllm validate` reported a finding**: look up its code in [The rules of smllm validate][sokf:reference-validate-rules], and its cause and fix on the page of the code, such as [FMT-1: The machine file][sokf:error-fmt-1].

## The sections

- **Tutorials**: lessons that build one working thing, step by step.
- **How-to guides**: one task each, from start to finish.
- **Reference**: the facts to look up, such as each rule of `smllm validate` and the findings under it.
- **Explanations**: why smllm works the way it does.

<!-- sokf:links -->

[sokf:error-fmt-1]: /docs/reference/errors/fmt-1.md
[sokf:explanation-why-a-state-machine]: /docs/explanation/why-a-state-machine.md
[sokf:how-to-show-smllm-in-the-status-line]: /docs/how-to/show-smllm-in-the-status-line.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
[sokf:tutorial-check-the-agents-work]: /docs/tutorials/check-the-agents-work.md
[sokf:tutorial-connect-an-agent]: /docs/tutorials/connect-an-agent.md
[sokf:tutorial-first-machine]: /docs/tutorials/first-machine.md
[sokf:tutorial-run-the-engine-in-node]: /docs/tutorials/run-the-engine-in-node.md
