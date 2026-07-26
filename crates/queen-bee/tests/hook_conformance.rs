//! hook_conformance — D7b conformance rig (rust-port-7): proves
//! `queen-bee hook tools-logger` / `queen-bee hook codex-subagent-audit`
//! match the REAL `.bee/bin/hooks/bee-tools-logger.mjs` /
//! `bee-codex-subagent-audit.mjs` (run through the shared, frozen
//! `adapter.mjs`) — never a reimplementation guess.
//!
//! RIG DISCIPLINE (cell rust-port-7, inherited by every corpus later cells
//! build on this rig):
//!
//! (i) SEEDING — every temp root used to run the NODE side gets a fresh copy
//!     of `.bee/bin/lib/`, `.bee/bin/hooks/`, `.bee/onboarding.json`, and an
//!     enabling `config.json`. The node oracle always runs the SEEDED copy
//!     of the hook file (never the live repo file directly, except in the
//!     negative-control fixture, which has nothing to seed by design) so
//!     the sha256 check below proves the oracle itself is genuine.
//! (ii) NON-TRIVIALITY BOTH WAYS — every "deny" fixture (suppressed
//!      write/no audit record) is paired with an "allow" twin in the SAME
//!      shape with exactly one field flipped, so a mis-seeded root can
//!      never read as a genuine allow either way.
//! (iii) NEGATIVE CONTROL — an unseeded root must be DETECTED as invalid by
//!       the rig's own verifier, not just observed to behave differently.
//! (iv) Fixture names are descriptive test-function names (cargo test's own
//!      output lists them per class) so cap evidence can quote class
//!      coverage, not just a passed count.
//!
//! Every temp root below comes from `tempfile::tempdir()` — never the
//! repo's live `.bee/` store.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    panic!(
        "could not locate .bee/bin/hooks/adapter.mjs walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn queen_bee_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_queen-bee"))
}

fn adapter_oracle_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/adapter_encoding_oracle.mjs")
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

/// The oracle surface: files whose seeded copy must sha256-match the repo
/// source, every time. A doctored copy of any of these is a rig failure.
const SHA_CHECKED: &[&str] = &[
    "bin/hooks/adapter.mjs",
    "bin/hooks/bee-tools-logger.mjs",
    "bin/hooks/bee-codex-subagent-audit.mjs",
    "bin/lib/state.mjs",
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
    fs::write(
        root.join(".bee/config.json"),
        serde_json::to_string_pretty(&json!({ "hooks": hooks_config })).expect("serialize config.json"),
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

/// Two independently seeded, identically configured roots — node runs
/// against one, the rust port against the other, so appends from either
/// side can never land in the same file.
fn seed_pair(hooks_config: Value) -> (SeededRoot, SeededRoot) {
    (seed_root(hooks_config.clone()), seed_root(hooks_config))
}

/// A root seeded like [`seed_root`], except `.bee/bin/lib/state.mjs` is
/// replaced with a deliberately-throwing stub — the crash-fail-open fault
/// injection point. `state.mjs` is a downstream DEPENDENCY tools-logger
/// consults, not the oracle unit under test; breaking it is an INPUT
/// condition (exactly like malformed stdin), never a doctored copy of the
/// hook/adapter files actually being diffed — those are still sha256
/// checked below.
fn seed_crash_root() -> SeededRoot {
    let repo = repo_root();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();

    copy_dir_all(&repo.join(".bee/bin/lib"), &root.join(".bee/bin/lib"));
    copy_dir_all(&repo.join(".bee/bin/hooks"), &root.join(".bee/bin/hooks"));
    fs::copy(repo.join(".bee/onboarding.json"), root.join(".bee/onboarding.json")).expect("copy onboarding.json");
    fs::write(
        root.join(".bee/config.json"),
        serde_json::to_string_pretty(&json!({ "hooks": { "tools-logger": true, "codex-subagent-audit": true } }))
            .unwrap(),
    )
    .expect("write config.json");

    fs::write(
        root.join(".bee/bin/lib/state.mjs"),
        b"export function hookEnabled() { throw new Error('rig-injected-fault: state.mjs unavailable'); }\n",
    )
    .expect("inject crash fault into state.mjs");

    // The oracle surface itself (adapter + the two hook files) must still
    // be pristine — only the fault-injection dependency deviates.
    for rel in [
        "bin/hooks/adapter.mjs",
        "bin/hooks/bee-tools-logger.mjs",
        "bin/hooks/bee-codex-subagent-audit.mjs",
    ] {
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

/// Runs the SEEDED copy of the hook file living inside `root` — the real
/// oracle per rig discipline (i).
fn run_seeded_node_hook(root: &Path, hook_file: &str, argv: &[&str], stdin: &str) -> RunResult {
    let mut cmd = Command::new("node");
    cmd.arg(root.join(".bee/bin/hooks").join(hook_file));
    cmd.args(argv);
    cmd.current_dir(root);
    run(cmd, stdin)
}

/// Runs the LIVE repo hook file directly — used only by the negative-control
/// fixture, which has nothing to seed by design.
fn run_live_node_hook(hook_file: &str, cwd: &Path, argv: &[&str], stdin: &str) -> RunResult {
    let repo = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(repo.join(".bee/bin/hooks").join(hook_file));
    cmd.args(argv);
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_rust_hook(name: &str, cwd: &Path, argv: &[&str], stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg(name);
    cmd.args(argv);
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_adapter_oracle(args: &[&str]) -> String {
    let out = Command::new("node")
        .arg(adapter_oracle_script())
        .args(args)
        .output()
        .expect("spawn adapter oracle");
    assert!(out.status.success(), "adapter oracle failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ---------------------------------------------------------------------------
// Normalization + the differ
// ---------------------------------------------------------------------------

fn redact_ts(mut v: Value) -> Value {
    if let Value::Object(ref mut m) = v {
        if m.contains_key("ts") {
            m.insert("ts".into(), Value::String("<ts>".into()));
        }
    }
    v
}

fn read_jsonl_normalized(path: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| redact_ts(serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}"))))
        .collect()
}

fn assert_process_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(
        node.status, rust.status,
        "{label}: exit code diverged (node={} rust={}) node_stderr={:?} rust_stderr={:?}",
        node.status, rust.status, node.stderr, rust.stderr
    );
    assert_eq!(
        node.stdout.trim(),
        rust.stdout.trim(),
        "{label}: stdout diverged (node={:?} rust={:?})",
        node.stdout,
        rust.stdout
    );
}

fn assert_jsonl_conformant(label: &str, node_lines: &[Value], rust_lines: &[Value]) {
    assert_eq!(
        node_lines.len(),
        rust_lines.len(),
        "{label}: jsonl line count diverged — node={node_lines:?} rust={rust_lines:?}"
    );
    for (i, (n, r)) in node_lines.iter().zip(rust_lines.iter()).enumerate() {
        assert_eq!(n, r, "{label}: jsonl line {i} diverged — node={n} rust={r}");
    }
}

fn enabling_config() -> Value {
    json!({ "tools-logger": true, "codex-subagent-audit": true })
}

// ---------------------------------------------------------------------------
// Fixture class: rig self-check (sha256)
// ---------------------------------------------------------------------------

#[test]
fn rig_self_check_seeded_files_match_repo_sha256() {
    let seeded = seed_root(enabling_config());
    assert!(is_seeded_valid(&seeded.root), "a freshly seeded root must verify as valid");
}

// ---------------------------------------------------------------------------
// Fixture class: negative-control unseeded root
// ---------------------------------------------------------------------------

#[test]
fn negative_control_unseeded_root_detected_as_invalid_by_verifier() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!is_seeded_valid(dir.path()), "an unseeded root must never verify as a valid rig setup");
}

#[test]
fn negative_control_unseeded_root_both_runtimes_exit_zero_silently() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cwd = dir.path();
    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash"}"#;

    let node = run_live_node_hook("bee-tools-logger.mjs", cwd, &[], stdin);
    let rust = run_rust_hook("tools-logger", cwd, &[], stdin);
    assert_process_conformant("negative-control/tools-logger", &node, &rust);
    assert_eq!(node.status, 0, "no discoverable root must still be exit 0 (fail-open)");
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
    assert!(!cwd.join(".bee").exists(), "no root discoverable => no .bee dir should ever be created");
}

// ---------------------------------------------------------------------------
// Fixture class: disabled-hook + allow-with-twin-deny pairing (tools-logger)
// ---------------------------------------------------------------------------

#[test]
fn disabled_hook_tools_logger_suppresses_write_and_enabled_twin_writes_matches_oracle() {
    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash","agent_id":"a1","agent_type":"worker"}"#;

    // Deny twin: hooks.tools-logger = false.
    let (deny_node, deny_rust) = seed_pair(json!({ "tools-logger": false, "codex-subagent-audit": true }));
    let node_off = run_seeded_node_hook(&deny_node.root, "bee-tools-logger.mjs", &[], stdin);
    let rust_off = run_rust_hook("tools-logger", &deny_rust.root, &[], stdin);
    assert_process_conformant("disabled-hook/tools-logger", &node_off, &rust_off);
    assert!(!deny_node.root.join(".bee/logs/tools.jsonl").exists(), "node oracle must not write when disabled");
    assert!(!deny_rust.root.join(".bee/logs/tools.jsonl").exists(), "rust port must not write when disabled");

    // Allow twin: SAME shape, only `tools-logger` flipped true — proves the
    // deny result above was a genuine gate, not a mis-seeded root that can
    // never produce output either way.
    let (allow_node, allow_rust) = seed_pair(json!({ "tools-logger": true, "codex-subagent-audit": true }));
    let node_on = run_seeded_node_hook(&allow_node.root, "bee-tools-logger.mjs", &[], stdin);
    let rust_on = run_rust_hook("tools-logger", &allow_rust.root, &[], stdin);
    assert_process_conformant("enabled-hook/tools-logger", &node_on, &rust_on);

    let node_lines = read_jsonl_normalized(&allow_node.root.join(".bee/logs/tools.jsonl"));
    let rust_lines = read_jsonl_normalized(&allow_rust.root.join(".bee/logs/tools.jsonl"));
    assert_eq!(node_lines.len(), 1, "node oracle: enabled hook must write exactly one entry");
    assert_jsonl_conformant("enabled-hook/tools-logger", &node_lines, &rust_lines);
    assert_eq!(rust_lines[0]["tool_name"], json!("Bash"));
    assert_eq!(rust_lines[0]["agent_id"], json!("a1"));
    assert_eq!(rust_lines[0]["agent_type"], json!("worker"));
}

// ---------------------------------------------------------------------------
// Fixture class: disabled-hook (parity, non-gated) + allow-with-twin-deny
// pairing on event name (codex-subagent-audit)
// ---------------------------------------------------------------------------

#[test]
fn codex_subagent_audit_ignores_hook_toggle_and_gates_on_event_name_matches_oracle() {
    let start_stdin =
        r#"{"hook_event_name":"SubagentStart","session_id":"s1","agent_id":"a1","agent_name":"Kevin","agent_type":"worker"}"#;
    let unsupported_stdin = r#"{"hook_event_name":"SomethingElse","session_id":"s1"}"#;

    // "disabled" config is a parity check here, not a gate: the frozen
    // source never consults the hooks toggle for this hook — both runtimes
    // must still log.
    let (allow_node, allow_rust) = seed_pair(json!({ "codex-subagent-audit": false }));
    let node_on = run_seeded_node_hook(&allow_node.root, "bee-codex-subagent-audit.mjs", &[], start_stdin);
    let rust_on = run_rust_hook("codex-subagent-audit", &allow_rust.root, &[], start_stdin);
    assert_process_conformant("disabled-hook(parity)/codex-subagent-audit", &node_on, &rust_on);
    let node_lines = read_jsonl_normalized(&allow_node.root.join(".bee/logs/hooks.jsonl"));
    let rust_lines = read_jsonl_normalized(&allow_rust.root.join(".bee/logs/hooks.jsonl"));
    assert_eq!(
        node_lines.len(),
        1,
        "node oracle: codex-subagent-audit is not gated by the hooks toggle — it must still log"
    );
    assert_jsonl_conformant("disabled-hook(parity)/codex-subagent-audit", &node_lines, &rust_lines);
    assert_eq!(node_lines[0]["lifecycle"], json!("start"));
    assert_eq!(node_lines[0]["agent_name"], json!("Kevin"));

    // The real allow/deny pairing for this hook: SubagentStart (allow) vs an
    // unsupported event name (deny -> coverage gap only, no audit record).
    let (deny_node, deny_rust) = seed_pair(json!({ "codex-subagent-audit": true }));
    let node_off = run_seeded_node_hook(&deny_node.root, "bee-codex-subagent-audit.mjs", &[], unsupported_stdin);
    let rust_off = run_rust_hook("codex-subagent-audit", &deny_rust.root, &[], unsupported_stdin);
    assert_process_conformant("unsupported-event/codex-subagent-audit", &node_off, &rust_off);
    let node_gap_lines = read_jsonl_normalized(&deny_node.root.join(".bee/logs/hooks.jsonl"));
    let rust_gap_lines = read_jsonl_normalized(&deny_rust.root.join(".bee/logs/hooks.jsonl"));
    assert_eq!(node_gap_lines.len(), 1, "unsupported event must produce exactly one coverage-gap line");
    assert_eq!(node_gap_lines[0]["gap"], json!("unsupported-subagent-event"));
    assert_jsonl_conformant("unsupported-event/codex-subagent-audit", &node_gap_lines, &rust_gap_lines);
}

// ---------------------------------------------------------------------------
// Fixture class: malformed stdin (table-driven, both hooks)
// ---------------------------------------------------------------------------

#[test]
fn malformed_stdin_matches_oracle_for_both_hooks() {
    let cases: &[(&str, &str)] = &[
        ("empty_stdin", ""),
        ("plain_garbage", "not json at all"),
        ("top_level_array", "[1,2,3]"),
        ("top_level_null", "null"),
        ("top_level_number", "42"),
    ];

    for hook in ["tools-logger", "codex-subagent-audit"] {
        let hook_file = format!("bee-{hook}.mjs");
        for (name, stdin) in cases {
            let (node_seed, rust_seed) = seed_pair(enabling_config());
            let node = run_seeded_node_hook(&node_seed.root, &hook_file, &[], stdin);
            let rust = run_rust_hook(hook, &rust_seed.root, &[], stdin);
            let label = format!("malformed-stdin/{hook}/{name}");
            assert_process_conformant(&label, &node, &rust);

            let node_gap_lines = read_jsonl_normalized(&node_seed.root.join(".bee/logs/hooks.jsonl"));
            let rust_gap_lines = read_jsonl_normalized(&rust_seed.root.join(".bee/logs/hooks.jsonl"));
            assert_jsonl_conformant(&label, &node_gap_lines, &rust_gap_lines);
            if !stdin.trim().is_empty() {
                assert!(
                    node_gap_lines.iter().any(|l| l["gap"] == json!("malformed-payload")),
                    "{label}: expected a malformed-payload coverage-gap line from the oracle"
                );
            }

            if hook == "tools-logger" {
                let node_tool_lines = read_jsonl_normalized(&node_seed.root.join(".bee/logs/tools.jsonl"));
                let rust_tool_lines = read_jsonl_normalized(&rust_seed.root.join(".bee/logs/tools.jsonl"));
                assert_jsonl_conformant(&format!("{label}/tools.jsonl"), &node_tool_lines, &rust_tool_lines);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture class: invalid-source coverage-gap (table-driven, both argv
// forms, both hooks)
// ---------------------------------------------------------------------------

#[test]
fn source_identity_argv_forms_matches_oracle_for_both_hooks() {
    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash"}"#;
    let cases: &[(&str, &[&str], bool)] = &[
        ("space_form_invalid", &["--source", "bogus"], true),
        ("equals_form_invalid", &["--source=bogus"], true),
        ("space_form_valid_plugin", &["--source", "plugin"], false),
        ("equals_form_valid_repo", &["--source=repo"], false),
        ("space_form_missing_value", &["--source"], true),
    ];

    for hook in ["tools-logger", "codex-subagent-audit"] {
        let hook_file = format!("bee-{hook}.mjs");
        for (name, argv, expect_gap) in cases {
            let (node_seed, rust_seed) = seed_pair(enabling_config());
            let node = run_seeded_node_hook(&node_seed.root, &hook_file, argv, stdin);
            let rust = run_rust_hook(hook, &rust_seed.root, argv, stdin);
            let label = format!("source-identity/{hook}/{name}");
            assert_process_conformant(&label, &node, &rust);

            let node_lines = read_jsonl_normalized(&node_seed.root.join(".bee/logs/hooks.jsonl"));
            let rust_lines = read_jsonl_normalized(&rust_seed.root.join(".bee/logs/hooks.jsonl"));
            assert_jsonl_conformant(&label, &node_lines, &rust_lines);

            let has_gap = node_lines.iter().any(|l| l["gap"] == json!("invalid-source"));
            assert_eq!(has_gap, *expect_gap, "{label}: invalid-source gap presence diverged from expectation");
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture class: advisory-vs-context output encoding (shared adapter)
// ---------------------------------------------------------------------------

#[test]
fn advisory_vs_context_encoding_matches_adapter_oracle() {
    for event in ["PreCompact", "SubagentStop", "Stop", "SessionStart", "UserPromptSubmit", "", "Weird"] {
        let oracle: Value = serde_json::from_str(&run_adapter_oracle(&["is-advisory", event])).unwrap();
        let oracle_result = oracle["result"].as_bool().unwrap();
        assert_eq!(
            queen_bee::adapter::is_advisory_event(event),
            oracle_result,
            "is_advisory_event diverged for event={event:?}"
        );
    }

    for (event, text) in [
        ("PreCompact", "advisory context text"),
        ("Stop", "advisory block-shaped text"),
        ("SubagentStop", "child advisory"),
        ("SessionStart", "plain developer context"),
        ("UserPromptSubmit", "more plain context"),
        ("SessionStart", "   "),
    ] {
        let oracle_out = run_adapter_oracle(&["emit", event, text]);
        let oracle_val: Value = serde_json::from_str(&oracle_out).unwrap();
        let expected = oracle_val["out"].as_str().unwrap_or("");
        let rust_out = queen_bee::adapter::emit_hook_output(event, text).unwrap_or_default();
        assert_eq!(rust_out, expected, "emit_hook_output diverged for event={event:?} text={text:?}");
    }

    let block_oracle: Value = serde_json::from_str(&run_adapter_oracle(&["encode-block", "denied for reasons"])).unwrap();
    let block_rust: Value = serde_json::from_str(&queen_bee::adapter::encode_block("denied for reasons")).unwrap();
    assert_eq!(block_oracle, block_rust, "encode_block diverged from the adapter oracle");

    let advisory_oracle: Value = serde_json::from_str(&run_adapter_oracle(&["encode-advisory", "hello there"])).unwrap();
    let advisory_rust: Value = serde_json::from_str(&queen_bee::adapter::encode_advisory("hello there")).unwrap();
    assert_eq!(advisory_oracle, advisory_rust, "encode_advisory diverged from the adapter oracle");
}

// ---------------------------------------------------------------------------
// Fixture class: crash fail-open
// ---------------------------------------------------------------------------

#[test]
fn crash_fail_open_tools_logger_node_oracle_stays_exit_zero_and_logs_crash() {
    let seeded = seed_crash_root();
    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash"}"#;
    let node = run_seeded_node_hook(&seeded.root, "bee-tools-logger.mjs", &[], stdin);
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must still exit 0 — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty(), "a crash must never leak to stdout");
    assert!(
        !seeded.root.join(".bee/logs/tools.jsonl").exists(),
        "a crashed write must never produce a partial/successful tools.jsonl entry"
    );
    let lines = read_jsonl_normalized(&seeded.root.join(".bee/logs/hooks.jsonl"));
    assert!(
        lines
            .iter()
            .any(|l| l.get("error").and_then(Value::as_str).map(|s| s.contains("rig-injected-fault")).unwrap_or(false)),
        "expected a crash log line naming the injected fault, got {lines:?}"
    );
}

#[test]
fn crash_fail_open_tools_logger_rust_port_stays_exit_zero_when_logs_path_is_blocked() {
    // Cross-runtime identity isn't the claim here — node's dynamic-import
    // failure surface and a compiled Rust write have genuinely different
    // fault shapes. What must hold on BOTH sides independently is the
    // fail-open CONTRACT: an internal fault never flips the exit code or a
    // decision. The node oracle proves its half above; this proves the Rust
    // port's own `log_crash`/fail-open wrapping engages under a Rust-native
    // fault (`.bee/logs` occupied by a plain file, so `create_dir_all`
    // fails for both the intended write AND the crash-log write itself —
    // matching the mjs source's own "a log write failure never changes the
    // hook's decision" guarantee).
    let seeded = seed_root(enabling_config());
    fs::write(seeded.root.join(".bee/logs"), b"not a directory").expect("occupy .bee/logs with a file");

    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash"}"#;
    let rust = run_rust_hook("tools-logger", &seeded.root, &[], stdin);
    assert_eq!(rust.status, 0, "fail-open contract: an internal fault must still exit 0 — stderr={:?}", rust.stderr);
    assert!(rust.stdout.trim().is_empty());
}

// ---------------------------------------------------------------------------
// Red-first meta-proof: the differ itself must have teeth.
// ---------------------------------------------------------------------------

#[test]
fn rig_diff_detector_flags_a_seeded_mutation_as_divergent() {
    let stdin = r#"{"hook_event_name":"PostToolUse","tool_name":"Bash"}"#;
    let node_seed = seed_root(json!({ "tools-logger": true, "codex-subagent-audit": true }));
    let rust_seed = seed_root(json!({ "tools-logger": false, "codex-subagent-audit": true }));

    let _node = run_seeded_node_hook(&node_seed.root, "bee-tools-logger.mjs", &[], stdin);
    let _rust = run_rust_hook("tools-logger", &rust_seed.root, &[], stdin);

    let node_lines = read_jsonl_normalized(&node_seed.root.join(".bee/logs/tools.jsonl"));
    let rust_lines = read_jsonl_normalized(&rust_seed.root.join(".bee/logs/tools.jsonl"));

    let result = std::panic::catch_unwind(|| assert_jsonl_conformant("deliberately-mutated", &node_lines, &rust_lines));
    assert!(result.is_err(), "the differ failed to flag a deliberately seeded divergence — the rig has no teeth");
}
