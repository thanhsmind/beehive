//! writeguard_read — D7b conformance corpus for the write-guard READ side,
//! Codex `apply_patch` target proving, and `AskUserQuestion` schema guard
//! (rust-port-12, split 3 of 3): proves `queen-bee hook write-guard` matches
//! the REAL `.bee/bin/hooks/bee-write-guard.mjs` (run through the shared,
//! frozen adapter + guards libs) for check classes checkRead (privacy +
//! scout), the large-read size guard, Codex `apply_patch` provability, and
//! AskUserQuestion validation (schema deny + ask-guard-autofix repair) —
//! never a reimplementation guess. The core write spine is rust-port-9's
//! corpus (writeguard_core.rs); the Bash path is rust-port-11's
//! (writeguard_bash.rs).
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
//!
//! JSON stdout comparison note: the `AskUserQuestion` autofix fixture's
//! allow+notice stdout is compared STRUCTURALLY (parse both sides, compare
//! as `serde_json::Value`), not byte-for-byte — `serde_json::Value`'s default
//! `Map` has no `preserve_order` feature enabled in this workspace, so the
//! Rust side's `hookSpecificOutput` envelope serializes its five fields
//! alphabetically while the mjs source's own object literal serializes them
//! in ITS insertion order (`hookEventName, permissionDecision,
//! permissionDecisionReason, updatedInput, additionalContext` — not
//! alphabetical, since `additionalContext` is written last). Both orders
//! are valid, semantically identical JSON; a structural compare is the
//! correct conformance check here, not a byte diff of one arbitrary key
//! order against another. (The echoed `updatedInput` substructure itself
//! independently ends up byte-identical in practice, because this test
//! constructs the shared stdin text via `serde_json` too, so both runtimes
//! start from the SAME already-alphabetized source order.)

use serde_json::{json, Map, Value};
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

/// The oracle surface for the write-guard read side / apply_patch /
/// AskUserQuestion checks: every module the deny decisions flow through. A
/// doctored seeded copy of any of these is a rig failure.
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

fn config_with_max_read_lines(threshold: u64) -> Value {
    json!({ "hooks": { "write-guard": true }, "guards": { "max_read_lines": threshold } })
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

fn write_text_file(root: &Path, rel: &str, content: &[u8]) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
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

fn run_rust(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg("write-guard");
    cmd.current_dir(root);
    run(cmd, stdin)
}

fn read_payload(root: &Path, tool_name: &str, file_path: &str, tool_extra: &[(&str, Value)]) -> String {
    let mut tool_input = Map::new();
    tool_input.insert("file_path".to_string(), json!(file_path));
    for (k, v) in tool_extra {
        tool_input.insert((*k).to_string(), v.clone());
    }
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": tool_name,
        "tool_input": Value::Object(tool_input),
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

fn apply_patch_payload(root: &Path, tool_name: &str, input_text: &str) -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": tool_name,
        "tool_input": { "input": input_text },
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

fn ask_payload(root: &Path, questions: Value) -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "AskUserQuestion",
        "tool_input": { "questions": questions },
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

fn well_formed_option(label: &str, description: &str) -> Value {
    json!({ "label": label, "description": description })
}

fn well_formed_question(header: &str) -> Value {
    json!({
        "header": header,
        "question": "Which approach should we take?",
        "options": [
            well_formed_option("Option A", "Do the first thing"),
            well_formed_option("Option B", "Do the second thing"),
        ],
    })
}

// ---------------------------------------------------------------------------
// Differ
// ---------------------------------------------------------------------------

/// Node-verdict-non-trivial (rig discipline ii): the node oracle must have
/// exited 2 on its own before any diffing, the deny reason must carry the
/// expected class marker, and the rust stderr must be byte-identical to the
/// FULL node stderr. Unlike a single-line reason, `checkRead`'s privacy
/// class writes a two-line reason (question line + `@@BEE_PRIVACY@@` marker
/// line) joined by `\n` — this hook never writes anything else to stderr on
/// a deny (a single `process.stderr.write(denial.reason)` call is the only
/// stderr write in the whole source), so a full-string compare is both
/// simpler and stricter than a last-line-only trick.
fn assert_deny_conformant(label: &str, node: &RunResult, rust: &RunResult, expected_marker: &str) {
    assert_eq!(node.status, 2, "{label}: node oracle did NOT deny (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 2, "{label}: rust did not deny (stderr={:?})", rust.stderr);
    assert!(
        node.stderr.contains(expected_marker),
        "{label}: node deny reason missing expected marker {expected_marker:?} — got {:?}",
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
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "{label}: stdout diverged");
    assert!(rust.stderr.is_empty(), "{label}: rust allow must be silent on stderr — got {:?}", rust.stderr);
}

/// Allow-with-notice, compared STRUCTURALLY (see module doc comment) — the
/// key-order caveat only, everything else (exit code, silence on stderr,
/// content equivalence) is the same conformance bar as `assert_allow_conformant`.
fn assert_allow_notice_json_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(node.status, 0, "{label}: node oracle denied unexpectedly (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 0, "{label}: rust denied unexpectedly (stderr={:?})", rust.stderr);
    assert!(rust.stderr.is_empty(), "{label}: rust allow must be silent on stderr — got {:?}", rust.stderr);
    let node_json: Value = serde_json::from_str(node.stdout.trim())
        .unwrap_or_else(|e| panic!("{label}: node stdout not valid JSON: {e} — {:?}", node.stdout));
    let rust_json: Value = serde_json::from_str(rust.stdout.trim())
        .unwrap_or_else(|e| panic!("{label}: rust stdout not valid JSON: {e} — {:?}", rust.stdout));
    assert_eq!(node_json, rust_json, "{label}: stdout JSON diverged (structural, order-independent)");
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

    let stdin = read_payload(dir.path(), "Read", "src/app.js", &[]);
    let node = {
        let mut cmd = Command::new("node");
        cmd.arg(repo_root().join(".bee/bin/hooks/bee-write-guard.mjs"));
        cmd.current_dir(dir.path());
        run(cmd, &stdin)
    };
    let rust = run_rust(dir.path(), &stdin);
    assert_eq!(node.status, 0, "no discoverable root must be exit 0 (fail-open) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
    assert!(!dir.path().join(".bee").exists(), "no root discoverable => no .bee dir may be created");
}

// ---------------------------------------------------------------------------
// Fixture class: checkRead privacy — secret-shaped paths deny with the
// @@BEE_PRIVACY@@ marker; a non-secret twin allows
// ---------------------------------------------------------------------------

#[test]
fn privacy_denies_dotenv_read_with_marker_and_allow_twin() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", ".env", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", ".env", &[]));
    assert_deny_conformant(
        "privacy/deny(.env)",
        &node,
        &rust,
        "bee privacy guard: \".env\" looks like a secret/credential file. Ask the user before reading it.",
    );
    assert!(
        node.stderr.contains("@@BEE_PRIVACY@@") && node.stderr.contains("@@END@@"),
        "privacy deny must carry the marker envelope on both runtimes — got {:?}",
        node.stderr
    );
    assert!(
        node.stderr.contains(r#""file":".env""#) && node.stderr.contains(r#""question":"#),
        "privacy marker body must carry the file/question JSON fields — got {:?}",
        node.stderr
    );

    // Twin: SAME shape, target flipped to a non-secret path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "src/app.js", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "src/app.js", &[]));
    assert_allow_conformant("privacy/allow-twin(src/app.js)", &node, &rust);
}

#[test]
fn privacy_denies_id_rsa_and_credentials_variants() {
    for secret_path in ["id_rsa", "id_rsa.pub", "credentials.json", "secrets.yaml", "config/app.pem"] {
        let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
        let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", secret_path, &[]));
        let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", secret_path, &[]));
        assert_deny_conformant(&format!("privacy/deny({secret_path})"), &node, &rust, "bee privacy guard:");
    }
}

// ---------------------------------------------------------------------------
// Fixture class: checkRead scout — vendored/generated dirs deny; a sibling
// source path twin allows
// ---------------------------------------------------------------------------

#[test]
fn scout_denies_node_modules_read_and_allow_twin() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "node_modules/foo/index.js", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "node_modules/foo/index.js", &[]));
    assert_deny_conformant(
        "scout/deny(node_modules)",
        &node,
        &rust,
        "bee scout guard: \"node_modules/foo/index.js\" is inside \"node_modules/\"",
    );

    // Twin: SAME shape, target flipped to a plain source path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "src/lib/index.js", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "src/lib/index.js", &[]));
    assert_allow_conformant("scout/allow-twin(src/lib)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: large-read size guard — an unbounded Read of an oversized
// text file denies; an `offset`/`limit` twin and a below-threshold twin
// allow
// ---------------------------------------------------------------------------

#[test]
fn read_size_guard_denies_oversized_read_and_offset_limit_twin_allows() {
    let big_text = b"line1\nline2\nline3\nline4\nline5\nline6\n"; // 6 lines
    let setup = |root: &Path| write_text_file(root, "big.txt", big_text);
    let (node_seed, rust_seed) = seed_pair_with(config_with_max_read_lines(5), setup);
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "big.txt", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "big.txt", &[]));
    assert_deny_conformant(
        "read-size/deny(oversized)",
        &node,
        &rust,
        "bee read-size guard: \"big.txt\" is 6 lines (threshold: 5) and this Read has neither `offset` nor `limit`",
    );

    // Twin: SAME oversized file, `limit` supplied — the size guard never
    // fires when the call already carries offset or limit (D4).
    let (node_seed, rust_seed) = seed_pair_with(config_with_max_read_lines(5), setup);
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "big.txt", &[("limit", json!(5))]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "big.txt", &[("limit", json!(5))]));
    assert_allow_conformant("read-size/allow-twin(limit supplied)", &node, &rust);
}

#[test]
fn read_size_guard_below_threshold_twin_allows() {
    let small_text = b"line1\nline2\n"; // 2 lines
    let setup = |root: &Path| write_text_file(root, "small.txt", small_text);
    let (node_seed, rust_seed) = seed_pair_with(config_with_max_read_lines(5), setup);
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "small.txt", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "small.txt", &[]));
    assert_allow_conformant("read-size/allow(below-threshold)", &node, &rust);
}

#[test]
fn read_size_guard_binary_sniff_allows_despite_oversized_line_count() {
    // Same oversized line count as the deny fixture above, but a leading
    // null byte marks it binary — the size guard's null-byte sniff must
    // short-circuit to allow rather than deny on a meaningless "line count".
    let mut binary_content: Vec<u8> = vec![0u8];
    binary_content.extend_from_slice(b"line1\nline2\nline3\nline4\nline5\nline6\n");
    let setup = |root: &Path| write_text_file(root, "big.bin", &binary_content);
    let (node_seed, rust_seed) = seed_pair_with(config_with_max_read_lines(5), setup);
    let node = run_node(&node_seed.root, &read_payload(&node_seed.root, "Read", "big.bin", &[]));
    let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, "Read", "big.bin", &[]));
    assert_allow_conformant("read-size/allow(binary-sniff)", &node, &rust);
}

#[test]
fn read_size_guard_never_fires_for_glob_or_grep() {
    // The `toolName === "Read"` gate means Glob/Grep never carry whole-file
    // content to load — an oversized target must never deny for either.
    let big_text = b"line1\nline2\nline3\nline4\nline5\nline6\n";
    for tool_name in ["Glob", "Grep"] {
        let setup = |root: &Path| write_text_file(root, "big.txt", big_text);
        let (node_seed, rust_seed) = seed_pair_with(config_with_max_read_lines(5), setup);
        let node = run_node(&node_seed.root, &read_payload(&node_seed.root, tool_name, "big.txt", &[]));
        let rust = run_rust(&rust_seed.root, &read_payload(&rust_seed.root, tool_name, "big.txt", &[]));
        assert_allow_conformant(&format!("read-size/allow({tool_name} never sized)"), &node, &rust);
    }
}

// ---------------------------------------------------------------------------
// Fixture class: Codex apply_patch target proving — a provable target runs
// the ordinary write checks and allows; an unprovable target denies; a
// malformed (non-canonical) envelope fails open
// ---------------------------------------------------------------------------

#[test]
fn apply_patch_provable_target_allows() {
    let setup = |root: &Path| write_state(root, "executing", true);
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let patch = "*** Begin Patch\n*** Update File: src/app.js\n*** End Patch\n";
    let node = run_node(&node_seed.root, &apply_patch_payload(&node_seed.root, "apply_patch", patch));
    let rust = run_rust(&rust_seed.root, &apply_patch_payload(&rust_seed.root, "apply_patch", patch));
    assert_allow_conformant("apply_patch/provable-allow", &node, &rust);
}

#[test]
fn apply_patch_unprovable_target_denies() {
    let setup = |root: &Path| write_state(root, "executing", true);
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let patch = "*** Begin Patch\n*** Add File: ../outside.js\n*** End Patch\n";
    let node = run_node(&node_seed.root, &apply_patch_payload(&node_seed.root, "apply_patch", patch));
    let rust = run_rust(&rust_seed.root, &apply_patch_payload(&rust_seed.root, "apply_patch", patch));
    assert_deny_conformant(
        "apply_patch/unprovable-deny",
        &node,
        &rust,
        "bee apply_patch guard: this patch's target set could not be fully proved inside the repo",
    );
}

#[test]
fn apply_patch_malformed_envelope_fails_open_allow() {
    let setup = |root: &Path| write_state(root, "executing", true);
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let node = run_node(
        &node_seed.root,
        &apply_patch_payload(&node_seed.root, "apply_patch", "not a recognizable patch envelope"),
    );
    let rust = run_rust(
        &rust_seed.root,
        &apply_patch_payload(&rust_seed.root, "apply_patch", "not a recognizable patch envelope"),
    );
    assert_allow_conformant("apply_patch/malformed-envelope-allow", &node, &rust);
}

#[test]
fn apply_patch_alternate_tool_name_and_move_to_target_denies_when_unprovable() {
    // ApplyPatch (alternate runtime spelling) + a "Move to:" destination
    // line — both recognized shapes.
    let setup = |root: &Path| write_state(root, "executing", true);
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), setup);
    let patch = "*** Begin Patch\n*** Update File: src/old.js\n*** Move to: ../escape.js\n*** End Patch\n";
    let node = run_node(&node_seed.root, &apply_patch_payload(&node_seed.root, "ApplyPatch", patch));
    let rust = run_rust(&rust_seed.root, &apply_patch_payload(&rust_seed.root, "ApplyPatch", patch));
    assert_deny_conformant(
        "apply_patch/ApplyPatch-move-to-deny",
        &node,
        &rust,
        "bee apply_patch guard: this patch's target set could not be fully proved inside the repo",
    );
}

// ---------------------------------------------------------------------------
// Fixture class: AskUserQuestion schema guard — question count, option
// count, missing label/description all deny; a well-formed call allows
// ---------------------------------------------------------------------------

#[test]
fn ask_user_question_count_out_of_range_denies_and_allow_twin() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let questions = json!([]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_deny_conformant(
        "ask/count-deny(0)",
        &node,
        &rust,
        "bee AskUserQuestion guard: 0 question(s) — the tool takes 1–4 per call. Split into separate calls.",
    );

    // Twin: SAME shape, one well-formed question.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let questions = json!([well_formed_question("Pick one")]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_allow_conformant("ask/count-allow-twin(1 question)", &node, &rust);
}

#[test]
fn ask_user_question_option_count_out_of_range_denies() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let mut q = well_formed_question("Pick one");
    q["options"] = json!([well_formed_option("Only one", "desc")]);
    let questions = json!([q]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_deny_conformant(
        "ask/option-count-deny(1)",
        &node,
        &rust,
        "bee AskUserQuestion guard: 1 option(s) — each question needs 2–4 options",
    );
}

#[test]
fn ask_user_question_missing_label_denies() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let mut q = well_formed_question("Pick one");
    q["options"] = json!([{ "label": "", "description": "desc" }, well_formed_option("Option B", "desc B")]);
    let questions = json!([q]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_deny_conformant(
        "ask/missing-label-deny",
        &node,
        &rust,
        "bee AskUserQuestion guard: option 1 is missing a non-empty \"label\".",
    );
}

#[test]
fn ask_user_question_missing_description_denies() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let mut q = well_formed_question("Pick one");
    q["options"] = json!([{ "label": "Option A", "description": "" }, well_formed_option("Option B", "desc B")]);
    let questions = json!([q]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_deny_conformant(
        "ask/missing-description-deny",
        &node,
        &rust,
        "bee AskUserQuestion guard: option \"Option A\" is missing a non-empty \"description\".",
    );
}

#[test]
fn ask_user_question_well_formed_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let questions = json!([well_formed_question("Pick one")]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_allow_conformant("ask/well-formed-allow", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: ask-guard-autofix — an over-long header alone is REPAIRED
// (allow + PreToolUse updatedInput notice), never denied
// ---------------------------------------------------------------------------

#[test]
fn ask_user_question_autofix_truncates_long_header_and_allows_with_updated_input() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |_| {});
    let questions = json!([well_formed_question("This header is definitely too long")]);
    let node = run_node(&node_seed.root, &ask_payload(&node_seed.root, questions.clone()));
    let rust = run_rust(&rust_seed.root, &ask_payload(&rust_seed.root, questions));
    assert_allow_notice_json_conformant("ask/autofix-header", &node, &rust);

    let node_json: Value = serde_json::from_str(node.stdout.trim()).expect("node stdout is JSON");
    let new_header =
        node_json["hookSpecificOutput"]["updatedInput"]["questions"][0]["header"].as_str().unwrap_or("");
    // "This header" is 11 chars, trim_end() is a no-op (no trailing
    // whitespace), plus the U+2026 ellipsis = 12 chars total.
    assert_eq!(new_header, "This header\u{2026}", "unexpected truncated header — got {new_header:?}");
    assert_eq!(
        node_json["hookSpecificOutput"]["permissionDecision"].as_str(),
        Some("allow"),
        "autofix must allow, never deny"
    );
}

// ---------------------------------------------------------------------------
// Fixture class: crash fail-open — an internal fault exits 0 (never a
// denial) plus a hooks.jsonl crash line
// ---------------------------------------------------------------------------

#[test]
fn crash_fail_open_node_oracle_exits_zero_and_logs_crash_instead_of_denying() {
    // The seeded root is shaped so a HEALTHY run would DENY (a privacy read)
    // — proving the exit 0 below is the fail-open contract engaging, not a
    // quietly-allowing fixture. guards.mjs is a downstream dependency
    // (fault-injection input), not part of the diffed oracle surface; the
    // hook + adapter files stay sha256-pristine.
    let seeded = seed_root_with_config(enabling_config());
    let healthy = run_node(&seeded.root, &read_payload(&seeded.root, "Read", ".env", &[]));
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

    let node = run_node(&seeded.root, &read_payload(&seeded.root, "Read", ".env", &[]));
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
    // The compiled port's crash class is a genuine panic inside the decision
    // region — the SAME fail-open boundary run() wraps every decision in.
    // Proven directly against the public wrapper with a real panic: exit
    // code 0 and a hooks.jsonl crash line, never a deny.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let code = queen_bee::hooks::write_guard::run_fail_open(Some(root), Some("repo"), || {
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
