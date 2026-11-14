use crate::onboard::apply::run_skillstation;
use crate::verbs::knowledge::{g_prelude, pre_json_scan, GPre};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

pub(crate) fn principles_on_hub(root: &Path) -> bool {
    crate::state::read_config_raw(root)
        .get("skillstation")
        .and_then(|s| s.get("principles"))
        .and_then(Value::as_str)
        == Some("hub")
}

#[cfg(not(test))]
pub(crate) fn hub_runner(root: &Path) -> Box<dyn Fn(&[&str]) -> Result<Value, String>> {
    let root = root.to_path_buf();
    Box::new(move |args| run_skillstation(&root, args))
}

#[cfg(test)]
thread_local! {
    static TEST_HUB: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(&[&str]) -> Result<Value, String>>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_test_hub(run: Option<std::rc::Rc<dyn Fn(&[&str]) -> Result<Value, String>>>) {
    TEST_HUB.with(|h| *h.borrow_mut() = run);
}

#[cfg(test)]
pub(crate) fn hub_runner(_root: &Path) -> Box<dyn Fn(&[&str]) -> Result<Value, String>> {
    match TEST_HUB.with(|h| h.borrow().clone()) {
        Some(run) => Box::new(move |args| run(args)),
        None => Box::new(|_| Err("skillstation is not on PATH".to_string())),
    }
}

pub(crate) struct Related {
    pub(crate) id: &'static str,
    pub(crate) repo: &'static str,
    pub(crate) path: &'static str,
}

const PLUGINS_REPO: &str = "https://github.com/cursor/plugins";

pub(crate) const RELATED_SKILLS: [Related; 3] = [
    Related { id: "statechart", repo: PLUGINS_REPO, path: "pstack/skills/statechart" },
    Related {
        id: "create-formal-verification-skill",
        repo: PLUGINS_REPO,
        path: "pstack/skills/create-formal-verification-skill",
    },
    Related {
        id: "maintain-formal-verification-skill",
        repo: PLUGINS_REPO,
        path: "pstack/skills/maintain-formal-verification-skill",
    },
];

pub(crate) type Run<'a> = &'a dyn Fn(&[&str]) -> Result<Value, String>;

pub(crate) struct Hub {
    pub(crate) status: String,
    pub(crate) description: String,
    pub(crate) disable_model: bool,
    pub(crate) files: Vec<(String, String)>,
}

pub(crate) enum HubErr {
    Missing,
    Skip(String),
}

fn frontmatter_disables_model(content: &str) -> bool {
    let mut lines = content.lines();
    if lines.next().map(str::trim) != Some("---") {
        return false;
    }
    for line in lines {
        let line = line.trim_end();
        if line == "---" {
            return false;
        }
        if line == "disable-model-invocation: true" {
            return true;
        }
    }
    false
}

pub(crate) fn hub_show(run: Run, id: &str) -> Result<Hub, HubErr> {
    let v = run(&["skill", "show", id, "--json"]).map_err(|reason| {
        if reason.contains("skill not found") {
            HubErr::Missing
        } else {
            HubErr::Skip(reason)
        }
    })?;
    let status = v
        .pointer("/manifest/status")
        .and_then(Value::as_str)
        .ok_or_else(|| HubErr::Skip("skillstation show returned no manifest status".to_string()))?
        .to_string();
    let description = v
        .pointer("/manifest/description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let disable_model = v
        .get("content")
        .and_then(Value::as_str)
        .is_some_and(frontmatter_disables_model);
    let files = v
        .pointer("/manifest/resources")
        .and_then(Value::as_array)
        .map(|resources| {
            resources
                .iter()
                .filter_map(|r| {
                    let path = r.get("path").and_then(Value::as_str)?;
                    let digest = r.get("digest").and_then(Value::as_str)?;
                    let rel = path.split_once(&format!("/{id}/"))?.1;
                    Some((rel.to_string(), digest.strip_prefix("sha256:").unwrap_or(digest).to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Hub { status, description, disable_model, files })
}

pub(crate) fn render_alias(id: &str, description: &str, disable_model: bool) -> String {
    let quoted = serde_json::to_string(description).unwrap_or_else(|_| "\"\"".to_string());
    let disable = if disable_model { "disable-model-invocation: true\n" } else { "" };
    format!(
        "---\nname: {id}\ndescription: {quoted}\n{disable}metadata:\n  skillstation_alias: {id}\n---\n\
This skill lives in the Skillstation hub.\n\
Call the skillstation MCP tool `skill_get` with skill_id `{id}`.\n\
Follow the returned content as this skill.\n\
If the tool is missing, tell the user to run `skillstation connect -g`.\n"
    )
}

pub(crate) fn carries_alias_marker(skill_md: &str, id: &str) -> bool {
    let marker = format!("skillstation_alias: {id}");
    skill_md.lines().any(|l| l.trim() == marker)
}

fn valid_skill_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !id.contains("..")
}

fn write_alias(root: &Path, id: &str, hub: &Hub, force: bool) -> Result<PathBuf, String> {
    let dir = root.join(".claude").join("skills").join(id);
    let file = dir.join("SKILL.md");
    if let Ok(meta) = std::fs::symlink_metadata(&dir) {
        let owned = !meta.file_type().is_symlink()
            && std::fs::read_to_string(&file).is_ok_and(|s| carries_alias_marker(&s, id));
        if !owned {
            if !force {
                return Err(format!(
                    "{} already exists and is not a skillstation alias. FIX: move it away, or pass --force to replace its SKILL.md.",
                    dir.display()
                ));
            }
            if meta.file_type().is_symlink() {
                std::fs::remove_file(&dir)
                    .map_err(|e| format!("could not replace the link {} - {e}", dir.display()))?;
            }
        }
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not create {} - {e}", dir.display()))?;
    std::fs::write(&file, render_alias(id, &hub.description, hub.disable_model))
        .map_err(|e| format!("could not write {} - {e}", file.display()))?;
    Ok(file)
}

fn row(id: &str, state: &str, reason: &str, next: &str) -> Value {
    json!({ "id": id, "state": state, "reason": reason, "next": next })
}

fn install_row(root: &Path, run: Run, rel: &Related, apply: bool) -> Value {
    let id = rel.id;
    let hub = match hub_show(run, id) {
        Ok(h) => h,
        Err(HubErr::Skip(reason)) => {
            return row(id, "skipped", &reason, "install skillstation, then re-run `bee skillstation install`");
        }
        Err(HubErr::Missing) => {
            if !apply {
                return row(id, "missing", "", "run `bee skillstation install --apply` to add it as a draft");
            }
            return match run(&["skill", "add", rel.repo, "--path", rel.path, "--id", id, "--yes", "--json"]) {
                Ok(_) => row(
                    id,
                    "added-draft",
                    "",
                    &format!("the hub owner runs `skillstation skill review {id}`"),
                ),
                Err(reason) => row(id, "skipped", &format!("skill add - {reason}"), "fix the reason, then re-run with --apply"),
            };
        }
    };
    if hub.status != "active" {
        let next = if hub.status == "draft" {
            format!("the hub owner runs `skillstation skill review {id}`")
        } else {
            format!("the hub owner brings {id} back to active")
        };
        return row(id, &format!("present-{}", hub.status), "", &next);
    }
    if !hub.disable_model {
        return row(id, "present-active", "", "none - agents can route to it");
    }
    if !apply {
        return row(
            id,
            "present-active",
            "",
            &format!("run `bee skillstation install --apply` to write the /{id} alias"),
        );
    }
    match write_alias(root, id, &hub, false) {
        Ok(file) => {
            let mut r = row(id, "present-active", "", &format!("use /{id}"));
            r["alias"] = json!(file.display().to_string());
            r
        }
        Err(why) => row(id, "present-active", &why, &format!("run `bee skillstation alias {id} --force` to replace it")),
    }
}

fn install_report(root: &Path, run: Run, apply: bool) -> Value {
    let rows: Vec<Value> = RELATED_SKILLS.iter().map(|rel| install_row(root, run, rel, apply)).collect();
    json!({ "apply": apply, "skills": rows })
}

fn install_text(report: &Value) -> String {
    let mut lines = Vec::new();
    for r in report["skills"].as_array().into_iter().flatten() {
        let mut line = format!("{}: {}", r["id"].as_str().unwrap_or(""), r["state"].as_str().unwrap_or(""));
        if let Some(reason) = r["reason"].as_str().filter(|s| !s.is_empty()) {
            line.push_str(&format!(" ({reason})"));
        }
        if let Some(alias) = r["alias"].as_str() {
            line.push_str(&format!(" [alias {alias}]"));
        }
        line.push_str(&format!(" - next: {}", r["next"].as_str().unwrap_or("")));
        lines.push(line);
    }
    if report["apply"] != json!(true) {
        lines.push("Preview only - nothing was written. Add --apply to act.".to_string());
    }
    lines.join("\n")
}

fn alias_one(root: &Path, run: Run, id: &str, force: bool) -> Result<PathBuf, String> {
    if !valid_skill_id(id) {
        return Err(format!("{id:?} is not a valid skill id"));
    }
    let hub = match hub_show(run, id) {
        Ok(h) => h,
        Err(HubErr::Missing) => {
            return Err(format!("skill {id} is not on the hub. FIX: `skillstation skill list` shows what is."));
        }
        Err(HubErr::Skip(reason)) => return Err(reason),
    };
    if hub.status != "active" {
        return Err(format!(
            "skill {id} is {} on the hub, not active. FIX: the hub owner runs `skillstation skill review {id}`.",
            hub.status
        ));
    }
    write_alias(root, id, &hub, force)
}

enum Cmd {
    Install { apply: bool },
    Alias { id: String, force: bool },
}

fn parse(verb: &str, toks: &[&str]) -> Option<(Cmd, bool)> {
    let json = toks.contains(&"--json");
    let has = |flag: &str| toks.contains(&flag);
    match verb {
        "install" => {
            if !toks.iter().all(|t| matches!(*t, "--apply" | "--json")) {
                return None;
            }
            Some((Cmd::Install { apply: has("--apply") }, json))
        }
        "alias" => {
            let (id, rest) = toks.split_first()?;
            if id.starts_with("--") || !rest.iter().all(|t| matches!(*t, "--force" | "--json")) {
                return None;
            }
            Some((Cmd::Alias { id: (*id).to_string(), force: has("--force") }, json))
        }
        _ => None,
    }
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "skillstation" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    let (cmd, json) = parse(verb, &toks)?;
    let label = if matches!(cmd, Cmd::Install { .. }) { "skillstation install" } else { "skillstation alias" };
    let ctx = match g_prelude(label, json, pre_json_scan(&toks), t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let run = |a: &[&str]| run_skillstation(&ctx.root, a);
    Some(match cmd {
        Cmd::Install { apply } => {
            let report = install_report(&ctx.root, &run, apply);
            let text = install_text(&report);
            ctx.emit(&report, &text, 0)
        }
        Cmd::Alias { id, force } => match alias_one(&ctx.root, &run, &id, force) {
            Ok(file) => {
                let rel = file.strip_prefix(&ctx.root).unwrap_or(&file).display().to_string();
                let result = json!({ "id": id, "alias": rel });
                let text = format!("Wrote {rel}. Use /{id}; the agent loads the skill through skill_get.");
                ctx.emit(&result, &text, 0)
            }
            Err(why) => ctx.fail(&why),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn hub(status: &str, disable: bool) -> Value {
        let fm = if disable { "disable-model-invocation: true\n" } else { "" };
        json!({
            "status": "ok",
            "manifest": { "status": status, "description": "A hub\nskill." },
            "content": format!("---\nname: x\n{fm}---\nbody\n"),
        })
    }

    fn fixed(answers: Vec<(&'static str, Result<Value, String>)>) -> impl Fn(&[&str]) -> Result<Value, String> {
        move |args: &[&str]| {
            if args.first() == Some(&"skill") && args.get(1) == Some(&"add") {
                return Ok(json!({ "status": "ok" }));
            }
            let id = args.get(2).copied().unwrap_or("");
            answers
                .iter()
                .find(|(k, _)| *k == id)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| Err("skill not found: ".to_string() + id))
        }
    }

    fn skill_md(root: &Path, id: &str) -> PathBuf {
        root.join(".claude").join("skills").join(id).join("SKILL.md")
    }

    fn principles_cfg(root: &Path, tracked: Option<&str>, local: Option<&str>) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        if let Some(t) = tracked {
            std::fs::write(root.join(".bee/config.json"), t).unwrap();
        }
        if let Some(l) = local {
            std::fs::write(root.join(".bee/config.local.json"), l).unwrap();
        }
    }

    #[test]
    fn principles_on_hub_is_true_only_for_the_string_hub() {
        let d = tempfile::tempdir().unwrap();
        assert!(!principles_on_hub(d.path()));
        principles_cfg(d.path(), Some("{}"), None);
        assert!(!principles_on_hub(d.path()));
        principles_cfg(d.path(), Some(r#"{"skillstation":{"principles":"hub"}}"#), None);
        assert!(principles_on_hub(d.path()));
        principles_cfg(d.path(), Some(r#"{"skillstation":{"principles":"local"}}"#), None);
        assert!(!principles_on_hub(d.path()));
        principles_cfg(d.path(), Some(r#"{"skillstation":{"principles":true}}"#), None);
        assert!(!principles_on_hub(d.path()));
    }

    #[test]
    fn principles_on_hub_honours_the_local_overlay() {
        let d = tempfile::tempdir().unwrap();
        principles_cfg(
            d.path(),
            Some(r#"{"skillstation":{"principles":"local"}}"#),
            Some(r#"{"skillstation":{"principles":"hub"}}"#),
        );
        assert!(principles_on_hub(d.path()));
        principles_cfg(
            d.path(),
            Some(r#"{"skillstation":{"principles":"hub"}}"#),
            Some(r#"{"skillstation":{"principles":"local"}}"#),
        );
        assert!(!principles_on_hub(d.path()));
    }

    #[test]
    fn alias_render_carries_skill_get_marker_and_only_the_hub_flag() {
        let on = render_alias("statechart", "Check a model.", true);
        assert!(on.contains("skill_get"));
        assert!(on.contains("skill_id `statechart`"));
        assert!(on.contains("skillstation connect -g"));
        assert!(on.contains("disable-model-invocation: true\n"));
        assert!(on.contains("metadata:\n  skillstation_alias: statechart\n"));
        assert!(on.contains("description: \"Check a model.\"\n"));
        let off = render_alias("statechart", "Check a model.", false);
        assert!(!off.contains("disable-model-invocation"));
        let body = on.split("---\n").nth(2).unwrap();
        assert!(body.lines().count() <= 6);
    }

    #[test]
    fn frontmatter_flag_is_read_only_from_the_frontmatter() {
        assert!(frontmatter_disables_model("---\nname: a\ndisable-model-invocation: true\n---\n"));
        assert!(!frontmatter_disables_model("---\nname: a\n---\ndisable-model-invocation: true\n"));
        assert!(!frontmatter_disables_model("name: a\ndisable-model-invocation: true\n"));
    }

    #[test]
    fn alias_writes_one_file_and_rewrites_its_own_alias() {
        let dir = tempfile::tempdir().unwrap();
        let run = fixed(vec![("statechart", Ok(hub("active", false)))]);
        let file = alias_one(dir.path(), &run, "statechart", false).unwrap();
        assert_eq!(file, skill_md(dir.path(), "statechart"));
        let first = std::fs::read_to_string(&file).unwrap();
        assert!(first.contains("skill_get") && first.contains("skillstation_alias: statechart"));
        assert!(!first.contains("disable-model-invocation"));
        assert!(alias_one(dir.path(), &run, "statechart", false).is_ok());
        let entries: Vec<_> = std::fs::read_dir(file.parent().unwrap()).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn a_foreign_skill_folder_is_refused_until_force() {
        let dir = tempfile::tempdir().unwrap();
        let file = skill_md(dir.path(), "statechart");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "mine\n").unwrap();
        let run = fixed(vec![("statechart", Ok(hub("active", true)))]);
        let err = alias_one(dir.path(), &run, "statechart", false).unwrap_err();
        assert!(err.contains(&file.parent().unwrap().display().to_string()), "{err}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "mine\n");
        alias_one(dir.path(), &run, "statechart", true).unwrap();
        let after = std::fs::read_to_string(&file).unwrap();
        assert!(after.contains("skillstation_alias: statechart") && after.contains("disable-model-invocation: true"));
    }

    #[test]
    fn an_alias_for_another_id_is_not_owned() {
        let dir = tempfile::tempdir().unwrap();
        let file = skill_md(dir.path(), "statechart");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "---\nmetadata:\n  skillstation_alias: other\n---\n").unwrap();
        let run = fixed(vec![("statechart", Ok(hub("active", false)))]);
        assert!(alias_one(dir.path(), &run, "statechart", false).is_err());
    }

    #[test]
    fn alias_refuses_a_draft_a_missing_skill_and_a_bad_id() {
        let dir = tempfile::tempdir().unwrap();
        let run = fixed(vec![("drafty", Ok(hub("draft", false)))]);
        let draft = alias_one(dir.path(), &run, "drafty", false).unwrap_err();
        assert!(draft.contains("draft"), "{draft}");
        let gone = alias_one(dir.path(), &run, "nope", false).unwrap_err();
        assert!(gone.contains("not on the hub"), "{gone}");
        assert!(alias_one(dir.path(), &run, "../evil", false).is_err());
        assert!(!dir.path().join(".claude").exists());
    }

    #[test]
    fn install_without_apply_lists_every_row_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let run = fixed(vec![
            ("statechart", Ok(hub("active", true))),
            ("create-formal-verification-skill", Ok(hub("draft", false))),
        ]);
        let report = install_report(dir.path(), &run, false);
        let rows = report["skills"].as_array().unwrap();
        assert_eq!(rows.len(), RELATED_SKILLS.len());
        let states: Vec<&str> = rows.iter().map(|r| r["state"].as_str().unwrap()).collect();
        assert_eq!(states, ["present-active", "present-draft", "missing"]);
        assert!(!dir.path().join(".claude").exists());
        let text = install_text(&report);
        assert!(text.contains("skillstation skill review create-formal-verification-skill"));
        assert!(text.contains("Preview only"));
    }

    #[test]
    fn install_apply_adds_missing_rows_and_aliases_only_user_started_skills() {
        let dir = tempfile::tempdir().unwrap();
        let calls = Mutex::new(Vec::<String>::new());
        let inner = fixed(vec![
            ("statechart", Ok(hub("active", true))),
            ("create-formal-verification-skill", Ok(hub("active", false))),
        ]);
        let run = |a: &[&str]| {
            calls.lock().unwrap().push(a.join(" "));
            inner(a)
        };
        let report = install_report(dir.path(), &run, true);
        assert!(skill_md(dir.path(), "statechart").is_file());
        assert!(!skill_md(dir.path(), "create-formal-verification-skill").exists());
        let missing = &report["skills"][2];
        assert_eq!(missing["state"], "added-draft");
        assert!(missing["next"].as_str().unwrap().contains("skillstation skill review maintain-formal-verification-skill"));
        let calls = calls.lock().unwrap();
        assert!(calls.contains(
            &"skill add https://github.com/cursor/plugins --path pstack/skills/maintain-formal-verification-skill --id maintain-formal-verification-skill --yes --json".to_string()
        ));
        assert!(calls.iter().all(|c| !c.contains("activate") && !c.contains("approve")));
    }

    #[test]
    fn install_apply_reports_a_foreign_folder_and_still_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let file = skill_md(dir.path(), "statechart");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "mine\n").unwrap();
        let run = fixed(vec![("statechart", Ok(hub("active", true)))]);
        let report = install_report(dir.path(), &run, true);
        assert!(report["skills"][0]["reason"].as_str().unwrap().contains("not a skillstation alias"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "mine\n");
    }

    #[test]
    fn a_missing_binary_or_bad_json_is_a_skip_row_per_skill() {
        let dir = tempfile::tempdir().unwrap();
        let absent = |_: &[&str]| Err::<Value, String>("skillstation is not on PATH".to_string());
        let report = install_report(dir.path(), &absent, true);
        let rows = report["skills"].as_array().unwrap();
        assert_eq!(rows.len(), RELATED_SKILLS.len());
        assert!(rows.iter().all(|r| r["state"] == "skipped" && r["reason"] == "skillstation is not on PATH"));
        let bare = |_: &[&str]| Ok::<Value, String>(json!({ "status": "ok" }));
        let report = install_report(dir.path(), &bare, false);
        assert!(report["skills"].as_array().unwrap().iter().all(|r| r["state"] == "skipped"));
        assert!(alias_one(dir.path(), &absent, "statechart", false).is_err());
    }

    #[cfg(unix)]
    mod fake_cli {
        use super::*;

        fn script(dir: &Path) -> PathBuf {
            let path = dir.join("skillstation");
            let body = format!(
                "#!/bin/sh\nPATH=/usr/bin:/bin\necho \"$@\" >> '{log}'\nif [ \"$1 $2\" = \"skill show\" ] && [ -f '{dir}/show-'\"$3\"'.json' ]; then cat '{dir}/show-'\"$3\"'.json'; exit 0; fi\nif [ \"$1 $2\" = \"skill add\" ]; then echo '{{\"status\":\"ok\"}}'; exit 0; fi\necho '{{\"status\":\"error\",\"error\":{{\"render\":{{\"WHY\":\"skill not found: '\"$3\"'\"}}}}}}'\nexit 2\n",
                log = dir.join("calls.log").display(),
                dir = dir.display()
            );
            std::fs::write(&path, body).unwrap();
            path
        }

        fn runner(script: PathBuf) -> impl Fn(&[&str]) -> Result<Value, String> {
            move |args: &[&str]| {
                let out = std::process::Command::new("/bin/sh").arg(&script).args(args).output().map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                let v: Value = serde_json::from_str(&text).map_err(|_| "no JSON".to_string())?;
                if !out.status.success() {
                    return Err(v.pointer("/error/render/WHY").and_then(Value::as_str).unwrap_or("failed").to_string());
                }
                Ok(v)
            }
        }

        #[test]
        fn a_fake_skillstation_script_drives_present_missing_and_apply() {
            let tools = tempfile::tempdir().unwrap();
            let repo = tempfile::tempdir().unwrap();
            let run = runner(script(tools.path()));
            std::fs::write(tools.path().join("show-statechart.json"), hub("active", true).to_string()).unwrap();

            let preview = install_report(repo.path(), &run, false);
            assert_eq!(preview["skills"][0]["state"], "present-active");
            assert_eq!(preview["skills"][1]["state"], "missing");
            assert!(!repo.path().join(".claude").exists());
            let log = std::fs::read_to_string(tools.path().join("calls.log")).unwrap();
            assert!(!log.contains("skill add"));

            let applied = install_report(repo.path(), &run, true);
            assert_eq!(applied["skills"][1]["state"], "added-draft");
            assert!(skill_md(repo.path(), "statechart").is_file());
            let log = std::fs::read_to_string(tools.path().join("calls.log")).unwrap();
            assert!(log.contains(
                "skill add https://github.com/cursor/plugins --path pstack/skills/create-formal-verification-skill --id create-formal-verification-skill --yes"
            ));
        }
    }
}
