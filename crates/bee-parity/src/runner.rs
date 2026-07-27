//! Spawns one command invocation against a given root and captures a
//! [`crate::differ::RunResult`]. One shared spawn path so the root-safety
//! sanity read and the actual diffed run never drift apart — same command,
//! same capture shape, every time (CONTEXT.md D7a: "run the same command
//! through node .bee/bin/bee.mjs ... in one and ... in the other").
//!
//! rust-port-15 generalized this along two axes without changing what
//! `--self-check` does: WHICH runtime executes (`node bee.mjs` vs the
//! compiled `queen-bee`) and WHICH leg is captured (`--json` vs the human
//! text render).
//!
//! rpl-1 generalized it along two more:
//!
//! - **arbitrary argv.** [`run_argv`] takes the token list directly, so the
//!   harness is no longer `status`-shaped. [`run_status`] is now a thin
//!   wrapper over it and behaves exactly as before.
//! - **stderr is captured.** [`crate::differ::RunResult`] carried stdout and
//!   the exit code only, which made a whole class of contract invisible.
//!   Several behaviors later ledger cells must prove are stderr-ONLY: the
//!   reviews fail-open warning (`packages/bee/lib/reviews.mjs:130`
//!   `console.warn`), the `assertSafeContent` rejection text whose JS
//!   regex-literal spelling rpl-3 must reproduce
//!   (`packages/bee/lib/decisions.mjs:280-283`), every per-group unknown-VERB
//!   usage fallback and every unknown-FLAG refusal (`bee.mjs:6985`
//!   `emitError` writes to stderr whenever `--json` was not requested).
//!   Diffing stdout alone would have declared all of those "parity" without
//!   ever comparing the bytes that carry them.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::differ::RunResult;

/// Which runtime executes the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    /// `node <bee.mjs> …` — the frozen mjs oracle (D1).
    Mjs,
    /// `<queen-bee> …` — the ported Rust command.
    QueenBee,
}

impl Runtime {
    pub fn label(self) -> &'static str {
        match self {
            Runtime::Mjs => "mjs",
            Runtime::QueenBee => "queen-bee",
        }
    }
}

/// Which output leg is captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leg {
    /// `status --json` — the machine payload.
    Json,
    /// `status` — the human text render.
    Text,
    /// `status --json --lanes-full` — the legacy full per-lane array
    /// (lpsp-2). A SEPARATE leg because it is the only shape that
    /// serializes every lane record in full regardless of which session is
    /// bound, and it is therefore the one that exercises
    /// `build_lane_rows`'s key-order handling (rust-port-15: the
    /// `shift_remove` vs `swap_remove` hazard lives exactly there).
    JsonLanesFull,
}

impl Leg {
    pub fn label(self) -> &'static str {
        match self {
            Leg::Json => "json",
            Leg::Text => "text",
            Leg::JsonLanesFull => "json+lanes-full",
        }
    }

    /// Whether this leg's stdout is a JSON object (vs the text render).
    pub fn is_json(self) -> bool {
        matches!(self, Leg::Json | Leg::JsonLanesFull)
    }

    fn args(self) -> &'static [&'static str] {
        match self {
            Leg::Json => &["status", "--json"],
            Leg::Text => &["status"],
            Leg::JsonLanesFull => &["status", "--json", "--lanes-full"],
        }
    }
}

/// Where each runtime's entry point lives, resolved once by the caller.
#[derive(Debug, Clone)]
pub struct Binaries {
    pub bee_mjs: PathBuf,
    pub queen_bee: PathBuf,
}

/// Run ANY argv under one runtime, with `cwd` set to `root`.
///
/// `session_id` pins `resolveSessionId`'s env chain (claims.mjs:139) for
/// BOTH runtimes. This is not a convenience — it is COVERAGE CONTROL
/// (rust-port-15). `resolveSessionId` reads `BEE_SESSION_ID` then
/// `CLAUDE_CODE_SESSION_ID` from the ambient environment, and
/// `buildLaneSummary` uses the result to decide whether there IS an active
/// lane to serialize in full. Inheriting the parent's environment therefore
/// made this harness's branch coverage depend on who ran it: under a Claude
/// Code session, `CLAUDE_CODE_SESSION_ID` is set to an id with no session
/// record in the fixture, so `active` resolved to null and the full lane
/// row was never serialized — silently, and only on developer machines.
/// Both runtimes read the same env so parity itself was never at risk, but
/// the PROOF quietly shrank. Every leg now runs under an explicitly
/// controlled environment instead.
pub fn run_argv(
    bins: &Binaries,
    runtime: Runtime,
    argv: &[String],
    root: &Path,
    session_id: Option<&str>,
) -> Result<RunResult, String> {
    let mut command = match runtime {
        Runtime::Mjs => {
            let mut c = Command::new("node");
            c.arg(&bins.bee_mjs);
            c
        }
        Runtime::QueenBee => Command::new(&bins.queen_bee),
    };
    for arg in argv {
        command.arg(arg);
    }
    // Always cleared, then set only when the scenario asks for it — an
    // ambient value must never reach either runtime.
    command.env_remove("CLAUDE_CODE_SESSION_ID");
    match session_id {
        Some(id) => {
            command.env("BEE_SESSION_ID", id);
        }
        None => {
            command.env_remove("BEE_SESSION_ID");
        }
    }
    let output = command.current_dir(root).output().map_err(|e| {
        format!(
            "failed to spawn {} {} with cwd {}: {e}",
            describe(bins, runtime),
            argv.join(" "),
            root.display()
        )
    })?;

    Ok(RunResult {
        root: root.to_path_buf(),
        runtime,
        argv: argv.to_vec(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Run one leg of `status` under one runtime — the rust-port-15 shape,
/// unchanged, now expressed over [`run_argv`].
pub fn run_status(
    bins: &Binaries,
    runtime: Runtime,
    leg: Leg,
    root: &Path,
    session_id: Option<&str>,
) -> Result<RunResult, String> {
    let argv: Vec<String> = leg.args().iter().map(|s| (*s).to_string()).collect();
    run_argv(bins, runtime, &argv, root, session_id)
}

fn describe(bins: &Binaries, runtime: Runtime) -> String {
    match runtime {
        Runtime::Mjs => format!("`node {}`", bins.bee_mjs.display()),
        Runtime::QueenBee => format!("`{}`", bins.queen_bee.display()),
    }
}

/// The `--self-check` spawn shape, unchanged: the mjs leg's `status
/// --json` against `root`, with no session id pinned (the plain fixture
/// carries no session records, so there is no active lane either way).
pub fn run_bee_status(bee_mjs: &Path, root: &Path) -> Result<RunResult, String> {
    let bins = Binaries { bee_mjs: bee_mjs.to_path_buf(), queen_bee: PathBuf::new() };
    run_status(&bins, Runtime::Mjs, Leg::Json, root, None)
}
