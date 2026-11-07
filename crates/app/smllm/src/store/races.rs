//! The file store under writes that race: readers that open a file while the
//! store replaces it, and writers that load the same version (STORE_P-1,
//! STORE_P-2).

use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;

use proptest::collection::vec;
use proptest::prelude::*;
use smllm_core::host::{HostError, Store};
#[cfg(unix)]
use smllm_core::record::{Instance, Session};

use super::FsStore;
use super::tests::inst;
use crate::test_support::cases;

/// A store over the folders of `d`, as each process opens it.
fn store(d: &Path) -> FsStore {
    FsStore::new(
        d.join("user"),
        [("dev".to_string(), d.join("proj/state"))].into(),
    )
}

/// Write with `write` in one thread while this one reads `path` as often
/// as it can, and give each content the reader sees to `check`.
#[cfg(unix)]
fn race(path: &Path, write: impl FnOnce() + Send, check: impl Fn(&str)) {
    use std::sync::atomic::{AtomicBool, Ordering};
    let done = AtomicBool::new(false);
    let mut seen = 0;
    std::thread::scope(|s| {
        s.spawn(|| {
            write();
            done.store(true, Ordering::SeqCst);
        });
        // One more look after the writer stops, so a writer that finishes
        // before the first look still leaves something to check.
        loop {
            let finished = done.load(Ordering::SeqCst);
            let paths: Vec<PathBuf> = if path.is_dir() {
                fs::read_dir(path)
                    .unwrap()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|x| x == "json"))
                    .collect()
            } else {
                vec![path.to_path_buf()]
            };
            for p in paths {
                if let Ok(text) = fs::read_to_string(&p) {
                    check(&text);
                    seen += 1;
                }
            }
            if finished {
                break;
            }
        }
    });
    assert!(seen > 0, "{}: the reader saw no write", path.display());
}

/// Text of up to `max` characters, made of one generated character
/// repeated, so that a size near `max` costs no more to generate than a
/// short one.
#[cfg(unix)]
fn sized(max: usize) -> impl Strategy<Value = String> {
    (any::<char>(), 0..max).prop_map(|(c, n)| c.to_string().repeat(n))
}

/// The writes of one kind of file: the contents it alternates between.
#[cfg(unix)]
const WRITES: usize = 20;

#[cfg(unix)]
proptest! {
    #![proptest_config(cases(3))]

    // A reader that opens a file while the store replaces it, again and
    // again, sees the whole of one content, never a part: what a write
    // that stops at any point leaves behind. Each kind of file alternates
    // between two generated contents of generated sizes. Unix only, where
    // a reader does not block the rename.
    // sokf-tests: STORE_P-1
    #[test]
    fn readers_see_whole_files_while_the_store_writes(
        configs in [vec(sized(4096), 0..64), vec(sized(4096), 0..64)],
        keys in ["[a-z0-9-]{1,8}", "[a-z0-9-]{1,8}"],
        lengths in [0..100_000usize, 0..100_000usize],
        states in [sized(100_000), sized(100_000)],
        r#ref in "[a-z]{1,7}",
    ) {
        let d = crate::test_support::temp_dir("whole");
        // Sessions.
        let sessions = configs.map(|configs| Session {
            key: "sm-1".into(),
            configs,
            ..Default::default()
        });
        let texts = sessions.each_ref().map(|s| serde_json::to_string_pretty(s).unwrap());
        let mut st = store(&d);
        st.put_session(&sessions[0]).unwrap();
        race(
            &d.join("user/sessions/sm-1.json"),
            || {
                for n in 0..WRITES {
                    st.put_session(&sessions[n % 2]).unwrap();
                }
            },
            |t| assert!(texts.iter().any(|x| x == t), "part of a session: {}", t.len()),
        );
        // Bindings.
        let keys = [0, 1].map(|n| format!("sm-{}{}", keys[n], "b".repeat(lengths[n])));
        let mut st = store(&d);
        st.put_binding("claude", "cc-1", &keys[0]).unwrap();
        race(
            &d.join("user/bindings/claude/cc-1"),
            || {
                for n in 0..WRITES {
                    st.put_binding("claude", "cc-1", &keys[n % 2]).unwrap();
                }
            },
            |t| assert!(keys.iter().any(|k| k == t), "part of a binding: {}", t.len()),
        );
        // Instances.
        let mut st = store(&d);
        st.put_instance(&inst(1)).unwrap();
        race(
            &d.join("proj/state/dev/active/i-1.json"),
            || {
                for v in 2..2 + WRITES as u64 {
                    let mut i = inst(v);
                    i.state = states[v as usize % 2].clone();
                    st.put_instance(&i).unwrap();
                }
            },
            |t| {
                let i: Instance = serde_json::from_str(t).expect("part of an instance");
                assert!(states.contains(&i.state) || i.state == "A");
            },
        );
        // Ref markers: refs that differ only in case share one marker file,
        // which each new instance rewrites with one more entry.
        let mut st = store(&d);
        let mut first = inst(1);
        first.id = "r-0".into();
        first.r#ref = Some(r#ref.clone());
        st.put_instance(&first).unwrap();
        let variants = (1u32 << r#ref.len().min(5)).min(WRITES as u32);
        race(
            &d.join("proj/state/dev/refs"),
            || {
                for n in 1..variants {
                    let r: String = r#ref
                        .chars()
                        .enumerate()
                        .map(|(b, c)| if n >> b & 1 == 1 { c.to_ascii_uppercase() } else { c })
                        .collect();
                    let mut i = inst(1);
                    i.id = format!("r-{n}");
                    i.r#ref = Some(r);
                    st.put_instance(&i).unwrap();
                }
            },
            |t| {
                serde_json::from_str::<std::collections::BTreeMap<String, String>>(t)
                    .expect("part of a ref marker");
            },
        );
        fs::remove_dir_all(d).ok();
    }
}

/// One step of two writers over one folder: a writer loads the instance, or
/// saves its loaded copy with its version moved by `delta`.
#[derive(Debug, Clone)]
enum Step {
    Load(usize),
    Save(usize, i64),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (0..2usize).prop_map(Step::Load),
        (0..2usize, -2..=3i64).prop_map(|(w, d)| Step::Save(w, d)),
    ]
}

proptest! {
    #![proptest_config(cases(32))]

    // Two writers, two stores over one folder as two processes would be,
    // load and save in a generated order, each with a generated version:
    // the store accepts exactly the saves whose version is the stored
    // version plus one, and keeps the last one accepted.
    // sokf-tests: STORE_P-2
    #[test]
    fn a_save_is_accepted_only_at_the_next_version(steps in vec(step(), 0..40)) {
        let d = crate::test_support::temp_dir("versions");
        store(&d).put_instance(&inst(1)).unwrap();
        let mut writers = [store(&d), store(&d)];
        let mut loaded = [inst(1), inst(1)];
        let mut stored = 1u64;
        for (n, s) in steps.iter().enumerate() {
            match *s {
                Step::Load(w) => {
                    loaded[w] = writers[w].instance("dev", "i-1").unwrap().unwrap();
                    prop_assert_eq!(loaded[w].version, stored);
                }
                Step::Save(w, delta) => {
                    let mut i = loaded[w].clone();
                    i.version = i.version.saturating_add_signed(delta);
                    i.state = format!("S{n}");
                    let r = writers[w].put_instance(&i);
                    if i.version == stored + 1 {
                        prop_assert_eq!(r, Ok(()), "{:?}", steps);
                        stored = i.version;
                    } else {
                        prop_assert_eq!(r, Err(HostError::Conflict), "{:?}", steps);
                    }
                }
            }
            prop_assert_eq!(store(&d).instance("dev", "i-1").unwrap().unwrap().version, stored);
        }
        fs::remove_dir_all(d).ok();
    }
}

// Two writers race in two threads to save the next version of the instance
// that they load: each version is accepted exactly once, and every other
// write is a conflict. Then a version below, at, or past the next one is
// refused.
// sokf-tests: STORE_P-2
#[test]
fn racing_writers_save_each_version_once() {
    let d = crate::test_support::temp_dir("race");
    store(&d).put_instance(&inst(1)).unwrap();
    let mut accepted: Vec<u64> = std::thread::scope(|s| {
        let writers: Vec<_> = (0..2)
            .map(|w| {
                let d = &d;
                s.spawn(move || {
                    let mut st = store(d);
                    let mut mine = Vec::new();
                    for _ in 0..100 {
                        let mut i = st.instance("dev", "i-1").unwrap().unwrap();
                        i.version += 1;
                        i.state = format!("W{w}");
                        match st.put_instance(&i) {
                            Ok(()) => mine.push(i.version),
                            Err(HostError::Conflict) => {}
                            Err(e) => panic!("{e:?}"),
                        }
                    }
                    mine
                })
            })
            .collect();
        writers
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    accepted.sort_unstable();
    let last = store(&d).instance("dev", "i-1").unwrap().unwrap().version;
    assert_eq!(accepted, (2..=last).collect::<Vec<_>>());
    let mut st = store(&d);
    for v in [0, 1, last - 1, last, last + 2, u64::MAX] {
        assert_eq!(st.put_instance(&inst(v)), Err(HostError::Conflict), "{v}");
    }
    st.put_instance(&inst(last + 1)).unwrap();
    fs::remove_dir_all(d).ok();
}
