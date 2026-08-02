// decisions, the capture queue, backlog counts and reviews
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

// ─── decisions (decisions.mjs) ─────────────────────────────────────────────

pub(crate) fn decisions_path(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("decisions.jsonl")
}

/// decisions.mjs buildTagOverlay — last tag event per target wins after a
/// (date, index) stable sort.
pub(crate) fn build_tag_overlay(ctx: &Ctx) -> HashMap<String, (Option<Vec<Value>>, Option<String>)> {
    let events = read_jsonl(&decisions_path(ctx));
    let mut tag_events: Vec<(usize, &Value)> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            truthy(e)
                && str_eq(vget(e, "type"), "tag")
                && matches!(vget(e, "target"), Some(Value::String(_)))
        })
        .collect();
    tag_events.sort_by(|(ai, a), (bi, b)| {
        let ams = date_parse_val(vget(a, "date"));
        let bms = date_parse_val(vget(b, "date"));
        if ams.is_finite() && bms.is_finite() && ams != bms {
            return ams.partial_cmp(&bms).unwrap_or(Ordering::Equal);
        }
        ai.cmp(bi)
    });
    let mut overlay = HashMap::new();
    for (_, event) in tag_events {
        let target = vget(event, "target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tags = match vget(event, "tags") {
            Some(Value::Array(a)) => Some(a.clone()),
            _ => None,
        };
        let scope = match vget(event, "scope") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };
        overlay.insert(target, (tags, scope));
    }
    overlay
}

pub(crate) fn apply_tag_overlay(
    event: &Value,
    overlay: &HashMap<String, (Option<Vec<Value>>, Option<String>)>,
) -> Value {
    let id = vget(event, "id").and_then(|v| v.as_str());
    let Some(id) = id else { return event.clone() };
    let Some((tags, scope)) = overlay.get(id) else {
        return event.clone();
    };
    let mut next = match event {
        Value::Object(m) => m.clone(),
        _ => return event.clone(),
    };
    if let Some(tags) = tags {
        next.insert("tags".into(), Value::Array(tags.clone()));
    }
    if let Some(scope) = scope {
        next.insert("scope".into(), json!(scope));
    }
    Value::Object(next)
}

/// decisions.mjs activeDecisions (default branch — `all` is never true on
/// the status/orient path): decide/supersede events not superseded/redacted,
/// newest first (reverse file order), overlay applied, optional recent cap.
pub(crate) fn active_decisions(ctx: &Ctx, recent: Option<usize>) -> Vec<Value> {
    let overlay = build_tag_overlay(ctx);
    let events = read_jsonl(&decisions_path(ctx));
    let mut superseded: Vec<Value> = Vec::new();
    let mut redacted: Vec<Value> = Vec::new();
    for event in &events {
        if str_eq(vget(event, "type"), "supersede") && opt_truthy(vget(event, "supersedes")) {
            superseded.push(vget(event, "supersedes").unwrap().clone());
        }
        if str_eq(vget(event, "type"), "redact") && opt_truthy(vget(event, "redacts")) {
            redacted.push(vget(event, "redacts").unwrap().clone());
        }
    }
    let in_set = |set: &[Value], id: Option<&Value>| -> bool {
        // JS Set.has uses SameValueZero; ids are strings in practice.
        set.iter().any(|v| strict_eq(Some(v), id))
    };
    let mut active: Vec<Value> = events
        .iter()
        .filter(|event| {
            let ty = vget(event, "type");
            (str_eq(ty, "decide") || str_eq(ty, "supersede"))
                && !in_set(&superseded, vget(event, "id"))
                && !in_set(&redacted, vget(event, "id"))
        })
        .cloned()
        .collect();
    active.reverse();
    let mut out: Vec<Value> = active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect();
    if let Some(n) = recent {
        out.truncate(n);
    }
    out
}

/// decisions.mjs datamark — neutralize resurfaced text.
pub(crate) fn datamark(text: Option<&Value>) -> String {
    // String(text ?? '')
    let s = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // .replace(/```+/g, '') — runs of >= 3 backticks removed.
    let mut no_ticks = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut j = i;
            while j < chars.len() && chars[j] == '`' {
                j += 1;
            }
            if j - i >= 3 {
                i = j;
                continue;
            }
            for k in i..j {
                no_ticks.push(chars[k]);
            }
            i = j;
            continue;
        }
        no_ticks.push(chars[i]);
        i += 1;
    }
    // .replace(/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi, '')
    let no_tags = strip_role_tags(&no_ticks);
    // control-char strip (keeps \t \n \r), then trim.
    let cleaned: String = no_tags
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            !(cp <= 0x08 || cp == 0x0B || cp == 0x0C || (0x0E..=0x1F).contains(&cp) || cp == 0x7F)
        })
        .collect();
    format!("«{}»", js_trim(&cleaned))
}

pub(crate) fn strip_role_tags(s: &str) -> String {
    const ROLES: [&str; 5] = ["system", "assistant", "user", "developer", "tool"];
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    'outer: while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            if j < chars.len() && chars[j] == '/' {
                j += 1;
            }
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            for role in ROLES {
                let rl = role.chars().count();
                if j + rl <= chars.len() {
                    let seg: String = chars[j..j + rl].iter().collect::<String>().to_lowercase();
                    if seg == role {
                        let after = j + rl;
                        // \b: next char must not be a word char.
                        let boundary = after >= chars.len()
                            || !(chars[after].is_ascii_alphanumeric() || chars[after] == '_');
                        if boundary {
                            // [^>]*>
                            let mut k = after;
                            while k < chars.len() && chars[k] != '>' {
                                k += 1;
                            }
                            if k < chars.len() {
                                i = k + 1;
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ─── capture queue (capture.mjs) ───────────────────────────────────────────

/// capture.mjs pendingCaptureStubs + captureQueue -> {count, ids}.
pub(crate) fn capture_queue_summary(ctx: &Ctx) -> JMap {
    let events = read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        if str_eq(vget(event, "kind"), "flush") && opt_truthy(vget(event, "id")) {
            flushed.push(vget(event, "id").unwrap().clone());
        } else if str_eq(vget(event, "kind"), "stub") && opt_truthy(vget(event, "id")) {
            stubs.push(event);
        }
    }
    let mut pending: Vec<&Value> = stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .collect();
    pending.sort_by(|a, b| locale_cmp(&tpl(vget(a, "at")), &tpl(vget(b, "at")), false));
    let mut out = JMap::new();
    out.insert("count".into(), json!(pending.len()));
    out.insert(
        "ids".into(),
        Value::Array(pending.iter().map(|s| vget(s, "id").cloned().unwrap_or(Value::Null)).collect()),
    );
    out
}

// ─── backlog counts (backlog.mjs) ──────────────────────────────────────────

/// backlog.mjs tokenKey: 'in-flight' -> 'inFlight'.
pub(crate) fn token_key(token: &str) -> String {
    let mut out = String::new();
    let mut upper_next = false;
    for c in token.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// backlog.mjs foldPbis + foldedBacklogCounts + legacyBacklogCounts ->
/// readBacklogCounts. Returns None only when neither store parses.
pub(crate) fn read_backlog_counts(ctx: &mut Ctx) -> R<Option<JMap>> {
    // foldPbis over .bee/backlog.jsonl.
    let text = read_text_opt(&ctx.root.join(".bee").join("backlog.jsonl"));
    let mut has_events = false;
    let mut order: Vec<String> = Vec::new();
    let mut items: HashMap<String, String> = HashMap::new(); // id -> status
    if let Some(text) = text {
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let trimmed = js_trim(line);
            if trimmed.is_empty() {
                continue;
            }
            let Ok(row) = serde_json::from_str::<Value>(trimmed) else { continue };
            if !matches!(row, Value::Object(_)) || !str_eq(vget(&row, "kind"), "pbi") {
                continue;
            }
            has_events = true;
            let id = match vget(&row, "id") {
                Some(Value::String(s)) if !s.is_empty() => s.clone(),
                _ => continue,
            };
            let event = vget(&row, "event").and_then(|v| v.as_str()).unwrap_or("");
            match event {
                "add" => {
                    if items.contains_key(&id) {
                        continue;
                    }
                    let status = match vget(&row, "status").and_then(|v| v.as_str()) {
                        Some(s) if PBI_STATUSES.contains(&s) => s.to_string(),
                        _ => "proposed".to_string(),
                    };
                    order.push(id.clone());
                    items.insert(id, status);
                }
                "status" => {
                    if let Some(item) = items.get_mut(&id) {
                        if let Some(s) = vget(&row, "status").and_then(|v| v.as_str()) {
                            if PBI_STATUSES.contains(&s) {
                                *item = s.to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if has_events {
        let mut counts = JMap::new();
        for status in PBI_STATUSES {
            counts.insert(token_key(status), json!(0));
        }
        let mut total = 0i64;
        for status in items.values() {
            if PBI_STATUSES.contains(&status.as_str()) {
                let key = token_key(status);
                let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                counts.insert(key, json!(n));
                total += 1;
            }
        }
        counts.insert("total".into(), json!(total));
        return Ok(Some(counts));
    }
    // legacyBacklogCounts over <productRoot>/docs/backlog.md.
    let product_root = resolve_product_root(ctx)?;
    let file = product_root.join("docs").join("backlog.md");
    let Some(text) = read_text_opt(&file) else {
        return Ok(None);
    };
    let mut counts = JMap::new();
    for status in BACKLOG_STATUSES {
        counts.insert(token_key(status), json!(0));
    }
    let normalize_status = |cell: &str| -> String {
        cell.chars()
            .filter(|c| !matches!(c, '*' | '`' | '_'))
            .collect::<String>()
            .trim()
            .to_lowercase()
    };
    let split_row = |line: &str| -> Vec<String> {
        let mut cells: Vec<String> = line.split('|').map(|c| js_trim(c).to_string()).collect();
        if cells.first().map(|c| c.is_empty()).unwrap_or(false) {
            cells.remove(0);
        }
        if cells.last().map(|c| c.is_empty()).unwrap_or(false) {
            cells.pop();
        }
        cells
    };
    let mut status_index: Option<usize> = None;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.contains('|') {
            continue;
        }
        let cells = split_row(line);
        match status_index {
            None => {
                if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                    status_index = Some(idx);
                }
            }
            Some(idx) => {
                if cells.len() <= idx {
                    continue;
                }
                let token = normalize_status(&cells[idx]);
                if BACKLOG_STATUSES.contains(&token.as_str()) {
                    let key = token_key(&token);
                    let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                    counts.insert(key, json!(n));
                }
            }
        }
    }
    let total: i64 = BACKLOG_STATUSES
        .iter()
        .map(|s| counts.get(&token_key(s)).and_then(|v| v.as_i64()).unwrap_or(0))
        .sum();
    counts.insert("total".into(), json!(total));
    Ok(Some(counts))
}

// ─── reviews (reviews.mjs) ─────────────────────────────────────────────────

/// reviews.mjs listReviews — fail-open per file. A corrupt session file now
/// warns twice, as Node did: readJson's could-not-parse line (our wording),
/// then the deterministic skip line, because readJson's `null` fallback lands
/// in the same non-object branch. A file that parses to a non-object prints
/// only the skip warning.
pub(crate) fn list_reviews(ctx: &mut Ctx) -> R<Vec<Value>> {
    let dir = ctx.root.join(".bee").join("reviews");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut sessions = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let session = rj(ctx, &entry.path())?;
        match session {
            Some(v @ Value::Object(_)) => sessions.push(v),
            _ => {
                ctx.warn(format!(
                    "reviews: skipping corrupt session file {name} (list stays fail-open)"
                ));
            }
        }
    }
    sessions.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(sessions)
}

/// reviews.mjs listCandidates.
pub(crate) fn list_candidates(ctx: &Ctx) -> Vec<Value> {
    read_jsonl(&ctx.root.join(".bee").join("review-candidates.jsonl"))
}

/// reviews.mjs sessionCoversCandidate. Property access on a null/undefined
/// candidate is the caller's throw (handled there).
pub(crate) fn session_covers_candidate(session: &Value, candidate: &Value) -> bool {
    let Some(Value::Array(included)) = vget(session, "included") else {
        return false;
    };
    let cand_feature = vget(candidate, "feature");
    let feature_match = included.iter().any(|e| {
        truthy(e) && str_eq(vget(e, "type"), "feature") && strict_eq(vget(e, "id"), cand_feature)
    });
    if feature_match {
        return true;
    }
    let cells: Vec<&Value> = match vget(candidate, "cells") {
        Some(Value::Array(c)) => c.iter().filter(|v| truthy(v)).collect(),
        _ => Vec::new(),
    };
    if cells.is_empty() {
        return false;
    }
    let included_cell_ids: Vec<Option<&Value>> = included
        .iter()
        .filter(|e| truthy(e) && str_eq(vget(e, "type"), "cell"))
        .map(|e| vget(e, "id"))
        .collect();
    cells
        .iter()
        .all(|id| included_cell_ids.iter().any(|iid| strict_eq(*iid, Some(id))))
}

/// reviews.mjs isSessionOpen.
pub(crate) fn is_session_open(session: &Value) -> bool {
    let decision = vget(session, "decision");
    !opt_truthy(decision) || !str_eq(decision.and_then(|d| vget(d, "status")), "approved")
}

pub(crate) enum GitAnswer {
    Covered(Option<bool>, bool), // (covered, unresolved)
    Since(Option<f64>, bool),    // (count, unresolved)
}

pub(crate) fn run_git(root: &Path, args: &[&str]) -> Option<(i32, String)> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    Some((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    ))
}

/// reviews.mjs headCoveredBy — non-string git args would throw in Node's
/// spawnSync (caught by buildReviewBlock) -> Ex::Thrown.
pub(crate) fn head_covered_by(
    root: &Path,
    head: Option<&Value>,
    ref_: Option<&Value>,
    memo: &mut HashMap<String, GitAnswer>,
) -> R<(Option<bool>, bool)> {
    if strict_eq(head, ref_) {
        return Ok((Some(true), false));
    }
    let key = format!("covered {} {}", tpl(head), tpl(ref_));
    if let Some(GitAnswer::Covered(c, u)) = memo.get(&key) {
        return Ok((*c, *u));
    }
    let (Some(Value::String(h)), Some(Value::String(r))) = (head, ref_) else {
        return Err(Ex::Thrown); // spawnSync TypeError on non-string args
    };
    let result = run_git(root, &["merge-base", "--is-ancestor", h, r]);
    let value = match result {
        Some((0, _)) => (Some(true), false),
        Some((1, _)) => (Some(false), false),
        _ => (None, true),
    };
    memo.insert(key, GitAnswer::Covered(value.0, value.1));
    Ok(value)
}

/// reviews.mjs commitsSince — `${ref}..HEAD` template-coerces.
pub(crate) fn commits_since(
    root: &Path,
    ref_: Option<&Value>,
    memo: &mut HashMap<String, GitAnswer>,
) -> (Option<f64>, bool) {
    let key = format!("since {}", tpl(ref_));
    if let Some(GitAnswer::Since(c, u)) = memo.get(&key) {
        return (*c, *u);
    }
    let range = format!("{}..HEAD", tpl(ref_));
    let result = run_git(root, &["rev-list", &range, "--count"]);
    let value = match result {
        Some((0, stdout)) => {
            // parseInt(trim, 10): leading integer prefix.
            let t = js_trim(&stdout);
            let digits: String = t
                .chars()
                .take_while(|c| c.is_ascii_digit() || (*c == '-' && t.starts_with('-')))
                .collect();
            match digits.parse::<f64>() {
                Ok(n) if n.is_finite() => (Some(n), false),
                _ => (None, true),
            }
        }
        _ => (None, true),
    };
    memo.insert(key, GitAnswer::Since(value.0, value.1));
    value
}

/// reviews.mjs deriveCandidateStatus. Returns (status, session_id, note).
pub(crate) fn derive_candidate_status(
    root: &Path,
    candidate: &Value,
    sessions: &[Value],
    memo: &mut HashMap<String, GitAnswer>,
) -> R<(String, Option<Value>, Option<String>)> {
    // Node would throw a TypeError on a null/undefined candidate at property
    // access; the caller (buildReviewBlock) catches — model as Thrown.
    if matches!(candidate, Value::Null) {
        return Err(Ex::Thrown);
    }
    let covering: Vec<&Value> = sessions
        .iter()
        .filter(|s| session_covers_candidate(s, candidate))
        .collect();
    let open: Vec<&&Value> = covering.iter().filter(|s| is_session_open(s)).collect();
    if !open.is_empty() {
        let session = open[open.len() - 1];
        return Ok(("in review".into(), vget(session, "id").cloned(), None));
    }
    let approved: Vec<&&Value> = covering.iter().filter(|s| !is_session_open(s)).collect();
    let mut unresolved_session: Option<&Value> = None;
    for session in &approved {
        let (covered, unresolved) =
            head_covered_by(root, vget(candidate, "head"), vget(session, "head"), memo)?;
        if unresolved {
            if unresolved_session.is_none() {
                unresolved_session = Some(session);
            }
            continue;
        }
        if covered != Some(true) {
            continue;
        }
        let (count, since_unresolved) = commits_since(root, vget(session, "head"), memo);
        if since_unresolved {
            return Ok((
                "review stale".into(),
                vget(session, "id").cloned(),
                Some("range unresolvable".into()),
            ));
        }
        if count.unwrap_or(0.0) > 0.0 {
            return Ok(("review stale".into(), vget(session, "id").cloned(), None));
        }
        return Ok(("reviewed".into(), vget(session, "id").cloned(), None));
    }
    if let Some(session) = unresolved_session {
        return Ok((
            "review stale".into(),
            vget(session, "id").cloned(),
            Some("range unresolvable".into()),
        ));
    }
    Ok(("unreviewed".into(), None, None))
}

/// bee.mjs buildReviewBlock — fail-open: a Thrown anywhere degrades; Bail
/// propagates (Node would have warned, we delegate).
pub(crate) fn build_review_block(ctx: &mut Ctx) -> R<JMap> {
    let empty = || -> JMap {
        let mut counts = JMap::new();
        counts.insert("total".into(), json!(0));
        counts.insert("unreviewed".into(), json!(0));
        counts.insert("in_review".into(), json!(0));
        counts.insert("reviewed".into(), json!(0));
        counts.insert("stale".into(), json!(0));
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Object(counts));
        m.insert("open_sessions".into(), json!([]));
        m.insert("high_risk_unreviewed".into(), json!(0));
        m
    };
    let attempt = |ctx: &mut Ctx| -> R<JMap> {
        let candidates = list_candidates(ctx);
        let sessions = list_reviews(ctx)?;
        let (mut unreviewed, mut in_review, mut reviewed, mut stale) = (0i64, 0i64, 0i64, 0i64);
        let mut high_risk_unreviewed = 0i64;
        let mut memo: HashMap<String, GitAnswer> = HashMap::new();
        for candidate in &candidates {
            let (status, _sid, _note) =
                derive_candidate_status(&ctx.root, candidate, &sessions, &mut memo)?;
            match status.as_str() {
                "unreviewed" => unreviewed += 1,
                "in review" => in_review += 1,
                "reviewed" => reviewed += 1,
                "review stale" => stale += 1,
                _ => {}
            }
            if truthy(candidate)
                && str_eq(vget(candidate, "mode"), "high-risk")
                && (status == "unreviewed" || status == "review stale")
            {
                high_risk_unreviewed += 1;
            }
        }
        let open_sessions: Vec<Value> = sessions
            .iter()
            .filter(|s| is_session_open(s))
            .map(|s| vget(s, "id").cloned().unwrap_or(Value::Null))
            .collect();
        let mut counts = JMap::new();
        counts.insert("total".into(), json!(candidates.len()));
        counts.insert("unreviewed".into(), json!(unreviewed));
        counts.insert("in_review".into(), json!(in_review));
        counts.insert("reviewed".into(), json!(reviewed));
        counts.insert("stale".into(), json!(stale));
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Object(counts));
        m.insert("open_sessions".into(), Value::Array(open_sessions));
        m.insert("high_risk_unreviewed".into(), json!(high_risk_unreviewed));
        Ok(m)
    };
    match attempt(ctx) {
        Ok(m) => Ok(m),
        Err(Ex::Thrown) => {
            let mut m = empty();
            m.insert("degraded".into(), json!(true));
            Ok(m)
        }
        Err(e) => Err(e),
    }
}
