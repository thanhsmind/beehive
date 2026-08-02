// recovery, runtime drift and source identity
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw, Bail};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

// ─── recovery (recovery.mjs) ───────────────────────────────────────────────

/// perf.mjs resolveTranscript.
pub(crate) fn resolve_transcript(
    projects_root: Option<&str>,
    project_path: Option<&str>,
    session_id: Option<&str>,
    transcript_path: Option<&str>,
) -> Option<String> {
    if let Some(tp) = transcript_path {
        let t = js_trim(tp);
        if !t.is_empty() && Path::new(t).exists() {
            return Some(t.to_string());
        }
    }
    let (Some(projects_root), Some(project_path)) = (projects_root, project_path) else {
        return None;
    };
    let dir = PathBuf::from(normalize_abs_lexical(&format!(
        "{}{}{}",
        projects_root,
        std::path::MAIN_SEPARATOR,
        encode_project_dir(project_path)
    )));
    if let Some(sid) = session_id {
        let file = dir.join(format!("{sid}.jsonl"));
        if file.exists() {
            return Some(file.to_string_lossy().into_owned());
        }
        return None;
    }
    None // the newest-mtime branch is never reached from status/orient
}

/// recovery.mjs hasCleanEndTrio.
pub(crate) fn has_clean_end_trio(events: &[Value]) -> bool {
    if events.is_empty() {
        return false;
    }
    let is_conversational =
        |e: &Value| str_eq(vget(e, "type"), "user") || str_eq(vget(e, "type"), "assistant");
    let mut stop_idx: Option<usize> = None;
    for i in (0..events.len()).rev() {
        let e = &events[i];
        if truthy(e)
            && str_eq(vget(e, "type"), "system")
            && str_eq(vget(e, "subtype"), "stop_hook_summary")
        {
            stop_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(stop_idx) = stop_idx else { return false };
    let mut turn_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(stop_idx + 1) {
        if truthy(e)
            && str_eq(vget(e, "type"), "system")
            && str_eq(vget(e, "subtype"), "turn_duration")
        {
            turn_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(turn_idx) = turn_idx else { return false };
    let mut last_prompt_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(turn_idx + 1) {
        if truthy(e) && str_eq(vget(e, "type"), "last-prompt") {
            last_prompt_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(last_prompt_idx) = last_prompt_idx else { return false };
    for e in events.iter().skip(last_prompt_idx + 1) {
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    true
}

/// recovery.mjs eventTimestampMs.
pub(crate) fn event_timestamp_ms(event: &Value) -> f64 {
    if !matches!(event, Value::Object(_)) {
        return f64::NAN;
    }
    if let Some(Value::String(ts)) = vget(event, "timestamp") {
        return js_date_parse(ts);
    }
    if let Some(Value::String(at)) = vget(event, "at") {
        return js_date_parse(at);
    }
    f64::NAN
}

/// recovery.mjs toMs.
pub(crate) fn to_ms(v: Option<&Value>) -> f64 {
    match v {
        None | Some(Value::Null) => f64::NAN,
        Some(Value::String(s)) => js_date_parse(s),
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::Bool(b)) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Some(_) => f64::NAN,
    }
}

pub(crate) struct TranscriptRoot {
    pub(crate) runtime: String,
    pub(crate) path: String,
    pub(crate) scanned: bool,
    pub(crate) reason: Option<String>,
}

/// recovery.mjs scanTranscriptRoots — the Claude default root plus every
/// configured recovery.transcript_roots entry, each probed fresh.
pub(crate) fn scan_transcript_roots(ctx: &mut Ctx, projects_root: &str) -> R<Vec<TranscriptRoot>> {
    let config = read_config(ctx)?;
    let configured_raw = config
        .raw
        .get("recovery")
        .filter(|v| truthy(v))
        .and_then(|r| vget(r, "transcript_roots"))
        .cloned();
    let mut entries: Vec<(String, String, bool)> =
        vec![("claude".into(), projects_root.to_string(), false)];
    if let Some(Value::Array(items)) = configured_raw {
        for entry in items {
            let Value::Object(o) = &entry else { continue };
            let runtime = o.get("runtime").and_then(|v| v.as_str()).map(js_trim).unwrap_or("");
            let root_path = o.get("path").and_then(|v| v.as_str()).map(js_trim).unwrap_or("");
            if runtime.is_empty() || root_path.is_empty() {
                continue;
            }
            entries.push((runtime.to_string(), root_path.to_string(), true));
        }
    }
    let mut out = Vec::new();
    for (runtime, root_path, is_configured) in entries {
        let mut scanned = false;
        let mut reason: Option<String> = None;
        match std::fs::metadata(&root_path) {
            Ok(meta) => {
                scanned = meta.is_dir();
                if !scanned {
                    reason = Some("not-a-directory".into());
                }
            }
            Err(err) => {
                reason = Some(match err.kind() {
                    std::io::ErrorKind::NotFound => "ENOENT".into(),
                    std::io::ErrorKind::PermissionDenied => "EACCES".into(),
                    std::io::ErrorKind::NotADirectory => "ENOTDIR".into(),
                    _ => "unreadable".into(),
                });
            }
        }
        if !scanned && is_configured {
            ctx.warn(format!(
                "recovery: configured transcript root \"{root_path}\" (runtime \"{runtime}\") is {} — skipping (config: recovery.transcript_roots)",
                reason.as_deref().unwrap_or("unreadable")
            ));
        }
        out.push(TranscriptRoot { runtime, path: root_path, scanned, reason });
    }
    Ok(out)
}

/// recovery.mjs lastDurableSettlement with cp-1 injected shared inputs.
pub(crate) fn last_durable_settlement(
    lane: Option<&Value>,
    decisions: &[Value],
    capture_events: &[Value],
    cells: &[Value],
) -> Option<f64> {
    let mut max_ms: Option<f64> = None;
    let mut bump = |ms: f64| {
        if ms.is_finite() && max_ms.map(|m| ms > m).unwrap_or(true) {
            max_ms = Some(ms);
        }
    };
    for event in decisions {
        bump(date_parse_val(if truthy(event) { vget(event, "date") } else { None }));
    }
    let lane_truthy = lane.map(truthy).unwrap_or(false);
    for event in capture_events {
        if !truthy(event) || !str_eq(vget(event, "kind"), "stub") {
            continue;
        }
        if lane_truthy && !strict_eq(vget(event, "lane"), lane) {
            continue;
        }
        bump(date_parse_val(vget(event, "at")));
    }
    for cell in cells {
        if lane_truthy && !strict_eq(vget(cell, "feature"), lane) {
            continue;
        }
        let capped_at = vget(cell, "trace").and_then(|t| vget(t, "capped_at"));
        if opt_truthy(capped_at) {
            bump(date_parse_val(capped_at));
        }
    }
    max_ms
}

/// recovery.mjs sessionHasActiveClaim (control-root claims).
pub(crate) fn session_has_active_claim(ctx: &Ctx, control_root: &Path, session_id: &Value, now: f64) -> R<bool> {
    let Ok(entries) = std::fs::read_dir(claims_dir(control_root)) else {
        return Ok(false);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        let claim = match read_claim(ctx, control_root, stem) {
            Ok(c) => c,
            // readClaim's requireId throw propagates in Node (no local catch
            // here) — buildRecoveryBlock's own catch absorbs it.
            Err(e) => return Err(e),
        };
        let Some(claim) = claim else { continue };
        if strict_eq(claim.get("session"), Some(session_id)) && is_claim_active(&claim, now) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// recovery.mjs detectCrashCandidates.
pub(crate) fn detect_crash_candidates(ctx: &mut Ctx, projects_root: &str) -> R<Vec<Value>> {
    // resolveSessionId({flag: null}) — env chain only, no root adoption.
    let resolved_current = {
        let mut found: Option<String> = None;
        for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
            if let Ok(v) = std::env::var(var) {
                if !js_trim(&v).is_empty() {
                    found = Some(js_trim(&v).to_string());
                    break;
                }
            }
        }
        found
    };
    let control_root = control_root_for(ctx)?;
    let sessions = list_session_records(ctx, &control_root)?;
    if sessions.is_empty() {
        return Ok(Vec::new());
    }
    let roots = scan_transcript_roots(ctx, projects_root)?;
    let now = now_ms();
    let project_path = ctx.root.to_string_lossy().into_owned();

    let mut shared: Option<(Vec<Value>, Vec<Value>, Vec<Value>)> = None;
    let mut candidates = Vec::new();
    for session in &sessions {
        if !session.contains_key("id") || !opt_truthy(session.get("id")) {
            continue;
        }
        if let Some(current) = &resolved_current {
            if str_eq(session.get("id"), current) {
                continue;
            }
        }
        if !heartbeat_stale(session, now) {
            continue;
        }
        let mut transcript: Option<String> = None;
        let mut transcript_runtime: Option<String> = None;
        let stored_path = session
            .get("transcript_path")
            .and_then(|v| v.as_str())
            .map(js_trim)
            .filter(|s| !s.is_empty());
        if let Some(stored) = stored_path {
            if let Some(found) = resolve_transcript(None, None, None, Some(stored)) {
                let matched = roots.iter().find(|r| {
                    if !r.scanned {
                        return false;
                    }
                    let sep = std::path::MAIN_SEPARATOR;
                    let prefix = if r.path.ends_with(sep) {
                        r.path.clone()
                    } else {
                        format!("{}{}", r.path, sep)
                    };
                    found.starts_with(&prefix)
                });
                transcript_runtime = matched.map(|r| r.runtime.clone());
                transcript = Some(found);
            }
        }
        if transcript.is_none() {
            let sid = session.get("id").and_then(|v| v.as_str()).unwrap_or("");
            for r in &roots {
                if !r.scanned {
                    continue;
                }
                if let Some(found) =
                    resolve_transcript(Some(&r.path), Some(&project_path), Some(sid), None)
                {
                    transcript = Some(found);
                    transcript_runtime = Some(r.runtime.clone());
                    break;
                }
            }
        }
        let Some(transcript) = transcript else { continue };
        let tail = read_transcript_tail(Path::new(&transcript), DEFAULT_TAIL_MAX_BYTES)?;
        if has_clean_end_trio(&tail) {
            continue;
        }
        // lane = session.lane || null
        let lane: Value = match session.get("lane") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        if shared.is_none() {
            shared = Some((
                active_decisions(ctx, None),
                read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl")),
                list_cells(ctx, None, None)?,
            ));
        }
        let (decisions, capture_events, cells) = shared.as_ref().unwrap();
        let since_ms_opt = last_durable_settlement(Some(&lane), decisions, capture_events, cells);
        let since_ms = match since_ms_opt {
            Some(ms) => ms,
            None => to_ms(session.get("started_at")),
        };

        let mut work_signal: Option<&'static str> = None;
        if truthy(&lane) {
            let lane_str = jsjson::js_to_string(&lane);
            let lane_record = read_lane(ctx, &lane_str)?;
            if let Some(record) = lane_record {
                let phase = record.get("phase").and_then(|v| v.as_str()).unwrap_or("");
                if !TERMINAL_LANE_PHASES.contains(&phase) {
                    work_signal = Some("lane");
                }
            }
        }
        if work_signal.is_none() {
            let sid = session.get("id").cloned().unwrap_or(Value::Null);
            if session_has_active_claim(ctx, &control_root, &sid, now)? {
                work_signal = Some("claimed_cells");
            }
        }
        if work_signal.is_none() {
            let mut last_activity: Option<f64> = None;
            for event in &tail {
                let t = event_timestamp_ms(event);
                if t.is_finite() && last_activity.map(|l| t > l).unwrap_or(true) {
                    last_activity = Some(t);
                }
            }
            if let Some(last) = last_activity {
                if since_ms.is_finite() && last > since_ms {
                    work_signal = Some("transcript_activity");
                }
            }
        }
        let Some(work_signal) = work_signal else { continue };

        let mut row = JMap::new();
        row.insert("session_id".into(), session.get("id").cloned().unwrap_or(Value::Null));
        row.insert("lane".into(), lane);
        row.insert("transcript".into(), json!(transcript));
        row.insert(
            "runtime".into(),
            transcript_runtime.map(|r| json!(r)).unwrap_or(Value::Null),
        );
        // started_at/last_heartbeat: `session.x || null`.
        let started = match session.get("started_at") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        row.insert("started_at".into(), started);
        let heartbeat = match session.get("last_heartbeat") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        row.insert("last_heartbeat".into(), heartbeat);
        row.insert("work_signal".into(), json!(work_signal));
        row.insert(
            "since".into(),
            if since_ms.is_finite() { json!(to_iso(since_ms)) } else { Value::Null },
        );
        candidates.push(Value::Object(row));
    }
    Ok(candidates)
}

/// bee.mjs buildRecoveryBlock — Thrown degrades, Bail propagates.
pub(crate) fn build_recovery_block(ctx: &mut Ctx) -> R<JMap> {
    let attempt = |ctx: &mut Ctx| -> R<JMap> {
        let projects_root = claude_projects_root();
        let candidates = detect_crash_candidates(ctx, &projects_root)?;
        let roots = scan_transcript_roots(ctx, &projects_root)?;
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Array(candidates));
        m.insert(
            "roots".into(),
            Value::Array(
                roots
                    .into_iter()
                    .map(|r| {
                        let mut o = JMap::new();
                        o.insert("runtime".into(), json!(r.runtime));
                        o.insert("path".into(), json!(r.path));
                        o.insert("scanned".into(), json!(r.scanned));
                        o.insert(
                            "reason".into(),
                            r.reason.map(|x| json!(x)).unwrap_or(Value::Null),
                        );
                        Value::Object(o)
                    })
                    .collect(),
            ),
        );
        Ok(m)
    };
    match attempt(ctx) {
        Ok(m) => Ok(m),
        Err(Ex::Thrown) => {
            let mut m = JMap::new();
            m.insert("candidates".into(), json!([]));
            m.insert("degraded".into(), json!(true));
            Ok(m)
        }
        Err(e) => Err(e),
    }
}

// ─── runtime drift + source identity (bee.mjs / source-identity.mjs) ───────

/// bee.mjs computeRuntimeDrift — live vendored-file hashes vs the onboarding
/// ledger's managed map; fail-open to the version-only signal.
pub(crate) fn compute_runtime_drift(ctx: &Ctx, onboarding_raw: &Value) -> (bool, Vec<String>) {
    let version_drift = truthy(onboarding_raw) && {
        let v = vget(onboarding_raw, "bee_version");
        opt_truthy(v) && !str_eq(v, BEE_VERSION)
    };
    let managed = vget(onboarding_raw, "managed");
    let Some(managed @ (Value::Object(_) | Value::Array(_))) = managed else {
        return (version_drift, Vec::new());
    };
    let mut detail: Vec<String> = Vec::new();
    let mut check_group = |recorded: Option<&Value>, rel_dir: &str| {
        let Some(Value::Object(recorded)) = recorded else { return };
        for (name, recorded_hash) in recorded {
            let abs = if rel_dir.is_empty() {
                ctx.root.join(".bee").join("bin").join(name)
            } else {
                ctx.root.join(".bee").join("bin").join(rel_dir).join(name)
            };
            let rel_posix = if rel_dir.is_empty() {
                format!(".bee/bin/{name}")
            } else {
                format!(".bee/bin/{rel_dir}/{name}")
            };
            match hash_file(&abs) {
                None => detail.push(format!("{rel_posix} (missing)")),
                Some(live) => {
                    if !str_eq(Some(recorded_hash), &live) {
                        detail.push(rel_posix);
                    }
                }
            }
        }
    };
    check_group(vget(managed, "lib"), "lib");
    check_group(vget(managed, "helpers"), "");
    check_group(vget(managed, "prompts"), "prompts");
    if let Some(Value::Object(lib)) = vget(managed, "lib") {
        if let Ok(entries) = std::fs::read_dir(ctx.root.join(".bee").join("bin").join("lib")) {
            for entry in entries.filter_map(|e| e.ok()) {
                let f = entry.file_name().to_string_lossy().into_owned();
                if f.ends_with(".mjs") && !lib.contains_key(&f) {
                    detail.push(format!(".bee/bin/lib/{f} (extra)"));
                }
            }
        }
    }
    (version_drift || !detail.is_empty(), detail)
}

/// bee.mjs findRepoHive — canonical-first candidate order.
pub(crate) fn find_repo_hive(ctx: &Ctx) -> Option<PathBuf> {
    for segs in [vec!["skills"], vec![".claude", "skills"], vec![".agents", "skills"]] {
        let mut p = ctx.root.clone();
        for s in &segs {
            p = p.join(s);
        }
        let hive = p.join("bee-hive");
        if hive.exists() {
            return Some(hive);
        }
    }
    None
}

/// source-identity.mjs classifySource — only the (kind, root) pair status
/// consumes.
pub(crate) fn classify_source(hive_dir: &Path, home: &str) -> (String, Option<String>) {
    let source_root = hive_dir.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let plugin_root = source_root.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let plugin_root_str = plugin_root.to_string_lossy().into_owned();
    if source_root.join(".bee-render.json").exists() {
        return ("rendered_projection".into(), Some(plugin_root_str));
    }
    if !home.is_empty() {
        let global_root = PathBuf::from(normalize_abs_lexical(&format!(
            "{}{sep}.claude{sep}skills",
            home,
            sep = std::path::MAIN_SEPARATOR
        )));
        let rp = dunce::canonicalize(&source_root).ok();
        let rp_global = dunce::canonicalize(&global_root).ok();
        if let (Some(a), Some(b)) = (rp, rp_global) {
            if a == b {
                return ("legacy_global".into(), Some(plugin_root_str));
            }
        }
    }
    let projection_parent = path_basename(&plugin_root_str);
    if projection_parent == ".agents" || projection_parent == ".claude" {
        return ("project_projection".into(), Some(plugin_root_str));
    }
    let plugin_manifest = plugin_root.join(".claude-plugin").join("plugin.json");
    if plugin_manifest.exists() {
        // Node: JSON.parse(readFileSync(...,'utf8')) — NO BOM strip here, so a
        // BOM'd manifest parses as unknown, matching that exact behavior.
        let parse_ok = read_text_opt(&plugin_manifest)
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .is_some();
        if !parse_ok {
            return ("unknown".into(), Some(plugin_root_str));
        }
        if plugin_root.join(".git").exists() {
            return ("source_checkout".into(), Some(plugin_root_str));
        }
        return ("plugin_package".into(), Some(plugin_root_str));
    }
    ("unknown".into(), Some(plugin_root_str))
}
