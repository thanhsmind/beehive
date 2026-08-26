// `decisions tag` / `redact` / `archive`
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::roots::Roots;
use crate::textutil::truncate_chars_head;
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── the decisions family's own prelude (WIDE door) ────────────────────────
//
// `decisions *` (log, active, search, redact, render, supersede, tag,
// archive) addresses only `.bee/decisions.jsonl` + its archive under the
// store root — no sessions, claims, workers, workflows, handoff mailboxes or
// cross-worktree holds. Every decisions verb used to share
// `crate::verbs::reservations::prelude`, the NARROW door, only because
// verbs/cells.rs, verbs/state_group.rs and verbs/drivers.rs do too — not
// because decisions itself needed the control-plane refusal (roots.rs's door
// audit, "WIDE"). This is the dedicated replacement: same shape as
// `reservations::prelude`, resolved through `resolve_store_root_any` so a
// GRANTED worktree operates on its own workspace-local store instead of
// refusing. An ordinary checkout and an ungranted worktree see the identical
// root either door would have resolved.
pub(crate) fn decisions_prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> {
    let cwd = std::env::current_dir().ok()?;
    decisions_prelude_at(&cwd, cmd, use_json, t0)
}

/// `decisions_prelude`, parameterized on the start directory instead of
/// reading `std::env::current_dir()` itself — the seam `tests.rs` uses to
/// exercise a granted-worktree fixture without mutating the test runner's
/// own process-wide cwd across parallel tests (the hazard
/// `tests/workflow_verbs.rs` documents for this exact family of `prelude`
/// functions). `emit_unsupported_root` / `emit_no_root_error` only use
/// `start` for the `.bee/timings.jsonl` append, so this is the real
/// production path, not a parallel copy of it.
pub(crate) fn decisions_prelude_at(
    start: &Path,
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
) -> Option<Pre> {
    let root = match crate::roots::resolve_store_root_any(start) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(Pre::Emitted(crate::verbs::emit_unsupported_root(
                start, cmd, use_json, t0, &why,
            )))
        }
        Roots::None => {
            return Some(Pre::Emitted(crate::verbs::emit_no_root_error(
                start, cmd, use_json, t0,
            )))
        }
    };
    let drift = crate::registry::check_manifest_drift(&root);
    Some(Pre::Go(Ctx {
        root,
        cmd,
        use_json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    }))
}

// ─── decisions tag (flag form AND --stdin batch) ───────────────────────────
//
// `--stdin` is read and validated here. The precedent is verbs/cells.rs
// run_add/run_update: every remaining delegate trigger is settled BEFORE the
// read, and nothing after it may answer None.

pub(crate) fn run_tag(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["target", "tags", "scope", "stdin"]) {
        return None;
    }
    // `flags.stdin === true` — STRICTLY. A bare `--stdin` is boolean true;
    // `--stdin=true` is the STRING "true", which is not `=== true` and so
    // takes the flag form; any other value fails validate() → delegate.
    let stdin_batch = match flags.get("stdin") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" || s == "false" => false,
        Some(FlagV::S(_)) => return None,
    };

    if stdin_batch {
        // Everything that can still answer None runs FIRST: `prelude` (root
        // resolution, --json plumbing) is the last such gate, and the batch
        // body below has none. Only then is the pipe consumed.
        let ctx = match decisions_prelude("decisions tag", use_json, t0)? {
            Pre::Go(c) => c,
            Pre::Emitted(code) => return Some(code),
        };
        let out = (|| -> R2<Out> {
            let text = read_stdin_text()?;
            let entries = match parse_stdin_batch(&text) {
                Ok(e) => e,
                Err(msg) => return Ok(Out::Thrown(msg)),
            };
            let events = match tag_decisions_batch(
                &ctx.root,
                &entries,
                DECISIONS_LOCK_RETRY_ATTEMPTS,
            )? {
                Ok(e) => e,
                Err(msg) => return Ok(Out::Thrown(msg)),
            };
            let text = events
                .iter()
                .map(tag_event_summary)
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Out::Emit(Value::Array(events), text, 0))
        })();
        return finish(&ctx, out);
    }

    let target = flags.req_str("target")?.to_string();
    let tags = split_list(flags.req_str("tags")?);
    let scope = str_flag(&flags, "scope")?;

    let ctx = match decisions_prelude("decisions tag", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = do_tag(&ctx.root, &target, &tags, scope.as_deref(), DECISIONS_LOCK_RETRY_ATTEMPTS);
    finish(&ctx, out)
}

/// The two stdin refusals. `JSON.parse` is modeled with serde — a
/// lone-surrogate escape is simply "not valid JSON" here, since no Rust
/// string can hold it.
pub(crate) fn parse_stdin_batch(text: &str) -> Result<Vec<Value>, String> {
    // No BOM strip: readFileSync(0, 'utf8') hands JSON.parse the raw text, so
    // a leading BOM is a parse error in Node too.
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return Err("decisions tag --stdin: input is not valid JSON.".to_string());
    };
    let parsed = js_numberify(&parsed).unwrap_or(parsed);
    match parsed {
        Value::Array(entries) => Ok(entries),
        _ => Err(
            "decisions tag --stdin: input must be a JSON array of {target, tags, scope?}."
                .to_string(),
        ),
    }
}

/// `fs.readFileSync(0, 'utf8')`. Copied from verbs/cells.rs `read_stdin_text`
/// (private there, and that file belongs to another cell) — same lossy-utf8
/// decode Node's 'utf8' encoding performs.
pub(crate) fn read_stdin_text() -> Result<String, Err2> {
    use std::io::Read;
    let mut bytes = Vec::new();
    match std::io::stdin().lock().read_to_end(&mut bytes) {
        Ok(_) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) => Err(Err2::Msg(format!("decisions tag --stdin: could not read stdin ({e})."))),
    }
}

pub(crate) fn tag_event_summary(event: &Value) -> String {
    let scope_suffix = match jget(event, "scope") {
        Some(v) if truthy(&v) => format!(" scope={}", js_disp(&v)),
        _ => String::new(),
    };
    let tags = match jget(event, "tags") {
        Some(Value::Array(a)) => a.iter().map(js_disp).collect::<Vec<_>>().join(", "),
        _ => String::new(),
    };
    format!(
        "Tagged {} with [{tags}]{scope_suffix}.",
        js_disp_opt(jget(event, "target"))
    )
}

/// The active+archive union keyed by id (active entries replace in place,
/// archive entries only land for ids the active file does not carry),
/// filtered to decide/supersede.
pub(crate) fn decision_target_candidates(root: &Path) -> Vec<(String, Value)> {
    let active_events = read_jsonl(&decisions_path(root));
    let archived_events = read_jsonl(&decisions_archive_path(root));
    let mut by_id: Vec<(String, Value)> = Vec::new();
    for e in &active_events {
        if let Some(Value::String(id)) = jget(e, "id") {
            match by_id.iter_mut().find(|(k, _)| k.as_str() == id.as_str()) {
                Some(slot) => slot.1 = e.clone(), // Map.set keeps the position
                None => by_id.push((id.clone(), e.clone())),
            }
        }
    }
    for e in &archived_events {
        if let Some(Value::String(id)) = jget(e, "id") {
            if !by_id.iter().any(|(k, _)| k.as_str() == id.as_str()) {
                by_id.push((id.clone(), e.clone()));
            }
        }
    }
    by_id.into_iter().filter(|(_, e)| is_decide_or_supersede(e)).collect()
}

/// `Err` carries the refusal message.
pub(crate) fn resolve_tag_target(candidates: &[(String, Value)], target: Option<&Value>) -> Result<String, String> {
    // `typeof target === 'string' ? target.trim() : ''`
    let raw = match target {
        Some(Value::String(s)) => js_trim(s),
        _ => "",
    };
    if raw.is_empty() {
        return Err("decisions tag: target id (full id or short8) is required.".to_string());
    }
    if let Some((id, _)) = candidates.iter().find(|(id, _)| id == raw) {
        return Ok(id.clone());
    }
    // SHORT8_PATTERN /^[0-9a-f]{8}$/i
    let is_short8 = raw.chars().count() == 8 && raw.chars().all(|c| c.is_ascii_hexdigit());
    let mut matches: Vec<&String> = Vec::new();
    if is_short8 {
        let low = raw.to_ascii_lowercase();
        for (id, _) in candidates {
            if id.to_lowercase().starts_with(&low) {
                matches.push(id);
            }
        }
    }
    match matches.len() {
        1 => Ok(matches[0].clone()),
        n if n > 1 => {
            let list = matches.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            Err(format!(
                "decisions tag: short id {} is ambiguous — matches {n} events ({list}); use the full id.",
                js_quote(raw)
            ))
        }
        _ => Err(format!(
            "decisions tag: target {} does not resolve to any decide/supersede event in the active+archive union.",
            js_quote(raw)
        )),
    }
}

/// The batch form, over a RAW JSON value where `entry.tags` is whatever the
/// payload carried.
pub(crate) fn normalize_tag_event_tags_value(tags: Option<&Value>) -> Result<Vec<String>, String> {
    let items = match tags {
        Some(Value::Array(a)) if !a.is_empty() => a,
        _ => {
            return Err(
                "decisions tag: --tags is required (at least one lowercase slug, e.g. \"billing,nightly-job\").".to_string(),
            )
        }
    };
    // `String(tag).trim()` for every element, then TAG_PATTERN per element.
    let cleaned: Vec<String> = items.iter().map(|t| js_trim(&js_disp(t)).to_string()).collect();
    for tag in &cleaned {
        if !tag_pattern_test(tag) {
            return Err(format!(
                "decisions tag: tag {} is not a valid lowercase slug (must match {TAG_PATTERN_DISPLAY}).",
                js_quote(tag)
            ));
        }
    }
    Ok(cleaned)
}

/// Every event is built BEFORE the lock is taken (dp-3's lock doctrine) — the
/// critical section is exactly the one batch-append write. `Ok(Err(msg))` is
/// a refusal with zero writes; `Err(Err2::…)` is a lock refusal or an fs
/// write failure.
pub(crate) fn tag_decisions_batch(
    root: &Path,
    entries: &[Value],
    lock_retries: u32,
) -> R2<Result<Vec<Value>, String>> {
    if entries.is_empty() {
        return Ok(Err(
            "decisions tag: at least one entry ({target, tags, scope?}) is required.".to_string(),
        ));
    }
    let candidates = decision_target_candidates(root);
    let now = now_iso(); // ONE `new Date().toISOString()` for the whole batch
    let mut events: Vec<Value> = Vec::new();
    for entry in entries {
        // `!entry || typeof entry !== 'object' || Array.isArray(entry)`
        let Value::Object(fields) = entry else {
            return Ok(Err(format!(
                "decisions tag: batch entry must be an object {{target, tags, scope?}}, got {}.",
                jsjson::stringify(entry)
            )));
        };
        let target_id = match resolve_tag_target(&candidates, fields.get("target")) {
            Ok(id) => id,
            Err(msg) => return Ok(Err(msg)),
        };
        let tags = match normalize_tag_event_tags_value(fields.get("tags")) {
            Ok(t) => t,
            Err(msg) => return Ok(Err(msg)),
        };
        // `entry.scope !== undefined && entry.scope !== null && String(entry.scope).trim()`
        let scope: Option<String> = match fields.get("scope") {
            None | Some(Value::Null) => None,
            Some(v) => {
                let s = js_disp(v);
                let t = js_trim(&s).to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            }
        };
        if let Err(msg) = assert_safe_content("scope", scope.as_deref()) {
            return Ok(Err(msg));
        }
        let mut event = Map::new();
        event.insert("id".into(), Value::String(pseudo_uuid_v4()));
        event.insert("type".into(), Value::String("tag".into()));
        event.insert("date".into(), Value::String(now.clone()));
        event.insert("target".into(), Value::String(target_id));
        event.insert(
            "tags".into(),
            Value::Array(tags.into_iter().map(Value::String).collect()),
        );
        if let Some(s) = scope {
            event.insert("scope".into(), Value::String(s));
        }
        events.push(Value::Object(event));
    }

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    let written = append_jsonl_batch(&decisions_path(root), &events);
    drop(guard);
    written.map_err(|_| Err2::Ex)?;
    Ok(Ok(events))
}

/// The flag form: runs the same batch body over a one-entry batch and emits
/// the single event.
pub(crate) fn do_tag(
    root: &Path,
    target: &str,
    tags: &[String],
    scope: Option<&str>,
    lock_retries: u32,
) -> R2<Out> {
    let mut entry = Map::new();
    entry.insert("target".into(), Value::String(target.to_string()));
    entry.insert(
        "tags".into(),
        Value::Array(tags.iter().cloned().map(Value::String).collect()),
    );
    if let Some(s) = scope {
        entry.insert("scope".into(), Value::String(s.to_string()));
    }
    let entries = [Value::Object(entry)];
    let events = match tag_decisions_batch(root, &entries, lock_retries)? {
        Ok(e) => e,
        Err(msg) => return Ok(Out::Thrown(msg)),
    };
    let event = events.into_iter().next().expect("a one-entry batch yields one event");
    let text = tag_event_summary(&event);
    Ok(Out::Emit(event, text, 0))
}

// ─── decisions redact ──────────────────────────────────────────────────────

pub(crate) fn run_redact(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["id", "reason"]) {
        return None;
    }
    let redacts = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let ctx = match decisions_prelude("decisions redact", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = do_redact(&ctx.root, &redacts, &reason, DECISIONS_LOCK_RETRY_ATTEMPTS);
    finish(&ctx, out)
}

pub(crate) fn do_redact(root: &Path, redacts: &str, reason: &str, lock_retries: u32) -> R2<Out> {
    if js_trim(redacts).is_empty() {
        return Ok(Out::Thrown(
            "redactDecision: redacts (decision id) is required.".into(),
        ));
    }
    if js_trim(reason).is_empty() {
        return Ok(Out::Thrown("redactDecision: reason is required.".into()));
    }
    let mut event = Map::new();
    event.insert("id".into(), Value::String(pseudo_uuid_v4()));
    event.insert("type".into(), Value::String("redact".into()));
    event.insert("date".into(), Value::String(now_iso()));
    event.insert("redacts".into(), Value::String(js_trim(redacts).to_string()));
    event.insert("reason".into(), Value::String(js_trim(reason).to_string()));
    let event = Value::Object(event);

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    append_jsonl(&decisions_path(root), &event).map_err(|_| Err2::Ex)?;
    drop(guard);

    let text = format!("Redacted {}.", js_disp_opt(jget(&event, "redacts")));
    Ok(Out::Emit(event, text, 0))
}

// ─── decisions reattribute ─────────────────────────────────────────────────

/// decision-attribution D5: the feature a decision's own text claims, when it
/// opens with the `<slug> D<n>` convention every provable mis-stamp followed.
///
/// Deliberately strict. The slug must be lowercase alphanumerics and dashes,
/// followed by a single space, `D`, and at least one digit. Anything else
/// returns `None` and the record is left alone — this reads a claim the record
/// already makes about itself, it never infers one.
pub(crate) fn feature_from_decision_text(text: &str) -> Option<String> {
    let t = js_trim(text);
    let (slug, rest) = t.split_once(' ')?;
    if slug.is_empty() || slug.len() > 64 {
        return None;
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return None;
    }
    if !slug.chars().any(|c| c.is_ascii_lowercase()) {
        return None;
    }
    let marker = rest.strip_prefix('D')?;
    let digits: String = marker.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    Some(slug.to_string())
}

/// decision-attribution D5: one planned correction.
pub(crate) struct Reattribution {
    pub(crate) id: String,
    pub(crate) from: String,
    pub(crate) to: String,
}

/// decision-attribution D5: decide, for one already-parsed event, whether its
/// stamp contradicts its own text.
///
/// Acts ONLY on a contradiction: the record carries a `feature`, its text
/// names a feature, and the two differ. A record with NO stamp is left
/// unstamped — post-D1 that is a legitimate, common state, and filling it in
/// from a prose convention would be the inference D2 explicitly rejected.
pub(crate) fn plan_reattribution(event: &Value) -> Option<Reattribution> {
    let id = jget(event, "id").and_then(Value::as_str)?.to_string();
    let stamped = jget(event, "feature").and_then(Value::as_str)?;
    if js_trim(stamped).is_empty() {
        return None;
    }
    let text = jget(event, "decision").and_then(Value::as_str)?;
    let claimed = feature_from_decision_text(text)?;
    if claimed == js_trim(stamped) {
        return None;
    }
    Some(Reattribution { id, from: js_trim(stamped).to_string(), to: claimed })
}

pub(crate) fn run_reattribute(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["dry-run", "id", "to"]) {
        return None;
    }
    let dry_run = bool_flag_present(&flags, "dry-run")?;
    let id = flags.truthy_str("id").map(str::to_string);
    let to = match flags.get("to") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
    };
    let ctx = match decisions_prelude("decisions reattribute", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = match (id, to) {
        (None, None) => do_reattribute(&ctx.root, dry_run, DECISIONS_LOCK_RETRY_ATTEMPTS),
        (Some(id), Some(to)) => {
            do_reattribute_named(&ctx.root, &id, &to, DECISIONS_LOCK_RETRY_ATTEMPTS)
        }
        _ => Ok(Out::Thrown(
            "decisions reattribute: --id and --to come together or not at all — the pair names one explicit correction; the bare verb runs the automatic text-claim pass."
                .into(),
        )),
    };
    finish(&ctx, out)
}

/// decision-attribution D5's residual (filed P3): the correction a HUMAN
/// names, for a record whose text makes no `<slug> D<n>` claim — the
/// automatic pass correctly declines those, because it corrects
/// contradictions and never invents an attribution. This door does not
/// invent one either: it records the operator's explicit word, and it
/// REFUSES to contradict a record whose own text claims a different
/// feature — that territory belongs to the automatic pass.
pub(crate) fn do_reattribute_named(
    root: &Path,
    id: &str,
    to: &str,
    lock_retries: u32,
) -> R2<Out> {
    let id = js_trim(id);
    let to = js_trim(to);
    if id.is_empty() {
        return Ok(Out::Thrown("decisions reattribute: --id is empty.".into()));
    }
    if to.is_empty() {
        return Ok(Out::Thrown(
            "decisions reattribute: --to is empty — name the feature this decision is about."
                .into(),
        ));
    }
    let path = decisions_path(root);
    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            drop(guard);
            return Ok(Out::Thrown("decisions reattribute: no decisions store.".into()));
        }
    };

    let mut out_lines: Vec<String> = Vec::new();
    let mut result: Option<(String, String)> = None; // (from, to)
    let mut found = false;
    let mut refusal: Option<String> = None;
    for line in raw.lines() {
        // First match wins outright: a short --id prefix must never rewrite
        // a second record further down.
        if found || refusal.is_some() {
            out_lines.push(line.to_string());
            continue;
        }
        let parsed = serde_json::from_str::<Value>(line).ok();
        let matches = parsed
            .as_ref()
            .and_then(|e| jget(e, "id"))
            .and_then(Value::as_str)
            .is_some_and(|eid| eid == id || eid.starts_with(id));
        if !matches {
            out_lines.push(line.to_string());
            continue;
        }
        found = true;
        let mut event = parsed.unwrap();
        let stamped = jget(&event, "feature").and_then(Value::as_str).map(js_trim);
        if stamped == Some(to) {
            out_lines.push(line.to_string());
            result = Some((to.to_string(), to.to_string()));
            continue;
        }
        let text_claim = jget(&event, "decision")
            .and_then(Value::as_str)
            .and_then(feature_from_decision_text);
        if let Some(claimed) = text_claim {
            if claimed != to {
                refusal = Some(format!(
                    "decisions reattribute: {id} opens with \"{claimed} D<n>\" — its own text claims a feature, and this door never contradicts a record's text. The automatic pass (`bee decisions reattribute` with no flags) owns text-claimed corrections; pass --to \"{claimed}\" if that is what you meant."
                ));
                out_lines.push(line.to_string());
                continue;
            }
        }
        let from = stamped.unwrap_or("").to_string();
        if let Some(obj) = event.as_object_mut() {
            obj.insert("feature".into(), Value::String(to.to_string()));
        }
        out_lines.push(serde_json::to_string(&event).map_err(|_| Err2::Ex)?);
        result = Some((from, to.to_string()));
    }

    if let Some(msg) = refusal {
        drop(guard);
        return Ok(Out::Thrown(msg));
    }
    if !found {
        drop(guard);
        return Ok(Out::Thrown(format!(
            "decisions reattribute: no decision matches id \"{id}\"."
        )));
    }
    let (from, to_val) = result.unwrap();
    let changed = from != to_val;
    if changed {
        let tmp = path.with_extension("jsonl.reattribute.tmp");
        let mut body = out_lines.join("\n");
        body.push('\n');
        let write = std::fs::write(&tmp, body).and_then(|()| std::fs::rename(&tmp, &path));
        if write.is_err() {
            let _ = std::fs::remove_file(&tmp);
            drop(guard);
            return Err(Err2::Ex);
        }
    }
    drop(guard);
    let text = if changed {
        let shown = if from.is_empty() { "(none)".to_string() } else { from.clone() };
        format!("decisions reattribute: corrected 1 — {} : {shown} -> {to_val}", truncate_chars_head(id, 8))
    } else {
        format!(
            "decisions reattribute: corrected 0 — {} already carries \"{to_val}\".",
            truncate_chars_head(id, 8)
        )
    };
    Ok(Out::Emit(
        json!({"scanned": 1, "changed": if changed {1} else {0}, "id": id, "from": from, "to": to_val}),
        text,
        0,
    ))
}

/// decision-attribution D5. The lock is held across the WHOLE pass — the read
/// as well as the write. `cells backfill-roles` shipped a scan outside its
/// lock and silently reversed an operator's concurrent write; sibling sessions
/// append to this file continuously, so the same shape would lose decisions.
/// Every untouched line is written back byte-for-byte, so only the records
/// this verb corrects can differ at all.
pub(crate) fn do_reattribute(root: &Path, dry_run: bool, lock_retries: u32) -> R2<Out> {
    let path = decisions_path(root);
    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            drop(guard);
            return Ok(Out::Emit(
                json!({"scanned": 0, "changed": 0, "dry_run": dry_run, "changes": []}),
                "decisions reattribute: no decisions store — nothing to do.".to_string(),
                0,
            ));
        }
    };

    let mut scanned = 0usize;
    let mut changes: Vec<Reattribution> = Vec::new();
    let mut out_lines: Vec<String> = Vec::new();
    for line in raw.lines() {
        if js_trim(line).is_empty() {
            out_lines.push(line.to_string());
            continue;
        }
        scanned += 1;
        let Ok(mut event) = serde_json::from_str::<Value>(line) else {
            // Unparseable lines are never rewritten, only carried through.
            out_lines.push(line.to_string());
            continue;
        };
        match plan_reattribution(&event) {
            None => out_lines.push(line.to_string()),
            Some(plan) => {
                if let Some(obj) = event.as_object_mut() {
                    // `preserve_order` keeps the key in place, so the line
                    // differs in this value and nothing else.
                    obj.insert("feature".into(), Value::String(plan.to.clone()));
                }
                out_lines.push(serde_json::to_string(&event).map_err(|_| Err2::Ex)?);
                changes.push(plan);
            }
        }
    }

    if !dry_run && !changes.is_empty() {
        let tmp = path.with_extension("jsonl.reattribute.tmp");
        let mut body = out_lines.join("\n");
        body.push('\n');
        let write = std::fs::write(&tmp, body).and_then(|()| std::fs::rename(&tmp, &path));
        if write.is_err() {
            let _ = std::fs::remove_file(&tmp);
            drop(guard);
            return Err(Err2::Ex);
        }
    }
    drop(guard);

    let listed: Vec<Value> = changes
        .iter()
        .map(|c| json!({"id": c.id, "from": c.from, "to": c.to}))
        .collect();
    let verb = if dry_run { "would correct" } else { "corrected" };
    let mut text = format!(
        "decisions reattribute{}: {scanned} decision(s) scanned, {verb} {}.",
        if dry_run { " --dry-run" } else { "" },
        changes.len()
    );
    for c in &changes {
        text.push_str(&format!("\n  {} : {} -> {}", truncate_chars_head(&c.id, 8), c.from, c.to));
    }
    if dry_run && !changes.is_empty() {
        text.push_str("\nNothing was written. Re-run without --dry-run to apply.");
    }
    Ok(Out::Emit(
        json!({"scanned": scanned, "changed": changes.len(), "dry_run": dry_run, "changes": listed}),
        text,
        0,
    ))
}

// ─── decisions archive ─────────────────────────────────────────────────────

pub(crate) fn run_archive(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["before"]) {
        return None;
    }
    let before = flags.req_str("before")?.to_string();
    let ctx = match decisions_prelude("decisions archive", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    // Pre-check: the partition is a pure function of the store — run it dry
    // BEFORE the lock so Exotic input delegates without any lock telemetry.
    let precheck = (|| -> Ex<()> {
        let before_ms = match js_date_parse(js_trim(&before)) {
            Ok(Some(ms)) => ms,
            Ok(None) => return Ok(()), // Node's own "--before must be valid" throw
            Err(e) => return Err(e),
        };
        let events = read_jsonl(&decisions_path(&ctx.root));
        partition_archive(&events, before_ms)?;
        Ok(())
    })();
    if precheck.is_err() {
        return None;
    }
    let out = do_archive(&ctx.root, &before, DECISIONS_LOCK_RETRY_ATTEMPTS);
    finish(&ctx, out)
}

pub(crate) fn partition_archive(events: &[Value], before_ms: f64) -> Ex<(Vec<Value>, Vec<Value>)> {
    let mut superseded = VSet::new();
    let mut redacted = VSet::new();
    for e in events {
        if matches!(jget(e, "type"), Some(t) if v_is_str(t, "supersede")) {
            if let Some(s) = jget(e, "supersedes") {
                if truthy(s) {
                    superseded.add(s);
                }
            }
        }
        if matches!(jget(e, "type"), Some(t) if v_is_str(t, "redact")) {
            if let Some(r) = jget(e, "redacts") {
                if truthy(r) {
                    redacted.add(r);
                }
            }
        }
    }
    let mut to_archive = Vec::new();
    let mut to_keep = Vec::new();
    for e in events {
        let id = match jget(e, "id") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };
        let Some(id) = id else {
            to_keep.push(e.clone()); // never drop a malformed-but-parsed line
            continue;
        };
        let idv = Value::String(id);
        if superseded.has(&idv) || redacted.has(&idv) {
            to_archive.push(e.clone());
            continue;
        }
        if matches!(jget(e, "type"), Some(t) if v_is_str(t, "decide")) {
            if let Some(ms) = date_parse_val(jget(e, "date"))? {
                if ms < before_ms {
                    to_archive.push(e.clone());
                    continue;
                }
            }
        }
        to_keep.push(e.clone());
    }
    Ok((to_archive, to_keep))
}

pub(crate) fn do_archive(root: &Path, before: &str, lock_retries: u32) -> R2<Out> {
    let before_str = js_trim(before).to_string();
    if before_str.is_empty() {
        return Ok(Out::Thrown(
            "archiveDecisions: --before <ISO date> is required — decisions archive never runs a default age-based purge (decision-propagation D4c).".into(),
        ));
    }
    let before_ms = match js_date_parse(&before_str)? {
        Some(ms) => ms,
        None => {
            return Ok(Out::Thrown(format!(
                "archiveDecisions: --before must be a valid ISO date, got {}.",
                js_quote(&before_str)
            )));
        }
    };

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    let out = (|| -> R2<Out> {
        let events = read_jsonl(&decisions_path(root));
        let (to_archive, to_keep) = partition_archive(&events, before_ms)?;
        if to_archive.is_empty() {
            return Ok(Out::Thrown(format!(
                "archiveDecisions: nothing qualifies for archiving — no superseded/redacted events and no decide events strictly older than {before_str} (decision-propagation D4c: --before is explicit or the verb refuses; there is never a default age-based purge)."
            )));
        }
        // Crash ordering: archive-append FIRST (one appendFileSync), then the
        // pruned active file via the atomic temp-write+rename.
        let archive_path = decisions_archive_path(root);
        if let Some(dir) = archive_path.parent() {
            ensure_dir(dir).map_err(|_| Err2::Ex)?;
        }
        let body = to_archive.iter().map(jsjson::stringify).collect::<Vec<_>>().join("\n");
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&archive_path)
                .map_err(|_| Err2::Ex)?;
            f.write_all(format!("{body}\n").as_bytes()).map_err(|_| Err2::Ex)?;
        }
        write_jsonl_atomic(&decisions_path(root), &to_keep).map_err(|_| Err2::Ex)?;

        let archived_ids: Vec<Value> = to_archive
            .iter()
            .map(|e| jget(e, "id").cloned().unwrap_or(Value::Null))
            .collect();
        let result = json!({
            "archived": archived_ids,
            "kept": to_keep.len() as f64,
            "before": before_str,
        });
        let text = format!(
            "Archived {} decision(s) to .bee/decisions-archive.jsonl (kept {} active, cutoff {}).",
            to_archive.len(),
            to_keep.len(),
            before_str
        );
        Ok(Out::Emit(result, text, 0))
    })();
    drop(guard);
    out
}
