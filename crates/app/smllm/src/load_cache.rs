//! Loads kept on disk between runs (CLI-4_AC-2), so that a command that runs
//! the machines, such as a status line refresh or a hook, reads the compiled
//! machines instead of parsing their YAML. A saved load is used only when the
//! binary and every file that the load read or looked for are as they were:
//! the same size and modification time, and still missing when they were
//! missing. A load is saved only when every one of those files is clearly
//! older than the load, the rule that git uses for its index ("racy git"), so
//! a later edit, even within the same timestamp tick, gets a newer time.
//! Only clean loads are saved: a load with findings is redone each time, so
//! it reports them as before. Every failure to read or write the cache falls
//! back to a fresh load.

use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use smllm_core::model::Config;
use smllm_format::{ConfigFile, Loaded, MachineSource, Mode, load_configs};

use crate::paths;

/// How much older than the load a file's time must be for the load to be
/// saved: covers a file system clock that differs from the process's (a VM or
/// network share) as well as a coarse timestamp.
const RACY: Duration = Duration::from_secs(2);

/// A file's size and modification time; `None` when it is missing.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Stamp {
    path: PathBuf,
    file: Option<(SystemTime, u64)>,
}

impl Stamp {
    fn now(path: &Path) -> Self {
        let file = fs::metadata(path)
            .ok()
            .and_then(|m| Some((m.modified().ok()?, m.len())));
        Self {
            path: path.to_path_buf(),
            file,
        }
    }
}

/// Where a saved machine lives: [`MachineSource`] on disk.
#[derive(Serialize, Deserialize)]
struct Source {
    id: String,
    file: PathBuf,
    config: PathBuf,
    state_dir: PathBuf,
}

/// One saved load.
#[derive(Serialize, Deserialize)]
struct Saved {
    /// The version and the binary that wrote it: another build may compile
    /// the machines differently.
    version: String,
    binary: Stamp,
    /// The config files loaded, in order.
    configs: Vec<PathBuf>,
    /// Every file that the load read or looked for.
    inputs: Vec<Stamp>,
    config: Config,
    machines: Vec<Source>,
}

/// The load of `files` to run them: the saved one when nothing it depends on
/// changed, else a fresh load, saved when it is clean.
pub fn load(files: &[ConfigFile]) -> Loaded {
    let Some(at) = cache_file(files) else {
        return load_configs(files, Mode::Run);
    };
    let binary = std::env::current_exe().map(|p| Stamp::now(&p)).ok();
    if let Some(saved) = binary.as_ref().and_then(|b| read(&at, files, b)) {
        let loaded = Loaded {
            config: saved.config,
            machines: saved.machines.into_iter().map(Source::into).collect(),
            findings: Default::default(),
            inputs: saved.inputs.into_iter().map(|s| s.path).collect(),
        };
        debug_assert!(
            same(&loaded, &load_configs(files, Mode::Run)),
            "the saved load of {at:?} differs from a fresh one"
        );
        return loaded;
    }
    // The time is taken before reading, so a file written during the load is
    // too recent to save.
    let built = SystemTime::now();
    let loaded = load_configs(files, Mode::Run);
    if let Some(binary) = binary {
        save(&at, files, binary, built, &loaded);
    }
    loaded
}

/// The cache file of `files`: one for each list of configs.
fn cache_file(files: &[ConfigFile]) -> Option<PathBuf> {
    if files.is_empty() {
        return None;
    }
    let mut h = std::hash::DefaultHasher::new();
    for f in files {
        f.path.hash(&mut h);
    }
    let dir = paths::user_cache_dir().ok()?;
    Some(dir.join("loads").join(format!("{:016x}.json", h.finish())))
}

/// The saved load at `at`, when it is for `files` and is still current.
// sokf-implements: CLI-4_AC-2
fn read(at: &Path, files: &[ConfigFile], binary: &Stamp) -> Option<Saved> {
    let saved: Saved = serde_json::from_slice(&fs::read(at).ok()?).ok()?;
    let current = saved.version == env!("CARGO_PKG_VERSION")
        && saved.binary == *binary
        && saved.configs.iter().eq(files.iter().map(|f| &f.path))
        && saved.inputs.iter().all(|s| *s == Stamp::now(&s.path));
    current.then_some(saved)
}

/// Save `loaded` when it is clean and none of its files is too recent.
fn save(at: &Path, files: &[ConfigFile], binary: Stamp, built: SystemTime, loaded: &Loaded) {
    if !loaded.findings.0.is_empty() || binary.file.is_none() {
        return;
    }
    let inputs: Vec<Stamp> = loaded.inputs.iter().map(|p| Stamp::now(p)).collect();
    let old = |s: &Stamp| s.file.is_none_or(|(t, _)| t + RACY < built);
    if !inputs.iter().all(old) {
        return;
    }
    let saved = Saved {
        version: env!("CARGO_PKG_VERSION").to_string(),
        binary,
        configs: files.iter().map(|f| f.path.clone()).collect(),
        inputs,
        config: loaded.config.clone(),
        machines: loaded.machines.iter().map(Source::from).collect(),
    };
    let Ok(text) = serde_json::to_vec(&saved) else {
        return;
    };
    // Written aside and renamed, so a concurrent run reads the old file or
    // the new one, never half of one.
    let Some(dir) = at.parent() else { return };
    let tmp = dir.join(format!(".{}.tmp", std::process::id()));
    let ok = fs::create_dir_all(dir).is_ok() && fs::write(&tmp, text).is_ok();
    if !ok || fs::rename(&tmp, at).is_err() {
        fs::remove_file(&tmp).ok();
    }
}

/// Whether two loads give the same runtime.
fn same(a: &Loaded, b: &Loaded) -> bool {
    a.config == b.config
        && a.machines == b.machines
        && a.inputs == b.inputs
        && b.findings.0.is_empty()
}

impl From<&MachineSource> for Source {
    fn from(m: &MachineSource) -> Self {
        Self {
            id: m.id.clone(),
            file: m.file.clone(),
            config: m.config.clone(),
            state_dir: m.state_dir.clone(),
        }
    }
}

impl From<Source> for MachineSource {
    fn from(s: Source) -> Self {
        Self {
            id: s.id,
            file: s.file,
            config: s.config,
            state_dir: s.state_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smllm_format::Origin;

    fn machine(id: &str, description: &str) -> String {
        format!(
            "id: {id}\ndescription: {description}\ninitial: A\nmeta: {{ smllm: 1 }}\nstates:\n  A: {{ type: final }}\n"
        )
    }

    fn set_time(path: &Path, t: SystemTime) {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(t)
            .unwrap();
    }

    // sokf-tests: CLI-4_AC-2
    #[test]
    fn a_saved_load_is_used_only_while_its_files_are_unchanged() {
        let d = crate::test_support::temp_dir("load-cache");
        let (config, m) = (d.join("config.toml"), d.join("m.smllm.yaml"));
        fs::write(&config, "[machines]\nfiles = [\"m.smllm.yaml\"]\n").unwrap();
        fs::write(&m, machine("m", "aaa")).unwrap();
        let files = [ConfigFile {
            path: config.clone(),
            origin: Origin::Explicit,
        }];
        let at = d.join("cache.json");
        let binary = Stamp::now(&d.join("binary"));
        fs::write(d.join("binary"), "1").unwrap();
        let binary_now = || Stamp::now(&d.join("binary"));
        let old = SystemTime::now() - Duration::from_secs(60);
        let age = || {
            for f in [&config, &m] {
                set_time(f, old);
            }
        };
        let fresh = || load_configs(&files, Mode::Run);

        // A file newer than the margin: the load is not saved.
        save(&at, &files, binary_now(), SystemTime::now(), &fresh());
        assert!(!at.exists());
        // Old files: saved, then read back the same as a fresh load.
        age();
        save(&at, &files, binary_now(), SystemTime::now(), &fresh());
        let saved = read(&at, &files, &binary_now()).expect("saved");
        assert_eq!(saved.config, fresh().config);
        // A changed size or time, another binary, or other configs: not
        // current.
        fs::write(&m, machine("m", "bbbb")).unwrap();
        assert!(read(&at, &files, &binary_now()).is_none());
        fs::write(&m, machine("m", "aaa")).unwrap();
        set_time(&m, old + Duration::from_secs(1));
        assert!(read(&at, &files, &binary_now()).is_none());
        set_time(&m, old);
        assert!(read(&at, &files, &binary_now()).is_some());
        assert!(read(&at, &files, &binary).is_none());
        assert!(read(&at, &[], &binary_now()).is_none());
        // A load with findings is not saved.
        fs::remove_file(&at).unwrap();
        fs::write(&m, "id: m\n").unwrap();
        age();
        let broken = fresh();
        assert!(!broken.findings.0.is_empty());
        save(&at, &files, binary_now(), SystemTime::now(), &broken);
        assert!(!at.exists());
        fs::remove_dir_all(d).ok();
    }
}
