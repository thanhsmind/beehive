//! modelguard_conformance — D7b conformance corpus for the model-guard
//! dispatch-transport hook (rust-port-10): proves `queen-bee hook
//! model-guard` matches the REAL `.bee/bin/hooks/bee-model-guard.mjs` (run
//! through the shared, frozen adapter + `.bee/bin/lib/dispatch-guard.mjs` +
//! `.bee/bin/lib/state.mjs`) for every deny class dispatch-guard.mjs
//! enumerates — never a reimplementation guess.
//!
//! RIG DISCIPLINE (inherited from rust-port-7's hook_conformance rig, per
//! panel B3 / writeguard_core.rs's own restatement):
//! (i) SEEDING — every node-oracle run happens inside a fresh temp root
//!     seeded with `.bee/bin/lib/`, `.bee/bin/hooks/`, `.bee/onboarding.json`,
//!     and a `config.json`; the oracle executes the SEEDED copy of
//!     `bee-model-guard.mjs`, sha256-verified against the repo source.
//! (ii) NON-TRIVIALITY BOTH WAYS — every deny fixture asserts the node
//!      oracle exited 2 BEFORE diffing, and is paired with an allow twin in
//!      the same shape with exactly one field flipped.
//! (iii) NEGATIVE CONTROL — an unseeded root must be DETECTED as invalid by
//!       the rig's own verifier.
//! (iv) Descriptive per-class fixture names, listed by cargo test output.
//!
//! Every temp root comes from `tempfile::tempdir()` — never the live
//! `.bee/` store.
//!
//! Deny-class coverage (all seven `dispatch-guard.mjs` classes):
//! `codex-spawn-unmarked`, `generic-type-denied`, `param-tier-mismatch`,
//! `param-on-nameless-tier`, `param-not-configured`, `cli-tier-denied`,
//! `bare-denied`.

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
        if dir.join(".bee/bin/hooks/bee-model-guard.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/hooks/bee-model-guard.mjs walking ancestors from {}",
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

/// The oracle surface for the model-guard dispatch decision: every module
/// the deny/allow verdicts flow through. A doctored seeded copy of any of
/// these is a rig failure.
const SHA_CHECKED: &[&str] = &["bin/hooks/adapter.mjs", "bin/hooks/bee-model-guard.mjs", "bin/lib/state.mjs", "bin/lib/dispatch-guard.mjs"];

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
    fs::write(root.join(".bee/config.json"), serde_json::to_string_pretty(&config).expect("serialize config.json")).expect("write config.json");

    for rel in SHA_CHECKED {
        let src = repo.join(".bee").join(rel);
        let dst = root.join(".bee").join(rel);
        assert_eq!(sha256_hex(&src), sha256_hex(&dst), "seeded {rel} diverged from repo source — rig failure (doctored oracle copy)");
    }

    SeededRoot { _dir: dir, root }
}

fn enabling_config(models: Value) -> Value {
    json!({ "hooks": { "model-guard": true }, "models": models })
}

/// Two independently seeded, identically configured roots — node runs
/// against one, the rust port against the other, so log appends from either
/// side can never land in the same file.
fn seed_pair(config: Value) -> (SeededRoot, SeededRoot) {
    (seed_root_with_config(config.clone()), seed_root_with_config(config))
}

/// A root seeded like [`seed_root_with_config`], except `.bee/bin/lib/
/// dispatch-guard.mjs` — a downstream decision-logic DEPENDENCY
/// `bee-model-guard.mjs` dynamically imports, not the oracle unit under
/// test at THIS assertion's narrower scope — is replaced with a
/// deliberately-throwing stub. The fault-injection point mirrors
/// `writeguard_core.rs`'s own crash fixture (which corrupts `guards.mjs`,
/// a peer downstream dependency, while keeping `adapter.mjs` /
/// `bee-write-guard.mjs` / `state.mjs` pristine).
fn seed_crash_root() -> SeededRoot {
    let repo = repo_root();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();

    copy_dir_all(&repo.join(".bee/bin/lib"), &root.join(".bee/bin/lib"));
    copy_dir_all(&repo.join(".bee/bin/hooks"), &root.join(".bee/bin/hooks"));
    fs::copy(repo.join(".bee/onboarding.json"), root.join(".bee/onboarding.json")).expect("copy onboarding.json");
    fs::write(root.join(".bee/config.json"), serde_json::to_string_pretty(&enabling_config(json!({}))).unwrap()).expect("write config.json");

    fs::write(root.join(".bee/bin/lib/dispatch-guard.mjs"), b"throw new Error('rig-injected-fault: dispatch-guard unavailable');\n")
        .expect("inject crash fault into dispatch-guard.mjs");

    // The narrower oracle surface for THIS fixture (the hook wrapper +
    // adapter + state resolution) must still be pristine — only the
    // fault-injection dependency deviates.
    for rel in ["bin/hooks/adapter.mjs", "bin/hooks/bee-model-guard.mjs", "bin/lib/state.mjs"] {
        let src = repo.join(".bee").join(rel);
        let dst = root.join(".bee").join(rel);
        assert_eq!(sha256_hex(&src), sha256_hex(&dst), "seeded {rel} diverged from repo source — rig failure");
    }

    SeededRoot { _dir: dir, root }
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

/// Runs the SEEDED copy of bee-model-guard.mjs living inside `root` — the
/// real oracle per rig discipline (i).
fn run_node(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new("node");
    cmd.arg(root.join(".bee/bin/hooks/bee-model-guard.mjs"));
    cmd.current_dir(root);
    run(cmd, stdin)
}

/// Runs the LIVE repo hook file directly — used only by the negative-control
/// fixture, which has nothing to seed by design.
fn run_live_node(cwd: &Path, stdin: &str) -> RunResult {
    let repo = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(repo.join(".bee/bin/hooks/bee-model-guard.mjs"));
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_rust(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg("model-guard");
    cmd.current_dir(root);
    run(cmd, stdin)
}

fn agent_payload(root: &Path, tool_input: Value) -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Agent",
        "tool_input": tool_input,
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

fn spawn_payload(root: &Path, tool_input: Value) -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "spawn_agent",
        "tool_input": tool_input,
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Differ
// ---------------------------------------------------------------------------

/// Node-verdict-non-trivial (rig discipline ii): the node oracle must have
/// exited 2 on its own before any diffing, and the rust stderr must be
/// byte-identical to the node reason.
fn assert_deny_conformant(label: &str, node: &RunResult, rust: &RunResult, expected_substr: &str) {
    assert_eq!(node.status, 2, "{label}: node oracle did NOT deny (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 2, "{label}: rust did not deny (stderr={:?})", rust.stderr);
    assert!(
        node.stderr.contains(expected_substr),
        "{label}: node deny reason missing expected substring {expected_substr:?} — got {:?}",
        node.stderr
    );
    assert_eq!(node.stderr, rust.stderr, "{label}: deny reason diverged between node and rust");
    assert!(node.stdout.trim().is_empty(), "{label}: a deny must not write stdout (node)");
    assert!(rust.stdout.trim().is_empty(), "{label}: a deny must not write stdout (rust)");
}

/// Allow-with-twin-deny pairing support: a genuine, silent allow on BOTH
/// runtimes (exit 0, empty stdout, empty rust stderr).
fn assert_allow_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(node.status, 0, "{label}: node oracle denied unexpectedly (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 0, "{label}: rust denied unexpectedly (stderr={:?})", rust.stderr);
    assert!(node.stderr.trim().is_empty(), "{label}: node allow must be silent on stderr — got {:?}", node.stderr);
    assert!(rust.stderr.trim().is_empty(), "{label}: rust allow must be silent on stderr — got {:?}", rust.stderr);
}

fn read_hooks_log(root: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(root.join(".bee/logs/hooks.jsonl")) else {
        return Vec::new();
    };
    text.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}"))).collect()
}

fn read_dispatch_log(root: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")) else {
        return Vec::new();
    };
    text.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}"))).collect()
}

/// Volatile-field normalization for dispatch.jsonl line-shape parity: `ts`
/// is a wall-clock timestamp, expected to differ between two independent
/// process runs — every other field must match exactly.
fn redact_ts(mut line: Value) -> Value {
    if let Value::Object(map) = &mut line {
        map.insert("ts".to_string(), Value::String("<ts>".to_string()));
    }
    line
}

// ---------------------------------------------------------------------------
// Fixture class: rig self-check + negative control
// ---------------------------------------------------------------------------

#[test]
fn rig_self_check_seeded_files_match_repo_sha256() {
    let seeded = seed_root_with_config(enabling_config(json!({})));
    assert!(is_seeded_valid(&seeded.root), "a freshly seeded root must verify as valid");
}

#[test]
fn negative_control_unseeded_root_detected_as_invalid_and_both_runtimes_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!is_seeded_valid(dir.path()), "an unseeded root must never verify as a valid rig setup");

    let stdin = agent_payload(dir.path(), json!({ "description": "[bee-tier: generation] do a thing" }));
    let node = run_live_node(dir.path(), &stdin);
    let rust = run_rust(dir.path(), &stdin);
    assert_eq!(node.status, 0, "no discoverable root must be exit 0 (fail-open) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
    assert!(!dir.path().join(".bee").exists(), "no root discoverable => no .bee dir may be created");
}

// ---------------------------------------------------------------------------
// Fixture class: bare-denied <-> marker allow twin
// ---------------------------------------------------------------------------

#[test]
fn bare_dispatch_denies_and_marker_twin_allows() {
    let config = enabling_config(json!({ "claude": { "generation": "sonnet" } }));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, json!({ "subagent_type": "general-purpose" })));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, json!({ "subagent_type": "general-purpose" })));
    assert_deny_conformant("bare/deny", &node, &rust, "needs an explicit tier");

    // Twin: SAME shape, a [bee-tier: ceiling] marker added (ceiling is
    // exempt from the pinned-type rule, so general-purpose stays legal).
    let (node_seed, rust_seed) = seed_pair(config);
    let allowed_input = json!({ "description": "[bee-tier: ceiling] do a thing", "subagent_type": "general-purpose" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("bare/allow-twin(ceiling marker)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: generic-type-denied <-> pinned subagent_type twin
// ---------------------------------------------------------------------------

#[test]
fn generic_type_denied_for_pinned_tier_on_general_purpose_and_pinned_type_twin_allows() {
    let config = enabling_config(json!({ "claude": { "generation": "sonnet" } }));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let denied_input = json!({ "description": "[bee-tier: generation] do a thing", "subagent_type": "general-purpose" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("generic-type/deny", &node, &rust, "must spawn its pinned agent type");

    // Twin: SAME shape, subagent_type flipped to the pinned agent.
    let (node_seed, rust_seed) = seed_pair(config);
    let allowed_input = json!({ "description": "[bee-tier: generation] do a thing", "subagent_type": "bee-gather" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("generic-type/allow-twin(pinned agent)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: param-tier-mismatch <-> matching-param twin (model-param
// allow transport)
// ---------------------------------------------------------------------------

#[test]
fn param_tier_mismatch_denies_and_matching_param_twin_allows() {
    let config = enabling_config(json!({ "claude": { "generation": "sonnet" } }));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let denied_input = json!({ "description": "[bee-tier: generation] do a thing", "model": "haiku", "subagent_type": "bee-gather" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("param-tier-mismatch/deny", &node, &rust, "resolves to model");

    // Twin: SAME shape, model param matches the tier's resolved model.
    let (node_seed, rust_seed) = seed_pair(config);
    let allowed_input = json!({ "description": "[bee-tier: generation] do a thing", "model": "sonnet", "subagent_type": "bee-gather" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("param-tier-mismatch/allow-twin(matching param)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: param-on-nameless-tier <-> marker-alone twin
// ---------------------------------------------------------------------------

#[test]
fn param_on_nameless_tier_denies_for_ceiling_plus_model_and_marker_alone_twin_allows() {
    let config = enabling_config(json!({}));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let denied_input = json!({ "description": "[bee-tier: ceiling] do a thing", "model": "sonnet" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("param-on-nameless-tier/deny", &node, &rust, "resolves to no model name");

    // Twin: SAME shape, the model param dropped — ceiling alone allows.
    let (node_seed, rust_seed) = seed_pair(config);
    let allowed_input = json!({ "description": "[bee-tier: ceiling] do a thing" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("param-on-nameless-tier/allow-twin(marker alone)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: param-not-configured <-> configured-model twin
// ---------------------------------------------------------------------------

#[test]
fn param_not_configured_denies_and_configured_model_twin_allows() {
    let config = enabling_config(json!({ "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" } }));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let denied_input = json!({ "model": "gpt-5" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("param-not-configured/deny", &node, &rust, "is not a model configured");

    // Twin: SAME shape, the model matches a configured tier slot.
    let (node_seed, rust_seed) = seed_pair(config);
    let allowed_input = json!({ "model": "haiku" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("param-not-configured/allow-twin(configured model)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: cli-tier-denied <-> plain-model twin (same slot, kind flipped)
// ---------------------------------------------------------------------------

#[test]
fn cli_tier_denied_for_marker_only_cli_slot_and_plain_model_twin_allows() {
    let cli_config = enabling_config(json!({ "claude": { "review": { "kind": "cli", "command": "codex exec -s read-only -" } } }));
    let (node_seed, rust_seed) = seed_pair(cli_config);
    let denied_input = json!({ "description": "[bee-tier: review] do a thing", "subagent_type": "bee-review" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("cli-tier-denied/deny", &node, &rust, "resolves to a cli executor");

    // Twin: SAME slot, the value's kind flipped from cli to a plain model
    // name — the marker-alone path now resolves to a real model and allows.
    let model_config = enabling_config(json!({ "claude": { "review": "opus" } }));
    let (node_seed, rust_seed) = seed_pair(model_config);
    let allowed_input = json!({ "description": "[bee-tier: review] do a thing", "subagent_type": "bee-review" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("cli-tier-denied/allow-twin(plain model)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: codex-spawn-unmarked <-> marker twin
// ---------------------------------------------------------------------------

#[test]
fn codex_spawn_unmarked_denies_and_marker_twin_allows() {
    let config = enabling_config(json!({}));
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let denied_input = json!({ "message": "do a thing without a tier" });
    let node = run_node(&node_seed.root, &spawn_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &spawn_payload(&rust_seed.root, denied_input));
    assert_deny_conformant("codex-spawn-unmarked/deny", &node, &rust, "every Codex spawn_agent needs an explicit tier");

    // Twin: SAME shape, the message opens with an anchored marker.
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let allowed_input = json!({ "message": "[bee-tier: generation] do a thing" });
    let node = run_node(&node_seed.root, &spawn_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &spawn_payload(&rust_seed.root, allowed_input));
    assert_allow_conformant("codex-spawn-unmarked/allow-twin(marker)", &node, &rust);

    // The Codex-branch marker vocabulary additionally recognizes `advisor`
    // (never valid on the Claude branch) — proves the two marker regexes
    // stay genuinely distinct across both runtimes.
    let (node_seed, rust_seed) = seed_pair(config);
    let advisor_input = json!({ "message": "[bee-tier: advisor] do a thing" });
    let node = run_node(&node_seed.root, &spawn_payload(&node_seed.root, advisor_input.clone()));
    let rust = run_rust(&rust_seed.root, &spawn_payload(&rust_seed.root, advisor_input));
    assert_allow_conformant("codex-spawn-unmarked/allow(advisor marker)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: dispatch.jsonl line-shape parity (volatile fields
// normalized)
// ---------------------------------------------------------------------------

#[test]
fn dispatch_jsonl_line_shape_parity_for_allow_and_deny() {
    let config = enabling_config(json!({ "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" } }));

    // Allow line (model-param transport, economics: pinned).
    let (node_seed, rust_seed) = seed_pair(config.clone());
    let allowed_input = json!({ "model": "haiku", "description": "extract a fact from the repo" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, allowed_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, allowed_input));
    assert_eq!(node.status, 0);
    assert_eq!(rust.status, 0);
    let node_lines = read_dispatch_log(&node_seed.root);
    let rust_lines = read_dispatch_log(&rust_seed.root);
    assert_eq!(node_lines.len(), 1, "expected exactly one node dispatch.jsonl line, got {node_lines:?}");
    assert_eq!(rust_lines.len(), 1, "expected exactly one rust dispatch.jsonl line, got {rust_lines:?}");
    assert_eq!(
        redact_ts(node_lines[0].clone()),
        redact_ts(rust_lines[0].clone()),
        "allow dispatch.jsonl line shape diverged (ts-normalized)"
    );
    assert_eq!(node_lines[0].get("transport").and_then(Value::as_str), Some("model-param"));
    assert_eq!(node_lines[0].get("effective_model_status").and_then(Value::as_str), Some("pinned"));

    // Deny line (bare-denied transport, economics: prompt-budget/unverified).
    let (node_seed, rust_seed) = seed_pair(config);
    let denied_input = json!({ "subagent_type": "general-purpose" });
    let node = run_node(&node_seed.root, &agent_payload(&node_seed.root, denied_input.clone()));
    let rust = run_rust(&rust_seed.root, &agent_payload(&rust_seed.root, denied_input));
    assert_eq!(node.status, 2);
    assert_eq!(rust.status, 2);
    let node_lines = read_dispatch_log(&node_seed.root);
    let rust_lines = read_dispatch_log(&rust_seed.root);
    assert_eq!(node_lines.len(), 1);
    assert_eq!(rust_lines.len(), 1);
    assert_eq!(
        redact_ts(node_lines[0].clone()),
        redact_ts(rust_lines[0].clone()),
        "deny dispatch.jsonl line shape diverged (ts-normalized)"
    );
    assert_eq!(node_lines[0].get("transport").and_then(Value::as_str), Some("bare-denied"));

    // Both denies also append a hooks.jsonl `deny` event line — proved
    // conformant the same way (redacted ts, deep-equal otherwise).
    let node_hooks = read_hooks_log(&node_seed.root);
    let rust_hooks = read_hooks_log(&rust_seed.root);
    assert_eq!(node_hooks.len(), 1, "expected exactly one node hooks.jsonl line, got {node_hooks:?}");
    assert_eq!(rust_hooks.len(), 1, "expected exactly one rust hooks.jsonl line, got {rust_hooks:?}");
    assert_eq!(redact_ts(node_hooks[0].clone()), redact_ts(rust_hooks[0].clone()), "hooks.jsonl deny line shape diverged (ts-normalized)");
}

// ---------------------------------------------------------------------------
// Fixture class: crash fail-open — an internal fault exits 0 (never a
// denial) plus a hooks.jsonl crash line
// ---------------------------------------------------------------------------

#[test]
fn crash_fail_open_node_oracle_exits_zero_and_logs_crash_instead_of_denying() {
    // The seeded root is shaped so a HEALTHY run would DENY (a bare
    // dispatch) — proving the exit 0 below is the fail-open contract
    // engaging, not a quietly-allowing fixture.
    let seeded = seed_crash_root();
    let stdin = agent_payload(&seeded.root, json!({}));

    let node = run_node(&seeded.root, &stdin);
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must exit 0, never deny — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty(), "a crash must never leak to stdout");
    assert!(read_dispatch_log(&seeded.root).is_empty(), "a crashed evaluation must never produce a dispatch.jsonl line");
    let lines = read_hooks_log(&seeded.root);
    assert!(
        lines.iter().any(|l| l.get("error").and_then(Value::as_str).map(|s| s.contains("rig-injected-fault")).unwrap_or(false)),
        "expected a crash line naming the injected fault, got {lines:?}"
    );
}

#[test]
fn crash_fail_open_rust_wrapper_catches_panic_exits_zero_and_writes_crash_line() {
    // The compiled port has no dynamic-import fault surface, so its crash
    // class is a genuine panic inside the decision region — the SAME
    // fail-open boundary `model_guard::run()` wraps every decision in
    // (it reuses `write_guard::run_fail_open`, one implementation for both
    // hooks). Proven directly against the public wrapper with a real
    // panic: exit code 0 and a hooks.jsonl crash line, never a deny.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let code = queen_bee::hooks::write_guard::run_fail_open(Some(root), "model-guard", Some("repo"), || {
        panic!("rig-injected-fault: deliberate panic in the model-guard decision region");
    });
    assert_eq!(code, 0, "fail-open contract: a panic must resolve to exit 0");
    let lines = read_hooks_log(root);
    assert!(
        lines.iter().any(|l| l.get("error").and_then(Value::as_str).map(|s| s.contains("rig-injected-fault")).unwrap_or(false)),
        "expected a crash line naming the injected panic, got {lines:?}"
    );
}

// ---------------------------------------------------------------------------
// Red-first meta-proof: the differ itself must have teeth.
// ---------------------------------------------------------------------------

#[test]
fn rig_diff_detector_flags_a_seeded_mutation_as_divergent() {
    let node_seed = seed_root_with_config(enabling_config(json!({ "claude": { "generation": "sonnet" } })));
    let rust_seed = seed_root_with_config(enabling_config(json!({ "claude": { "generation": "opus" } })));
    let stdin_for = |root: &Path| agent_payload(root, json!({ "description": "[bee-tier: generation] x", "model": "sonnet", "subagent_type": "bee-gather" }));

    let node = run_node(&node_seed.root, &stdin_for(&node_seed.root));
    let rust = run_rust(&rust_seed.root, &stdin_for(&rust_seed.root));

    let result = std::panic::catch_unwind(|| assert_deny_conformant("deliberately-mutated", &node, &rust, "resolves to model"));
    assert!(result.is_err(), "the differ failed to flag a deliberately seeded divergence — the rig has no teeth");
}
