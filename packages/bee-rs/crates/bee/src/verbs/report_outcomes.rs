use crate::jsjson;
use crate::roots::{resolve_store_root_worktree, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const CMD: &str = "report outcomes";

struct Opts {
    feature: Option<String>,
    json: bool,
}

fn parse(tokens: &[OsString]) -> Option<Opts> {
    let toks: Vec<&str> = tokens.iter().map(|t| t.to_str()).collect::<Option<_>>()?;
    let mut feature = None;
    let mut json = false;
    let mut i = 0;
    while i < toks.len() {
        let tok = toks[i];
        if tok == "--json" {
            json = true;
            i += 1;
            continue;
        }
        if tok == "--feature" {
            i += 1;
            feature = Some(toks.get(i)?.to_string());
            i += 1;
            continue;
        }
        if let Some(val) = tok.strip_prefix("--feature=") {
            feature = Some(val.to_string());
            i += 1;
            continue;
        }
        return None;
    }
    Some(Opts { feature, json })
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "report" || args.get(1)?.to_str()? != "outcomes" {
        return None;
    }
    let opts = parse(&args[2..])?;
    Some(run(&opts, t0))
}

fn run(opts: &Opts, t0: Instant) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => return fail(Path::new("."), opts.json, "bad_cwd", format!("bee {CMD}: {e}"), t0),
    };
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return emit_unsupported_root(&cwd, CMD, opts.json, t0, &why)
        }
        RootsWt::None => return emit_no_root_error(&cwd, CMD, opts.json, t0),
    };
    let report = collect_outcomes(&ledger_root(&roots), opts.feature.as_deref());
    if opts.json {
        println!("{}", jsjson::stringify_pretty(&render_json(&report)));
    } else {
        println!("{}", render_text(&report));
    }
    record_timing(&roots.root, CMD, t0, true);
    ExitCode::SUCCESS
}

fn ledger_root(roots: &StoreRoots) -> PathBuf {
    roots.main_root()
}

fn fail(root: &Path, use_json: bool, kind: &'static str, msg: String, t0: Instant) -> ExitCode {
    if use_json {
        println!("{}", jsjson::stringify(&json!({"error": msg, "kind": kind})));
    } else {
        eprintln!("{msg}");
    }
    record_timing(root, CMD, t0, false);
    ExitCode::FAILURE
}

#[derive(Default, Debug)]
struct ReactionMetrics {
    confirm: usize,
    partial: usize,
    challenge: usize,
    blocked: usize,
    null: usize,
    absent: usize,
}

impl ReactionMetrics {
    fn cells_with_record(&self) -> usize {
        self.confirm + self.partial + self.challenge + self.blocked + self.null
    }

    fn changed(&self) -> usize {
        self.partial + self.challenge + self.blocked
    }
}

#[derive(Default, Debug)]
struct DissentMetrics {
    cells_with_record: usize,
    accept: usize,
    reject: usize,
    escalate: usize,
    unanswered: usize,
}

impl DissentMetrics {
    fn total_entries(&self) -> usize {
        self.accept + self.reject + self.escalate + self.unanswered
    }

    fn changed(&self) -> usize {
        self.accept + self.escalate
    }
}

#[derive(Default, Debug)]
struct JudgeMetrics {
    cells_with_record: usize,
    pass: usize,
    needs_revision: usize,
    other: usize,
}

impl JudgeMetrics {
    fn total_entries(&self) -> usize {
        self.pass + self.needs_revision + self.other
    }

    fn changed(&self) -> usize {
        self.needs_revision
    }
}

#[derive(Default, Debug)]
struct LeaderCheckMetrics {
    cells_with_record: usize,
    verdicts: BTreeMap<String, usize>,
}

impl LeaderCheckMetrics {
    fn total_entries(&self) -> usize {
        self.verdicts.values().sum()
    }

    fn changed(&self) -> usize {
        self.verdicts
            .iter()
            .filter(|(k, _)| k.as_str() != "ok")
            .map(|(_, v)| *v)
            .sum()
    }
}

#[derive(Default, Debug)]
struct ReworkMetrics {
    cells_with_record: usize,
}

impl ReworkMetrics {
    fn changed(&self) -> usize {
        self.cells_with_record
    }
}

#[derive(Default, Debug)]
struct RoleRerouteMetrics {
    cells_with_record: usize,
    entries: usize,
}

impl RoleRerouteMetrics {
    fn changed(&self) -> usize {
        self.entries
    }
}

#[derive(Default, Debug)]
struct OutcomesReport {
    feature: Option<String>,
    cells_scanned: usize,
    unreadable: Vec<String>,
    reaction: ReactionMetrics,
    dissent: DissentMetrics,
    judge: JudgeMetrics,
    leader_check: LeaderCheckMetrics,
    rework: ReworkMetrics,
    role_reroute: RoleRerouteMetrics,
}

fn find_cell_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cells_dir = root.join(".bee").join("cells");
    if let Ok(entries) = std::fs::read_dir(&cells_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".json") && !name.starts_with('.') {
                        files.push(path);
                    }
                }
            }
        }
    }
    let archive_dir = cells_dir.join("archive");
    if let Ok(feat_entries) = std::fs::read_dir(&archive_dir) {
        for feat_entry in feat_entries.flatten() {
            let feat_path = feat_entry.path();
            if feat_path.is_dir() {
                if let Ok(cell_entries) = std::fs::read_dir(&feat_path) {
                    for cell_entry in cell_entries.flatten() {
                        let path = cell_entry.path();
                        if path.is_file() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.ends_with(".json") && !name.starts_with('.') {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    files.sort();
    files
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map_or(false, |f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) => true,
        Value::Object(_) => true,
    }
}

fn collect_outcomes(root: &Path, feature_filter: Option<&str>) -> OutcomesReport {
    let mut report = OutcomesReport {
        feature: feature_filter.map(str::to_string),
        cells_scanned: 0,
        unreadable: Vec::new(),
        reaction: ReactionMetrics::default(),
        dissent: DissentMetrics::default(),
        judge: JudgeMetrics::default(),
        leader_check: LeaderCheckMetrics::default(),
        rework: ReworkMetrics::default(),
        role_reroute: RoleRerouteMetrics::default(),
    };
    let files = find_cell_files(root);
    for file in files {
        let raw = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => {
                report.unreadable.push(file.display().to_string());
                continue;
            }
        };
        let val: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                report.unreadable.push(file.display().to_string());
                continue;
            }
        };
        let Some(obj) = val.as_object() else {
            report.unreadable.push(file.display().to_string());
            continue;
        };
        if let Some(feat) = feature_filter {
            let cell_feat = obj.get("feature").and_then(Value::as_str);
            if cell_feat != Some(feat) {
                continue;
            }
        }
        report.cells_scanned += 1;

        let trace_obj = obj.get("trace").and_then(Value::as_object);

        match trace_obj.and_then(|t| t.get("reaction")) {
            None => report.reaction.absent += 1,
            Some(Value::Null) => report.reaction.null += 1,
            Some(Value::String(s)) => {
                match s.trim().to_lowercase().as_str() {
                    "confirm" => report.reaction.confirm += 1,
                    "partial" => report.reaction.partial += 1,
                    "challenge" => report.reaction.challenge += 1,
                    "blocked" => report.reaction.blocked += 1,
                    _ => report.reaction.null += 1,
                }
            }
            Some(_) => report.reaction.null += 1,
        }

        if let Some(dissent_arr) = trace_obj.and_then(|t| t.get("dissent")).and_then(Value::as_array) {
            if !dissent_arr.is_empty() {
                report.dissent.cells_with_record += 1;
                for entry in dissent_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str);
                    match verdict.map(|v| v.trim().to_lowercase()).as_deref() {
                        Some("accept") => report.dissent.accept += 1,
                        Some("reject") => report.dissent.reject += 1,
                        Some("escalate") => report.dissent.escalate += 1,
                        _ => report.dissent.unanswered += 1,
                    }
                }
            }
        }

        if let Some(judge_arr) = trace_obj.and_then(|t| t.get("semantic_judge")).and_then(Value::as_array) {
            if !judge_arr.is_empty() {
                report.judge.cells_with_record += 1;
                for entry in judge_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str);
                    match verdict {
                        Some("PASS") => report.judge.pass += 1,
                        Some("NEEDS_REVISION") => report.judge.needs_revision += 1,
                        _ => report.judge.other += 1,
                    }
                }
            }
        }

        if let Some(leader_arr) = trace_obj.and_then(|t| t.get("leader_check")).and_then(Value::as_array) {
            if !leader_arr.is_empty() {
                report.leader_check.cells_with_record += 1;
                for entry in leader_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str).unwrap_or("unknown");
                    *report.leader_check.verdicts.entry(verdict.to_string()).or_insert(0) += 1;
                }
            }
        }

        let reworked = ["reopened_for_rework", "reopened_at"]
            .iter()
            .any(|k| trace_obj.and_then(|t| t.get(*k)).is_some_and(is_truthy));
        if reworked {
            report.rework.cells_with_record += 1;
        }

        if let Some(reroute_arr) = obj.get("role_reroutes").and_then(Value::as_array) {
            if !reroute_arr.is_empty() {
                report.role_reroute.cells_with_record += 1;
                report.role_reroute.entries += reroute_arr.len();
            }
        }
    }
    report
}

fn render_text(r: &OutcomesReport) -> String {
    let mut lines = Vec::new();

    if r.reaction.cells_with_record() == 0 {
        lines.push("worker reaction: not recorded".to_string());
    } else {
        lines.push(format!(
            "worker reaction: {} cells ({} changed) — confirm: {}, partial: {}, challenge: {}, blocked: {}, null: {}, absent: {}",
            r.reaction.cells_with_record(),
            r.reaction.changed(),
            r.reaction.confirm,
            r.reaction.partial,
            r.reaction.challenge,
            r.reaction.blocked,
            r.reaction.null,
            r.reaction.absent
        ));
    }

    if r.dissent.cells_with_record == 0 {
        lines.push("dissent: not recorded".to_string());
    } else {
        lines.push(format!(
            "dissent: {} cells, {} records ({} changed) — accept: {}, reject: {}, escalate: {}, unanswered: {}",
            r.dissent.cells_with_record,
            r.dissent.total_entries(),
            r.dissent.changed(),
            r.dissent.accept,
            r.dissent.reject,
            r.dissent.escalate,
            r.dissent.unanswered
        ));
    }

    if r.judge.cells_with_record == 0 {
        lines.push("semantic judge: not recorded".to_string());
    } else {
        lines.push(format!(
            "semantic judge: {} cells, {} records ({} changed) — PASS: {}, NEEDS_REVISION: {}, other: {}",
            r.judge.cells_with_record,
            r.judge.total_entries(),
            r.judge.changed(),
            r.judge.pass,
            r.judge.needs_revision,
            r.judge.other
        ));
    }

    if r.leader_check.cells_with_record == 0 {
        lines.push("leader check: not recorded".to_string());
    } else {
        let v_str = r.leader_check.verdicts.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ");
        lines.push(format!(
            "leader check: {} cells, {} records ({} changed) — {}",
            r.leader_check.cells_with_record,
            r.leader_check.total_entries(),
            r.leader_check.changed(),
            v_str
        ));
    }

    if r.rework.cells_with_record == 0 {
        lines.push("rework: not recorded".to_string());
    } else {
        lines.push(format!(
            "rework: {} cells ({} changed)",
            r.rework.cells_with_record,
            r.rework.changed()
        ));
    }

    if r.role_reroute.cells_with_record == 0 {
        lines.push("role reroute: not recorded".to_string());
    } else {
        lines.push(format!(
            "role reroute: {} cells, {} records ({} changed)",
            r.role_reroute.cells_with_record,
            r.role_reroute.entries,
            r.role_reroute.changed()
        ));
    }

    if !r.unreadable.is_empty() {
        lines.push(format!("unreadable: {} ({})", r.unreadable.len(), r.unreadable.join(", ")));
    }

    lines.join("\n")
}

fn render_json(r: &OutcomesReport) -> Value {
    let mut m = Map::new();
    m.insert(
        "feature".into(),
        match &r.feature {
            Some(f) => Value::String(f.clone()),
            None => Value::Null,
        },
    );
    m.insert("cells_scanned".into(), json!(r.cells_scanned));
    m.insert("unreadable".into(), json!(r.unreadable));

    m.insert(
        "reaction".into(),
        json!({
            "recorded": r.reaction.cells_with_record() > 0,
            "cells_with_record": r.reaction.cells_with_record(),
            "records": r.reaction.cells_with_record(),
            "changed": r.reaction.changed(),
            "confirm": r.reaction.confirm,
            "partial": r.reaction.partial,
            "challenge": r.reaction.challenge,
            "blocked": r.reaction.blocked,
            "null": r.reaction.null,
            "absent": r.reaction.absent,
        }),
    );

    m.insert(
        "dissent".into(),
        json!({
            "recorded": r.dissent.cells_with_record > 0,
            "cells_with_record": r.dissent.cells_with_record,
            "records": r.dissent.total_entries(),
            "changed": r.dissent.changed(),
            "accept": r.dissent.accept,
            "reject": r.dissent.reject,
            "escalate": r.dissent.escalate,
            "unanswered": r.dissent.unanswered,
            "verdicts": {
                "accept": r.dissent.accept,
                "reject": r.dissent.reject,
                "escalate": r.dissent.escalate,
                "unanswered": r.dissent.unanswered,
            }
        }),
    );

    m.insert(
        "semantic_judge".into(),
        json!({
            "recorded": r.judge.cells_with_record > 0,
            "cells_with_record": r.judge.cells_with_record,
            "records": r.judge.total_entries(),
            "changed": r.judge.changed(),
            "PASS": r.judge.pass,
            "NEEDS_REVISION": r.judge.needs_revision,
            "other": r.judge.other,
            "verdicts": {
                "PASS": r.judge.pass,
                "NEEDS_REVISION": r.judge.needs_revision,
                "other": r.judge.other,
            }
        }),
    );

    m.insert(
        "leader_check".into(),
        json!({
            "recorded": r.leader_check.cells_with_record > 0,
            "cells_with_record": r.leader_check.cells_with_record,
            "records": r.leader_check.total_entries(),
            "changed": r.leader_check.changed(),
            "verdicts": r.leader_check.verdicts,
        }),
    );

    m.insert(
        "rework".into(),
        json!({
            "recorded": r.rework.cells_with_record > 0,
            "cells_with_record": r.rework.cells_with_record,
            "records": r.rework.cells_with_record,
            "changed": r.rework.changed(),
        }),
    );

    m.insert(
        "role_reroute".into(),
        json!({
            "recorded": r.role_reroute.cells_with_record > 0,
            "cells_with_record": r.role_reroute.cells_with_record,
            "records": r.role_reroute.entries,
            "changed": r.role_reroute.changed(),
        }),
    );

    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn setup_fixture_store() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let cells = root.join(".bee").join("cells");
        let arch = cells.join("archive").join("slp-article-gaps");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(root.join(".bee/onboarding.json"), r#"{"completed":true}"#).unwrap();
        std::fs::write(root.join(".bee/state.json"), r#"{"phase":"executing"}"#).unwrap();

        std::fs::write(
            cells.join("c-01.json"),
            r#"{"id":"c-01","feature":"slp-article-gaps","trace":{"reaction":"confirm"}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-02.json"),
            r#"{"id":"c-02","feature":"slp-article-gaps","trace":{"reaction":"partial"}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-03.json"),
            r#"{"id":"c-03","feature":"slp-article-gaps","trace":{"reaction":"challenge"}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-04.json"),
            r#"{"id":"c-04","feature":"slp-article-gaps","trace":{"reaction":"blocked"}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-05.json"),
            r#"{"id":"c-05","feature":"slp-article-gaps","trace":{"reaction":null}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-06.json"),
            r#"{"id":"c-06","feature":"slp-article-gaps","trace":{}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-07.json"),
            r#"{"id":"c-07","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"accept"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-08.json"),
            r#"{"id":"c-08","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"reject"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-09.json"),
            r#"{"id":"c-09","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"escalate"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-10.json"),
            r#"{"id":"c-10","feature":"slp-article-gaps","trace":{"dissent":[{"claim":"issue"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-11.json"),
            r#"{"id":"c-11","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"PASS"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-12.json"),
            r#"{"id":"c-12","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"NEEDS_REVISION"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-13.json"),
            r#"{"id":"c-13","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"OTHER"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-14.json"),
            r#"{"id":"c-14","feature":"slp-article-gaps","trace":{"leader_check":[{"verdict":"ok"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-15.json"),
            r#"{"id":"c-15","feature":"slp-article-gaps","trace":{"leader_check":[{"verdict":"gap"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-16.json"),
            r#"{"id":"c-16","feature":"slp-article-gaps","trace":{"reopened_for_rework":{"reason":"fix"}}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-16b.json"),
            r#"{"id":"c-16b","feature":"slp-article-gaps","trace":{"reopened_at":"2026-11-04T00:00:00Z","reopened_reason":"goal-check miss"}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-17.json"),
            r#"{"id":"c-17","feature":"slp-article-gaps","role_reroutes":[{"from":"code","to":"test"}]}"#,
        ).unwrap();

        std::fs::write(
            arch.join("c-18.json"),
            r#"{"id":"c-18","feature":"slp-article-gaps","trace":{"reaction":"confirm"}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-19-other.json"),
            r#"{"id":"c-19","feature":"other-feat","trace":{"reaction":"blocked"}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("corrupt.json"),
            "not valid json {{{",
        ).unwrap();

        (tmp, root)
    }

    #[test]
    fn test_report_outcomes_fixture_counts_with_feature_filter() {
        let (_tmp, root) = setup_fixture_store();
        let report = collect_outcomes(&root, Some("slp-article-gaps"));

        assert_eq!(report.cells_scanned, 19);
        assert_eq!(report.unreadable.len(), 1);
        assert!(report.unreadable[0].ends_with("corrupt.json"));

        assert_eq!(report.reaction.confirm, 2);
        assert_eq!(report.reaction.partial, 1);
        assert_eq!(report.reaction.challenge, 1);
        assert_eq!(report.reaction.blocked, 1);
        assert_eq!(report.reaction.null, 1);
        assert_eq!(report.reaction.absent, 13);
        assert_eq!(report.reaction.cells_with_record(), 6);
        assert_eq!(report.reaction.changed(), 3);

        assert_eq!(report.dissent.cells_with_record, 4);
        assert_eq!(report.dissent.accept, 1);
        assert_eq!(report.dissent.reject, 1);
        assert_eq!(report.dissent.escalate, 1);
        assert_eq!(report.dissent.unanswered, 1);
        assert_eq!(report.dissent.changed(), 2);

        assert_eq!(report.judge.cells_with_record, 3);
        assert_eq!(report.judge.pass, 1);
        assert_eq!(report.judge.needs_revision, 1);
        assert_eq!(report.judge.other, 1);
        assert_eq!(report.judge.changed(), 1);

        assert_eq!(report.leader_check.cells_with_record, 2);
        assert_eq!(report.leader_check.verdicts.get("ok"), Some(&1));
        assert_eq!(report.leader_check.verdicts.get("gap"), Some(&1));
        assert_eq!(report.leader_check.changed(), 1);

        assert_eq!(report.rework.cells_with_record, 2);
        assert_eq!(report.rework.changed(), 2);

        assert_eq!(report.role_reroute.cells_with_record, 1);
        assert_eq!(report.role_reroute.entries, 1);
        assert_eq!(report.role_reroute.changed(), 1);

        let text = render_text(&report);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 7);
        assert!(lines[0].starts_with("worker reaction: 6 cells (3 changed)"));
        assert!(lines[0].contains("confirm: 2, partial: 1, challenge: 1, blocked: 1, null: 1, absent: 13"));
        assert!(lines[1].starts_with("dissent: 4 cells, 4 records (2 changed)"));
        assert!(lines[1].contains("accept: 1, reject: 1, escalate: 1, unanswered: 1"));
        assert!(lines[2].starts_with("semantic judge: 3 cells, 3 records (1 changed)"));
        assert!(lines[2].contains("PASS: 1, NEEDS_REVISION: 1, other: 1"));
        assert!(lines[3].starts_with("leader check: 2 cells, 2 records (1 changed)"));
        assert!(lines[3].contains("gap: 1, ok: 1"));
        assert_eq!(lines[4], "rework: 2 cells (2 changed)");
        assert_eq!(lines[5], "role reroute: 1 cells, 1 records (1 changed)");
        assert!(lines[6].starts_with("unreadable: 1"));

        let json = render_json(&report);
        assert_eq!(json["cells_scanned"], 19);
        assert_eq!(json["reaction"]["confirm"], 2);
        assert_eq!(json["reaction"]["changed"], 3);
        assert_eq!(json["dissent"]["accept"], 1);
        assert_eq!(json["dissent"]["changed"], 2);
        assert_eq!(json["semantic_judge"]["NEEDS_REVISION"], 1);
        assert_eq!(json["semantic_judge"]["changed"], 1);
        assert_eq!(json["leader_check"]["verdicts"]["gap"], 1);
        assert_eq!(json["leader_check"]["changed"], 1);
        assert_eq!(json["rework"]["changed"], 2);
        assert_eq!(json["role_reroute"]["changed"], 1);
    }

    #[test]
    fn test_report_outcomes_unfiltered_includes_other_feature() {
        let (_tmp, root) = setup_fixture_store();
        let report = collect_outcomes(&root, None);

        assert_eq!(report.cells_scanned, 20);
        assert_eq!(report.reaction.blocked, 2);
        assert_eq!(report.reaction.changed(), 4);
    }

    #[test]
    fn test_empty_store_prints_not_recorded_never_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".bee/cells")).unwrap();

        let report = collect_outcomes(&root, None);
        assert_eq!(report.cells_scanned, 0);

        let text = render_text(&report);
        for line in text.lines() {
            assert!(line.ends_with("not recorded"), "line must end with 'not recorded': {line}");
            assert!(!line.contains(": 0"), "line must not contain ': 0': {line}");
        }

        assert!(text.contains("worker reaction: not recorded"));
        assert!(text.contains("dissent: not recorded"));
        assert!(text.contains("semantic judge: not recorded"));
        assert!(text.contains("leader check: not recorded"));
        assert!(text.contains("rework: not recorded"));
        assert!(text.contains("role reroute: not recorded"));
    }

    #[test]
    fn test_try_native_flag_parsing() {
        let opts = parse(&[OsString::from("--json")]).unwrap();
        assert!(opts.json);
        assert_eq!(opts.feature, None);

        let opts = parse(&[OsString::from("--feature"), OsString::from("xyz")]).unwrap();
        assert!(!opts.json);
        assert_eq!(opts.feature.as_deref(), Some("xyz"));

        let opts = parse(&[OsString::from("--feature=abc"), OsString::from("--json")]).unwrap();
        assert!(opts.json);
        assert_eq!(opts.feature.as_deref(), Some("abc"));

        assert!(parse(&[OsString::from("--unknown")]).is_none());
    }

    fn git(cwd: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    #[test]
    fn test_report_outcomes_linked_worktree_reads_main_store() {
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main");
        std::fs::create_dir_all(main.join(".bee/cells")).unwrap();
        std::fs::write(main.join(".bee/onboarding.json"), "{}").unwrap();
        git(&main, &["init", "-q", "."]);
        git(
            &main,
            &[
                "-c",
                "user.email=t@t.t",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "i",
            ],
        );
        let wt = dir.path().join("wt");
        git(
            &main,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "wt/x"],
        );

        let main_cell = json!({
            "id": "cell-main",
            "feature": "feat-a",
            "trace": {
                "reaction": "confirm"
            }
        });
        std::fs::write(
            main.join(".bee/cells/cell-main.json"),
            serde_json::to_string(&main_cell).unwrap(),
        )
        .unwrap();

        std::fs::create_dir_all(wt.join(".bee/cells")).unwrap();
        let wt_cell = json!({
            "id": "cell-wt-stale",
            "feature": "feat-a",
            "trace": {
                "reaction": "blocked"
            }
        });
        std::fs::write(
            wt.join(".bee/cells/cell-wt-stale.json"),
            serde_json::to_string(&wt_cell).unwrap(),
        )
        .unwrap();

        std::fs::create_dir_all(main.join(".bee/runtime")).unwrap();
        std::fs::write(
            main.join(".bee/runtime/worktree-grants.json"),
            r#"{"wt": true}"#,
        )
        .unwrap();

        let RootsWt::Go(roots) = resolve_store_root_worktree(&wt) else {
            panic!("worktree did not resolve");
        };
        assert!(roots.linked.as_ref().is_some_and(|l| l.granted()));

        let report = collect_outcomes(&ledger_root(&roots), None);
        assert_eq!(report.cells_scanned, 1);
        assert_eq!(report.reaction.confirm, 1);
        assert_eq!(report.reaction.blocked, 0);
    }
}
