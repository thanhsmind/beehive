// `bee cells archive --all-but-active` — the sweep.
//
// WHY IT EXISTS. `.bee/cells/` is on the hot read path: `list_cells` PARSES
// every file in it on every `status` and `orient`. Cells accumulate for the
// life of a repo and nothing retires them, so the cost of asking "where am I"
// grows monotonically with the amount of finished work. On the bee repo that
// had reached 455 active cell files across 118 features, 441 of them belonging
// to features that were completely done: `orient` 300ms, `status` 255ms. After
// archiving the terminal features: 110ms and 93ms.
//
// `cells archive --feature <f>` already existed and does exactly the right
// thing — one feature at a time. Retiring 115 finished features through it
// takes a shell loop, which is the shape of a missing verb: the relief valve
// was there and the pressure still built, because using it was a chore nobody
// does on a Tuesday.
//
// THE RISK IT MUST NOT TAKE. "All but active" is only safe while "active" is
// known. With an unresolvable active feature the phrase silently means "all",
// and the one feature whose cells must never leave the scan path is the
// in-flight one. `state workflows close --all-but-active` already refuses on
// exactly that ground; this refuses the same way, for the same reason.

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

/// A repo with three features: one fully terminal, one holding an open cell,
/// and one that is the active feature (terminal, but off limits).
fn fixture(base: &Path, active: Option<&str>) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    let feature_field = match active {
        Some(f) => format!("\"{f}\""),
        None => "null".to_string(),
    };
    std::fs::write(
        dir.join(".bee/state.json"),
        format!(r#"{{"phase":"executing","feature":{feature_field},"gates":{{}}}}"#),
    )
    .unwrap();

    let cell = |id: &str, feature: &str, status: &str| {
        std::fs::write(
            dir.join(".bee/cells").join(format!("{id}.json")),
            format!(r#"{{"id":"{id}","feature":"{feature}","status":"{status}","title":"t"}}"#),
        )
        .unwrap();
    };
    cell("done-1", "all-done", "capped");
    cell("done-2", "all-done", "dropped");
    cell("live-1", "half-done", "capped");
    cell("live-2", "half-done", "open");
    cell("cur-1", "current", "capped");
    dir
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

/// stdout alone. Every command trails a `[bee] <verb> Nms` timing line on
/// stderr, so the combined stream above is for prose assertions only.
fn run_json(cwd: &Path, args: &[&str]) -> (i32, serde_json::Value) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let v = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("`bee {}` did not print JSON: {stdout} ({e})", args.join(" ")));
    (out.status.code().unwrap_or(-1), v)
}

fn active_cell_ids(repo: &Path) -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(repo.join(".bee/cells"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .filter(|n| n.ends_with(".json"))
        .collect();
    ids.sort();
    ids
}

#[test]
fn the_sweep_retires_finished_features_and_keeps_everything_still_in_play() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), Some("current"));

    let (code, v) = run_json(&repo, &["cells", "archive", "--all-but-active", "--json"]);
    assert_eq!(code, 0, "{v}");

    assert_eq!(v["archived"].as_array().unwrap().len(), 1, "{v}");
    assert_eq!(v["archived"][0]["feature"], "all-done");
    assert_eq!(v["archived"][0]["moved"], 2);
    assert_eq!(v["counts"]["capped"], 1.0);
    assert_eq!(v["counts"]["dropped"], 1.0);

    // The two it kept are NAMED with the reason it kept them. A sweep that
    // passed over a feature in silence would be indistinguishable from one
    // that had nothing to do.
    let skipped: Vec<(String, String)> = v["skipped"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["feature"].as_str().unwrap().into(), s["reason"].as_str().unwrap().into()))
        .collect();
    assert_eq!(skipped.len(), 2, "{v}");
    let by = |f: &str| skipped.iter().find(|(n, _)| n == f).map(|(_, r)| r.clone());
    assert!(by("current").unwrap().contains("active feature"), "{skipped:?}");
    assert!(by("half-done").unwrap().contains("live-2 (open)"), "{skipped:?}");

    // The store agrees with the report.
    assert_eq!(active_cell_ids(&repo), ["cur-1.json", "live-1.json", "live-2.json"]);
    assert!(repo.join(".bee/cells/archive/all-done/done-1.json").is_file());
    assert!(repo.join(".bee/cells/archive/all-done/done-2.json").is_file());
    // …and the summary ledger carries the counts, so `status` can report an
    // honest archived total without walking the archive tree.
    let summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(repo.join(".bee/cells/archive/summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["all-done"]["capped"], 1.0);
    assert_eq!(summary["all-done"]["dropped"], 1.0);

    // Idempotent: a second sweep has nothing left and says so without failing.
    let (code, v) = run_json(&repo, &["cells", "archive", "--all-but-active", "--json"]);
    assert_eq!(code, 0, "{v}");
    assert!(v["archived"].as_array().unwrap().is_empty(), "{v}");
    assert_eq!(active_cell_ids(&repo), ["cur-1.json", "live-1.json", "live-2.json"]);
}

#[test]
fn an_unresolvable_active_feature_refuses_the_whole_sweep() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), None);
    let before = active_cell_ids(&repo);

    let (code, out) = run(&repo, &["cells", "archive", "--all-but-active"]);
    assert_ne!(code, 0, "a refusal must exit non-zero: {out}");
    assert!(out.contains("would degrade into \"all\""), "{out}");
    assert!(out.contains("Nothing was archived"), "{out}");
    assert_eq!(active_cell_ids(&repo), before, "the refusal must touch no file");
    assert!(!repo.join(".bee/cells/archive").exists(), "not even the archive dir");
}

#[test]
fn the_two_modes_are_exclusive() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), Some("current"));
    let before = active_cell_ids(&repo);

    for argv in [
        vec!["cells", "archive", "--all-but-active", "--feature", "all-done"],
        // …and neither is just as wrong as both. This shape used to fall
        // through to the router's generic unsupported-shape line; it now says
        // which of the two the caller has to pick.
        vec!["cells", "archive"],
    ] {
        let (code, out) = run(&repo, &argv);
        assert_ne!(code, 0, "{argv:?} -> {out}");
        assert!(out.contains("requires exactly one of"), "{argv:?} -> {out}");
        assert_eq!(active_cell_ids(&repo), before, "{argv:?}");
    }
}

/// The single-feature form is untouched by the sweep's arrival — including its
/// own refusals, which the sweep reaches through the same code.
#[test]
fn the_single_feature_form_still_behaves_exactly_as_it_did() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), Some("current"));

    let (code, out) = run(&repo, &["cells", "archive", "--feature", "half-done"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("non-terminal cell(s)") && out.contains("live-2"), "{out}");

    let (code, out) = run(&repo, &["cells", "archive", "--feature", "current"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("is the active feature"), "{out}");

    let (code, out) = run(&repo, &["cells", "archive", "--feature", "all-done"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("2 cell(s) moved"), "{out}");
    assert_eq!(active_cell_ids(&repo), ["cur-1.json", "live-1.json", "live-2.json"]);
}

// ─── the automatic half ────────────────────────────────────────────────────
//
// The sweep above is a command someone has to run. What actually keeps the
// store from growing is that nobody has to: `bee close` retires the feature
// it just closed, and `status`/`orient` name whatever never went through
// close. Between them the only cells left in the hot scan path are the ones
// still in play.

/// A repo whose declared test command is `exit <code>`, with `feature` active.
fn close_fixture(base: &Path, code: i32, opt_out: bool) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    let opt = if opt_out { r#","cells_archive_on_close":false"# } else { "" };
    std::fs::write(
        dir.join(".bee/config.json"),
        format!(r#"{{"commands":{{"test":"exit {code}"}}{opt}}}"#),
    )
    .unwrap();
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"phase":"executing","feature":"shipping","gates":{}}"#,
    )
    .unwrap();
    let cell = |id: &str, feature: &str, status: &str| {
        std::fs::write(
            dir.join(".bee/cells").join(format!("{id}.json")),
            format!(r#"{{"id":"{id}","feature":"{feature}","status":"{status}","title":"t"}}"#),
        )
        .unwrap();
    };
    cell("s-1", "shipping", "capped");
    cell("s-2", "shipping", "dropped");
    cell("o-1", "other", "open");
    dir
}

#[test]
fn a_green_close_retires_the_feature_it_just_closed() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = close_fixture(tmp.path(), 0, false);

    let (code, out) = run(&repo, &["close", "--feature", "shipping"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Retired \"shipping\": 2 cell(s) moved"), "{out}");
    // It names the way back. A tidy-up nobody can reverse is a deletion.
    assert!(out.contains("bee cells unarchive --feature shipping"), "{out}");

    assert_eq!(active_cell_ids(&repo), ["o-1.json"]);
    assert!(repo.join(".bee/cells/archive/shipping/s-1.json").is_file());
    assert!(repo.join(".bee/cells/archive/shipping/s-2.json").is_file());

    // Note what is NOT required: `shipping` is still state.feature. Close has
    // just run the declared suite green over it, which is the evidence
    // `cells archive`'s active-feature guard stands in for — so close carries
    // its own guard (all cells terminal) instead of that proxy.
    let state = std::fs::read_to_string(repo.join(".bee/state.json")).unwrap();
    assert!(state.contains("\"shipping\""), "the fixture's premise: {state}");
}

#[test]
fn a_feature_that_closes_green_holding_live_work_keeps_its_cells() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = close_fixture(tmp.path(), 0, false);
    // close's ONLY blocking door is tests, so a feature can close green with
    // an open cell. That cell belongs in the active scan.
    let (code, out) = run(&repo, &["close", "--feature", "other"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Cells kept in the active scan"), "{out}");
    assert!(out.contains("o-1 (open)"), "the reason must name the cell: {out}");
    assert!(repo.join(".bee/cells/o-1.json").is_file());
}

/// D7 (docs/history/test-doctrine/CONTEXT.md): `bee close` no longer spawns
/// `commands.test` at all, so `close_fixture`'s `exit <code>` declaration
/// can no longer make close refuse. The tests door now refuses on recorded
/// proof instead (verbs/cells/proof.rs) — a capped cell whose report
/// carries no valid proof line stands in for the old red run.
#[test]
fn a_red_close_retires_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = close_fixture(tmp.path(), 1, false);
    std::fs::write(
        repo.join(".bee/cells/s-1.json"),
        r#"{"id":"s-1","feature":"shipping","status":"capped","title":"t","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"","deviations":[]}}}"#,
    )
    .unwrap();
    let before = active_cell_ids(&repo);

    let (code, out) = run(&repo, &["close", "--feature", "shipping"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(active_cell_ids(&repo), before, "a refused close must touch no cell");
    assert!(!repo.join(".bee/cells/archive").exists(), "{out}");
}

#[test]
fn the_opt_out_is_honoured_and_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = close_fixture(tmp.path(), 0, true);
    let before = active_cell_ids(&repo);

    let (code, out) = run(&repo, &["close", "--feature", "shipping"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(active_cell_ids(&repo), before);
    // A switch the owner set is not news — no line either way.
    assert!(!out.contains("Retired"), "{out}");
    assert!(!out.contains("Cells kept"), "{out}");
}

/// The backstop. Close only ever speaks for features that go through it; a
/// repo that finished work before close existed (or abandoned a lane) carries
/// the cost silently forever. `status` names it once the backlog is worth a
/// command, and stays quiet below that.
#[test]
fn status_names_finished_features_that_never_went_through_close() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = close_fixture(tmp.path(), 0, false);
    let cell = |id: &str, feature: &str, status: &str| {
        std::fs::write(
            repo.join(".bee/cells").join(format!("{id}.json")),
            format!(r#"{{"id":"{id}","feature":"{feature}","status":"{status}","title":"t"}}"#),
        )
        .unwrap();
    };

    // Below the floor: real, but not yet worth a line.
    for n in 1..=3 {
        cell(&format!("old{n}-1"), &format!("old{n}"), "capped");
    }
    let (_, v) = run_json(&repo, &["status", "--json"]);
    assert_eq!(v["cells"]["archivable"]["features"], 3, "{}", v["cells"]);
    let (_, text) = run(&repo, &["status"]);
    assert!(!text.contains("Finished features not retired"), "{text}");

    // Over it: one line, one command, and it names what it counted.
    for n in 4..=6 {
        cell(&format!("old{n}-1"), &format!("old{n}"), "capped");
    }
    let (_, v) = run_json(&repo, &["status", "--json"]);
    assert_eq!(v["cells"]["archivable"]["features"], 6);
    assert_eq!(v["cells"]["archivable"]["cells"], 6);
    let (_, text) = run(&repo, &["status"]);
    assert!(text.contains("Finished features not retired: 6 feature(s), 6 cell(s)"), "{text}");
    assert!(text.contains("bee cells archive --all-but-active"), "{text}");

    // The active feature and anything holding live work are never counted —
    // otherwise the nudge could never be cleared.
    let ids = v["cells"]["archivable"]["ids"].as_array().unwrap();
    assert!(!ids.iter().any(|i| i == "shipping" || i == "other"), "{ids:?}");

    // …and running what it names clears it.
    let (code, out) = run(&repo, &["cells", "archive", "--all-but-active"]);
    assert_eq!(code, 0, "{out}");
    let (_, text) = run(&repo, &["status"]);
    assert!(!text.contains("Finished features not retired"), "{text}");
}
