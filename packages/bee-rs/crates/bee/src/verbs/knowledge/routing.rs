// routing and the check/index/list/context verbs
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "knowledge" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    match verb {
        "check" => run_check(flags, json, pre_json, t0),
        "index" => run_index(flags, json, pre_json, t0),
        "list" => run_list(flags, json, pre_json, t0),
        "context" => run_context(flags, json, pre_json, t0),
        "search" => run_search(flags, json, pre_json, t0),
        "promote" => run_promote(flags, json, pre_json, t0),
        "bootstrap" => run_bootstrap(flags, json, pre_json, t0),
        "report" => run_report(flags, json, pre_json, t0),
        _ => None, // unknown verbs (group-usage fallback) → Node
    }
}

pub(crate) fn run_check(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["strict"]) {
        return None;
    }
    let strict = js_bool_flag(&flags, "strict")?;
    let ctx = match g_prelude("knowledge check", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let report = check_bundle(&dir, strict)?;
    let failing = !report.ok;

    let mut lines: Vec<String> = Vec::new();
    let line_of = |f: &Value, tag: &str| {
        format!(
            "{tag} [{}] {}: {}",
            f.get("code").and_then(Value::as_str).unwrap_or(""),
            f.get("file").and_then(Value::as_str).unwrap_or(""),
            f.get("message").and_then(Value::as_str).unwrap_or("")
        )
    };
    for f in &report.okf_errors {
        lines.push(line_of(f, "ERROR"));
    }
    for f in &report.profile_errors {
        lines.push(line_of(f, "ERROR"));
    }
    for f in &report.warnings {
        lines.push(line_of(f, if strict { "ERROR(strict)" } else { "WARN" }));
    }
    for n in &report.notes {
        lines.push(format!("NOTE: {n}"));
    }
    lines.push(format!(
        "knowledge check: {} concept(s) in {} file(s), {} OKF error(s), {} profile error(s), {} profile warning(s){} — {}",
        report.concepts,
        report.files,
        report.okf_errors.len(),
        report.profile_errors.len(),
        report.warnings.len(),
        if strict { " [--strict]" } else { "" },
        if failing { "FAIL" } else { "OK" }
    ));

    let mut counts = Map::new();
    counts.insert("files".into(), Value::from(report.files));
    counts.insert("concepts".into(), Value::from(report.concepts));
    counts.insert("errors".into(), Value::from(report.okf_errors.len()));
    counts.insert("profile_errors".into(), Value::from(report.profile_errors.len()));
    counts.insert("warnings".into(), Value::from(report.warnings.len()));
    let mut okf = Map::new();
    okf.insert("errors".into(), Value::Array(report.okf_errors));
    let mut profile = Map::new();
    profile.insert("errors".into(), Value::Array(report.profile_errors));
    profile.insert("warnings".into(), Value::Array(report.warnings));
    let mut result = Map::new();
    result.insert("okf".into(), Value::Object(okf));
    result.insert("profile".into(), Value::Object(profile));
    result.insert("counts".into(), Value::Object(counts));
    if !report.notes.is_empty() {
        result.insert("notes".into(), Value::Array(report.notes.iter().map(|s| Value::String(s.clone())).collect()));
    }

    Some(ctx.emit(&Value::Object(result), &lines.join("\n"), u8::from(failing)))
}

pub(crate) fn run_index(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["check"]) {
        return None;
    }
    let check = js_bool_flag(&flags, "check")?;
    let ctx = match g_prelude("knowledge index", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let expected = compute_index_files(&dir)?;

    if check {
        let mut stale: Vec<String> = Vec::new();
        for (rel, content) in &expected {
            let on_disk = read_file_lossy(&join_rel(&dir, rel)).ok();
            if on_disk.as_deref() != Some(content.as_str()) {
                stale.push(format!("docs/knowledge/{rel}"));
            }
        }
        let drift = !stale.is_empty();
        let mut lines: Vec<String> = stale.iter().map(|f| format!("STALE {f}")).collect();
        lines.push(format!(
            "knowledge index --check: {} expected index file(s), {} stale — {}",
            expected.len(),
            stale.len(),
            if drift { "FAIL (regenerate: bee knowledge index)" } else { "OK" }
        ));
        let mut result = Map::new();
        result.insert("checked".into(), Value::from(expected.len()));
        result.insert("stale".into(), Value::Array(stale.into_iter().map(Value::String).collect()));
        result.insert("drift".into(), Value::Bool(drift));
        return Some(ctx.emit(&Value::Object(result), &lines.join("\n"), u8::from(drift)));
    }

    let mut written: Vec<String> = Vec::new();
    for (rel, content) in &expected {
        let abs = join_rel(&dir, rel);
        let write = abs
            .parent()
            .map(std::fs::create_dir_all)
            .unwrap_or(Ok(()))
            .and_then(|()| std::fs::write(&abs, content));
        if let Err(e) = write {
            // DIVERGENCE (header note): partial writes forbid delegation, so
            // the Rust io message stands in for Node's V8-worded one.
            return Some(ctx.fail(&e.to_string()));
        }
        written.push(format!("docs/knowledge/{rel}"));
    }
    let count = written.len();
    let text = format!("Rendered {count} generated index file(s) under docs/knowledge/.");
    let mut result = Map::new();
    result.insert("written".into(), Value::Array(written.into_iter().map(Value::String).collect()));
    result.insert("count".into(), Value::from(count));
    Some(ctx.emit(&Value::Object(result), &text, 0))
}

pub(crate) fn run_list(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["type", "lifecycle", "area"]) {
        return None;
    }
    // handler: `typeof flags.x === 'string' ? flags.x : null` — bare booleans
    // cannot occur (none of these are FLAG_ALONE_BOOLEANS), so every present
    // flag is a string filter, empty strings included.
    let filter = |name: &str| -> Option<String> {
        match flags.get(name) {
            Some(FlagV::S(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let f_type = filter("type");
    let f_lifecycle = filter("lifecycle");
    let f_area = filter("area");

    let ctx = match g_prelude("knowledge list", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let concepts = collect_concepts(&dir)?;

    let mut rows: Vec<Value> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for concept in &concepts {
        let bee = bee_of(&concept.data);
        let id = str_field(&bee, "id");
        let c_type = str_field(&concept.data, "type");
        let lifecycle = str_field(&bee, "lifecycle");
        let title = str_field(&concept.data, "title");
        if let Some(t) = &f_type {
            if c_type != Some(t.as_str()) {
                continue;
            }
        }
        if let Some(l) = &f_lifecycle {
            if lifecycle != Some(l.as_str()) {
                continue;
            }
        }
        if let Some(a) = &f_area {
            let areas = bee.get("areas");
            let member = matches!(areas, Some(Value::Array(items)) if items.iter().any(|v| matches!(v, Value::String(s) if s == a)));
            if !member {
                continue;
            }
        }
        lines.push(format!(
            "{} · {} · {} · {} · {}",
            concept.path,
            id.unwrap_or("-"),
            c_type.unwrap_or("-"),
            lifecycle.unwrap_or("-"),
            title.unwrap_or("-")
        ));
        let mut row = Map::new();
        row.insert("path".into(), Value::String(concept.path.clone()));
        let opt = |v: Option<&str>| v.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null);
        row.insert("id".into(), opt(id));
        row.insert("type".into(), opt(c_type));
        row.insert("lifecycle".into(), opt(lifecycle));
        row.insert("title".into(), opt(title));
        rows.push(Value::Object(row));
    }
    lines.push(format!("{} concept(s).", rows.len()));

    let mut result = Map::new();
    let count = rows.len();
    result.insert("concepts".into(), Value::Array(rows));
    result.insert("count".into(), Value::from(count));
    Some(ctx.emit(&Value::Object(result), &lines.join("\n"), 0))
}

/// i54-closeout D3 lane presets (KNOWLEDGE_CONTEXT_LANE_BUDGETS).
pub(crate) fn lane_budget(lane: &str) -> Option<f64> {
    match lane {
        "tiny" => Some(8000.0),
        "small" => Some(12000.0),
        "standard" => Some(20000.0),
        "high-risk" => Some(30000.0),
        _ => None,
    }
}

/// JS Number(<string>) over the plain decimal/scientific grammar; None =>
/// delegate (hex/binary/Infinity/other legacy shapes Node must answer).
pub(crate) fn js_number_conv(raw: &str) -> Option<f64> {
    let t = js_trim(raw);
    if t.is_empty() {
        return Some(0.0);
    }
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return None;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return None;
        }
    }
    if i != bytes.len() {
        return None;
    }
    t.parse::<f64>().ok().filter(|f| f.is_finite())
}

pub(crate) fn run_context(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["work", "budget", "lane"]) {
        return None;
    }
    // validate(): work required (present, non-'').
    let work = match flags.get("work") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    // resolveKnowledgeContextLaneBudget: an explicit non-empty --budget wins;
    // otherwise a recognized --lane fills it; otherwise validate refuses
    // (required, missing) — Node's own message, delegate.
    let (budget, budget_raw): (f64, Value) = match flags.get("budget") {
        Some(FlagV::S(s)) if !s.is_empty() => {
            if js_trim(s).is_empty() {
                return None; // validate: invalid type (whitespace-only)
            }
            (js_number_conv(s)?, Value::String(s.clone()))
        }
        _ => match flags.get("lane") {
            Some(FlagV::S(l)) if !l.is_empty() => {
                let preset = lane_budget(l)?;
                (preset, num(preset))
            }
            _ => return None,
        },
    };

    let ctx = match g_prelude("knowledge context", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let manifest = match build_context_manifest(&dir, &work, budget, &budget_raw) {
        ManifestOut::Built(m) => m,
        ManifestOut::Thrown(msg) => return Some(ctx.fail(&msg)),
        ManifestOut::NeedsNode => return None,
    };
    if !crate::verbs::feedback::value_js_safe(&manifest) {
        return None;
    }

    let g = |k: &str| manifest.get(k).cloned().unwrap_or(Value::Null);
    let arr = |k: &str| match manifest.get(k) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let mut lines = vec![format!(
        "work: {} · budget: {} token(s) · estimator: {}",
        jsjson::js_to_string(&g("work")),
        jsjson::js_to_string(&g("budget")),
        jsjson::js_to_string(&g("estimator"))
    )];
    let decisions = arr("decisions");
    if !decisions.is_empty() {
        lines.push(format!(
            "decisions: {}",
            decisions.iter().map(jsjson::js_to_string).collect::<Vec<_>>().join(" · ")
        ));
    }
    lines.push("PATH · BYTES · EST TOKENS · REASON".to_string());
    for entry in arr("entries") {
        lines.push(format!(
            "{} · {} · {} · {}",
            js_str_or_undefined(entry.get("path")),
            js_str_or_undefined(entry.get("bytes")),
            js_str_or_undefined(entry.get("est_tokens")),
            js_str_or_undefined(entry.get("reason"))
        ));
    }
    for cut in arr("truncated") {
        lines.push(format!("TRUNCATED {}", jsjson::js_to_string(&cut)));
    }
    for dropped in arr("excluded") {
        lines.push(format!(
            "EXCLUDED {} · {} · {}",
            js_str_or_undefined(dropped.get("path")),
            js_str_or_undefined(dropped.get("score")),
            js_str_or_undefined(dropped.get("reason"))
        ));
    }
    lines.push(format!(
        "knowledge context: {} entry(ies), {} est token(s) of {} budget (estimator {}), {} truncated, {} excluded of {} critical pattern(s); zero_signal_count {}; floor {}.",
        arr("entries").len(),
        jsjson::js_to_string(&g("total_est")),
        jsjson::js_to_string(&g("budget")),
        jsjson::js_to_string(&g("estimator")),
        arr("truncated").len(),
        arr("excluded").len(),
        jsjson::js_to_string(&g("critical_total")),
        jsjson::js_to_string(&g("zero_signal_count")),
        arr("floor").len()
    ));

    Some(ctx.emit(&manifest, &lines.join("\n"), 0))
}
