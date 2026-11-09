#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, resolve_store_root_any as resolve_store_root, Resolution, Roots};
use crate::state::read_config_raw;
use crate::verbs::drivers::{
    append_pack_record, docs_root_for_feature, feature_arm, learned_context_lines, pack_line_path,
    split_pack_lines, Delegate,
};

type D<T> = Result<T, Delegate>;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const PACK_STAGES: [&str; 2] = ["shaping", "planning"];

pub(crate) const PACK_DEFAULT_LANE: &str = "standard";

pub(crate) const HOLDOUT_NOTE: &str =
    "holdout: this feature loads no pack (measurement arm, knowledge_holdout_percent)";

pub(crate) fn control_root_of(root: &Path) -> PathBuf {
    match resolve_roots_core(root) {
        Resolution::LinkedValid { main_root, .. } => main_root,
        Resolution::Ordinary { work_root, .. } => work_root,
        _ => root.to_path_buf(),
    }
}

struct PackLine {
    path: String,
    title: Option<String>,
}

fn pack_lines_of(lines: &[String]) -> Vec<PackLine> {
    lines
        .iter()
        .map(|line| match line.split_once(" — ") {
            Some((_, title)) => PackLine {
                path: pack_line_path(line),
                title: (!title.is_empty()).then(|| title.to_string()),
            },
            None => PackLine { path: pack_line_path(line), title: None },
        })
        .collect()
}

fn code_lines_of(lines: &[String]) -> Vec<(String, String)> {
    lines
        .iter()
        .map(|line| match line.split_once(" — anchored by ") {
            Some((path, concept)) => (pack_line_path(path), js_trim(concept).to_string()),
            None => (pack_line_path(line), String::new()),
        })
        .collect()
}

fn grades_for(dir: &Path, listed: &[PackLine]) -> BTreeMap<String, &'static str> {
    let mut out = BTreeMap::new();
    let Some(concepts) = collect_concepts(dir) else { return out };
    let shown: Vec<&Concept> = concepts
        .iter()
        .filter(|c| listed.iter().any(|l| l.path == format!("docs/knowledge/{}", c.path)))
        .filter(|c| !string_anchors(&bee_of(&c.data)).is_empty())
        .collect();
    let Some(checkout) = checkout_of(&dir) else { return out };
    for (rel, grade) in grade_area_concepts(checkout, &shown) {
        out.insert(format!("docs/knowledge/{rel}"), grade.grade);
    }
    out
}

fn string_anchors(bee: &Map<String, Value>) -> Vec<String> {
    match bee.get("anchors") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(|s| js_trim(s).to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) struct PackAnswer {
    pub(crate) value: Value,
    pub(crate) lines: Vec<String>,
    pub(crate) control_root: PathBuf,
}

pub(crate) fn build_pack(
    root: &Path,
    feature: &str,
    stage: &str,
    lane: &str,
) -> D<PackAnswer> {
    let control_root = control_root_of(root);
    let checkout = docs_root_for_feature(root, feature);
    let bundle = bundle_root(root, Some(feature));
    let arm = feature_arm(&control_root, feature);

    let (knowledge_lines, code_lines) = if arm == "holdout" {
        (Vec::new(), Vec::new())
    } else {
        let cell = json!({ "id": feature, "feature": feature, "lane": lane });
        split_pack_lines(&learned_context_lines(&checkout, &cell)?)
    };

    let listed = pack_lines_of(&knowledge_lines);
    let graded = grades_for(&bundle, &listed);
    let grade_of = |path: &str| -> Option<&'static str> { graded.get(path).copied() };
    let codes = code_lines_of(&code_lines);

    let knowledge: Vec<Value> = listed
        .iter()
        .map(|item| {
            json!({
                "path": item.path,
                "title": item.title,
                "grade": grade_of(&item.path),
            })
        })
        .collect();
    let code: Vec<Value> = codes
        .iter()
        .map(|(path, by)| json!({ "path": path, "anchored_by": by }))
        .collect();

    let mut lines: Vec<String> = Vec::new();
    if arm == "holdout" {
        lines.push(HOLDOUT_NOTE.to_string());
    } else {
        for item in &listed {
            lines.push(match grade_of(&item.path) {
                Some(grade) => format!("{} [{grade}]", render_knowledge(item)),
                None => render_knowledge(item),
            });
        }
        lines.extend(code_lines.iter().cloned());
    }

    let value = json!({
        "feature": feature,
        "stage": stage,
        "knowledge_arm": arm,
        "knowledge": knowledge,
        "code": code,
        "writes": [".bee/logs/dispatch.jsonl"],
    });
    Ok(PackAnswer { value, lines, control_root })
}

fn record_row(answer: &PackAnswer, feature: &str, stage: &str) -> Map<String, Value> {
    let list = |key: &str, field: &str| {
        Value::Array(
            answer.value[key]
                .as_array()
                .map(|rows| {
                    rows.iter().filter_map(|r| r[field].as_str().map(|s| Value::String(s.to_string()))).collect()
                })
                .unwrap_or_default(),
        )
    };
    let mut record = Map::new();
    record.insert("feature".into(), Value::String(feature.to_string()));
    record.insert("stage".into(), Value::String(stage.to_string()));
    record.insert("knowledge_arm".into(), answer.value["knowledge_arm"].clone());
    record.insert("learned_context".into(), list("knowledge", "path"));
    record.insert("pack_code".into(), list("code", "path"));
    record
}

fn render_knowledge(item: &PackLine) -> String {
    match &item.title {
        Some(title) => format!("- {} — {title}", item.path),
        None => format!("- {}", item.path),
    }
}

pub(crate) fn run_pack(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "stage", "lane"]) {
        return None;
    }
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return None,
    };
    let stage = match flags.get("stage") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return None,
    };
    let lane = match flags.get("lane") {
        None => PACK_DEFAULT_LANE.to_string(),
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return None,
    };
    if lane_budget(&lane).is_none() {
        return None;
    }

    let ctx = match g_prelude("knowledge pack", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    if !PACK_STAGES.contains(&stage.as_str()) {
        return Some(ctx.fail(&format!(
            "knowledge pack: bad_stage — --stage must be one of {}, got {stage:?}.",
            PACK_STAGES.join("|")
        )));
    }
    let answer = build_pack(&ctx.root, &feature, &stage, &lane).ok()?;
    if !crate::verbs::feedback::value_js_safe(&answer.value) {
        return None;
    }
    append_pack_record(&answer.control_root, &record_row(&answer, &feature, &stage));
    Some(ctx.emit(&answer.value, &answer.lines.join("\n"), 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git").args(args).current_dir(root).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn put(root: &Path, rel: &str, body: &str) {
        let file = join_rel(root, rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, body).unwrap();
    }

    fn commit(root: &Path, message: &str) -> String {
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", message]);
        git(root, &["rev-parse", "HEAD"])
    }

    fn init_repo(root: &Path) -> String {
        std::fs::create_dir_all(root).unwrap();
        git(root, &["init", "-q"]);
        git(root, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        git(root, &["config", "user.email", "pack@example.com"]);
        git(root, &["config", "user.name", "pack tests"]);
        git(root, &["config", "commit.gpgsign", "false"]);
        put(root, ".bee/onboarding.json", "{}\n");
        put(root, ".bee/config.json", "{}\n");
        put(root, "src/lib.rs", "pub fn one() {}\n");
        put(root, "src/two.rs", "pub fn two() {}\n");
        put(root, "src/docs-shadow.rs", "pub fn shadow() {}\n");
        commit(root, "code")
    }

    fn area(id: &str, anchors: Option<&[&str]>, verified: Option<&str>) -> String {
        let mut text = format!(
            "---\ntype: bee.area\ntitle: {id} area\ndescription: the {id} area\nbee:\n  id: {id}\n  lifecycle: active\n  areas: [{id}]\n"
        );
        if let Some(list) = anchors {
            text.push_str(&format!("  anchors: [{}]\n", list.join(", ")));
        }
        if let Some(vc) = verified {
            text.push_str(&format!("  verified_at: 2026-01-01\n  verified_commit: {vc}\n"));
        }
        text.push_str("---\nAnchored by hand.\n");
        text
    }

    fn work_item(feature: &str, required: &[&str]) -> String {
        format!(
            "---\ntype: bee.work-item\ntitle: {feature} item\ndescription: the {feature} work item\nbee:\n  id: {feature}\n  lifecycle: active\n  required_context: [{}]\n---\nBody.\n",
            required.join(", ")
        )
    }

    fn bundle_repo(root: &Path) -> String {
        let first = init_repo(root);
        put(
            root,
            "docs/knowledge/tasks/demo/item.md",
            &work_item("demo", &["areas/demo/one.md", "areas/demo/two.md", "areas/demo/plain.md"]),
        );
        put(root, "docs/knowledge/areas/demo/one.md", &area("one", Some(&["src/lib.rs"]), Some(&first)));
        put(
            root,
            "docs/knowledge/areas/demo/two.md",
            &area("two", Some(&["src/two.rs", "src/gone.rs", "docs/knowledge/areas/demo/one.md"]), None),
        );
        put(root, "docs/knowledge/areas/demo/plain.md", &area("plain", None, None));
        put(root, "src/anchored-target.rs", "pub fn three() {}\n");
        put(
            root,
            "docs/knowledge/decisions/demo/keep.md",
            "---\ntype: bee.decision\ntitle: Keep it\ndescription: a kept decision\nbee:\n  id: keep\n  lifecycle: active\n  areas: [one]\n---\nKept.\n",
        );
        commit(root, "concepts");
        first
    }

    struct Cwd {
        _lock: std::sync::MutexGuard<'static, ()>,
        orig: PathBuf,
    }

    impl Cwd {
        fn enter(path: &Path) -> Self {
            let lock = crate::verbs::drivers::TEST_CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let orig = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { _lock: lock, orig }
        }
    }

    impl Drop for Cwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.orig);
        }
    }

    fn run_verb(args: &[&str]) -> Option<ExitCode> {
        let mut argv: Vec<OsString> = vec![OsString::from("knowledge")];
        argv.extend(args.iter().map(OsString::from));
        try_native(&argv, Instant::now())
    }

    fn rows(root: &Path) -> Vec<Value> {
        let path = root.join(".bee/logs/dispatch.jsonl");
        let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
        text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
    }

    fn pack_row(root: &Path) -> Value {
        rows(root)
            .into_iter()
            .filter(|r| r["source"] == json!("pack"))
            .next_back()
            .expect("a pack row")
    }

    fn feature_into_arm(root: &Path, want: &str) -> String {
        for i in 0..200 {
            let feature = format!("feature-{i}");
            if feature_arm(root, &feature) == want {
                return feature;
            }
        }
        panic!("no feature landed in the {want} arm");
    }

    fn set_holdout(root: &Path, percent: u64) {
        put(root, ".bee/config.json", &format!("{{\"knowledge_holdout_percent\":{percent}}}\n"));
    }

    #[test]
    fn feature_pack_lists_knowledge_then_the_code_its_concepts_anchor() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        bundle_repo(&root);
        set_holdout(&root, 0);
        let _cwd = Cwd::enter(&root);
        assert_eq!(
            run_verb(&["pack", "--feature", "demo", "--stage", "shaping", "--json"]),
            Some(ExitCode::SUCCESS)
        );
        let row = pack_row(&root);
        assert_eq!(row["kind"], json!("pack"));
        assert_eq!(row["feature"], json!("demo"));
        assert_eq!(row["stage"], json!("shaping"));
        assert_eq!(row["knowledge_arm"], json!("knowledge"));
        let knowledge = row["learned_context"].as_array().unwrap().clone();
        assert!(
            knowledge.iter().any(|p| p == &json!("docs/knowledge/areas/demo/one.md")),
            "{knowledge:?}"
        );
        let code = row["pack_code"].as_array().unwrap().clone();
        assert_eq!(code, vec![json!("src/lib.rs"), json!("src/two.rs")]);
    }

    #[test]
    fn feature_pack_skips_anchors_under_docs_missing_or_already_read_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        bundle_repo(&root);
        set_holdout(&root, 0);
        let _cwd = Cwd::enter(&root);
        run_verb(&["pack", "--feature", "demo", "--stage", "shaping", "--json"]);
        let code: Vec<String> = pack_row(&root)["pack_code"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(!code.iter().any(|p| p == "src/gone.rs"), "{code:?}");
        assert!(!code.iter().any(|p| p.starts_with("docs/")), "{code:?}");
        assert_eq!(code.iter().filter(|p| *p == "src/lib.rs").count(), 1, "{code:?}");
    }

    #[test]
    fn feature_pack_grades_area_concepts_and_leaves_anchorless_ones_null() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        bundle_repo(&root);
        set_holdout(&root, 0);
        let _cwd = Cwd::enter(&root);
        run_verb(&["pack", "--feature", "demo", "--stage", "planning", "--json"]);
        let listed = pack_row(&root)["learned_context"].as_array().unwrap().clone();
        assert!(listed.iter().any(|p| p == &json!("docs/knowledge/areas/demo/plain.md")), "{listed:?}");
        let value = pack_value(&root, "demo", "planning");
        let plain = value["knowledge"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"] == json!("docs/knowledge/areas/demo/plain.md"))
            .cloned()
            .unwrap();
        assert_eq!(plain["grade"], Value::Null);
        assert_eq!(plain["title"], json!("plain area"));
        let one = value["knowledge"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"] == json!("docs/knowledge/areas/demo/one.md"))
            .cloned()
            .unwrap();
        assert_eq!(one["grade"], json!("confirmed"));
        let code = value["code"].as_array().unwrap().clone();
        assert!(
            code.iter()
                .any(|r| r["path"] == json!("src/lib.rs")
                    && r["anchored_by"] == json!("docs/knowledge/areas/demo/one.md")),
            "{code:?}"
        );
        assert_eq!(value["writes"], json!([".bee/logs/dispatch.jsonl"]));
        assert_eq!(value["knowledge_arm"], json!("knowledge"));
    }

    fn pack_value(root: &Path, feature: &str, stage: &str) -> Value {
        build_pack(root, feature, stage, PACK_DEFAULT_LANE).ok().expect("a pack").value
    }

    #[test]
    fn feature_pack_with_an_unanchored_bundle_leaves_the_cell_block_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let first = init_repo(&root);
        put(&root, "docs/knowledge/tasks/demo/item.md", &work_item("demo", &["areas/demo/plain.md"]));
        put(&root, "docs/knowledge/areas/demo/plain.md", &area("plain", None, Some(&first)));
        commit(&root, "concept");
        let cell = json!({ "id": "c-1", "feature": "demo", "lane": "small" });
        let lines = learned_context_lines(&root, &cell).unwrap();
        assert_eq!(
            lines,
            vec![
                "- docs/knowledge/tasks/demo/item.md — demo item",
                "- docs/knowledge/areas/demo/plain.md — plain area",
            ]
        );
        assert!(split_pack_lines(&lines).1.is_empty());
    }

    #[test]
    fn a_feature_pack_stops_at_eight_code_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let first = init_repo(&root);
        let mut anchors: Vec<String> = Vec::new();
        for i in 0..9 {
            let rel = format!("src/gen-{i}.rs");
            put(&root, &rel, "x\n");
            anchors.push(rel);
        }
        let refs: Vec<&str> = anchors.iter().map(String::as_str).collect();
        put(&root, "docs/knowledge/tasks/demo/item.md", &work_item("demo", &["areas/demo/many.md"]));
        put(&root, "docs/knowledge/areas/demo/many.md", &area("many", Some(&refs), Some(&first)));
        commit(&root, "concept");
        let cell = json!({ "id": "c-1", "feature": "demo", "lane": "small" });
        let lines = learned_context_lines(&root, &cell).unwrap();
        let (knowledge, code) = split_pack_lines(&lines);
        assert_eq!(knowledge.len(), 2);
        assert_eq!(code.len(), 8);
    }

    #[test]
    fn feature_pack_on_the_holdout_arm_prints_an_empty_pack_and_records_holdout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        bundle_repo(&root);
        set_holdout(&root, 100);
        let feature = feature_into_arm(&root, "holdout");
        let _cwd = Cwd::enter(&root);
        assert_eq!(
            run_verb(&["pack", "--feature", &feature, "--stage", "shaping", "--json"]),
            Some(ExitCode::SUCCESS)
        );
        let row = pack_row(&root);
        assert_eq!(row["knowledge_arm"], json!("holdout"));
        assert_eq!(row["learned_context"], json!([]));
        assert_eq!(row["pack_code"], json!([]));
    }

    #[test]
    fn feature_pack_refuses_a_stage_outside_shaping_and_planning_without_a_row() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        bundle_repo(&root);
        set_holdout(&root, 0);
        let _cwd = Cwd::enter(&root);
        let before = rows(&root).len();
        assert_ne!(
            run_verb(&["pack", "--feature", "demo", "--stage", "deploy"]),
            Some(ExitCode::SUCCESS)
        );
        assert_eq!(rows(&root).len(), before, "a refused stage must append no row");
        assert_eq!(
            run_verb(&["pack", "--feature", "demo", "--stage", "shaping"]),
            Some(ExitCode::SUCCESS)
        );
        assert_eq!(rows(&root).len(), before + 1);
    }

    #[test]
    fn feature_pack_with_no_bundle_is_an_empty_pack_and_exit_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        init_repo(&root);
        set_holdout(&root, 0);
        let _cwd = Cwd::enter(&root);
        assert_eq!(
            run_verb(&["pack", "--feature", "nothing", "--stage", "shaping", "--json"]),
            Some(ExitCode::SUCCESS)
        );
        let row = pack_row(&root);
        assert_eq!(row["knowledge_arm"], json!("knowledge"));
        assert_eq!(row["learned_context"], json!([]));
        assert_eq!(row["pack_code"], json!([]));
    }

    #[test]
    fn the_feature_arm_is_a_hash_of_the_feature_slug() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        init_repo(&root);
        set_holdout(&root, 0);
        assert_eq!(feature_arm(&root, "anything"), "knowledge");
        set_holdout(&root, 100);
        assert_eq!(feature_arm(&root, "anything"), "holdout");
        assert_eq!(feature_arm(&root, "anything"), feature_arm(&root, "anything"));
        assert_ne!(
            crate::verbs::reservations::sha256_hex("feature:anything"),
            crate::verbs::reservations::sha256_hex("anything")
        );
    }
}
