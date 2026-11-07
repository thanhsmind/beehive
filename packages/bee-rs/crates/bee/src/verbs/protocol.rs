use crate::verbs::drivers::{ctx_serving_granted, declared_test_commands};
use crate::verbs::reservations::{finish, keys_known, sha256_hex, FlagV, Flags, Out};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const PROTOCOL_FILE: &str = "WORKSPACE_PROTOCOL.md";

const BOUNDARIES_KEY: &str = "HUMAN_DECISION_BOUNDARIES";

const VENDORED_TREES: [&str; 6] = [
    ".claude/skills",
    ".agents/skills",
    ".codex/skills",
    ".opencode/skills",
    ".claude-plugin/skills",
    ".codex-plugin/skills",
];

const REVIEW_FILES: [&str; 5] = ["LEADER.md", "AGENTS.md", "CLAUDE.md", "CONTRIBUTING.md", "docs/DEPLOY.md"];

const RELEASE_ENTRIES: [&str; 3] = ["scripts/release.sh", "scripts/deploy.sh", "docs/DEPLOY.md"];

const BASE_BOUNDARIES: [&str; 6] = [
    "a release, deploy, tag or push to a shared remote",
    "the uat acceptance of a standard or high-risk feature",
    "a change to gate_bypass or another gate rule",
    "reading or printing a secret-shaped file",
    "a P1 review finding",
    "a change that removes or weakens a guard or a pinned test",
];

const MERGE_AFTER_UAT: &str = "human approves uat, then the agent runs bee worktree merge";

const MERGE_BY_AGENT: &str = "agent, through bee worktree merge after a green bee test; the owner accepts at the uat door";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProtocolValue {
    Scalar(String),
    List(Vec<String>),
}

impl ProtocolValue {
    fn is_filled(&self) -> bool {
        match self {
            ProtocolValue::Scalar(s) => !s.is_empty(),
            ProtocolValue::List(items) => !items.is_empty(),
        }
    }

    fn joined(&self) -> String {
        match self {
            ProtocolValue::Scalar(s) => s.clone(),
            ProtocolValue::List(items) => items.join(" && "),
        }
    }

    fn holds_placeholder(&self) -> bool {
        let one = |s: &String| (s.contains('<') && s.contains('>')) || s.contains(" | ");
        match self {
            ProtocolValue::Scalar(s) => one(s),
            ProtocolValue::List(items) => items.iter().any(one),
        }
    }

    fn to_json(&self) -> Value {
        match self {
            ProtocolValue::Scalar(s) => Value::String(s.clone()),
            ProtocolValue::List(items) => json!(items),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProtocolState {
    Missing,
    Unreadable,
    Invalid,
    Valid,
}

impl ProtocolState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ProtocolState::Missing => "missing",
            ProtocolState::Unreadable => "unreadable",
            ProtocolState::Invalid => "invalid",
            ProtocolState::Valid => "valid",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProtocolReport {
    pub state: ProtocolState,
    pub issues: Vec<String>,
    pub keys: BTreeMap<String, ProtocolValue>,
    pub sha256: Option<String>,
}

struct Scan {
    keys: BTreeMap<String, ProtocolValue>,
    duplicates: Vec<String>,
}

fn key_line(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let key = &line[..colon];
    let mut chars = key.chars();
    if !chars.next()?.is_ascii_uppercase() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
        return None;
    }
    Some((key, line[colon + 1..].trim()))
}

fn list_item(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('-')?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest.trim())
    } else {
        None
    }
}

fn scan(text: &str) -> Scan {
    let mut entries: Vec<(String, ProtocolValue)> = Vec::new();
    let mut collecting = false;
    for line in text.lines() {
        if let Some((key, value)) = key_line(line) {
            collecting = value.is_empty();
            entries.push((key.to_string(), ProtocolValue::Scalar(value.to_string())));
        } else if collecting {
            match list_item(line) {
                Some("") => {}
                Some(item) => {
                    if let Some((_, value)) = entries.last_mut() {
                        match value {
                            ProtocolValue::List(items) => items.push(item.to_string()),
                            ProtocolValue::Scalar(_) => {
                                *value = ProtocolValue::List(vec![item.to_string()])
                            }
                        }
                    }
                }
                None => collecting = false,
            }
        }
    }
    let mut keys = BTreeMap::new();
    let mut duplicates = Vec::new();
    for (key, value) in entries {
        if keys.insert(key.clone(), value).is_some() && !duplicates.contains(&key) {
            duplicates.push(key);
        }
    }
    Scan { keys, duplicates }
}

pub(crate) fn parse(text: &str) -> BTreeMap<String, ProtocolValue> {
    scan(text).keys
}

fn issues_for(root: &Path, scan: &Scan) -> Vec<String> {
    let keys = &scan.keys;
    let mut issues: Vec<String> =
        scan.duplicates.iter().map(|k| format!("duplicate key {k}")).collect();
    for owner in ["MERGE_OWNER", "DEPLOY_OWNER"] {
        if !keys.get(owner).is_some_and(ProtocolValue::is_filled) {
            issues.push(format!("{owner} is missing or empty — name the owner, for example `{owner}: human`"));
        }
    }
    match keys.get(BOUNDARIES_KEY) {
        None => issues.push(format!("{BOUNDARIES_KEY} is missing — add the key with at least one `- item` line")),
        Some(ProtocolValue::Scalar(s)) if s.is_empty() => {
            issues.push(format!("{BOUNDARIES_KEY} has no items — add at least one `- item` line under it"))
        }
        Some(ProtocolValue::Scalar(_)) => issues.push(format!(
            "{BOUNDARIES_KEY} is a single value — use the `- item` list form, one boundary per line"
        )),
        Some(ProtocolValue::List(_)) => {}
    }
    for (key, value) in keys {
        if value.holds_placeholder() {
            issues.push(format!("{key} still holds a template placeholder"));
        }
    }
    let full_test = keys.get("FULL_TEST").map(ProtocolValue::joined).filter(|v| !v.is_empty());
    if let (Some(full_test), Ok(declared)) = (full_test, declared_test_commands(root)) {
        let configured = declared.map(|commands| commands.join(" && ")).unwrap_or_default();
        if full_test != configured {
            issues.push(
                "FULL_TEST drifts from .bee/config.json commands.test — delete FULL_TEST or make it equal"
                    .to_string(),
            );
        }
    }
    issues
}

pub(crate) fn protocol_state(root: &Path) -> ProtocolReport {
    let report = |state, issues, keys, sha256| ProtocolReport { state, issues, keys, sha256 };
    let unreadable = |why: String| {
        report(ProtocolState::Unreadable, vec![why], BTreeMap::new(), None)
    };
    let bytes = match std::fs::read(root.join(PROTOCOL_FILE)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return report(ProtocolState::Missing, Vec::new(), BTreeMap::new(), None)
        }
        Err(e) => return unreadable(format!("{PROTOCOL_FILE} cannot be read: {e}")),
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return unreadable(format!("{PROTOCOL_FILE} is not valid UTF-8"));
    };
    let scanned = scan(&text);
    let issues = issues_for(root, &scanned);
    let state = if issues.is_empty() { ProtocolState::Valid } else { ProtocolState::Invalid };
    report(state, issues, scanned.keys, Some(sha256_hex(&text)))
}

pub(crate) fn protocol_sha256(control_root: &Path) -> Option<String> {
    std::fs::read_to_string(control_root.join(PROTOCOL_FILE)).ok().map(|text| sha256_hex(&text))
}

pub(crate) struct Facts {
    pub project_id: Option<String>,
    pub branch: Option<String>,
    pub branch_from_origin: bool,
    pub remote: Option<String>,
    pub merge_after_uat: bool,
    pub release_entry: Option<String>,
    pub vendored: Vec<String>,
}

struct Fact {
    key: &'static str,
    value: ProtocolValue,
    source: String,
}

fn git_lines(root: &Path, args: &[&str]) -> Vec<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn git_first(root: &Path, args: &[&str]) -> Option<String> {
    git_lines(root, args).into_iter().next()
}

fn strip_credentials(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let (authority, tail) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    format!("{scheme}://{host}{tail}")
}

fn release_entry(root: &Path) -> Option<String> {
    if let Some(found) = RELEASE_ENTRIES.iter().find(|rel| root.join(rel).is_file()) {
        return Some((*found).to_string());
    }
    workflow_files(root).into_iter().find(|rel| {
        let name = rel.rsplit('/').next().unwrap_or_default().to_lowercase();
        name.contains("release") || name.contains("deploy")
    })
}

fn workflow_files(root: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(root.join(".github/workflows"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(|name| format!(".github/workflows/{name}")))
        .collect();
    files.sort();
    files
}

fn review_files(root: &Path) -> Vec<String> {
    let mut files: Vec<String> =
        REVIEW_FILES.iter().filter(|rel| root.join(rel).is_file()).map(|rel| (*rel).to_string()).collect();
    files.extend(workflow_files(root));
    files
}

pub(crate) fn gather_facts(root: &Path) -> Facts {
    let origin_head = git_first(root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .and_then(|name| name.strip_prefix("origin/").map(str::to_string));
    let branch_from_origin = origin_head.is_some();
    let branch = origin_head.or_else(|| git_first(root, &["branch", "--show-current"]));
    let merge_after_uat = crate::state::read_config_raw(root).get("uat_stop").and_then(Value::as_str) == Some("merge");
    Facts {
        project_id: root.file_name().and_then(|n| n.to_str()).map(str::to_string),
        branch,
        branch_from_origin,
        remote: git_first(root, &["remote", "get-url", "origin"]).map(|url| strip_credentials(&url)),
        merge_after_uat,
        release_entry: release_entry(root),
        vendored: VENDORED_TREES.iter().filter(|dir| root.join(dir).is_dir()).map(|dir| (*dir).to_string()).collect(),
    }
}

fn fact_rows(facts: &Facts) -> Vec<Fact> {
    let scalar = |key, value: &str, source: &str| Fact {
        key,
        value: ProtocolValue::Scalar(value.to_string()),
        source: source.to_string(),
    };
    let mut rows = Vec::new();
    if let Some(id) = &facts.project_id {
        rows.push(scalar("PROJECT_ID", id, "the control root directory name"));
    }
    if let Some(branch) = &facts.branch {
        let source =
            if facts.branch_from_origin { "git refs/remotes/origin/HEAD" } else { "git current branch" };
        rows.push(scalar("DEFAULT_BRANCH", branch, source));
    }
    if let Some(remote) = &facts.remote {
        rows.push(scalar("REPOSITORY_REMOTE", remote, "git origin fetch URL, credentials removed"));
    }
    let (merge_owner, merge_source) = if facts.merge_after_uat {
        (MERGE_AFTER_UAT, ".bee/config.json uat_stop is merge")
    } else {
        (MERGE_BY_AGENT, ".bee/config.json uat_stop is not merge")
    };
    rows.push(scalar("MERGE_OWNER", merge_owner, merge_source));
    rows.push(match &facts.release_entry {
        Some(entry) => scalar(
            "DEPLOY_OWNER",
            &format!("human; the agent runs {entry} only when the owner asks"),
            &format!("the release entry {entry}"),
        ),
        None => scalar("DEPLOY_OWNER", "human", "no release entry found"),
    });
    rows.push(Fact {
        key: BOUNDARIES_KEY,
        value: ProtocolValue::List(BASE_BOUNDARIES.iter().map(|b| (*b).to_string()).collect()),
        source: "bee's base set".to_string(),
    });
    let mut areas = vec![".bee/*.json and .bee/*.jsonl change only through the bee CLI".to_string()];
    areas.extend(facts.vendored.iter().map(|dir| format!("{dir} is vendored by bee; never hand-edit it")));
    let source = if facts.vendored.is_empty() {
        "bee's base set".to_string()
    } else {
        format!("bee's base set; the vendored trees {}", facts.vendored.join(", "))
    };
    rows.push(Fact { key: "PROTECTED_AREAS", value: ProtocolValue::List(areas), source });
    rows
}

pub(crate) fn render_protocol(facts: &Facts) -> String {
    let rows = fact_rows(facts);
    let title = facts.project_id.as_deref().map_or(String::new(), |id| format!(" — {id}"));
    let mut out = format!(
        "# Workspace protocol{title}\n\nThe project owner owns this file. The leader reads it in full before it\nroutes work. `bee protocol check` reports its state.\n\nTest commands are not here. They live in `.bee/config.json` `commands.test`,\nand `bee test` runs them.\n\n## Sources\n\n"
    );
    for row in &rows {
        out.push_str(&format!("- {}: {}\n", row.key, row.source));
    }
    out.push_str("\n```text\nWORKSPACE_PROTOCOL_VERSION: 1\nPROJECT_CRITICALITY: medium\n");
    for row in &rows {
        if matches!(row.key, "MERGE_OWNER" | BOUNDARIES_KEY | "PROTECTED_AREAS") {
            out.push('\n');
        }
        match &row.value {
            ProtocolValue::Scalar(value) => out.push_str(&format!("{}: {value}\n", row.key)),
            ProtocolValue::List(items) => {
                out.push_str(&format!("{}:\n", row.key));
                for item in items {
                    out.push_str(&format!("- {item}\n"));
                }
            }
        }
    }
    out.push_str("```\n");
    out
}

fn facts_json(facts: &Facts) -> Value {
    let rows: Vec<Value> = fact_rows(facts)
        .into_iter()
        .map(|row| {
            let value = match row.value {
                ProtocolValue::Scalar(s) => s,
                ProtocolValue::List(items) => items.join("; "),
            };
            json!({"key": row.key, "value": value, "source": row.source})
        })
        .collect();
    json!(rows)
}

fn next_step(review: &[String]) -> String {
    let read = if review.is_empty() {
        String::new()
    } else {
        format!("read {}, then ", review.join(", "))
    };
    format!(
        "next: {read}add the project's own boundaries, protected areas and anti-patterns, set PROJECT_CRITICALITY, never add a test command, then run `bee protocol check`."
    )
}

fn failure(kind: &str, reason: &str, path: &Path, fix: String) -> Value {
    json!({
        "ok": false,
        "type": kind,
        "reason": reason,
        "path": path.display().to_string(),
        "fix": fix,
    })
}

pub(crate) fn init_protocol(root: &Path) -> Result<Value, Value> {
    let path = root.join(PROTOCOL_FILE);
    let facts = gather_facts(root);
    let created = std::fs::OpenOptions::new().write(true).create_new(true).open(&path);
    let mut file = match created {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(failure(
                "refused",
                "protocol_exists",
                &path,
                format!("{PROTOCOL_FILE} already exists and was left untouched — edit it by hand and run `bee protocol check` to read its state"),
            ))
        }
        Err(e) => {
            return Err(failure("failed", "write_failed", &path, format!("could not create {PROTOCOL_FILE}: {e}")))
        }
    };
    if let Err(e) = file.write_all(render_protocol(&facts).as_bytes()) {
        return Err(failure("failed", "write_failed", &path, format!("could not write {PROTOCOL_FILE}: {e}")));
    }
    let review = review_files(root);
    Ok(json!({
        "ok": true,
        "path": path.display().to_string(),
        "state": protocol_state(root).state.as_str(),
        "facts": facts_json(&facts),
        "review": review,
        "next": next_step(&review),
    }))
}

pub(crate) fn print_protocol(root: &Path) -> Value {
    let facts = gather_facts(root);
    json!({
        "ok": true,
        "path": root.join(PROTOCOL_FILE).display().to_string(),
        "rendered": render_protocol(&facts),
        "facts": facts_json(&facts),
        "review": review_files(root),
    })
}

fn check_value(root: &Path) -> (Value, String) {
    let report = protocol_state(root);
    let path = root.join(PROTOCOL_FILE);
    let keys: Map<String, Value> =
        report.keys.iter().map(|(k, v)| (k.clone(), v.to_json())).collect();
    let state = report.state.as_str();
    let mut lines = vec![format!("workspace protocol: {state} ({})", path.display())];
    match report.state {
        ProtocolState::Missing => {
            lines.push("optional — `bee protocol init` writes a starter file".to_string())
        }
        _ => {
            for issue in &report.issues {
                lines.push(format!("issue: {issue}"));
            }
            if !report.keys.is_empty() {
                lines.push(format!("keys: {}", report.keys.keys().cloned().collect::<Vec<_>>().join(", ")));
            }
            if let Some(sha) = &report.sha256 {
                lines.push(format!("sha256: {sha}"));
            }
        }
    }
    let value = json!({
        "ok": true,
        "path": path.display().to_string(),
        "state": state,
        "issues": report.issues,
        "keys": keys,
        "sha256": report.sha256,
    });
    (value, lines.join("\n"))
}

fn init_value(root: &Path) -> (Value, String, u8) {
    match init_protocol(root) {
        Ok(value) => {
            let text = format!(
                "wrote {}\n{}",
                value["path"].as_str().unwrap_or(PROTOCOL_FILE),
                value["next"].as_str().unwrap_or_default()
            );
            (value, text, 0)
        }
        Err(value) => {
            let text = format!(
                "protocol init {}: {} — {}",
                value["type"].as_str().unwrap_or("refused"),
                value["path"].as_str().unwrap_or(PROTOCOL_FILE),
                value["fix"].as_str().unwrap_or_default()
            );
            (value, text, 1)
        }
    }
}

fn print_value(root: &Path) -> (Value, String, u8) {
    let value = print_protocol(root);
    let text = value["rendered"].as_str().unwrap_or_default().to_string();
    (value, text, 0)
}

pub(crate) fn run(sub: &str, flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    let (cmd, init): (&'static str, bool) = match sub {
        "check" => ("protocol check", false),
        "init" => ("protocol init", true),
        _ => return None,
    };
    if !keys_known(&flags, if init { &["print"] } else { &[] }) {
        return None;
    }
    let print = match flags.get("print") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" => true,
        Some(FlagV::S(s)) if s == "false" => false,
        Some(FlagV::S(_)) => return None,
    };
    let ctx = match ctx_serving_granted(cmd, use_json, t0)? {
        Ok(ctx) => ctx,
        Err(code) => return Some(code),
    };
    let (value, text, code) = if init && print {
        print_value(&ctx.root)
    } else if init {
        init_value(&ctx.root)
    } else {
        let (value, text) = check_value(&ctx.root);
        (value, text, 0)
    };
    finish(&ctx, Ok(Out::Emit(value, text, code)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASEO_PI_TEAM: &str = "# Workspace Protocol — example

Place this file in the root.

```text
WORKSPACE_PROTOCOL_VERSION: 1

PROJECT_ID: demo
MERGE_OWNER: human
DEPLOY_OWNER: external-system

REQUIRED_DOCUMENTS:
- AGENTS.md
  - docs/architecture.md

HUMAN_DECISION_BOUNDARIES:
- product behavior changes
- deployment

WRITER_WORKTREE: .worktrees/task   # inside the shared workspace
NOTE: free prose is a harmless key
```
";

    const PASEO_DOCTRINE: &str = "# Workspace Protocol — Paseo Product

<!-- PASEO_WORKSPACE_PROTOCOL_VERSION: 3 -->

- identity: `owner=Human; version=1`
- project risk/protected areas: daemon lifecycle
- ownership/hotspots: one write Owner per scope
";

    fn write(root: &Path, rel: &str, body: &str) {
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, body).unwrap();
    }

    fn scalar(s: &str) -> ProtocolValue {
        ProtocolValue::Scalar(s.to_string())
    }

    fn list(items: &[&str]) -> ProtocolValue {
        ProtocolValue::List(items.iter().map(|s| s.to_string()).collect())
    }

    fn valid_body(extra: &str) -> String {
        format!("MERGE_OWNER: human\nDEPLOY_OWNER: human\nHUMAN_DECISION_BOUNDARIES:\n- deployment\n{extra}")
    }

    #[test]
    fn parses_a_paseo_pi_team_fenced_file() {
        let keys = parse(PASEO_PI_TEAM);
        assert_eq!(keys["WORKSPACE_PROTOCOL_VERSION"], scalar("1"));
        assert_eq!(keys["MERGE_OWNER"], scalar("human"));
        assert_eq!(keys["REQUIRED_DOCUMENTS"], list(&["AGENTS.md", "docs/architecture.md"]));
        assert_eq!(
            keys["HUMAN_DECISION_BOUNDARIES"],
            list(&["product behavior changes", "deployment"])
        );
        assert_eq!(
            keys["WRITER_WORKTREE"],
            scalar(".worktrees/task   # inside the shared workspace")
        );
        assert_eq!(keys["NOTE"], scalar("free prose is a harmless key"));
    }

    #[test]
    fn a_dash_key_value_doctrine_file_reads_no_keys_and_gives_three_issues() {
        assert!(parse(PASEO_DOCTRINE).is_empty());
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), PROTOCOL_FILE, PASEO_DOCTRINE);
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Invalid);
        assert_eq!(report.issues.len(), 3, "{:?}", report.issues);
        assert!(report.issues[0].contains("MERGE_OWNER"));
        assert!(report.issues[1].contains("DEPLOY_OWNER"));
        assert!(report.issues[2].contains("HUMAN_DECISION_BOUNDARIES"));
    }

    #[test]
    fn a_missing_file_is_missing_with_no_digest() {
        let tmp = tempfile::tempdir().unwrap();
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Missing);
        assert!(report.issues.is_empty());
        assert_eq!(report.sha256, None);
        assert_eq!(protocol_sha256(tmp.path()), None);
    }

    #[test]
    fn non_utf8_bytes_are_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(PROTOCOL_FILE), [0x4d, 0xff, 0xfe, 0x3a]).unwrap();
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Unreadable);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.sha256, None);
        assert_eq!(protocol_sha256(tmp.path()), None);
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn git_project(origin: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        git(tmp.path(), &["init", "-q", "-b", "trunk"]);
        if let Some(url) = origin {
            git(tmp.path(), &["remote", "add", "origin", url]);
        }
        tmp
    }

    fn rendered(root: &Path) -> String {
        render_protocol(&gather_facts(root))
    }

    fn row_value(root: &Path, key: &str) -> Option<String> {
        facts_json(&gather_facts(root))
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["key"] == key)
            .map(|row| row["value"].as_str().unwrap().to_string())
    }

    fn fake_facts() -> Facts {
        Facts {
            project_id: Some("demo".to_string()),
            branch: Some("main".to_string()),
            branch_from_origin: true,
            remote: Some("https://example.com/o/r.git".to_string()),
            merge_after_uat: false,
            release_entry: Some("scripts/release.sh".to_string()),
            vendored: vec![".claude/skills".to_string()],
        }
    }

    #[test]
    fn init_writes_a_valid_file_and_names_the_judgment_step() {
        let tmp = tempfile::tempdir().unwrap();
        let (value, text, code) = init_value(tmp.path());
        assert_eq!(code, 0);
        assert_eq!(value["ok"], true);
        assert_eq!(value["state"], "valid");
        assert!(text.starts_with("wrote "), "{text}");
        assert_eq!(text.lines().count(), 2, "{text}");
        let next = text.lines().last().unwrap();
        assert!(next.contains("add the project's own boundaries, protected areas and anti-patterns"), "{next}");
        assert!(next.contains("set PROJECT_CRITICALITY"), "{next}");
        assert!(next.contains("never add a test command"), "{next}");
        assert!(next.contains("bee protocol check"), "{next}");
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Valid, "{:?}", report.issues);
        assert_eq!(report.sha256, protocol_sha256(tmp.path()));
        let (checked, _) = check_value(tmp.path());
        assert_eq!(checked["state"], "valid");
        assert_eq!(checked["keys"]["PROJECT_CRITICALITY"], "medium");
        assert_eq!(checked["keys"]["MERGE_OWNER"], MERGE_BY_AGENT);
        assert_eq!(checked["keys"]["DEPLOY_OWNER"], "human");
        assert_eq!(checked["keys"]["PROTECTED_AREAS"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn the_origin_url_loses_its_credentials_and_the_branch_comes_from_origin_head() {
        let tmp = git_project(Some("https://u:p@example.com/o/r.git"));
        git(tmp.path(), &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/develop"]);
        let text = rendered(tmp.path());
        let keys = parse(&text);
        assert_eq!(keys["REPOSITORY_REMOTE"], scalar("https://example.com/o/r.git"));
        assert_eq!(keys["DEFAULT_BRANCH"], scalar("develop"));
        assert!(!text.contains("u:p@"), "{text}");
        assert!(text.contains("- DEFAULT_BRANCH: git refs/remotes/origin/HEAD"), "{text}");
        let (value, _, _) = init_value(tmp.path());
        assert!(!value.to_string().contains("u:p@"), "{value}");
    }

    #[test]
    fn credentials_are_stripped_from_every_url_shape() {
        assert_eq!(strip_credentials("https://tok@example.com/o/r.git"), "https://example.com/o/r.git");
        assert_eq!(strip_credentials("https://u:p@example.com:8443/o/@r"), "https://example.com:8443/o/@r");
        assert_eq!(strip_credentials("https://example.com/o/r.git"), "https://example.com/o/r.git");
        assert_eq!(strip_credentials("git@github.com:o/r.git"), "git@github.com:o/r.git");
        assert_eq!(strip_credentials("ssh://git@example.com"), "ssh://example.com");
    }

    #[test]
    fn no_origin_leaves_the_remote_out_and_the_branch_is_the_current_one() {
        let tmp = git_project(None);
        let text = rendered(tmp.path());
        let keys = parse(&text);
        assert!(!keys.contains_key("REPOSITORY_REMOTE"), "{text}");
        assert_eq!(keys["DEFAULT_BRANCH"], scalar("trunk"));
        assert!(text.contains("- DEFAULT_BRANCH: git current branch"), "{text}");
        assert!(!text.contains("- REPOSITORY_REMOTE:"), "{text}");
        assert_eq!(protocol_state(tmp.path()).state, ProtocolState::Missing);
    }

    #[test]
    fn uat_stop_merge_and_absent_give_the_two_merge_owner_texts() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(row_value(tmp.path(), "MERGE_OWNER").unwrap(), MERGE_BY_AGENT);
        write(tmp.path(), ".bee/config.json", r#"{"uat_stop":"merge"}"#);
        assert_eq!(
            row_value(tmp.path(), "MERGE_OWNER").unwrap(),
            "human approves uat, then the agent runs bee worktree merge"
        );
        assert_eq!(parse(&rendered(tmp.path()))["MERGE_OWNER"], scalar(MERGE_AFTER_UAT));
        write(tmp.path(), ".bee/config.json", r#"{"uat_stop":"close"}"#);
        assert_eq!(row_value(tmp.path(), "MERGE_OWNER").unwrap(), MERGE_BY_AGENT);
    }

    #[test]
    fn deploy_owner_names_the_first_release_entry_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(row_value(tmp.path(), "DEPLOY_OWNER").unwrap(), "human");
        write(tmp.path(), ".github/workflows/ci.yml", "");
        assert_eq!(row_value(tmp.path(), "DEPLOY_OWNER").unwrap(), "human");
        write(tmp.path(), ".github/workflows/release.yml", "");
        assert_eq!(
            row_value(tmp.path(), "DEPLOY_OWNER").unwrap(),
            "human; the agent runs .github/workflows/release.yml only when the owner asks"
        );
        write(tmp.path(), "docs/DEPLOY.md", "");
        assert!(row_value(tmp.path(), "DEPLOY_OWNER").unwrap().contains("docs/DEPLOY.md"));
        write(tmp.path(), "scripts/deploy.sh", "");
        assert!(row_value(tmp.path(), "DEPLOY_OWNER").unwrap().contains("scripts/deploy.sh"));
        write(tmp.path(), "scripts/release.sh", "");
        assert_eq!(
            row_value(tmp.path(), "DEPLOY_OWNER").unwrap(),
            "human; the agent runs scripts/release.sh only when the owner asks"
        );
    }

    #[test]
    fn a_vendored_tree_adds_its_protected_area_item_only_when_it_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let bare = parse(&rendered(tmp.path()));
        assert_eq!(bare["PROTECTED_AREAS"], list(&[".bee/*.json and .bee/*.jsonl change only through the bee CLI"]));
        std::fs::create_dir_all(tmp.path().join(".claude/skills")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".codex-plugin/skills")).unwrap();
        let text = rendered(tmp.path());
        let keys = parse(&text);
        assert_eq!(
            keys["PROTECTED_AREAS"],
            list(&[
                ".bee/*.json and .bee/*.jsonl change only through the bee CLI",
                ".claude/skills is vendored by bee; never hand-edit it",
                ".codex-plugin/skills is vendored by bee; never hand-edit it",
            ])
        );
        assert!(!text.contains(".agents/skills"), "{text}");
        assert!(text.contains("the vendored trees .claude/skills, .codex-plugin/skills"), "{text}");
    }

    #[test]
    fn the_render_checks_valid_holds_no_test_command_and_sources_every_derived_key() {
        let tmp = git_project(Some("https://example.com/o/r.git"));
        write(tmp.path(), "scripts/release.sh", "");
        std::fs::create_dir_all(tmp.path().join(".agents/skills")).unwrap();
        write(tmp.path(), PROTOCOL_FILE, &rendered(tmp.path()));
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Valid, "{:?}", report.issues);
        assert!(!report.keys.contains_key("FULL_TEST"));
        for (key, value) in &report.keys {
            assert!(!value.holds_placeholder(), "{key}");
        }
        let text = std::fs::read_to_string(tmp.path().join(PROTOCOL_FILE)).unwrap();
        let sources = text.split("## Sources").nth(1).unwrap().split("```").next().unwrap();
        let rows = fact_rows(&gather_facts(tmp.path()));
        assert_eq!(rows.len(), 7);
        for row in &rows {
            assert!(sources.contains(&format!("- {}: {}\n", row.key, row.source)), "{}: {sources}", row.key);
            assert!(report.keys.contains_key(row.key), "{}", row.key);
        }
        assert!(sources.contains("- DEPLOY_OWNER: the release entry scripts/release.sh"), "{sources}");
    }

    #[test]
    fn render_protocol_renders_fake_facts_without_touching_disk() {
        let text = render_protocol(&fake_facts());
        assert!(text.starts_with("# Workspace protocol — demo\n"), "{text}");
        let keys = parse(&text);
        assert_eq!(keys["PROJECT_ID"], scalar("demo"));
        assert_eq!(keys["WORKSPACE_PROTOCOL_VERSION"], scalar("1"));
        assert_eq!(keys["HUMAN_DECISION_BOUNDARIES"], list(&BASE_BOUNDARIES));
        assert!(text.contains("- PROJECT_ID: the control root directory name\n"), "{text}");
        let bare = Facts {
            project_id: None,
            branch: None,
            branch_from_origin: false,
            remote: None,
            merge_after_uat: true,
            release_entry: None,
            vendored: Vec::new(),
        };
        let keys = parse(&render_protocol(&bare));
        assert!(!keys.contains_key("PROJECT_ID") && !keys.contains_key("DEFAULT_BRANCH"));
        assert_eq!(keys["MERGE_OWNER"], scalar(MERGE_AFTER_UAT));
    }

    #[test]
    fn review_lists_only_the_files_that_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let (value, text, _) = init_value(tmp.path());
        assert_eq!(value["review"], json!([]));
        assert!(text.contains("next: add the project's own"), "{text}");
        let other = tempfile::tempdir().unwrap();
        write(other.path(), "AGENTS.md", "");
        write(other.path(), "docs/DEPLOY.md", "");
        write(other.path(), ".github/workflows/b.yml", "");
        write(other.path(), ".github/workflows/a.yml", "");
        let (value, text, _) = init_value(other.path());
        assert_eq!(
            value["review"],
            json!(["AGENTS.md", "docs/DEPLOY.md", ".github/workflows/a.yml", ".github/workflows/b.yml"])
        );
        assert!(
            text.contains("next: read AGENTS.md, docs/DEPLOY.md, .github/workflows/a.yml, .github/workflows/b.yml, then add"),
            "{text}"
        );
        assert_eq!(value["facts"].as_array().unwrap().len(), 5, "{value}");
    }

    #[test]
    fn print_on_an_existing_file_returns_the_render_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), PROTOCOL_FILE, "MERGE_OWNER: me\n");
        write(tmp.path(), "CLAUDE.md", "");
        let (value, text, code) = print_value(tmp.path());
        assert_eq!(code, 0);
        assert_eq!(value["ok"], true);
        assert_eq!(value["rendered"].as_str().unwrap(), rendered(tmp.path()));
        assert_eq!(text, rendered(tmp.path()));
        assert_eq!(value["review"], json!(["CLAUDE.md"]));
        assert!(value["facts"].as_array().unwrap().iter().any(|row| row["key"] == "MERGE_OWNER"));
        assert_eq!(std::fs::read(tmp.path().join(PROTOCOL_FILE)).unwrap(), b"MERGE_OWNER: me\n");

        let fresh = tempfile::tempdir().unwrap();
        let (_, _, code) = print_value(fresh.path());
        assert_eq!(code, 0);
        assert!(!fresh.path().join(PROTOCOL_FILE).exists());
    }

    #[test]
    fn init_twice_refuses_and_keeps_the_first_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), PROTOCOL_FILE, "MERGE_OWNER: me\n");
        let (value, _, code) = init_value(tmp.path());
        assert_eq!(code, 1);
        assert_eq!(value["ok"], false);
        assert_eq!(value["type"], "refused");
        assert_eq!(value["reason"], "protocol_exists");
        assert!(value["path"].as_str().unwrap().ends_with(PROTOCOL_FILE));
        assert!(value["fix"].as_str().unwrap().contains("bee protocol check"));
        assert!(!tmp.path().join(".bee").exists());
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(PROTOCOL_FILE)).unwrap(),
            "MERGE_OWNER: me\n"
        );

        let fresh = tempfile::tempdir().unwrap();
        assert!(init_protocol(fresh.path()).is_ok());
        let first = std::fs::read(fresh.path().join(PROTOCOL_FILE)).unwrap();
        assert!(init_protocol(fresh.path()).is_err());
        assert_eq!(std::fs::read(fresh.path().join(PROTOCOL_FILE)).unwrap(), first);
    }

    #[test]
    fn full_test_equal_to_commands_test_is_fine() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"cargo test"}}"#);
        write(tmp.path(), PROTOCOL_FILE, &valid_body("FULL_TEST: cargo test\n"));
        assert_eq!(protocol_state(tmp.path()).state, ProtocolState::Valid);

        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":["cargo fmt --check","cargo test"]}}"#);
        write(tmp.path(), PROTOCOL_FILE, &valid_body("FULL_TEST: cargo fmt --check && cargo test\n"));
        assert_eq!(protocol_state(tmp.path()).state, ProtocolState::Valid);

        write(tmp.path(), PROTOCOL_FILE, &valid_body("FULL_TEST:\n"));
        assert_eq!(protocol_state(tmp.path()).state, ProtocolState::Valid);
    }

    #[test]
    fn full_test_that_differs_is_one_issue_naming_commands_test() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"cargo test"}}"#);
        write(tmp.path(), PROTOCOL_FILE, &valid_body("FULL_TEST: npm test\n"));
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Invalid);
        assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
        assert!(report.issues[0].contains("commands.test"), "{:?}", report.issues);
        assert!(report.issues[0].starts_with("FULL_TEST drifts"), "{:?}", report.issues);
    }

    #[test]
    fn a_duplicate_key_gives_an_issue_and_the_last_value() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), PROTOCOL_FILE, &valid_body("MERGE_OWNER: ci-bot\n"));
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Invalid);
        assert!(report.issues.iter().any(|i| i == "duplicate key MERGE_OWNER"), "{:?}", report.issues);
        assert_eq!(report.keys["MERGE_OWNER"], scalar("ci-bot"));
    }

    #[test]
    fn a_scalar_boundaries_value_names_the_list_form() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            PROTOCOL_FILE,
            "MERGE_OWNER: human\nDEPLOY_OWNER: human\nHUMAN_DECISION_BOUNDARIES: deployment\n",
        );
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Invalid);
        assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
        assert!(report.issues[0].contains("`- item` list form"), "{:?}", report.issues);
    }

    #[test]
    fn a_placeholder_value_is_an_issue() {
        let tmp = tempfile::tempdir().unwrap();
        for body in [
            valid_body("PROJECT_CRITICALITY: low | medium | high\n"),
            valid_body("WRITER_WORKTREE: .worktrees/<TASK_ID>\n"),
            "MERGE_OWNER: human | named-integration-owner\nDEPLOY_OWNER: human\nHUMAN_DECISION_BOUNDARIES:\n- deployment\n".to_string(),
            "MERGE_OWNER: human\nDEPLOY_OWNER: human\nHUMAN_DECISION_BOUNDARIES:\n- <other sources>\n".to_string(),
        ] {
            write(tmp.path(), PROTOCOL_FILE, &body);
            let report = protocol_state(tmp.path());
            assert_eq!(report.state, ProtocolState::Invalid, "{body}");
            assert!(
                report.issues.iter().any(|i| i.ends_with("still holds a template placeholder")),
                "{body}: {:?}",
                report.issues
            );
        }
    }

    #[test]
    fn a_copied_unfilled_paseo_template_is_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let body = "WORKSPACE_PROTOCOL_VERSION: 1\nPROJECT_CRITICALITY: low | medium | high\nMERGE_OWNER: human | named-integration-owner\nDEPLOY_OWNER: human | external-system\n\nREQUIRED_DOCUMENTS:\n- AGENTS.md\n- <other sources>\n\nHUMAN_DECISION_BOUNDARIES:\n- product behavior changes\n- deployment\n";
        write(tmp.path(), PROTOCOL_FILE, body);
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Invalid);
        assert_eq!(report.issues.len(), 4, "{:?}", report.issues);
        assert!(report.issues.iter().all(|i| i.ends_with("still holds a template placeholder")));
    }

    #[test]
    fn a_list_ends_at_a_blank_line() {
        let keys = parse("HUMAN_DECISION_BOUNDARIES:\n- a\n- b\n\n- c\nMERGE_OWNER: human\n");
        assert_eq!(keys["HUMAN_DECISION_BOUNDARIES"], list(&["a", "b"]));
        assert_eq!(keys["MERGE_OWNER"], scalar("human"));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn a_list_ends_at_the_next_key_line() {
        let keys = parse("A_LIST:\n  - one\n- two\nNEXT_KEY:\n- three\n");
        assert_eq!(keys["A_LIST"], list(&["one", "two"]));
        assert_eq!(keys["NEXT_KEY"], list(&["three"]));
    }

    #[test]
    fn check_reports_state_issues_keys_and_digest_in_json_and_text() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), PROTOCOL_FILE, "MERGE_OWNER: human\n");
        let (value, text) = check_value(tmp.path());
        assert_eq!(value["ok"], true);
        assert_eq!(value["state"], "invalid");
        assert_eq!(value["issues"].as_array().unwrap().len(), 2);
        assert_eq!(value["keys"]["MERGE_OWNER"], "human");
        assert_eq!(value["sha256"], json!(protocol_sha256(tmp.path())));
        assert!(text.contains("workspace protocol: invalid"), "{text}");
        assert!(text.contains("issue: DEPLOY_OWNER"), "{text}");
        assert!(text.contains("sha256: "), "{text}");

        let missing = tempfile::tempdir().unwrap();
        let (value, text) = check_value(missing.path());
        assert_eq!(value["state"], "missing");
        assert_eq!(value["sha256"], Value::Null);
        assert!(text.contains("bee protocol init"), "{text}");
    }

    #[test]
    fn run_serves_check_and_init_and_leaves_other_shapes_to_the_router() {
        let flags = || crate::verbs::reservations::parse_flags(&[]).unwrap().0;
        assert!(run("nope", flags(), false, Instant::now()).is_none());
        let with_unknown = crate::verbs::reservations::parse_flags(&["--wat=1"]).unwrap().0;
        assert!(run("check", with_unknown, false, Instant::now()).is_none());
    }
}
