// verbs — natively-served porcelain verbs. Each module exposes
// `try_native(args, t0) -> Option<ExitCode>`: Some(code) when it served the
// command end to end, None to fall through to the Node delegate. A probe
// must produce NO output before returning None.
//
// Shared plumbing lives here: the direct-run timing wrapper (timings.jsonl +
// the "[bee] <cmd> Nms" stderr line) and the no-root error path — both
// byte-identical to bee.mjs's own.

use crate::fsutil::append_jsonl;
use crate::jsjson;
use serde_json::json;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub mod backlog;
pub mod capture;
pub mod cells;
pub mod decisions;
pub mod drivers;
pub mod feedback;
pub mod help;
pub mod intent_group;
pub mod knowledge;
pub mod reservations;
pub mod reviews;
pub mod state_group;
pub mod status_brief;
pub mod status_full;
pub mod test_runner;
pub mod tmp_group;
pub mod workflow_store; // library module (no try_native) — never probed below
pub mod workspace_store; // library module (no try_native) — never probed below
pub mod worktree;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    // help probes FIRST: in bee.mjs, --help short-circuits main() before root
    // resolution, and group-scoped help fires before dispatch — so a `--help`
    // anywhere in the flag section must never reach a verb probe.
    if let Some(code) = help::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = status_brief::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = status_full::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = cells::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = state_group::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = drivers::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = test_runner::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = reservations::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = worktree::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = decisions::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = backlog::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = capture::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = feedback::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = intent_group::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = reviews::try_native(args, t0) {
        return Some(code);
    }
    if let Some(code) = tmp_group::try_native(args, t0) {
        return Some(code);
    }
    knowledge::try_native(args, t0)
}

/// bee.mjs's direct-run timing wrapper: fail-open append to
/// .bee/logs/timings.jsonl + the always-printed stderr line.
pub(crate) fn record_timing(root: &Path, cmd: &str, t0: Instant, ok: bool) {
    let ms = t0.elapsed().as_millis() as u64;
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let _ = append_jsonl(
        &root.join(".bee").join("logs").join("timings.jsonl"),
        &json!({"ts": ts, "cmd": cmd, "ms": ms, "ok": ok}),
    );
    eprintln!("[bee] {cmd} {ms}ms");
}

/// main()'s no-root emitError path + timing, for any verb.
pub(crate) fn emit_no_root_error(cwd: &Path, cmd: &str, use_json: bool, t0: Instant) -> ExitCode {
    let msg = "No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding.";
    if use_json {
        println!("{}", jsjson::stringify(&json!({ "error": msg })));
    } else {
        eprintln!("{msg}");
    }
    record_timing(cwd, cmd, t0, false);
    ExitCode::FAILURE
}
