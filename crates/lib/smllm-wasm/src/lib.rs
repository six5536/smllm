//! smllm-wasm: `smllm-core` for JavaScript hosts (browser or Node).
//!
//! The machines come in as `smllm compile` JSON. The JS host supplies guards,
//! actions, prompt files, pattern matching, time and randomness through one
//! object; sessions and instances live in an in-memory store the host can
//! export and import as JSON. Every call returns the core `Reply` as JSON.

use core::sync::atomic::{AtomicU32, Ordering};
use smllm_core::host::{
    Action, Call, Clock, Guard, Host, HostError, Ids, InstructionSource, Matcher, MemoryStore,
    Outcome, Store,
};
use smllm_core::model::Config;
use smllm_core::record::{AuditEntry, HistoryEntry, Instance, Session, Status};
use smllm_core::{Bind, ParamValue, Stop};

use smllm_json::{Fields, FromJson, JsonValue, ToJson};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// The JS host object. Every method must exist but `wantsAudit` and
    /// `audit`. Each is called with `catch`: a method that throws, or is
    /// missing, is a host failure the engine handles (a failed guard or
    /// action, a bad pattern, no audit), never an unwind through it.
    pub type JsHost;

    /// Guard of a host kind: `true` passes.
    #[wasm_bindgen(method, catch)]
    fn check(
        this: &JsHost,
        kind: &str,
        params: &str,
        env: &str,
        cwd: &str,
    ) -> Result<bool, JsValue>;
    /// Action of a host kind: `""` = success, else the failure detail.
    #[wasm_bindgen(method, catch)]
    fn run(
        this: &JsHost,
        kind: &str,
        params: &str,
        env: &str,
        cwd: &str,
    ) -> Result<String, JsValue>;
    /// Host kinds supported (`"command"` …).
    #[wasm_bindgen(method, catch)]
    fn supports(this: &JsHost, kind: &str) -> Result<bool, JsValue>;
    /// A prompt file's text, or `undefined` when absent.
    #[wasm_bindgen(method, catch)]
    fn read(this: &JsHost, file: &str) -> Result<Option<String>, JsValue>;
    /// ECMA-262 `new RegExp(pattern).test(value)`.
    #[wasm_bindgen(method, catch, js_name = isMatch)]
    fn is_match(this: &JsHost, pattern: &str, value: &str) -> Result<bool, JsValue>;
    /// Whether the host checks a JSON value against its schema; missing
    /// means no.
    #[wasm_bindgen(method, catch, js_name = checksJson)]
    fn checks_json(this: &JsHost) -> Result<bool, JsValue>;
    /// The errors of the JSON text `value` against the JSON Schema `schema`,
    /// as a JSON array of strings.
    #[wasm_bindgen(method, catch, js_name = checkJson)]
    fn check_json(this: &JsHost, schema: &str, value: &str) -> Result<String, JsValue>;
    /// `Date.now()`.
    #[wasm_bindgen(method, catch)]
    fn now(this: &JsHost) -> Result<f64, JsValue>;
    /// A random 32-bit unsigned integer.
    #[wasm_bindgen(method, catch)]
    fn random(this: &JsHost) -> Result<u32, JsValue>;
    /// One history entry of instance `id` of `machine`, as JSON: the host
    /// keeps the log, the engine never reads it back.
    #[wasm_bindgen(method, catch)]
    fn history(this: &JsHost, machine: &str, id: &str, entry: &str) -> Result<(), JsValue>;
    /// A record as it is saved (`session`, `binding` or `instance`), as
    /// JSON: the host persists what changed, never a snapshot per call
    /// (WASM-2).
    #[wasm_bindgen(method, catch)]
    fn put(this: &JsHost, kind: &str, key: &str, record: &str) -> Result<(), JsValue>;
    /// Whether the host keeps an audit (ENG-11); missing means no.
    #[wasm_bindgen(method, catch, js_name = wantsAudit)]
    fn wants_audit(this: &JsHost) -> Result<bool, JsValue>;
    /// One audit entry as JSON, and the text of the call's reply.
    #[wasm_bindgen(method, catch)]
    fn audit(this: &JsHost, entry: &str, reply: &str) -> Result<(), JsValue>;

    /// The global `String(value)`: an exception's text (`Error: …`).
    #[wasm_bindgen(js_name = String)]
    fn js_string(value: &JsValue) -> String;
}

/// What a host method threw, as text.
fn thrown(what: &str, e: &JsValue) -> String {
    format!("host {what} threw: {}", js_string(e))
}

/// Ids when the host's `random` throws: distinct per call, so id retries
/// always end.
static FALLBACK_IDS: AtomicU32 = AtomicU32::new(0);

struct Js<'a>(&'a JsHost);

fn json<T: ToJson + ?Sized>(v: &T) -> String {
    smllm_json::to_string(v)
}

fn env_json(env: &[(String, String)]) -> String {
    let mut out = String::new();
    let mut o = smllm_json::object(&mut out);
    for (k, v) in env {
        o.field(k, v);
    }
    o.end();
    out
}

impl Js<'_> {
    /// `supports`, a throw meaning no.
    fn kind_supported(&self, kind: &str) -> bool {
        self.0.supports(kind).unwrap_or(false)
    }

    fn random_u32(&self) -> u32 {
        self.0
            .random()
            .unwrap_or_else(|_| FALLBACK_IDS.fetch_add(1, Ordering::Relaxed))
    }
}

impl Guard for Js<'_> {
    fn supports(&self, kind: &str) -> bool {
        self.kind_supported(kind)
    }
    fn check(&mut self, c: &Call<'_>) -> Outcome {
        match self
            .0
            .check(c.kind, &json(c.params), &env_json(c.env), c.cwd)
        {
            Ok(ok) => Outcome {
                ok,
                detail: String::new(),
            },
            Err(e) => Outcome {
                ok: false,
                detail: thrown("check", &e),
            },
        }
    }
}

impl Action for Js<'_> {
    fn supports(&self, kind: &str) -> bool {
        self.kind_supported(kind)
    }
    fn run(&mut self, c: &Call<'_>) -> Outcome {
        let detail = self
            .0
            .run(c.kind, &json(c.params), &env_json(c.env), c.cwd)
            .unwrap_or_else(|e| thrown("run", &e));
        Outcome {
            ok: detail.is_empty(),
            detail,
        }
    }
}

impl InstructionSource for Js<'_> {
    fn read(&self, file: &str) -> Result<Option<String>, String> {
        self.0.read(file).map_err(|e| thrown("read", &e))
    }
}

impl Matcher for Js<'_> {
    fn is_match(&self, pattern: &str, value: &str) -> Result<bool, String> {
        self.0
            .is_match(pattern, value)
            .map_err(|e| thrown("isMatch", &e))
    }

    // sokf-implements: WASM-1_AC-3
    /// The host's `checkJson`, when its `checksJson` says that it checks: a
    /// throw or an answer that isn't a list of strings is an error of the
    /// value, so a broken check never lets a value through.
    fn check_json(&self, schema: &str, value: &str) -> Option<Vec<String>> {
        if !Matcher::checks_json(self) {
            return None;
        }
        Some(match self.0.check_json(schema, value) {
            Ok(text) => smllm_json::from_str::<Vec<String>>(&text).unwrap_or_else(|e| {
                vec![format!("host checkJson answered no list of strings: {e}")]
            }),
            Err(e) => vec![thrown("checkJson", &e)],
        })
    }

    fn checks_json(&self) -> bool {
        self.0.checks_json().unwrap_or(false)
    }
}

impl Clock for Js<'_> {
    fn now_ms(&self) -> u64 {
        self.0.now().unwrap_or(0.0) as u64
    }
}

impl Ids for Js<'_> {
    fn random(&mut self) -> u64 {
        (u64::from(self.random_u32()) << 32) | u64::from(self.random_u32())
    }
}

/// The in-memory store, with history handed to the host: state and log
/// apart, so a snapshot holds current state only.
struct WasmStore<'a> {
    mem: &'a mut MemoryStore,
    host: &'a JsHost,
}

impl Store for WasmStore<'_> {
    fn session(&mut self, key: &str) -> Result<Option<Session>, HostError> {
        self.mem.session(key)
    }
    // Each write goes to memory, then to the host: a host that throws loses
    // that record, never the transition (as history).
    // sokf-implements: WASM-2_AC-1
    fn put_session(&mut self, session: &Session) -> Result<(), HostError> {
        self.mem.put_session(session)?;
        let _ = self.host.put("session", &session.key, &json(session));
        Ok(())
    }
    fn binding(&mut self, harness: &str, host_session: &str) -> Result<Option<String>, HostError> {
        self.mem.binding(harness, host_session)
    }
    fn put_binding(
        &mut self,
        harness: &str,
        host_session: &str,
        key: &str,
    ) -> Result<(), HostError> {
        self.mem.put_binding(harness, host_session, key)?;
        let binding = format!("{harness}/{host_session}");
        let _ = self.host.put("binding", &binding, &json(key));
        Ok(())
    }
    fn instance(&mut self, machine: &str, id: &str) -> Result<Option<Instance>, HostError> {
        self.mem.instance(machine, id)
    }
    fn instances(&mut self, machine: &str) -> Result<Vec<Instance>, HostError> {
        self.mem.instances(machine)
    }
    // Every method delegates, defaults included: a default would answer by
    // cloning every instance, not from the store's indexes.
    fn instances_with(
        &mut self,
        machine: &str,
        status: Status,
    ) -> Result<Vec<Instance>, HostError> {
        self.mem.instances_with(machine, status)
    }
    fn instance_by_ref(
        &mut self,
        machine: &str,
        r#ref: &str,
    ) -> Result<Option<Instance>, HostError> {
        self.mem.instance_by_ref(machine, r#ref)
    }
    fn count(&mut self, machine: &str, status: Status) -> Result<usize, HostError> {
        self.mem.count(machine, status)
    }
    fn recent(
        &mut self,
        machine: &str,
        status: Status,
        limit: usize,
    ) -> Result<Vec<Instance>, HostError> {
        self.mem.recent(machine, status, limit)
    }
    fn put_instance(&mut self, instance: &Instance) -> Result<(), HostError> {
        // A conflict (version, ref) saves nothing: nothing to hand on.
        self.mem.put_instance(instance)?;
        let key = format!("{}/{}", instance.machine, instance.id);
        let _ = self.host.put("instance", &key, &json(instance));
        Ok(())
    }
    /// A host that throws loses that entry, never the transition, which is
    /// already saved.
    fn append_history(
        &mut self,
        machine: &str,
        id: &str,
        entry: &HistoryEntry,
    ) -> Result<(), HostError> {
        let _ = self.host.history(machine, id, &json(entry));
        Ok(())
    }
    // sokf-implements: WASM-10_AC-1
    fn wants_audit(&self) -> bool {
        self.host.wants_audit().unwrap_or(false)
    }
    /// A host that throws loses that entry, never the call, as history.
    fn append_audit(&mut self, entry: &AuditEntry, reply: &str) -> Result<(), HostError> {
        let _ = self.host.audit(&json(entry), reply);
        Ok(())
    }
}

/// An smllm engine with an in-memory store.
#[wasm_bindgen]
pub struct Engine {
    engine: smllm_core::Engine,
    store: MemoryStore,
    host: JsHost,
}

/// A param's value as the engine takes it: a string as a JSON string, any
/// other JSON value as its JSON text (MCP-1).
fn param_value(v: JsonValue) -> ParamValue {
    match v {
        JsonValue::String(s) => ParamValue::Str(s),
        other => ParamValue::Json(smllm_json::to_string(&other)),
    }
}

/// An object's fields as params. A repeated param's last value wins, as
/// when read into a map.
fn params_of(fields: Vec<(String, JsonValue)>) -> Vec<(String, ParamValue)> {
    let mut params: smllm_core::SmallMap<ParamValue> = smllm_core::SmallMap::new();
    for (k, v) in fields {
        params.insert(k, param_value(v));
    }
    let mut out = Vec::new();
    for (k, v) in params.iter() {
        out.push((k.to_string(), v.clone()));
    }
    out
}

/// `fire`'s params: a JSON object, whose strings are JSON strings and whose
/// other values only a JSON param takes.
fn parse_params(json: &str) -> Result<Vec<(String, ParamValue)>, JsError> {
    let fields = from_json::<Fields>(json, "params must be a JSON object")?;
    Ok(params_of(fields.into_vec()))
}

/// JSON text written as it is.
struct Raw<'a>(&'a str);

impl ToJson for Raw<'_> {
    fn write_json(&self, out: &mut String) {
        out.push_str(self.0);
    }
}

/// An answer's params as one object of JSON values: a string as a JSON
/// string, any other value as its JSON text.
struct Params<'a>(&'a [(String, ParamValue)]);

impl ToJson for Params<'_> {
    fn write_json(&self, out: &mut String) {
        let mut o = smllm_json::object(out);
        for (k, v) in self.0 {
            match v {
                ParamValue::Str(s) | ParamValue::Text(s) => o.field(k, s),
                ParamValue::Json(j) => o.field(k, &Raw(j)),
            };
        }
        o.end();
    }
}

/// Offered events: `name`, `description`, and `params`, each with `name`,
/// `required`, `type`, `enum`, `pattern`, `description`, and `schema`, a
/// JSON param's bundled JSON Schema as an object.
struct Offers<'a>(&'a [smllm_core::Offer]);

impl ToJson for Offers<'_> {
    fn write_json(&self, out: &mut String) {
        out.push('[');
        for (i, offer) in self.0.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let mut o = smllm_json::object(out);
            o.field("name", &offer.name)
                .field("description", &offer.description)
                .field("params", &ParamViews(&offer.params));
            o.end();
        }
        out.push(']');
    }
}

// sokf-implements: WASM-9_AC-2
/// The state's invocations, each `{src, input}`, with `input` as JSON.
struct Invokes<'a>(&'a [smllm_core::model::Invoke]);

impl ToJson for Invokes<'_> {
    fn write_json(&self, out: &mut String) {
        out.push('[');
        for (i, inv) in self.0.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let mut o = smllm_json::object(out);
            o.field("src", &inv.src);
            if let Some(input) = &inv.input {
                o.field("input", &Raw(input));
            }
            o.end();
        }
        out.push(']');
    }
}

struct ParamViews<'a>(&'a [smllm_core::ParamView]);

impl ToJson for ParamViews<'_> {
    fn write_json(&self, out: &mut String) {
        out.push('[');
        for (i, p) in self.0.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let mut o = smllm_json::object(out);
            o.field("name", &p.name)
                .field("required", &p.required)
                .field("type", &p.kind)
                .field("enum", &p.enum_values)
                .field("pattern", &p.pattern)
                .field("description", &p.description)
                .field("schema", &p.schema.as_deref().map(Raw));
            o.end();
        }
        out.push(']');
    }
}

/// A tool call's session, event and params.
type ToolArgs = (Option<String>, Option<String>, Vec<(String, ParamValue)>);

/// A tool call's `session`, `event` and `params`, with the MCP server's
/// rules: an empty or null string is absent; params is an object.
fn tool_args(args: &str) -> Result<ToolArgs, String> {
    let mut f = smllm_json::parse(args)
        .and_then(Fields::from_json)
        .map_err(|_| "the arguments must be a JSON object".to_string())?;
    let mut text = |k: &str| match f.take(k) {
        Ok(None | Some(JsonValue::Null)) => Ok(None),
        Ok(Some(JsonValue::String(s))) if s.is_empty() => Ok(None),
        Ok(Some(JsonValue::String(s))) => Ok(Some(s)),
        _ => Err(format!("{k} must be a string")),
    };
    let (session, event) = (text("session")?, text("event")?);
    let params = match f.take("params") {
        Ok(None | Some(JsonValue::Null)) => Vec::new(),
        Ok(Some(JsonValue::Object(fields))) => params_of(fields),
        _ => return Err("params must be an object".to_string()),
    };
    Ok((session, event, params))
}

/// Parse JSON; `what` says what failed, before the parser's own detail.
fn from_json<T: FromJson>(json: &str, what: &str) -> Result<T, JsError> {
    smllm_json::from_str(json).map_err(|e| JsError::new(&format!("{what}: {e}")))
}

fn err(e: impl core::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen]
impl Engine {
    /// Load compiled machines (`smllm compile` output).
    #[wasm_bindgen(constructor)]
    pub fn new(compiled: &str, host: JsHost) -> Result<Engine, JsError> {
        let config: Config = from_json(
            compiled,
            "invalid compiled machines: expected `smllm compile` JSON",
        )?;
        Ok(Engine {
            engine: smllm_core::Engine::new(config),
            store: MemoryStore::default(),
            host,
        })
    }

    fn with<R>(&mut self, f: impl FnOnce(&smllm_core::Engine, &mut Host<'_>) -> R) -> R {
        let (mut g, mut a, mut i) = (Js(&self.host), Js(&self.host), Js(&self.host));
        let s = Js(&self.host);
        let mut store = WasmStore {
            mem: &mut self.store,
            host: &self.host,
        };
        let mut host = Host {
            store: &mut store,
            guards: &mut g,
            actions: &mut a,
            source: &s,
            matcher: &s,
            clock: &s,
            ids: &mut i,
        };
        f(&self.engine, &mut host)
    }

    /// Guard and action kinds this host cannot run, and the `json` params
    /// whose schema it can't check, as a JSON array.
    pub fn unsupported(&mut self) -> String {
        let js = Js(&self.host);
        let mut out = self.engine.unsupported(&js, &js);
        out.extend(self.engine.unchecked_schemas(&js));
        json(&out)
    }

    /// Bind a session; returns the reply JSON (its `session` is the key).
    pub fn bind(
        &mut self,
        harness: &str,
        host_session: Option<String>,
        cwd: &str,
    ) -> Result<String, JsError> {
        let b = Bind {
            harness,
            host_session: host_session.as_deref(),
            cwd,
            configs: &[],
        };
        self.with(|e, h| e.bind(h, &b))
            .map(|r| json(&r))
            .map_err(err)
    }

    /// Read-only view.
    pub fn view(&mut self, key: &str) -> Result<String, JsError> {
        self.with(|e, h| e.view(h, key))
            .map(|r| json(&r))
            .map_err(err)
    }

    /// Where the session is, as the `smllm statusline --json` object (ENG-3).
    // sokf-implements: ENG-3_AC-1
    pub fn status(&mut self, key: &str) -> Result<String, JsError> {
        self.with(|e, h| e.status(h, key))
            .map(|s| json(&s))
            .map_err(err)
    }

    /// The events list.
    pub fn events(&mut self, key: &str) -> Result<String, JsError> {
        self.with(|e, h| e.menu(h, key))
            .map(|r| json(&r))
            .map_err(err)
    }

    /// Fire an event for the agent; `params` is a JSON object, of strings
    /// except for a JSON param's value.
    pub fn fire(
        &mut self,
        key: Option<String>,
        event: &str,
        params: &str,
    ) -> Result<String, JsError> {
        self.fire_from(key, event, params, false)
    }

    /// Fire an event as the host, host-only events included (ENG-6).
    // sokf-implements: WASM-8_AC-1
    #[wasm_bindgen(js_name = fireAsHost)]
    pub fn fire_as_host(
        &mut self,
        key: Option<String>,
        event: &str,
        params: &str,
    ) -> Result<String, JsError> {
        self.fire_from(key, event, params, true)
    }

    fn fire_from(
        &mut self,
        key: Option<String>,
        event: &str,
        params: &str,
        as_host: bool,
    ) -> Result<String, JsError> {
        let ps: Vec<(String, ParamValue)> = if params.is_empty() {
            Vec::new()
        } else {
            parse_params(params)?
        };
        let b = Bind {
            harness: "wasm",
            ..Bind::default()
        };
        self.with(|e, h| {
            if as_host {
                e.fire_as_host(h, key.as_deref(), event, &ps, &b)
            } else {
                e.fire(h, key.as_deref(), event, &ps, &b)
            }
        })
        .map(|r| json(&r))
        .map_err(err)
    }

    /// The stop decision as JSON (AGT-8..6): `{"decision": "allow"}` = may
    /// stop; `{"decision": "block", "text"}` = hold the agent with the events
    /// list; `{"decision": "runaway", "text"}` = let it stop, showing the
    /// list to the user (the harness was already continuing because of a
    /// block, and no event was fired since).
    pub fn stop(&mut self, key: &str, stop_hook_active: bool) -> Result<String, JsError> {
        let decision = self
            .with(|e, h| e.stop(h, key, stop_hook_active))
            .map_err(err)?;
        let (name, text) = match decision {
            Stop::Allow => ("allow", None),
            Stop::Block(t) => ("block", Some(t)),
            Stop::Runaway(t) => ("runaway", Some(t)),
        };
        let mut out = String::new();
        let mut o = smllm_json::object(&mut out);
        o.field("decision", &name);
        if let Some(text) = &text {
            o.field("text", text);
        }
        o.end();
        Ok(out)
    }

    /// A user prompt arrived.
    #[wasm_bindgen(js_name = promptSubmitted)]
    pub fn prompt_submitted(&mut self, key: &str) -> Result<(), JsError> {
        self.with(|e, h| e.prompt_submitted(h, key)).map_err(err)
    }

    /// The agent-facing tool, as JSON `{name, description, inputSchema}`:
    /// the core's definition, as the MCP server offers it (WASM-1).
    // sokf-implements: WASM-1_AC-1
    pub fn tool(&self) -> String {
        let mut out = String::new();
        let mut o = smllm_json::object(&mut out);
        o.field("name", &smllm_core::TOOL_NAME)
            .field("description", &smllm_core::tool_description())
            .field("inputSchema", &Raw(smllm_core::TOOL_INPUT_SCHEMA));
        o.end();
        out
    }

    /// A tool call: the agent's arguments as JSON, answered as the MCP
    /// server answers them (WASM-1). A malformed call or an engine error is
    /// a reply with `ok: false` and `error: …`, never a throw: the agent
    /// sees its mistake as the tool's answer.
    // sokf-implements: WASM-1_AC-2
    #[wasm_bindgen(js_name = callTool)]
    pub fn call_tool(&mut self, args: &str) -> String {
        let answer = match tool_args(args) {
            Ok((key, event, params)) => {
                let bind = Bind {
                    harness: "wasm",
                    ..Bind::default()
                };
                self.with(|e, h| e.call(h, key.as_deref(), event.as_deref(), &params, &bind))
                    .map_err(|e| (key, e.to_string()))
            }
            Err(msg) => Err((None, msg)),
        };
        let reply = answer.unwrap_or_else(|(key, msg)| smllm_core::Reply {
            ok: false,
            session: key.unwrap_or_default(),
            location: smllm_core::Location::default(),
            text: format!("error: {msg}"),
        });
        json(&reply)
    }

    /// The state's text for a model without tools; `schemas` shows each
    /// JSON param's schema (ENG-7).
    // sokf-implements: WASM-9_AC-1
    #[wasm_bindgen(js_name = stateText)]
    pub fn state_text(&mut self, key: &str, schemas: bool) -> Result<String, JsError> {
        self.with(|e, h| e.state_text(h, key, schemas)).map_err(err)
    }

    /// The JSON Schema of a valid answer, as JSON (ENG-8).
    #[wasm_bindgen(js_name = answerSchema)]
    pub fn answer_schema(&mut self, key: &str) -> Result<String, JsError> {
        self.with(|e, h| e.answer_schema(h, key)).map_err(err)
    }

    /// A model's answer checked (ENG-9), as JSON: `{"ok": true, "event",
    /// "params"}`, whose params are JSON values ready for `fire`, or
    /// `{"ok": false, "error"}`.
    #[wasm_bindgen(js_name = checkAnswer)]
    pub fn check_answer(&mut self, key: &str, text: &str) -> Result<String, JsError> {
        let checked = self
            .with(|e, h| e.check_answer(h, key, text))
            .map_err(err)?;
        let mut out = String::new();
        let mut o = smllm_json::object(&mut out);
        match &checked {
            Ok(a) => {
                o.field("ok", &true)
                    .field("event", &a.event)
                    .field("params", &Params(&a.params));
            }
            Err(e) => {
                o.field("ok", &false).field("error", e);
            }
        }
        o.end();
        Ok(out)
    }

    /// The state's offered events in two lists, the model's and the
    /// host's, with where the session is, as JSON (ENG-10).
    #[wasm_bindgen(js_name = hostView)]
    pub fn host_view(&mut self, key: &str) -> Result<String, JsError> {
        let v = self.with(|e, h| e.host_view(h, key)).map_err(err)?;
        let mut out = String::new();
        let mut o = smllm_json::object(&mut out);
        o.field("location", &v.location)
            .field("model", &Offers(&v.model))
            .field("host", &Offers(&v.host))
            .field("invoke", &Invokes(&v.invoke));
        o.end();
        Ok(out)
    }

    /// Sessions, bindings and instances as JSON (history went to the host).
    #[wasm_bindgen(js_name = exportState)]
    pub fn export_state(&self) -> String {
        json(&self.store)
    }

    /// Replace the store from [`Engine::export_state`] JSON.
    #[wasm_bindgen(js_name = importState)]
    pub fn import_state(&mut self, state: &str) -> Result<(), JsError> {
        let read = |state| -> Result<MemoryStore, smllm_json::Error> {
            let mut f = Fields::from_json(smllm_json::parse(state)?)?;
            Ok(MemoryStore::from_parts(
                f.req("sessions")?,
                f.req("bindings")?,
                f.req("instances")?,
                Default::default(),
            ))
        };
        self.store = read(state).map_err(|e| {
            JsError::new(&format!("invalid state: expected `exportState` JSON: {e}"))
        })?;
        Ok(())
    }
}
