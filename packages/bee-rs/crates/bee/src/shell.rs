// shell — the one resolver for the POSIX shell every declared-command spawn
// goes through (`bee test`, `bee cells finish`, `bee close`).
//
// WHY THIS MODULE EXISTS. Three copies of `posix_shell()` (verbs/test_runner.rs,
// verbs/drivers.rs, verbs/cells.rs) each spawned a BARE `bash` on Windows and
// trusted PATH order to land on Git Bash. It does not: on any host with WSL
// enabled, `%SystemRoot%\System32\bash.exe` — the WSL launcher — sits AHEAD of
// `C:\Program Files\Git\bin` on the system PATH, so the bare name resolves to
// Linux. Proven live on win32 with `commands.test = "uname -a"`:
//
//     Linux DESKTOP-… 6.6.87.2-microsoft-standard-WSL2 … x86_64 GNU/Linux
//
// That is the proof gate — the command `bee cells finish` runs to decide
// whether a cell may be capped — executing inside a different operating
// system, against `/mnt/<drive>` paths, with a different toolchain, after a
// multi-second VM cold start. A red there is not evidence about the repo, and
// the cold start under a parallel test run is where the suite's own
// intermittent failures came from.
//
// THE FIX keeps argv[0] as the bare word `bash` — so bash's own `bash: line
// 1: …` error prefixes are unchanged in failure excerpts — and instead hands
// the child a PATH whose FIRST entry is a directory holding a real Win32
// bash. Rust's Windows spawn searches the CHILD's PATH before the parent's,
// so resolution becomes deterministic instead of PATH-order-dependent.
//
// The WSL launcher is identified structurally, not by name: it is the only
// `bash.exe` that lives under `%SystemRoot%`. Every Git for Windows / MSYS2 /
// Cygwin bash lives elsewhere. `BEE_POSIX_SHELL` overrides the whole search
// with an explicit path, which is also the way to opt back INTO WSL.
//
// The probe runs ONCE per process and is memoized. The three old copies
// re-probed on every call; with WSL answering that probe, `bee test` paid a
// cold VM start just to ask whether a shell existed.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// What the resolver settled on: the program name to spawn (always the bare
/// `bash` / `/bin/sh` so argv[0] is stable) plus, on Windows, the directory to
/// put at the head of the child's PATH.
struct Resolved {
    program: &'static str,
    prepend: Option<PathBuf>,
}

fn resolved() -> Option<&'static Resolved> {
    static CELL: OnceLock<Option<Resolved>> = OnceLock::new();
    CELL.get_or_init(resolve).as_ref()
}

/// The declared-test shell, or None when this host has no POSIX shell to run
/// declared commands through. Callers treat None as "cannot run the proof
/// command here" — never as "the tests passed".
pub fn posix_shell() -> Option<&'static str> {
    resolved().map(|r| r.program)
}

/// A `Command` for the resolved shell, with the child PATH pinned so the bare
/// program name cannot re-resolve to something else mid-session.
pub fn command() -> Option<Command> {
    let r = resolved()?;
    let mut cmd = Command::new(r.program);
    if let Some(path) = child_path(r.prepend.as_deref()) {
        cmd.env("PATH", path);
    }
    Some(cmd)
}

/// `<prepend>;<inherited PATH>` on Windows; the inherited PATH untouched
/// elsewhere. Setting PATH explicitly is what makes Rust search the child's
/// list first, so it is set even when there is nothing to prepend.
fn child_path(prepend: Option<&Path>) -> Option<OsString> {
    let inherited = std::env::var_os("PATH");
    if !cfg!(windows) {
        return None;
    }
    match (prepend, inherited) {
        (Some(dir), Some(rest)) => {
            let mut out = OsString::from(dir);
            out.push(";");
            out.push(rest);
            Some(out)
        }
        (Some(dir), None) => Some(OsString::from(dir)),
        (None, rest) => rest,
    }
}

fn probe(program: &str, prepend: Option<&Path>) -> bool {
    let mut cmd = Command::new(program);
    if let Some(path) = child_path(prepend) {
        cmd.env("PATH", path);
    }
    cmd.args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn resolve() -> Option<Resolved> {
    if !cfg!(windows) {
        // POSIX: Node's `shell: true` already IS /bin/sh. Probe it the same
        // way so a host without one declines instead of embedding a spawn
        // error in a test record.
        return probe("/bin/sh", None).then_some(Resolved { program: "/bin/sh", prepend: None });
    }
    for dir in windows_bash_dirs() {
        if probe("bash", Some(&dir)) {
            return Some(Resolved { program: "bash", prepend: Some(dir) });
        }
    }
    None
}

/// Candidate directories holding a Win32 `bash.exe`, best first. Deduped, and
/// every `%SystemRoot%` hit dropped — that is the WSL launcher.
fn windows_bash_dirs() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    fn push(out: &mut Vec<PathBuf>, dir: PathBuf) {
        if !dir.join("bash.exe").is_file() || is_under_system_root(&dir) || out.contains(&dir) {
            return;
        }
        out.push(dir);
    }

    // 1. The explicit override — a full path to the bash to use. This is also
    //    how a host opts back INTO WSL, or into an MSYS2/Cygwin bash.
    if let Some(explicit) = std::env::var_os("BEE_POSIX_SHELL") {
        let p = PathBuf::from(explicit);
        if let Some(parent) = p.parent() {
            // Deliberately bypasses the %SystemRoot% filter: an explicit
            // setting is the operator's call, not a PATH-order accident.
            if p.is_file() && !out.contains(&parent.to_path_buf()) {
                out.push(parent.to_path_buf());
            }
        }
    }

    // 2. PATH, in order, minus %SystemRoot%.
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            push(&mut out, dir);
        }
    }

    // 3. Git for Windows' usual homes, for a PATH that never had Git on it.
    for base in [
        std::env::var_os("ProgramFiles"),
        std::env::var_os("ProgramFiles(x86)"),
        std::env::var_os("ProgramW6432"),
        std::env::var_os("LOCALAPPDATA").map(|l| {
            let mut p = PathBuf::from(l);
            p.push("Programs");
            p.into_os_string()
        }),
    ]
    .into_iter()
    .flatten()
    {
        push(&mut out, Path::new(&base).join("Git").join("bin"));
        push(&mut out, Path::new(&base).join("Git").join("usr").join("bin"));
    }

    // 4. Derived from wherever git.exe itself lives (`…/cmd/git.exe` and
    //    `…/bin/git.exe` both sit one level under the install root).
    for dir in git_exe_dirs() {
        if let Some(root) = dir.parent() {
            push(&mut out, root.join("bin"));
            push(&mut out, root.join("usr").join("bin"));
        }
    }
    out
}

fn git_exe_dirs() -> Vec<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else { return Vec::new() };
    std::env::split_paths(&path).filter(|d| d.join("git.exe").is_file()).collect()
}

/// True for `C:\Windows\System32`, `…\Sysnative`, `…\SysWOW64` and anything
/// else under the Windows directory — the only place the WSL `bash.exe`
/// shim lives.
fn is_under_system_root(dir: &Path) -> bool {
    let Some(root) = std::env::var_os("SystemRoot").or_else(|| std::env::var_os("windir")) else {
        return false;
    };
    let norm = |s: &OsStr| s.to_string_lossy().to_lowercase().replace('/', "\\");
    let root = norm(&root);
    let root = root.trim_end_matches('\\');
    let dir = norm(dir.as_os_str());
    dir == root || dir.starts_with(&format!("{root}\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_probe_is_memoized_across_calls() {
        // Two calls, one answer object — the old per-call probe paid a WSL
        // cold start every time `bee test` asked whether a shell existed.
        let a = resolved().map(|r| r as *const Resolved);
        let b = resolved().map(|r| r as *const Resolved);
        assert_eq!(a, b);
    }

    #[cfg(windows)]
    #[test]
    fn system_root_directories_are_never_candidates() {
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let sys32 = Path::new(&root).join("System32");
        assert!(is_under_system_root(&sys32));
        assert!(is_under_system_root(Path::new(&root)));
        assert!(!is_under_system_root(Path::new("C:\\Program Files\\Git\\bin")));
        assert!(
            !windows_bash_dirs().iter().any(|d| is_under_system_root(d)),
            "the WSL launcher's directory must never be a candidate"
        );
    }

    /// The regression that motivated the module: the resolved shell must see
    /// the SAME filesystem the binary does. WSL bash reports the repo as
    /// `/mnt/d/…`; a Win32 bash reports `/d/…` (MSYS) or `/cygdrive/d/…`.
    #[cfg(windows)]
    #[test]
    fn the_resolved_shell_is_not_wsl() {
        let Some(mut cmd) = command() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let out = cmd.args(["-c", "pwd"]).current_dir(tmp.path()).output().unwrap();
        let pwd = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert!(
            !pwd.starts_with("/mnt/"),
            "the proof gate resolved to WSL bash (pwd={pwd}) — the test command would run in \
             another OS against another toolchain"
        );
        // …and it is genuinely the directory it was pointed at, not a VM's
        // idea of one.
        assert!(!pwd.is_empty(), "the resolved shell produced no pwd at all");
    }

    #[test]
    fn a_resolved_shell_runs_the_command_it_is_given() {
        let Some(mut cmd) = command() else { return };
        let out = cmd.args(["-c", "echo bee-shell-ok"]).output().unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "bee-shell-ok");
    }
}
