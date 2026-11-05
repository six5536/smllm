//! `smllm graph`: states and transitions, guards shown, as text, JSON,
//! Mermaid, an interactive page, or a PNG picture (CLI-8).

use std::path::{Path, PathBuf};

use agent_harness_adapter_core::fs::write_atomic;
use serde_json::{Value, json};
use smllm_core::model::{ActionDef, Machine};

use crate::cli::GraphArgs;
use crate::error::{Error, Result};
use crate::output::{self, EXIT_OK};
use crate::paths;
use crate::picture::{self, guard_label, state_edges as edges};
use crate::runtime::Runtime;

fn text(m: &Machine) -> String {
    let mut out = format!("{} (initial {})\n", m.id, m.initial);
    for s in &m.states {
        let mut tags = Vec::new();
        if s.entry_point {
            tags.push("entry point");
        }
        if s.fallback {
            tags.push("fallback");
        }
        if s.is_final {
            tags.push("final");
        }
        let tags = if tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", tags.join(", "))
        };
        out.push_str(&format!("  {}{tags}\n", s.name));
        for (event, t) in edges(s) {
            let target = t.target.as_deref().unwrap_or("(stays)");
            let mut line = format!("    {event}");
            if let Some(g) = &t.guard {
                line.push_str(&format!(" [{}]", guard_label(g)));
            }
            line.push_str(&format!(" → {target}"));
            if t.reenter {
                line.push_str(" (reenter)");
            }
            if t.actions.contains(&ActionDef::SetRef) {
                line.push_str(" (setRef)");
            }
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

fn mermaid(m: &Machine) -> String {
    let mut out = format!(
        "---\ntitle: {}\n---\nstateDiagram-v2\n    [*] --> {}\n",
        m.id, m.initial
    );
    for s in &m.states {
        for (event, t) in edges(s) {
            let target = t.target.as_deref().unwrap_or(&s.name);
            let label = match &t.guard {
                Some(g) => format!("{event} [{}]", guard_label(g).replace(['"', '`', ':'], "")),
                None => event.to_string(),
            };
            out.push_str(&format!("    {} --> {target}: {label}\n", s.name));
        }
        if s.is_final {
            out.push_str(&format!("    {} --> [*]\n", s.name));
        }
    }
    out
}

fn to_json(m: &Machine) -> Value {
    let states: Vec<Value> = m
        .states
        .iter()
        .map(|s| {
            let ts: Vec<Value> = edges(s)
                .into_iter()
                .map(|(event, t)| {
                    json!({
                        "event": event,
                        "target": t.target,
                        "guard": t.guard.as_ref().map(guard_label),
                        "reenter": t.reenter,
                    })
                })
                .collect();
            json!({
                "name": s.name,
                "final": s.is_final,
                "entryPoint": s.entry_point,
                "fallback": s.fallback,
                "transitions": ts,
            })
        })
        .collect();
    json!({ "id": m.id, "initial": m.initial, "states": states })
}

/// `smllm graph [ID] [--json|--mermaid|--open|--html FILE|--png FILE]`.
// sokf-implements: CLI-8_AC-1
pub fn graph(args: &GraphArgs, explicit: Option<&Path>) -> Result<u8> {
    let cwd = paths::cwd()?;
    let rt = Runtime::lookup(explicit, &cwd)?;
    let machines: Vec<&Machine> = match &args.id {
        Some(id) => vec![
            rt.engine
                .config()
                .machine(id)
                .ok_or_else(|| Error::msg(format!("no state machine {id}")))?,
        ],
        None => rt.engine.config().machines.iter().collect(),
    };
    if args.open || args.html.is_some() || args.png.is_some() {
        return pictures(args, &machines);
    }
    if args.json {
        output::json(
            &json!({ "machines": machines.iter().map(|m| to_json(m)).collect::<Vec<_>>() }),
        )?;
    } else if args.mermaid {
        output::text(
            &machines
                .iter()
                .map(|m| mermaid(m))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    } else {
        output::text(
            &machines
                .iter()
                .map(|m| text(m))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }
    Ok(EXIT_OK)
}

/// A new folder in the temporary folder, for the page that `--open` opens.
fn temp_folder() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for attempt in 0..100u64 {
        let n = getrandom::u64()
            .unwrap_or_else(|_| u64::from(std::process::id()) << 32)
            .wrapping_add(attempt);
        let dir = base.join(format!("smllm-graph-{n:016x}"));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(Error::io(&dir, e)),
        }
    }
    Err(Error::msg(format!(
        "cannot make a new folder in {}",
        smllm_format::path_text(&base)
    )))
}

/// The file of each machine's PNG: `file` for one machine, and for several,
/// `STEM-ID.EXT` beside `file` for each machine `ID`.
// sokf-implements: CLI-8_AC-9
fn png_paths(file: &Path, ids: &[&str]) -> Vec<PathBuf> {
    if let [_] = ids {
        return vec![file.to_path_buf()];
    }
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let ext = file
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    ids.iter()
        .map(|id| file.with_file_name(format!("{stem}-{id}{ext}")))
        .collect()
}

// sokf-implements: CLI-8_AC-3
// sokf-implements: CLI-8_AC-4
// sokf-implements: CLI-8_AC-7
// sokf-implements: CLI-8_AC-8
/// `--open`, `--html FILE`, and `--png FILE`: build every output first, open
/// the browser on a page of its own, and then write the files.
fn pictures(args: &GraphArgs, machines: &[&Machine]) -> Result<u8> {
    if machines.is_empty() {
        return Err(Error::msg("no state machine to draw"));
    }
    let ids: Vec<&str> = machines.iter().map(|m| m.id.as_str()).collect();
    let title = match ids.as_slice() {
        [one] => format!("State machine {one}"),
        many => format!("State machines {}", many.join(", ")),
    };
    let page = (args.open || args.html.is_some()).then(|| picture::page(machines, &title));
    let mut pngs = Vec::new();
    if let Some(file) = &args.png {
        for (path, m) in png_paths(file, &ids).into_iter().zip(machines) {
            pngs.push((path, picture::png(m)?));
        }
    }
    let mut done = String::new();
    if let Some(page) = page.as_deref().filter(|_| args.open) {
        let dir = temp_folder()?;
        let file = dir.join(format!("{}.html", args.id.as_deref().unwrap_or("machines")));
        let opened = write_atomic(&file, page)
            .map_err(Error::from)
            .and_then(|()| picture::open(&file));
        if let Err(e) = opened {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
        done.push_str(&format!("opened {} in the browser\n", output::shown(&file)));
    }
    if let (Some(file), Some(page)) = (&args.html, &page) {
        write_atomic(file, page)?;
        done.push_str(&format!("wrote {}\n", output::shown(file)));
    }
    for (file, png) in &pngs {
        write_atomic(file, png)?;
        done.push_str(&format!("wrote {}\n", output::shown(file)));
    }
    output::text(&done)?;
    Ok(EXIT_OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_png_for_each_machine() {
        let f = Path::new("out/m.png");
        assert_eq!(png_paths(f, &["dev"]), [PathBuf::from("out/m.png")]);
        assert_eq!(
            png_paths(f, &["dev", "plan"]),
            [
                PathBuf::from("out/m-dev.png"),
                PathBuf::from("out/m-plan.png")
            ]
        );
        assert_eq!(
            png_paths(Path::new("m"), &["a", "b"])[1],
            PathBuf::from("m-b")
        );
    }
}
