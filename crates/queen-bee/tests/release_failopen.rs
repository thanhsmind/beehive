//! release_failopen — rust-port-18 (FIX-FIRST, validation decision
//! 2026-07-26, P1): proves the fail-open contract (D7b) holds in the
//! ACTUAL shipped artifact — `crates/target/release/queen-bee`, built with
//! `[profile.release]` from `crates/Cargo.toml` — not merely under
//! `cargo test`'s dev/test profile (`panic = "unwind"` by default).
//!
//! WHY THIS TARGET EXISTS: `crates/Cargo.toml` used to set
//! `panic = "abort"` on `[profile.release]`. `run_fail_open`
//! (`crates/queen-bee/src/hooks/write_guard.rs`) wraps every hook decision
//! in `std::panic::catch_unwind` so an internal bug fails open (exit 0 +
//! a `.bee/logs/hooks.jsonl` crash line) instead of ever flipping a
//! decision. `catch_unwind` cannot catch anything once the panic runtime
//! is `abort` — the process aborts (SIGABRT / a non-zero, non-catchable
//! exit) instead. `cargo test` (no `--release`) always builds and runs
//! against the dev/test profile (`panic = "unwind"`), so every prior
//! green in `hook_conformance.rs`, `writeguard_core.rs`, `writeguard_bash.rs`,
//! `writeguard_read.rs`, and `modelguard_conformance.rs` proved the
//! fail-open contract under a profile the host never ships — the same
//! failure class as the vendoring-drift pattern (a test that runs the
//! wrong artifact proves nothing).
//!
//! CRASH TRIGGER SEAM: the existing crash fixtures
//! (`hook_conformance.rs::seed_crash_root`, and the "logs path is
//! blocked" fixture in the same file) inject a HANDLED I/O fault
//! (`create_dir_all` returning `Err`, absorbed via `Result`) — it never
//! unwinds the stack, so it is silent on the `panic = "abort"` question
//! either way. `crash_seam_panic_if_armed` (`write_guard.rs`) is a
//! genuine `panic!()`, inert unless the fixture-only env var
//! `BEE_QUEEN_BEE_CRASH_SEAM` is set to the exact hook name — never set by
//! any hook runtime, host repo, or CI path outside this test file.
//!
//! RED-FIRST (recorded in the cell trace, not re-run by this file itself):
//! with `panic = "abort"` still in `[profile.release]`, running this exact
//! target against the release binary aborts the CHILD PROCESS (observed
//! as `status: None`/signal termination, never `Ok(0)`) instead of the
//! green assertions below passing. That run and its verbatim output is
//! quoted in `docs/history/rust-port/reports/rust-port-18.md`'s linked
//! cell trace. The fix — dropping `panic = "abort"` from
//! `[profile.release]` (workspace `members` list untouched) — is what
//! turns this target green.
//!
//! Every temp root below comes from `tempfile::tempdir()` — never the
//! live `.bee/` store (same rig discipline as the other conformance
//! corpora in this crate).

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

const CRASH_SEAM_ENV: &str = "BEE_QUEEN_BEE_CRASH_SEAM";

// ---------------------------------------------------------------------------
// Release-binary resolution
// ---------------------------------------------------------------------------

/// The workspace's `target/release` output. Deliberately NOT
/// `env!("CARGO_BIN_EXE_queen-bee")` — that resolves to whatever profile
/// `cargo test` itself built (dev/test unless `cargo test --release` was
/// passed, which this cell's verify command never does), exactly the
/// artifact this target exists to stop trusting. The cell's verify command
/// runs `cargo build --release --manifest-path crates/Cargo.toml` before
/// this target, so the binary is expected to already exist here.
fn release_binary_path() -> PathBuf {
    // CARGO_MANIFEST_DIR for the queen-bee package is `crates/queen-bee`;
    // the workspace (and its default target dir) is one level up, `crates/`.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("queen-bee crate has a parent dir")
        .to_path_buf();
    let bin = workspace_root.join("target").join("release").join("queen-bee");
    assert!(
        bin.exists(),
        "release binary not found at {bin:?} — run `cargo build --release --manifest-path crates/Cargo.toml` first \
         (this target's own cell verify command does exactly that before invoking `cargo test`)"
    );
    // Structural proof this is really the release artifact, not a
    // dev/test-profile build living under a differently named target dir.
    let components: Vec<_> = bin.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    assert!(
        components.iter().any(|c| c == "release"),
        "resolved binary path {bin:?} does not contain a `release` path component"
    );
    assert!(
        !components.iter().any(|c| c == "debug"),
        "resolved binary path {bin:?} unexpectedly contains a `debug` path component"
    );
    bin
}

// ---------------------------------------------------------------------------
// Minimal seeding — only what `write-guard`/`model-guard`'s `run()` needs to
// reach the `run_fail_open` closure: an onboarding marker (root discovery,
// `adapter.rs::locate_onboarded_root`) and a present (content-irrelevant)
// `.bee/bin/lib/state.mjs` (the early-exit existence check both hooks share).
// No `.git` dir, so `resolve_roots` takes the ordinary (non-worktree) path.
// Hook enablement defaults to `true` when `.bee/config.json` is absent
// (`hookconfig::hook_enabled`), so no config file is needed either.
// ---------------------------------------------------------------------------

struct SeededRoot {
    _dir: TempDir,
    root: PathBuf,
}

fn seed_minimal_root() -> SeededRoot {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    fs::create_dir_all(root.join(".bee/bin/lib")).expect("mkdir .bee/bin/lib");
    fs::write(root.join(".bee/onboarding.json"), b"{}\n").expect("write onboarding.json");
    fs::write(root.join(".bee/bin/lib/state.mjs"), b"").expect("write state.mjs marker");
    SeededRoot { _dir: dir, root }
}

// ---------------------------------------------------------------------------
// Running the release binary
// ---------------------------------------------------------------------------

struct RunResult {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_release_hook(hook_name: &str, root: &Path, stdin: &str, arm_seam: bool) -> RunResult {
    let mut cmd = Command::new(release_binary_path());
    cmd.arg("hook").arg(hook_name);
    cmd.current_dir(root);
    cmd.env_remove("BEE_AGENT_NAME");
    if arm_seam {
        cmd.env(CRASH_SEAM_ENV, hook_name);
    } else {
        cmd.env_remove(CRASH_SEAM_ENV);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap_or_else(|e| panic!("spawn {hook_name} release binary failed: {e}"));
    use std::io::Write as _;
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait for child");
    RunResult {
        status: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// A tool name that is a no-op for BOTH hooks' non-crash decision logic:
/// not in write-guard's `WRITE_TOOLS`/`Bash`/`APPLY_PATCH_TOOLS`/
/// `READ_TOOLS`, and not `Agent`/`Task`/`spawn_agent` for model-guard — so
/// the unarmed baseline is a clean, side-effect-free `Outcome::Allow` /
/// early-return-0 on both, and only the armed crash seam (which fires
/// before any tool-name branching) can produce a non-zero/non-clean
/// outcome. Keeps these fixtures about the fail-open boundary only, never
/// tangled with unrelated deny logic.
const NOOP_TOOL: &str = "ReleaseFailopenProbe";

fn hook_stdin() -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": NOOP_TOOL,
    })
    .to_string()
}

fn read_hooks_log(root: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(root.join(".bee/logs/hooks.jsonl")) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}")))
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture class: crash seam armed — proves the RELEASE binary's
// catch_unwind boundary genuinely fails open under a real unwind.
// ---------------------------------------------------------------------------

#[test]
fn write_guard_release_binary_fails_open_on_armed_crash_seam() {
    let seeded = seed_minimal_root();
    let result = run_release_hook("write-guard", &seeded.root, &hook_stdin(), true);
    assert_eq!(
        result.status,
        Some(0),
        "fail-open contract: an internal panic in the RELEASE binary must still exit 0 — stdout={:?} stderr={:?}",
        result.stdout,
        result.stderr
    );
    assert!(result.stdout.trim().is_empty(), "a crash must never leak to stdout");
}

#[test]
fn write_guard_release_binary_crash_seam_logs_hooks_jsonl_crash_line() {
    let seeded = seed_minimal_root();
    run_release_hook("write-guard", &seeded.root, &hook_stdin(), true);
    let lines = read_hooks_log(&seeded.root);
    assert!(
        lines.iter().any(|l| {
            l.get("error").and_then(Value::as_str).map(|s| s.contains("rust-port-18 test-only crash seam")).unwrap_or(false)
        }),
        "expected a crash log line naming the armed seam, got {lines:?}"
    );
}

#[test]
fn model_guard_release_binary_fails_open_on_armed_crash_seam() {
    let seeded = seed_minimal_root();
    let result = run_release_hook("model-guard", &seeded.root, &hook_stdin(), true);
    assert_eq!(
        result.status,
        Some(0),
        "fail-open contract: an internal panic in the RELEASE binary must still exit 0 — stdout={:?} stderr={:?}",
        result.stdout,
        result.stderr
    );
    assert!(result.stdout.trim().is_empty(), "a crash must never leak to stdout");
}

#[test]
fn model_guard_release_binary_crash_seam_logs_hooks_jsonl_crash_line() {
    let seeded = seed_minimal_root();
    run_release_hook("model-guard", &seeded.root, &hook_stdin(), true);
    let lines = read_hooks_log(&seeded.root);
    assert!(
        lines.iter().any(|l| {
            l.get("error").and_then(Value::as_str).map(|s| s.contains("rust-port-18 test-only crash seam")).unwrap_or(false)
        }),
        "expected a crash log line naming the armed seam, got {lines:?}"
    );
}

// ---------------------------------------------------------------------------
// Fixture class: seam is inert without its exact env gate — proves the
// trigger can never fire outside this test file's deliberate arming.
// ---------------------------------------------------------------------------

#[test]
fn write_guard_release_binary_crash_seam_stays_inert_without_env_gate() {
    let seeded = seed_minimal_root();
    let result = run_release_hook("write-guard", &seeded.root, &hook_stdin(), false);
    assert_eq!(result.status, Some(0), "unarmed run must exit 0 — stderr={:?}", result.stderr);
    let lines = read_hooks_log(&seeded.root);
    assert!(lines.is_empty(), "unarmed run must never produce a crash log line, got {lines:?}");
}

#[test]
fn model_guard_release_binary_crash_seam_stays_inert_without_env_gate() {
    let seeded = seed_minimal_root();
    let result = run_release_hook("model-guard", &seeded.root, &hook_stdin(), false);
    assert_eq!(result.status, Some(0), "unarmed run must exit 0 — stderr={:?}", result.stderr);
    let lines = read_hooks_log(&seeded.root);
    assert!(lines.is_empty(), "unarmed run must never produce a crash log line, got {lines:?}");
}

/// A mismatched hook name in the env var must never arm a DIFFERENT hook's
/// seam — proves the gate compares the exact hook name, not "any value set".
#[test]
fn crash_seam_env_set_for_a_different_hook_name_does_not_arm() {
    let seeded = seed_minimal_root();
    let mut cmd = Command::new(release_binary_path());
    cmd.arg("hook").arg("write-guard");
    cmd.current_dir(&seeded.root);
    cmd.env_remove("BEE_AGENT_NAME");
    cmd.env(CRASH_SEAM_ENV, "model-guard");
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn write-guard release binary");
    use std::io::Write as _;
    child.stdin.take().unwrap().write_all(hook_stdin().as_bytes()).unwrap();
    let out = child.wait_with_output().expect("wait for child");
    assert_eq!(out.status.code(), Some(0), "mismatched-name env must not arm the seam");
    let lines = read_hooks_log(&seeded.root);
    assert!(lines.is_empty(), "mismatched-name env must never log a crash, got {lines:?}");
}
