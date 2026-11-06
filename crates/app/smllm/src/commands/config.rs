//! `init`, `new`, `validate`, `compile`, `info schema` (CLI-1..3, 11, 13).

use std::path::{Path, PathBuf};

use agent_harness_adapter_core::fs::write_atomic;
use serde_json::json;
use smllm_format::{
    ConfigFile, Finding, Findings, Mode, Origin, Severity, compile, config_template, json_schema,
    load_machine, machine_template,
};

use crate::cli::{CompileArgs, InitArgs, NewArgs, ValidateArgs};
use crate::error::{Error, Result};
use crate::output::{self, EXIT_ERRORS, EXIT_OK};
use crate::paths::{self, CONFIG_FILE, PROJECT_DIR};
use crate::runtime::Runtime;

/// `smllm init [--user]`.
// sokf-implements: CLI-1_AC-1
pub fn init(args: &InitArgs) -> Result<u8> {
    let path = if args.user {
        paths::user_config_dir()?.join(CONFIG_FILE)
    } else {
        paths::cwd()?.join(PROJECT_DIR).join(CONFIG_FILE)
    };
    // A file holding only declined harness parts gets the template too,
    // its own text kept after it.
    let created = !path.exists() || !paths::configures(&path);
    if created {
        let kept = std::fs::read_to_string(&path).unwrap_or_default();
        let text = if kept.trim().is_empty() {
            config_template().to_string()
        } else {
            format!("{}\n{kept}", config_template())
        };
        write_atomic(&path, &text)?;
    }
    if args.json {
        output::json(&json!({ "path": smllm_format::path_text(&path), "created": created }))?;
    } else if created {
        output::text(&format!("created {}\n", output::shown(&path)))?;
    } else {
        output::text(&format!(
            "{} already exists, so nothing changed\n",
            output::shown(&path)
        ))?;
    }
    Ok(EXIT_OK)
}

/// `smllm new <ID> [--dir DIR] [--write]`.
// sokf-implements: CLI-2_AC-1
pub fn new(args: &NewArgs, explicit: Option<&Path>) -> Result<u8> {
    if !args
        .id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || args.id.is_empty()
    {
        return Err(Error::msg(format!(
            "id {:?} must contain only letters, digits, _, and -",
            args.id
        )));
    }
    let text = machine_template(&args.id);
    if !args.write {
        if args.json {
            output::json(&json!({ "id": args.id, "path": null, "text": text }))?;
        } else {
            output::text(&text)?;
        }
        return Ok(EXIT_OK);
    }
    let here = paths::cwd()?;
    // `--config` / SMLLM_CONFIG, else the nearest project config.
    let config = match paths::explicit(explicit) {
        Some(p) => here.join(p),
        None => paths::project_config(&here).ok_or_else(|| {
            Error::msg("no .smllm/config.toml in this directory or above it; run smllm init first")
        })?,
    };
    let config_dir = config.parent().unwrap_or(Path::new(".")).to_path_buf();
    let dir = args
        .dir
        .clone()
        .map_or(config_dir.clone(), |d| here.join(d));
    let file = dir.join(format!("{}.smllm.yaml", args.id));
    // A dangling symlink counts too: the write would follow it.
    if std::fs::symlink_metadata(&file).is_ok() {
        return Err(Error::msg(format!(
            "{} already exists",
            output::shown(&file)
        )));
    }
    // Prepare the config edit first: a failure leaves every file as found (CLI-12).
    let registered = register(&config, &config_dir, &file)?;
    // The folders that the write creates, deepest first.
    let created: Vec<&Path> = dir.ancestors().take_while(|d| !d.exists()).collect();
    let unmake = || {
        for d in &created {
            let _ = std::fs::remove_dir(d);
        }
    };
    if let Err(e) = write_atomic(&file, &text) {
        unmake();
        return Err(e.into());
    }
    // A config that can't be written takes the new machine file, and the
    // folders made for it, with it.
    // sokf-implements: CLI-12_AC-1
    if let Err(e) = write_atomic(&config, &registered) {
        let e = Error::from(e);
        return Err(match std::fs::remove_file(&file) {
            Ok(()) => {
                unmake();
                e
            }
            Err(why) => Error::msg(format!(
                "{e}; {} stays, because smllm couldn't remove it: {}",
                output::shown(&file),
                smllm_format::io_reason(&why)
            )),
        });
    }
    if args.json {
        output::json(
            &json!({ "id": args.id, "path": smllm_format::path_text(&file), "config": smllm_format::path_text(&config) }),
        )?;
    } else {
        output::text(&format!(
            "created {} and listed it in {}\n",
            output::shown(&file),
            output::shown(&config)
        ))?;
    }
    Ok(EXIT_OK)
}

/// `config` with `file` added to `[machines] files`, formatting kept.
fn register(config: &Path, config_dir: &Path, file: &Path) -> Result<String> {
    let text = std::fs::read_to_string(config).map_err(|e| Error::io(config, e))?;
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e: toml_edit::TomlError| {
        agent_harness_adapter_core::Error::file(smllm_format::path_text(config), e.message())
    })?;
    let rel = smllm_format::path_text(&relative(config_dir, file));
    let machines = doc.entry("machines").or_insert(toml_edit::table());
    let files = machines
        .as_table_like_mut()
        .ok_or_else(|| Error::msg("[machines] is not a table"))?
        .entry("files")
        .or_insert(toml_edit::value(toml_edit::Array::new()));
    let arr = files
        .as_array_mut()
        .ok_or_else(|| Error::msg("machines.files is not an array"))?;
    if !arr.iter().any(|v| v.as_str() == Some(&rel)) {
        arr.push(rel);
    }
    Ok(doc.to_string())
}

/// `path` relative to `base` (both absolute), with `..` where needed.
fn relative(base: &Path, path: &Path) -> PathBuf {
    let (b, p): (Vec<_>, Vec<_>) = (base.components().collect(), path.components().collect());
    let common = b.iter().zip(&p).take_while(|(x, y)| x == y).count();
    let mut out = PathBuf::new();
    for _ in common..b.len() {
        out.push("..");
    }
    for c in &p[common..] {
        out.push(c);
    }
    out
}

// sokf-implements: FMT-10_AC-10
/// The schemas of `json` params that smllm can't use to check a value
/// (FMT-10), such as one with a `pattern` that isn't a regular expression,
/// which would refuse every value at run time.
fn unusable_schemas(rt: &Runtime) -> Findings {
    let mut out = Findings::default();
    for src in &rt.sources {
        let Some(m) = rt.engine.config().machine(&src.id) else {
            continue;
        };
        for (event, e) in m.events.iter() {
            for p in &e.params {
                let Some(schema) = &p.schema else { continue };
                if let Err(why) = crate::host::json_validator(schema) {
                    out.push(Finding {
                        level: Severity::Error,
                        file: src.file.clone(),
                        line: None,
                        path: Some(format!(
                            "meta.events.{event}.params.properties.{}.schema",
                            p.name
                        )),
                        message: why,
                        hint: Some(
                            "correct the schema, so that smllm can check the param's values"
                                .to_string(),
                        ),
                        rule: "FMT-10",
                    });
                }
            }
        }
    }
    out
}

/// Saved instances whose state no longer exists (CLI-3), and instance
/// files that cannot be read (skipped everywhere else, STORE-1).
// sokf-implements: CLI-3_AC-2
fn saved_states(rt: &mut Runtime) -> Findings {
    let mut out = Findings::default();
    for src in rt.sources.clone() {
        for (file, why) in rt.store.unreadable_markers(&src.id) {
            out.push(Finding {
                level: Severity::Warning,
                file,
                line: None,
                path: None,
                message: format!("can't read the ref marker of {}: {why}", src.id),
                hint: Some(
                    "smllm finds these refs more slowly without the marker, and rewrites the marker when it next sets one of them. Don't delete the marker, because smllm reads a missing marker as no such ref".to_string(),
                ),
                rule: "STORE-1",
            });
        }
        let (instances, unreadable) = rt.store.scan(&src.id);
        for (file, why) in unreadable {
            out.push(Finding {
                level: Severity::Warning,
                file,
                line: None,
                path: None,
                message: format!("can't read the instance file of {}: {why}", src.id),
                hint: Some("smllm skips it, so fix or delete the file".to_string()),
                rule: "STORE-1",
            });
        }
        let Some(m) = rt.engine.config().machine(&src.id) else {
            continue;
        };
        let states: Vec<String> = m.states.iter().map(|s| s.name.clone()).collect();
        for i in instances {
            if !states.contains(&i.state) {
                out.push(Finding {
                    level: Severity::Warning,
                    file: src.file.clone(),
                    line: None,
                    path: None,
                    message: format!(
                        "saved instance {} is in state {}, which no longer exists",
                        i.label(),
                        i.state
                    ),
                    hint: Some("to repair it, fire enter with its ref and a state".to_string()),
                    rule: "CLI-3",
                });
            }
        }
    }
    out
}

/// `smllm validate [PATHS]`.
// sokf-implements: CLI-3_AC-1
pub fn validate(args: &ValidateArgs, explicit: Option<&Path>) -> Result<u8> {
    let here = paths::cwd()?;
    let mut findings = Findings::default();
    if args.paths.is_empty() {
        let files = paths::lookup(explicit, &here)?;
        if files.is_empty() {
            return Err(Error::msg(
                "no config found: run smllm init, or pass --config",
            ));
        }
        let mut rt = Runtime::checked(&files)?;
        findings.extend(rt.findings.clone());
        findings.extend(saved_states(&mut rt));
        findings.extend(unusable_schemas(&rt));
    } else {
        for p in &args.paths {
            let p = here.join(p);
            if p.extension().is_some_and(|e| e == "toml") {
                let mut rt = Runtime::checked(&[ConfigFile {
                    path: p,
                    origin: Origin::Explicit,
                }])?;
                findings.extend(rt.findings.clone());
                findings.extend(saved_states(&mut rt));
                findings.extend(unusable_schemas(&rt));
            } else {
                findings.extend(load_machine(&p, Mode::Check).1);
            }
        }
    }
    let report = output::report(&findings);
    if args.json {
        output::json(&report)?;
    } else {
        let mut show = Vec::new();
        if args.warnings {
            show.push(Severity::Warning);
        }
        if args.info {
            show.push(Severity::Info);
        }
        output::text(&report.to_text(&show))?;
    }
    Ok(if report.has_errors() {
        EXIT_ERRORS
    } else {
        EXIT_OK
    })
}

/// `smllm compile <FILE> [-o OUT]`.
pub fn compile_cmd(args: &CompileArgs) -> Result<u8> {
    let file = paths::cwd()?.join(&args.file);
    let (json, findings) = compile(&file);
    let Some(json) = json else {
        let report = output::report(&findings);
        std::io::Write::write_all(&mut std::io::stderr(), report.to_text(&[]).as_bytes()).ok();
        return Ok(EXIT_ERRORS);
    };
    match &args.out {
        Some(out) => {
            let out = paths::cwd()?.join(out);
            write_atomic(&out, format!("{json}\n"))?;
        }
        None => output::text(&format!("{json}\n"))?,
    }
    Ok(EXIT_OK)
}

// sokf-implements: CLI-9_AC-1
/// `smllm info schema`.
pub fn schema() -> Result<u8> {
    output::text(&format!("{}\n", json_schema()))?;
    Ok(EXIT_OK)
}

/// `info rules`: the rules that `validate` cites, as Markdown.
// sokf-implements: FMT-6_AC-1
pub fn rules() -> Result<u8> {
    output::text(&smllm_format::rules::markdown())?;
    Ok(EXIT_OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths() {
        assert_eq!(
            relative(Path::new("/a/b"), Path::new("/a/b/c.yaml")),
            PathBuf::from("c.yaml")
        );
        assert_eq!(
            relative(Path::new("/a/b"), Path::new("/a/m/c.yaml")),
            PathBuf::from("../m/c.yaml")
        );
    }
}
