//! Open a page in the default web browser: the program that `BROWSER`
//! names, or else the platform's opener (CLI-8).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt as _;

use crate::error::{Error, Result};

/// How long the opener has to fail before smllm takes it to have started
/// the browser, which some openers run until it closes.
const WAIT: Duration = Duration::from_secs(5);

/// The command that opens `page`: `BROWSER PAGE` when `BROWSER` is set and
/// not empty, else `open` on macOS, `cmd /C start` on Windows, and
/// `xdg-open` elsewhere.
fn opener(browser: Option<OsString>, page: &Path) -> Command {
    if let Some(b) = browser.filter(|b| !b.is_empty()) {
        let mut c = Command::new(b);
        c.arg(page);
        return c;
    }
    if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg(page);
        c
    } else if cfg!(windows) {
        windows_start(page)
    } else {
        let mut c = Command::new("xdg-open");
        c.arg(page);
        c
    }
}

#[cfg(windows)]
fn windows_start(page: &Path) -> Command {
    use std::os::windows::process::CommandExt as _;
    // `cmd` never undoes the standard library's quoting, so the line goes raw;
    // a Windows path can't hold a `"`.
    let mut c = Command::new("cmd");
    c.raw_arg(format!(
        "/C start \"\" \"{}\"",
        smllm_format::system_path_text(page)
    ));
    c
}

#[cfg(not(windows))]
fn windows_start(page: &Path) -> Command {
    let mut c = Command::new("cmd");
    c.arg("/C").arg("start").arg("").arg(page);
    c
}

/// Open `page` in the default web browser.
// sokf-implements: CLI-8_AC-3
pub(crate) fn open(page: &Path) -> Result<()> {
    let mut cmd = opener(std::env::var_os("BROWSER"), page);
    let program = smllm_format::path_text(&PathBuf::from(cmd.get_program()));
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::msg(format!("cannot open the browser with {program}: {e}")))?;
    match child.wait_timeout(WAIT) {
        Ok(Some(status)) if !status.success() => Err(Error::msg(match status.code() {
            Some(code) => format!("cannot open the browser: {program} exited with code {code}"),
            None => format!("cannot open the browser: {program} was killed by a signal"),
        })),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(c: &Command) -> Vec<String> {
        std::iter::once(c.get_program())
            .chain(c.get_args())
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn browser_wins_unless_empty() {
        let page = Path::new("/tmp/p.html");
        let c = opener(Some("firefox".into()), page);
        assert_eq!(line(&c), ["firefox", "/tmp/p.html"]);
        let c = opener(Some("".into()), page);
        let want = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(windows) {
            "cmd"
        } else {
            "xdg-open"
        };
        assert_eq!(line(&c)[0], want);
        assert_eq!(line(&windows_start(page))[0], "cmd");
    }
}
