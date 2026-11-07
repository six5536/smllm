//! The CLI host: command guards and actions, prompt files, regex patterns,
//! the clock and randomness (CLI-11..9, CLI-11).

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use smllm_core::host::{Action, Call, Clock, Guard, Ids, InstructionSource, Matcher, Outcome};
use smllm_core::model::Value;
use wait_timeout::ChildExt as _;

/// `timeoutSecs` default (CLI-11).
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;
/// Output tail kept for the trace.
const TAIL_CHARS: usize = 400;
/// Output bytes buffered while a command runs: enough for [`TAIL_CHARS`]
/// of any UTF-8 text.
const KEPT_BYTES: usize = TAIL_CHARS * 4 + 4096;

/// Runs `command` guards and actions.
#[derive(Debug, Default, Clone, Copy)]
pub struct Commands;

/// The last `TAIL_CHARS` of `text`, on one line.
fn tail(text: &str) -> String {
    let t = text.trim();
    let start = t.char_indices().rev().nth(TAIL_CHARS).map_or(0, |(i, _)| i);
    let s = &t[start..];
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if start > 0 { format!("…{s}") } else { s }
}

/// Run a `command` call: string → system shell, list → exec; env only;
/// timeout; exit 0 = ok (CLI-11..6, CLI-11).
// sokf-implements: CLI-11_AC-1
// sokf-implements: CLI-11_AC-3
// sokf-implements: CLI-11_AC-4
pub fn run_command(call: &Call<'_>) -> Outcome {
    let mut cmd = match call.params.get("run") {
        Some(Value::Str(s)) => shell(s),
        // sokf-implements: CLI-11_AC-2
        Some(Value::List(argv)) if !argv.is_empty() => {
            let mut c = Command::new(&argv[0]);
            c.args(&argv[1..]);
            c
        }
        _ => {
            return Outcome {
                ok: false,
                detail: "no run param".into(),
            };
        }
    };
    let cwd = call
        .params
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or(call.cwd);
    if !cwd.is_empty() {
        cmd.current_dir(cwd);
    }
    cmd.envs(call.env.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    // Its own process group, so a timeout kills what it started too.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let secs = call
        .params
        .get("timeoutSecs")
        .and_then(Value::as_int)
        .and_then(|n| u64::try_from(n).ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Outcome {
                ok: false,
                detail: format!("could not start: {e}"),
            };
        }
    };
    // Drain both pipes on threads so a chatty command cannot fill a pipe and
    // stall before the timeout. Output is collected as it arrives: a
    // background process the command left running keeps the pipes open, and
    // must not hold smllm hostage after the command itself exits (CLI-11).
    // Only the tail is kept, and once the command is done the readers stop
    // at their next read, so such a process cannot grow smllm's memory.
    let collected = Arc::new(Mutex::new(Vec::new()));
    let done = Arc::new(AtomicBool::new(false));
    // Each reader holds a sender until it ends: the channel disconnects once
    // both have reached end-of-file.
    let (reading, all_read) = mpsc::channel::<()>();
    let drain = |r: Option<Box<dyn Read + Send>>| {
        let sink = Arc::clone(&collected);
        let done = Arc::clone(&done);
        let reading = reading.clone();
        std::thread::spawn(move || {
            let _reading = reading;
            let Some(mut r) = r else { return };
            let mut buf = [0u8; 4096];
            while let Ok(n) = r.read(&mut buf) {
                if n == 0 || done.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(mut v) = sink.lock() {
                    v.extend_from_slice(&buf[..n]);
                    let over = v.len().saturating_sub(KEPT_BYTES);
                    v.drain(..over);
                }
            }
        })
    };
    drain(
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn Read + Send>),
    );
    drain(
        child
            .stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn Read + Send>),
    );
    drop(reading);
    let waited = child.wait_timeout(Duration::from_secs(secs));
    let status = match waited {
        Ok(Some(s)) => s,
        Ok(None) => {
            done.store(true, Ordering::Relaxed);
            kill_tree(&mut child);
            return Outcome {
                ok: false,
                detail: format!("timed out after {secs}s"),
            };
        }
        Err(e) => {
            done.store(true, Ordering::Relaxed);
            return Outcome {
                ok: false,
                detail: format!("wait failed: {e}"),
            };
        }
    };
    if status.success() {
        // A pass shows no output (CLI-11).
        done.store(true, Ordering::Relaxed);
        return Outcome {
            ok: true,
            detail: String::new(),
        };
    }
    // Give the readers a moment to reach end-of-file, then take what arrived.
    let _ = all_read.recv_timeout(Duration::from_millis(200));
    done.store(true, Ordering::Relaxed);
    let text = collected
        .lock()
        .map(|v| String::from_utf8_lossy(&v).into_owned())
        .unwrap_or_default();
    let t = tail(&text);
    let code = status.code().map_or_else(
        || "killed by a signal".to_string(),
        |c| format!("exited {c}"),
    );
    Outcome {
        ok: false,
        detail: if t.is_empty() {
            code
        } else {
            format!("{code}: {t}")
        },
    }
}

/// The system shell running `script`: `sh -c` (CLI-11).
#[cfg(not(windows))]
fn shell(script: &str) -> Command {
    let mut c = Command::new("sh");
    c.args(["-c", script]);
    c
}

/// The system shell running `script`: `cmd /C`, the script passed as is.
/// Through `args`, std would quote it for the MSVC runtime (`"` → `\"`),
/// which `cmd` never undoes, mangling every quote in it.
#[cfg(windows)]
fn shell(script: &str) -> Command {
    use std::os::windows::process::CommandExt as _;
    let mut c = Command::new("cmd");
    c.arg("/C").raw_arg(script);
    c
}

/// Kill a timed-out command and, on unix, its whole process group.
// sokf-implements: CLI-11_AC-3
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    kill_group(child);
    let _ = child.kill();
    let _ = child.wait();
}

/// Signal the child's process group with `kill -s KILL -- -<pgid>`, only
/// while the group is certainly the child's (D3-1). Unreaped, the child's pid
/// (its group id, `process_group(0)`) cannot have been reused; a pid ≤ 1
/// would make `-<pid>` mean every process. Without `--`, procps `kill` reads
/// `-<pid>` as an option and signals `-<first digit>`: `-1`, every process.
#[cfg(unix)]
fn kill_group(child: &mut std::process::Child) {
    let pid = child.id();
    if pid <= 1 || !matches!(child.try_wait(), Ok(None)) {
        return;
    }
    // A missing `kill` is a spawn error: the caller's `child.kill()` remains.
    let _ = Command::new("kill")
        .args(["-s", "KILL", "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

impl Guard for Commands {
    fn supports(&self, kind: &str) -> bool {
        kind == "command"
    }
    fn check(&mut self, call: &Call<'_>) -> Outcome {
        run_command(call)
    }
}

// sokf-implements: CLI-11_AC-5
impl Action for Commands {
    fn supports(&self, kind: &str) -> bool {
        kind == "command"
    }
    fn run(&mut self, call: &Call<'_>) -> Outcome {
        run_command(call)
    }
}

/// Prompt files from disk, read at request time through the loader's own
/// source (ENG-4, FMT-11).
#[derive(Debug, Default, Clone, Copy)]
pub struct Files;

impl InstructionSource for Files {
    fn read(&self, file: &str) -> Result<Option<String>, String> {
        smllm_format::FileSource.read(file)
    }
}

impl Matcher for Files {
    // sokf-implements: AGT-3_AC-10
    /// The errors of `value` against the bundled `schema`, with
    /// `jsonschema`, which fetches no schema from the network or a file.
    /// A value or a schema that can't be read refuses the value, so that no
    /// value passes unchecked.
    fn check_json(&self, schema: &str, value: &str) -> Option<Vec<String>> {
        let validator = match json_validator(schema) {
            Ok(v) => v,
            Err(e) => return Some(vec![e]),
        };
        let value: serde_json::Value = match serde_json::from_str(value) {
            Ok(v) => v,
            Err(e) => {
                return Some(vec![format!(
                    "smllm can't check the value against the schema: {e}"
                )]);
            }
        };
        Some(
            validator
                .iter_errors(&value)
                .map(|e| match e.instance_path().to_string() {
                    p if p.is_empty() => format!("at the root: {e}"),
                    p => format!("at {p}: {e}"),
                })
                .collect(),
        )
    }

    fn checks_json(&self) -> bool {
        true
    }

    fn is_match(&self, pattern: &str, value: &str) -> Result<bool, String> {
        regex::Regex::new(pattern)
            .map(|r| r.is_match(value))
            .map_err(|e| e.to_string())
    }
}

impl Clock for Files {
    fn now_ms(&self) -> u64 {
        fixed_now().unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as u64)
        })
    }
}

// sokf-implements: DOC-1_AC-7
/// In a debug build, the time that `SMLLM_NOW` fixes, in seconds since the
/// Unix epoch, so that the example test of the documentation sees the same
/// times on every run. A release build ignores the variable.
fn fixed_now() -> Option<u64> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let secs: u64 = std::env::var("SMLLM_NOW").ok()?.parse().ok()?;
    secs.checked_mul(1000)
}

/// The validator of a bundled JSON Schema, with `format` an annotation, as
/// both drafts make it by default, or why smllm can't use the schema. The
/// check of a value and `smllm validate` share it.
pub(crate) fn json_validator(schema: &str) -> Result<jsonschema::Validator, String> {
    let schema: serde_json::Value =
        serde_json::from_str(schema).map_err(|e| format!("smllm can't use the schema: {e}"))?;
    jsonschema::options()
        .should_validate_formats(false)
        .build(&schema)
        .map_err(|e| format!("smllm can't use the schema: {e}"))
}

/// OS randomness, or in a debug build a sequence fixed by `SMLLM_SEED`.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsIds;

impl Ids for OsIds {
    fn random(&mut self) -> u64 {
        seeded().unwrap_or_else(|| {
            getrandom::u64()
                .unwrap_or_else(|_| Files.now_ms() ^ u64::from(std::process::id()) << 32)
        })
    }
}

/// The next draw of the sequence that `SMLLM_SEED` fixes, in a debug build.
/// Each draw depends on the seed, the command's arguments, and the number of
/// draws before it, so that the example test of the documentation sees the
/// same session keys and instance IDs on every run, and two commands don't
/// draw the same ID. A release build ignores the variable.
// sokf-implements: DOC-1_AC-4
fn seeded() -> Option<u64> {
    static DRAWS: AtomicU64 = AtomicU64::new(0);
    if !cfg!(debug_assertions) {
        return None;
    }
    let seed: u64 = std::env::var("SMLLM_SEED").ok()?.parse().ok()?;
    // FNV-1a over the arguments.
    let mut args = 0xCBF2_9CE4_8422_2325_u64;
    for arg in std::env::args_os().skip(1) {
        for b in arg.as_encoded_bytes().iter().chain(&[0]) {
            args = (args ^ u64::from(*b)).wrapping_mul(0x0100_0000_01B3);
        }
    }
    Some(draw(seed ^ args, DRAWS.fetch_add(1, Ordering::Relaxed)))
}

/// Draw `n` of the sequence that `start` begins: SplitMix64, whose steps are
/// distinct for distinct `n` and whose mix is a bijection, so the draws of one
/// sequence never repeat.
fn draw(start: u64, n: u64) -> u64 {
    let mut z = start.wrapping_add(n.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: DOC_P-1
    proptest::proptest! {
        #![proptest_config(crate::test_support::cases(512))]
        #[test]
        fn the_draws_of_one_run_never_repeat(start: u64, a in 0..u64::MAX / 2, b in 0..u64::MAX / 2) {
            proptest::prop_assume!(a != b);
            proptest::prop_assert_ne!(draw(start, a), draw(start, b));
        }
    }
    use smllm_core::SmallMap;

    fn call<'a>(params: &'a SmallMap<Value>, env: &'a [(String, String)]) -> Call<'a> {
        Call {
            machine: "m",
            kind: "command",
            params,
            env,
            cwd: "",
        }
    }

    #[cfg(unix)]
    #[test]
    fn shell_exec_env_timeout_and_failure() {
        let env = vec![("SMLLM_REF".to_string(), "GH-1".to_string())];
        let mut p = SmallMap::new();
        p.insert("run", Value::Str("test \"$SMLLM_REF\" = GH-1".into()));
        assert!(run_command(&call(&p, &env)).ok);
        p.insert("run", Value::Str("echo boom >&2; exit 3".into()));
        let o = run_command(&call(&p, &env));
        assert_eq!(o.detail, "exited 3: boom");
        p.insert("run", Value::List(vec!["true".into()]));
        assert!(run_command(&call(&p, &env)).ok);
        p.insert(
            "run",
            Value::List(vec!["definitely-not-a-command-xyz".into()]),
        );
        assert!(
            run_command(&call(&p, &env))
                .detail
                .starts_with("could not start")
        );
        // A megabyte of output is buffered only as its tail.
        p.insert(
            "run",
            Value::Str("head -c 1000000 /dev/zero | tr '\\0' x; echo END; exit 4".into()),
        );
        let o = run_command(&call(&p, &env));
        assert!(
            o.detail.starts_with("exited 4: …x") && o.detail.ends_with("xEND"),
            "{}",
            &o.detail[..40]
        );
        // A background process holding the pipes does not stall the result;
        // a pass waits for no output at all.
        p.insert("run", Value::Str("sleep 5 & exit 0".into()));
        let t0 = std::time::Instant::now();
        assert!(run_command(&call(&p, &env)).ok);
        assert!(t0.elapsed() < std::time::Duration::from_millis(150));
        p.insert("run", Value::Str("sleep 5 & echo late; exit 2".into()));
        let t0 = std::time::Instant::now();
        assert_eq!(run_command(&call(&p, &env)).detail, "exited 2: late");
        assert!(t0.elapsed() < std::time::Duration::from_secs(3));
        p.insert("run", Value::Str("sleep 5".into()));
        p.insert("timeoutSecs", Value::Int(1));
        assert_eq!(run_command(&call(&p, &env)).detail, "timed out after 1s");
        let empty = SmallMap::new();
        assert!(!run_command(&call(&empty, &env)).ok);
        let mut c = Commands;
        assert!(Guard::supports(&c, "command") && !Action::supports(&c, "x"));
        let mut p = SmallMap::new();
        p.insert("run", Value::Str("pwd".into()));
        p.insert("cwd", Value::Str("/".into()));
        assert!(Action::run(&mut c, &call(&p, &env)).ok);
        assert!(Guard::check(&mut c, &call(&p, &env)).ok);
    }

    // The Windows twin: `cmd /C` gets the script as
    // written, quotes included (F24); env, failure, timeout.
    // sokf-tests: CLI-11_AC-1
    // sokf-tests: CLI-11_AC-3
    #[cfg(windows)]
    #[test]
    fn cmd_exec_env_quotes_timeout_and_failure() {
        let env = vec![("SMLLM_REF".to_string(), "GH-1".to_string())];
        let mut p = SmallMap::new();
        p.insert(
            "run",
            Value::Str("if \"%SMLLM_REF%\"==\"GH-1\" (exit 0) else (exit 1)".into()),
        );
        assert!(run_command(&call(&p, &env)).ok);
        p.insert("run", Value::Str("echo \"two words\" 1>&2 & exit 3".into()));
        assert_eq!(
            run_command(&call(&p, &env)).detail,
            "exited 3: \"two words\""
        );
        p.insert("run", Value::Str("ping -n 6 127.0.0.1 >nul".into()));
        p.insert("timeoutSecs", Value::Int(1));
        assert_eq!(run_command(&call(&p, &env)).detail, "timed out after 1s");
    }

    // A timeout kills the command's whole group, and only it: this test
    // process survives, and so does the sleep started beside it (D3-1).
    // sokf-tests: CLI-11_AC-3
    #[cfg(target_os = "linux")]
    #[test]
    fn a_timeout_kills_the_group_and_nothing_else() {
        let alive = |pid: &str| {
            std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|s| {
                s.rsplit(')')
                    .next()
                    .is_some_and(|r| !r.trim_start().starts_with('Z'))
            })
        };
        let dir = crate::test_support::temp_dir("kill");
        let pidfile = dir.join("pid");
        let mut bystander = Command::new("sleep").arg("30").spawn().unwrap();
        let mut p = SmallMap::new();
        p.insert(
            "run",
            Value::Str(format!(
                "sleep 30 & echo $! > '{}'; wait",
                pidfile.display()
            )),
        );
        p.insert("timeoutSecs", Value::Int(1));
        assert_eq!(run_command(&call(&p, &[])).detail, "timed out after 1s");
        let grandchild = std::fs::read_to_string(&pidfile).unwrap();
        let grandchild = grandchild.trim();
        let t0 = std::time::Instant::now();
        while alive(grandchild) && t0.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!alive(grandchild), "the group's sleep survived");
        assert!(matches!(bystander.try_wait(), Ok(None)), "a bystander died");
        let _ = bystander.kill();
        let _ = bystander.wait();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tails_are_short_single_lines() {
        assert_eq!(tail(" a\nb \n"), "a b");
        let long = "x".repeat(1000);
        let t = tail(&long);
        assert!(t.starts_with('…') && t.chars().count() <= TAIL_CHARS + 2);
    }

    #[test]
    fn files_patterns_clock_ids() {
        assert_eq!(Files.read("/definitely/not/here").unwrap(), None);
        // Unreadable as text on every OS (Windows reads "/" without error).
        let bad = std::env::temp_dir().join(format!("smllm-not-utf8-{}", std::process::id()));
        std::fs::write(&bad, [0xff, 0xfe, 0xfd]).unwrap();
        assert!(Files.read(bad.to_str().unwrap()).is_err());
        let _ = std::fs::remove_file(&bad);
        assert!(Files.is_match("^GH-\\d+$", "GH-12").unwrap());
        assert!(!Files.is_match("^GH-\\d+$", "x").unwrap());
        assert!(Files.is_match("(", "x").is_err());
        assert!(Files.now_ms() > 0);
        let mut ids = OsIds;
        assert_ne!(ids.random(), ids.random());
    }
}
