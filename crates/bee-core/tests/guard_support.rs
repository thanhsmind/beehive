//! guard_support — the single integration target for rust-port-8's
//! guard-support readers, command-registry JSON bridge loader, and
//! tokenizer port (CONTEXT.md D3/D5). Run via
//! `cargo test --manifest-path crates/Cargo.toml -p bee-core --test guard_support`
//! (this cell's own `verify`, chained after `node scripts/dump_command_registry.mjs`
//! so the registry-freshness tests below have a real dump to load).
//!
//! ALL tests for this cell live in this one file (not scattered as
//! `#[cfg(test)]` blocks in `src/`) per the cell's own must-have. Sections,
//! in cell order: tokenize (oracle-diffed against BOTH frozen mjs copies),
//! registry bridge, then the guard-support readers (config, state, cells,
//! reservations/leases, holds, workspace, claims).
//!
//! Every reader test uses a fresh `tempfile::tempdir()` root — never this
//! repo's own live `.bee/` store (prohibition: "tests never touch the live
//! .bee/ store"). The two exceptions, both read-only and both explicitly
//! sanctioned: the tokenize oracle drivers read the real, FROZEN mjs
//! tokenizer files (same posture as `fsutil_oracle.rs`/`lock_interop.rs`),
//! and the registry-freshness tests read `.bee/bin/lib/command-registry.mjs`
//! plus the `.bee/cache/command-registry.json` this cell's own `verify`
//! chain just generated (the dump script's `.bee/cache/` write is the one
//! sanctioned write exception, per the cell's own prohibition wording) —
//! neither test writes anywhere in the live store.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

use bee_core::claims::{self, Session};
use bee_core::registry::{self, Registry};
use bee_core::{cells, config, holds, jsdate, reservations, state, tokenize, workspace};

// ─── shared repo-root helper (mirrors fsutil_oracle.rs/lock_interop.rs) ────

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(".bee").join("bin").join("lib").join("command-registry.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("guard_support: could not locate .bee/bin/lib/command-registry.mjs above CARGO_MANIFEST_DIR");
}

// ─── tokenize: oracle-diffed against BOTH frozen mjs copies ────────────────

fn tokenize_corpus() -> Vec<&'static str> {
    vec![
        "git status",
        "echo \"hello world\"",
        "echo 'hello world'",
        "foo\"bar\"baz",
        "echo hi;echo bye",
        "a && b || c",
        "cmd1 & cmd2",
        "cmd1 | cmd2",
        "echo foo 2>/dev/null;rm -rf /",
        "FOO=bar BAZ=qux node script.js",
        "cat <<'EOF'\nsome content\nEOF",
        "(cd /tmp && ls)",
        "$(echo sub)",
        "echo \\$HOME",
        "grep -n 'pattern' file.txt | head -5",
        "node -e \"console.log(1)\"",
        "echo \"unterminated",
        "echo 'unterminated",
        "printf 'a\\tb\\n'",
        "echo a\\ b\\ c",
        "",
        "   ",
        "cmd1;;cmd2",
        "cmd1&&&&cmd2",
        "echo $(date)  > out.txt 2>&1",
        "ls -la > /tmp/out.txt",
        "cat file | grep foo || echo none",
        "echo \"a'b\"",
        "echo 'a\"b'",
        "true;false;true",
    ]
}

struct TokenizeOracleOutput {
    tokenize_command: Vec<Vec<String>>,
    guards_tokenize: Vec<Vec<String>>,
}

fn run_tokenize_oracle(corpus: &[&str]) -> TokenizeOracleOutput {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/tokenize_oracle.mjs");
    let mut cmd = Command::new("node");
    cmd.arg(&script);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn node tokenize_oracle driver — is `node` on PATH?");
    {
        let mut stdin = child.stdin.take().expect("child stdin");
        let payload = serde_json::to_string(corpus).expect("serialize corpus");
        stdin.write_all(payload.as_bytes()).expect("write oracle stdin");
    }
    let output = child.wait_with_output().expect("wait for tokenize_oracle driver");
    assert!(
        output.status.success(),
        "tokenize_oracle driver exited non-zero — stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout).expect("parse tokenize_oracle stdout as JSON");
    let to_vec_vec = |v: &Value| -> Vec<Vec<String>> {
        v.as_array()
            .expect("expected array")
            .iter()
            .map(|row| {
                row.as_array()
                    .expect("expected array of tokens")
                    .iter()
                    .map(|t| t.as_str().expect("expected string token").to_string())
                    .collect()
            })
            .collect()
    };
    TokenizeOracleOutput {
        tokenize_command: to_vec_vec(&parsed["tokenize_command"]),
        guards_tokenize: to_vec_vec(&parsed["guards_tokenize"]),
    }
}

/// Truth: "tokenize parity: rust tokens identical to BOTH
/// tokenize-command.mjs and guards.mjs tokenize over the full corpus."
/// Table-driven over the shapes named in the cell (heredoc markers,
/// redirects, env prefixes, subshells, quotes/escapes, chain separators) —
/// one test, one oracle spawn, the whole corpus.
#[test]
fn tokenize_matches_both_frozen_mjs_copies_over_full_corpus() {
    let corpus = tokenize_corpus();
    let oracle = run_tokenize_oracle(&corpus);

    for (i, cmd) in corpus.iter().enumerate() {
        let rust_tokens = tokenize::tokenize_command(cmd);
        assert_eq!(
            rust_tokens, oracle.tokenize_command[i],
            "case {i} ({cmd:?}): rust tokenize_command diverged from the real tokenize-command.mjs"
        );
        assert_eq!(
            rust_tokens, oracle.guards_tokenize[i],
            "case {i} ({cmd:?}): rust tokenize_command diverged from the real guards.mjs tokenize"
        );
        // The two mjs copies are hand-synced duplicates (module doc comment)
        // — this proves they still agree with EACH OTHER too, which is
        // exactly the drift this oracle exists to surface, not paper over.
        assert_eq!(
            oracle.tokenize_command[i], oracle.guards_tokenize[i],
            "case {i} ({cmd:?}): the two mjs copies (tokenize-command.mjs vs guards.mjs) disagree with each other"
        );
    }
}

#[test]
fn tokenize_unterminated_quote_runs_to_end_of_string() {
    assert_eq!(tokenize::tokenize_command("echo \"unterminated"), vec!["echo", "unterminated"]);
}

#[test]
fn tokenize_backslash_at_end_of_string_is_dropped_with_no_trailing_char() {
    // mjs's `i + 1 < str.length` guard means a trailing lone backslash
    // falls through to the plain "copy this char" branch instead of the
    // escape branch — proven directly here since it is a boundary the
    // corpus table above does not happen to hit.
    assert_eq!(tokenize::tokenize_command("foo\\"), vec!["foo\\"]);
}

// ─── registry bridge ────────────────────────────────────────────────────

/// Truth: "registry lookup serves all 116 entries and flags staleness when
/// command-registry.mjs sha differs" (fresh half). Reads the REAL
/// `.bee/cache/command-registry.json` this cell's own `verify` chain just
/// generated (`node scripts/dump_command_registry.mjs &&` runs first) and
/// the REAL frozen source file — both read-only.
#[test]
fn registry_loads_fresh_and_serves_every_entry_by_name() {
    let root = repo_root();
    let dump_path = root.join(".bee/cache/command-registry.json");
    let source_path = root.join(".bee/bin/lib/command-registry.mjs");
    assert!(
        dump_path.exists(),
        "expected {} to exist — this test's cell verify runs `node scripts/dump_command_registry.mjs` first",
        dump_path.display()
    );

    let registry = registry::load_registry(&dump_path, &source_path).expect("load_registry");
    assert!(registry.is_fresh(), "expected a freshly-generated dump to match the live source file's sha256");
    let dump = registry.dump();
    assert_eq!(dump.entries.len(), 116, "expected all 116 COMMAND_REGISTRY entries to survive the JSON round-trip");
    assert!(dump.find("status").is_some(), "expected the always-present `status` entry to be findable by name");
    assert!(dump.find("cells.update").is_some());
    assert!(dump.find("this-command-does-not-exist").is_none());
}

/// Truth: "...and flags staleness when command-registry.mjs sha differs"
/// (stale half). Copies the real dump to a temp file with a deliberately
/// wrong `source_sha256` and proves the loader flags it rather than
/// silently trusting a snapshot that no longer matches the source.
#[test]
fn registry_flags_staleness_when_embedded_sha_does_not_match_live_source() {
    let root = repo_root();
    let dump_path = root.join(".bee/cache/command-registry.json");
    let source_path = root.join(".bee/bin/lib/command-registry.mjs");
    assert!(dump_path.exists(), "expected {} to exist", dump_path.display());

    let real_text = std::fs::read_to_string(&dump_path).expect("read real dump");
    let mut doc: Value = serde_json::from_str(&real_text).expect("parse real dump");
    doc["source_sha256"] = json!("0000000000000000000000000000000000000000000000000000000000aa");

    let dir = tempfile::tempdir().unwrap();
    let stale_dump_path = dir.path().join("command-registry.json");
    std::fs::write(&stale_dump_path, doc.to_string()).unwrap();

    let registry = registry::load_registry(&stale_dump_path, &source_path).expect("load_registry");
    assert!(!registry.is_fresh());
    match &registry {
        Registry::Stale { embedded_sha256, live_sha256, .. } => {
            assert_eq!(embedded_sha256, "0000000000000000000000000000000000000000000000000000000000aa");
            assert_ne!(live_sha256, embedded_sha256);
        }
        Registry::Fresh(_) => panic!("expected Stale"),
    }
    // Stale is still readable — a caller can inspect the (out-of-date) payload.
    assert_eq!(registry.dump().entries.len(), 116);
}

#[test]
fn registry_missing_dump_file_returns_an_error() {
    let root = repo_root();
    let source_path = root.join(".bee/bin/lib/command-registry.mjs");
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist.json");
    assert!(registry::load_registry(&missing, &source_path).is_err());
}

// ─── config reader ──────────────────────────────────────────────────────

#[test]
fn config_read_missing_file_falls_open_to_default_off_bypass() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config::read_config(dir.path());
    assert_eq!(cfg.bypass_level(), "off");
    assert!(cfg.hook_enabled("write-guard")); // unlisted hook defaults to enabled
}

#[test]
fn config_round_trips_unknown_fields_and_normalizes_bypass_level() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
    std::fs::write(
        config::config_path(dir.path()),
        json!({
            "hooks": {"write-guard": false},
            "gate_bypass": "total",
            "models": {"claude": {"advisor": "fable"}},
            "commands": {"verify": "node scripts/run_verify.mjs"},
            "a_future_field_this_reader_does_not_know_about": {"nested": [1, 2, 3]}
        })
        .to_string(),
    )
    .unwrap();

    let cfg = config::read_config(dir.path());
    assert_eq!(cfg.bypass_level(), "total");
    assert!(!cfg.hook_enabled("write-guard"));
    assert!(cfg.hook_enabled("state-sync"));
    assert_eq!(cfg.model_slot("claude", "advisor"), Some(&json!("fable")));
    assert_eq!(cfg.extra.get("commands"), Some(&json!({"verify": "node scripts/run_verify.mjs"})));
    assert_eq!(
        cfg.extra.get("a_future_field_this_reader_does_not_know_about"),
        Some(&json!({"nested": [1, 2, 3]}))
    );
}

#[test]
fn config_bypass_level_legacy_true_normalizes_to_normal() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
    std::fs::write(config::config_path(dir.path()), json!({"gate_bypass": true}).to_string()).unwrap();
    assert_eq!(config::read_config(dir.path()).bypass_level(), "normal");
}

// ─── state reader ────────────────────────────────────────────────────────

#[test]
fn state_read_missing_file_falls_open_to_idle_default() {
    let dir = tempfile::tempdir().unwrap();
    let s = state::read_state(dir.path());
    assert_eq!(s.phase, "idle");
    assert!(!state::gate_approved(&s, "execution"));
}

#[test]
fn state_round_trips_unknown_fields_and_defaults_absent_gates_false() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
    std::fs::write(
        state::state_path(dir.path()),
        json!({
            "phase": "swarming",
            "feature": "rust-port",
            "mode": "high-risk",
            "approved_gates": {"context": true, "shape": true, "execution": true},
            "gate_bypass": "total",
            "a_future_field": {"nested": true}
        })
        .to_string(),
    )
    .unwrap();

    let s = state::read_state(dir.path());
    assert_eq!(s.phase, "swarming");
    assert_eq!(s.feature.as_deref(), Some("rust-port"));
    assert!(state::gate_approved(&s, "execution"));
    assert!(!state::gate_approved(&s, "review")); // absent from the file -> default false
    assert_eq!(s.extra.get("gate_bypass"), Some(&json!("total")));
    assert_eq!(s.extra.get("a_future_field"), Some(&json!({"nested": true})));
}

// ─── cells listing ───────────────────────────────────────────────────────

#[test]
fn cells_list_skips_archive_dir_and_reads_worker_from_nested_trace() {
    let dir = tempfile::tempdir().unwrap();
    let cells_dir = cells::cells_dir(dir.path());
    std::fs::create_dir_all(cells_dir.join("archive")).unwrap();
    std::fs::write(cells_dir.join("archive").join("old-1.json"), json!({"id": "old-1", "status": "capped"}).to_string()).unwrap();
    std::fs::write(
        cells_dir.join("rust-port-8.json"),
        json!({
            "id": "rust-port-8",
            "status": "claimed",
            "files": ["crates/bee-core/*"],
            "lane": "high-risk",
            "trace": {"worker": "Stuart"}
        })
        .to_string(),
    )
    .unwrap();

    let listed = cells::list_cells(dir.path());
    assert_eq!(listed.len(), 1, "archive/ entries must not appear in the default listing");
    assert_eq!(listed[0].id, "rust-port-8");
    assert_eq!(listed[0].status.as_deref(), Some("claimed"));
    assert_eq!(listed[0].worker(), Some("Stuart"));
    assert_eq!(listed[0].lane(), Some("high-risk"));
    assert_eq!(listed[0].files, vec!["crates/bee-core/*".to_string()]);
}

#[test]
fn cells_list_missing_dir_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert!(cells::list_cells(dir.path()).is_empty());
}

// ─── reservations projection + sharded leases ──────────────────────────

#[test]
fn reservations_projection_round_trips_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
    std::fs::write(
        reservations::reservations_path(dir.path()),
        json!({
            "reservations": [{
                "agent": "Stuart",
                "cell": "rust-port-8",
                "path": "crates/bee-core/*",
                "ttl_seconds": 3600,
                "reserved_at": "2026-07-26T00:00:00.000Z",
                "released_at": null,
                "kind": "lease",
                "a_future_field": {"x": 1}
            }]
        })
        .to_string(),
    )
    .unwrap();

    let rows = reservations::read_reservations_projection(dir.path());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].agent.as_deref(), Some("Stuart"));
    assert_eq!(rows[0].path.as_deref(), Some("crates/bee-core/*"));
    assert_eq!(rows[0].extra.get("kind"), Some(&json!("lease")));
    assert_eq!(rows[0].extra.get("a_future_field"), Some(&json!({"x": 1})));
}

#[test]
fn reservations_projection_missing_file_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert!(reservations::read_reservations_projection(dir.path()).is_empty());
}

#[test]
fn leases_list_reads_both_cells_and_paths_subdirs_and_defaults_omitted_kind() {
    let dir = tempfile::tempdir().unwrap();
    let base = reservations::leases_root(dir.path());
    std::fs::create_dir_all(base.join("cells")).unwrap();
    std::fs::create_dir_all(base.join("paths")).unwrap();
    std::fs::write(
        base.join("cells").join("rust-port-8.json"),
        json!({
            "resource": "cell:rust-port-8",
            "mode": "write",
            "workflow_id": "rust-port-8",
            "session_id": "sess-1",
            "workspace_id": "agent:Stuart",
            "epoch": 0,
            "acquired_at": "2026-07-26T00:00:00.000Z",
            "expires_at": null,
            "kind": "lease"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        base.join("paths").join("abc123.json"),
        json!({
            "resource": "path:crates/bee-core/*",
            "mode": "write",
            "workflow_id": "rust-port-8",
            "session_id": "sess-1",
            "workspace_id": "agent:Stuart",
            "epoch": 0,
            "acquired_at": "2026-07-26T00:00:00.000Z",
            "expires_at": null
        })
        .to_string(),
    )
    .unwrap();

    let mut leases = reservations::list_leases(dir.path());
    leases.sort_by(|a, b| a.resource.cmp(&b.resource)); // "cell:..." < "path:..." lexically
    assert_eq!(leases.len(), 2);
    assert_eq!(leases[0].resource, "cell:rust-port-8");
    assert_eq!(leases[0].kind.as_deref(), Some("lease"));
    assert_eq!(leases[1].resource, "path:crates/bee-core/*");
    assert_eq!(leases[1].kind, None); // omitted in the source record
}

// ─── worktree holds (checkWrite support set) ─────────────────────────────

#[test]
fn holds_paths_overlap_matches_glob_and_exact_semantics() {
    assert!(holds::paths_overlap("crates/bee-core/*", "crates/bee-core/src/lib.rs"));
    assert!(holds::paths_overlap("a/b", "a/b"));
    assert!(!holds::paths_overlap("a/b", "a/c"));
    assert!(holds::paths_overlap("*", "anything/at/all"));
}

fn write_holds(root: &Path, holds_array: Value) {
    std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
    std::fs::write(holds::holds_ledger_path(root), json!({"holds": holds_array}).to_string()).unwrap();
}

#[test]
fn holds_find_foreign_excludes_own_holder_and_released_entries() {
    let dir = tempfile::tempdir().unwrap();
    // ttl_seconds: 0 -> "never expires" (matches worktree-holds.mjs's
    // isExpired: `ttl <= 0` short-circuits to `false`), so this fixture's
    // liveness is independent of wall-clock time — only holder/released_at
    // decide activity here, which is what this test is proving.
    write_holds(
        dir.path(),
        json!([
            {
                "path": "crates/bee-core/src/lib.rs", "holder": "other-worktree", "feature": null,
                "session": "sess-1", "cell": "rust-port-2", "ttl_seconds": 0,
                "mirrored_at": "2026-07-26T00:00:00.000Z", "released_at": null
            },
            {
                "path": "crates/bee-core/src/lock.rs", "holder": "main", "feature": null,
                "session": "sess-2", "cell": "rust-port-3", "ttl_seconds": 0,
                "mirrored_at": "2026-07-26T00:00:00.000Z", "released_at": null
            },
            {
                "path": "crates/bee-core/src/fsutil.rs", "holder": "other-worktree", "feature": null,
                "session": "sess-3", "cell": "rust-port-5", "ttl_seconds": 0,
                "mirrored_at": "2026-07-26T00:00:00.000Z", "released_at": "2026-07-26T00:05:00.000Z"
            }
        ]),
    );

    let foreign = holds::find_foreign_holds(dir.path(), "main", &["crates/bee-core/*"]);
    assert_eq!(foreign.len(), 1, "must exclude the same-holder row and the released row");
    assert_eq!(foreign[0].cell.as_deref(), Some("rust-port-2"));
}

#[test]
fn holds_find_foreign_treats_ttl_expired_entries_as_inactive() {
    let dir = tempfile::tempdir().unwrap();
    write_holds(
        dir.path(),
        json!([{
            "path": "crates/bee-core/*", "holder": "other-worktree", "feature": null,
            "session": "sess-1", "cell": "rust-port-2", "ttl_seconds": 1,
            "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null
        }]),
    );
    assert!(holds::find_foreign_holds(dir.path(), "main", &["crates/bee-core/*"]).is_empty());
}

#[test]
fn holds_store_corrupt_false_when_missing_true_when_malformed() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!holds::holds_store_corrupt(dir.path()), "missing store must read as not-corrupt");

    std::fs::create_dir_all(dir.path().join(".bee").join("runtime")).unwrap();
    std::fs::write(holds::holds_ledger_path(dir.path()), b"{ not json").unwrap();
    assert!(holds::holds_store_corrupt(dir.path()));
}

// ─── workspace-store records ──────────────────────────────────────────────

#[test]
fn workspace_read_missing_record_is_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(workspace::read_workspace(dir.path(), "main").is_none());
}

#[test]
fn workspace_read_round_trips_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(workspace::workspaces_dir(dir.path())).unwrap();
    std::fs::write(
        workspace::workspace_path(dir.path(), "main"),
        json!({
            "id": "main", "type": "main", "root": "/repo", "branch": null, "base_sha": null,
            "write_owner_session": null, "fence_epoch": 0, "attached_sessions": [],
            "created_at": "2026-07-25T12:45:57.777Z", "a_future_field": {"nested": [1, 2]}
        })
        .to_string(),
    )
    .unwrap();

    let record = workspace::read_workspace(dir.path(), "main").unwrap();
    assert_eq!(record.id, "main");
    assert_eq!(record.workspace_type, "main");
    assert_eq!(record.fence_epoch, json!(0));
    assert_eq!(record.extra.get("created_at"), Some(&json!("2026-07-25T12:45:57.777Z")));
    assert_eq!(record.extra.get("a_future_field"), Some(&json!({"nested": [1, 2]})));
}

// ─── claims (readSession, heartbeatStale) ────────────────────────────────

#[test]
fn claims_read_session_missing_is_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(claims::read_session(dir.path(), "abc").is_none());
}

#[test]
fn claims_read_session_round_trips_and_rejects_id_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(claims::sessions_dir(dir.path())).unwrap();
    std::fs::write(
        claims::session_path(dir.path(), "sess-1"),
        json!({
            "id": "sess-1",
            "started_at": "2026-07-26T00:00:00.000Z",
            "last_heartbeat": "2026-07-26T00:05:00.000Z",
            "lane": "rust-port"
        })
        .to_string(),
    )
    .unwrap();

    let session = claims::read_session(dir.path(), "sess-1").unwrap();
    assert_eq!(session.id, "sess-1");
    // `lane` is now a NAMED field (rust-port-20: `buildLaneRows`/
    // `buildLaneSummary`/`detectCrashCandidates` all read it), so it no
    // longer lands in `extra` — `#[serde(flatten)]` only collects keys not
    // already claimed by a named field.
    assert_eq!(session.lane.as_deref(), Some("rust-port"));
    assert!(session.extra.get("lane").is_none());

    // A file present but whose own `id` doesn't match the requested id
    // (stale rename/copy) reads as absent, matching claims.mjs's guard.
    std::fs::write(
        claims::session_path(dir.path(), "sess-2"),
        json!({"id": "sess-1", "last_heartbeat": "2026-07-26T00:05:00.000Z"}).to_string(),
    )
    .unwrap();
    assert!(claims::read_session(dir.path(), "sess-2").is_none());
}

#[test]
fn claims_heartbeat_stale_true_for_absent_and_unparseable_and_old() {
    assert!(claims::heartbeat_stale(None, 0, claims::DEFAULT_HEARTBEAT_STALE_SECONDS));

    let no_beat = Session {
        id: "s".into(),
        started_at: None,
        last_heartbeat: None,
        lane: None,
        transcript_path: None,
        extra: Default::default(),
    };
    assert!(claims::heartbeat_stale(Some(&no_beat), 0, claims::DEFAULT_HEARTBEAT_STALE_SECONDS));

    let fresh = Session {
        id: "s".into(),
        started_at: None,
        last_heartbeat: Some("2026-07-26T00:00:00.000Z".into()),
        lane: None,
        transcript_path: None,
        extra: Default::default(),
    };
    let beat_ms = jsdate::parse_iso_ms("2026-07-26T00:00:00.000Z").unwrap();
    assert!(!claims::heartbeat_stale(Some(&fresh), beat_ms + 1000, 900), "1s after the beat, within the 900s window");
    assert!(
        claims::heartbeat_stale(Some(&fresh), beat_ms + 900 * 1000, 900),
        "exactly at the 900s window boundary must read as stale (mjs's <= comparison)"
    );
}
