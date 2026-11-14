use crate::jsjson;
use crate::verbs::knowledge::{g_prelude_at, pre_json_scan, GPre};
use crate::verbs::reservations::{keys_known, parse_flags, Err2, FlagV};
use crate::verbs::state_group::{
    advisor_plan_path, anchor_target, claims_rows, heading_title, resolve_mutation_target, split_row, AnchorTarget,
    ROUTE_CLASS_VALUES,
};
use crate::verbs::workflow_store::{find_live_workflow, list_workflows, read_lane_display};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const SCHEMA: &str = "bee-plan-check/v1";
const EXIT_FINDINGS: u8 = 10;
const FULL_SECTIONS: [&str; 7] =
    ["Summary", "Requirements", "Load-bearing claims", "Approach", "Role assignments", "Risk map", "Cells"];
const SHORT_SECTIONS: [&str; 3] = ["Summary", "Load-bearing claims", "Cells"];
const PLAYBOOKS: &str = "skills/bee-planning/playbooks/";

pub(crate) struct Finding {
    pub(crate) check: &'static str,
    pub(crate) detail: String,
}

impl Finding {
    fn json(&self) -> Value {
        json!({"check": self.check, "detail": self.detail})
    }
}

pub(crate) struct PlanReport {
    pub(crate) mode: Option<String>,
    pub(crate) findings: Vec<Finding>,
}

fn frontmatter_mode(text: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            return None;
        }
        if let Some(value) = line.strip_prefix("mode:") {
            let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
            return (!value.is_empty()).then(|| value.to_string());
        }
    }
    None
}

fn section_findings(text: &str, mode: Option<&str>, out: &mut Vec<Finding>) {
    let required: &[&str] = match mode {
        Some("standard") | Some("high-risk") => &FULL_SECTIONS,
        _ => &SHORT_SECTIONS,
    };
    let headings: Vec<&str> = text.lines().filter(|l| l.starts_with("## ")).collect();
    let mut last: Option<(usize, &str)> = None;
    for name in required {
        let wanted = format!("## {name}");
        let Some(at) = headings.iter().position(|h| h.starts_with(&wanted)) else {
            out.push(Finding { check: "sections", detail: format!("missing section \"{wanted}\"") });
            continue;
        };
        if let Some((prev_at, prev)) = last {
            if at < prev_at {
                out.push(Finding {
                    check: "sections",
                    detail: format!("\"{wanted}\" comes before \"## {prev}\" but must come after it"),
                });
                continue;
            }
        }
        last = Some((at, name));
    }
}

fn playbook_findings(text: &str, class: Option<&str>, out: &mut Vec<Finding>) {
    let cited = |name: &str| text.contains(&format!("{PLAYBOOKS}{name}.md"));
    match class.filter(|c| ROUTE_CLASS_VALUES.contains(c)) {
        Some(class) => {
            if !cited(class) {
                out.push(Finding {
                    check: "playbook",
                    detail: format!("plan.md does not cite {PLAYBOOKS}{class}.md"),
                });
            }
        }
        None => {
            let any = text.match_indices(PLAYBOOKS).any(|(at, _)| {
                let rest = &text[at + PLAYBOOKS.len()..];
                let name: String =
                    rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
                !name.is_empty() && rest[name.len()..].starts_with(".md")
            });
            if !any {
                out.push(Finding { check: "playbook", detail: format!("plan.md cites no {PLAYBOOKS}<class>.md") });
            }
        }
    }
}

fn locked_decision_ids(context: &str) -> Vec<String> {
    let mut in_section = false;
    let mut ids = Vec::new();
    for line in context.lines() {
        if heading_title(line).is_some() {
            in_section = heading_title(line).is_some_and(|t| t.eq_ignore_ascii_case("locked decisions"));
            continue;
        }
        if !in_section || !line.trim().starts_with('|') {
            continue;
        }
        let first = split_row(line).into_iter().next().unwrap_or_default();
        let digits = first.strip_prefix('D').unwrap_or("");
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            ids.push(first);
        }
    }
    ids
}

fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

fn decision_findings(plan: &str, context: Option<&str>, out: &mut Vec<Finding>) {
    let Some(context) = context else {
        out.push(Finding { check: "decisions", detail: "CONTEXT.md is missing next to plan.md".to_string() });
        return;
    };
    for id in locked_decision_ids(context) {
        if !has_word(plan, &id) {
            out.push(Finding { check: "decisions", detail: format!("{id} of CONTEXT.md is absent from plan.md") });
        }
    }
}

fn strip_backticks(cell: &str) -> &str {
    let cell = cell.trim();
    if cell.len() >= 2 && cell.starts_with('`') && cell.ends_with('`') {
        &cell[1..cell.len() - 1]
    } else {
        cell
    }
}

fn evidence_findings(plan: &str, repo: &Path, out: &mut Vec<Finding>) {
    for row in claims_rows(plan) {
        if row.label != "read" {
            continue;
        }
        let Some(target) = anchor_target(&row.anchor) else { continue };
        let (path, span) = match target {
            AnchorTarget::Whole(p) => (p, None),
            AnchorTarget::Lines(p, a, b) => (p, Some((a, b))),
        };
        let bad = |why: String| Finding { check: "evidence", detail: format!("claims row {}: {why}", row.n) };
        let Ok(bytes) = std::fs::read(repo.join(path)) else {
            out.push(bad(format!("anchor path \"{path}\" cannot be read")));
            continue;
        };
        let file = String::from_utf8_lossy(&bytes);
        let haystack = match span {
            None => file.to_string(),
            Some((a, b)) => {
                let lines: Vec<&str> = file.lines().collect();
                if a == 0 || b < a || b > lines.len() {
                    out.push(bad(format!("anchor \"{}\" is outside {path}, which has {} lines", row.anchor, lines.len())));
                    continue;
                }
                lines[a - 1..b].join("\n")
            }
        };
        let evidence = strip_backticks(&row.evidence).replace("\\|", "|");
        if !haystack.contains(&evidence) {
            out.push(bad(format!("evidence is not a substring of the anchored bytes at {}", row.anchor)));
        }
    }
}

pub(crate) fn plan_report(plan_path: &Path, plan: &str, class: Option<&str>) -> PlanReport {
    let mode = frontmatter_mode(plan);
    let feature_dir = plan_path.parent();
    let context = feature_dir.and_then(|d| std::fs::read_to_string(d.join("CONTEXT.md")).ok());
    let repo = plan_path.ancestors().nth(4);
    let mut findings = Vec::new();
    section_findings(plan, mode.as_deref(), &mut findings);
    playbook_findings(plan, class, &mut findings);
    decision_findings(plan, context.as_deref(), &mut findings);
    if let Some(repo) = repo {
        evidence_findings(plan, repo, &mut findings);
    }
    PlanReport { mode, findings }
}

pub(crate) fn route_class(root: &Path, feature: &str) -> Option<String> {
    let class_of = |v: &Map<String, Value>| {
        v.get("route").and_then(|r| r.get("class")).and_then(Value::as_str).map(str::to_string)
    };
    let workflows = list_workflows(root).unwrap_or_default();
    find_live_workflow(&workflows, feature)
        .and_then(class_of)
        .or_else(|| read_lane_display(root, feature).ok().flatten().as_ref().and_then(class_of))
}

pub(crate) fn warning_lines(report: &PlanReport) -> String {
    report
        .findings
        .iter()
        .map(|f| format!("warning: plan check: {}: {}\n", f.check, f.detail))
        .collect()
}

fn refuse(json: bool, message: &str) -> ExitCode {
    if json {
        println!("{}", jsjson::stringify(&json!({ "error": message })));
    } else {
        eprintln!("{message}");
    }
    ExitCode::FAILURE
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "plan" {
        return None;
    }
    if args.get(1)?.to_str()? != "check" {
        return None;
    }
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    if !keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature_flag = match flags.get("feature") {
        None => None,
        Some(FlagV::S(s)) if !s.is_empty() => Some(s.clone()),
        Some(_) => return None,
    };
    let ctx = match g_prelude_at("plan check", json, pre_json, t0, Some(Path::new(".")))? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let feature = match feature_flag {
        Some(f) => f,
        None => {
            let resolved = match resolve_mutation_target(&ctx.root, None, "plan check", false) {
                Ok(target) => target.record().get("feature").and_then(Value::as_str).map(str::to_string),
                Err(Err2::Msg(m)) => return Some(refuse(json, &m)),
                Err(Err2::Ex) => None,
            };
            match resolved {
                Some(f) if !f.is_empty() => f,
                _ => return Some(refuse(json, "plan check: no feature resolves for this session. FIX: pass --feature <slug>.")),
            }
        }
    };
    let plan_path = advisor_plan_path(&ctx.root, &feature);
    let plan = match std::fs::read(&plan_path) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => {
            return Some(refuse(json, &format!("plan check: cannot read {} ({e}).", plan_path.display())));
        }
    };
    let class = route_class(&ctx.root, &feature);
    let report = plan_report(&plan_path, &plan, class.as_deref());
    let ok = report.findings.is_empty();
    if json {
        let result = json!({
            "schema": SCHEMA,
            "ok": ok,
            "feature": feature,
            "mode": report.mode,
            "findings": report.findings.iter().map(Finding::json).collect::<Vec<_>>(),
        });
        println!("{}", jsjson::stringify_pretty(&result));
    } else {
        let mut lines: Vec<String> = report.findings.iter().map(|f| format!("{}: {}", f.check, f.detail)).collect();
        lines.push(if ok {
            format!("plan check: {feature}: ok")
        } else {
            format!("plan check: {feature}: {} finding(s)", report.findings.len())
        });
        println!("{}", lines.join("\n"));
    }
    Some(ExitCode::from(if ok { 0 } else { EXIT_FINDINGS }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;

    const CONTEXT: &str = "# Context\n\n## Locked Decisions\n\n| ID | Decision | Store id |\n|---|---|---|\n| D1 | do it | abc |\n| D2 | do more | def |\n\n## Read set\n";

    fn plan_body(extra: &str) -> String {
        format!(
            "---\nmode: standard\n---\n\n# Plan\n\n## Summary\n\nD1 and D2. `skills/bee-planning/playbooks/feature.md`\n\n## Requirements (from CONTEXT.md)\n\nr\n\n## Load-bearing claims\n\n| # | Claim | Label | Anchor | Verbatim evidence |\n|---|---|---|---|---|\n{extra}\n## Approach\n\na\n\n## Role assignments\n\nr\n\n## Risk map\n\nr\n\n## Cells — current slice\n\nc\n"
        )
    }

    fn repo(plan: &str, context: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee/onboarding.json"), r#"{"schema_version":"1.0","bee_version":"0.1.0"}"#).unwrap();
        let dir = tmp.path().join("docs/history/demo");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("plan.md"), plan).unwrap();
        if let Some(c) = context {
            std::fs::write(dir.join("CONTEXT.md"), c).unwrap();
        }
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/a.rs"), "fn one() {}\nfn two() {}\nfn three() {}\n").unwrap();
        tmp
    }

    fn check(root: &Path) -> (Option<i32>, Value, String) {
        let out = Command::cargo_bin("bee")
            .unwrap()
            .args(["plan", "check", "--feature", "demo", "--json"])
            .env_remove("BEE_HERDING_WORKER")
            .current_dir(root)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let parsed = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
        (out.status.code(), parsed, stdout + &String::from_utf8_lossy(&out.stderr))
    }

    fn details(v: &Value, check: &str) -> Vec<String> {
        v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["check"] == check)
            .map(|f| f["detail"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn plan_group_clean_plan_exits_zero_and_writes_nothing() {
        let row = "| 1 | c | read | `src/a.rs:2` | `fn two` |\n";
        let tmp = repo(&plan_body(row), Some(CONTEXT));
        let before = std::fs::read_dir(tmp.path().join(".bee")).unwrap().count();
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(0), "{text}");
        assert_eq!(v["schema"], "bee-plan-check/v1");
        assert_eq!(v["ok"], true);
        assert_eq!(v["mode"], "standard");
        assert_eq!(v["feature"], "demo");
        assert_eq!(std::fs::read_dir(tmp.path().join(".bee")).unwrap().count(), before);
    }

    #[test]
    fn plan_group_missing_risk_map_is_a_sections_finding() {
        let body = plan_body("").replace("## Risk map\n\nr\n\n", "");
        let tmp = repo(&body, Some(CONTEXT));
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(v["ok"], false);
        assert!(details(&v, "sections").iter().any(|d| d.contains("## Risk map")), "{text}");
    }

    #[test]
    fn plan_group_out_of_order_sections_are_a_sections_finding() {
        let body = plan_body("").replace("## Approach", "## Zzz").replace("## Risk map", "## Approach").replace("## Zzz", "## Risk map");
        let tmp = repo(&body, Some(CONTEXT));
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(10), "{text}");
        assert!(details(&v, "sections").iter().any(|d| d.contains("comes before")), "{text}");
    }

    #[test]
    fn plan_group_missing_playbook_citation_is_a_playbook_finding() {
        let body = plan_body("").replace("`skills/bee-planning/playbooks/feature.md`", "");
        let tmp = repo(&body, Some(CONTEXT));
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(details(&v, "playbook").len(), 1, "{text}");
    }

    #[test]
    fn plan_group_missing_decision_number_is_named() {
        let body = plan_body("").replace("D1 and D2.", "D1 only.");
        let tmp = repo(&body, Some(CONTEXT));
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(10), "{text}");
        let found = details(&v, "decisions");
        assert_eq!(found.len(), 1, "{text}");
        assert!(found[0].contains("D2"), "{text}");
    }

    #[test]
    fn plan_group_evidence_mismatch_names_the_row() {
        let row = "| 1 | c | read | `src/a.rs:2` | fn one |\n";
        let tmp = repo(&plan_body(row), Some(CONTEXT));
        let (code, v, text) = check(tmp.path());
        assert_eq!(code, Some(10), "{text}");
        let found = details(&v, "evidence");
        assert_eq!(found.len(), 1, "{text}");
        assert!(found[0].contains("row 1"), "{text}");
    }

    #[test]
    fn plan_group_evidence_matches_across_a_line_range() {
        let row = "| 1 | c | read | `src/a.rs:1-2` | `fn one() {}\nfn two` |\n| 2 | c | read | `src/a.rs:1-2` | fn one() {} |\n";
        let tmp = repo(&plan_body(row), Some(CONTEXT));
        let (_, v, text) = check(tmp.path());
        let found = details(&v, "evidence");
        assert_eq!(found.len(), 1, "{text}");
        assert!(found[0].contains("row 1"), "{text}");
    }

    #[test]
    fn plan_group_preview_prints_the_warning_and_still_succeeds() {
        let cell = r#"[{"id":"c1","feature":"demo","title":"t","lane":"standard","role":"code","action":"act","files":["src/a.rs"],"read_first":[],"must_haves":{"truths":["x"]},"verify":"true"}]"#;
        let body = plan_body("").replace("D1 and D2.", "D1 only.").replace("## Role assignments\n\nr\n\n", "").replace("\nc\n", &format!("\n```json\n{cell}\n```\n"));
        let tmp = repo(&body, Some(CONTEXT));
        let state = r#"{"schema_version":"1.0","feature":"demo","phase":"planning","approved_gates":{}}"#;
        std::fs::write(tmp.path().join(".bee/state.json"), state).unwrap();
        let out = Command::cargo_bin("bee")
            .unwrap()
            .args(["gate", "--preview"])
            .env_remove("BEE_HERDING_WORKER")
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
        assert!(text.contains("warning: plan check: decisions: D2"), "{text}");
        assert!(!text.contains("gate preview: plan.md"), "{text}");
    }
}
