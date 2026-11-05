//! `smllm session show KEY --audit`: a session's audit (CLI-14).

use smllm_core::format_utc;
use smllm_core::host::Store;
use smllm_core::record::AuditEntry;

use crate::error::{Error, Result};
use crate::output::{self, EXIT_OK};
use crate::store::FsStore;

/// Print the audit of session `key`: one line for each entry, or with
/// `json` the file's lines as they are.
// sokf-implements: CLI-14_AC-1
// sokf-implements: CLI-14_AC-2
pub fn show(key: &str, json: bool) -> Result<u8> {
    let mut store = FsStore::user()?;
    let lines = match store.audit(key)? {
        Some(lines) => lines,
        None if store.session(key)?.is_some() => Vec::new(),
        None => return Err(Error::msg(format!("no smllm session {key}"))),
    };
    let mut out = String::new();
    for l in &lines {
        if json {
            out.push_str(l);
        } else {
            let entry: AuditEntry = serde_json::from_str(l)
                .map_err(|e| Error::msg(format!("the audit of session {key}: {e}")))?;
            out.push_str(&line(&entry));
        }
        out.push('\n');
    }
    if lines.is_empty() && !json {
        out.push_str("no audit entries\n");
    }
    output::text(&out)?;
    Ok(EXIT_OK)
}

/// `TIME  SOURCE  CALL  PLACE  OUTCOME` for one entry.
fn line(e: &AuditEntry) -> String {
    let call = match &e.event {
        Some(ev) => format!("{} {ev}", e.call),
        None => e.call.clone(),
    };
    let side = |s: &Option<String>| s.as_deref().unwrap_or("idle").to_string();
    let place = match &e.machine {
        None => "idle".to_string(),
        Some(m) => {
            let states = if e.from == e.to {
                side(&e.from)
            } else {
                format!("{} → {}", side(&e.from), side(&e.to))
            };
            format!("{m} {} {states}", e.instance.as_deref().unwrap_or("-"))
        }
    };
    let outcome = match &e.reason {
        Some(r) => format!("{}: {r}", e.outcome),
        None => e.outcome.clone(),
    };
    format!(
        "{}  {}  {call}  {place}  {outcome}",
        format_utc(e.at),
        e.source
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_shows_where_the_call_left_the_session() {
        let base = AuditEntry {
            at: 0,
            source: "agent".into(),
            call: "fire".into(),
            event: Some("submit".into()),
            outcome: "accepted".into(),
            ..AuditEntry::default()
        };
        assert_eq!(
            line(&base),
            "1970-01-01 00:00 UTC  agent  fire submit  idle  accepted"
        );
        let moved = AuditEntry {
            machine: Some("dev".into()),
            instance: Some("i-1".into()),
            from: Some("WORK".into()),
            to: Some("REVIEW".into()),
            ..base.clone()
        };
        assert!(line(&moved).ends_with("fire submit  dev i-1 WORK → REVIEW  accepted"));
        let paused = AuditEntry {
            to: None,
            ..moved.clone()
        };
        assert!(line(&paused).contains("dev i-1 WORK → idle"));
        let stayed = AuditEntry {
            to: Some("WORK".into()),
            outcome: "refused".into(),
            reason: Some("x is not offered here".into()),
            ..moved
        };
        assert!(line(&stayed).ends_with("dev i-1 WORK  refused: x is not offered here"));
    }
}
