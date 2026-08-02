// the frozen judge and the judge-verdict schema
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── frozen judge (lib/cells.mjs P12) + judge verdict schema (judge.mjs) ───

/// FROZEN_JUDGE_PATTERNS — hand matchers (regexes are anchored/segmented, so
/// each collapses to segment/suffix checks; all case-insensitive).
pub(crate) fn frozen_judge_rule(file: &str) -> Option<&'static str> {
    let lower = file.to_lowercase();
    let seg_starts: Vec<usize> = std::iter::once(0)
        .chain(lower.match_indices('/').map(|(i, _)| i + 1))
        .collect();
    let seg_prefix = |names: &[&str]| -> bool {
        seg_starts
            .iter()
            .any(|&s| names.iter().any(|n| lower[s..].starts_with(n)))
    };
    let last_seg = seg_starts.last().map(|&s| &lower[s..]).unwrap_or(&lower);
    // /(^|\/)(tests?|__tests__|specs?)\//i
    if seg_prefix(&["tests/", "test/", "__tests__/", "specs/", "spec/"]) {
        return Some("test sources");
    }
    // /\.(test|spec)\.[a-z]+$/i
    for marker in [".test.", ".spec."] {
        if let Some(pos) = lower.rfind(marker) {
            let ext = &lower[pos + marker.len()..];
            if !ext.is_empty() && ext.chars().all(|c| c.is_ascii_lowercase()) {
                return Some("test file");
            }
        }
    }
    // /(^|\/)__snapshots__\/|\.snap$/i
    if seg_prefix(&["__snapshots__/"]) || lower.ends_with(".snap") {
        return Some("snapshot");
    }
    // CI config
    if seg_prefix(&[".github/workflows/", ".circleci/"])
        || [".gitlab-ci.yml", "jenkinsfile", "azure-pipelines.yml"].contains(&last_seg)
    {
        return Some("CI config");
    }
    // lockfile
    if [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lock",
        "bun.lockb",
        "cargo.lock",
        "poetry.lock",
        "uv.lock",
        "go.sum",
        "composer.lock",
        "gemfile.lock",
    ]
    .contains(&last_seg)
    {
        return Some("lockfile");
    }
    // package manifest
    if ["package.json", "pyproject.toml", "cargo.toml", "go.mod", "composer.json", "gemfile"]
        .contains(&last_seg)
    {
        return Some("package manifest");
    }
    // test config: last segment starts with one of the prefixes
    if [
        "jest.config",
        "vitest.config",
        "playwright.config",
        "karma.conf",
        "pytest.ini",
        "tox.ini",
        "phpunit.xml",
    ]
    .iter()
    .any(|p| last_seg.starts_with(p))
    {
        return Some("test config");
    }
    // /(^|\/)\.bee\/config\.json$/i
    if lower == ".bee/config.json" || lower.ends_with("/.bee/config.json") {
        return Some("bee verify config");
    }
    None
}

/// lib/cells.mjs normalizePath (frozen-judge flavor: trim LAST).
pub(crate) fn frozen_normalize_path(p: &Value) -> String {
    let mut s = jsjson::js_to_string(p).replace('\\', "/");
    if let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string();
    }
    js_trim(&s).to_string()
}

/// declaredCovers glob: '*' within a segment, '**' across segments; exact
/// and dir-prefix matches first.
pub(crate) fn declared_covers(declared: &[Value], file: &str) -> bool {
    for raw in declared {
        let entry = frozen_normalize_path(raw);
        if entry.is_empty() {
            continue;
        }
        if entry == file {
            return true;
        }
        if entry.ends_with('/') && file.starts_with(&entry) {
            return true;
        }
        if entry.contains('*') && glob_covers(&entry, file) {
            return true;
        }
    }
    false
}

/// Wildcard match: `**` -> `.*`, `*` -> `[^/]*`, everything else literal.
pub(crate) fn glob_covers(pattern: &str, text: &str) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Tok {
        Any,     // '**'  (.*)
        Seg,     // '*'   ([^/]*)
        Lit(char),
    }
    let mut toks: Vec<Tok> = Vec::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '*' {
            if i + 1 < chars.len() && chars[i + 1] == '*' {
                toks.push(Tok::Any);
                i += 2;
            } else {
                toks.push(Tok::Seg);
                i += 1;
            }
        } else {
            toks.push(Tok::Lit(chars[i]));
            i += 1;
        }
    }
    let text: Vec<char> = text.chars().collect();
    // DP over (token, text position).
    let n = toks.len();
    let m = text.len();
    let mut dp = vec![vec![false; m + 1]; n + 1];
    dp[0][0] = true;
    for ti in 0..n {
        for pos in 0..=m {
            if !dp[ti][pos] {
                continue;
            }
            match toks[ti] {
                Tok::Lit(c) => {
                    if pos < m && text[pos] == c {
                        dp[ti + 1][pos + 1] = true;
                    }
                }
                Tok::Seg => {
                    let mut k = pos;
                    dp[ti + 1][k] = true;
                    while k < m && text[k] != '/' {
                        k += 1;
                        dp[ti + 1][k] = true;
                    }
                }
                Tok::Any => {
                    for k in pos..=m {
                        dp[ti + 1][k] = true;
                    }
                }
            }
        }
    }
    dp[n][m]
}

/// frozenJudgeHits — [{file, rule}] rows.
pub(crate) fn frozen_judge_hits(changed: &Value, declared: &Value) -> Vec<(String, &'static str)> {
    let declared_list: Vec<Value> = match declared {
        Value::Array(a) => a.clone(),
        _ => Vec::new(),
    };
    let changed_list: Vec<Value> = match changed {
        Value::Array(a) => a.clone(),
        _ => Vec::new(),
    };
    let mut hits = Vec::new();
    for raw in &changed_list {
        let file = frozen_normalize_path(raw);
        if file.is_empty() {
            continue;
        }
        let Some(rule) = frozen_judge_rule(&file) else { continue };
        if declared_covers(&declared_list, &file) {
            continue;
        }
        hits.push((file, rule));
    }
    hits
}

// ─── judge.mjs — verdict schema validation + model independence ────────────

pub(crate) const JUDGE_VERDICT_SCHEMA: &str = "judge-verdict/1";

pub(crate) const JUDGE_VERDICTS: [&str; 2] = ["PASS", "NEEDS_REVISION"];

pub(crate) const CHECK_STATUSES: [&str; 2] = ["PASS", "FAIL"];

pub(crate) const JUDGE_FIXABILITIES: [&str; 2] = ["automatic", "authority"];

pub(crate) const JUDGE_CONFIDENCES: [&str; 3] = ["low", "medium", "high"];

pub(crate) const PINNED_MODEL_STATUS: &str = "pinned"; // dispatch-guard.mjs

fn is_nonempty_string_value(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::String(s)) if !js_trim(s).is_empty())
}

/// validateJudgeVerdict -> (ok, errors). `verdict` may be any JSON value —
/// free prose (a string) is the non-object error.
pub(crate) fn validate_judge_verdict(obj: &Value) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    let map = match obj {
        Value::Object(m) => m,
        _ => {
            errors.push(
                "verdict must be a JSON object per schema \"judge-verdict/1\" (got free-form/non-object output) — a judge that returns free prose is a failed judge run, not a valid verdict."
                    .to_string(),
            );
            return (false, errors);
        }
    };
    if !matches!(map.get("schema"), Some(Value::String(s)) if s == JUDGE_VERDICT_SCHEMA) {
        errors.push(format!(
            "schema must be \"{JUDGE_VERDICT_SCHEMA}\", got {}.",
            js_json_or_undefined(map.get("schema"))
        ));
    }
    let verdict_ok =
        matches!(map.get("verdict"), Some(Value::String(s)) if JUDGE_VERDICTS.contains(&s.as_str()));
    if !verdict_ok {
        errors.push(format!(
            "verdict must be one of {}, got {}.",
            JUDGE_VERDICTS.join("|"),
            js_json_or_undefined(map.get("verdict"))
        ));
    }
    let mut any_fail = false;
    match map.get("checks") {
        Some(Value::Array(checks)) if !checks.is_empty() => {
            for (i, entry) in checks.iter().enumerate() {
                let entry_map = match entry {
                    Value::Object(m) => m,
                    _ => {
                        errors.push(format!("checks[{i}] must be a JSON object."));
                        continue;
                    }
                };
                if !is_nonempty_string_value(entry_map.get("id")) {
                    errors.push(format!("checks[{i}].id must be a non-empty string."));
                }
                match entry_map.get("status") {
                    Some(Value::String(s)) if CHECK_STATUSES.contains(&s.as_str()) => {
                        if s == "FAIL" {
                            any_fail = true;
                        }
                    }
                    other => errors.push(format!(
                        "checks[{i}].status must be one of {}, got {}.",
                        CHECK_STATUSES.join("|"),
                        js_json_or_undefined(other)
                    )),
                }
                if !is_nonempty_string_value(entry_map.get("evidence")) {
                    errors.push(format!("checks[{i}].evidence must be a non-empty string."));
                }
            }
            let verdict_is = |name: &str| matches!(map.get("verdict"), Some(Value::String(s)) if s == name);
            if verdict_is("PASS") && any_fail {
                errors.push(
                    "verdict must not be PASS when any check has status FAIL — a PASS verdict must not carry a FAIL check."
                        .to_string(),
                );
            }
            if verdict_is("NEEDS_REVISION") && !any_fail {
                errors.push(
                    "verdict NEEDS_REVISION requires at least one check with status FAIL — got no FAIL check among the checks."
                        .to_string(),
                );
            }
        }
        _ => errors.push("checks must be a non-empty array.".to_string()),
    }
    if !matches!(map.get("fixability"), Some(Value::String(s)) if JUDGE_FIXABILITIES.contains(&s.as_str())) {
        errors.push(format!(
            "fixability must be one of {}, got {}.",
            JUDGE_FIXABILITIES.join("|"),
            js_json_or_undefined(map.get("fixability"))
        ));
    }
    if !matches!(map.get("confidence"), Some(Value::String(s)) if JUDGE_CONFIDENCES.contains(&s.as_str())) {
        errors.push(format!(
            "confidence must be one of {}, got {}.",
            JUDGE_CONFIDENCES.join("|"),
            js_json_or_undefined(map.get("confidence"))
        ));
    }
    let fs = map.get("failure_signature");
    if any_fail && !is_nonempty_string_value(fs) {
        errors.push(
            "failure_signature is required (non-empty string) when any check has status FAIL.".to_string(),
        );
    } else if let Some(v) = fs {
        if !matches!(v, Value::Null) && !is_nonempty_string_value(Some(v)) {
            errors.push("failure_signature, when present, must be a non-empty string.".to_string());
        }
    }
    (errors.is_empty(), errors)
}

/// judge.mjs deriveModelIndependence.
pub(crate) fn derive_model_independence(
    builder_model: Option<&str>,
    builder_status: Option<&str>,
    judge_model: Option<&str>,
    judge_status: Option<&str>,
) -> &'static str {
    let both_pinned =
        builder_status == Some(PINNED_MODEL_STATUS) && judge_status == Some(PINNED_MODEL_STATUS);
    let named = |m: Option<&str>| m.map(|s| !js_trim(s).is_empty()).unwrap_or(false);
    if !both_pinned || !named(builder_model) || !named(judge_model) {
        return "unverified";
    }
    if builder_model == judge_model {
        "same-model"
    } else {
        "confirmed"
    }
}
