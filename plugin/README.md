# smllm Claude Code plugin

This plugin installs smllm into Claude Code with one command. It contains three parts:

- The three hooks `SessionStart`, `UserPromptSubmit`, and `Stop`, which run
  `smllm harness hook claude session-start|prompt-submit|stop`.
- The `smllm` MCP server.
- The `smllm-statusline` skill. To use it, ask Claude to "add smllm to my status line". The
  [status line guide](../docs/how-to/show-smllm-in-the-status-line.md) has the details.

The plugin wires up the same parts that `smllm harness install claude` writes into a project,
except the instructions block in `AGENTS.md` or `CLAUDE.md`. The plugin has no need of the block,
because the description of the MCP tool carries the same rules.

To install the plugin, install the `smllm` binary that it calls, then add the marketplace and the
plugin:

```sh
npm install -g @six5536/smllm      # the binary the plugin calls
claude plugin marketplace add six5536/smllm
claude plugin install smllm@smllm
```

Then add a `.smllm/config.toml` to a project with `smllm init`.
