//! status_readers_b2 — the single integration target for rust-port-20's
//! status-read spine panel B2 (CONTEXT.md D3/D5): recovery/contention
//! (`read_transcript_tail`'s bounded-window semantics are unit-tested
//! inline in `src/recovery.rs`; this target proves the higher-level
//! `build_recovery_block`/`build_contention_summary`), the READERS PANEL
//! B2 found with no home (`build_lane_rows`/`build_lane_summary`,
//! `compute_runtime_drift`, `find_repo_hive`, `ungranted_worktree_notice`),
//! and the remaining state helpers (`is_known_phase`,
//! `validate_models_config`, `validate_agent_files_drift`,
//! `has_stale_advisor_key`, `classify_source`, `read_onboarding`,
//! `read_handoff`, `bypass_level`). Run via `cargo test --manifest-path
//! crates/Cargo.toml -p bee-core --test status_readers_b2` (this cell's
//! own `verify`).
//!
//! ORACLE DISCIPLINE (cell must-have) — TWO SPLITS, per this cell's own
//! action field:
//!   - importable units get a FINE-GRAINED file-based node driver
//!     (`tests/support/status_readers_b2_oracle.mjs`, same pattern as
//!     `status_readers_a_oracle.mjs`/`status_readers_b1_oracle.mjs`) that
//!     calls the REAL, FROZEN mjs function directly;
//!   - the seven `bee.mjs`-PRIVATE helpers (`buildRecoveryBlock`,
//!     `buildContentionSummary`, `buildLaneRows`/`buildLaneSummary`,
//!     `computeRuntimeDrift`, `findRepoHive`, `ungrantedWorktreeNotice`)
//!     have no importable home — they are oracled through the
//!     WHOLE-COMMAND driver, `node .bee/bin/bee.mjs status --json` run as
//!     a real subprocess with cwd at the fixture root, diffed sub-object
//!     by sub-object. Never a hand-invented reimplementation of those
//!     seven functions in the oracle script.
//!
//! Every path used below is a fresh `tempfile::tempdir()` — never this
//! repo's own live `.bee/` store (prohibition: "tests never touch the
//! live .bee/ store"), and the injected transcript root
//! (`.bee/config.json` `recovery.transcript_roots`) is always fixture-
//! local — never the host `$HOME/.claude/projects` (D5).
//!
//! NON-DETERMINISM DISCIPLINE (cell must-have #4): every clock-sensitive
//! fixture below pins `now_ms` to a literal constant rather than
//! `SystemTime::now()`/`Date.now()` — heartbeat staleness (claims.mjs:367)
//! and reservation expiry (bee.mjs:741-746) are both boundary-tested this
//! way. The reservation-reference-identity fixture
//! (`reservation_reference_identity_bug_is_replicated_not_fixed`) is the
//! one this must-have names explicitly.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

use bee_core::{claims, recovery, reservations, source_identity, state};

// ─── oracle driver plumbing ─────────────────────────────────────────────

fn oracle_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/status_readers_b2_oracle.mjs")
}

/// Runs `status_readers_b2_oracle.mjs <op> [root]` with `stdin` (if any)
/// JSON-encoded on stdin, and parses stdout as JSON. Panics loudly on a
/// non-zero exit or unparseable stdout — an oracle failure is never
/// swallowed into a false pass.
fn run_oracle(op: &str, root: Option<&Path>, stdin: Option<Value>) -> Value {
    let mut cmd = Command::new("node");
    cmd.arg(oracle_script()).arg(op);
    if let Some(r) = root {
        cmd.arg(r);
    }
    // This harness itself typically runs inside an active Claude Code
    // session (`CLAUDE_CODE_SESSION_ID`/`BEE_SESSION_ID` set in ITS OWN
    // process env) — the real `resolveSessionId` the "status" whole-command
    // op's subprocess calls checks those env vars BEFORE its root-inference
    // branch, so an inherited value would silently resolve to a session
    // that doesn't exist in the fixture rather than the single fresh
    // session a test wrote there. Cleared so every oracle run resolves
    // deterministically from the fixture's OWN on-disk state, matching the
    // Rust side's caller-supplied `resolved_session_id` parameter (D5).
    cmd.env_remove("BEE_SESSION_ID").env_remove("CLAUDE_CODE_SESSION_ID");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn node status_readers_b2_oracle driver — is `node` on PATH?");
    {
        let mut si = child.stdin.take().expect("child stdin");
        let payload = stdin.map(|v| v.to_string()).unwrap_or_default();
        si.write_all(payload.as_bytes()).expect("write oracle stdin");
    }
    let output = child.wait_with_output().expect("wait for status_readers_b2_oracle driver");
    assert!(
        output.status.success(),
        "oracle op {op} exited non-zero — stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("oracle op {op}: could not parse stdout {stdout:?}: {e}"))
}

// ─── fixture helpers ─────────────────────────────────────────────────────

fn write_file(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn bee_dir(root: &Path) -> PathBuf {
    root.join(".bee")
}

/// The minimal store that lets the REAL `bee.mjs` resolve `root` as its
/// own bee root without needing a git repo at all: `resolveRootsCore`'s
/// first branch matches on `.bee/onboarding.json` present AND no `.git`
/// at the same level — empirically confirmed against the real CLI before
/// writing this fixture helper.
fn write_minimal_onboarding(root: &Path) {
    write_file(&bee_dir(root).join("onboarding.json"), "{}");
}

fn write_lease(root: &Path, name: &str, body: &Value) {
    write_file(&root.join(".bee").join("runtime").join("leases").join("paths").join(format!("{name}.json")), &body.to_string());
}

fn write_session(root: &Path, id: &str, body: &Value) {
    write_file(&bee_dir(root).join("sessions").join(format!("{id}.json")), &body.to_string());
}

fn write_lane(root: &Path, feature: &str, body: &Value) {
    write_file(&bee_dir(root).join("lanes").join(format!("{feature}.json")), &body.to_string());
}

/// Sorts a JSON array of objects by their `path` key so ordering
/// differences between two independent directory listings (Rust
/// `fs::read_dir` vs Node `fs.readdirSync`, both unsorted-by-contract)
/// never cause a false failure — content is what this oracle proves,
/// never incidental OS directory order.
fn sorted_by_path(v: Value) -> Vec<Value> {
    let mut arr = v.as_array().cloned().unwrap_or_default();
    arr.sort_by(|a, b| {
        let ap = a.get("path").and_then(Value::as_str).unwrap_or("");
        let bp = b.get("path").and_then(Value::as_str).unwrap_or("");
        ap.cmp(bp)
    });
    arr
}

// ─── host-real fixture (queen-bench --generate, prebuilt binary as-is) ─────
// Same discipline as status_readers_a.rs (rust-port-13)/status_readers_b1.rs
// (rust-port-14): the CURRENT compiled queen-bench binary is used as a
// fixed fixture-generation artifact, never rebuilt or edited from here.

fn queen_bench_bin() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        let candidate =
            dir.join("crates").join("target").join("debug").join(if cfg!(windows) { "queen-bench.exe" } else { "queen-bench" });
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "status_readers_b2: could not locate a prebuilt queen-bench binary walking ancestors from {} \
         — this cell uses the CURRENT compiled queen-bench binary as a fixed fixture-generation artifact \
         rather than rebuilding it itself; build the workspace once \
         (`cargo build --manifest-path crates/Cargo.toml -p queen-bench`) if it is genuinely missing.",
        env!("CARGO_MANIFEST_DIR")
    );
}

/// Generates a host-real-shaped `.bee` store (rust-port-19: real git
/// ancestry, review candidates, and ONE heartbeat-stale crash-candidate
/// session with an injected — never host `$HOME` — transcript root) via
/// the prebuilt `queen-bench --generate` binary. Panics loudly on a
/// generation failure.
fn host_real_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(queen_bench_bin()).arg("--generate").arg("--out").arg(dir.path()).output().expect("failed to spawn queen-bench --generate");
    assert!(output.status.success(), "queen-bench --generate failed — stderr: {}", String::from_utf8_lossy(&output.stderr));
    dir
}

// ═══════════════════════════════════════════════════════════════════════
// is_known_phase
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn is_known_phase_matches_mjs_across_every_named_and_unknown_phase() {
    let cases = ["idle", "exploring", "planning", "validating", "swarming", "reviewing", "scribing", "compounding", "grooming", "compounding-complete", "bogus-phase", ""];
    for phase in cases {
        let rust_result = state::is_known_phase(phase);
        let mjs_result = run_oracle("is-known-phase", None, Some(json!({"phase": phase})));
        assert_eq!(json!(rust_result), mjs_result, "phase {phase:?} diverged from mjs isKnownPhase");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// validate_models_config — table-driven over every problem code (test-
// economy D3: a new scenario for behavior the suite already covers is a
// new ROW here, never a new file).
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn validate_models_config_matches_mjs_across_every_problem_code() {
    let cases: Vec<(&str, Value)> = vec![
        ("no-models-key", json!({})),
        ("valid-model-string", json!({"models": {"claude": {"generation": "sonnet"}}})),
        ("runtime-malformed", json!({"models": {"claude": "nope"}})),
        ("slot-value-malformed", json!({"models": {"claude": {"generation": 5}}})),
        ("composite-primary-malformed", json!({"models": {"claude": {"generation": {"primary": {"kind": "native"}}}}})),
        (
            "composite-fallback-policy-missing",
            json!({"models": {"claude": {"generation": {"primary": {"kind": "native", "model": "sonnet"}}}}}),
        ),
        (
            "composite-fork-turns-unknown-and-fallback-malformed",
            json!({"models": {"claude": {"generation": {
                "primary": {"kind": "native", "model": "sonnet", "fork_turns": "bogus"},
                "fallback_policy": "explicit-only",
                "fallback": {"kind": "not-cli"},
            }}}}),
        ),
        ("native-model-missing", json!({"models": {"claude": {"generation": {"kind": "native"}}}})),
        ("native-fork-turns-unknown", json!({"models": {"claude": {"generation": {"kind": "native", "model": "sonnet", "fork_turns": "bogus"}}}})),
        ("cli-malformed-no-command", json!({"models": {"claude": {"generation": {"kind": "cli"}}}})),
        ("cli-prompt-transport-missing", json!({"models": {"claude": {"generation": {"kind": "cli", "command": "codex exec -"}}}})),
        (
            "cli-unsafe-flag",
            json!({"models": {"claude": {"generation": {"kind": "cli", "command": "codex exec --yolo -", "promptVia": "stdin"}}}}),
        ),
        (
            "cli-advice-slot-writable-double-fire",
            json!({"models": {"claude": {"review": {"kind": "cli", "command": "codex exec --sandbox danger-full-access -", "promptVia": "stdin"}}}}),
        ),
        ("model-shape-malformed", json!({"models": {"claude": {"generation": {}}}})),
    ];

    for (name, config) in cases {
        let raw = state::RawConfig::Value(config.clone());
        let rust_result = serde_json::to_value(state::validate_models_config(&raw)).unwrap();
        let mjs_result = run_oracle("validate-models-config", None, Some(json!({"present": true, "config": config})));
        assert_eq!(rust_result, mjs_result, "case {name} diverged from mjs validateModelsConfig");
    }
}

#[test]
fn validate_models_config_absent_file_is_zero_problems() {
    let rust_result = serde_json::to_value(state::validate_models_config(&state::RawConfig::Absent)).unwrap();
    let mjs_result = run_oracle("validate-models-config", None, Some(json!({"present": false})));
    assert_eq!(rust_result, json!([]));
    assert_eq!(rust_result, mjs_result);
}

#[test]
fn validate_models_config_malformed_content_reports_config_malformed() {
    for config in [Value::Null, json!("nope"), json!([1, 2])] {
        let raw = state::RawConfig::Value(config.clone());
        let rust_result = serde_json::to_value(state::validate_models_config(&raw)).unwrap();
        let mjs_result = run_oracle("validate-models-config", None, Some(json!({"present": true, "config": config})));
        assert_eq!(rust_result, mjs_result);
        assert_eq!(rust_result[0]["code"], json!("config-malformed"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// read_raw_config_for_validation — the cell action names this reader
// explicitly (readRawConfigForValidation, bee.mjs:600, "feeding both
// validators") and it is bee.mjs-PRIVATE (not exported by state.mjs), so
// it is oracle-diffed the WHOLE-COMMAND way: `readRawConfigForValidation`
// feeds `validateModelsConfig`, whose problems are rendered into
// `staleness_warnings` verbatim (bee.mjs:777-779) — a visible, diffable
// surface for the file->Absent / Value(null) / Value(object) mapping.
// ═══════════════════════════════════════════════════════════════════════

/// `config validate [${code}]${runtime ? ' models.RT.SLOT:' : ''} ${message}`
/// (bee.mjs:778) — reconstructed here so this test can recognize the
/// REAL CLI's own rendering of `validateModelsConfig(
/// readRawConfigForValidation(root))`'s output inside `staleness_warnings`,
/// never a second validation implementation.
fn config_validate_message(p: &state::ModelConfigProblem) -> String {
    let cond = match &p.runtime {
        Some(rt) => format!(" models.{}.{}:", rt, p.slot.as_deref().unwrap_or("")),
        None => String::new(),
    };
    format!("config validate [{}]{cond} {}", p.code, p.message)
}

#[test]
fn read_raw_config_for_validation_matches_mjs_absent_malformed_and_present() {
    // (1) Absent: no .bee/config.json at all -> readRawConfigForValidation
    // returns JS `undefined` -> validateModelsConfig reports ZERO problems
    // -> zero "config validate" lines in the real status output.
    let dir_absent = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir_absent.path());
    let raw_absent = state::read_raw_config_for_validation(dir_absent.path());
    assert!(matches!(raw_absent, state::RawConfig::Absent));
    assert_eq!(state::validate_models_config(&raw_absent), Vec::<state::ModelConfigProblem>::new());
    let mjs_absent = run_oracle("status", Some(dir_absent.path()), None);
    let warnings_absent = mjs_absent["staleness_warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        !warnings_absent.iter().any(|w| w.as_str().unwrap_or("").starts_with("config validate")),
        "an absent .bee/config.json must produce zero config-validate staleness lines"
    );

    // (2) Present but unparseable: readJson(file, null) falls back to
    // `null` on a JSON parse failure (fsutil.mjs) -> RawConfig::Value(Null)
    // -> ONE config-malformed problem -> ONE matching staleness line.
    let dir_malformed = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir_malformed.path());
    write_file(&bee_dir(dir_malformed.path()).join("config.json"), "not valid json at all {{{");
    let raw_malformed = state::read_raw_config_for_validation(dir_malformed.path());
    let problems_malformed = match &raw_malformed {
        state::RawConfig::Value(v) => {
            assert!(v.is_null(), "an unparseable config.json must read as RawConfig::Value(Null), matching readJson's null fallback");
            state::validate_models_config(&raw_malformed)
        }
        state::RawConfig::Absent => panic!("a PRESENT (if malformed) config.json must never read as Absent"),
    };
    assert_eq!(problems_malformed.len(), 1);
    assert_eq!(problems_malformed[0].code, "config-malformed");
    let mjs_malformed = run_oracle("status", Some(dir_malformed.path()), None);
    let warnings_malformed = mjs_malformed["staleness_warnings"].as_array().cloned().unwrap_or_default();
    let expected_malformed = config_validate_message(&problems_malformed[0]);
    assert!(
        warnings_malformed.iter().any(|w| w.as_str() == Some(expected_malformed.as_str())),
        "expected {expected_malformed:?} in the real mjs staleness_warnings, got: {warnings_malformed:?}"
    );

    // (3) Present, parseable, a normal object with a real models problem
    // -> RawConfig::Value(object) -> validateModelsConfig finds the SAME
    // problem set the mjs reader would, rendered into the SAME staleness
    // line.
    let dir_present = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir_present.path());
    let config_body = json!({"models": {"claude": {"generation": {"kind": "cli"}}}});
    write_file(&bee_dir(dir_present.path()).join("config.json"), &config_body.to_string());
    let raw_present = state::read_raw_config_for_validation(dir_present.path());
    let problems_present = match &raw_present {
        state::RawConfig::Value(v) => {
            assert_eq!(v, &config_body, "a present, parseable config.json must read back byte-identical to what was written");
            state::validate_models_config(&raw_present)
        }
        state::RawConfig::Absent => panic!("a present config.json must never read as Absent"),
    };
    assert_eq!(problems_present.len(), 1);
    assert_eq!(problems_present[0].code, "cli-malformed");
    let mjs_present = run_oracle("status", Some(dir_present.path()), None);
    let warnings_present = mjs_present["staleness_warnings"].as_array().cloned().unwrap_or_default();
    let expected_present = config_validate_message(&problems_present[0]);
    assert!(
        warnings_present.iter().any(|w| w.as_str() == Some(expected_present.as_str())),
        "expected {expected_present:?} in the real mjs staleness_warnings, got: {warnings_present:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// validate_agent_files_drift
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn validate_agent_files_drift_absent_file_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let raw = state::RawConfig::Value(json!({}));
    let rust_result = serde_json::to_value(state::validate_agent_files_drift(dir.path(), &raw)).unwrap();
    let mjs_result = run_oracle("validate-agent-files-drift", Some(dir.path()), Some(json!({"present": true, "rawConfig": {}})));
    assert_eq!(rust_result, mjs_result);
    assert_eq!(rust_result, json!([]));
}

#[test]
fn validate_agent_files_drift_malformed_frontmatter_is_agent_file_malformed() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir.path().join(".claude").join("agents").join("bee-gather.md"), "no frontmatter here at all");
    let raw = state::RawConfig::Value(json!({}));
    let rust_result = serde_json::to_value(state::validate_agent_files_drift(dir.path(), &raw)).unwrap();
    let mjs_result = run_oracle("validate-agent-files-drift", Some(dir.path()), Some(json!({"present": true, "rawConfig": {}})));
    assert_eq!(rust_result, mjs_result);
    assert_eq!(rust_result[0]["code"], json!("agent-file-malformed"));
    assert_eq!(rust_result[0]["agent"], json!("bee-gather"));
}

#[test]
fn validate_agent_files_drift_matching_model_is_clean_and_mismatch_drifts() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir.path().join(".claude").join("agents").join("bee-extract.md"), "---\nname: bee-extract\nmodel: haiku\n---\nbody text\n");
    let raw = state::RawConfig::Value(json!({"models": {"claude": {"extraction": "haiku"}}}));
    let rust_clean = serde_json::to_value(state::validate_agent_files_drift(dir.path(), &raw)).unwrap();
    let mjs_clean = run_oracle(
        "validate-agent-files-drift",
        Some(dir.path()),
        Some(json!({"present": true, "rawConfig": {"models": {"claude": {"extraction": "haiku"}}}})),
    );
    assert_eq!(rust_clean, mjs_clean);
    assert_eq!(rust_clean, json!([]));

    let raw_drift = state::RawConfig::Value(json!({"models": {"claude": {"extraction": "opus"}}}));
    let rust_drift = serde_json::to_value(state::validate_agent_files_drift(dir.path(), &raw_drift)).unwrap();
    let mjs_drift = run_oracle(
        "validate-agent-files-drift",
        Some(dir.path()),
        Some(json!({"present": true, "rawConfig": {"models": {"claude": {"extraction": "opus"}}}})),
    );
    assert_eq!(rust_drift, mjs_drift);
    assert_eq!(rust_drift[0]["code"], json!("agent-file-drift"));
}

// ═══════════════════════════════════════════════════════════════════════
// has_stale_advisor_key / read_onboarding / read_handoff / bypass_level
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn has_stale_advisor_key_present_and_absent_match_mjs() {
    for (name, body) in [("with-advisor", json!({"advisor": "fable"})), ("without-advisor", json!({"gate_bypass": "off"})), ("no-file", Value::Null)] {
        let dir = tempfile::tempdir().unwrap();
        if !body.is_null() {
            write_file(&bee_dir(dir.path()).join("config.json"), &body.to_string());
        }
        let rust_result = state::has_stale_advisor_key(dir.path());
        let mjs_result = run_oracle("has-stale-advisor-key", Some(dir.path()), None);
        assert_eq!(json!(rust_result), mjs_result, "case {name} diverged from mjs hasStaleAdvisorKey");
    }
}

#[test]
fn read_onboarding_matches_mjs_present_and_absent() {
    let dir = tempfile::tempdir().unwrap();
    let mjs_absent = run_oracle("read-onboarding", Some(dir.path()), None);
    assert_eq!(state::read_onboarding(dir.path()), mjs_absent);
    assert_eq!(mjs_absent, Value::Null);

    write_minimal_onboarding(dir.path());
    let mjs_present = run_oracle("read-onboarding", Some(dir.path()), None);
    assert_eq!(state::read_onboarding(dir.path()), mjs_present);
}

#[test]
fn read_handoff_normalizes_kind_matching_mjs() {
    let dir = tempfile::tempdir().unwrap();
    // Absent file.
    assert_eq!(state::read_handoff(dir.path()), run_oracle("read-handoff", Some(dir.path()), None));

    // Present, unknown kind -> normalizes to "pause".
    write_file(&bee_dir(dir.path()).join("HANDOFF.json"), &json!({"written_at": "2026-01-01T00:00:00.000Z"}).to_string());
    let rust_result = state::read_handoff(dir.path());
    let mjs_result = run_oracle("read-handoff", Some(dir.path()), None);
    assert_eq!(rust_result, mjs_result);
    assert_eq!(rust_result["kind"], json!("pause"));

    // Present, kind:"planned-next" -> passes through verbatim.
    write_file(&bee_dir(dir.path()).join("HANDOFF.json"), &json!({"kind": "planned-next"}).to_string());
    let rust_result = state::read_handoff(dir.path());
    let mjs_result = run_oracle("read-handoff", Some(dir.path()), None);
    assert_eq!(rust_result, mjs_result);
    assert_eq!(rust_result["kind"], json!("planned-next"));
}

#[test]
fn bypass_level_reads_overlay_merged_config_matching_mjs() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(json!(state::bypass_level(dir.path())), run_oracle("bypass-level", Some(dir.path()), None));
    assert_eq!(state::bypass_level(dir.path()), "off");

    write_file(&bee_dir(dir.path()).join("config.json"), &json!({"gate_bypass": "normal"}).to_string());
    assert_eq!(state::bypass_level(dir.path()), "normal");
    assert_eq!(json!(state::bypass_level(dir.path())), run_oracle("bypass-level", Some(dir.path()), None));

    // The LOCAL overlay wins over the tracked file — proves this reads
    // the overlay-merged value, not `Config::bypass_level`'s tracked-only
    // scope.
    write_file(&bee_dir(dir.path()).join("config.local.json"), &json!({"gate_bypass": "total"}).to_string());
    assert_eq!(state::bypass_level(dir.path()), "total");
    assert_eq!(json!(state::bypass_level(dir.path())), run_oracle("bypass-level", Some(dir.path()), None));
}

// ═══════════════════════════════════════════════════════════════════════
// classify_source
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn classify_source_matches_mjs_across_every_kind() {
    // no hiveDir at all
    let mjs_none = run_oracle("classify-source", None, Some(json!({})));
    assert_eq!(source_identity::classify_source(None, None), mjs_none);

    // source_checkout: plugin.json + .git
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("skills").join("bee-hive");
    fs::create_dir_all(&hive).unwrap();
    fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
    fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    let rust_result = source_identity::classify_source(Some(&hive), None);
    let mjs_result = run_oracle("classify-source", None, Some(json!({"hiveDir": hive.to_string_lossy()})));
    assert_eq!(rust_result, mjs_result);
    assert_eq!(rust_result["kind"], json!("source_checkout"));

    // plugin_package: plugin.json, no .git
    let dir2 = tempfile::tempdir().unwrap();
    let hive2 = dir2.path().join("skills").join("bee-hive");
    fs::create_dir_all(&hive2).unwrap();
    fs::create_dir_all(dir2.path().join(".claude-plugin")).unwrap();
    fs::write(dir2.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
    let rust_result2 = source_identity::classify_source(Some(&hive2), None);
    let mjs_result2 = run_oracle("classify-source", None, Some(json!({"hiveDir": hive2.to_string_lossy()})));
    assert_eq!(rust_result2, mjs_result2);
    assert_eq!(rust_result2["kind"], json!("plugin_package"));

    // project_projection: under .claude/skills, no manifest at all
    let dir3 = tempfile::tempdir().unwrap();
    let hive3 = dir3.path().join(".claude").join("skills").join("bee-hive");
    fs::create_dir_all(&hive3).unwrap();
    let rust_result3 = source_identity::classify_source(Some(&hive3), None);
    let mjs_result3 = run_oracle("classify-source", None, Some(json!({"hiveDir": hive3.to_string_lossy()})));
    assert_eq!(rust_result3, mjs_result3);
    assert_eq!(rust_result3["kind"], json!("project_projection"));

    // unknown: no manifest, not a projection
    let dir4 = tempfile::tempdir().unwrap();
    let hive4 = dir4.path().join("skills").join("bee-hive");
    fs::create_dir_all(&hive4).unwrap();
    let rust_result4 = source_identity::classify_source(Some(&hive4), None);
    let mjs_result4 = run_oracle("classify-source", None, Some(json!({"hiveDir": hive4.to_string_lossy()})));
    assert_eq!(rust_result4, mjs_result4);
    assert_eq!(rust_result4["kind"], json!("unknown"));
}

// ═══════════════════════════════════════════════════════════════════════
// whole-command oracle: the seven bee.mjs-private helpers, via a real
// `node .bee/bin/bee.mjs status --json` subprocess, diffed sub-object by
// sub-object (never a hand-invented reimplementation).
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn source_and_find_repo_hive_match_mjs_status_on_a_hand_built_hive_fixture() {
    let dir = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir.path());
    let hive = dir.path().join("skills").join("bee-hive");
    fs::create_dir_all(&hive).unwrap();
    fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
    fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();

    let rust_hive = state::find_repo_hive(dir.path());
    assert_eq!(rust_hive, Some(hive.clone()));
    let rust_source = source_identity::classify_source(rust_hive.as_deref(), None);

    let mjs_status = run_oracle("status", Some(dir.path()), None);
    // bee.mjs's status `source` field is `{kind, root}` ONLY (markers are
    // stripped before rendering) — matches buildStatus's own `sourceId`
    // destructure at bee.mjs:833.
    let mjs_source = json!({"kind": mjs_status["source"]["kind"], "root": mjs_status["source"]["root"]});
    let rust_source_trimmed = json!({"kind": rust_source["kind"], "root": rust_source["root"]});
    assert_eq!(rust_source_trimmed, mjs_source);
    assert_eq!(mjs_source["kind"], json!("source_checkout"));
}

#[test]
fn build_recovery_block_matches_mjs_status_on_host_real_crash_candidate_fixture() {
    let fixture = host_real_fixture();
    let root = fixture.path();

    // Locate the queen-bench-generated crash-candidate session and its
    // injected (fixture-local, never host $HOME) transcript root from the
    // fixture's OWN config — never assumed/hardcoded.
    let sessions = claims::list_session_records(root);
    assert!(!sessions.is_empty(), "queen-bench's grown fixture must carry at least one session record (rust-port-19)");

    let mjs_status = run_oracle("status", Some(root), None);
    let mjs_recovery = &mjs_status["recovery"];
    assert!(mjs_recovery["candidates"].as_array().map(|a| !a.is_empty()).unwrap_or(false), "the host-real fixture's own non-triviality contract (rust-port-19) requires >=1 crash candidate");

    // Rebuild the SAME inputs bee.mjs's buildStatus assembles for
    // buildRecoveryBlock: root===control_root here (no worktree in this
    // fixture), projectsRoot from config.recovery.transcript_roots (the
    // injected root), projectPath===root, now pinned to "at generation
    // time" (queen-bench's fixture bakes a heartbeat-stale session
    // relative to ITS OWN generation clock, so `now` must be recent
    // enough that the session still reads as stale — any timestamp at or
    // after generation time satisfies that).
    let now_ms = bee_core::jsdate::parse_iso_ms(&chrono_now_far_future()).unwrap();

    // `projects_root` here stands in for `claudeProjectsRoot()`'s DEFAULT
    // (the real CLI, run for real below via the "status" whole-command
    // op, always resolves the actual host `$HOME/.claude/projects` — this
    // test never reads that directory's content, but for the comparison
    // to be apples-to-apples the Rust side's "claude" default entry must
    // likewise NOT be the fixture's own injected root (queen-bench
    // configures ONLY a "fixture"-runtime entry, per rust-port-19) — an
    // empty, unrelated tempdir reproduces "the real claude default root
    // exists but never matches this fixture's transcript" without this
    // test depending on what is actually in the host's real directory.
    let unrelated_claude_default = tempfile::tempdir().unwrap();

    // rust-port-23: `build_recovery_block` takes the per-invocation shared
    // memo (caller-supplied, lazy). A fresh one is exactly the old
    // self-loading behavior.
    let shared = bee_core::shared_reads::SharedReads::new(root);
    let rust_recovery = recovery::build_recovery_block(&shared, root, unrelated_claude_default.path(), root, now_ms, None);
    assert_eq!(rust_recovery["candidates"], mjs_recovery["candidates"], "detect_crash_candidates diverged from mjs on the host-real fixture");

    // `roots`: the "claude" entry's PATH is inherently host-specific (the
    // real subprocess resolves the actual `$HOME/.claude/projects`, this
    // test's Rust call resolves an unrelated tempdir standing in for it —
    // see the comment above) so only its shape (present, tagged "claude")
    // is checked; the "fixture"-runtime entry (queen-bench's injected,
    // fixture-local root, rust-port-19) must match byte-for-byte.
    let rust_roots = rust_recovery["roots"].as_array().expect("roots array");
    let mjs_roots = mjs_recovery["roots"].as_array().expect("roots array");
    assert_eq!(rust_roots.len(), mjs_roots.len());
    assert_eq!(rust_roots[0]["runtime"], json!("claude"));
    assert_eq!(mjs_roots[0]["runtime"], json!("claude"));
    assert_eq!(rust_roots[1], mjs_roots[1], "the injected fixture-local transcript root entry must match mjs exactly");
}

/// A fixed, far-future ISO timestamp — never `SystemTime::now()` — used
/// only as "now" for the host-real fixture's crash-candidate check (the
/// session's heartbeat is already stale relative to ITS OWN generation
/// time, so any later `now` still classifies it as stale; a literal
/// constant keeps this test's `now` pinned rather than wall-clock-derived).
fn chrono_now_far_future() -> String {
    "2030-01-01T00:00:00.000Z".to_string()
}

#[test]
fn build_contention_summary_matches_mjs_status_present_and_absent() {
    // Absent log -> bee.mjs omits the `contention` key entirely.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir.path());
    assert_eq!(recovery::build_contention_summary(dir.path()), None);
    let mjs_status = run_oracle("status", Some(dir.path()), None);
    assert!(mjs_status.get("contention").is_none(), "an absent contention.jsonl must omit the key entirely");

    // Present log with busy + non-busy events -> a real summary.
    let dir2 = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir2.path());
    let events = [
        json!({"ts": "2026-01-01T00:00:00.000Z", "lock_name": "cells", "result": "busy", "holder_session": "s1", "caller_session": "s2", "lock_wait_ms": 150}),
        json!({"ts": "2026-01-01T00:00:01.000Z", "lock_name": "cells", "result": "busy", "holder_session": "s1", "caller_session": "s3", "lock_wait_ms": 300}),
        json!({"ts": "2026-01-01T00:00:02.000Z", "lock_name": "reservations", "result": "busy", "holder_session": "s4", "caller_session": "s3", "lock_wait_ms": 900}),
        json!({"ts": "2026-01-01T00:00:03.000Z", "lock_name": "reservations", "result": "acquired", "lock_wait_ms": 1200}),
    ];
    let body: String = events.iter().map(|e| format!("{e}\n")).collect();
    write_file(&bee_dir(dir2.path()).join("logs").join("contention.jsonl"), &body);

    let rust_result = recovery::build_contention_summary(dir2.path());
    let mjs_status2 = run_oracle("status", Some(dir2.path()), None);
    let mjs_contention = mjs_status2.get("contention").cloned().expect("contention key must be present when busy events exist");
    assert_eq!(json!(rust_result), mjs_contention);
    let rust_result = rust_result.unwrap();
    assert_eq!(rust_result["busy_count"], json!(3));
    assert_eq!(rust_result["worst_wait_ms"], json!(1200), "worst_wait_ms spans EVERY event in the window, not just busy ones");
    assert_eq!(rust_result["top_locks"][0]["lock_name"], json!("cells"));
    assert_eq!(rust_result["top_locks"][0]["busy_count"], json!(2));
}

#[test]
fn build_lane_rows_and_summary_match_mjs_status_default_and_lanes_full() {
    let dir = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir.path());
    write_lane(dir.path(), "rust-port", &json!({"phase": "swarming", "feature": "rust-port"}));
    write_lane(dir.path(), "other-feature", &json!({"phase": "planning", "feature": "other-feature"}));
    // ONE fresh-heartbeat session bound to "rust-port". `resolveSessionId`'s
    // root-inference branch (the real mjs function, called with no `now`
    // override) filters on `heartbeatStale(session)`, which uses REAL
    // `Date.now()` — so the heartbeat here must be actual wall-clock
    // "now", not a pinned literal (unlike the reservation/heartbeat
    // BOUNDARY tests elsewhere in this file, which pin `now` explicitly
    // through a parameter the real functions DO accept). With no
    // BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID in the oracle subprocess's env
    // (stripped by `run_oracle`), root-inference resolves to this session
    // (the only live one) — matching the `resolved_session_id` this test
    // passes to `build_lane_summary` directly.
    let heartbeat_now = bee_core::lock::iso8601_millis(
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
    );
    write_session(dir.path(), "sess-1", &json!({"id": "sess-1", "started_at": "2026-01-01T00:00:00.000Z", "last_heartbeat": heartbeat_now, "lane": "rust-port"}));

    let mjs_full = run_oracle("status", Some(dir.path()), Some(json!({"lanesFull": true})));
    let rust_rows = recovery_free_build_lane_rows(dir.path());
    assert_eq!(rust_rows, mjs_full["lanes"], "build_lane_rows diverged from mjs's --lanes-full array");

    let mjs_default = run_oracle("status", Some(dir.path()), None);
    let rust_summary = state::build_lane_summary(dir.path(), dir.path(), Some("sess-1"));
    assert_eq!(rust_summary, mjs_default["lanes"], "build_lane_summary diverged from mjs's default summary");
    assert_eq!(rust_summary["active"]["feature"], json!("rust-port"));
    assert_eq!(rust_summary["ids"], json!(["other-feature"]));
}

fn recovery_free_build_lane_rows(root: &Path) -> Value {
    json!(state::build_lane_rows(root))
}

#[test]
fn compute_runtime_drift_matches_mjs_status_clean_and_drifted() {
    let dir = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir.path());
    let lib_file = dir.path().join(".bee").join("bin").join("lib").join("fake.mjs");
    write_file(&lib_file, "export const X = 1;\n");
    let live_hash = bee_core::fsutil::hash_file(&lib_file).unwrap();

    // Clean: onboarding's recorded hash matches the live file exactly.
    write_file(
        &bee_dir(dir.path()).join("onboarding.json"),
        &json!({"bee_version": state::BEE_VERSION, "managed": {"lib": {"fake.mjs": live_hash}, "helpers": {}}}).to_string(),
    );
    let onboarding_raw = state::read_onboarding(dir.path());
    let rust_clean = state::compute_runtime_drift(dir.path(), &onboarding_raw);
    let mjs_status_clean = run_oracle("status", Some(dir.path()), None);
    assert_eq!(rust_clean["drift"], mjs_status_clean["onboarding"]["drift"]);
    assert_eq!(rust_clean["drift"], json!(false));
    assert!(mjs_status_clean["onboarding"].get("drift_detail").is_none());

    // Drifted: recorded hash no longer matches the live file's content.
    write_file(
        &bee_dir(dir.path()).join("onboarding.json"),
        &json!({"bee_version": state::BEE_VERSION, "managed": {"lib": {"fake.mjs": "0000000000000000000000000000000000000000000000000000000000000000"}, "helpers": {}}}).to_string(),
    );
    let onboarding_raw2 = state::read_onboarding(dir.path());
    let rust_drift = state::compute_runtime_drift(dir.path(), &onboarding_raw2);
    let mjs_status_drift = run_oracle("status", Some(dir.path()), None);
    assert_eq!(rust_drift["drift"], mjs_status_drift["onboarding"]["drift"]);
    assert_eq!(rust_drift["drift"], json!(true));
    assert_eq!(rust_drift["detail"], mjs_status_drift["onboarding"]["drift_detail"]);
    assert_eq!(rust_drift["detail"], json!([".bee/bin/lib/fake.mjs"]));
}

// ═══════════════════════════════════════════════════════════════════════
// ungranted_worktree_notice — messaging-only, caller-supplied topology
// (no live git-worktree fixture: this function never re-derives its own
// input, per rust-port-16's established control-root-parameter pattern —
// see WorktreeRootsView's own doc comment). Direct proof against the
// exact mjs message text instead of a subprocess oracle.
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn ungranted_worktree_notice_fires_only_for_an_ungranted_linked_worktree() {
    let main = PathBuf::from("/repo/main");
    let ungranted = state::WorktreeRootsView {
        worktree_resolution: "linked-valid".to_string(),
        store_root: Some(main.clone()),
        main_root: Some(main.clone()),
    };
    let notice = state::ungranted_worktree_notice(&ungranted).expect("an ungranted linked worktree must notice");
    assert!(notice.starts_with("⚠ This linked worktree is UNGRANTED"));
    assert!(notice.contains("bee worktree new --feature <slug>"));
    assert!(notice.contains("bee worktree register --feature <slug>"));

    let granted = state::WorktreeRootsView {
        worktree_resolution: "linked-valid".to_string(),
        store_root: Some(PathBuf::from("/repo/worktrees/feat")),
        main_root: Some(main.clone()),
    };
    assert_eq!(state::ungranted_worktree_notice(&granted), None, "a GRANTED linked worktree (store_root != main_root) never notices");

    let ordinary = state::WorktreeRootsView { worktree_resolution: "ordinary".to_string(), store_root: Some(main.clone()), main_root: Some(main) };
    assert_eq!(state::ungranted_worktree_notice(&ordinary), None, "an ordinary checkout never notices");
}

// ═══════════════════════════════════════════════════════════════════════
// reservations: the reference-identity fixture (must-have #4) — pinned
// clock, never wall time.
// ═══════════════════════════════════════════════════════════════════════

const PINNED_NOW_MS: i64 = 1_893_456_000_000; // 2030-01-01T12:00:00.000Z — a literal constant, never SystemTime::now()

fn iso(ms: i64) -> String {
    bee_core::lock::iso8601_millis(ms)
}

#[test]
fn reservation_reference_identity_bug_is_replicated_not_fixed() {
    let dir = tempfile::tempdir().unwrap();
    write_lease(
        dir.path(),
        "active",
        &json!({
            "resource": "path:crates/foo.rs",
            "workflow_id": "cell-A",
            "workspace_id": "agent:Mel",
            "acquired_at": iso(PINNED_NOW_MS - 1_000),
            "expires_at": iso(PINNED_NOW_MS + 100_000),
        }),
    );
    write_lease(
        dir.path(),
        "expired",
        &json!({
            "resource": "path:crates/bar.rs",
            "workflow_id": "cell-B",
            "workspace_id": "agent:Stuart",
            "acquired_at": iso(PINNED_NOW_MS - 200_000),
            "expires_at": iso(PINNED_NOW_MS - 100_000),
        }),
    );

    // listReservations({activeOnly:false}) legitimately returns BOTH —
    // this alone is correct and not the bug.
    let rust_all = reservations::list_reservations(dir.path(), false, PINNED_NOW_MS);
    assert_eq!(rust_all.len(), 2);
    let mjs_all = run_oracle("list-reservations", Some(dir.path()), Some(json!({"activeOnly": false, "now": PINNED_NOW_MS})));
    assert_eq!(sorted_by_path(serde_json::to_value(&rust_all).unwrap()), sorted_by_path(mjs_all));

    // listReservations({activeOnly:true}) correctly excludes the expired
    // one — `is_lease_record_expired`/`isLeaseRecordExpired` themselves
    // are NOT buggy.
    let rust_active = reservations::list_reservations(dir.path(), true, PINNED_NOW_MS);
    assert_eq!(rust_active.len(), 1);
    assert_eq!(rust_active[0].path, "crates/foo.rs");
    let mjs_active = run_oracle("list-reservations", Some(dir.path()), Some(json!({"activeOnly": true, "now": PINNED_NOW_MS})));
    assert_eq!(sorted_by_path(serde_json::to_value(&rust_active).unwrap()), sorted_by_path(mjs_active));

    // THE BUG (bee.mjs:741-746, D1-frozen, deliberately replicated): the
    // buildStatus-inline "expired unreleased" computation counts BOTH
    // rows, not just the genuinely-expired one. This fixture's two leases
    // are DISTINGUISHABLE (different resource/workflow_id/workspace_id/
    // timestamps) — it proves the bug still fires even when a naive
    // value-equality port could tell the rows apart by content, not only
    // when two rows happen to be indistinguishable. The companion test
    // below (`..._with_deeply_equal_projected_rows`) is the sharper case
    // truth #4 names explicitly: two rows so `LeaseReservationView`-equal
    // that even the STRUCTURE of a value-equality "fix" collapses them.
    let rust_expired_unreleased = reservations::expired_unreleased_reservations(dir.path(), PINNED_NOW_MS);
    assert_eq!(rust_expired_unreleased.len(), 2, "mjs's reference-identity bug means the ACTIVE reservation is misclassified as expired-unreleased too — a value-equality port would wrongly return 1 here");
}

#[test]
fn reservation_reference_identity_bug_with_deeply_equal_projected_rows() {
    // Truth #4's sharper fixture: two lease records that project to
    // DEEPLY EQUAL `LeaseReservationView` rows — same `resource` (so same
    // `path`), no `acquired_at` (so both `ttl_seconds` compute to the same
    // 0 sentinel — `lease_ttl_seconds` requires acquired_at to be a real
    // ISO string to do anything else, reservations.rs), no
    // `workflow_id`/`workspace_id` (`cell`/`agent` both absent on both
    // rows), and `released_at` is always `null` regardless of
    // `expires_at`. The ONE field that differs between the two on-disk
    // records — `expires_at` — is dropped entirely by the projection
    // (`LeaseReservationView` has no `expires_at` field at all), so the
    // two OUTPUT rows are indistinguishable even by full structural
    // equality, not just by path. A value-equality "dedup" over the
    // projected shape would collapse these two into one; the real mjs
    // (and this port) never does — it returns both, unconditionally.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_onboarding(dir.path());
    write_lease(dir.path(), "past", &json!({"resource": "path:a.rs", "expires_at": "2020-01-01T00:00:00.000Z"}));
    write_lease(dir.path(), "future", &json!({"resource": "path:a.rs", "expires_at": "2099-01-01T00:00:00.000Z"}));

    let rust_expired_unreleased = reservations::expired_unreleased_reservations(dir.path(), PINNED_NOW_MS);
    assert_eq!(rust_expired_unreleased.len(), 2);
    let row_a = serde_json::to_value(&rust_expired_unreleased[0]).unwrap();
    let row_b = serde_json::to_value(&rust_expired_unreleased[1]).unwrap();
    assert_eq!(row_a, row_b, "the two projected rows must be DEEPLY EQUAL — that is exactly what makes this fixture discriminate a value-equality port, which would see one row and collapse the pair to a count of 1");

    // Whole-command oracle: the real frozen CLI's own staleness_warnings
    // message (bee.mjs:765-768) must also report 2, never a deduped 1.
    let mjs_status = run_oracle("status", Some(dir.path()), None);
    let warnings = mjs_status["staleness_warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        warnings.iter().any(|w| w.as_str() == Some("2 reservation(s) expired but never released — run bee_reservations.mjs sweep.")),
        "expected the real mjs status --json staleness_warnings to report exactly 2 expired-unreleased reservations, got: {warnings:?}"
    );
}

#[test]
fn lease_expiry_boundary_is_pinned_not_wall_time() {
    let dir = tempfile::tempdir().unwrap();
    // expires_at exactly == now -> expired (`<=`, not `<`).
    write_lease(
        dir.path(),
        "boundary-expired",
        &json!({"resource": "path:a.rs", "acquired_at": iso(PINNED_NOW_MS - 1000), "expires_at": iso(PINNED_NOW_MS)}),
    );
    // expires_at one ms after now -> still active.
    write_lease(
        dir.path(),
        "boundary-active",
        &json!({"resource": "path:b.rs", "acquired_at": iso(PINNED_NOW_MS - 1000), "expires_at": iso(PINNED_NOW_MS + 1)}),
    );
    let rust_active = reservations::list_reservations(dir.path(), true, PINNED_NOW_MS);
    assert_eq!(rust_active.len(), 1);
    assert_eq!(rust_active[0].path, "b.rs");
    let mjs_active = run_oracle("list-reservations", Some(dir.path()), Some(json!({"activeOnly": true, "now": PINNED_NOW_MS})));
    assert_eq!(sorted_by_path(serde_json::to_value(&rust_active).unwrap()), sorted_by_path(mjs_active));
}

#[test]
fn heartbeat_staleness_boundary_is_pinned_not_wall_time() {
    // claims.mjs:367's `<=` boundary: a heartbeat exactly
    // `stale_seconds` old is ALREADY stale, matching
    // `beatMs + staleSeconds*1000 <= nowMs`.
    let beat_ms = bee_core::jsdate::parse_iso_ms("2026-01-01T00:00:00.000Z").unwrap();
    let stale_seconds = claims::DEFAULT_HEARTBEAT_STALE_SECONDS;
    let session = claims_session_with_heartbeat("2026-01-01T00:00:00.000Z");

    let exactly_at_boundary = beat_ms + stale_seconds * 1000;
    assert!(claims::heartbeat_stale(Some(&session), exactly_at_boundary, stale_seconds), "exactly at the boundary must already read stale (<=)");

    let one_ms_before_boundary = exactly_at_boundary - 1;
    assert!(!claims::heartbeat_stale(Some(&session), one_ms_before_boundary, stale_seconds), "one ms before the boundary must still read fresh");
}

fn claims_session_with_heartbeat(heartbeat: &str) -> claims::Session {
    let raw = json!({"id": "s", "last_heartbeat": heartbeat});
    serde_json::from_value(raw).unwrap()
}
