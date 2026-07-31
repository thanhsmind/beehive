// Delegation to the Node runtime for verbs not yet ported.
//
// Entry resolution order:
//   1. BEE_JS_ENTRY env var (absolute path to bee.mjs) — used by the diff
//      harness and by vendored installs whose layout differs from the repo.
//   2. Walk upward from the executable's directory, then from cwd, looking
//      for packages/bee/bee.mjs (repo checkout layout).
//
// stdio is inherited, so hook stdin JSON, TTY detection, and streaming output
// behave exactly as a direct `node bee.mjs` invocation. Exit code propagates.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

const ENTRY_REL: &str = "packages/bee/bee.mjs";

fn find_entry() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("BEE_JS_ENTRY") {
        let p = PathBuf::from(p);
        return if p.is_file() {
            Ok(p)
        } else {
            // Explicit config wins: a set-but-wrong BEE_JS_ENTRY is a hard
            // error, never a silent fallback to path search.
            Err(format!("BEE_JS_ENTRY points to a missing file: {}", p.display()))
        };
    }
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            starts.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    for start in starts {
        let mut dir = Some(start.as_path());
        while let Some(d) = dir {
            let candidate = d.join(ENTRY_REL);
            if candidate.is_file() {
                return Ok(candidate);
            }
            dir = d.parent();
        }
    }
    Err(format!(
        "cannot locate {ENTRY_REL} — set BEE_JS_ENTRY or run inside a bee checkout"
    ))
}

pub fn delegate(args: &[OsString]) -> ExitCode {
    let entry = match find_entry() {
        Ok(e) => e,
        Err(msg) => {
            eprintln!("bee(rs): {msg}");
            return ExitCode::from(127);
        }
    };

    if std::env::var_os("BEE_RS_TRACE").is_some() {
        eprintln!("bee(rs): delegate -> node {}", entry.display());
    }

    let status = Command::new("node").arg(&entry).args(args).status();
    match status {
        Ok(s) => ExitCode::from(s.code().unwrap_or(1).clamp(0, 255) as u8),
        Err(e) => {
            eprintln!("bee(rs): failed to spawn node: {e}");
            ExitCode::from(127)
        }
    }
}
