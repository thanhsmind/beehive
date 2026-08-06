// `state start-feature`, `state workflows list`, `state workflows close`.
//
// WHY AN INTEGRATION TEST, NOT A UNIT TEST. All three verbs are
// `fn run_x(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode>`,
// printing as a side effect, and their root comes from
// `verbs::reservations::emit::prelude` reading `std::env::current_dir()` —
// there is no `run_x_body(root, flags)` split for this trio (unlike
// `state_group::set_gate::run_set_body`). Calling them in-process would mean
// mutating the test runner's own cwd across parallel tests, which is exactly
// the kind of assertion-on-nothing the cell's own notes warn against. The
// sanctioned home is the built binary against a temp repo root, the same
// shape `cells_archive_sweep.rs` already uses for `cells archive` / `close`.
//
// Everything below these three verbs — ensure_workflow_record_for_feature
// idempotence, close_workflows_for_feature's keep-only-named rule,
// seed_legacy_workflows gating, create_workflow's own refusals,
// resolve_active_feature_for_workflows_close's Ok/Err split — already has
// unit coverage in verbs/state_group/tests.rs and verbs/workflow_store/tests.rs.
// These tests cover exactly the layer above that: flag parsing, target
// resolution, the refusal wording, the emitted payload, and the exit code.

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> PathBuf {
    let mut dir = std::env::current_exe().expect("test binary path");
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    let bin = dir.join(format!("bee{}", std::env::consts::EXE_SUFFIX));
    assert!(bin.is_file(), "built bee binary not found at {}", bin.display());
    bin
}

/// A bare repo: onboarding complete, idle default state, no cells, no
/// reservations, no workflow records.
fn fixture(base: &Path) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    write_state(&dir, "idle", "null");
    dir
}

fn write_state(repo: &Path, phase: &str, feature_json: &str) {
    std::fs::write(
        repo.join(".bee/state.json"),
        format!(r#"{{"phase":"{phase}","feature":{feature_json},"gates":{{}}}}"#),
    )
    .unwrap();
}

fn write_cell(repo: &Path, id: &str, feature: &str, status: &str) {
    std::fs::write(
        repo.join(".bee/cells").join(format!("{id}.json")),
        format!(r#"{{"id":"{id}","feature":"{feature}","status":"{status}","title":"t"}}"#),
    )
    .unwrap();
}

fn state_json(repo: &Path) -> String {
    std::fs::read_to_string(repo.join(".bee/state.json")).unwrap()
}

fn workflows_dir(repo: &Path) -> PathBuf {
    repo.join(".bee/runtime/workflows")
}

fn run(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn run_json(cwd: &Path, args: &[&str]) -> (i32, serde_json::Value) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let v = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("`bee {}` did not print JSON: {stdout} ({e})", args.join(" ")));
    (out.status.code().unwrap_or(-1), v)
}

/// `state workflows list --json`, sorted by feature so assertions are
/// order-independent of the store's own (newest-first) sort.
fn list_by_feature(repo: &Path) -> Vec<serde_json::Value> {
    let (code, v) = run_json(repo, &["state", "workflows", "list", "--json"]);
    assert_eq!(code, 0, "{v}");
    let mut records: Vec<serde_json::Value> = v.as_array().unwrap().clone();
    records.sort_by(|a, b| a["feature"].as_str().cmp(&b["feature"].as_str()));
    records
}

// ── state start-feature: the clean path ────────────────────────────────────

#[test]
fn a_clean_start_feature_creates_the_workflow_record_and_it_is_listed() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    let (code, v) =
        run_json(&repo, &["state", "start-feature", "--feature", "wf-a", "--json"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["feature"], "wf-a");
    assert_eq!(v["phase"], "exploring");

    let records = list_by_feature(&repo);
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["feature"], "wf-a");
    assert_eq!(records[0]["status"], "active");
    assert_eq!(records[0]["phase"], "exploring");
    assert!(records[0]["id"].as_str().unwrap().starts_with("wf-"), "{records:?}");

    // The plain-text list line names the same four fields the JSON does.
    let (code, text) = run(&repo, &["state", "workflows", "list"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("feature=wf-a"), "{text}");
    assert!(text.contains("status=active"), "{text}");
    assert!(text.contains("phase=exploring"), "{text}");
}

#[test]
fn workflows_list_is_empty_before_any_feature_starts() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let (code, text) = run(&repo, &["state", "workflows", "list"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("No workflow records."), "{text}");
    assert_eq!(list_by_feature(&repo).len(), 0);
}

#[test]
fn workflows_list_reports_every_record_with_its_own_status_and_phase() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    // Two lanes at different phases, both left live …
    run(&repo, &["state", "start-feature", "--feature", "wf-a", "--as-lane", "--phase", "planning"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-b", "--as-lane", "--phase", "swarming"]);
    // … and a third, closed by --id, so the list must report a MIX of
    // statuses, not just "whatever is live".
    let before = list_by_feature(&repo);
    let wf_b_id = before.iter().find(|r| r["feature"] == "wf-b").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let (code, out) = run(&repo, &["state", "workflows", "close", "--id", &wf_b_id]);
    assert_eq!(code, 0, "{out}");

    let records = list_by_feature(&repo);
    assert_eq!(records.len(), 2, "{records:?}");
    assert_eq!(records[0]["feature"], "wf-a");
    assert_eq!(records[0]["status"], "active");
    assert_eq!(records[0]["phase"], "planning");
    assert_eq!(records[1]["feature"], "wf-b");
    assert_eq!(records[1]["status"], "closed");
    assert_eq!(records[1]["phase"], "swarming");
}

// ── state start-feature: guarded refusals, each fails closed ───────────────

#[test]
fn start_feature_refuses_a_non_idle_non_terminal_phase_with_zero_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    // `feature: null` so the C1 legacy-seeding step (which runs BEFORE this
    // guard, on every call, independent of whether the call itself succeeds
    // — see run_start_feature's own comment) has nothing to seed either; the
    // phase-alone refusal is the only thing under test here.
    write_state(&repo, "swarming", "null");
    let before = state_json(&repo);

    let (code, out) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("current phase is \"swarming\""), "{out}");
    assert!(out.contains("not idle or the terminal alias"), "{out}");

    assert_eq!(state_json(&repo), before, "a refused start must touch no byte of state.json");
    assert!(!workflows_dir(&repo).exists(), "no workflow record may be created");
}

#[test]
fn start_feature_refuses_over_an_active_reservation_with_zero_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let before = state_json(&repo);
    let (rc, rout) = run(
        &repo,
        &["reservations", "reserve", "--agent", "a1", "--cell", "c1", "--path", "foo/bar.txt"],
    );
    assert_eq!(rc, 0, "{rout}");

    let (code, out) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("active reservation(s) remain"), "{out}");
    assert!(out.contains("a1:foo/bar.txt"), "the blocker must be named: {out}");

    assert_eq!(state_json(&repo), before, "a refused start must touch no byte of state.json");
    assert!(!workflows_dir(&repo).exists(), "no workflow record may be created");
}

#[test]
fn start_feature_refuses_over_a_claimed_cell_with_zero_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    write_cell(&repo, "c-1", "other-feat", "claimed");
    let before_state = state_json(&repo);
    let before_cell = std::fs::read_to_string(repo.join(".bee/cells/c-1.json")).unwrap();

    let (code, out) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("claimed cell(s) remain: c-1"), "the blocker must be named: {out}");

    assert_eq!(state_json(&repo), before_state);
    assert_eq!(
        std::fs::read_to_string(repo.join(".bee/cells/c-1.json")).unwrap(),
        before_cell,
        "the claimed cell itself must be untouched"
    );
    assert!(!workflows_dir(&repo).exists(), "no workflow record may be created");
}

#[test]
fn start_feature_refuses_over_a_nonterminal_cell_of_the_prior_feature_with_zero_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    write_state(&repo, "idle", "\"prior-feat\"");
    write_cell(&repo, "c-2", "prior-feat", "open");
    let before_state = state_json(&repo);
    let before_cell = std::fs::read_to_string(repo.join(".bee/cells/c-2.json")).unwrap();

    let (code, out) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(
        out.contains("prior feature \"prior-feat\" has nonterminal cell(s): c-2(open)"),
        "the blocker must be named: {out}"
    );

    // state.json and the blocking cell are untouched by the refusal — the
    // named `prior-feat` picking up a workflow record here is the unrelated
    // C1 legacy-seeding step (run_start_feature seeds BEFORE this guard, on
    // every call reaching it, success or refusal — see
    // start_feature_refuses_a_non_idle_non_terminal_phase_with_zero_mutations'
    // comment), not a mutation this refusal itself performed. What matters
    // to THIS guard is that the refused TARGET feature never gets one.
    assert_eq!(state_json(&repo), before_state);
    assert_eq!(std::fs::read_to_string(repo.join(".bee/cells/c-2.json")).unwrap(), before_cell);
    assert!(
        !list_by_feature(&repo).iter().any(|r| r["feature"] == "wf-a"),
        "the refused target feature must never get a workflow record"
    );
}

#[test]
fn start_feature_refuses_a_live_workflow_already_standing_for_the_target_feature() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let (c1, o1) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_eq!(c1, 0, "{o1}");
    // Wind the default record back to idle so the SAME refusal under test —
    // the live-workflow guard, not the non-idle-phase one — is the one that
    // fires on the second attempt.
    write_state(&repo, "idle", "null");
    let before_state = state_json(&repo);
    let before_records = list_by_feature(&repo);

    let (code, out) = run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(
        out.contains("a live workflow already exists for feature \"wf-a\""),
        "the blocker must be named: {out}"
    );

    assert_eq!(state_json(&repo), before_state, "a refused start must touch no byte of state.json");
    assert_eq!(
        list_by_feature(&repo),
        before_records,
        "the existing live workflow record must be untouched"
    );
}

// ── state workflows close ───────────────────────────────────────────────────

#[test]
fn workflows_close_by_feature_closes_only_that_features_records_and_leaves_others_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-a", "--as-lane"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-b", "--as-lane"]);
    let before_b = list_by_feature(&repo).into_iter().find(|r| r["feature"] == "wf-b").unwrap();

    let (code, out) = run(&repo, &["state", "workflows", "close", "--feature", "wf-a"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Closed 1 workflow record(s) for feature \"wf-a\""), "{out}");

    let records = list_by_feature(&repo);
    let a = records.iter().find(|r| r["feature"] == "wf-a").unwrap();
    let b = records.iter().find(|r| r["feature"] == "wf-b").unwrap();
    assert_eq!(a["status"], "closed");
    assert_eq!(b, &before_b, "a feature not named by --feature must be untouched, byte for byte");
}

#[test]
fn workflows_close_by_feature_refuses_the_currently_active_feature() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-c"]);
    let before = list_by_feature(&repo);

    let (code, out) = run(&repo, &["state", "workflows", "close", "--feature", "wf-c"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("is the currently active feature"), "{out}");
    assert!(out.contains("use --id"), "{out}");

    assert_eq!(list_by_feature(&repo), before, "a refused close must touch no record");
}

#[test]
fn workflows_close_by_id_closes_only_the_named_record() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-a", "--as-lane"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-b", "--as-lane"]);
    let before = list_by_feature(&repo);
    let id_a = before.iter().find(|r| r["feature"] == "wf-a").unwrap()["id"].as_str().unwrap();

    let (code, out) = run(&repo, &["state", "workflows", "close", "--id", id_a]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("Closed 1 workflow record: {id_a}")), "{out}");

    let records = list_by_feature(&repo);
    assert_eq!(records.iter().find(|r| r["feature"] == "wf-a").unwrap()["status"], "closed");
    assert_eq!(
        records.iter().find(|r| r["feature"] == "wf-b").unwrap(),
        before.iter().find(|r| r["feature"] == "wf-b").unwrap(),
        "wf-b must be untouched"
    );
}

#[test]
fn workflows_close_requires_exactly_one_selector() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-a"]);
    let before = list_by_feature(&repo);

    let (code, out) = run(&repo, &["state", "workflows", "close"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("requires exactly one of"), "{out}");
    assert_eq!(list_by_feature(&repo), before);

    let (code, out) =
        run(&repo, &["state", "workflows", "close", "--feature", "wf-a", "--all-but-active"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("requires exactly one of"), "{out}");
    assert_eq!(list_by_feature(&repo), before);
}

// ── state workflows close --all-but-active ──────────────────────────────────

#[test]
fn close_all_but_active_closes_every_other_live_record_and_spares_the_active_one() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    // The default (non-lane) start makes wf-c the active feature; the two
    // lanes started after it are untouched by its own outgoing-work close
    // (that close only ever runs on a DEFAULT start), so both stay live.
    run(&repo, &["state", "start-feature", "--feature", "wf-c"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-d", "--as-lane"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-e", "--as-lane"]);
    let before = list_by_feature(&repo);
    assert!(before.iter().all(|r| r["status"] == "active"), "{before:?}");

    let (code, out) = run(&repo, &["state", "workflows", "close", "--all-but-active"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Closed 2 workflow record(s), kept active feature \"wf-c\""), "{out}");

    let records = list_by_feature(&repo);
    let c = records.iter().find(|r| r["feature"] == "wf-c").unwrap();
    assert_eq!(c, before.iter().find(|r| r["feature"] == "wf-c").unwrap(), "the active feature must be untouched");
    assert_eq!(records.iter().find(|r| r["feature"] == "wf-d").unwrap()["status"], "closed");
    assert_eq!(records.iter().find(|r| r["feature"] == "wf-e").unwrap()["status"], "closed");
}

#[test]
fn close_all_but_active_refuses_when_there_is_nothing_else_live() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-c"]);
    let before = list_by_feature(&repo);

    let (code, out) = run(&repo, &["state", "workflows", "close", "--all-but-active"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("nothing to close"), "{out}");
    assert!(out.contains("no live workflow record other than the active feature"), "{out}");

    assert_eq!(list_by_feature(&repo), before, "a refused close must touch no record");
}

/// resolve_active_feature_for_workflows_close's `Err` arm (its own unit
/// coverage lives at state_group/tests.rs:1070) — driven here through the
/// verb: an unreadable default record makes "active" unresolvable, and
/// --all-but-active refuses rather than silently degrading into "close
/// everything".
#[test]
fn close_all_but_active_refuses_rather_than_degrade_to_all_when_active_is_unresolvable() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-a", "--as-lane"]);
    run(&repo, &["state", "start-feature", "--feature", "wf-b", "--as-lane"]);
    let before = list_by_feature(&repo);
    // Corrupt the default record AFTER both lanes are live: readStateStrict
    // now throws, so resolveActiveFeatureForWorkflowsClose can't answer.
    std::fs::write(repo.join(".bee/state.json"), "{not json").unwrap();

    let (code, out) = run(&repo, &["state", "workflows", "close", "--all-but-active"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("would silently degrade into \"all\""), "{out}");
    assert!(out.contains("Nothing was closed."), "{out}");
    assert!(out.contains("Underlying resolution failure:"), "{out}");

    assert_eq!(list_by_feature(&repo), before, "a refused close must touch no record");
}

#[test]
fn close_by_feature_refuses_rather_than_guess_when_active_is_unresolvable() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    run(&repo, &["state", "start-feature", "--feature", "wf-a", "--as-lane"]);
    let before = list_by_feature(&repo);
    std::fs::write(repo.join(".bee/state.json"), "{not json").unwrap();

    let (code, out) = run(&repo, &["state", "workflows", "close", "--feature", "wf-a"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("the currently active feature could not be resolved"), "{out}");
    assert!(out.contains("Nothing was closed."), "{out}");

    assert_eq!(list_by_feature(&repo), before, "a refused close must touch no record");
}
