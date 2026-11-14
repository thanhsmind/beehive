use crate::audit;
use crate::jsjson;
use crate::verbs::knowledge::{
    build_knowledge_used, build_knowledge_wrong, check_bundle, g_prelude_at, pre_json_scan, GPre, ORPHAN, SUSPECT,
};
use crate::verbs::reservations::{keys_known, parse_flags, FlagV};
use crate::verbs::worktree::run_git;
use chrono::DateTime;
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const SUSPECT_LOAD_MIN: usize = 5;
const PATHS_NAMED: usize = 3;
const SCHEMA: &str = "bee-upkeep-due/v1";
const EXIT_DUE: u8 = 10;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Knowledge,
    Verify,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Knowledge => "knowledge",
            Kind::Verify => "verify",
        }
    }

    fn skill(self) -> &'static str {
        match self {
            Kind::Knowledge => "bee-knowledge-upkeep",
            Kind::Verify => "bee-verify-upkeep",
        }
    }

    fn audit_kind(self) -> &'static str {
        match self {
            Kind::Knowledge => "knowledge-upkeep",
            Kind::Verify => "verify-upkeep",
        }
    }
}

struct LastRun {
    at: String,
    outcome: String,
    note: String,
}

impl LastRun {
    fn json(&self) -> Value {
        json!({"at": self.at, "outcome": self.outcome, "note": self.note})
    }
}

type Entry = (&'static str, String);

fn entries_json(entries: &[Entry]) -> Value {
    Value::Array(entries.iter().map(|(code, detail)| json!({"code": code, "detail": detail})).collect())
}

fn last_run(root: &Path, kind: Kind) -> Option<LastRun> {
    let row = audit::rows(root, Some(kind.audit_kind()), None).pop()?;
    let note = row.get("note").and_then(Value::as_str).unwrap_or_default().to_string();
    Some(LastRun {
        at: row.get("ts").and_then(Value::as_str).unwrap_or_default().to_string(),
        outcome: note.split_whitespace().next().unwrap_or_default().to_string(),
        note,
    })
}

fn epoch(iso: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(iso.trim()).ok().map(|d| d.timestamp())
}

fn newest_release_tag(root: &Path) -> Option<(String, String)> {
    let out = run_git(
        root,
        &[
            "for-each-ref",
            "--sort=-creatordate",
            "--count=1",
            "--format=%(refname:short)|%(creatordate:iso-strict)",
            "refs/tags/v*",
        ],
    );
    if out.status != Some(0) {
        return None;
    }
    let line = out.stdout?.trim().to_string();
    let (tag, at) = line.split_once('|')?;
    Some((tag.to_string(), at.to_string()))
}

fn named(paths: &[String]) -> String {
    let head: Vec<&str> = paths.iter().take(PATHS_NAMED).map(String::as_str).collect();
    let more = paths.len().saturating_sub(PATHS_NAMED);
    if more > 0 {
        format!("{} (+{more} more)", head.join(", "))
    } else {
        head.join(", ")
    }
}

fn release_and_never_run(root: &Path, last: &Option<LastRun>, reasons: &mut Vec<Entry>) {
    match last {
        None => reasons.push(("never_run", "no audit row of this kind has been logged".to_string())),
        Some(run) => {
            let Some((tag, tag_at)) = newest_release_tag(root) else { return };
            if let (Some(tag_epoch), Some(run_epoch)) = (epoch(&tag_at), epoch(&run.at)) {
                if tag_epoch > run_epoch {
                    reasons.push((
                        "release_since_last_run",
                        format!("tag {tag} ({tag_at}) is newer than the last run ({})", run.at),
                    ));
                }
            }
        }
    }
}

fn knowledge_signals(root: &Path, last: &Option<LastRun>, reasons: &mut Vec<Entry>, info: &mut Vec<Entry>) {
    let since = last.as_ref().map(|r| r.at.as_str());
    let (wrong, _) = build_knowledge_wrong(root);
    let reported: Vec<String> = wrong
        .get("concepts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|c| match since {
            None => true,
            Some(at) => epoch(at).is_some_and(|cut| {
                c.get("reported_at")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter_map(epoch)
                    .any(|t| t > cut)
            }),
        })
        .filter_map(|c| c.get("path").and_then(Value::as_str).map(str::to_string))
        .collect();
    if !reported.is_empty() {
        reasons.push((
            "reported_wrong",
            format!("{} concept(s) reported wrong by a cell capped since the last run: {}", reported.len(), named(&reported)),
        ));
    }

    let dir = root.join("docs").join("knowledge");
    let freshness = check_bundle(&dir, false).map(|r| r.freshness).unwrap_or_default();
    let orphans: Vec<String> = freshness.iter().filter(|(_, g)| g.grade == ORPHAN).map(|(p, _)| p.clone()).collect();
    if !orphans.is_empty() {
        reasons.push(("orphan", format!("{} concept(s) anchor a path that is gone: {}", orphans.len(), named(&orphans))));
    }

    let (used, _) = build_knowledge_used(root);
    let loads_of = |key: &str| -> Vec<(String, usize)> {
        used.get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|e| {
                Some((e.get("path")?.as_str()?.to_string(), e.get("loads")?.as_u64()? as usize))
            })
            .filter(|(_, n)| *n >= SUSPECT_LOAD_MIN)
            .collect()
    };
    let loaded = loads_of("loaded");
    let suspect_loaded: Vec<String> = freshness
        .iter()
        .filter(|(_, g)| g.grade == SUSPECT)
        .map(|(p, _)| format!("docs/knowledge/{p}"))
        .filter(|full| loaded.iter().any(|(path, _)| path == full))
        .collect();
    if !suspect_loaded.is_empty() {
        reasons.push((
            "suspect_loaded",
            format!(
                "{} suspect concept(s) loaded {SUSPECT_LOAD_MIN} or more times: {}",
                suspect_loaded.len(),
                named(&suspect_loaded)
            ),
        ));
    }
    let never_used: Vec<String> = loads_of("never_used").into_iter().map(|(p, _)| p).collect();
    if !never_used.is_empty() {
        info.push((
            "loaded_never_used",
            format!("{} concept(s) loaded {SUSPECT_LOAD_MIN} or more times and never used: {}", never_used.len(), named(&never_used)),
        ));
    }
}

struct Item {
    kind: Kind,
    present: bool,
    reasons: Vec<Entry>,
    info: Vec<Entry>,
    last: Option<LastRun>,
}

impl Item {
    fn due(&self) -> bool {
        self.present && !self.reasons.is_empty()
    }

    fn json(&self) -> Value {
        json!({
            "kind": self.kind.name(),
            "skill": self.kind.skill(),
            "invoke": format!("/{}", self.kind.skill()),
            "present": self.present,
            "due": self.due(),
            "reasons": entries_json(&self.reasons),
            "info": entries_json(&self.info),
            "last_run": self.last.as_ref().map(LastRun::json),
        })
    }

    fn lines(&self) -> Vec<String> {
        if !self.present {
            return vec![format!("{}: not present", self.kind.name())];
        }
        if !self.due() {
            return vec![format!("{}: not due", self.kind.name())];
        }
        let codes: Vec<&str> = self.reasons.iter().map(|(c, _)| *c).collect();
        vec![
            format!("{}: due — {}", self.kind.name(), codes.join(", ")),
            format!("  run: /{}", self.kind.skill()),
        ]
    }
}

fn assess(root: &Path, kind: Kind) -> Item {
    let last = last_run(root, kind);
    let present = match kind {
        Kind::Knowledge => root.join("docs").join("knowledge").join("index.md").is_file(),
        Kind::Verify => root.join(".bee").join("verify").join("verify-app").join("SKILL.md").is_file(),
    };
    let mut reasons = Vec::new();
    let mut info = Vec::new();
    if present {
        release_and_never_run(root, &last, &mut reasons);
        if kind == Kind::Knowledge {
            knowledge_signals(root, &last, &mut reasons, &mut info);
        }
    }
    Item { kind, present, reasons, info, last }
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
    if args.first()?.to_str()? != "upkeep" {
        return None;
    }
    if args.get(1)?.to_str()? != "due" {
        return None;
    }
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    if !keys_known(&flags, &["kind", "root"]) {
        return None;
    }
    let target = match flags.get("root") {
        None => PathBuf::from("."),
        Some(FlagV::S(s)) if !s.is_empty() => PathBuf::from(s),
        Some(_) => return None,
    };
    let kind_flag = match flags.get("kind") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(_) => return None,
    };
    let ctx = match g_prelude_at("upkeep due", json, pre_json, t0, Some(&target))? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let kinds: Vec<Kind> = match kind_flag.as_deref() {
        None => vec![Kind::Knowledge, Kind::Verify],
        Some("knowledge") => vec![Kind::Knowledge],
        Some("verify") => vec![Kind::Verify],
        Some(other) => {
            return Some(refuse(json, &format!("upkeep due: unknown --kind \"{other}\" — must be knowledge or verify.")));
        }
    };

    let items: Vec<Item> = kinds.into_iter().map(|k| assess(&ctx.root, k)).collect();
    let any_due = items.iter().any(Item::due);
    if json {
        let result = json!({
            "schema": SCHEMA,
            "any_due": any_due,
            "items": items.iter().map(Item::json).collect::<Vec<_>>(),
        });
        println!("{}", jsjson::stringify_pretty(&result));
    } else {
        let lines: Vec<String> = items.iter().flat_map(Item::lines).collect();
        println!("{}", lines.join("\n"));
    }
    Some(ExitCode::from(if any_due { EXIT_DUE } else { 0 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;

    fn store() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee/onboarding.json"), r#"{"schema_version":"1.0","bee_version":"0.1.0"}"#).unwrap();
        tmp
    }

    fn add_bundle(root: &Path) {
        let dir = root.join("docs/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.md"), "# Index\n").unwrap();
    }

    fn add_verify_skill(root: &Path) {
        let dir = root.join(".bee/verify/verify-app");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("SKILL.md"), "---\nname: verify-app\n---\n").unwrap();
    }

    fn log_row(root: &Path, kind: &str, ts: &str, note: &str) {
        let row = json!({"id": "x", "ts": ts, "kind": kind, "feature": "upkeep", "note": note, "source": "test"});
        let path = root.join(".bee/audit.jsonl");
        let mut text = std::fs::read_to_string(&path).unwrap_or_default();
        text.push_str(&format!("{row}\n"));
        std::fs::write(path, text).unwrap();
    }

    fn due(root: &Path, args: &[&str]) -> (Option<i32>, Value, String) {
        let mut full = vec!["upkeep", "due"];
        full.extend_from_slice(args);
        let out = Command::cargo_bin("bee").unwrap().args(&full).env_remove("BEE_HERDING_WORKER").current_dir(root).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let parsed = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
        (out.status.code(), parsed, stdout + &String::from_utf8_lossy(&out.stderr))
    }

    fn item<'a>(v: &'a Value, kind: &str) -> &'a Value {
        v["items"].as_array().unwrap().iter().find(|i| i["kind"] == kind).unwrap()
    }

    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push((p.clone(), std::fs::read(&p).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out
    }

    #[test]
    fn no_bundle_and_no_verify_skill_is_not_due_and_not_present() {
        let tmp = store();
        let (code, v, text) = due(tmp.path(), &["--json"]);
        assert_eq!(code, Some(0), "{text}");
        assert_eq!(v["schema"], "bee-upkeep-due/v1");
        assert_eq!(v["any_due"], false);
        for kind in ["knowledge", "verify"] {
            let i = item(&v, kind);
            assert_eq!(i["present"], false);
            assert_eq!(i["due"], false);
            assert_eq!(i["reasons"].as_array().unwrap().len(), 0);
            assert!(i["last_run"].is_null());
        }
        assert_eq!(item(&v, "knowledge")["invoke"], "/bee-knowledge-upkeep");
        assert_eq!(item(&v, "verify")["skill"], "bee-verify-upkeep");
    }

    #[test]
    fn a_bundle_with_no_audit_row_is_due_never_run() {
        let tmp = store();
        add_bundle(tmp.path());
        let (code, v, text) = due(tmp.path(), &["--json"]);
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(v["any_due"], true);
        let k = item(&v, "knowledge");
        assert_eq!(k["due"], true);
        assert_eq!(k["reasons"][0]["code"], "never_run");
        assert_eq!(item(&v, "verify")["due"], false);
    }

    #[test]
    fn a_verify_skill_with_no_audit_row_is_due_never_run() {
        let tmp = store();
        add_verify_skill(tmp.path());
        let (code, v, text) = due(tmp.path(), &["--kind", "verify", "--json"]);
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(v["items"].as_array().unwrap().len(), 1);
        assert_eq!(item(&v, "verify")["reasons"][0]["code"], "never_run");
    }

    #[test]
    fn an_upkeep_row_clears_never_run_and_reports_the_last_run() {
        let tmp = store();
        add_bundle(tmp.path());
        log_row(tmp.path(), "knowledge-upkeep", "2026-11-14T01:00:00.000Z", "clean — nothing to change");
        let (code, v, text) = due(tmp.path(), &["--json"]);
        assert_eq!(code, Some(0), "{text}");
        let k = item(&v, "knowledge");
        assert_eq!(k["due"], false);
        assert_eq!(k["last_run"]["outcome"], "clean");
        assert_eq!(k["last_run"]["at"], "2026-11-14T01:00:00.000Z");
    }

    #[test]
    fn a_tag_newer_than_the_last_run_makes_the_run_due_and_a_newer_row_clears_it() {
        let tmp = store();
        add_bundle(tmp.path());
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git").args(args).current_dir(tmp.path()).output().unwrap();
            assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        };
        git(&["init", "-q"]);
        git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "c"]);
        git(&["-c", "user.name=t", "-c", "user.email=t@t", "tag", "-a", "v1.0.0", "-m", "v1"]);
        log_row(tmp.path(), "knowledge-upkeep", "2020-01-01T00:00:00.000Z", "clean — old");
        let (code, v, text) = due(tmp.path(), &["--kind", "knowledge", "--json"]);
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(item(&v, "knowledge")["reasons"][0]["code"], "release_since_last_run");
        log_row(tmp.path(), "knowledge-upkeep", "2999-01-01T00:00:00.000Z", "clean — fresh");
        let (code, v, text) = due(tmp.path(), &["--kind", "knowledge", "--json"]);
        assert_eq!(code, Some(0), "{text}");
        assert_eq!(item(&v, "knowledge")["reasons"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn a_cell_reporting_a_concept_wrong_after_the_last_run_makes_it_due() {
        let tmp = store();
        add_bundle(tmp.path());
        log_row(tmp.path(), "knowledge-upkeep", "2026-11-14T01:00:00.000Z", "clean — ok");
        std::fs::create_dir_all(tmp.path().join(".bee/cells")).unwrap();
        let cell = |id: &str, at: &str| {
            json!({"id": id, "feature": "f", "status": "capped", "trace": {"capped_at": at, "knowledge_wrong": [{"path": format!("docs/knowledge/patterns/{id}.md"), "wrong": "stale"}]}})
        };
        std::fs::write(tmp.path().join(".bee/cells/old.json"), cell("old", "2026-11-13T00:00:00.000Z").to_string()).unwrap();
        let (code, _, text) = due(tmp.path(), &["--json"]);
        assert_eq!(code, Some(0), "{text}");
        std::fs::write(tmp.path().join(".bee/cells/new.json"), cell("new", "2026-11-14T02:00:00.000Z").to_string()).unwrap();
        let (code, v, text) = due(tmp.path(), &["--json"]);
        assert_eq!(code, Some(10), "{text}");
        let reason = &item(&v, "knowledge")["reasons"][0];
        assert_eq!(reason["code"], "reported_wrong");
        let detail = reason["detail"].as_str().unwrap();
        assert!(detail.contains("new.md") && !detail.contains("old.md"), "{detail}");
    }

    #[test]
    fn a_bad_kind_refuses_and_names_both_kinds() {
        let tmp = store();
        let (code, _, text) = due(tmp.path(), &["--kind", "vibes"]);
        assert_eq!(code, Some(1), "{text}");
        assert!(text.contains("knowledge") && text.contains("verify") && text.contains("vibes"), "{text}");
    }

    #[test]
    fn text_output_names_the_invoke_command_for_a_due_item() {
        let tmp = store();
        add_bundle(tmp.path());
        let (code, _, text) = due(tmp.path(), &[]);
        assert_eq!(code, Some(10), "{text}");
        assert!(text.contains("knowledge: due — never_run"), "{text}");
        assert!(text.contains("/bee-knowledge-upkeep"), "{text}");
        assert!(text.contains("verify: not present"), "{text}");
    }

    #[test]
    fn root_reads_another_store_and_nothing_is_written() {
        let target = store();
        add_bundle(target.path());
        add_verify_skill(target.path());
        let elsewhere = store();
        let before_target = snapshot(target.path());
        let before_elsewhere = snapshot(elsewhere.path());
        let (code, v, text) = due(elsewhere.path(), &["--root", target.path().to_str().unwrap(), "--json"]);
        assert_eq!(code, Some(10), "{text}");
        assert_eq!(item(&v, "knowledge")["present"], true);
        assert_eq!(item(&v, "verify")["present"], true);
        assert!(snapshot(target.path()) == before_target, "the target store changed");
        assert!(snapshot(elsewhere.path()) == before_elsewhere, "the cwd store changed");
    }
}
