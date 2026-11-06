// Types of the typed smllm-wasm API, shared by the web and bundler entry
// points. They mirror what
// smllm-core serializes.

/** Where a session is after a call; all `null` in idle. */
export interface Location {
  machine: string | null;
  state: string | null;
  instance: string | null;
  ref: string | null;
}

/** The answer to a call: the `<smllm>` text for the agent, and where the session is. */
export interface Reply {
  /** `false` when the event was rejected or the instance moved. */
  ok: boolean;
  session: string;
  location: Location;
  text: string;
}

/** The stop hook's decision (AGT-8..6). */
export type StopDecision =
  | { decision: "allow" }
  /** `block`: hold the agent and show it `text`; `runaway`: let it stop, show `text` to the user. */
  | { decision: "block" | "runaway"; text: string };

export interface InstanceStatus {
  machine: string;
  kind: string;
  id: string;
  ref: string | null;
  label: string;
  status: "active" | "interrupted" | "paused" | "completed";
}

/** Where a session is, as data (the `smllm statusline --json` object). */
export interface SessionStatus {
  session: string;
  idle: boolean;
  machine: string | null;
  state: string | null;
  visit: number | null;
  yielded: boolean;
  instance: InstanceStatus | null;
  interrupted: InstanceStatus | null;
  paused: number;
}

/** One history entry, handed to `Host.history` as it happens. */
export interface HistoryEntry {
  /** Unix ms. */
  at: number;
  session: string;
  event: string;
  from: string | null;
  to: string | null;
  params: Record<string, string>;
  trace: string[];
}

/**
 * One audit entry: a call that drove a session, accepted or refused, handed
 * to `audit` with the text of its reply when the host or the storage wants
 * an audit.
 */
export interface AuditEntry {
  /** Unix ms. */
  at: number;
  session: string;
  /** `host` for `fireAsHost`, else `agent`. */
  source: "agent" | "host";
  /** `view` is a tool call with no event. */
  call: "bind" | "view" | "menu" | "fire" | "stop";
  event: string | null;
  machine: string | null;
  instance: string | null;
  /** The instance's state before the call. */
  from: string | null;
  /** The state where the session is after the call. */
  to: string | null;
  /** Each param as its text, a JSON value as its compact JSON. */
  params: Record<string, string>;
  /** The names of the events that the reply offered. */
  offered: string[];
  /** `accepted` or `refused`; for `stop`, the decision. */
  outcome: "accepted" | "refused" | "allow" | "block" | "runaway";
  /** Why the engine refused the call. */
  reason: string | null;
  trace: string[];
  /** The binding of the session that the call created. */
  binding: { harness: string; hostSession: string | null; cwd: string; configs: string[] } | null;
}

/**
 * What the engine needs from its host. Every method is optional:
 * - `supports`, `check`, `run`: host guard/action kinds (`command`); none by default.
 *   A command's `params.cwd` is relative to the compiled config file (`nodeHost`'s `configDir`).
 * - `read`: prompt files (`smllm compile` inlines them, so rarely needed).
 * - `isMatch`: `new RegExp(pattern).test(value)` by default.
 * - `checkJson`: the errors of a `json` param's value against its schema, one message each, such as
 *   `at /steps: [] has fewer than 1 item`, and none when the value passes. Without it, the engine takes
 *   the value unchecked, and `unsupported()` names each such param.
 * - `now`, `random`: `Date.now()` and `crypto.getRandomValues` by default.
 * - `history`: each history entry as it happens; dropped by default.
 * - `put`: each session, binding and instance as it is saved (write-through storage); dropped by default.
 * - `wantsAudit`, `audit`: whether to keep an audit, false by default, and each audit entry with the
 *   text of its reply. The host keeps what it likes of the text, such as `sha256:` and the hex of the
 *   SHA-256 of its UTF-8 bytes, the default that `smllm` uses.
 * A method that throws is a host failure the engine reports, never a broken engine.
 */
export interface Host {
  supports?(kind: string): boolean;
  check?(kind: string, params: Record<string, unknown>, env: Record<string, string>, cwd: string): boolean;
  /** Nothing (or `""`) on success, else the failure detail. */
  run?(kind: string, params: Record<string, unknown>, env: Record<string, string>, cwd: string): string | void;
  read?(file: string): string | undefined;
  isMatch?(pattern: string, value: string): boolean;
  checkJson?(schema: Record<string, unknown> | boolean, value: unknown): string[];
  now?(): number;
  random?(): number;
  history?(machine: string, id: string, entry: HistoryEntry): void;
  put?(kind: RecordKind, key: string, record: unknown): void;
  wantsAudit?(): boolean;
  audit?(entry: AuditEntry, reply: string): void;
}

/** What `put` hands on: a session (key: its key), a binding (`harness/hostSession`, record: the session key) or an instance (`machine/id`). */
export type RecordKind = "session" | "binding" | "instance";

/**
 * Where an engine's state lives (WASM-2): `load` once, into `importState`;
 * then each saved record through `put` and each history entry through
 * `history`, as they happen, so persisting costs what changed; and, when
 * `wantsAudit` returns true, each audit entry through `audit`.
 */
export interface Storage {
  load(): State | undefined;
  put(kind: RecordKind, key: string, record: unknown): void;
  history(machine: string, id: string, entry: HistoryEntry): void;
  /** Whether the storage keeps an audit; false when absent. */
  wantsAudit?(): boolean;
  /** Each audit entry, with the text of the call's reply. */
  audit?(entry: AuditEntry, reply: string): void;
}

/** The agent-facing `smllm` tool: the same definition the MCP server offers. */
export interface Tool {
  name: string;
  description: string;
  /** JSON Schema of `ToolArgs`. */
  inputSchema: Record<string, unknown>;
}

/** The agent's tool arguments. */
export interface ToolArgs {
  session?: string;
  event?: string;
  /** A string for each param, or any JSON value for a param that the events list shows as a JSON value. */
  params?: Record<string, unknown>;
}

/** How `stateText` renders the state. */
export interface StateTextOptions {
  /** Show each JSON param's schema; `true` by default. */
  schemas?: boolean;
}

/** A param of an offered event. */
export interface OfferedParam {
  name: string;
  required: boolean;
  /** `json`: the param takes any JSON value. */
  type: "string" | "json";
  /** The allowed values; empty when any string is allowed. */
  enum: string[];
  pattern: string | null;
  description: string | null;
  /** A JSON param's JSON Schema, bundled, or `null`. */
  schema: Record<string, unknown> | boolean | null;
}

/** An event that the state offers. */
export interface Offered {
  name: string;
  description: string | null;
  params: OfferedParam[];
}

/**
 * The state as a host that drives the machine sees it: the events that the
 * model can answer with, and the events that only the host fires (with
 * `fireAsHost`). Neither list holds the built-in events, which the host
 * fires as it needs them.
 */
export interface HostView {
  location: Location;
  model: Offered[];
  host: Offered[];
  /** The host's work that the state invokes: run each, and fire its result with `fireAsHost`. */
  invoke: Invoked[];
}

/** One piece of the host's work that a state names with `invoke`: its name, and its fixed input. */
export interface Invoked {
  src: string;
  input?: Record<string, unknown>;
}

/** A model's answer, checked: the event and its params, ready for `fire`, or the refusal to send back to the model. */
export type CheckedAnswer =
  | { ok: true; event: string; params: Record<string, unknown> }
  | { ok: false; error: string };

/** Sessions, bindings and instances by key, as `exportState` returns them and `importState` takes them. */
export interface State {
  sessions: Record<string, unknown>;
  bindings: Record<string, string>;
  instances: Record<string, unknown>;
}
