//! `smllm harness hook <HARNESS> <EVENT>`: one hook call for any harness
//! (HARN-3, HARN-3, HARN-3, HARN-3).

use std::io::Read as _;
use std::path::{Path, PathBuf};

use agent_harness_adapter_core::Harness;
use agent_harness_adapter_core::harness;
use agent_harness_adapter_core::hook::{Answer, Event, HookInput, emit};
use smllm_core::Stop;
use smllm_core::host::Store as _;
use smllm_core::record::Session;

use crate::cli::HookArgs;
use crate::error::Result;
use crate::output::{self, EXIT_ERRORS};
use crate::paths;
use crate::runtime::Runtime;
use crate::store::FsStore;

/// The events smllm hooks into.
pub const EVENTS: [Event; 3] = [Event::SessionStart, Event::PromptSubmit, Event::Stop];

/// The hook command every harness runs.
pub const COMMAND: &str = "smllm harness hook {harness} {event}";

/// An event by name, also under the names smllm 0.2 installed, so hooks
/// installed before keep working until reinstalled.
// sokf-implements: HARN-3_AC-4
pub fn event(name: &str) -> Option<Event> {
    match name {
        "user-prompt-submit" => Some(Event::PromptSubmit),
        other => other.parse().ok(),
    }
}

/// The session key bound to a harness's session id.
pub(crate) fn bound_key(harness: &str, session_id: Option<&str>) -> Result<Option<String>> {
    let Some(sid) = session_id else {
        return Ok(None);
    };
    Ok(FsStore::user()?.binding(harness, sid)?)
}

/// Whether `h` can give `answer` at `event`.
fn can(h: &dyn Harness, event: Event, answer: &Answer) -> bool {
    h.answer(event, answer).is_ok()
}

/// One hook call: `input` from the harness `h` in, an answer out. Bindings
/// are kept per harness, the one that sent the input (Cursor running Claude
/// Code's hooks binds as `cursor`). Only a view loads the session's configs:
/// a prompt touches the session alone, and a stop the session allows needs
/// no more (HARN-4). `explicit` is `--config`, which only the session start
/// of an unbound session reads.
// sokf-implements: HARN-3_AC-5
// sokf-implements: HARN-3_AC-6
// sokf-implements: HARN-3_AC-2
pub fn answer(
    h: &dyn Harness,
    event: Event,
    input: &HookInput,
    explicit: Option<&Path>,
) -> Result<Answer> {
    let allow = Answer::Allow { stderr: None };
    let harness = if input.harness.is_empty() {
        h.id()
    } else {
        input.harness.as_str()
    };
    let key = bound_key(harness, input.session_id.as_deref())?;
    let session = match &key {
        Some(k) => FsStore::user()?.session(k),
        None => Ok(None),
    };
    match event {
        Event::SessionStart => {
            // The entry block cannot reach this agent now: bind nothing yet
            // (HARN-3_AC-3).
            if !can(h, event, &context()) {
                return Ok(allow);
            }
            start(harness, input, session.ok().flatten(), explicit)
        }
        Event::PromptSubmit => {
            if let Some(s) = session? {
                Runtime::new(&[])?.with(|e, h| e.prompt_submitted(h, &s.key))?;
            }
            Ok(allow)
        }
        Event::Stop => {
            let Some(s) = session?.filter(|s| !s.may_stop()) else {
                return Ok(allow);
            };
            let mut rt = Runtime::bound(Some(&s))?;
            Ok(match rt.with(|e, h| e.stop(h, &s.key, input.continuing))? {
                Stop::Allow => allow,
                Stop::Block(reason) => Answer::Continue { reason },
                Stop::Runaway(text) => Answer::Allow { stderr: Some(text) },
            })
        }
        // smllm installs no other hook: nothing to say.
        _ => Ok(allow),
    }
}

/// An empty context answer, to ask a harness whether it takes context.
fn context() -> Answer {
    Answer::Context {
        text: String::new(),
    }
}

/// The entry block for a session start: `view` of the bound `session`, else
/// a new binding from the explicit config, else from the configs found at
/// the input's `cwd`; allow when there are none (HARN-3).
fn start(
    harness: &str,
    input: &HookInput,
    session: Option<Session>,
    explicit: Option<&Path>,
) -> Result<Answer> {
    // A bound session's bind is its view, recorded in its audit (ENG-11).
    if let Some(s) = &session {
        let mut rt = Runtime::bound(Some(s))?;
        let reply = rt.bind(harness, input.session_id.as_deref(), &s.cwd)?;
        return Ok(Answer::Context { text: reply.text });
    }
    let cwd = match &input.cwd {
        Some(c) => PathBuf::from(c),
        None => paths::cwd()?,
    };
    // sokf-implements: CLI-4_AC-1
    let files = paths::lookup(explicit, &cwd)?;
    if files.is_empty() {
        return Ok(Answer::Allow { stderr: None });
    }
    let mut rt = Runtime::new(&files)?;
    let reply = rt.bind(
        harness,
        input.session_id.as_deref(),
        &smllm_format::system_path_text(&cwd),
    )?;
    Ok(Answer::Context { text: reply.text })
}

/// `answer` in a form `h` can give at `event`: a stop it cannot block lets
/// the agent stop with the events list on stderr; context it cannot add is
/// dropped.
// sokf-implements: HARN-3_AC-3
pub fn degrade(h: &dyn Harness, event: Event, answer: Answer) -> Answer {
    if can(h, event, &answer) {
        return answer;
    }
    match answer {
        Answer::Continue { reason } => Answer::Allow {
            stderr: Some(reason),
        },
        Answer::Allow { .. } => answer,
        _ => Answer::Allow { stderr: None },
    }
}

/// `smllm harness hook <HARNESS> <EVENT>`: failures exit 1 with stderr only.
// sokf-implements: HARN-3_AC-1
// sokf-implements: PLUG-1_AC-1
pub fn hook(args: &HookArgs, explicit: Option<&Path>) -> Result<u8> {
    // A failed hook exits 1, stderr only (HARN-3).
    let Some(h) = harness::find(&args.name) else {
        eprintln!("error: no harness named {}", args.name);
        return Ok(EXIT_ERRORS);
    };
    let Some(event) = event(&args.hook) else {
        eprintln!(
            "error: no hook event named {} (expected one of: session-start, prompt-submit, stop)",
            args.hook
        );
        return Ok(EXIT_ERRORS);
    };
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    // Input that does not parse is empty: no field is required (HARN-3).
    let input = HookInput::parse(h.as_ref(), event, &stdin).unwrap_or_default();
    let result = answer(h.as_ref(), event, &input, explicit).map(|a| degrade(h.as_ref(), event, a));
    emit(
        h.as_ref(),
        event,
        result,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    )
    .map_err(output::stdout_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: HARN-3_AC-4
    #[test]
    fn old_event_names_still_parse() {
        assert_eq!(event("user-prompt-submit"), Some(Event::PromptSubmit));
        assert_eq!(event("prompt-submit"), Some(Event::PromptSubmit));
        assert_eq!(event("session-start"), Some(Event::SessionStart));
        assert_eq!(event("stop"), Some(Event::Stop));
        assert_eq!(event("nope"), None);
    }

    // sokf-tests: HARN-3_AC-3
    #[test]
    fn answers_a_harness_cannot_give_degrade_to_allow() {
        let reason = || "<smllm>events</smllm>".to_string();
        for h in harness::builtin().iter().filter(|h| h.id() != "agents") {
            // Every harness with hooks can keep the agent going at stop.
            assert_eq!(
                degrade(
                    h.as_ref(),
                    Event::Stop,
                    Answer::Continue { reason: reason() }
                ),
                Answer::Continue { reason: reason() },
                "{}",
                h.id()
            );
            let ctx = Answer::Context { text: "x".into() };
            let got = degrade(h.as_ref(), Event::SessionStart, ctx.clone());
            assert!(got == ctx || got == Answer::Allow { stderr: None });
            assert!(can(h.as_ref(), Event::SessionStart, &got), "{}", h.id());
        }
        let copilot = harness::find("copilot").unwrap();
        assert_eq!(
            degrade(
                copilot.as_ref(),
                Event::SessionStart,
                Answer::Context { text: "x".into() }
            ),
            Answer::Allow { stderr: None }
        );
        let agents = harness::find("agents").unwrap();
        assert_eq!(
            degrade(
                agents.as_ref(),
                Event::Stop,
                Answer::Continue { reason: reason() }
            ),
            Answer::Allow {
                stderr: Some(reason())
            }
        );
    }
}
