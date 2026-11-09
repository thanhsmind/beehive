use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(root.join("docs").join("knowledge")).unwrap();
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    Fixture { _dir: dir, root }
}

fn write_cell(root: &Path, archived: bool, id: &str, status: &str, trace: Value) {
    let dir = if archived {
        root.join(".bee").join("cells").join("archive").join("demo")
    } else {
        root.join(".bee").join("cells")
    };
    std::fs::create_dir_all(&dir).unwrap();
    let cell = json!({"id": id, "feature": "demo", "status": status, "trace": trace});
    std::fs::write(dir.join(format!("{id}.json")), serde_json::to_string_pretty(&cell).unwrap()).unwrap();
}

fn dispatch_line(source: &str, kind: &str, cell: &str, arm: Option<&str>) -> String {
    let mut row = json!({"ts": "2026-11-07T00:00:00.000Z", "source": source, "kind": kind, "cell": cell});
    if let Some(arm) = arm {
        row["knowledge_arm"] = json!(arm);
    }
    format!("{row}\n")
}

fn write_dispatch(root: &Path, text: &str) {
    std::fs::write(root.join(".bee").join("logs").join("dispatch.jsonl"), text).unwrap();
}

fn report(fx: &Fixture, json_out: bool) -> (String, Value) {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("bee"));
    cmd.args(["knowledge", "report"]).current_dir(&fx.root);
    if json_out {
        cmd.arg("--json");
    }
    let out = cmd.output().unwrap();
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let parsed = if json_out { serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not json ({e}): {stdout}")) } else { Value::Null };
    (stdout, parsed)
}

fn outcomes(fx: &Fixture) -> Value {
    let (_, v) = report(fx, true);
    let found = v.get("outcomes").or_else(|| v.get("result").and_then(|r| r.get("outcomes")));
    found.unwrap_or_else(|| panic!("no outcomes key in: {v}")).clone()
}

fn seed_mixed(fx: &Fixture) {
    let root = &fx.root;
    write_cell(root, false, "k1", "capped", json!({"blocked_reason": "waiting on owner"}));
    write_cell(root, false, "k2", "dropped", json!({"reopened_for_rework": {"at": "2026-11-01T00:00:00Z"}}));
    write_cell(
        root,
        true,
        "k3",
        "capped",
        json!({"attempts": [{"verdict": "tests-red"}], "mistakes": [{"what": "wrong file"}]}),
    );
    write_cell(root, true, "k4", "capped", json!({"blocked_reason": "", "attempts": [], "mistakes": [], "reopened_at": null}));
    write_cell(root, false, "h1", "capped", json!({"reopened_at": "2026-11-02T00:00:00Z", "blocked_reason": null}));
    write_cell(root, true, "h2", "capped", json!({}));
    write_cell(root, false, "u1", "capped", json!({"blocked_reason": "no arm recorded"}));
    write_cell(root, false, "o1", "claimed", json!({"blocked_reason": "open cells never count"}));
    write_cell(root, false, "o2", "open", json!({}));

    let mut log = String::new();
    log += &dispatch_line("prepare", "cell", "k1", Some("knowledge"));
    log += &dispatch_line("prepare", "cell", "k2", Some("knowledge"));
    log += &dispatch_line("prepare", "cell", "k3", Some("knowledge"));
    log += &dispatch_line("prepare", "cell", "k4", Some("knowledge"));
    log += &dispatch_line("prepare", "cell", "k4", Some("holdout"));
    log += "not json at all\n";
    log += &dispatch_line("prepare", "cell", "h1", Some("holdout"));
    log += &dispatch_line("prepare", "cell", "h2", Some("holdout"));
    log += &dispatch_line("prepare", "cell", "u1", None);
    log += &dispatch_line("prepare", "cell", "o1", Some("knowledge"));
    log += &dispatch_line("herding-run", "cell", "u1", Some("holdout"));
    log += &dispatch_line("prepare", "gather", "u1", Some("holdout"));
    write_dispatch(root, &log);
}

fn assert_rate(v: &Value, expected: f64) {
    let got = v.as_f64().unwrap_or_else(|| panic!("rate is not a number: {v}"));
    assert!((got - expected).abs() < 1e-9, "rate {got} != {expected}");
}

#[test]
fn outcomes_count_each_arm_and_signal_exactly() {
    let fx = fixture();
    seed_mixed(&fx);
    let o = outcomes(&fx);

    let k = &o["arms"]["knowledge"];
    assert_eq!(k["cells"], 4, "{o}");
    assert_eq!(k["blocked"], 1, "{o}");
    assert_eq!(k["reopened"], 1, "{o}");
    assert_eq!(k["red_rounds"], 1, "{o}");
    assert_eq!(k["with_mistakes"], 1, "{o}");
    for key in ["blocked", "reopened", "red_rounds", "with_mistakes"] {
        assert_rate(&k["rates"][key], 0.25);
    }

    let h = &o["arms"]["holdout"];
    assert_eq!(h["cells"], 2, "{o}");
    assert_eq!(h["blocked"], 0, "{o}");
    assert_eq!(h["reopened"], 1, "{o}");
    assert_eq!(h["red_rounds"], 0, "{o}");
    assert_eq!(h["with_mistakes"], 0, "{o}");
    assert_rate(&h["rates"]["reopened"], 0.5);
    assert_rate(&h["rates"]["blocked"], 0.0);

    assert_eq!(o["unrecorded"], 1, "a cell with no recorded arm is unrecorded, in neither arm: {o}");
    assert_eq!(o["min_per_arm"], 30, "{o}");
    assert_eq!(o["comparable"], false, "{o}");
}

#[test]
fn rates_are_rounded_to_three_decimals() {
    let fx = fixture();
    write_cell(&fx.root, false, "a1", "capped", json!({"blocked_reason": "x"}));
    write_cell(&fx.root, false, "a2", "capped", json!({}));
    write_cell(&fx.root, false, "a3", "capped", json!({}));
    let mut log = String::new();
    for id in ["a1", "a2", "a3"] {
        log += &dispatch_line("prepare", "cell", id, Some("knowledge"));
    }
    write_dispatch(&fx.root, &log);
    let o = outcomes(&fx);
    assert_rate(&o["arms"]["knowledge"]["rates"]["blocked"], 0.333);
}

#[test]
fn an_empty_arm_reports_zero_rates() {
    let fx = fixture();
    let o = outcomes(&fx);
    assert_eq!(o["arms"]["holdout"]["cells"], 0, "{o}");
    assert_rate(&o["arms"]["holdout"]["rates"]["blocked"], 0.0);
    assert_eq!(o["unrecorded"], 0, "{o}");
    assert_eq!(o["comparable"], false, "{o}");
}

#[test]
fn no_dispatch_log_leaves_every_closed_cell_unrecorded() {
    let fx = fixture();
    write_cell(&fx.root, false, "c1", "capped", json!({}));
    write_cell(&fx.root, true, "c2", "dropped", json!({}));
    write_cell(&fx.root, false, "c3", "open", json!({}));
    let o = outcomes(&fx);
    assert_eq!(o["unrecorded"], 2, "{o}");
    assert_eq!(o["arms"]["knowledge"]["cells"], 0, "{o}");
    assert_eq!(o["arms"]["holdout"]["cells"], 0, "{o}");
}

#[test]
fn comparable_needs_thirty_cells_in_both_arms() {
    let fx = fixture();
    let mut log = String::new();
    for i in 0..30 {
        let id = format!("kn{i}");
        write_cell(&fx.root, false, &id, "capped", json!({}));
        log += &dispatch_line("prepare", "cell", &id, Some("knowledge"));
    }
    for i in 0..29 {
        let id = format!("ho{i}");
        write_cell(&fx.root, false, &id, "capped", json!({}));
        log += &dispatch_line("prepare", "cell", &id, Some("holdout"));
    }
    write_dispatch(&fx.root, &log);
    let o = outcomes(&fx);
    assert_eq!(o["arms"]["knowledge"]["cells"], 30, "{o}");
    assert_eq!(o["arms"]["holdout"]["cells"], 29, "{o}");
    assert_eq!(o["comparable"], false, "29 holdout cells is below the floor: {o}");

    write_cell(&fx.root, false, "ho29", "capped", json!({}));
    log += &dispatch_line("prepare", "cell", "ho29", Some("holdout"));
    write_dispatch(&fx.root, &log);
    let o = outcomes(&fx);
    assert_eq!(o["arms"]["holdout"]["cells"], 30, "{o}");
    assert_eq!(o["comparable"], true, "{o}");
}

#[test]
fn text_output_has_a_line_per_arm_and_a_too_few_line() {
    let fx = fixture();
    seed_mixed(&fx);
    let (text, _) = report(&fx, false);
    assert!(text.lines().any(|l| l.starts_with("outcomes knowledge:") && l.contains("4 cell(s)")), "{text}");
    assert!(text.lines().any(|l| l.starts_with("outcomes holdout:") && l.contains("2 cell(s)")), "{text}");
    assert!(
        text.lines().any(|l| l.contains("not comparable") && l.contains("26 more knowledge") && l.contains("28 more holdout")),
        "{text}"
    );
}

#[test]
fn text_output_has_no_too_few_line_when_comparable() {
    let fx = fixture();
    let mut log = String::new();
    for i in 0..30 {
        for arm in ["knowledge", "holdout"] {
            let id = format!("{arm}{i}");
            write_cell(&fx.root, false, &id, "capped", json!({}));
            log += &dispatch_line("prepare", "cell", &id, Some(arm));
        }
    }
    write_dispatch(&fx.root, &log);
    let (text, _) = report(&fx, false);
    assert!(text.lines().any(|l| l.starts_with("outcomes knowledge:")), "{text}");
    assert!(!text.contains("not comparable"), "{text}");
}

#[test]
fn existing_report_sections_are_unchanged() {
    let fx = fixture();
    seed_mixed(&fx);
    let (_, v) = report(&fx, true);
    let body = v.get("result").unwrap_or(&v);
    for key in ["measured", "unmeasured", "measured_count", "unmeasured_count", "evidence_ladder"] {
        assert!(body.get(key).is_some(), "{key} missing from: {v}");
    }
    assert_eq!(body["measured_count"], 0);
    assert_eq!(body["evidence_ladder"]["counts"]["present"], 0);
}

#[test]
fn feature_outcomes_report_the_feature_arm_beside_the_cell_arms() {
    let fx = fixture();
    write_cell(&fx.root, false, "c-1", "capped", json!({"blocked_reason": "x"}));
    let pack = json!({"ts": "2026-11-09T00:00:00.000Z", "source": "pack", "kind": "pack", "feature": "demo", "stage": "shaping", "knowledge_arm": "holdout"});
    write_dispatch(&fx.root, &format!("{}{pack}\n", dispatch_line("prepare", "cell", "c-1", Some("knowledge"))));
    let (_, v) = report(&fx, true);
    let section = v
        .get("feature_outcomes")
        .or_else(|| v.get("result").and_then(|r| r.get("feature_outcomes")))
        .unwrap_or_else(|| panic!("no feature_outcomes in: {v}"))
        .clone();
    assert_eq!(section["arms"]["holdout"]["features"], 1, "{section}");
    assert_eq!(section["arms"]["holdout"]["cells"], 1);
    assert_eq!(section["arms"]["holdout"]["blocked"], 1);
    assert_eq!(outcomes(&fx)["arms"]["knowledge"]["cells"], 1);
    let (text, _) = report(&fx, false);
    assert!(text.contains("feature outcomes holdout: 1 feature(s), 1 cell(s)"), "{text}");
}
