//! smllm-core: the engine that puts declarative state machines in charge of
//! an LLM agent's turn loop.
//!
//! `no_std` + `alloc`, with no IO, time or randomness: a host supplies them
//! through the traits in [`host`] (ENG-5). The model in [`model`] is built by
//! `smllm-format` from YAML, or deserialised from `smllm compile` JSON (the
//! `serde` feature).
//!
//! ```
//! use smllm_core::{Bind, Engine, host::MemoryStore, model::Config};
//! # use smllm_core::host::*;
//! # struct H; impl Guard for H { fn supports(&self,_:&str)->bool{false} fn check(&mut self,_:&Call<'_>)->Outcome{Outcome{ok:false,detail:String::new()}} }
//! # impl Action for H { fn supports(&self,_:&str)->bool{false} fn run(&mut self,_:&Call<'_>)->Outcome{Outcome{ok:false,detail:String::new()}} }
//! # impl InstructionSource for H { fn read(&self,_:&str)->Result<Option<String>,String>{Ok(None)} }
//! # impl Matcher for H { fn is_match(&self,_:&str,_:&str)->Result<bool,String>{Ok(true)} }
//! # impl Clock for H { fn now_ms(&self)->u64{0} }
//! # impl Ids for H { fn random(&mut self)->u64{42} }
//! # let (mut g, mut a, mut i, h) = (H, H, H, H);
//! let mut store = MemoryStore::default();
//! let mut host = Host { store: &mut store, guards: &mut g, actions: &mut a, source: &h, matcher: &h, clock: &h, ids: &mut i };
//! let engine = Engine::new(Config::default());
//! let reply = engine.bind(&mut host, &Bind { harness: "none", ..Bind::default() })?;
//! assert!(reply.text.starts_with("<smllm>\nsession sm-"));
//! # Ok::<(), smllm_core::Error>(())
//! ```
//!
//! # A host that drives a model without tools
//!
//! A host can drive a machine for a model that only answers in text. The
//! host starts an instance with a keyless `enter` and `harness: "none"`,
//! which creates the session and starts the machine in one call. Then it
//! loops: [`Engine::host_view`] gives the state's events in two lists, the
//! model's and the host's own. While the model's list isn't empty, the host
//! shows the model [`Engine::state_text`], or passes [`Engine::answer_schema`]
//! to a model that can constrain its output, and checks the answer with
//! [`Engine::check_answer`]. It fires an accepted answer with
//! [`Engine::fire`], because the model chose the event, and sends a refusal
//! back to the model as it stands. It fires its own events, such as the
//! result of its checks, a timeout, or a person's decision, with
//! [`Engine::fire_as_host`]. The loop ends when the instance reaches a final
//! state and the session is idle again.
//!
//! The model is never offered a built-in event, such as `yield` or `pause`,
//! or an event that the machine marks with `source: host`, because those
//! belong to whoever drives the session, here the host. An outcome that the
//! model can choose, such as "I can't do this" or "ask the user", is an event
//! that the machine declares. A state whose events are all the host's is a
//! step of the host, and a state that offers both lists is a step of the
//! model that the host can interrupt, so one machine can mix the model's
//! steps, the host's deterministic steps, and a person's steps, for whom the
//! host fires.
//!
//! ```
//! use smllm_core::{Bind, Engine, ParamValue, host::{Host, MemoryStore}};
//! # use smllm_core::host::*;
//! # use smllm_core::model::*;
//! # use smllm_core::SmallMap;
//! # struct H; impl Guard for H { fn supports(&self,_:&str)->bool{false} fn check(&mut self,_:&Call<'_>)->Outcome{Outcome{ok:false,detail:String::new()}} }
//! # impl Action for H { fn supports(&self,_:&str)->bool{false} fn run(&mut self,_:&Call<'_>)->Outcome{Outcome{ok:false,detail:String::new()}} }
//! # impl InstructionSource for H { fn read(&self,_:&str)->Result<Option<String>,String>{Ok(None)} }
//! # impl Matcher for H { fn is_match(&self,_:&str,_:&str)->Result<bool,String>{Ok(true)} }
//! # impl Clock for H { fn now_ms(&self)->u64{0} }
//! # impl Ids for H { fn random(&mut self)->u64{42} }
//! # let (mut g, mut a, mut i, h) = (H, H, H, H);
//! # // `build`: in WRITE the model answers with `triage`, a JSON param;
//! # // in CHECKING only the host fires `checks_passed` or `checks_failed`.
//! # let mut events = SmallMap::new();
//! # events.insert("triage", EventDef { description: Some("Answer with the triage.".into()), params: vec![ParamSpec { name: "json".into(), required: true, kind: ParamKind::Json, ..Default::default() }], host_only: false });
//! # events.insert("checks_passed", EventDef { host_only: true, ..Default::default() });
//! # events.insert("checks_failed", EventDef { host_only: true, ..Default::default() });
//! # let to = |t: &str| vec![Transition { target: Some(t.into()), ..Default::default() }];
//! # let on = |e: &str, t: &str| On { event: e.into(), transitions: to(t) };
//! # let write = State { name: "WRITE".into(), entry: vec![ActionDef::Prompt(Prompt::Text("Write the triage.".into()))], on: vec![on("triage", "CHECKING")], ..Default::default() };
//! # let checking = State { name: "CHECKING".into(), on: vec![on("checks_passed", "DONE"), on("checks_failed", "WRITE")], ..Default::default() };
//! # let done = State { name: "DONE".into(), is_final: true, ..Default::default() };
//! # let machine = Machine { id: "build".into(), description: None, initial: "WRITE".into(), instance: InstanceSpec::default(), events, shared: vec![], states: vec![write, checking, done] };
//! # let config = Config { machines: vec![machine], idle: vec![] };
//! # let mut answers = ["Here it is: {}", r#"{"event": "triage", "params": {"json": {"priority": "low", "labels": []}}}"#].into_iter();
//! # let mut ask_model = |_prompt: &str| answers.next().unwrap().to_string();
//! # let run_checks = |_json: &str| true;
//! let mut store = MemoryStore::default();
//! let mut host = Host { store: &mut store, guards: &mut g, actions: &mut a, source: &h, matcher: &h, clock: &h, ids: &mut i };
//! let engine = Engine::new(config);
//! let none = Bind { harness: "none", ..Bind::default() };
//! let start = [("stateMachine".to_string(), ParamValue::from("build"))];
//! let key = engine.fire(&mut host, None, "enter", &start, &none)?.session;
//! let mut triage = String::new();
//! loop {
//!     let view = engine.host_view(&mut host, &key)?;
//!     if view.location.machine.is_none() {
//!         break; // a final state: the session is idle again
//!     }
//!     if !view.model.is_empty() {
//!         let mut prompt = engine.state_text(&mut host, &key, true)?;
//!         loop {
//!             match engine.check_answer(&mut host, &key, &ask_model(&prompt))? {
//!                 Ok(answer) => {
//!                     if let Some((_, ParamValue::Json(json))) = answer.params.first() {
//!                         triage = json.clone();
//!                     }
//!                     engine.fire(&mut host, Some(&key), &answer.event, &answer.params, &none)?;
//!                     break;
//!                 }
//!                 // The refusal is fit to send back to the model as it stands.
//!                 Err(refusal) => prompt = refusal,
//!             }
//!         }
//!     } else {
//!         let event = if run_checks(&triage) { "checks_passed" } else { "checks_failed" };
//!         engine.fire_as_host(&mut host, Some(&key), event, &[], &none)?;
//!     }
//! }
//! assert_eq!(triage, r#"{"priority":"low","labels":[]}"#);
//! # Ok::<(), smllm_core::Error>(())
//! ```

#![no_std]
#![warn(missing_docs)]
// The tests run on the host, outside the wasm size rules of clippy.toml.
#![cfg_attr(test, allow(clippy::disallowed_methods, clippy::disallowed_types))]

extern crate alloc;

mod engine;
pub mod host;
#[cfg(feature = "json")]
mod json;
pub mod model;
mod prelude;
pub mod record;
mod render;
mod tool;
mod utils;

pub use engine::{
    Answer, BUILTINS, Bind, BuiltinsAt, Engine, Error, HostView, InstanceStatus, Location, Offer,
    ParamValue, ParamView, Reply, SessionStatus, Stop, builtin_offers,
};
pub use render::{AGENT_RULES, format_utc};
pub use tool::{TOOL_INPUT_SCHEMA, TOOL_NAME, tool_description};
pub use utils::SmallMap;
