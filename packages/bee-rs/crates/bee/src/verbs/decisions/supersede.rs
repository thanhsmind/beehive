// the capture-stub side effect and `decisions supersede`
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::textutil::truncate_chars_head;
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── capture.mjs addCaptureStub (the supersede-sweep side effect) ──────────

pub(crate) fn capture_queue_path(root: &Path) -> PathBuf {
    root.join(".bee").join("capture-queue.jsonl")
}

/// capture.mjs's own assertSafeContent — same pattern tables as decisions.mjs
/// (it imports them), different refusal wording.
pub(crate) fn assert_safe_capture_content(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Ok(());
    }
    let chars: Vec<char> = value.chars().collect();
    for (matcher, display) in SECRET_PATTERNS {
        if matcher(&chars) {
            return Err(format!(
                "Capture stub rejected: field \"{field}\" matches a secret pattern ({display}). Never queue credentials — describe the outcome without the secret."
            ));
        }
    }
    for (matcher, display) in INJECTION_PATTERNS {
        if matcher(&chars) {
            return Err(format!(
                "Capture stub rejected: field \"{field}\" contains instruction-like content ({display}). Stub text must be data, not instructions."
            ));
        }
    }
    Ok(())
}

/// addCaptureStub for the array-valued call shape handleDecisionsSupersede
/// uses (dids/files arrays, area/lane null, source a non-empty literal).
pub(crate) fn add_capture_stub(
    root: &Path,
    outcome: &str,
    dids: &[String],
    files: &[String],
    source: &str,
) -> R2<()> {
    let norm = |list: &[String]| -> Vec<Value> {
        list.iter()
            .map(|v| js_trim(v).to_string())
            .filter(|v| !v.is_empty())
            .map(Value::String)
            .collect()
    };
    let mut stub = Map::new();
    stub.insert("kind".into(), Value::String("stub".into()));
    stub.insert("id".into(), Value::String(pseudo_uuid_v4()));
    stub.insert("at".into(), Value::String(now_iso()));
    stub.insert("outcome".into(), Value::String(js_trim(outcome).to_string()));
    stub.insert("dids".into(), Value::Array(norm(dids)));
    stub.insert("area".into(), Value::Null);
    stub.insert("files".into(), Value::Array(norm(files)));
    stub.insert("lane".into(), Value::Null);
    stub.insert("source".into(), Value::String(source.to_string()));
    let stub = Value::Object(stub);
    if let Err(msg) = assert_safe_capture_content("outcome", js_trim(outcome)) {
        return Err(Err2::Msg(msg));
    }
    append_jsonl(&capture_queue_path(root), &stub).map_err(|_| Err2::Ex)?;
    Ok(())
}

// ─── decisions supersede ───────────────────────────────────────────────────

pub(crate) fn run_supersede(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["id", "decision", "rationale", "tags", "scope"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let decision = flags.req_str("decision")?.to_string();
    let rationale = flags.req_str("rationale")?.to_string();
    let tags: Option<Vec<String>> = match flags.get("tags") {
        None => None,
        Some(FlagV::S(s)) => Some(split_list(s)),
        Some(FlagV::Present) => return None,
    };
    let scope = str_flag(&flags, "scope")?;
    // The sweep's `i`-flag regex is only provably ASCII-folding for an ASCII
    // needle; a non-ASCII id leaves the modeled region.
    if !id.is_ascii() {
        return None;
    }

    let ctx = match crate::verbs::reservations::prelude("decisions supersede", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = do_supersede(
        &ctx.root,
        SupersedeParams { id, decision, rationale, tags, scope },
        DECISIONS_LOCK_RETRY_ATTEMPTS,
    );
    finish(&ctx, out)
}

pub(crate) struct SupersedeParams {
    pub(crate) id: String,
    pub(crate) decision: String,
    pub(crate) rationale: String,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) scope: Option<String>,
}

pub(crate) fn do_supersede(root: &Path, p: SupersedeParams, lock_retries: u32) -> R2<Out> {
    // ── supersedeDecision (lib/decisions.mjs) ──────────────────────────────
    if js_trim(&p.id).is_empty() {
        return Ok(Out::Thrown(
            "supersedeDecision: supersedes (decision id) is required.".into(),
        ));
    }
    if js_trim(&p.decision).is_empty() {
        return Ok(Out::Thrown(
            "supersedeDecision: replacement decision text is required.".into(),
        ));
    }
    if js_trim(&p.rationale).is_empty() {
        return Ok(Out::Thrown("supersedeDecision: rationale is required.".into()));
    }
    let target_id = js_trim(&p.id).to_string();
    // assertSafe({ decision, rationale }) — the untrimmed values, per Node.
    for (field, value) in [
        ("decision", p.decision.as_str()),
        ("rationale", p.rationale.as_str()),
    ] {
        if let Err(msg) = assert_safe_content(field, Some(value)) {
            return Ok(Out::Thrown(msg));
        }
    }

    // Scope/tag inheritance consults the OVERLAY-APPLIED target (dp-6 W3).
    let events = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay(&events)?;
    let raw_target = events
        .iter()
        .find(|e| !e.is_null() && matches!(jget(e, "id"), Some(v) if v_is_str(v, &target_id)));
    let target = raw_target.map(|e| apply_tag_overlay(e, &overlay));

    let resolved_scope = match &p.scope {
        Some(s) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => match target.as_ref().and_then(|t| jget(t, "scope")) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
            _ => "repo".to_string(),
        },
    };
    if let Err(msg) = assert_safe_content("scope", Some(resolved_scope.as_str())) {
        return Ok(Out::Thrown(msg));
    }

    let resolved_tags: Option<Vec<String>> = if p.tags.is_some() {
        match normalize_tags(p.tags.clone()) {
            Ok(n) => n,
            Err(msg) => return Ok(Out::Thrown(msg)),
        }
    } else {
        let inherited: Option<Vec<String>> = match target.as_ref().and_then(|t| jget(t, "tags")) {
            Some(Value::Array(a)) if !a.is_empty() => Some(a.iter().map(js_disp).collect()),
            _ => None,
        };
        match inherited {
            None => None,
            Some(list) => match normalize_tags(Some(list)) {
                Ok(n) => n,
                Err(msg) => return Ok(Out::Thrown(msg)),
            },
        }
    };
    classify_decision_tags(root, &resolved_tags.clone().unwrap_or_default(), lock_retries)?;

    // Sweep BEFORE the append (lock doctrine): the event carries it inline.
    let short8 = truncate_chars_head(&target_id, 8);
    let sweep = sweep_decision_citations(root, &target_id, &short8);

    let mut event = Map::new();
    event.insert("id".into(), Value::String(pseudo_uuid_v4()));
    event.insert("type".into(), Value::String("supersede".into()));
    event.insert("date".into(), Value::String(now_iso()));
    event.insert("supersedes".into(), Value::String(target_id.clone()));
    event.insert("decision".into(), Value::String(js_trim(&p.decision).to_string()));
    event.insert("rationale".into(), Value::String(js_trim(&p.rationale).to_string()));
    event.insert("scope".into(), Value::String(resolved_scope));
    event.insert("sweep".into(), sweep.clone());
    if let Some(tags) = &resolved_tags {
        event.insert(
            "tags".into(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    let event = Value::Object(event);

    // Pre-flight the capture queue BEFORE the store write, so the only
    // post-write failure left is a genuine IO fault (a delegate there would
    // re-run the whole verb under Node and double-append the event).
    let queue = capture_queue_path(root);
    if let Some(dir) = queue.parent() {
        ensure_dir(dir).map_err(|_| Err2::Ex)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&queue)
        .map_err(|_| Err2::Ex)?;

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    let append = append_jsonl(&decisions_path(root), &event);
    drop(guard);
    append.map_err(|_| Err2::Ex)?;

    // ── handleDecisionsSupersede: one capture stub per citing line ─────────
    let new_id = js_disp_opt(jget(&event, "id"));
    let hits: Vec<Value> = match jget(&sweep, "files") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    for hit in &hits {
        let file = js_disp_opt(jget(hit, "file"));
        let line = js_disp_opt(jget(hit, "line"));
        let outcome = format!(
            "{file}:{line} still cites superseded decision {target_id} — reconcile against replacement {new_id}."
        );
        add_capture_stub(
            root,
            &outcome,
            &[target_id.clone(), new_id.clone()],
            &[file],
            "supersede-sweep",
        )?;
    }

    let header = format!("Superseded {target_id} with {new_id}.");
    let mut lines = vec![header];
    if hits.is_empty() {
        lines.push("Propagation sweep: no citations found under docs/**.".into());
    } else {
        lines.push(format!(
            "Propagation sweep: {} citation(s) found under docs/** — a capture stub was queued for each.",
            hits.len()
        ));
        for hit in &hits {
            lines.push(format!(
                "  {}:{}  {}",
                js_disp_opt(jget(hit, "file")),
                js_disp_opt(jget(hit, "line")),
                js_disp_opt(jget(hit, "excerpt"))
            ));
        }
    }
    Ok(Out::Emit(event, lines.join("\n"), 0))
}
