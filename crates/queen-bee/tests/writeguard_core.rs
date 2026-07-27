//! writeguard_core — D7b conformance corpus for the write-guard CORE spine
//! (rust-port-9): proves `queen-bee hook write-guard` matches the REAL
//! `.bee/bin/hooks/bee-write-guard.mjs` (run through the shared, frozen
//! adapter + guards libs) for check classes (a) gate/intake, (b)
//! reservations, (c) cross-session/cross-worktree holds, and (d) worktree
//! containment — never a reimplementation guess. Bash-path fixtures are
//! rust-port-11's corpus; read-side/apply_patch/AskUserQuestion fixtures are
//! rust-port-12's.
//!
//! RIG DISCIPLINE (inherited from rust-port-7's hook_conformance rig):
//! (i) SEEDING — every node-oracle run happens inside a fresh temp root
//!     seeded with `.bee/bin/lib/`, `.bee/bin/hooks/`, `.bee/onboarding.json`,
//!     and an enabling `config.json`; the oracle executes the SEEDED copy of
//!     `bee-write-guard.mjs`, sha256-verified against the repo source.
//! (ii) NON-TRIVIALITY BOTH WAYS — every deny fixture asserts the node
//!      oracle exited 2 BEFORE diffing, and is paired with an allow twin in
//!      the same shape with exactly one field flipped.
//! (iii) NEGATIVE CONTROL — an unseeded root must be DETECTED as invalid by
//!       the rig's own verifier.
//! (iv) Descriptive per-class fixture names, listed by cargo test output.
//!
//! Every temp root comes from `tempfile::tempdir()` — never the live `.bee/`.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Repo / binary resolution
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(".bee/bin/hooks/bee-write-guard.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/hooks/bee-write-guard.mjs walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn queen_bee_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_queen-bee"))
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
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {:?} -> {target:?}: {e}", entry.path()));
        }
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The oracle surface for the write-guard core spine: every module the deny
/// decisions flow through. A doctored seeded copy of any of these is a rig
/// failure.
const SHA_CHECKED: &[&str] = &[
    "bin/hooks/adapter.mjs",
    "bin/hooks/bee-write-guard.mjs",
    "bin/hooks/tokenize-command.mjs",
    "bin/lib/guards.mjs",
    "bin/lib/state.mjs",
    "bin/lib/reservations.mjs",
    "bin/lib/lease-store.mjs",
    "bin/lib/worktree-holds.mjs",
    "bin/lib/workspace-store.mjs",
    "bin/lib/claims.mjs",
    "bin/lib/fsutil.mjs",
];

struct SeededRoot {
    _dir: TempDir,
    root: PathBuf,
}

fn seed_root_with_config(config: Value) -> SeededRoot {
    let repo = repo_root();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();

    copy_dir_all(&repo.join(".bee/bin/lib"), &root.join(".bee/bin/lib"));
    copy_dir_all(&repo.join(".bee/bin/hooks"), &root.join(".bee/bin/hooks"));
    fs::copy(repo.join(".bee/onboarding.json"), root.join(".bee/onboarding.json")).expect("copy onboarding.json");
    fs::write(
        root.join(".bee/config.json"),
        serde_json::to_string_pretty(&config).expect("serialize config.json"),
    )
    .expect("write config.json");

    for rel in SHA_CHECKED {
        let src = repo.join(".bee").join(rel);
        let dst = root.join(".bee").join(rel);
        assert_eq!(
            sha256_hex(&src),
            sha256_hex(&dst),
            "seeded {rel} diverged from repo source — rig failure (doctored oracle copy)"
        );
    }

    SeededRoot { _dir: dir, root }
}

fn enabling_config() -> Value {
    json!({ "hooks": { "write-guard": true } })
}

/// Two independently seeded, identically configured + identically prepared
/// roots — node runs against one, the rust port against the other, so log
/// appends from either side can never land in the same file.
fn seed_pair_with(config: Value, setup: impl Fn(&Path)) -> (SeededRoot, SeededRoot) {
    let node = seed_root_with_config(config.clone());
    let rust = seed_root_with_config(config);
    setup(&node.root);
    setup(&rust.root);
    (node, rust)
}

fn write_json_file(root: &Path, rel: &str, value: &Value) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn write_state(root: &Path, phase: &str, execution_approved: bool) {
    write_json_file(
        root,
        ".bee/state.json",
        &json!({
            "schema_version": "1.0",
            "phase": phase,
            "feature": "fixture-feature",
            "mode": "standard",
            "approved_gates": { "context": true, "shape": true, "execution": execution_approved, "review": false },
        }),
    );
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

// ---------------------------------------------------------------------------
// Running both runtimes
// ---------------------------------------------------------------------------

struct RunResult {
    status: i32,
    stdout: String,
    stderr: String,
}

fn run(mut cmd: Command, stdin: &str) -> RunResult {
    // The swarm environment exports BEE_AGENT_NAME for reservation
    // ownership; inferAgentName's env fallback would silently absorb it and
    // make fixtures environment-dependent — strip it from both runtimes.
    cmd.env_remove("BEE_AGENT_NAME");
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap_or_else(|e| panic!("spawn failed: {e}"));
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait for child");
    RunResult {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Runs the SEEDED copy of bee-write-guard.mjs living inside `root` — the
/// real oracle per rig discipline (i).
fn run_node(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new("node");
    cmd.arg(root.join(".bee/bin/hooks/bee-write-guard.mjs"));
    cmd.current_dir(root);
    run(cmd, stdin)
}

/// Runs the LIVE repo hook file directly — used only by the negative-control
/// fixture, which has nothing to seed by design.
fn run_live_node(cwd: &Path, stdin: &str) -> RunResult {
    let repo = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(repo.join(".bee/bin/hooks/bee-write-guard.mjs"));
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_rust(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg("write-guard");
    cmd.current_dir(root);
    run(cmd, stdin)
}

fn edit_payload(root: &Path, file_path: &str, extra: &[(&str, Value)]) -> String {
    let mut payload = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": { "file_path": file_path },
        "cwd": root.to_string_lossy(),
    });
    for (key, value) in extra {
        payload[*key] = value.clone();
    }
    payload.to_string()
}

// ---------------------------------------------------------------------------
// Differ
// ---------------------------------------------------------------------------

/// Node-verdict-non-trivial (rig discipline ii): the node oracle must have
/// exited 2 on its own before any diffing, the deny reason must carry the
/// expected class marker, and the rust stderr must be byte-identical to the
/// node reason (node's stderr tail — the reason is written without a
/// trailing newline, so any earlier runtime warning lines split off).
fn assert_deny_conformant(label: &str, node: &RunResult, rust: &RunResult, expected_marker: &str) {
    assert_eq!(node.status, 2, "{label}: node oracle did NOT deny (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 2, "{label}: rust did not deny (stderr={:?})", rust.stderr);
    let node_reason = node.stderr.rsplit('\n').next().unwrap_or("");
    assert!(
        node_reason.contains(expected_marker),
        "{label}: node deny reason missing expected marker {expected_marker:?} — got {node_reason:?}"
    );
    assert_eq!(
        node_reason, rust.stderr,
        "{label}: deny reason diverged between node and rust"
    );
    assert!(node.stdout.trim().is_empty(), "{label}: a deny must not write stdout (node)");
    assert!(rust.stdout.trim().is_empty(), "{label}: a deny must not write stdout (rust)");
}

/// Allow-with-twin-deny pairing support: a genuine, silent allow on BOTH
/// runtimes (exit 0, empty stdout, empty rust stderr).
fn assert_allow_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(node.status, 0, "{label}: node oracle denied unexpectedly (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 0, "{label}: rust denied unexpectedly (stderr={:?})", rust.stderr);
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "{label}: stdout diverged");
    assert!(rust.stderr.is_empty(), "{label}: rust allow must be silent on stderr — got {:?}", rust.stderr);
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
// Fixture class: rig self-check + negative control
// ---------------------------------------------------------------------------

#[test]
fn rig_self_check_seeded_files_match_repo_sha256() {
    let seeded = seed_root_with_config(enabling_config());
    assert!(is_seeded_valid(&seeded.root), "a freshly seeded root must verify as valid");
}

#[test]
fn negative_control_unseeded_root_detected_as_invalid_and_both_runtimes_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!is_seeded_valid(dir.path()), "an unseeded root must never verify as a valid rig setup");

    let stdin = edit_payload(dir.path(), "src/app.js", &[]);
    let node = run_live_node(dir.path(), &stdin);
    let rust = run_rust(dir.path(), &stdin);
    assert_eq!(node.status, 0, "no discoverable root must be exit 0 (fail-open) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
    assert!(!dir.path().join(".bee").exists(), "no root discoverable => no .bee dir may be created");
}

// ---------------------------------------------------------------------------
// Fixture class (a): gate guard — deny before execution approval, allow
// twin on the approved gate and on the allowlist
// ---------------------------------------------------------------------------

#[test]
fn gate_denies_source_write_before_execution_gate_and_approved_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "planning", false));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_deny_conformant("gate/deny", &node, &rust, "bee gate: phase is \"planning\" and gate \"execution\" is not approved");

    // Twin: SAME shape, only approved_gates.execution flipped true.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "planning", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_allow_conformant("gate/allow-twin(execution approved)", &node, &rust);
}

#[test]
fn gate_allowlist_docs_write_allowed_in_gated_phase_without_execution_gate() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "planning", false));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "docs/notes.md", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "docs/notes.md", &[]));
    assert_allow_conformant("gate/allowlist-docs", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class (a): intake gate — idle AND compounding-complete both deny;
// bookkeeping twin and the idle_gate opt-out twin allow
// ---------------------------------------------------------------------------

#[test]
fn intake_gate_denies_source_writes_in_idle_and_compounding_complete_phases() {
    for phase in ["idle", "compounding-complete"] {
        let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, phase, true));
        let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
        let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
        assert_deny_conformant(
            &format!("intake/{phase}"),
            &node,
            &rust,
            &format!("bee intake gate: no bee work is active (phase: {phase})"),
        );
    }
}

#[test]
fn intake_gate_bookkeeping_twin_and_idle_gate_opt_out_twin_allow() {
    // Twin 1: SAME idle shape, target flipped to an allowlisted docs/ path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "docs/notes.md", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "docs/notes.md", &[]));
    assert_allow_conformant("intake/allow-twin(docs)", &node, &rust);

    // Twin 2: SAME idle shape + source target, only guards.idle_gate flipped
    // to false (the repo-level opt-out).
    let config = json!({ "hooks": { "write-guard": true }, "guards": { "idle_gate": false } });
    let (node_seed, rust_seed) = seed_pair_with(config, |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_allow_conformant("intake/allow-twin(idle_gate off)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class (a): direct-edit spine precedence — CLI-owned files deny in
// every phase, before the .bee/ allowlist could save them
// ---------------------------------------------------------------------------

#[test]
fn direct_edit_guard_denies_state_json_with_sibling_bee_path_twin_allowed() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, ".bee/state.json", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, ".bee/state.json", &[]));
    assert_deny_conformant(
        "direct-edit/state.json",
        &node,
        &rust,
        "bee direct-edit guard: \".bee/state.json\" is CLI-owned",
    );

    // Twin: SAME shape, target flipped to a non-CLI-owned .bee/ sibling.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, ".bee/notes.json", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, ".bee/notes.json", &[]));
    assert_allow_conformant("direct-edit/allow-twin(.bee sibling)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class (b): swarming reservation conflict — deny names the holder;
// own-agent twin allows; declared intent downgrades to an advisory warning
// ---------------------------------------------------------------------------

fn write_path_lease(root: &Path, file: &str, lease: &Value) {
    write_json_file(root, &format!(".bee/runtime/leases/paths/{file}"), lease);
}

fn bob_lease(path: &str, kind: &str, session: Option<&str>) -> Value {
    let mut lease = json!({
        "resource": format!("path:{path}"),
        "mode": "exclusive",
        "workflow_id": "rp-cell-1",
        "workspace_id": "agent:Bob",
        "epoch": 0,
        "acquired_at": "2026-01-01T00:00:00.000Z",
        "expires_at": "2099-01-01T00:00:00.000Z",
        "kind": kind,
    });
    if let Some(s) = session {
        lease["session_id"] = json!(s);
    }
    lease
}

#[test]
fn swarming_reservation_conflict_denies_naming_holder_and_own_agent_twin_allows() {
    let setup = |root: &Path| {
        write_state(root, "swarming", true);
        write_path_lease(root, "lease-1.json", &bob_lease("src/app.js", "lease", None));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let extra = [("agent_name", json!("Dave"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_deny_conformant(
        "reservation/deny(foreign agent)",
        &node,
        &rust,
        "bee reservation conflict: \"src/app.js\" is reserved by another agent — Bob holds \"src/app.js\" (cell rp-cell-1)",
    );

    // Twin: SAME shape, only the acting agent flipped to the holder itself.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let extra = [("agent_name", json!("Bob"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_allow_conformant("reservation/allow-twin(own agent)", &node, &rust);
}

#[test]
fn swarming_intent_reservation_downgrades_to_advisory_and_lease_kind_twin_denies() {
    // Broad 'intent' scope covering (not equal to) the write target:
    // advisory warning, never a hard deny (multisession-native-13 D4).
    let intent_setup = |root: &Path| {
        write_state(root, "swarming", true);
        write_path_lease(root, "lease-1.json", &bob_lease("src/*", "intent", None));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), intent_setup);
    let extra = [("agent_name", json!("Dave"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_eq!(node.status, 0, "intent must not hard-block (node) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0, "intent must not hard-block (rust) — stderr={:?}", rust.stderr);
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "advisory allow+notice JSON diverged");
    let notice: Value = serde_json::from_str(node.stdout.trim()).expect("advisory stdout must be JSON");
    assert_eq!(notice["hookSpecificOutput"]["permissionDecision"], json!("allow"));
    assert!(
        notice["systemMessage"].as_str().unwrap_or("").contains("advisory only (kind: intent)"),
        "advisory notice must name the intent downgrade — got {notice}"
    );

    // Twin: SAME broad path, only `kind` flipped to 'lease' — a glob lease
    // is a hard conflict.
    let lease_setup = |root: &Path| {
        write_state(root, "swarming", true);
        write_path_lease(root, "lease-1.json", &bob_lease("src/*", "lease", None));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), lease_setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_deny_conformant(
        "reservation/deny-twin(kind=lease glob)",
        &node,
        &rust,
        "bee reservation conflict: \"src/app.js\" is reserved by another agent",
    );
}

// ---------------------------------------------------------------------------
// Fixture class (c): cross-session holds — foreign live session denies with
// holder identity + expiry; own-session twin allows; corrupt store fails
// closed with a valid-store twin
// ---------------------------------------------------------------------------

#[test]
fn cross_session_hold_denies_foreign_session_with_expiry_and_own_session_twin_allows() {
    let setup = |root: &Path| {
        write_state(root, "executing", true);
        let mut lease = bob_lease("src/app.js", "lease", Some("s-other"));
        lease["workflow_id"] = json!("rp-cell-2");
        write_path_lease(root, "lease-1.json", &lease);
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let extra = [("session_id", json!("s-acting"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_deny_conformant(
        "hold/deny(foreign session)",
        &node,
        &rust,
        "bee cross-session hold: \"src/app.js\" is held by session \"s-other\" (agent Bob, cell rp-cell-2), expires 2099-01-01T00:00:00.000Z",
    );

    // Twin: SAME shape, only the acting session flipped to the holder's own.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let extra = [("session_id", json!("s-other"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_allow_conformant("hold/allow-twin(own session)", &node, &rust);
}

#[test]
fn corrupt_reservation_store_fails_closed_for_session_writes_and_valid_twin_allows() {
    let corrupt_setup = |root: &Path| {
        write_state(root, "executing", true);
        fs::write(root.join(".bee/reservations.json"), "{ definitely not json").unwrap();
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), corrupt_setup);
    let extra = [("session_id", json!("s-acting"))];
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_deny_conformant(
        "hold/deny(corrupt reservation store)",
        &node,
        &rust,
        "bee hold guard: the reservation store (.bee/reservations.json) is present but",
    );

    // Twin: SAME shape, only the store content flipped to valid JSON.
    let valid_setup = |root: &Path| {
        write_state(root, "executing", true);
        write_json_file(root, ".bee/reservations.json", &json!({ "reservations": [] }));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), valid_setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &extra));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &extra));
    assert_allow_conformant("hold/allow-twin(valid store)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class (c): cross-worktree holds — corrupt ledger fails closed;
// exclusive-path foreign hold denies; normal-path foreign hold downgrades
// to the advisory warning
// ---------------------------------------------------------------------------

#[test]
fn corrupt_cross_worktree_ledger_fails_closed_and_valid_twin_allows() {
    let corrupt_setup = |root: &Path| {
        write_state(root, "executing", true);
        fs::create_dir_all(root.join(".bee/runtime")).unwrap();
        fs::write(root.join(".bee/runtime/cross-worktree-holds.json"), "%% torn ledger %%").unwrap();
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), corrupt_setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_deny_conformant(
        "worktree-hold/deny(corrupt ledger)",
        &node,
        &rust,
        "bee cross-worktree hold guard: the shared holds ledger (.bee/runtime/cross-worktree-holds.json",
    );

    // Twin: SAME shape, ledger content flipped to valid-but-empty.
    let valid_setup = |root: &Path| {
        write_state(root, "executing", true);
        write_json_file(root, ".bee/runtime/cross-worktree-holds.json", &json!({ "holds": [] }));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), valid_setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_allow_conformant("worktree-hold/allow-twin(valid ledger)", &node, &rust);
}

fn foreign_hold_ledger(path: &str) -> Value {
    json!({
        "holds": [{
            "path": path,
            "holder": "wt-x",
            "feature": "feat-x",
            "session": "s9",
            "cell": "cx-1",
            "ttl_seconds": 3600,
            "mirrored_at": "2099-01-01T00:00:00.000Z",
            "released_at": null,
        }]
    })
}

#[test]
fn foreign_worktree_hold_on_exclusive_path_denies_hard() {
    let setup = |root: &Path| {
        write_state(root, "executing", true);
        write_json_file(root, ".bee/runtime/cross-worktree-holds.json", &foreign_hold_ledger("Cargo.lock"));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "Cargo.lock", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "Cargo.lock", &[]));
    assert_deny_conformant(
        "worktree-hold/deny(exclusive path)",
        &node,
        &rust,
        "bee cross-worktree hold: \"Cargo.lock\" is held by checkout \"wt-x\" (feature feat-x, cell cx-1), expires 2099-01-01T01:00:00.000Z",
    );
}

#[test]
fn foreign_worktree_hold_on_normal_path_downgrades_to_advisory_warning() {
    // Twin of the exclusive-path deny above: SAME ledger shape, only the
    // held path flipped to a non-exclusive resource — allow, but NEVER
    // silently (multisession-native-14 D4 prohibition).
    let setup = |root: &Path| {
        write_state(root, "executing", true);
        write_json_file(root, ".bee/runtime/cross-worktree-holds.json", &foreign_hold_ledger("src/feature.js"));
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/feature.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/feature.js", &[]));
    assert_eq!(node.status, 0, "advisory must allow (node) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0, "advisory must allow (rust) — stderr={:?}", rust.stderr);
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "advisory allow+notice JSON diverged");
    let notice: Value = serde_json::from_str(node.stdout.trim()).expect("advisory stdout must be JSON");
    assert!(
        notice["systemMessage"]
            .as_str()
            .unwrap_or("")
            .contains("is also held by checkout \"wt-x\""),
        "advisory must name the holding checkout — got {notice}"
    );
}

// ---------------------------------------------------------------------------
// Fixture class (d): worktree containment — traversal/absolute escapes deny
// with the generic message; in-tree twin allows; linked-invalid resolution
// exits 2; granted-sibling targets get the enriched denial; the companion
// mount resolves instead of denying
// ---------------------------------------------------------------------------

const GENERIC_CONTAINMENT: &str =
    "bee write guard denied this target: it could not be canonically contained inside the physical worktree.";

#[test]
fn containment_denies_traversal_and_absolute_outside_targets_and_inside_twin_allows() {
    for (name, target) in [("traversal", "../outside.txt"), ("absolute-outside", "/etc/hosts")] {
        let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
        let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, target, &[]));
        let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, target, &[]));
        assert_deny_conformant(&format!("containment/{name}"), &node, &rust, GENERIC_CONTAINMENT);
    }

    // Twin: SAME shape, target flipped to a plain in-worktree path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_allow_conformant("containment/allow-twin(inside)", &node, &rust);
}

#[test]
fn linked_invalid_worktree_resolution_denies_write_capable_tools_with_exit_2() {
    // A `.git` FILE whose gitdir pointer is unreadable/empty classifies as
    // linked-invalid: no worktree-local .bee store is trusted, deny exit 2
    // BEFORE any store/hookEnabled logic.
    let setup = |root: &Path| {
        write_state(root, "executing", true);
        fs::write(root.join(".git"), "").unwrap();
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_deny_conformant(
        "containment/linked-invalid",
        &node,
        &rust,
        "bee worktree guard denied this write: WORKTREE_LINK_INVALID",
    );

    // Twin: SAME shape without the broken .git marker — ordinary root, allow.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "src/app.js", &[]));
    assert_allow_conformant("containment/allow-twin(no marker)", &node, &rust);
}

/// Fabricates the bidirectional git-worktree metadata
/// `resolveGrantedWorktreeRoot` verifies: `<main>/.git/worktrees/<id>/gitdir`
/// pointing at `<worktree>/.git`, and the worktree's `.git` FILE pointing
/// back at that exact worktrees dir.
fn fabricate_sibling_worktree(main_root: &Path, worktree_root: &Path, id: &str, granted: bool) {
    let main_real = fs::canonicalize(main_root).unwrap();
    let worktree_real = fs::canonicalize(worktree_root).unwrap();
    let git_worktree_dir = main_real.join(".git/worktrees").join(id);
    fs::create_dir_all(&git_worktree_dir).unwrap();
    fs::write(
        git_worktree_dir.join("gitdir"),
        format!("{}\n", worktree_real.join(".git").display()),
    )
    .unwrap();
    fs::write(
        worktree_real.join(".git"),
        format!("gitdir: {}\n", git_worktree_dir.display()),
    )
    .unwrap();
    write_json_file(&main_real, ".bee/runtime/worktree-grants.json", &json!({ id: granted }));
}

/// One seeded root plus its OWN fabricated sibling worktree (the
/// bidirectional gitdir pointers are per-main-checkout, so node and rust
/// can never share one sibling dir — the second fabrication would re-point
/// the reverse pointer and break the first root's verification).
fn sibling_fixture(granted: bool) -> (SeededRoot, TempDir, String) {
    let seeded = seed_root_with_config(enabling_config());
    write_state(&seeded.root, "executing", true);
    let sibling_dir = tempfile::tempdir().expect("sibling tempdir");
    fabricate_sibling_worktree(&seeded.root, sibling_dir.path(), "wt1", granted);
    let target = fs::canonicalize(sibling_dir.path()).unwrap().join("inner.txt");
    let target_str = target.to_string_lossy().into_owned();
    (seeded, sibling_dir, target_str)
}

#[test]
fn sibling_worktree_target_denial_names_granted_worktree_and_ungranted_twin_stays_generic() {
    // Deny with ENRICHED message: target inside a GRANTED sibling checkout.
    // The enriched reason interpolates each root's own sibling realpath, so
    // only the stable message parts are compared across runtimes here (the
    // full byte-exact stderr comparison still runs on every OTHER deny
    // fixture via assert_deny_conformant).
    let (node_seed, _node_sibling, node_target) = sibling_fixture(true);
    let (rust_seed, _rust_sibling, rust_target) = sibling_fixture(true);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, &node_target, &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, &rust_target, &[]));
    assert_eq!(node.status, 2, "granted-sibling target must deny (node) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 2, "granted-sibling target must deny (rust) — stderr={:?}", rust.stderr);
    for (label, result) in [("node", &node), ("rust", &rust)] {
        let reason = result.stderr.rsplit('\n').next().unwrap_or("");
        assert!(
            reason.contains("it resolves inside worktree \"wt1\""),
            "{label}: enriched denial must name the granted worktree — got {reason:?}"
        );
        assert!(
            reason.contains("bee worktree merge --id wt1"),
            "{label}: enriched denial must offer the merge-back route — got {reason:?}"
        );
    }
    // The messages must be byte-identical once each root's own sibling
    // realpath is normalized out.
    let node_reason = node.stderr.rsplit('\n').next().unwrap_or("").replace(
        &fs::canonicalize(_node_sibling.path()).unwrap().to_string_lossy().into_owned(),
        "<SIBLING>",
    );
    let rust_reason = rust.stderr.replace(
        &fs::canonicalize(_rust_sibling.path()).unwrap().to_string_lossy().into_owned(),
        "<SIBLING>",
    );
    assert_eq!(node_reason, rust_reason, "enriched denial diverged beyond the per-root sibling path");

    // Twin: SAME topology, only the grant flipped false — enrichment must
    // NOT fire; the deny stays the generic containment message.
    let (node_seed, _node_sibling, node_target) = sibling_fixture(false);
    let (rust_seed, _rust_sibling, rust_target) = sibling_fixture(false);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, &node_target, &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, &rust_target, &[]));
    assert_deny_conformant("containment/sibling-worktree(ungranted twin)", &node, &rust, GENERIC_CONTAINMENT);
}

#[cfg(unix)]
#[test]
fn companion_mount_target_resolves_inside_marker_verified_mount_and_tampered_marker_twin_denies() {
    // Allow: the marker-declared companion mount (a symlink out of the
    // physical worktree) is recognized, so the write resolves to a
    // root-relative path instead of a containment denial.
    let companion_dir = tempfile::tempdir().expect("companion tempdir");
    let companion_real = fs::canonicalize(companion_dir.path()).unwrap();
    let setup = |root: &Path| {
        write_state(root, "executing", true);
        std::os::unix::fs::symlink(&companion_real, root.join("companion")).unwrap();
        write_json_file(
            root,
            ".bee/companion-session.json",
            &json!({
                "sessionId": "s1",
                "worktreePath": companion_real.to_string_lossy(),
                "mountPath": "companion",
            }),
        );
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "companion/newfile.txt", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "companion/newfile.txt", &[]));
    assert_allow_conformant("containment/companion-mount(allow)", &node, &rust);

    // Twin: SAME shape, only the marker's declared worktreePath flipped to a
    // DIFFERENT directory — a stale/tampered marker must not grant access to
    // wherever the symlink points today; the generic containment deny stays.
    let companion_dir = tempfile::tempdir().expect("companion tempdir");
    let companion_real = fs::canonicalize(companion_dir.path()).unwrap();
    let decoy_dir = tempfile::tempdir().expect("decoy tempdir");
    let decoy_real = fs::canonicalize(decoy_dir.path()).unwrap();
    let tampered_setup = |root: &Path| {
        write_state(root, "executing", true);
        std::os::unix::fs::symlink(&companion_real, root.join("companion")).unwrap();
        write_json_file(
            root,
            ".bee/companion-session.json",
            &json!({
                "sessionId": "s1",
                "worktreePath": decoy_real.to_string_lossy(),
                "mountPath": "companion",
            }),
        );
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), tampered_setup);
    let node = run_node(&node_seed.root, &edit_payload(&node_seed.root, "companion/newfile.txt", &[]));
    let rust = run_rust(&rust_seed.root, &edit_payload(&rust_seed.root, "companion/newfile.txt", &[]));
    assert_deny_conformant("containment/companion-mount(tampered twin)", &node, &rust, GENERIC_CONTAINMENT);
}

// ---------------------------------------------------------------------------
// Fixture class: crash fail-open — an internal fault exits 0 (never a
// denial) plus a hooks.jsonl crash line
// ---------------------------------------------------------------------------

#[test]
fn crash_fail_open_node_oracle_exits_zero_and_logs_crash_instead_of_denying() {
    // The seeded root is shaped so a HEALTHY run would DENY (planning phase,
    // execution unapproved) — proving the exit 0 below is the fail-open
    // contract engaging, not a quietly-allowing fixture. guards.mjs is a
    // downstream dependency (fault-injection input), not part of the diffed
    // oracle surface; the hook + adapter files stay sha256-pristine.
    let seeded = seed_root_with_config(enabling_config());
    write_state(&seeded.root, "planning", false);
    let healthy = run_node(&seeded.root, &edit_payload(&seeded.root, "src/app.js", &[]));
    assert_eq!(healthy.status, 2, "precondition: the healthy oracle denies this shape");

    fs::write(
        seeded.root.join(".bee/bin/lib/guards.mjs"),
        b"throw new Error('rig-injected-fault: guards.mjs unavailable');\n",
    )
    .unwrap();
    for rel in ["bin/hooks/adapter.mjs", "bin/hooks/bee-write-guard.mjs", "bin/lib/state.mjs"] {
        assert_eq!(
            sha256_hex(&repo_root().join(".bee").join(rel)),
            sha256_hex(&seeded.root.join(".bee").join(rel)),
            "oracle surface {rel} must stay pristine in the crash fixture"
        );
    }

    let node = run_node(&seeded.root, &edit_payload(&seeded.root, "src/app.js", &[]));
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must exit 0, never deny — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty(), "a crash must never leak to stdout");
    let lines = read_hooks_log(&seeded.root);
    assert!(
        lines.iter().any(|l| l
            .get("error")
            .and_then(Value::as_str)
            .map(|s| s.contains("rig-injected-fault"))
            .unwrap_or(false)),
        "expected a crash line naming the injected fault, got {lines:?}"
    );
}

#[test]
fn crash_fail_open_rust_wrapper_catches_panic_exits_zero_and_writes_crash_line() {
    // The compiled port has no dynamic-import fault surface, so its crash
    // class is a genuine panic inside the decision region — the SAME
    // fail-open boundary run() wraps every decision in. Proven directly
    // against the public wrapper with a real panic: exit code 0 and a
    // hooks.jsonl crash line, never a deny.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let code = queen_bee::hooks::write_guard::run_fail_open(Some(root), "write-guard", Some("repo"), || {
        panic!("rig-injected-fault: deliberate panic in the decision region");
    });
    assert_eq!(code, 0, "fail-open contract: a panic must resolve to exit 0");
    let lines = read_hooks_log(root);
    assert!(
        lines.iter().any(|l| l
            .get("error")
            .and_then(Value::as_str)
            .map(|s| s.contains("rig-injected-fault"))
            .unwrap_or(false)),
        "expected a crash line naming the injected panic, got {lines:?}"
    );
}
