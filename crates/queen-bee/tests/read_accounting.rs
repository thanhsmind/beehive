//! read_accounting — rust-port-22's own verify target (CONTEXT.md D3/D5,
//! `docs/history/rust-port/plan-slice3.md` slice 3 cell 1 of 3). This cell
//! adds the read-instrumentation seam (`bee_core::read_accounting`) and
//! proves it here as a per-fixture, per-site BASELINE TABLE — the counters
//! are site-labelled (see that module's doc comment) so this file's own
//! assertions ARE the table, not prose describing one.
//!
//! rust-port-23 UPDATE — this file is now BOTH the instrument and the
//! dedup's proof surface. rust-port-22 wrote it against the un-deduped
//! code (4 decisions parses / 6 cells scans / 2 transcript-root scans per
//! `build_status`); rust-port-23 landed the per-invocation shared reads and
//! those three totals are now 1 / 1 / 1, asserted below by the same test,
//! the same fixture and the same counter units that produced the baseline.
//! Three guards were added with the dedup:
//! `ready_cells_from_a_shared_inventory_still_resolves_an_archived_capped_dep`
//! (the archive trap — invisible to every counter here),
//! `a_second_build_status_invocation_re_reads_every_store` (per-invocation,
//! never a cache) and `absent_empty_and_malformed_stores_are_each_read_
//! exactly_once`.
//!
//! Every fixture below is a fresh `tempfile::tempdir()` — never this repo's
//! own live `.bee/` store (prohibition: "tests never touch the live .bee/
//! store").
//!
//! ## BASELINE-ONLY EVIDENCE (must-have truth 8 — read this before reusing
//! ## any number below against POST-dedup code)
//!
//! Every count and every reach-proof in this file is evidence about
//! TODAY'S un-deduped code only. Once rust-port-23 lands, a conditional
//! site that shows as "reached" here can start reporting its count
//! unconditionally for a reason that has nothing to do with the original
//! gate: if the dedup hoists a read above the conditional consumer that
//! used to gate it, the hoisted call fires once per invocation regardless
//! of whether that consumer still runs — the count still says "1", but "1"
//! no longer means "the gated consumer ran". A rust-port-23 worker reusing
//! this table must re-derive reachability against the NEW call graph, not
//! assume these reach-proofs still describe it.
//!
//! ## Placement rework (goal-check NEEDS_REVISION — read before trusting
//! ## "lowest shared primitive" as a static claim)
//!
//! The first cut of this seam counted `decisions_journal_parses` inside
//! `decisions.rs`'s own call sites. A goal-check judge proved that
//! gameable: it hand-wrote two extra real `read_jsonl(&decisions_path)`
//! calls immediately above the `active_decisions` call this file's bench
//! fixture test exercises (`queen_bee::status::status.rs:790`), from a
//! brand-new location OUTSIDE `decisions.rs` entirely — simulating exactly
//! the shape a rust-port-23 hoist could take — and the baseline test's
//! count did not move: three real store reads at a `build_status`-level
//! load point were completely invisible. The counter now lives inside
//! `fsutil::read_jsonl` itself (see `crate::fsutil::read_jsonl`'s and
//! `crate::read_accounting`'s doc comments), keyed on the path, so ANY
//! call site anywhere in the crate is counted.
//! `injected_reads_at_a_build_status_level_load_point_are_still_counted_for_both_stores`
//! below reproduces the judge's own experiment as a permanent regression
//! test, for both the decisions store and the cells store.
//!
//! ## Reconciling with decision e119fc8b (validation-slice3.md, REPAIRED)
//!
//! e119fc8b's "4/6/2" is correct FOR THE QUEEN-BENCH D5 FIXTURE specifically
//! — not a fixture-independent constant. `bench_fixture_baseline_
//! reconciles_with_e119fc8b_4_6_2` below runs the REAL, currently-compiled
//! `queen-bench --generate` binary (same artifact
//! `crates/bee-core/tests/status_readers_a.rs`'s `host_real_fixture()`
//! uses) and checks the counters against that reconciled total, not a
//! re-derived guess.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tempfile::TempDir;

use bee_core::read_accounting;

// ─────────────────────────────────────────────────────────────────────────
// Generic fixture plumbing
// ─────────────────────────────────────────────────────────────────────────

fn write_file(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
    fs::write(path, content).expect("write fixture file");
}

fn write_json(path: &Path, value: &Value) {
    write_file(path, &serde_json::to_string_pretty(value).unwrap());
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// A bare `.bee` root: onboarding marker + an idle, feature-less state.json
/// — the minimum `resolve_roots`/`read_state` need to behave predictably.
fn minimal_root() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write_json(&dir.path().join(".bee/onboarding.json"), &json!({}));
    write_json(
        &dir.path().join(".bee/state.json"),
        &json!({
            "schema_version": "1.0",
            "phase": "idle",
            "feature": null,
            "mode": null,
            "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
            "workers": [],
            "summary": "",
            "next_action": "No active bee work."
        }),
    );
    dir
}

fn set_state(root: &Path, phase: &str, feature: Option<&str>) {
    write_json(
        &root.join(".bee/state.json"),
        &json!({
            "schema_version": "1.0",
            "phase": phase,
            "feature": feature,
            "mode": null,
            "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
            "workers": [],
            "summary": "",
            "next_action": "No active bee work."
        }),
    );
}

fn seed_cell(root: &Path, id: &str, feature: &str, status: &str, deps: &[&str], behavior_change: bool, capped_at: Option<&str>) {
    let path = root.join(".bee/cells").join(format!("{id}.json"));
    write_json(
        &path,
        &json!({
            "id": id,
            "feature": feature,
            "status": status,
            "deps": deps,
            "trace": {"behavior_change": behavior_change, "capped_at": capped_at},
        }),
    );
}

/// `.bee/cells/archive/<feature>/<id>.json` — the REAL archive layout
/// (validation-slice3.md's repaired finding #2: feature-subdirectoried,
/// not a flat `archive/<id>.json`).
fn seed_archived_cell(root: &Path, id: &str, feature: &str, status: &str) {
    let path = root.join(".bee/cells/archive").join(feature).join(format!("{id}.json"));
    write_json(&path, &json!({ "id": id, "feature": feature, "status": status, "deps": [] }));
}

fn seed_session(root: &Path, id: &str, last_heartbeat_iso: &str, transcript_path: Option<&Path>) {
    let path = root.join(".bee/sessions").join(format!("{id}.json"));
    write_json(
        &path,
        &json!({
            "id": id,
            "started_at": last_heartbeat_iso,
            "last_heartbeat": last_heartbeat_iso,
            "transcript_path": transcript_path.map(|p| p.to_string_lossy().into_owned()),
        }),
    );
}

/// A transcript tail with NO clean-end trio (plain `assistant` events only)
/// — `has_clean_end_trio` is false, so a session pointing at this file is
/// never excluded as a clean stop.
fn write_open_transcript(path: &Path) {
    let mut body = String::new();
    for i in 0..3 {
        body.push_str(&json!({"type": "assistant", "timestamp": "2026-07-26T00:05:00.000Z", "n": i}).to_string());
        body.push('\n');
    }
    write_file(path, &body);
}

fn set_transcript_roots_config(root: &Path, extra_paths: &[&Path]) {
    let entries: Vec<Value> = extra_paths
        .iter()
        .enumerate()
        .map(|(i, p)| json!({"runtime": format!("fixture-{i}"), "path": p.to_string_lossy()}))
        .collect();
    write_json(&root.join(".bee/config.json"), &json!({ "hooks": {}, "recovery": { "transcript_roots": entries } }));
}

/// `.bee/bin/lib/state.mjs` — chain-nudge/state-sync's own early "is this
/// even a bee checkout" gate; content is never read, only existence.
fn seed_hook_lib_marker(root: &Path) {
    write_file(&root.join(".bee/bin/lib/state.mjs"), "// fixture marker only, never executed\n");
}

fn hook_payload(root: &Path, event: &str) -> String {
    json!({ "cwd": root.to_string_lossy(), "hook_event_name": event }).to_string()
}

// ─────────────────────────────────────────────────────────────────────────
// StatusContext, built by hand (never `from_process`) so every test stays
// hermetic and independent of this box's real $HOME/session state.
// ─────────────────────────────────────────────────────────────────────────

fn manual_status_context(root: &Path) -> queen_bee::status::StatusContext {
    queen_bee::status::StatusContext {
        root: root.to_path_buf(),
        control_root: root.to_path_buf(),
        cwd_roots: bee_core::state::WorktreeRootsView {
            worktree_resolution: "ordinary".to_string(),
            store_root: Some(root.to_path_buf()),
            main_root: None,
        },
        session_id: None,
        now_ms: now_ms(),
        home_dir: None,
        // Deliberately a directory that does not exist: every test's
        // "default Claude root" stat is then a deterministic ENOENT,
        // independent of whatever this box's real $HOME happens to hold.
        projects_root: root.join("nonexistent-claude-projects"),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The real queen-bench D5 fixture (must-have: reconcile with e119fc8b)
// ─────────────────────────────────────────────────────────────────────────

fn workspace_manifest() -> PathBuf {
    // CARGO_MANIFEST_DIR is `.../crates/queen-bee`; the workspace manifest
    // this test's own verify command already names (`--manifest-path
    // crates/Cargo.toml`) is its parent's `Cargo.toml`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("queen-bee crate must live under a workspace directory")
        .join("Cargo.toml")
}

fn find_queen_bench_bin() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin_name = if cfg!(windows) { "queen-bench.exe" } else { "queen-bench" };
    for _ in 0..12 {
        let candidate = dir.join("target").join("debug").join(bin_name);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// rework finding #2 (goal-check): the judge proved this leg was NOT
/// self-contained — deleting a prebuilt `target/debug/queen-bench` made it
/// fail loudly with a hint, which is fragility on a cold CI target
/// directory, not a proof hole. This now builds the `queen-bench` package
/// itself the first time it is needed, rather than requiring some earlier,
/// unrelated cargo invocation to have left a debug binary behind.
fn queen_bench_bin() -> PathBuf {
    if let Some(bin) = find_queen_bench_bin() {
        return bin;
    }
    let status = Command::new(env!("CARGO"))
        .arg("build")
        .arg("--manifest-path")
        .arg(workspace_manifest())
        .arg("-p")
        .arg("queen-bench")
        .status()
        .expect("failed to spawn `cargo build -p queen-bench` — is `cargo` on PATH?");
    assert!(status.success(), "cargo build --manifest-path {} -p queen-bench failed", workspace_manifest().display());

    find_queen_bench_bin().unwrap_or_else(|| {
        panic!(
            "read_accounting: queen-bench binary still not found under target/debug after building it \
             ourselves (searched ancestors of {})",
            env!("CARGO_MANIFEST_DIR")
        )
    })
}

fn host_real_bench_fixture() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(queen_bench_bin())
        .arg("--generate")
        .arg("--out")
        .arg(dir.path())
        .output()
        .expect("failed to spawn queen-bench --generate");
    assert!(output.status.success(), "queen-bench --generate failed — stderr: {}", String::from_utf8_lossy(&output.stderr));
    dir
}

// ─────────────────────────────────────────────────────────────────────────
// 1. Lowest-shared-primitive placement, not reader-function entries
//    (must-have: "demonstrated by a test showing a direct primitive call
//    increments the same counter a reader call does")
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn primitive_and_reader_calls_increment_the_same_shared_counter() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "c1", "demo", "open", &[], false, None);

    read_accounting::reset();
    let direct = bee_core::cells::list_cells(root);
    let after_direct = read_accounting::snapshot();
    assert_eq!(after_direct.cells_dir_scans, 1, "a direct list_cells call must increment the shared counter");
    assert_eq!(direct.len(), 1);

    read_accounting::reset();
    let _ = bee_core::cells::ready_cells(root, None);
    let after_reader = read_accounting::snapshot();
    assert_eq!(
        after_reader.cells_dir_scans, 1,
        "ready_cells (a higher-level reader) funnels through the SAME primitive and must \
         increment the SAME counter by the SAME amount as calling list_cells directly — proving \
         the counter lives at the shared primitive, not duplicated per reader-function entry"
    );

    // Same proof for decisions. Before rust-port-23 one call to
    // active_decisions performed TWO real reads (build_tag_overlay's own
    // read plus its own event read) and this assertion read 2; the dedup
    // makes the overlay share the caller's single read, so the
    // primitive-level counter now correctly reports 1 — the number moved
    // because a real read disappeared, which is exactly what a counter at
    // the shared primitive (rather than at the reader-function entry) is
    // for.
    read_accounting::reset();
    let _ = bee_core::decisions::active_decisions(root, None, false);
    let after_decisions = read_accounting::snapshot();
    assert_eq!(after_decisions.decisions_journal_parses, 1);
}

// ─────────────────────────────────────────────────────────────────────────
// 2. The centerpiece baseline table: the real queen-bench D5 fixture,
//    reconciled against decision e119fc8b's stated 4/6/2.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn bench_fixture_reads_each_store_once_per_build_status_invocation() {
    let fixture = host_real_bench_fixture();
    let root = fixture.path();
    let ctx = manual_status_context(root);

    read_accounting::reset();
    let _status = queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default());
    let snap = read_accounting::snapshot();

    println!("read-accounting table — queen-bench D5 fixture (state.feature: null), AFTER rust-port-23's dedup");
    println!("| store class                         | was | now | single load point |");
    println!("|--------------------------------------|-----|-----|-------------------|");
    println!(
        "| decisions_journal_parses              |   4 | {:>3} | SharedReads::decisions() — one active_decisions call (itself now one journal read, not two), shared by status's recent_decisions and recovery's last_durable_settlement |",
        snap.decisions_journal_parses
    );
    println!(
        "| cells_dir_scans                       |   6 | {:>3} | SharedReads::cells() — one list_cells, shared by the status counts, ready_cells, tier_mix, ceiling_scarcity, scribing_debt, global_scribing_debt and recovery |",
        snap.cells_dir_scans
    );
    println!(
        "| cell_dep_reads                        |   0 | {:>3} | UNCHANGED BY DESIGN — dep resolution still goes through read_cell (archive fallback intact); zero on this fixture because every cell is capped, none open |",
        snap.cell_dep_reads
    );
    println!(
        "| transcript_root_scan_invocations      |   2 | {:>3} | build_recovery_block's own scan, hoisted ABOVE detect_crash_candidates' no-sessions early return and passed down |",
        snap.transcript_root_scan_invocations
    );
    println!(
        "| transcript_root_stat_ops (fs-op unit) |   4 | {:>3} | 1 invocation x 2 roots (default Claude root + 1 configured fixture root) — still DIVERGES from the invocation count on purpose (W1) |",
        snap.transcript_root_stat_ops
    );
    println!(
        "WITHIN-INVOCATION CONSISTENCY: each store is now read at ONE instant per invocation; cross-store \
         consistency remains unguaranteed, exactly as today (decisions, cells and transcript roots are still \
         loaded at distinct moments). This is not snapshot semantics. status takes no D9 locks, so the lock and \
         lease conformance surface is untouched."
    );
    println!(
        "READ-COUNT EVIDENCE IS NOT ARCHIVE EVIDENCE: cell_dep_reads bundles read_cell's archive fallback, so \
         losing that fallback would move NO counter in this table. \
         `ready_cells_from_a_shared_inventory_still_resolves_an_archived_capped_dep` is the only guard for it."
    );

    // rust-port-23: ONE read per store per `build_status` invocation. The
    // pre-dedup baseline for this same fixture, same test, same counter
    // units was 4 / 6 / 2 (rust-port-22, decision e119fc8b) — those totals
    // are what these three assertions replace.
    assert_eq!(snap.decisions_journal_parses, 1, "decisions: one journal parse per build_status invocation (was 4)");
    assert_eq!(snap.cells_dir_scans, 1, "cells: one directory scan per build_status invocation (was 6)");
    assert_eq!(snap.transcript_root_scan_invocations, 1, "transcript roots: one scan per build_status invocation (was 2)");

    // cell_dep_reads is zero on THIS fixture for a documented reason (every
    // fixture cell is pre-capped) — recorded here, not silently ignored.
    // UNCHANGED by the dedup on purpose: dep resolution still goes through
    // `read_cell`, archive fallback and all (the archive trap).
    assert_eq!(snap.cell_dep_reads, 0);

    // The fs-operation-unit counter is a DIFFERENT number from the
    // invocation-unit counter on this fixture, because the fixture
    // configures exactly one extra transcript root beyond the always-
    // present default — see `transcript_root_stat_ops_scale_with_
    // configured_roots_while_invocations_do_not` below for the isolated
    // demonstration of why. One invocation x 2 roots = 2 (was 2 x 2 = 4).
    assert_eq!(snap.transcript_root_stat_ops, 2);
}

// ─────────────────────────────────────────────────────────────────────────
// 2b. Rework regression (goal-check NEEDS_REVISION): reproduces the judge's
//     own experiment as a permanent test — REAL extra reads injected at a
//     build_status-level load point, bypassing every domain-specific
//     reader entirely (the exact shape a rust-port-23 hoist could take),
//     must still be visible in BOTH the decisions counter and the cells
//     counter. This is what "lowest shared read primitive" (must-have #6)
//     actually has to survive, proven for both stores rather than argued
//     for one.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn injected_reads_at_a_build_status_level_load_point_are_still_counted_for_both_stores() {
    let fixture = host_real_bench_fixture();
    let root = fixture.path();
    let ctx = manual_status_context(root);

    read_accounting::reset();
    let _baseline = queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default());
    let baseline_snap = read_accounting::snapshot();
    assert_eq!(baseline_snap.decisions_journal_parses, 1);
    assert_eq!(baseline_snap.cells_dir_scans, 1);

    // Simulate the judge's hoist: TWO extra real decisions-journal reads
    // and ONE extra real cells-directory scan, called DIRECTLY against the
    // shared primitives from THIS test function — never through
    // `decisions.rs`/`cells.rs`'s own higher-level readers — immediately
    // before calling `build_status` again. A dedup that pre-loads these
    // stores in a new loader living outside both modules would look
    // exactly like this from the counters' point of view.
    read_accounting::reset();
    let _extra_decisions_a: Vec<Value> = bee_core::fsutil::read_jsonl(&bee_core::decisions::decisions_path(root));
    let _extra_decisions_b: Vec<Value> = bee_core::fsutil::read_jsonl(&bee_core::decisions::decisions_path(root));
    let _extra_cells = bee_core::cells::list_cells(root);
    let _status_again = queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default());
    let injected_snap = read_accounting::snapshot();

    assert_eq!(
        injected_snap.decisions_journal_parses,
        1 + 2,
        "two extra real decisions-journal reads injected from OUTSIDE decisions.rs must still show up in the total \
         — a counter placed at decisions.rs's own call sites (the pre-rework placement) would have stayed at 1. \
         This is the property that makes the '1 per store' claim above falsifiable: a dedup that reimplemented a \
         reader instead of calling it would report 1 while really reading twice."
    );
    assert_eq!(
        injected_snap.cells_dir_scans,
        1 + 1,
        "one extra real cells directory scan injected from OUTSIDE cells.rs's own readers must still show up in the total"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// 3. Reach-proof by removal: scribing_debt's cells scan is feature-gated
//    (validation-slice3.md's repaired finding #1 — cells.rs:332 does NOT
//    fire on a null-feature store).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn scribing_debt_cells_scan_is_reachable_only_with_a_resolved_feature() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "c1", "demo", "capped", &[], true, Some("2026-07-26T00:00:00.000Z"));

    // Neutralized: no feature resolves (neither an explicit override nor
    // state.feature) -> the early return at cells.rs:319-324 fires BEFORE
    // any directory scan.
    read_accounting::reset();
    let debt_no_feature = bee_core::cells::scribing_debt(root, None, None);
    let snap_no_feature = read_accounting::snapshot();
    assert_eq!(snap_no_feature.cells_dir_scans, 0, "no resolvable feature -> the scan never happens (count drops to zero)");
    assert_eq!(debt_no_feature["count"], json!(0));

    // Reached: an explicit feature override resolves -> exactly one scan.
    read_accounting::reset();
    let debt_with_feature = bee_core::cells::scribing_debt(root, Some("demo"), Some(0));
    let snap_with_feature = read_accounting::snapshot();
    assert_eq!(snap_with_feature.cells_dir_scans, 1, "a resolved feature -> exactly one directory scan");
    assert_eq!(debt_with_feature["count"], json!(1));
}

// ─────────────────────────────────────────────────────────────────────────
// 4. Reach-proof by removal: recovery's SharedInputs (decisions + cells)
//    is gated behind reaching the crash-candidate track, which is a
//    STRICTER gate than "any session exists" (which only gates the
//    transcript-root scan invocation count — see test 5 below).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn recovery_shared_reads_stay_gated_by_reaching_the_crash_candidate_track() {
    // THE POINT OF THIS TEST AFTER THE DEDUP: the shared memo is LAZY, so
    // moving recovery's three-store load into it must NOT make those reads
    // unconditional. Tiers 0 and 1 below are the proof — they still read
    // zero decisions and zero cells.
    //
    // Tier 0: no sessions at all.
    let fixture0 = minimal_root();
    let root0 = fixture0.path();
    let shared0 = bee_core::shared_reads::SharedReads::new(root0);
    read_accounting::reset();
    let _ = bee_core::recovery::build_recovery_block(&shared0, root0, &root0.join("no-claude-root"), root0, now_ms(), None);
    let snap0 = read_accounting::snapshot();
    assert_eq!(snap0.decisions_journal_parses, 0);
    assert_eq!(snap0.cells_dir_scans, 0);
    assert_eq!(snap0.transcript_root_scan_invocations, 1, "no sessions -> build_recovery_block's own single scan");

    // Tier 1: one session exists, but its heartbeat is FRESH (not stale) ->
    // the session is filtered out before ever reaching the transcript/
    // clean-end/shared-store checks -> decisions/cells stay 0.
    let fixture1 = minimal_root();
    let root1 = fixture1.path();
    let fresh_iso = bee_core::lock::iso8601_millis(now_ms());
    seed_session(root1, "fresh-session", &fresh_iso, None);
    let shared1 = bee_core::shared_reads::SharedReads::new(root1);
    read_accounting::reset();
    let _ = bee_core::recovery::build_recovery_block(&shared1, root1, &root1.join("no-claude-root"), root1, now_ms(), None);
    let snap1 = read_accounting::snapshot();
    assert_eq!(
        snap1.transcript_root_scan_invocations, 1,
        "a session existing no longer costs a SECOND scan — the hoisted scan is passed down (was 2)"
    );
    assert_eq!(snap1.decisions_journal_parses, 0, "a FRESH heartbeat never reaches the crash-candidate track");
    assert_eq!(snap1.cells_dir_scans, 0);

    // Tier 2: one session, stale heartbeat, a transcript with no clean-end
    // trio -> the crash-candidate track IS reached -> the memo fills, once.
    let fixture2 = minimal_root();
    let root2 = fixture2.path();
    let transcript_path = root2.join("transcript.jsonl");
    write_open_transcript(&transcript_path);
    seed_session(root2, "stale-session", "2020-01-01T00:00:00.000Z", Some(&transcript_path));
    let shared2 = bee_core::shared_reads::SharedReads::new(root2);
    read_accounting::reset();
    let block =
        bee_core::recovery::build_recovery_block(&shared2, root2, &root2.join("no-claude-root"), root2, now_ms(), None);
    let snap2 = read_accounting::snapshot();
    assert_eq!(snap2.transcript_root_scan_invocations, 1);
    assert_eq!(snap2.decisions_journal_parses, 1, "the track was reached -> one active_decisions call, now one journal read (was 2)");
    assert_eq!(snap2.cells_dir_scans, 1, "the track was reached -> one cells::list_cells call");
    // Sanity: the candidate really was detected (not just the reads firing
    // for an unrelated reason).
    let candidates = block["candidates"].as_array().expect("candidates array");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["session_id"], json!("stale-session"));
}

// ─────────────────────────────────────────────────────────────────────────
// 5. transcript_root_scan_invocations: gated by "any session exists"
//    (a WEAKER gate than the crash-candidate track above).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn transcript_root_scan_invocation_gated_by_any_session_existing() {
    let empty = minimal_root();
    read_accounting::reset();
    let _ = bee_core::recovery::detect_crash_candidates(empty.path(), empty.path(), &empty.path().join("nope"), empty.path(), now_ms(), None);
    assert_eq!(read_accounting::snapshot().transcript_root_scan_invocations, 0, "detect_crash_candidates returns before scanning when sessions.is_empty()");

    let with_session = minimal_root();
    seed_session(with_session.path(), "any-session", "2020-01-01T00:00:00.000Z", None);
    read_accounting::reset();
    let _ = bee_core::recovery::detect_crash_candidates(
        with_session.path(),
        with_session.path(),
        &with_session.path().join("nope"),
        with_session.path(),
        now_ms(),
        None,
    );
    assert_eq!(
        read_accounting::snapshot().transcript_root_scan_invocations, 1,
        "any session record present (regardless of whether it becomes a genuine candidate) -> the scan runs"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// 6. Unit distinction (validation W1): invocation count vs filesystem-
//    operation count for the SAME store class diverge whenever more than
//    one transcript root is configured.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn transcript_root_stat_ops_scale_with_configured_roots_while_invocations_do_not() {
    let fixture = minimal_root();
    let root = fixture.path();
    let extra_a = root.join("extra-root-a");
    let extra_b = root.join("extra-root-b");
    fs::create_dir_all(&extra_a).unwrap();
    fs::create_dir_all(&extra_b).unwrap();
    set_transcript_roots_config(root, &[&extra_a, &extra_b]);

    read_accounting::reset();
    let _ = bee_core::recovery::scan_transcript_roots(root, &root.join("nonexistent-claude-root"));
    let snap = read_accounting::snapshot();
    assert_eq!(snap.transcript_root_scan_invocations, 1, "one call to scan_transcript_roots");
    assert_eq!(
        snap.transcript_root_stat_ops, 3,
        "three real fs::metadata stats for that ONE call: the default Claude root (missing) + two configured roots"
    );

    // A second call doubles the stat-op count but not proportionally in a
    // way that could be confused with invocation count: two calls, three
    // roots each = six stats, two invocations.
    let _ = bee_core::recovery::scan_transcript_roots(root, &root.join("nonexistent-claude-root"));
    let snap2 = read_accounting::snapshot();
    assert_eq!(snap2.transcript_root_scan_invocations, 2);
    assert_eq!(snap2.transcript_root_stat_ops, 6);
}

// ─────────────────────────────────────────────────────────────────────────
// 7. Reach-proof by removal: cell_dep_reads only fires when ready_cells
//    actually has an OPEN cell carrying a dep to resolve.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn ready_cells_dep_read_count_drops_to_zero_when_no_open_cell_has_a_dep() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "open-with-dep", "demo", "open", &["some-dep"], false, None);
    seed_archived_cell(root, "some-dep", "demo", "capped");

    read_accounting::reset();
    let _ = bee_core::cells::ready_cells(root, Some("demo"));
    assert_eq!(read_accounting::snapshot().cell_dep_reads, 1, "one open cell with one dep -> exactly one read_cell attempt");

    // Neutralize: the SAME cell, now with an empty deps array (removing
    // the condition that gates this bucket) -> the count drops to zero,
    // not merely "the result changes".
    seed_cell(root, "open-with-dep", "demo", "open", &[], false, None);
    read_accounting::reset();
    let _ = bee_core::cells::ready_cells(root, Some("demo"));
    assert_eq!(read_accounting::snapshot().cell_dep_reads, 0, "no open cell carries a dep -> deps_all_capped is never invoked, count drops to zero");
}

/// Complements the reach-proof above with a correctness check on the
/// archive-fallback path itself (validation-slice3.md's repaired finding
/// #2/#3): an open cell whose dep is CAPPED but lives only in the
/// feature-subdirectoried archive must still resolve as ready.
#[test]
fn ready_cells_resolves_a_dep_through_the_archive_fallback() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "open-with-archived-dep", "demo", "open", &["archived-dep"], false, None);
    seed_archived_cell(root, "archived-dep", "demo", "capped");

    read_accounting::reset();
    let ready = bee_core::cells::ready_cells(root, Some("demo"));
    let snap = read_accounting::snapshot();
    assert_eq!(snap.cell_dep_reads, 1);
    assert_eq!(ready.len(), 1, "the archived, capped dep must resolve -> the open cell is ready");
    assert_eq!(ready[0].id, "open-with-archived-dep");

    // Negative control: remove the archived file entirely -> the attempt
    // still happens (same count) but the dep no longer resolves as
    // capped, so the open cell is NOT ready.
    fs::remove_file(root.join(".bee/cells/archive/demo/archived-dep.json")).unwrap();
    read_accounting::reset();
    let ready_after_removal = bee_core::cells::ready_cells(root, Some("demo"));
    let snap_after_removal = read_accounting::snapshot();
    assert_eq!(snap_after_removal.cell_dep_reads, 1, "the read_cell ATTEMPT still happens even on a miss");
    assert!(ready_after_removal.is_empty(), "an unresolvable dep -> the open cell is never ready");
}

// ─────────────────────────────────────────────────────────────────────────
// 7b. THE ARCHIVE TRAP (rust-port-23, validation-slice3.md B3). The single
//     guard standing between the shared-inventory dedup and a silent
//     regression: `list_cells` skips `archive/` while `read_cell` falls
//     back to `archive/<feature>/<id>.json`, so resolving deps against the
//     pre-loaded ACTIVE-ONLY inventory would read an archived capped dep as
//     uncapped and drop ready cells from the recommendation line — with
//     byte-parity green (no parity fixture has an archived dep) and with
//     NO counter moving (cell_dep_reads bundles the archive fallback).
//     Read counts cannot catch this; only this test can.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn ready_cells_from_a_shared_inventory_still_resolves_an_archived_capped_dep() {
    let fixture = minimal_root();
    let root = fixture.path();
    // The dep exists ONLY in the archive, at the feature-subdirectoried
    // path `read_cell` actually searches — it is invisible to `list_cells`,
    // and therefore invisible to the shared inventory.
    seed_cell(root, "open-with-archived-dep", "demo", "open", &["archived-dep"], false, None);
    seed_archived_cell(root, "archived-dep", "demo", "capped");

    let shared = bee_core::shared_reads::SharedReads::new(root);
    read_accounting::reset();
    let ready = bee_core::cells::ready_cells_from(&shared, Some("demo"));
    let snap = read_accounting::snapshot();

    // Confirm the trap's precondition really holds for this fixture: the
    // shared inventory genuinely does NOT contain the dep. Without this,
    // the assertion below could pass for the wrong reason.
    assert!(
        !shared.cells().iter().any(|c| c.id == "archived-dep"),
        "precondition: the archived dep must be absent from the active-only shared inventory"
    );
    assert_eq!(
        ready.len(),
        1,
        "THE ARCHIVE TRAP: dep resolution must still go through read_cell's archive fallback. If ready_cells_from \
         were changed to resolve deps against the shared active-only inventory, this dep would read as uncapped \
         and this cell would silently vanish from the ready list — with every parity leg still green."
    );
    assert_eq!(ready[0].id, "open-with-archived-dep");
    assert_eq!(snap.cells_dir_scans, 1, "the listing came from the shared inventory's single scan");
    assert_eq!(snap.cell_dep_reads, 1, "the dep went through read_cell, not through the inventory");

    // Negative control: remove the archived file -> the same single dep
    // read still happens, but nothing resolves, so the cell is not ready.
    fs::remove_file(root.join(".bee/cells/archive/demo/archived-dep.json")).unwrap();
    let shared_after = bee_core::shared_reads::SharedReads::new(root);
    read_accounting::reset();
    let ready_after = bee_core::cells::ready_cells_from(&shared_after, Some("demo"));
    let snap_after = read_accounting::snapshot();
    assert_eq!(snap_after.cell_dep_reads, 1, "the read_cell ATTEMPT still happens even on a miss");
    assert!(ready_after.is_empty(), "an unresolvable dep -> the open cell is never ready");
}

// ─────────────────────────────────────────────────────────────────────────
// 7c. PER INVOCATION, NEVER WIDER: the memo is born and dropped inside one
//     build_status call. A second invocation re-reads every store, so a
//     write landing between two invocations is observed — the property
//     that distinguishes this from the process-global/static/on-disk cache
//     the plan rejected.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn a_second_build_status_invocation_re_reads_every_store() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "c1", "demo", "open", &[], false, None);
    let ctx = manual_status_context(root);

    read_accounting::reset();
    let first = queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default());
    let after_first = read_accounting::snapshot();
    assert_eq!(after_first.decisions_journal_parses, 1);
    assert_eq!(after_first.cells_dir_scans, 1);
    assert_eq!(after_first.transcript_root_scan_invocations, 1);
    assert_eq!(first["cells"]["open"], json!(1));

    // A write between the two invocations.
    seed_cell(root, "c2", "demo", "open", &[], false, None);

    // Deliberately NOT reset: the counters accumulate across both calls, so
    // a memo that survived the first invocation would leave these at 1.
    let second = queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default());
    let after_second = read_accounting::snapshot();
    assert_eq!(after_second.decisions_journal_parses, 2, "invocation 2 parses the journal again — no cache spans invocations");
    assert_eq!(after_second.cells_dir_scans, 2, "invocation 2 scans the cells directory again");
    assert_eq!(after_second.transcript_root_scan_invocations, 2, "invocation 2 scans the transcript roots again");
    assert_eq!(second["cells"]["open"], json!(2), "the write between invocations is observed, not served from a stale memo");
}

// ─────────────────────────────────────────────────────────────────────────
// 7d. Degraded stores read once and still degrade. The mjs-oracle proof
//     that the DEGRADED VALUES themselves are unchanged lives in
//     `crates/bee-core/tests/status_readers_a.rs` (which diffs every reader
//     against the real mjs module on absent/empty/malformed fixtures, and
//     is part of this cell's verify); what is asserted here is the read
//     COUNT on exactly those store shapes, which no oracle can see.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn absent_empty_and_malformed_stores_are_each_read_exactly_once() {
    // Absent: `minimal_root` writes no decisions.jsonl, no cells/ directory
    // and no capture queue at all.
    let absent = minimal_root();
    let ctx_absent = manual_status_context(absent.path());
    read_accounting::reset();
    let payload_absent = queen_bee::status::build_status(&ctx_absent, queen_bee::status::StatusOptions::default());
    let snap_absent = read_accounting::snapshot();
    assert_eq!(snap_absent.decisions_journal_parses, 1, "a MISSING journal is still exactly one read attempt");
    assert_eq!(snap_absent.cells_dir_scans, 1, "a MISSING cells directory is still exactly one scan attempt");
    assert_eq!(snap_absent.transcript_root_scan_invocations, 1);
    assert_eq!(payload_absent["cells"]["open"], json!(0));
    assert_eq!(payload_absent["recent_decisions"], json!([]));

    // Empty: the files exist but hold nothing.
    let empty = minimal_root();
    write_file(&empty.path().join(".bee/decisions.jsonl"), "");
    fs::create_dir_all(empty.path().join(".bee/cells")).unwrap();
    write_file(&empty.path().join(".bee/capture-queue.jsonl"), "");
    let ctx_empty = manual_status_context(empty.path());
    read_accounting::reset();
    let payload_empty = queen_bee::status::build_status(&ctx_empty, queen_bee::status::StatusOptions::default());
    let snap_empty = read_accounting::snapshot();
    assert_eq!(snap_empty.decisions_journal_parses, 1);
    assert_eq!(snap_empty.cells_dir_scans, 1);
    assert_eq!(snap_empty.transcript_root_scan_invocations, 1);
    assert_eq!(payload_empty["recent_decisions"], json!([]));

    // Malformed: an unparseable journal line and an unparseable cell file —
    // both are skipped per-record (fail-open), never a panic, and still one
    // read each.
    let malformed = minimal_root();
    write_file(&malformed.path().join(".bee/decisions.jsonl"), "{not json at all\n{\"type\":\"decide\",\"id\":\"ok1\",\"date\":\"2026-07-26T00:00:00.000Z\",\"decision\":\"kept\"}\n");
    write_file(&malformed.path().join(".bee/cells/broken.json"), "{ nope");
    seed_cell(malformed.path(), "fine", "demo", "open", &[], false, None);
    let ctx_malformed = manual_status_context(malformed.path());
    read_accounting::reset();
    let payload_malformed = queen_bee::status::build_status(&ctx_malformed, queen_bee::status::StatusOptions::default());
    let snap_malformed = read_accounting::snapshot();
    assert_eq!(snap_malformed.decisions_journal_parses, 1);
    assert_eq!(snap_malformed.cells_dir_scans, 1);
    assert_eq!(snap_malformed.transcript_root_scan_invocations, 1);
    assert_eq!(payload_malformed["cells"]["open"], json!(1), "the malformed cell file is skipped, the good one still counted");
    assert_eq!(
        payload_malformed["recent_decisions"][0]["id"],
        json!("ok1"),
        "the malformed journal line is skipped, the good one still parsed"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// 8. The hook entry points (advisor note 1 — the sixth blocker): baselined
//    by rust-port-22 and RE-ASSERTED here after rust-port-23 moved both
//    call sites onto the shared-read signature. These are the tests that
//    would catch an eager shared-read shape turning chain-nudge's
//    feature-gated scan into an unconditional one on every hook event.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn chain_nudge_cells_scan_zero_with_no_active_feature() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_hook_lib_marker(root);
    set_state(root, "swarming", None);

    read_accounting::reset();
    let argv: Vec<String> = Vec::new();
    let code = queen_bee::hooks::chain_nudge::run(&argv, &hook_payload(root, "SubagentStop"));
    assert_eq!(code, 0);
    assert_eq!(read_accounting::snapshot().cells_dir_scans, 0, "no active feature -> scribing_debt's early return -> zero cells scans");
}

#[test]
fn chain_nudge_cells_scan_exactly_one_with_active_feature() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_hook_lib_marker(root);
    set_state(root, "swarming", Some("demo-feature"));

    read_accounting::reset();
    let argv: Vec<String> = Vec::new();
    let code = queen_bee::hooks::chain_nudge::run(&argv, &hook_payload(root, "SubagentStop"));
    assert_eq!(code, 0);
    assert_eq!(read_accounting::snapshot().cells_dir_scans, 1, "an active feature -> scribing_debt performs exactly one cells scan");
}

#[test]
fn state_sync_cell_counts_scan_is_unconditional_every_run() {
    // No active feature: state-sync's own cell_counts() still scans (it
    // never gates on feature at all — a DIFFERENT shape from chain-nudge's
    // scribing_debt).
    let fixture_no_feature = minimal_root();
    seed_hook_lib_marker(fixture_no_feature.path());
    set_state(fixture_no_feature.path(), "idle", None);
    read_accounting::reset();
    let argv: Vec<String> = Vec::new();
    let code = queen_bee::hooks::state_sync::run(&argv, &hook_payload(fixture_no_feature.path(), "Stop"));
    assert_eq!(code, 0);
    assert_eq!(read_accounting::snapshot().cells_dir_scans, 1, "state-sync's cell_counts() scans unconditionally, feature or not");

    let fixture_with_feature = minimal_root();
    seed_hook_lib_marker(fixture_with_feature.path());
    set_state(fixture_with_feature.path(), "swarming", Some("demo-feature"));
    read_accounting::reset();
    let code2 = queen_bee::hooks::state_sync::run(&argv, &hook_payload(fixture_with_feature.path(), "Stop"));
    assert_eq!(code2, 0);
    assert_eq!(read_accounting::snapshot().cells_dir_scans, 1, "still exactly one scan with a feature active — state-sync's count is unconditional, not feature-gated");
}

// ─────────────────────────────────────────────────────────────────────────
// 9. Negative control: the seam must never change observable output.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn build_status_output_is_byte_identical_regardless_of_prior_counter_state() {
    let fixture = minimal_root();
    let root = fixture.path();
    seed_cell(root, "c1", "demo", "open", &[], false, None);
    let ctx = manual_status_context(root);

    read_accounting::reset();
    let first = queen_bee::status::to_json_stdout(&queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default()));

    // Deliberately do NOT reset — the counters now carry whatever the
    // first build_status call left behind, simulating a process that has
    // already served other invocations on this thread.
    let second = queen_bee::status::to_json_stdout(&queen_bee::status::build_status(&ctx, queen_bee::status::StatusOptions::default()));

    assert_eq!(first, second, "build_status's stdout payload must be byte-identical no matter what the read-accounting counters hold");
}
