//! `fire`, `session list|show`, `instance list|show` (CLI-5..6).

use std::path::Path;

use serde_json::json;
use smllm_core::host::Store;
use smllm_core::{ParamValue, format_utc};

use crate::cli::{FireArgs, JsonArgs, KeyArgs, SessionShowArgs};
use crate::error::{Error, Result};
use crate::output::{self, EXIT_ERRORS, EXIT_OK};
use crate::paths;
use crate::runtime::Runtime;
use crate::store::FsStore;

/// `KEY=VALUE` params, each value as text with no JSON type: the engine
/// reads a JSON param's text as JSON, the one path that does (CLI-5).
// sokf-implements: CLI-5_AC-2
fn params(raw: &[String]) -> Result<Vec<(String, ParamValue)>> {
    raw.iter()
        .map(|p| {
            p.split_once('=')
                .map(|(k, v)| (k.to_string(), ParamValue::Text(v.to_string())))
                .ok_or_else(|| Error::msg(format!("--param {p}: expected KEY=VALUE")))
        })
        .collect()
}

/// `smllm fire` (CLI-5, MCP-1).
// sokf-implements: CLI-5_AC-1
pub fn fire(args: &FireArgs, explicit: Option<&Path>) -> Result<u8> {
    let ps = params(&args.params)?;
    let cwd = paths::cwd()?;
    let key = args.session.as_deref();
    let mut rt = Runtime::for_call(key, explicit, &cwd)?;
    let reply = rt.fire("none", key, &args.event, &ps, &cwd)?;
    output::reply(&reply, args.json)?;
    Ok(if reply.ok { EXIT_OK } else { EXIT_ERRORS })
}

/// `smllm session list`.
// sokf-implements: CLI-6_AC-1
pub fn session_list(args: &JsonArgs) -> Result<u8> {
    let store = FsStore::user()?;
    let sessions = store.sessions()?;
    if args.json {
        return output::json(&json!({ "sessions": sessions })).map(|()| EXIT_OK);
    }
    let mut out = String::new();
    for s in &sessions {
        let at = match &s.holding {
            Some(k) => format!("{} {}", k.machine, k.id),
            None => "idle".to_string(),
        };
        out.push_str(&format!(
            "{}  {}  {}  last active {}  {}\n",
            s.key,
            s.harness,
            at,
            format_utc(s.last_active),
            s.cwd
        ));
    }
    if sessions.is_empty() {
        out.push_str("no sessions\n");
    }
    output::text(&out)?;
    Ok(EXIT_OK)
}

/// `smllm session show KEY`: the tool's no-event view, which records no
/// audit entry; with `--audit`, the session's audit.
pub fn session_show(args: &SessionShowArgs) -> Result<u8> {
    if args.audit {
        return crate::commands::audit::show(&args.key, args.json);
    }
    let mut rt = Runtime::for_session(&args.key)?;
    let reply = rt.with(|e, h| e.view(h, &args.key))?;
    output::reply(&reply, args.json)?;
    Ok(EXIT_OK)
}

/// `smllm instance list` (CLI-7).
// sokf-implements: CLI-7_AC-1
pub fn instance_list(args: &JsonArgs, explicit: Option<&Path>) -> Result<u8> {
    let cwd = paths::cwd()?;
    let all = Runtime::lookup(explicit, &cwd)?.all_instances()?;
    if args.json {
        return output::json(&json!({ "instances": all })).map(|()| EXIT_OK);
    }
    let mut out = String::new();
    for i in &all {
        let held = i
            .holder
            .as_deref()
            .map(|h| format!("  held by {h}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}  {} ({})  {}  {}{held}\n",
            i.machine,
            i.label(),
            i.id,
            i.state,
            i.status.as_str()
        ));
    }
    if all.is_empty() {
        out.push_str("no instances\n");
    }
    output::text(&out)?;
    Ok(EXIT_OK)
}

/// `smllm instance show ID`: by id or ref, with history.
pub fn instance_show(args: &KeyArgs, explicit: Option<&Path>) -> Result<u8> {
    let cwd = paths::cwd()?;
    let mut rt = Runtime::lookup(explicit, &cwd)?;
    // By id, then by ref: one or two files per machine, never a scan.
    let mut found = None;
    for s in &rt.sources {
        found = match rt.store.instance(&s.id, &args.key)? {
            Some(i) => Some(i),
            None => rt.store.instance_by_ref(&s.id, &args.key)?,
        };
        if found.is_some() {
            break;
        }
    }
    let inst = found.ok_or_else(|| {
        Error::msg(format!(
            "no instance {} in the configured state machines",
            args.key
        ))
    })?;
    let history = rt.store.history(&inst.machine, &inst.id)?;
    if args.json {
        return output::json(&json!({ "instance": inst, "history": history })).map(|()| EXIT_OK);
    }
    let mut out = format!(
        "{} {} ({})\nstate: {}  status: {}  holder: {}\nvisits: {}\nhistory:\n",
        inst.machine,
        inst.label(),
        inst.id,
        inst.state,
        inst.status.as_str(),
        inst.holder.as_deref().unwrap_or("-"),
        inst.visits
            .iter()
            .map(|(s, n)| format!("{s}={n}"))
            .collect::<Vec<_>>()
            .join(" "),
    );
    for h in &history {
        let route = match (&h.from, &h.to) {
            (Some(f), Some(t)) => format!("{f} → {t}"),
            (None, Some(t)) => format!("→ {t}"),
            (Some(f), None) => format!("{f} →"),
            (None, None) => String::new(),
        };
        out.push_str(&format!(
            "  {}  {}  {}  {route}\n",
            format_utc(h.at),
            h.session,
            h.event
        ));
        for (k, v) in h.params.iter() {
            out.push_str(&format!("      {k} = {v}\n"));
        }
        for t in &h.trace {
            out.push_str(&format!("      {t}\n"));
        }
    }
    output::text(&out)?;
    Ok(EXIT_OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_need_an_equals_sign() {
        assert_eq!(
            params(&["a=b=c".into()]).unwrap(),
            vec![("a".into(), ParamValue::Text("b=c".into()))]
        );
        assert!(params(&["nope".into()]).is_err());
    }
}
