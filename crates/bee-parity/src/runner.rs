//! Spawns one `status` invocation against a given root and captures a
//! [`crate::differ::RunResult`]. One shared spawn path so the root-safety
//! sanity read and the actual diffed run never drift apart — same command,
//! same capture shape, every time (CONTEXT.md D7a: "run the same command
//! through node .bee/bin/bee.mjs ... in one and ... in the other").
//!
//! rust-port-15 generalized this along two axes without changing what
//! `--self-check` does: WHICH runtime executes (`node bee.mjs` vs the
//! compiled `queen-bee`) and WHICH leg is captured (`--json` vs the human
//! text render). Both legs must be diffable, since D3 byte-compatibility
//! covers the text renderer too, not only the JSON payload.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::differ::RunResult;

/// Which runtime executes the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    /// `node <bee.mjs> status ...` — the frozen mjs oracle (D1).
    Mjs,
    /// `<queen-bee> status ...` — the ported Rust command.
    QueenBee,
}

/// Which output leg is captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leg {
    /// `status --json` — the machine payload.
    Json,
    /// `status` — the human text render.
    Text,
}

impl Leg {
    pub fn label(self) -> &'static str {
        match self {
            Leg::Json => "json",
            Leg::Text => "text",
        }
    }

    fn args(self) -> &'static [&'static str] {
        match self {
            Leg::Json => &["status", "--json"],
            Leg::Text => &["status"],
        }
    }
}

/// Where each runtime's entry point lives, resolved once by the caller.
#[derive(Debug, Clone)]
pub struct Binaries {
    pub bee_mjs: PathBuf,
    pub queen_bee: PathBuf,
}

/// Run one leg of `status` under one runtime, with `cwd` set to `root`.
pub fn run_status(bins: &Binaries, runtime: Runtime, leg: Leg, root: &Path) -> Result<RunResult, String> {
    let mut command = match runtime {
        Runtime::Mjs => {
            let mut c = Command::new("node");
            c.arg(&bins.bee_mjs);
            c
        }
        Runtime::QueenBee => Command::new(&bins.queen_bee),
    };
    for arg in leg.args() {
        command.arg(arg);
    }
    let output = command
        .current_dir(root)
        .output()
        .map_err(|e| format!("failed to spawn {} {} with cwd {}: {e}", describe(bins, runtime), leg.args().join(" "), root.display()))?;

    Ok(RunResult {
        root: root.to_path_buf(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

fn describe(bins: &Binaries, runtime: Runtime) -> String {
    match runtime {
        Runtime::Mjs => format!("`node {}`", bins.bee_mjs.display()),
        Runtime::QueenBee => format!("`{}`", bins.queen_bee.display()),
    }
}

/// The `--self-check` spawn shape, unchanged: the mjs leg's `status
/// --json` against `root`.
pub fn run_bee_status(bee_mjs: &Path, root: &Path) -> Result<RunResult, String> {
    let bins = Binaries { bee_mjs: bee_mjs.to_path_buf(), queen_bee: PathBuf::new() };
    run_status(&bins, Runtime::Mjs, Leg::Json, root)
}
