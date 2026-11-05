# smllm-core

smllm-core is the engine of [smllm](https://crates.io/crates/smllm). smllm runs state machines
that drive the turn loop of a large language model (LLM) agent. This README describes what the crate contains and
where to find its API documentation.

The crate is `no_std` and needs `alloc`. It has no IO, time, or randomness of its own. A host
supplies them through the traits in `smllm_core::host`: `Store`, `Guard`, `Action`,
`InstructionSource`, `Matcher`, `Clock`, and `Ids`. The same engine runs in the `smllm` CLI and,
through `smllm-wasm`, in a browser.

The crate has these parts:

- The methods `Engine::bind`, `view`, `menu`, `fire`, `fire_as_host`, `stop`, and
  `prompt_submitted` make up the whole protocol. A host that drives a model without tools adds
  `host_view`, `state_text`, `answer_schema`, and `check_answer`.
- `model` holds the lowered state machine. `smllm-format` builds the model from YAML, and the
  `serde` feature deserialises the JSON output of `smllm compile`.
- `record` holds sessions, instances, history entries, and audit entries.
- A store that returns true from `Store::wants_audit` receives an `AuditEntry` through
  `Store::append_audit` for each call that drives a session, accepted or refused, with the text
  of the reply. The entry holds the time, the source (`agent`, or `host` for `fire_as_host`), the
  call, the states before and after, the params, the offered events, and the outcome. The store
  keeps what it likes of the text. A hash names its algorithm, as `sha256:` and the hex of the
  SHA-256 of the text's UTF-8 bytes, which is the default that the CLI uses. By default, a store
  wants no audit, and the engine builds no entry. `Engine::view` and `Engine::status` inspect a
  session and record nothing.
- Every reply is `<smllm>` text for the agent, together with where the session ended up.

A machine can mark an event with `source: host` in `meta.events`, such as an event that carries
the result of checks that the host runs outside the machine. The agent never sees such an event,
and `Engine::fire` refuses it. The host fires it with `Engine::fire_as_host`, which takes the same
arguments as `fire` and fires any event that the state offers. Fire an event that the agent chose
with `fire`, and keep `fire_as_host` for the host's own events, because `fire_as_host` would
otherwise take a host-only event from the agent. The marker is a security boundary only when the
host drives a model without tools: under a harness, the agent usually has a shell, and the marker
prevents mistakes. An engine from `Engine::for_harness`, which the CLI uses, refuses to start an
instance of a machine with host-only events, because nothing under a harness fires them.

## A host that drives a model without tools

A host can drive a machine for a model that has no tools and only answers in text, such as a
model that the host calls through the Messages API or through `claude -p` with every tool turned
off. Four read-only calls serve it:

- `Engine::host_view` gives where the session is and the state's events in two lists: the
  events that the model can fire, and the events that only the host fires.
- `Engine::state_text` renders the state for the model: its instructions, the model's events with
  their params, the schema of each JSON param unless the host leaves the schemas out, and a
  request to answer with one event as JSON, such as
  `{"event": "pages", "params": {"pages": "3,4"}}`.
- `Engine::answer_schema` gives a JSON Schema of a valid answer, for a model that can constrain
  its output: a `oneOf` with one branch for each of the model's events, and the bundled schema of
  each JSON param.
- `Engine::check_answer` reads the model's answer, a bare JSON object or one fenced block with
  nothing outside it, and returns the event and its params, ready for `fire`, or a refusal that
  the host sends back to the model as it stands. Every malformed answer gets the same refusal.

The host's loop runs from start to end in this way, with a `MemoryStore` when the host keeps
nothing:

1. Start the instance with `fire(None, "enter", [stateMachine], Bind { harness: "none", .. })`,
   which creates the session and enters the machine in one call.
2. Call `host_view`. When the session is idle again, the instance has reached a final state, and
   the loop ends.
3. While the model's list isn't empty, show the model the state's text, check its answer, and
   fire an accepted answer with `fire`, because the model chose the event.
4. Whenever the host has a reason, such as the result of its checks, a timeout, or a person's
   decision, fire its own event with `fire_as_host`.

The crate's API documentation shows the whole loop as a tested example.

The three calls for the model offer no built-in event, such as `yield`, `pause`, or `unmatched`,
and no host-only event. Those events belong to whoever drives the session, which here is the
host, and the host fires them as it needs them. An outcome that the model can choose, such as
"I can't do this" or "ask the user", is an event that the machine declares.

One machine can mix three kinds of step. A state that offers the model events is a step of the
model. A state whose events are all the host's is a deterministic step of the host, such as
checks that the host runs, which can be asynchronous. A state in which the host fires for a
person, such as an approval, is a person's step. A state that offers both lists is a step of the
model that the host can interrupt, such as with a timeout.

If you want the tool rather than the library, install [`smllm`](https://crates.io/crates/smllm).
The API isn't stable yet. The [API documentation](https://docs.rs/smllm-core) describes each
item.

## Licence

smllm-core is licensed under the MIT licence. The
[LICENSE](https://github.com/six5536/smllm/blob/main/LICENSE) file has the full text.
