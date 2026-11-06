//! The engine's public protocol: `bind`, `view`, `menu`, `fire`,
//! `fire_as_host`, `stop`, `prompt_submitted` (ENG-1), each call that drives
//! a session audited (ENG-11).

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::engine::audit::{self, Ask};
use crate::engine::turn::Turn;
use crate::engine::{ParamValue, idle, machine};
use crate::host::{Action, Guard, Host, HostError, Matcher};
use crate::model::{ActionDef, Config, GuardDef, Transition};
use crate::prelude::*;
use crate::record::Session;

/// Crockford base32, lower-case: no i, l, o, u.
const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// An engine failure: not a rejected event (that is a [`Reply`] with
/// `ok: false`) but a call that could not be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// No session has this key (AGT-3).
    UnknownSession(String),
    /// No key given, and the event cannot bind one (AGT-3).
    MissingSession,
    /// The store failed.
    Host(HostError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::UnknownSession(k) => {
                // The key is the caller's input: keep it on one line (AGT-1).
                let mut key = String::new();
                crate::render::push_safe(&mut key, k);
                write!(
                    f,
                    "no smllm session {key}: pass the key shown in the latest <smllm> header, or \
                     fire enter with no session to start a new one"
                )
            }
            Error::MissingSession => f.write_str(
                "no session given: pass the key shown in the latest <smllm> header, or fire \
                 enter with no session to start a new one",
            ),
            Error::Host(e) => write!(f, "store: {e}"),
        }
    }
}

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<HostError> for Error {
    fn from(e: HostError) -> Self {
        Error::Host(e)
    }
}

/// Where a session is after a call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub struct Location {
    /// State machine, or `None` in idle.
    pub machine: Option<String>,
    /// State.
    pub state: Option<String>,
    /// Instance id.
    pub instance: Option<String>,
    /// Instance ref.
    #[cfg_attr(feature = "serde", serde(rename = "ref"))]
    pub r#ref: Option<String>,
}

/// The answer to a call: the agent text, plus where the session ended up.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub struct Reply {
    /// False when the event was rejected or the instance moved (exit 1).
    pub ok: bool,
    /// The session key.
    pub session: String,
    /// Where the session is now.
    pub location: Location,
    /// The `<smllm>` text for the agent.
    pub text: String,
}

/// The stop hook's decision (AGT-8..6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// Let the turn end.
    Allow,
    /// Block with this events list.
    Block(String),
    /// Already continuing because of a stop hook and still no event: let it
    /// end, show the list elsewhere (AGT-8).
    Runaway(String),
}

/// How a session is bound (ENG-1).
#[derive(Debug, Clone, Default)]
pub struct Bind<'s> {
    /// Harness name (`claude`, `none`).
    pub harness: &'s str,
    /// The harness's session id, when it has one.
    pub host_session: Option<&'s str>,
    /// Working directory.
    pub cwd: &'s str,
    /// Config files, recorded on the session (STORE-2).
    pub configs: &'s [String],
}

/// The engine: a config plus the protocol over a [`Host`].
#[derive(Debug, Clone)]
pub struct Engine {
    config: Config,
    /// Made by [`Engine::for_harness`]: no host fires events (ENG-6).
    under_harness: bool,
}

// sokf-implements: ENG-1_AC-1
impl Engine {
    /// An engine for `config`, in a host that can fire the host-only events
    /// of its machines through [`Engine::fire_as_host`].
    pub fn new(config: Config) -> Self {
        Self {
            config,
            under_harness: false,
        }
    }

    /// An engine for `config` under a harness, such as Claude Code, where
    /// only the agent fires events: it refuses to start an instance of a
    /// machine with host-only events, because nothing could fire them
    /// (ENG-6).
    pub fn for_harness(config: Config) -> Self {
        Self {
            config,
            under_harness: true,
        }
    }

    /// The config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Guard and action kinds this host cannot run (ENG-4, ENG-4).
    // sokf-implements: ENG-4_AC-2
    // sokf-implements: ENG-4_AC-1
    pub fn unsupported(&self, guards: &dyn Guard, actions: &dyn Action) -> Vec<String> {
        let mut out = Vec::new();
        let mut note = |m: &str, s: &str, what: &str, kind: &str| {
            let line = format!("{m}.{s}: this host cannot run {what} type {kind}");
            if !out.contains(&line) {
                out.push(line);
            }
        };
        let action_list =
            |list: &[ActionDef], m: &str, s: &str, note: &mut dyn FnMut(&str, &str, &str, &str)| {
                for a in list {
                    if let ActionDef::Host { kind, .. } = a
                        && !actions.supports(kind)
                    {
                        note(m, s, "action", kind);
                    }
                }
            };
        let transition =
            |t: &Transition, m: &str, s: &str, note: &mut dyn FnMut(&str, &str, &str, &str)| {
                if let Some(GuardDef::Host { kind, .. }) = &t.guard
                    && !guards.supports(kind)
                {
                    note(m, s, "guard", kind);
                }
                action_list(&t.actions, m, s, note);
            };
        for m in &self.config.machines {
            for sh in &m.shared {
                action_list(&sh.entry, &m.id, "sharedActions", &mut note);
                action_list(&sh.exit, &m.id, "sharedActions", &mut note);
            }
            for s in &m.states {
                action_list(&s.entry, &m.id, &s.name, &mut note);
                action_list(&s.exit, &m.id, &s.name, &mut note);
                for o in &s.on {
                    for t in &o.transitions {
                        transition(t, &m.id, &s.name, &mut note);
                    }
                }
                for t in &s.always {
                    transition(t, &m.id, &s.name, &mut note);
                }
            }
        }
        out
    }

    // sokf-implements: ENG-4_AC-5
    /// The `json` params with a schema that `matcher` can't check, whose
    /// values the engine then takes unchecked (ENG-4).
    pub fn unchecked_schemas(&self, matcher: &dyn Matcher) -> Vec<String> {
        if matcher.checks_json() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for m in &self.config.machines {
            for (name, e) in m.events.iter() {
                for p in e.params.iter().filter(|p| p.schema.is_some()) {
                    out.push(format!(
                        "{}.{name}.{}: this host cannot check the schema of a json param",
                        m.id, p.name
                    ));
                }
            }
        }
        out
    }

    /// Bind a harness session: reuse its key, or create a session in idle.
    /// Returns the current entry block (session start / resume / compact).
    // sokf-implements: ENG-2_AC-1
    pub fn bind(&self, host: &mut Host<'_>, bind: &Bind<'_>) -> Result<Reply, Error> {
        let mut ask = Ask::agent("bind");
        if let Some(hs) = bind.host_session
            && let Some(key) = host.store.binding(bind.harness, hs)?
            && host.store.session(&key)?.is_some()
        {
            return self.show(host, &key, true, Some(ask));
        }
        let session = self.new_session(host, bind)?;
        if let Some(hs) = bind.host_session {
            host.store.put_binding(bind.harness, hs, &session.key)?;
        }
        let key = session.key.clone();
        host.store.put_session(&session)?;
        ask.created = true;
        self.show(host, &key, true, Some(ask))
    }

    // sokf-implements: ENG-2_AC-2
    fn new_session(&self, host: &mut Host<'_>, bind: &Bind<'_>) -> Result<Session, Error> {
        let key = loop {
            let k = format!("sm-{}", random_id(host, 6));
            if host.store.session(&k)?.is_none() {
                break k;
            }
        };
        let now = host.clock.now_ms();
        Ok(Session {
            key,
            harness: bind.harness.to_string(),
            host_session: bind.host_session.map(ToString::to_string),
            cwd: bind.cwd.to_string(),
            configs: bind.configs.to_vec(),
            created: now,
            last_active: now,
            ..Session::default()
        })
    }

    fn session(&self, host: &mut Host<'_>, key: &str) -> Result<Session, Error> {
        host.store
            .session(key)?
            .ok_or_else(|| Error::UnknownSession(key.to_string()))
    }

    /// Read-only "where am I": entry block + events list; changes nothing
    /// (AGT-3) and records no audit entry, for a host that shows a session
    /// to its user. The agent's own look is [`Engine::call`] with no event.
    // sokf-implements: AGT-3_AC-1
    // sokf-implements: ENG-11_AC-6
    pub fn view(&self, host: &mut Host<'_>, key: &str) -> Result<Reply, Error> {
        self.show(host, key, true, None)
    }

    /// A tool call (WASM-1): no event is a view, which needs a key; an
    /// event is fired. Every host maps the agent's tool arguments here.
    // sokf-implements: WASM-1_AC-2
    pub fn call(
        &self,
        host: &mut Host<'_>,
        key: Option<&str>,
        event: Option<&str>,
        params: &[(String, ParamValue)],
        bind: &Bind<'_>,
    ) -> Result<Reply, Error> {
        match event {
            None => {
                let key = key.ok_or(Error::MissingSession)?;
                self.show(host, key, true, Some(Ask::agent("view")))
            }
            Some(ev) => self.fire(host, key, ev, params, bind),
        }
    }

    /// The events list for the current state (the stop hook's text).
    pub fn menu(&self, host: &mut Host<'_>, key: &str) -> Result<Reply, Error> {
        self.show(host, key, false, Some(Ask::agent("menu")))
    }

    /// Where session `key` is; in a machine state, with its entry prompts
    /// when `entry`. Audited when `ask` says what the call was.
    fn show(
        &self,
        host: &mut Host<'_>,
        key: &str,
        entry: bool,
        ask: Option<Ask<'_>>,
    ) -> Result<Reply, Error> {
        let session = self.session(host, key)?;
        let mut turn = Turn::new(&self.config, host, session, "");
        if ask.is_some() {
            audit::begin(&mut turn);
        }
        let reply = if turn.session.holding.is_some() {
            machine::view(&mut turn, entry)?
        } else {
            idle::reply(&mut turn, true, None)?
        };
        if let Some(ask) = ask {
            let outcome = audit::outcome(reply.ok);
            audit::record(&mut turn, &ask, outcome, &reply.location, &reply.text)?;
        }
        Ok(reply)
    }

    /// Fire `event` with `params` for the agent (AGT-3). With no key, only
    /// `enter` may bind a new session, for harnesses without hooks (AGT-3).
    /// An event that the machine marks with `source: host` is refused: the
    /// host fires it through [`Engine::fire_as_host`]. A host that fires an
    /// event that its agent chose calls this method.
    // sokf-implements: AGT-3_AC-4
    pub fn fire(
        &self,
        host: &mut Host<'_>,
        key: Option<&str>,
        event: &str,
        params: &[(String, ParamValue)],
        bind: &Bind<'_>,
    ) -> Result<Reply, Error> {
        self.fire_from(host, key, event, params, bind, false)
    }

    /// Fire `event` with `params` as the host: [`Engine::fire`], which also
    /// takes the events that the machine marks with `source: host` (ENG-6).
    /// Call it only for the host's own events, such as the result of its
    /// checks, never for an event that the agent chose.
    // sokf-implements: ENG-6_AC-1
    pub fn fire_as_host(
        &self,
        host: &mut Host<'_>,
        key: Option<&str>,
        event: &str,
        params: &[(String, ParamValue)],
        bind: &Bind<'_>,
    ) -> Result<Reply, Error> {
        self.fire_from(host, key, event, params, bind, true)
    }

    fn fire_from(
        &self,
        host: &mut Host<'_>,
        key: Option<&str>,
        event: &str,
        params: &[(String, ParamValue)],
        bind: &Bind<'_>,
        as_host: bool,
    ) -> Result<Reply, Error> {
        let session = match key {
            Some(k) => self.session(host, k)?,
            // Saved only if the enter succeeds: a rejected keyless call
            // leaves nothing behind (AGT-3).
            None if event == "enter" => self.new_session(host, bind)?,
            None => return Err(Error::MissingSession),
        };
        let mut turn = Turn::new(&self.config, host, session, event);
        turn.unsaved = key.is_none();
        turn.as_host = as_host;
        turn.under_harness = self.under_harness;
        turn.session.last_active = turn.now;
        // Saved with the session by every path that records the event.
        turn.session.blocked = false;
        audit::begin(&mut turn);
        let reply = if turn.session.holding.is_some() {
            machine::fire(&mut turn, params)?
        } else {
            idle::fire(&mut turn, params)?
        };
        // A rejected keyless call saved no session: no audit to add to.
        if !(turn.unsaved && !reply.ok) {
            let ask = Ask {
                source: if as_host { "host" } else { "agent" },
                call: "fire",
                event: Some(event),
                params,
                created: turn.unsaved,
            };
            let outcome = audit::outcome(reply.ok);
            audit::record(&mut turn, &ask, outcome, &reply.location, &reply.text)?;
        }
        Ok(reply)
    }

    /// The stop hook (AGT-8..6): in a machine state, block with the events
    /// list unless the agent yielded; idle and unknown sessions may stop.
    // sokf-implements: AGT-8_AC-1
    // sokf-implements: AGT-8_AC-2
    // sokf-implements: AGT-8_AC-3
    pub fn stop(
        &self,
        host: &mut Host<'_>,
        key: &str,
        stop_hook_active: bool,
    ) -> Result<Stop, Error> {
        let Some(session) = host.store.session(key)? else {
            return Ok(Stop::Allow);
        };
        let mut turn = Turn::new(&self.config, host, session, "");
        audit::begin(&mut turn);
        let (stop, at) = Self::decide(&mut turn, stop_hook_active)?;
        let (outcome, text) = match &stop {
            Stop::Allow => ("allow", ""),
            Stop::Block(t) => ("block", t.as_str()),
            Stop::Runaway(t) => ("runaway", t.as_str()),
        };
        audit::record(&mut turn, &Ask::agent("stop"), outcome, &at, text)?;
        Ok(stop)
    }

    /// The stop decision for `turn`'s session, and where the session is.
    fn decide(turn: &mut Turn<'_, '_>, stop_hook_active: bool) -> Result<(Stop, Location), Error> {
        let allow = || Ok((Stop::Allow, Location::default()));
        if turn.session.may_stop() {
            return allow();
        }
        // The runaway release needs both the harness's flag and no event
        // fired since the last block: a fire clears `blocked`.
        let runaway = stop_hook_active && turn.session.blocked;
        // A moved instance is reported once, here, and the session drops to
        // idle (AGT-10); an unconfigured machine lets the agent stop.
        let reply = match machine::held(turn, true)? {
            Ok((m, inst)) => machine::view_held(turn, m, &inst, false),
            Err(machine::Gone::Moved(r)) => *r,
            Err(machine::Gone::Unconfigured(_)) => return allow(),
        };
        if runaway {
            return Ok((Stop::Runaway(reply.text), reply.location));
        }
        if !turn.session.blocked {
            turn.session.blocked = true;
            turn.host.store.put_session(&turn.session)?;
        }
        Ok((Stop::Block(reply.text), reply.location))
    }

    /// A user prompt arrived: clear the yielded flag; inject nothing (AGT-8).
    // sokf-implements: AGT-8_AC-4
    pub fn prompt_submitted(&self, host: &mut Host<'_>, key: &str) -> Result<(), Error> {
        if let Some(mut s) = host.store.session(key)?
            && (s.yielded || s.blocked)
        {
            s.yielded = false;
            s.blocked = false;
            host.store.put_session(&s)?;
        }
        Ok(())
    }
}

/// `n` characters from [`ALPHABET`].
pub(crate) fn random_id(host: &mut Host<'_>, n: usize) -> String {
    let mut bits = host.ids.random();
    let mut out = String::with_capacity(n);
    for _ in 0..n {
        out.push(ALPHABET[(bits & 31) as usize] as char);
        bits >>= 5;
    }
    out
}
