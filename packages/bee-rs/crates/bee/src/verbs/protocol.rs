use crate::verbs::drivers::{ctx_serving_granted, declared_test_commands};
use crate::verbs::reservations::{finish, keys_known, sha256_hex, Flags, Out};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const PROTOCOL_FILE: &str = "WORKSPACE_PROTOCOL.md";

const BOUNDARIES_KEY: &str = "HUMAN_DECISION_BOUNDARIES";

const NEXT_STEP: &str = "next: review and edit WORKSPACE_PROTOCOL.md — the defaults are bee's guess, not the owner's choice.";

pub(crate) const TEMPLATE: &str = "# Workspace protocol

WORKSPACE_PROTOCOL_VERSION: 1
PROJECT_CRITICALITY: medium
MERGE_OWNER: human
DEPLOY_OWNER: human

HUMAN_DECISION_BOUNDARIES:
- product behavior changes
- public API break
- irreversible schema migration
- secret or infrastructure mutation
- deployment
- cost above budget

PROTECTED_AREAS:
- none
";

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
    let created = std::fs::OpenOptions::new().write(true).create_new(true).open(&path);
    let mut file = match created {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(failure(
                "refused",
                "protocol_exists",
                &path,
                format!("{PROTOCOL_FILE} already exists and was left untouched — edit it by hand, and run `bee protocol check` to read its state"),
            ))
        }
        Err(e) => {
            return Err(failure("failed", "write_failed", &path, format!("could not create {PROTOCOL_FILE}: {e}")))
        }
    };
    if let Err(e) = file.write_all(TEMPLATE.as_bytes()) {
        return Err(failure("failed", "write_failed", &path, format!("could not write {PROTOCOL_FILE}: {e}")));
    }
    Ok(json!({
        "ok": true,
        "path": path.display().to_string(),
        "state": protocol_state(root).state.as_str(),
        "next": NEXT_STEP,
    }))
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
            let text = format!("wrote {}\n{NEXT_STEP}", value["path"].as_str().unwrap_or(PROTOCOL_FILE));
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

pub(crate) fn run(sub: &str, flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    let (cmd, init): (&'static str, bool) = match sub {
        "check" => ("protocol check", false),
        "init" => ("protocol init", true),
        _ => return None,
    };
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match ctx_serving_granted(cmd, use_json, t0)? {
        Ok(ctx) => ctx,
        Err(code) => return Some(code),
    };
    let (value, text, code) = if init {
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

    #[test]
    fn init_then_check_is_valid_and_prints_the_next_step() {
        let tmp = tempfile::tempdir().unwrap();
        let (value, text, code) = init_value(tmp.path());
        assert_eq!(code, 0);
        assert_eq!(value["ok"], true);
        assert_eq!(value["state"], "valid");
        assert!(text.contains("review and edit WORKSPACE_PROTOCOL.md"), "{text}");
        assert!(text.contains("bee's guess, not the owner's choice"), "{text}");
        let report = protocol_state(tmp.path());
        assert_eq!(report.state, ProtocolState::Valid, "{:?}", report.issues);
        assert!(report.sha256.is_some());
        assert_eq!(report.sha256, protocol_sha256(tmp.path()));
        let (checked, _) = check_value(tmp.path());
        assert_eq!(checked["state"], "valid");
        assert_eq!(checked["keys"]["MERGE_OWNER"], "human");
        assert_eq!(checked["keys"]["PROTECTED_AREAS"], json!(["none"]));
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
    fn the_template_holds_no_placeholder() {
        for (key, value) in parse(TEMPLATE) {
            assert!(!value.holds_placeholder(), "{key}");
        }
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
