//! heavyhooks_conformance — D7b conformance rig (rust-port-17) for the two
//! remaining heavy hooks whose dependencies are now all in Rust:
//! `queen-bee hook chain-nudge` (advisory SubagentStop nudge) and `queen-bee
//! hook state-sync` (silent state-projection/heartbeat/lease/hold refresh),
//! proved against the REAL `.bee/bin/hooks/bee-chain-nudge.mjs` /
//! `bee-state-sync.mjs` (run through the shared, frozen `adapter.mjs`) —
//! never a reimplementation guess.
//!
//! RIG DISCIPLINE (inherited from rust-port-7's hook_conformance rig, per
//! that file's own restatement in modelguard_conformance.rs):
//! (i) SEEDING — every node-oracle run happens inside a fresh temp root
//!     seeded with `.bee/bin/lib/`, `.bee/bin/hooks/`, `.bee/onboarding.json`,
//!     and a `config.json`; the oracle executes the SEEDED copy of the hook
//!     file, sha256-verified against the repo source (this rig's oracle
//!     surface additionally spans every lib module the two hooks
//!     dynamically import: state.mjs, cells.mjs, lock.mjs,
//!     state-projection.mjs, claims.mjs, reservations.mjs, lease-store.mjs,
//!     worktree-holds.mjs, workflow-store.mjs, fsutil.mjs).
//! (ii) NON-TRIVIALITY BOTH WAYS — every "silent"/"skip" fixture is paired
//!      with a "nudge"/"renew" twin in the same shape with exactly one
//!      field flipped, so a mis-seeded root can never read as a genuine
//!      allow either way.
//! (iii) NEGATIVE CONTROL — an unseeded root must be DETECTED as invalid by
//!       the rig's own verifier, not just observed to behave differently.
//! (iv) Fixture names are descriptive test-function names (cargo test's own
//!      output lists them per class) so cap evidence can quote class
//!      coverage, not just a passed count.
//!
//! Fixture construction for the write-path tests (session/claim/lease/hold)
//! goes through `tests/support/heavyhooks_fixture.mjs`, a tracked driver
//! that seeds authentic records via the REAL `claims.mjs`/`lease-store.mjs`/
//! `worktree-holds.mjs` functions (never a hand-guessed on-disk shape,
//! particularly for a lease file's sha256-hashed name) — same "never
//! `node -e`" discipline as `tests/support/lock_driver.mjs`.
//!
//! Every temp root below comes from `tempfile::tempdir()` — never the
//! repo's live `.bee/` store.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Repo / binary / oracle-script resolution
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(".bee/bin/hooks/adapter.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("could not locate .bee/bin/hooks/adapter.mjs walking ancestors from {}", env!("CARGO_MANIFEST_DIR"));
}

fn queen_bee_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_queen-bee"))
}

fn fixture_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/heavyhooks_fixture.mjs")
}

// ---------------------------------------------------------------------------
// Seeding (rig discipline i)
// ---------------------------------------------------------------------------

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {dst:?}: {e}"));
    for entry in fs::read_dir(src).unwrap_or_else(|e| panic!("read_dir {src:?}: {e}")) {
        let entry = entry.expect("dir entry");
        let file_type = entry.file_type().expect("file type");
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap_or_else(|e| panic!("copy {:?} -> {target:?}: {e}", entry.path()));
        }
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The oracle surface: every module BOTH hooks' dynamic imports reach. A
/// doctored seeded copy of any of these is a rig failure.
const SHA_CHECKED: &[&str] = &[
    "bin/hooks/adapter.mjs",
    "bin/hooks/bee-chain-nudge.mjs",
    "bin/hooks/bee-state-sync.mjs",
    "bin/lib/state.mjs",
    "bin/lib/cells.mjs",
    "bin/lib/lock.mjs",
    "bin/lib/state-projection.mjs",
    "bin/lib/workflow-store.mjs",
    "bin/lib/claims.mjs",
    "bin/lib/reservations.mjs",
    "bin/lib/lease-store.mjs",
    "bin/lib/worktree-holds.mjs",
    "bin/lib/fsutil.mjs",
];

struct SeededRoot {
    _dir: TempDir,
    root: PathBuf,
}

fn seed_root(hooks_config: Value) -> SeededRoot {
    let repo = repo_root();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();

    copy_dir_all(&repo.join(".bee/bin/lib"), &root.join(".bee/bin/lib"));
    copy_dir_all(&repo.join(".bee/bin/hooks"), &root.join(".bee/bin/hooks"));
    fs::copy(repo.join(".bee/onboarding.json"), root.join(".bee/onboarding.json")).expect("copy onboarding.json");
    fs::write(root.join(".bee/config.json"), serde_json::to_string_pretty(&json!({ "hooks": hooks_config })).expect("serialize config.json"))
        .expect("write config.json");

    for rel in SHA_CHECKED {
        let src = repo.join(".bee").join(rel);
        let dst = root.join(".bee").join(rel);
        assert_eq!(sha256_hex(&src), sha256_hex(&dst), "seeded {rel} diverged from repo source — rig failure (doctored oracle copy)");
    }

    SeededRoot { _dir: dir, root }
}

/// Two independently seeded, identically configured roots — node runs
/// against one, the rust port against the other, so writes from either side
/// can never land in the same file.
fn seed_pair(hooks_config: Value) -> (SeededRoot, SeededRoot) {
    (seed_root(hooks_config.clone()), seed_root(hooks_config))
}

/// A root seeded like [`seed_root`], except `.bee/bin/lib/state.mjs` is
/// replaced with a deliberately-throwing stub — the crash-fail-open fault
/// injection point (both hooks' very first in-try call is
/// `stateLib.hookEnabled(...)`, so the crash fires before any other lib
/// import runs). `state.mjs` is a downstream DEPENDENCY, not the oracle
/// unit under test; breaking it is an INPUT condition, never a doctored
/// copy of the hook/adapter files actually being diffed — those stay
/// sha256-checked below.
fn seed_crash_root(hooks_config: Value) -> SeededRoot {
    let seeded = seed_root(hooks_config);
    fs::write(
        seeded.root.join(".bee/bin/lib/state.mjs"),
        b"export function hookEnabled() { throw new Error('rig-injected-fault: state.mjs unavailable'); }\n",
    )
    .expect("inject crash fault into state.mjs");

    for rel in ["bin/hooks/adapter.mjs", "bin/hooks/bee-chain-nudge.mjs", "bin/hooks/bee-state-sync.mjs"] {
        let src = repo_root().join(".bee").join(rel);
        let dst = seeded.root.join(".bee").join(rel);
        assert_eq!(sha256_hex(&src), sha256_hex(&dst), "seeded {rel} diverged from repo source — rig failure");
    }
    seeded
}

/// Rig discipline (iii): the verifier itself must DETECT an invalid/unseeded
/// root, not rubber-stamp it.
fn is_seeded_valid(root: &Path) -> bool {
    let repo = repo_root();
    for rel in SHA_CHECKED {
        let dst = root.join(".bee").join(rel);
        if !dst.exists() {
            return false;
        }
        let src = repo.join(".bee").join(rel);
        if sha256_hex(&src) != sha256_hex(&dst) {
            return false;
        }
    }
    root.join(".bee/onboarding.json").exists() && root.join(".bee/config.json").exists()
}

fn enabling_config() -> Value {
    json!({ "chain-nudge": true, "state-sync": true })
}

// ---------------------------------------------------------------------------
// Fixture seeding (session/claim/lease/hold) via heavyhooks_fixture.mjs
// ---------------------------------------------------------------------------

fn run_fixture(root: &Path, op: &str, args: &[&str]) -> Value {
    let lib_dir = root.join(".bee/bin/lib");
    let mut cmd = Command::new("node");
    cmd.arg(fixture_script()).arg(op).arg(&lib_dir).arg(root).args(args);
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn heavyhooks_fixture.mjs {op}: {e}"));
    assert!(out.status.success(), "heavyhooks_fixture.mjs {op} failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("bad fixture JSON {stdout:?}: {e}"))
}

fn seed_session(root: &Path, id: &str, heartbeat_ago_secs: i64) -> PathBuf {
    let v = run_fixture(root, "session", &[id, &heartbeat_ago_secs.to_string()]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

fn seed_claim(root: &Path, cell: &str, session: &str, claimed_ago_secs: i64) -> PathBuf {
    let v = run_fixture(root, "claim", &[cell, session, &claimed_ago_secs.to_string()]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

fn seed_lease(root: &Path, session: &str, path_id: &str, ttl_secs: i64) -> PathBuf {
    let v = run_fixture(root, "lease", &[session, path_id, &ttl_secs.to_string()]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

fn seed_hold(root: &Path, holder: &str, session: &str, path: &str, ttl_secs: i64) -> PathBuf {
    let v = run_fixture(root, "hold", &[holder, session, path, &ttl_secs.to_string()]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

/// Seeds a live (`status: "active"`) workflow record via the real
/// `workflow-store.mjs` `createWorkflow` — routes `rebuildStateProjection`
/// through its AUTHORITATIVE branch instead of the no-workflow-records
/// no-op every other state-sync fixture in this file exercises (rework
/// finding 2).
fn seed_workflow(root: &Path, feature: &str, phase: &str, mode: &str, summary: &str, next_action: &str) -> PathBuf {
    let v = run_fixture(root, "workflow", &[feature, phase, mode, summary, next_action]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

/// Binds an existing session to a lane via the real `claims.mjs`
/// `bindSessionLane` (sets `session.lane` in place) — authentic field
/// naming for `state::resolve_pipeline`'s lane branch (rework rust-port-21,
/// finding 1's session-id fixture requirement).
fn bind_session_lane(root: &Path, session_id: &str, feature: &str) -> PathBuf {
    let v = run_fixture(root, "bind-lane", &[session_id, feature]);
    PathBuf::from(v["file"].as_str().expect("fixture file path"))
}

/// `.bee/lanes/<feature>.json` — plain object write (no hashing/locking
/// concerns here, unlike a lease file), matching the `State`-shaped lane
/// record `state::read_lane` expects (`feature` must equal the lane name).
fn write_lane_json(root: &Path, feature: &str, value: &Value) {
    let dir = root.join(".bee/lanes");
    fs::create_dir_all(&dir).expect("mkdir lanes");
    fs::write(dir.join(format!("{feature}.json")), serde_json::to_string_pretty(value).unwrap()).expect("write lane fixture");
}

fn seed_cell(root: &Path, id: &str, feature: &str, status: &str, behavior_change: bool, capped_at: Option<&str>) {
    let dir = root.join(".bee/cells");
    fs::create_dir_all(&dir).expect("mkdir cells");
    let mut trace = json!({});
    trace["behavior_change"] = json!(behavior_change);
    if let Some(ts) = capped_at {
        trace["capped_at"] = json!(ts);
    }
    let cell = json!({ "id": id, "feature": feature, "status": status, "trace": trace });
    fs::write(dir.join(format!("{id}.json")), serde_json::to_string_pretty(&cell).unwrap()).expect("write cell fixture");
}

fn write_state_json(root: &Path, value: &Value) {
    fs::create_dir_all(root.join(".bee")).expect("mkdir .bee");
    fs::write(root.join(".bee/state.json"), serde_json::to_string_pretty(value).unwrap()).expect("write state.json fixture");
}

fn read_json_file(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// The cross-worktree holds ledger is `{"holds": [...]}` — every fixture in
/// this file seeds exactly one row, so this pulls it out for direct
/// field-level comparison instead of comparing the whole ledger wrapper.
fn first_hold_row(ledger: &Value) -> Value {
    ledger
        .get("holds")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or_else(|| panic!("holds ledger had no rows: {ledger:?}"))
}

// ---------------------------------------------------------------------------
// Running both runtimes
// ---------------------------------------------------------------------------

struct RunResult {
    status: i32,
    stdout: String,
    stderr: String,
}

fn run(mut cmd: Command, stdin: &str) -> RunResult {
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap_or_else(|e| panic!("spawn failed: {e}"));
    child.stdin.take().expect("child stdin").write_all(stdin.as_bytes()).expect("write stdin");
    let out = child.wait_with_output().expect("wait for child");
    RunResult {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Runs the SEEDED copy of the hook file living inside `root` — the real
/// oracle per rig discipline (i).
fn run_seeded_node_hook(root: &Path, hook_file: &str, stdin: &str) -> RunResult {
    let mut cmd = Command::new("node");
    cmd.arg(root.join(".bee/bin/hooks").join(hook_file));
    cmd.current_dir(root);
    run(cmd, stdin)
}

/// Runs the LIVE repo hook file directly — used only by the negative-control
/// fixture, which has nothing to seed by design.
fn run_live_node_hook(hook_file: &str, cwd: &Path, stdin: &str) -> RunResult {
    let repo = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(repo.join(".bee/bin/hooks").join(hook_file));
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_rust_hook(name: &str, cwd: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg(name);
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

/// Same as [`run_rust_hook`], with the rust-port-18 test-only crash seam
/// armed for `name` — proves this cell's own `run_fail_open` wrapping (the
/// same one write-guard/model-guard already established) engages for these
/// two new hooks too.
fn run_rust_hook_with_crash_seam(name: &str, cwd: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg(name);
    cmd.current_dir(cwd);
    cmd.env("BEE_QUEEN_BEE_CRASH_SEAM", name);
    run(cmd, stdin)
}

fn assert_process_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(node.status, rust.status, "{label}: exit code diverged (node={} rust={}) node_stderr={:?} rust_stderr={:?}", node.status, rust.status, node.stderr, rust.stderr);
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "{label}: stdout diverged (node={:?} rust={:?})", node.stdout, rust.stdout);
}

fn read_jsonl_normalized(path: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut v: Value = serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}"));
            if let Value::Object(ref mut m) = v {
                if m.contains_key("ts") {
                    m.insert("ts".into(), Value::String("<ts>".into()));
                }
            }
            v
        })
        .collect()
}

fn read_text_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Redacts a JSON field's STRING VALUE directly in the RAW file text — never
/// via a parsed `serde_json::Value`. This is the load-bearing distinction
/// (rework finding 4): this workspace enables serde_json's `preserve_order`
/// feature, so `Value`'s `Map` is backed by an `IndexMap` — and
/// `IndexMap`'s `PartialEq` compares as a SET, deliberately IGNORING
/// element order. A comparison built on parsed `Value`s (this cell's
/// original technique) can therefore never see a key-order divergence in a
/// written store file, exactly the class of bug D3 (byte compatibility)
/// exists to catch — not hypothetical: rust-port-15's own rework found two
/// real key-order breaks its parsed-`Value` assertions stayed green on.
/// Every occurrence of `"field": "<value>"` is replaced (loop, not just the
/// first), preserving every surrounding byte — key order, indentation,
/// trailing commas — untouched.
fn redact_text_field(text: &str, field: &str) -> String {
    let needle = format!("\"{field}\": \"");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(idx) = rest.find(needle.as_str()) {
        let (before, after_needle_start) = rest.split_at(idx);
        out.push_str(before);
        out.push_str(&needle);
        let after_needle = &after_needle_start[needle.len()..];
        let end = after_needle.find('"').unwrap_or(after_needle.len());
        out.push_str("<redacted>");
        rest = &after_needle[end..];
    }
    out.push_str(rest);
    out
}

/// Byte-level equality between two on-disk JSON files, after redacting the
/// named wall-clock fields directly in the raw text (see
/// [`redact_text_field`]). Sensitive to key order and formatting — the fix
/// for the parsed-`Value` blind spot this rework closes.
fn assert_files_byte_equal_redacted(label: &str, a: &Path, b: &Path, redact_fields: &[&str]) {
    let mut ta = read_text_file(a);
    let mut tb = read_text_file(b);
    for field in redact_fields {
        ta = redact_text_field(&ta, field);
        tb = redact_text_field(&tb, field);
    }
    assert_eq!(ta, tb, "{label}: byte-level (key-order-sensitive) file comparison diverged");
}

// ---------------------------------------------------------------------------
// Fixture class: rig self-check + negative control
// ---------------------------------------------------------------------------

#[test]
fn rig_self_check_seeded_files_match_repo_sha256() {
    let seeded = seed_root(enabling_config());
    assert!(is_seeded_valid(&seeded.root), "a freshly seeded root must verify as valid");
}

#[test]
fn negative_control_unseeded_root_detected_as_invalid_and_both_hooks_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cwd = dir.path();
    assert!(!is_seeded_valid(cwd), "an unseeded root must never verify as a valid rig setup");

    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;
    for hook_file in ["bee-chain-nudge.mjs", "bee-state-sync.mjs"] {
        let name = hook_file.trim_start_matches("bee-").trim_end_matches(".mjs");
        let node = run_live_node_hook(hook_file, cwd, stdin);
        let rust = run_rust_hook(name, cwd, stdin);
        assert_process_conformant(&format!("negative-control/{name}"), &node, &rust);
        assert_eq!(node.status, 0, "no discoverable root must still be exit 0 (fail-open)");
        assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
        assert!(!cwd.join(".bee").exists(), "no root discoverable => no .bee dir should ever be created");
    }
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge disabled-hook + allow-with-twin-deny pairing
// ---------------------------------------------------------------------------

#[test]
fn chain_nudge_disabled_hook_silences_both_and_enabled_twin_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;

    let (deny_node, deny_rust) = seed_pair(json!({ "chain-nudge": false, "state-sync": true }));
    write_state_json(&deny_node.root, &json!({ "phase": "swarming" }));
    write_state_json(&deny_rust.root, &json!({ "phase": "swarming" }));
    let node_off = run_seeded_node_hook(&deny_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_off = run_rust_hook("chain-nudge", &deny_rust.root, stdin);
    assert_process_conformant("disabled-hook/chain-nudge", &node_off, &rust_off);
    assert!(node_off.stdout.trim().is_empty(), "disabled hook must stay silent even though phase=swarming would otherwise nudge");

    // Twin: SAME shape (phase=swarming, same payload), only the hook toggle
    // flipped true — proves the silence above was a genuine gate.
    let (allow_node, allow_rust) = seed_pair(enabling_config());
    write_state_json(&allow_node.root, &json!({ "phase": "swarming" }));
    write_state_json(&allow_rust.root, &json!({ "phase": "swarming" }));
    let node_on = run_seeded_node_hook(&allow_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_on = run_rust_hook("chain-nudge", &allow_rust.root, stdin);
    assert_process_conformant("enabled-hook/chain-nudge", &node_on, &rust_on);
    assert!(!node_on.stdout.trim().is_empty(), "enabled hook + phase=swarming must nudge");
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge silence (idle, unregistered) <-> worker-nudge
// twin (phase=swarming with an agent name)
// ---------------------------------------------------------------------------

#[test]
fn chain_nudge_silent_when_idle_and_worker_nudge_twin_when_swarming_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;

    // Silent: default (idle) state, unregistered agent name.
    let (silent_node, silent_rust) = seed_pair(enabling_config());
    let node_silent = run_seeded_node_hook(&silent_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_silent = run_rust_hook("chain-nudge", &silent_rust.root, stdin);
    assert_process_conformant("chain-nudge/silent-idle", &node_silent, &rust_silent);
    assert!(node_silent.stdout.trim().is_empty(), "idle phase + unregistered agent must stay silent");

    // Twin: SAME payload, state.json flipped to phase=swarming.
    let (nudge_node, nudge_rust) = seed_pair(enabling_config());
    write_state_json(&nudge_node.root, &json!({ "phase": "swarming" }));
    write_state_json(&nudge_rust.root, &json!({ "phase": "swarming" }));
    let node_nudge = run_seeded_node_hook(&nudge_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_nudge = run_rust_hook("chain-nudge", &nudge_rust.root, stdin);
    assert_process_conformant("chain-nudge/worker-nudge-twin(swarming)", &node_nudge, &rust_nudge);
    assert!(node_nudge.stdout.contains("Worker \\\"Kevin\\\" returned"), "expected the worker-nudge phrase in the advisory JSON, got {:?}", node_nudge.stdout);
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge reviewing-phase message
// ---------------------------------------------------------------------------

#[test]
fn chain_nudge_reviewing_phase_message_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop"}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());
    write_state_json(&node_seed.root, &json!({ "phase": "reviewing" }));
    write_state_json(&rust_seed.root, &json!({ "phase": "reviewing" }));
    let node = run_seeded_node_hook(&node_seed.root, "bee-chain-nudge.mjs", stdin);
    let rust = run_rust_hook("chain-nudge", &rust_seed.root, stdin);
    assert_process_conformant("chain-nudge/reviewing", &node, &rust);
    assert!(node.stdout.contains("present Gate 4"), "expected the reviewing-phase phrase, got {:?}", node.stdout);
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge scribing-debt suffix (present <-> absent twin)
// ---------------------------------------------------------------------------

#[test]
fn chain_nudge_scribing_debt_suffix_present_and_absent_twin_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;

    // Present: a capped behavior_change cell in the active feature, capped
    // strictly after the (zero, no ledger) threshold.
    let (debt_node, debt_rust) = seed_pair(enabling_config());
    for root in [&debt_node.root, &debt_rust.root] {
        write_state_json(root, &json!({ "phase": "swarming", "feature": "demo-feat" }));
        seed_cell(root, "demo-1", "demo-feat", "capped", true, Some("2026-01-01T00:00:00.000Z"));
    }
    let node_debt = run_seeded_node_hook(&debt_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_debt = run_rust_hook("chain-nudge", &debt_rust.root, stdin);
    assert_process_conformant("chain-nudge/scribing-debt-present", &node_debt, &rust_debt);
    assert!(node_debt.stdout.contains("Scribing debt"), "expected a scribing-debt suffix, got {:?}", node_debt.stdout);
    assert!(node_debt.stdout.contains("demo-1"), "expected the capped cell id named in the suffix, got {:?}", node_debt.stdout);

    // Twin: SAME shape, the cell is NOT behavior_change — no debt.
    let (nodebt_node, nodebt_rust) = seed_pair(enabling_config());
    for root in [&nodebt_node.root, &nodebt_rust.root] {
        write_state_json(root, &json!({ "phase": "swarming", "feature": "demo-feat" }));
        seed_cell(root, "demo-1", "demo-feat", "capped", false, Some("2026-01-01T00:00:00.000Z"));
    }
    let node_nodebt = run_seeded_node_hook(&nodebt_node.root, "bee-chain-nudge.mjs", stdin);
    let rust_nodebt = run_rust_hook("chain-nudge", &nodebt_rust.root, stdin);
    assert_process_conformant("chain-nudge/scribing-debt-absent-twin", &node_nodebt, &rust_nodebt);
    assert!(!node_nodebt.stdout.contains("Scribing debt"), "a non-behavior_change cell must never trigger the suffix, got {:?}", node_nodebt.stdout);
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge registered-worker trigger — every rung of the
// worker_name() fallback ladder (rework rust-port-21, finding 1: the real
// gap. Every fixture up to this point triggered the nudge via
// phase=swarming only; state.workers was never seeded, so
// is_registered_worker and the whole ported worker_name() ladder
// (chain_nudge.rs:62-77, mirroring bee-chain-nudge.mjs:36-47) were never
// diffed against the frozen oracle).
// ---------------------------------------------------------------------------

/// Table-driven (test-economy D3: >=3 same-shape cases belong in one row
/// set, not copy-pasted near-duplicates) — one row per rung of
/// `workerName(entry)`'s fallback ladder: a bare string entry, then an
/// object entry matched via `nickname`, `name`, `agent`, and finally
/// `worker` (each row's object carries ONLY the field that rung reads, so
/// a regression that skips a rung — e.g. checking `name` before
/// `nickname` — cannot accidentally still match via an earlier field).
/// `phase` is deliberately "idle" (never "swarming") on every row: the
/// nudge here fires ONLY because `is_registered_worker` is true, proving
/// the ladder is actually consulted rather than piggy-backing on the
/// phase=swarming trigger every earlier fixture used.
#[test]
fn chain_nudge_worker_name_ladder_every_rung_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;
    let rows: &[(&str, Value)] = &[
        ("string-entry", json!(["Kevin"])),
        ("nickname-rung", json!([{ "nickname": "Kevin" }])),
        ("name-rung", json!([{ "name": "Kevin" }])),
        ("agent-rung", json!([{ "agent": "Kevin" }])),
        ("worker-rung", json!([{ "worker": "Kevin" }])),
    ];

    for (label, workers) in rows {
        let (node_seed, rust_seed) = seed_pair(enabling_config());
        for root in [&node_seed.root, &rust_seed.root] {
            write_state_json(root, &json!({ "phase": "idle", "workers": workers }));
        }
        let node = run_seeded_node_hook(&node_seed.root, "bee-chain-nudge.mjs", stdin);
        let rust = run_rust_hook("chain-nudge", &rust_seed.root, stdin);
        assert_process_conformant(&format!("chain-nudge/worker-ladder/{label}"), &node, &rust);
        assert!(
            node.stdout.contains("Worker \\\"Kevin\\\" returned"),
            "{label}: expected the worker-nudge phrase (is_registered_worker must fire on this rung alone, phase is idle), got {:?}",
            node.stdout
        );
    }
}

/// Non-triviality twin for the ladder above: an entry shape that matches
/// NONE of the ladder's fields (a bare number, and an object with an
/// unrecognized key only) must stay silent — proving the ladder rows
/// above are a genuine match, not a fixture that nudges regardless of
/// `state.workers`'s content.
#[test]
fn chain_nudge_worker_name_ladder_no_match_stays_silent_twin_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());
    for root in [&node_seed.root, &rust_seed.root] {
        write_state_json(root, &json!({ "phase": "idle", "workers": [42, { "unrelated_field": "Kevin" }] }));
    }
    let node = run_seeded_node_hook(&node_seed.root, "bee-chain-nudge.mjs", stdin);
    let rust = run_rust_hook("chain-nudge", &rust_seed.root, stdin);
    assert_process_conformant("chain-nudge/worker-ladder/no-match-twin", &node, &rust);
    assert!(node.stdout.trim().is_empty(), "no ladder rung matches — must stay silent, got {:?}", node.stdout);
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge session_id exercises state::resolve_pipeline's
// lane branch (rework rust-port-21, finding 1's second half: no prior
// fixture ever passed session_id, so resolve_pipeline always short-circuited
// on the `None` branch without ever calling claims::read_session).
// ---------------------------------------------------------------------------

/// A session bound to a lane whose OWN record carries phase="swarming",
/// while the DEFAULT `state.json` stays phase="idle" with no workers at
/// all. Only resolve_pipeline's real lane lookup can produce a nudge here
/// — a regression that used `state.phase` directly (ignoring the pipeline
/// result, exactly the fallback path chain_nudge.rs's `Err(_) =>
/// bee_state.phase` arm exists for) would see "idle" and stay silent,
/// diverging from the real mjs oracle.
#[test]
fn chain_nudge_session_id_exercises_resolve_pipeline_lane_branch_matches_oracle() {
    let stdin = r#"{"hook_event_name":"SubagentStop","session_id":"sess-lane"}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());
    for root in [&node_seed.root, &rust_seed.root] {
        write_state_json(root, &json!({ "phase": "idle" })); // the DEFAULT record — deliberately non-swarming
        seed_session(root, "sess-lane", 0);
        bind_session_lane(root, "sess-lane", "demo-lane-feat");
        write_lane_json(
            root,
            "demo-lane-feat",
            &json!({
                "schema_version": "1.0",
                "feature": "demo-lane-feat",
                "phase": "swarming",
                "mode": null,
                "approved_gates": { "context": false, "shape": false, "execution": false, "review": false },
                "summary": "",
                "next_action": "",
            }),
        );
    }
    let node = run_seeded_node_hook(&node_seed.root, "bee-chain-nudge.mjs", stdin);
    let rust = run_rust_hook("chain-nudge", &rust_seed.root, stdin);
    assert_process_conformant("chain-nudge/resolve-pipeline-lane-branch", &node, &rust);
    assert!(
        !node.stdout.trim().is_empty(),
        "the LANE record's phase=swarming must drive the nudge even though the default record is idle, got empty stdout"
    );
    assert!(node.stdout.contains("[STATUS]"), "expected the worker-nudge phrase, got {:?}", node.stdout);
}

// ---------------------------------------------------------------------------
// Fixture class: chain-nudge crash fail-open
// ---------------------------------------------------------------------------

/// The first crash line matching `must_contain` in `error` — a helper, not
/// the load-bearing assertion; see the callers below for the field-level
/// hook-name comparison rework finding 1 requires.
fn find_crash_line<'a>(lines: &'a [Value], must_contain: &str) -> &'a Value {
    lines
        .iter()
        .find(|l| l.get("error").and_then(Value::as_str).map(|s| s.contains(must_contain)).unwrap_or(false))
        .unwrap_or_else(|| panic!("no crash line containing {must_contain:?} found in {lines:?}"))
}

#[test]
fn chain_nudge_crash_fail_open_both_runtimes_log_the_correct_hook_name() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;

    let node_seeded = seed_crash_root(enabling_config());
    write_state_json(&node_seeded.root, &json!({ "phase": "swarming" })); // a healthy run WOULD nudge here
    let node = run_seeded_node_hook(&node_seeded.root, "bee-chain-nudge.mjs", stdin);
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must still exit 0 — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty(), "a crash must never leak a nudge to stdout");
    let node_lines = read_jsonl_normalized(&node_seeded.root.join(".bee/logs/hooks.jsonl"));
    let node_crash = find_crash_line(&node_lines, "rig-injected-fault");

    let rust_seeded = seed_root(enabling_config());
    write_state_json(&rust_seeded.root, &json!({ "phase": "swarming" }));
    let rust = run_rust_hook_with_crash_seam("chain-nudge", &rust_seeded.root, stdin);
    assert_eq!(rust.status, 0, "fail-open contract: a panic must still exit 0 — stderr={:?}", rust.stderr);
    assert!(rust.stdout.trim().is_empty(), "a crash must never leak a nudge to stdout");
    let rust_lines = read_jsonl_normalized(&rust_seeded.root.join(".bee/logs/hooks.jsonl"));
    let rust_crash = find_crash_line(&rust_lines, "rust-port-18 test-only crash seam armed");

    // THE cross-runtime comparison rework finding 1 requires: the `hook`
    // field must name the ACTUAL crashing hook, never the shared
    // run_fail_open wrapper's own file-level HOOK_NAME constant
    // ("write-guard") that every caller used to silently inherit.
    assert_eq!(node_crash.get("hook"), Some(&json!("chain-nudge")), "node oracle crash line hook field, got {node_crash:?}");
    assert_eq!(rust_crash.get("hook"), Some(&json!("chain-nudge")), "rust port crash line hook field, got {rust_crash:?}");
    assert_eq!(
        node_crash.get("hook"),
        rust_crash.get("hook"),
        "cross-runtime: crash line hook field diverged (node={node_crash:?} rust={rust_crash:?})"
    );
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync disabled-hook (no side effects at all)
// ---------------------------------------------------------------------------

#[test]
fn state_sync_disabled_hook_makes_no_writes_on_either_runtime() {
    let stdin = r#"{"session_id":"sess-disabled"}"#;
    let (node_seed, rust_seed) = seed_pair(json!({ "chain-nudge": true, "state-sync": false }));
    let mut session_files = Vec::new();
    for root in [&node_seed.root, &rust_seed.root] {
        session_files.push(seed_session(root, "sess-disabled", 120)); // stale enough to renew if NOT gated off
    }
    // Captured as RAW TEXT, never a parsed `Value` (rework rust-port-21,
    // finding 2): this is a same-runtime before/after no-op check, so the
    // strongest and most honest claim available is "the file did not
    // change AT ALL" — a byte comparison, not a structural one that would
    // stay green through a pure key-reordering rewrite.
    let session_before: Vec<String> = session_files.iter().map(|f| read_text_file(f)).collect();

    let node = run_seeded_node_hook(&node_seed.root, "bee-state-sync.mjs", stdin);
    let rust = run_rust_hook("state-sync", &rust_seed.root, stdin);
    assert_process_conformant("state-sync/disabled", &node, &rust);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty(), "state-sync is always silent");
    assert!(!node_seed.root.join(".bee/state.json").exists(), "disabled hook must never write state.json (node)");
    assert!(!rust_seed.root.join(".bee/state.json").exists(), "disabled hook must never write state.json (rust)");

    for (i, label) in ["node", "rust"].iter().enumerate() {
        let session_after = read_text_file(&session_files[i]);
        assert_eq!(
            session_before[i], session_after,
            "{label}: byte-for-byte comparison — a disabled hook must never renew the heartbeat, even for a stale session"
        );
    }
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync state.json rebuild — the no-workflow-records
// overrides-only fall-through, AND (separately) the AUTHORITATIVE
// idle-bootstrap branch this cell exists to route through (rework finding
// 2: the first cut of this rig only ever exercised the fall-through, never
// diffed the authoritative rebuild against the mjs oracle, and the
// fall-through test's old name overclaimed "idle-bootstrap").
// ---------------------------------------------------------------------------

/// Zero workflow records anywhere: `rebuildStateProjection`/
/// `rebuild_state_projection` both take the `apply_overrides_only`
/// fall-through (state_projection.rs's own doc comment on that branch) —
/// only `cells`/`last_activity` are ever touched; `phase`/`feature`/`mode`/
/// `approved_gates`/`summary`/`next_action` are NOT rebuilt from anything
/// (there is nothing live to adopt). This is deliberately NOT named
/// "idle-bootstrap" (the cell's rework finding 2): idle-bootstrap is the
/// AUTHORITATIVE branch proved by the test immediately below, which is the
/// one that actually adopts a workflow record.
#[test]
fn state_sync_no_workflow_records_only_refreshes_overrides_matches_oracle() {
    let stdin = r#"{}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());
    for root in [&node_seed.root, &rust_seed.root] {
        seed_cell(root, "open-1", "demo-feat", "open", false, None);
        seed_cell(root, "claimed-1", "demo-feat", "claimed", false, None);
        seed_cell(root, "capped-1", "demo-feat", "capped", false, None);
        seed_cell(root, "blocked-1", "demo-feat", "blocked", false, None);
    }
    let node = run_seeded_node_hook(&node_seed.root, "bee-state-sync.mjs", stdin);
    let rust = run_rust_hook("state-sync", &rust_seed.root, stdin);
    assert_process_conformant("state-sync/no-workflow-records", &node, &rust);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());

    let node_state_file = node_seed.root.join(".bee/state.json");
    let rust_state_file = rust_seed.root.join(".bee/state.json");
    let state_node = read_json_file(&node_state_file).expect("node state.json written");
    let state_rust = read_json_file(&rust_state_file).expect("rust state.json written");
    assert_eq!(state_node["cells"], json!({ "open": 1, "claimed": 1, "capped": 1, "blocked": 1 }), "node cell counts");
    assert_eq!(state_rust["cells"], json!({ "open": 1, "claimed": 1, "capped": 1, "blocked": 1 }), "rust cell counts");
    assert!(state_node.get("last_activity").and_then(Value::as_str).is_some(), "node must stamp last_activity");
    assert!(state_rust.get("last_activity").and_then(Value::as_str).is_some(), "rust must stamp last_activity");
    // The phase/feature/mode fields must be UNTOUCHED (still the fresh
    // default) — proof this fixture really lands in the fall-through, not
    // the authoritative branch it used to be misnamed after.
    assert_eq!(state_node.get("phase"), Some(&json!("idle")), "node: fall-through must never touch phase");
    assert_eq!(state_rust.get("phase"), Some(&json!("idle")), "rust: fall-through must never touch phase");
    assert!(state_node.get("feature").is_none() || state_node["feature"].is_null(), "node: fall-through must never invent a feature");
    assert!(state_rust.get("feature").is_none() || state_rust["feature"].is_null(), "rust: fall-through must never invent a feature");

    assert_files_byte_equal_redacted("state-sync/no-workflow-records state.json", &node_state_file, &rust_state_file, &["last_activity"]);
}

/// A LIVE (`status: "active"`) workflow record exists and the default
/// record is idle with no feature — `rebuildStateProjection`'s idle-
/// bootstrap branch fires and adopts it: `phase`/`feature`/`mode`/
/// `approved_gates`/`summary`/`next_action` are all rebuilt FROM the
/// workflow record, the one behavior this cell's state-sync hook exists to
/// route through on every ordinary run. Proved against the real mjs oracle
/// (byte-level, key-order sensitive) — never only the do-nothing
/// fall-through the test above covers.
#[test]
fn state_sync_authoritative_idle_bootstrap_adopts_active_workflow_matches_oracle() {
    let stdin = r#"{}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());
    for root in [&node_seed.root, &rust_seed.root] {
        seed_workflow(root, "demo-feat", "executing", "standard", "doing the thing", "ship it");
    }
    let node = run_seeded_node_hook(&node_seed.root, "bee-state-sync.mjs", stdin);
    let rust = run_rust_hook("state-sync", &rust_seed.root, stdin);
    assert_process_conformant("state-sync/authoritative-idle-bootstrap", &node, &rust);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());

    let node_state_file = node_seed.root.join(".bee/state.json");
    let rust_state_file = rust_seed.root.join(".bee/state.json");
    let state_node = read_json_file(&node_state_file).expect("node state.json written");
    let state_rust = read_json_file(&rust_state_file).expect("rust state.json written");

    // THE proof this is the authoritative branch, not the fall-through:
    // phase/feature/mode/summary/next_action must all have been ADOPTED
    // from the seeded workflow record, on both runtimes.
    for (label, state) in [("node", &state_node), ("rust", &state_rust)] {
        assert_eq!(state.get("phase"), Some(&json!("executing")), "{label}: phase must be adopted from the active workflow");
        assert_eq!(state.get("feature"), Some(&json!("demo-feat")), "{label}: feature must be adopted from the active workflow");
        assert_eq!(state.get("mode"), Some(&json!("standard")), "{label}: mode must be adopted from the active workflow");
        assert_eq!(state.get("summary"), Some(&json!("doing the thing")), "{label}: summary must be adopted from the active workflow");
        assert_eq!(state.get("next_action"), Some(&json!("ship it")), "{label}: next_action must be adopted from the active workflow");
        assert_eq!(
            state.get("approved_gates"),
            Some(&json!({ "context": false, "shape": false, "execution": false, "review": false })),
            "{label}: approved_gates must be derived from the workflow's default (unapproved) gates"
        );
    }

    assert_files_byte_equal_redacted(
        "state-sync/authoritative-idle-bootstrap state.json",
        &node_state_file,
        &rust_state_file,
        &["last_activity"],
    );
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync heartbeat + claim + lease + hold renewal
// (session stale beyond the 60s touch throttle)
// ---------------------------------------------------------------------------

#[test]
fn state_sync_heartbeat_claim_lease_hold_renewal_matches_oracle() {
    let stdin = r#"{"session_id":"sess-stale"}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());

    let mut session_files = Vec::new();
    let mut claim_files = Vec::new();
    let mut lease_files = Vec::new();
    let mut hold_files = Vec::new();
    for root in [&node_seed.root, &rust_seed.root] {
        session_files.push(seed_session(root, "sess-stale", 120)); // stale: > 60s throttle
        claim_files.push(seed_claim(root, "cellA", "sess-stale", 30));
        lease_files.push(seed_lease(root, "sess-stale", "src/fixture.rs", 60));
        hold_files.push(seed_hold(root, "main", "sess-stale", "src/fixture.rs", 3600));
    }

    let session_before: Vec<Value> = session_files.iter().map(|f| read_json_file(f).expect("session fixture readable")).collect();
    let claim_before: Vec<Value> = claim_files.iter().map(|f| read_json_file(f).expect("claim fixture readable")).collect();
    let lease_before: Vec<Value> = lease_files.iter().map(|f| read_json_file(f).expect("lease fixture readable")).collect();
    let hold_before: Vec<Value> = hold_files
        .iter()
        .map(|f| first_hold_row(&read_json_file(f).expect("hold fixture readable")))
        .collect();

    let node = run_seeded_node_hook(&node_seed.root, "bee-state-sync.mjs", stdin);
    let rust = run_rust_hook("state-sync", &rust_seed.root, stdin);
    assert_process_conformant("state-sync/renewal", &node, &rust);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());

    for (i, label) in ["node", "rust"].iter().enumerate() {
        let session_after = read_json_file(&session_files[i]).unwrap();
        let claim_after = read_json_file(&claim_files[i]).unwrap();
        let lease_after = read_json_file(&lease_files[i]).unwrap();
        let hold_after = first_hold_row(&read_json_file(&hold_files[i]).unwrap());

        let hb_before = session_before[i]["last_heartbeat"].as_str().unwrap();
        let hb_after = session_after["last_heartbeat"].as_str().unwrap();
        assert!(hb_after > hb_before, "{label}: last_heartbeat must move forward (before={hb_before} after={hb_after})");

        let claimed_before = claim_before[i]["claimed_at"].as_str().unwrap();
        let claimed_after = claim_after["claimed_at"].as_str().unwrap();
        assert!(claimed_after > claimed_before, "{label}: claimed_at must move forward (before={claimed_before} after={claimed_after})");

        let expires_before = lease_before[i]["expires_at"].as_str().unwrap();
        let expires_after = lease_after["expires_at"].as_str().unwrap();
        assert!(expires_after > expires_before, "{label}: lease expires_at must move forward (before={expires_before} after={expires_after})");

        let mirrored_before = hold_before[i]["mirrored_at"].as_str().unwrap();
        let mirrored_after = hold_after["mirrored_at"].as_str().unwrap();
        assert!(mirrored_after > mirrored_before, "{label}: hold mirrored_at must move forward (before={mirrored_before} after={mirrored_after})");
    }

    // Structural parity across runtimes, BYTE-LEVEL (key-order-sensitive —
    // rework finding 4) with volatile stamps redacted. All four renewed
    // records get this treatment, including the hold ledger row (rework
    // finding 3: previously the only one of the four with no cross-runtime
    // comparison at all — only prose guarded its `Hold` typed round-trip
    // against the exact class of bug the session-write path had).
    assert_files_byte_equal_redacted("state-sync/renewal session.json", &session_files[0], &session_files[1], &["last_heartbeat", "started_at"]);
    assert_files_byte_equal_redacted("state-sync/renewal claim.json", &claim_files[0], &claim_files[1], &["claimed_at"]);
    assert_files_byte_equal_redacted("state-sync/renewal lease file", &lease_files[0], &lease_files[1], &["expires_at", "acquired_at"]);
    assert_files_byte_equal_redacted("state-sync/renewal cross-worktree-holds.json", &hold_files[0], &hold_files[1], &["mirrored_at"]);
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync throttled no-op (heartbeat still fresh)
// ---------------------------------------------------------------------------

#[test]
fn state_sync_throttled_no_op_when_heartbeat_fresh_matches_oracle() {
    let stdin = r#"{"session_id":"sess-fresh"}"#;
    let (node_seed, rust_seed) = seed_pair(enabling_config());

    let mut session_files = Vec::new();
    let mut claim_files = Vec::new();
    for root in [&node_seed.root, &rust_seed.root] {
        session_files.push(seed_session(root, "sess-fresh", 5)); // fresh: well under the 60s throttle
        claim_files.push(seed_claim(root, "cellA", "sess-fresh", 5));
    }
    // RAW TEXT, not parsed `Value` — same reasoning as the disabled-hook
    // test above (rework rust-port-21, finding 2): a same-runtime no-op
    // check should assert byte-for-byte identity, not structural equality
    // that a key-reordering rewrite would still pass.
    let session_before: Vec<String> = session_files.iter().map(|f| read_text_file(f)).collect();
    let claim_before: Vec<String> = claim_files.iter().map(|f| read_text_file(f)).collect();

    let node = run_seeded_node_hook(&node_seed.root, "bee-state-sync.mjs", stdin);
    let rust = run_rust_hook("state-sync", &rust_seed.root, stdin);
    assert_process_conformant("state-sync/throttled-no-op", &node, &rust);

    for (i, label) in ["node", "rust"].iter().enumerate() {
        let session_after = read_text_file(&session_files[i]);
        let claim_after = read_text_file(&claim_files[i]);
        assert_eq!(session_before[i], session_after, "{label}: byte-for-byte comparison — a throttled touch must never rewrite the session file");
        assert_eq!(claim_before[i], claim_after, "{label}: byte-for-byte comparison — a throttled touch must never renew a claim either");
    }
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync LOCK_BUSY skips the state-projection rebuild
// silently on both runtimes (session-side renewal is a SEPARATE lock and
// still proceeds)
// ---------------------------------------------------------------------------

/// Holds the named store lock (via bee-core's own D9 lock primitive, the
/// SAME lock file a real `lockLib.withStoreLock(root, "state", ...)` call
/// would contend) on a background thread until told to release — proves
/// LOCK_BUSY from the OUTSIDE, exactly like a real concurrent holder would,
/// never a stub or a mocked error.
struct LockHolder {
    release_tx: mpsc::Sender<()>,
    handle: std::thread::JoinHandle<()>,
}

fn hold_lock_in_background(root: &Path, name: &'static str) -> LockHolder {
    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let root = root.to_path_buf();
    let handle = std::thread::spawn(move || {
        let _ = bee_core::lock::with_store_lock(&root, name, bee_core::lock::LockOptions::default(), move || {
            ready_tx.send(()).ok();
            release_rx.recv_timeout(Duration::from_secs(10)).ok();
        });
    });
    ready_rx.recv_timeout(Duration::from_secs(5)).expect("holder thread never acquired the lock");
    LockHolder { release_tx, handle }
}

impl LockHolder {
    fn release(self) {
        self.release_tx.send(()).ok();
        self.handle.join().expect("holder thread panicked");
    }
}

#[test]
fn state_sync_lock_busy_skips_state_rebuild_silently_on_both_runtimes() {
    let stdin = r#"{}"#;
    let baseline = json!({
        "schema_version": "1.0",
        "phase": "idle",
        "cells": { "open": 9, "claimed": 9, "capped": 9, "blocked": 9 },
        "last_activity": "1999-01-01T00:00:00.000Z",
    });

    let node_seeded = seed_root(enabling_config());
    write_state_json(&node_seeded.root, &baseline);
    // Re-read the file we JUST wrote (rather than re-serializing `baseline`
    // ourselves) so the "before" text is the literal bytes on disk — the
    // rework rust-port-21 finding-2 fix: this used to compare a parsed
    // `Value` against `baseline` and claim "byte-identical" while being
    // key-order-blind (an `IndexMap`'s `PartialEq` ignores order under this
    // workspace's `preserve_order` feature). A byte comparison is now the
    // literal truth of the message.
    let node_before = read_text_file(&node_seeded.root.join(".bee/state.json"));
    let node_holder = hold_lock_in_background(&node_seeded.root, "state");
    let node = run_seeded_node_hook(&node_seeded.root, "bee-state-sync.mjs", stdin);
    assert_eq!(node.status, 0, "node: LOCK_BUSY must still exit 0 (fail-open) — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty());
    let node_after = read_text_file(&node_seeded.root.join(".bee/state.json"));
    assert_eq!(node_before, node_after, "node: byte-for-byte comparison — a busy state lock must leave state.json completely unchanged, no partial write");
    node_holder.release();

    let rust_seeded = seed_root(enabling_config());
    write_state_json(&rust_seeded.root, &baseline);
    let rust_before = read_text_file(&rust_seeded.root.join(".bee/state.json"));
    let rust_holder = hold_lock_in_background(&rust_seeded.root, "state");
    let rust = run_rust_hook("state-sync", &rust_seeded.root, stdin);
    assert_eq!(rust.status, 0, "rust: LOCK_BUSY must still exit 0 (fail-open) — stderr={:?}", rust.stderr);
    assert!(rust.stdout.trim().is_empty());
    let rust_after = read_text_file(&rust_seeded.root.join(".bee/state.json"));
    assert_eq!(rust_before, rust_after, "rust: byte-for-byte comparison — a busy state lock must leave state.json completely unchanged, no partial write");
    rust_holder.release();
}

// ---------------------------------------------------------------------------
// Fixture class: state-sync crash fail-open
// ---------------------------------------------------------------------------

#[test]
fn state_sync_crash_fail_open_both_runtimes_log_the_correct_hook_name() {
    let stdin = r#"{}"#;

    let node_seeded = seed_crash_root(enabling_config());
    let node = run_seeded_node_hook(&node_seeded.root, "bee-state-sync.mjs", stdin);
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must still exit 0 — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty());
    assert!(!node_seeded.root.join(".bee/state.json").exists(), "a crashed run must never leave a partial state.json");
    let node_lines = read_jsonl_normalized(&node_seeded.root.join(".bee/logs/hooks.jsonl"));
    let node_crash = find_crash_line(&node_lines, "rig-injected-fault");

    let rust_seeded = seed_root(enabling_config());
    let rust = run_rust_hook_with_crash_seam("state-sync", &rust_seeded.root, stdin);
    assert_eq!(rust.status, 0, "fail-open contract: a panic must still exit 0 — stderr={:?}", rust.stderr);
    assert!(rust.stdout.trim().is_empty());
    assert!(!rust_seeded.root.join(".bee/state.json").exists(), "a crashed run must never leave a partial state.json");
    let rust_lines = read_jsonl_normalized(&rust_seeded.root.join(".bee/logs/hooks.jsonl"));
    let rust_crash = find_crash_line(&rust_lines, "rust-port-18 test-only crash seam armed");

    assert_eq!(node_crash.get("hook"), Some(&json!("state-sync")), "node oracle crash line hook field, got {node_crash:?}");
    assert_eq!(rust_crash.get("hook"), Some(&json!("state-sync")), "rust port crash line hook field, got {rust_crash:?}");
    assert_eq!(
        node_crash.get("hook"),
        rust_crash.get("hook"),
        "cross-runtime: crash line hook field diverged (node={node_crash:?} rust={rust_crash:?})"
    );
}

// ---------------------------------------------------------------------------
// Red-first meta-proof: the differ itself must have teeth.
// ---------------------------------------------------------------------------

#[test]
fn rig_diff_detector_flags_a_seeded_mutation_as_divergent() {
    let stdin = r#"{"hook_event_name":"SubagentStop","agent_name":"Kevin"}"#;
    let node_seed = seed_root(enabling_config());
    let rust_seed = seed_root(enabling_config());
    write_state_json(&node_seed.root, &json!({ "phase": "swarming" }));
    write_state_json(&rust_seed.root, &json!({ "phase": "reviewing" })); // deliberately different

    let node = run_seeded_node_hook(&node_seed.root, "bee-chain-nudge.mjs", stdin);
    let rust = run_rust_hook("chain-nudge", &rust_seed.root, stdin);

    let result = std::panic::catch_unwind(|| assert_process_conformant("deliberately-mutated", &node, &rust));
    assert!(result.is_err(), "the differ failed to flag a deliberately seeded divergence — the rig has no teeth");
}

/// Rework finding 4's own meta-proof: a parsed-`serde_json::Value`
/// comparison (this cell's ORIGINAL technique for state.json/session.json/
/// claim.json/lease-file/hold-ledger parity) is blind to key-order
/// divergence under this workspace's `preserve_order` feature (`Value`'s
/// `Map` is an `IndexMap`, whose `PartialEq` compares as a set). This test
/// proves BOTH halves: (1) the blind spot is real — two texts with the
/// same keys/values in different order parse to `Value`-equal, and (2)
/// [`assert_files_byte_equal_redacted`]'s byte-level technique correctly
/// flags that exact same pair as divergent. Without this test, a future
/// edit reverting any of this cell's writers back to a Value-comparison
/// (or a key-reordering serialization change) could regress silently.
#[test]
fn byte_level_comparison_catches_key_order_divergence_that_value_equality_misses() {
    let same_keys_reordered = json!({ "alpha": "1", "beta": "2" });
    let a_text = "{\n  \"alpha\": \"1\",\n  \"beta\": \"2\"\n}\n";
    let b_text = "{\n  \"beta\": \"2\",\n  \"alpha\": \"1\"\n}\n";

    // (1) The blind spot: parsed-Value equality really does ignore order.
    let a_value: Value = serde_json::from_str(a_text).unwrap();
    let b_value: Value = serde_json::from_str(b_text).unwrap();
    assert_eq!(a_value, same_keys_reordered, "sanity: a_text parses to the expected object");
    assert_eq!(
        a_value, b_value,
        "sanity check failed: this build's serde_json Value equality is NOT order-blind — the premise for this rig's byte-level rework no longer holds, investigate before trusting the rest of this suite's file comparisons"
    );

    // (2) The fix: a byte-level (key-order-sensitive) comparison of the
    // SAME two texts must flag them as divergent.
    let dir = tempfile::tempdir().expect("tempdir");
    let a_path = dir.path().join("a.json");
    let b_path = dir.path().join("b.json");
    fs::write(&a_path, a_text).unwrap();
    fs::write(&b_path, b_text).unwrap();
    let result = std::panic::catch_unwind(|| assert_files_byte_equal_redacted("key-order meta-proof", &a_path, &b_path, &[]));
    assert!(result.is_err(), "the byte-level comparator failed to flag a key-order divergence between two Value-equal files — no teeth");
}
