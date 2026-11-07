use crate::fsutil::append_jsonl;
use crate::verbs::decisions::read_jsonl;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

pub(crate) const KINDS: [&str; 16] = [
    "gate-bypass",
    "close",
    "merge",
    "release",
    "scribing",
    "promote",
    "handoff",
    "judge-deferral",
    "uat-deferral",
    "capture-deferral",
    "knowledge-freshness-deferral",
    "impact-deferral",
    "routing-deferral",
    "doc-deferral",
    "leader-check-deferral",
    "dissent-deferral",
];

pub(crate) fn audit_path(root: &Path) -> PathBuf {
    root.join(".bee").join("audit.jsonl")
}

pub(crate) fn is_audit_only_tag(tag: &str) -> bool {
    let tag = tag.to_lowercase();
    tag == "gate-bypass" || (tag.ends_with("-deferral") && KINDS.contains(&tag.as_str()))
}

pub(crate) fn first_audit_only_tag(tags: &[String]) -> Option<&str> {
    tags.iter().map(String::as_str).find(|t| is_audit_only_tag(t))
}

pub(crate) fn unknown_kind_message(verb: &str, kind: &str) -> String {
    format!("{verb}: unknown kind \"{kind}\" — must be one of: {}.", KINDS.join(", "))
}

pub(crate) fn append(
    root: &Path,
    kind: &str,
    feature: &str,
    cell: Option<&str>,
    note: &str,
    source: &str,
    bypass_level: Option<&str>,
) -> std::io::Result<Value> {
    let mut row = Map::new();
    row.insert("id".into(), json!(crate::verbs::reservations::pseudo_uuid_v4()));
    row.insert("ts".into(), json!(crate::verbs::reservations::now_iso()));
    row.insert("kind".into(), json!(kind));
    row.insert("feature".into(), json!(feature));
    if let Some(cell) = cell {
        row.insert("cell".into(), json!(cell));
    }
    row.insert("note".into(), json!(note));
    row.insert("source".into(), json!(source));
    if let Some(level) = bypass_level {
        row.insert("bypass_level".into(), json!(level));
    }
    let row = Value::Object(row);
    append_jsonl(&audit_path(root), &row)?;
    Ok(row)
}

pub(crate) fn rows(root: &Path, kind: Option<&str>, feature: Option<&str>) -> Vec<Value> {
    read_jsonl(&audit_path(root))
        .into_iter()
        .filter(|r| kind.map_or(true, |k| r.get("kind").and_then(Value::as_str) == Some(k)))
        .filter(|r| feature.map_or(true, |f| r.get("feature").and_then(Value::as_str) == Some(f)))
        .collect()
}

pub(crate) fn has_entry(root: &Path, kind: &str, feature: &str) -> bool {
    !rows(root, Some(kind), Some(feature)).is_empty()
}

pub(crate) fn newest_note(root: &Path, kind: &str, feature: &str) -> Option<String> {
    rows(root, Some(kind), Some(feature))
        .last()
        .and_then(|r| r.get("note").and_then(Value::as_str).map(str::to_string))
}

pub(crate) fn hide_audit_tagged(decisions: Vec<Value>, tag: Option<&str>) -> Vec<Value> {
    if tag.is_some_and(is_audit_only_tag) {
        return decisions;
    }
    decisions
        .into_iter()
        .filter(|d| {
            !matches!(d.get("tags"), Some(Value::Array(tags))
                if tags.iter().any(|t| t.as_str().is_some_and(is_audit_only_tag)))
        })
        .collect()
}
