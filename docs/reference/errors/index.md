# errors

- [AGT-1: Author text][sokf:error-agt-1] - The finding that smllm validate reports when the text of a machine contains one of the closing tags that fence smllm's text for the agent.
- [AGT-2: Events list][sokf:error-agt-2] - The findings that smllm validate reports when a machine changes the params or the source of a built-in event.
- [AGT-5: Transitions][sokf:error-agt-5] - The findings that smllm validate reports when a transition leads to a state that doesn't exist, a state can't be reached, or always transitions can't settle.
- [AGT-6: Guards and actions][sokf:error-agt-6] - The findings that smllm validate reports when an entry of meta.sharedActions names a state that doesn't exist, or a position that smllm doesn't know.
- [CLI-11: Command guards and actions][sokf:error-cli-11] - The findings that smllm validate reports when a command guard or action has an empty or malformed run, uses $SMLLM_* variables without a shell, or has a time limit of 0.
- [CLI-3: Configs and saved instances][sokf:error-cli-3] - The findings that smllm validate reports when a saved instance is in a state that its machine no longer has, or when smllm can't read a config file.
- [FMT-1: The machine file][sokf:error-fmt-1] - The findings that smllm validate reports when a machine file isn't YAML that smllm reads, or has a key or a value that smllm doesn't know.
- [FMT-10: JSON params][sokf:error-fmt-10] - The findings that smllm validate reports when a param that takes a JSON value, or its schema, breaks a rule of JSON params.
- [FMT-2: States and transitions][sokf:error-fmt-2] - The findings that smllm validate reports when a state, a transition, or the ref param breaks a rule of how a machine moves from state to state.
- [FMT-3: Actions and guards][sokf:error-fmt-3] - The findings that smllm validate reports when an action or a guard isn't one that smllm knows, or has params that it can't use.
- [FMT-4: Event and instance declarations][sokf:error-fmt-4] - The findings that smllm validate reports when a declaration of an event, a param, or the instance's ref has a bad name, a bad pattern, or a part that nothing uses.
- [FMT-6: Files that smllm reads][sokf:error-fmt-6] - The findings that smllm validate reports when it can't read a machine file that the config or the command names.
- [FMT-7: Combining configs][sokf:error-fmt-7] - The findings that smllm validate reports when it combines the user config and the project config, and a machine of one replaces a machine of the other.
- [MCP-1: Param types][sokf:error-mcp-1] - The finding that smllm validate reports when an event's param has a type other than string and json.
- [STORE-1: The instance files][sokf:error-store-1] - The findings that smllm validate reports when it can't read an instance file or a ref marker of a machine.

<!-- sokf:links -->

[sokf:error-agt-1]: /docs/reference/errors/agt-1.md
[sokf:error-agt-2]: /docs/reference/errors/agt-2.md
[sokf:error-agt-5]: /docs/reference/errors/agt-5.md
[sokf:error-agt-6]: /docs/reference/errors/agt-6.md
[sokf:error-cli-11]: /docs/reference/errors/cli-11.md
[sokf:error-cli-3]: /docs/reference/errors/cli-3.md
[sokf:error-fmt-1]: /docs/reference/errors/fmt-1.md
[sokf:error-fmt-10]: /docs/reference/errors/fmt-10.md
[sokf:error-fmt-2]: /docs/reference/errors/fmt-2.md
[sokf:error-fmt-3]: /docs/reference/errors/fmt-3.md
[sokf:error-fmt-4]: /docs/reference/errors/fmt-4.md
[sokf:error-fmt-6]: /docs/reference/errors/fmt-6.md
[sokf:error-fmt-7]: /docs/reference/errors/fmt-7.md
[sokf:error-mcp-1]: /docs/reference/errors/mcp-1.md
[sokf:error-store-1]: /docs/reference/errors/store-1.md
