// the contention summary, worktree lookups and the lane rows
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

// ─── contention summary (bee.mjs buildContentionSummary) ───────────────────

pub(crate) fn build_contention_summary(ctx: &Ctx) -> R<Option<JMap>> {
    let file = ctx.root.join(".bee").join("logs").join("contention.jsonl");
    let events = read_transcript_tail(&file, CONTENTION_TAIL_MAX_BYTES)?;
    if events.is_empty() {
        return Ok(None);
    }
    let busy: Vec<&Value> = events
        .iter()
        .filter(|e| {
            truthy(e)
                && matches!(e, Value::Object(_) | Value::Array(_))
                && str_eq(vget(e, "result"), "busy")
        })
        .collect();
    if busy.is_empty() {
        return Ok(None);
    }
    // byLock: Map insertion order = first-seen.
    let mut lock_order: Vec<String> = Vec::new();
    let mut lock_counts: HashMap<String, i64> = HashMap::new();
    for e in &busy {
        let name = match vget(e, "lock_name") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => "unknown".to_string(),
        };
        if !lock_counts.contains_key(&name) {
            lock_order.push(name.clone());
        }
        *lock_counts.entry(name).or_insert(0) += 1;
    }
    let mut ranked: Vec<(String, i64)> = lock_order
        .iter()
        .map(|n| (n.clone(), lock_counts[n]))
        .collect();
    // sort((a,b) => b[1]-a[1]) — stable, insertion order on ties.
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    let top_locks: Vec<Value> = ranked
        .into_iter()
        .take(CONTENTION_TOP_LOCKS_LIMIT)
        .map(|(lock_name, busy_count)| {
            let mut o = JMap::new();
            o.insert("lock_name".into(), json!(lock_name));
            o.insert("busy_count".into(), json!(busy_count));
            Value::Object(o)
        })
        .collect();
    let mut worst_wait_ms: Option<f64> = None;
    let mut worst_wait_lock: Value = Value::Null;
    for e in &events {
        if !truthy(e) || !matches!(e, Value::Object(_) | Value::Array(_)) {
            continue;
        }
        let wait = vget(e, "lock_wait_ms")
            .and_then(|v| if v.is_number() { v.as_f64() } else { None })
            .filter(|f| f.is_finite());
        if let Some(w) = wait {
            if worst_wait_ms.map(|m| w > m).unwrap_or(true) {
                worst_wait_ms = Some(w);
                worst_wait_lock = match vget(e, "lock_name") {
                    Some(Value::String(s)) if !s.is_empty() => json!(s),
                    _ => Value::Null,
                };
            }
        }
    }
    let recent_busy: Vec<Value> = busy
        .iter()
        .rev()
        .take(CONTENTION_RECENT_BUSY_LIMIT)
        .map(|e| {
            let mut o = JMap::new();
            o.insert(
                "ts".into(),
                match vget(e, "ts") {
                    Some(Value::String(s)) => json!(s),
                    _ => Value::Null,
                },
            );
            o.insert(
                "lock_name".into(),
                match vget(e, "lock_name") {
                    Some(Value::String(s)) => json!(s),
                    _ => Value::Null,
                },
            );
            // holder/caller: `e.x ?? null`.
            o.insert(
                "holder_session".into(),
                match vget(e, "holder_session") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            o.insert(
                "caller_session".into(),
                match vget(e, "caller_session") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            o.insert(
                "lock_wait_ms".into(),
                match vget(e, "lock_wait_ms") {
                    Some(v @ Value::Number(_)) => v.clone(),
                    _ => Value::Null,
                },
            );
            Value::Object(o)
        })
        .collect();
    let mut m = JMap::new();
    m.insert("busy_count".into(), json!(busy.len()));
    m.insert("recent_busy".into(), Value::Array(recent_busy));
    m.insert("top_locks".into(), Value::Array(top_locks));
    m.insert(
        "worst_wait_ms".into(),
        worst_wait_ms.map(json_num).unwrap_or(Value::Null),
    );
    m.insert("worst_wait_lock".into(), worst_wait_lock);
    Ok(Some(m))
}

// ─── worktree lookups (worktree-store.mjs / bee.mjs) ───────────────────────

/// worktree-store.mjs resolveWorktreeById — bidirectional gitdir validation.
pub(crate) fn resolve_worktree_by_id(main_root: &Path, id: &str) -> Option<String> {
    let git_worktree_dir = main_root.join(".git").join("worktrees").join(id);
    let meta = std::fs::metadata(&git_worktree_dir).ok()?;
    if !meta.is_dir() {
        return None;
    }
    let forward_raw = read_text_opt(&git_worktree_dir.join("gitdir"))?;
    let forward_raw = js_trim(&forward_raw);
    if forward_raw.is_empty() {
        return None;
    }
    let resolved_git_file = path_resolve(&git_worktree_dir, forward_raw);
    let worktree_root = path_dirname(&resolved_git_file);
    let reverse_raw = read_text_opt(Path::new(&worktree_root).join(".git").as_path())?;
    let reverse_raw = js_trim(&reverse_raw);
    // /^gitdir:\s*(.+)$/ over the single-line trimmed content.
    let rest = reverse_raw.strip_prefix("gitdir:")?;
    // /^gitdir:\s*(.+)$/ — \s* consumes any whitespace, then (.+)$ demands a
    // single newline-free remainder.
    let target = rest.trim_start();
    if target.is_empty() || target.contains('\n') {
        return None;
    }
    let reverse_resolved = path_resolve(Path::new(&worktree_root), js_trim(target));
    if !crate::path_identity::canonical_paths_equal(
        Path::new(&reverse_resolved),
        &git_worktree_dir,
    ) {
        return None;
    }
    Some(worktree_root)
}

/// worktree-store.mjs readWorktreeCreationFeature / readWorktreeStateFeature.
pub(crate) fn read_worktree_feature(worktree_root: &str) -> Option<String> {
    let read_feature = |file: PathBuf| -> Option<String> {
        let raw = read_text_opt(&file)?;
        let parsed = serde_json::from_str::<Value>(&raw).ok()?;
        match vget(&parsed, "feature") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        }
    };
    let base = Path::new(worktree_root).join(".bee");
    read_feature(base.join("runtime").join("worktree-identity.json"))
        .or_else(|| read_feature(base.join("state.json")))
}

/// worktree-store.mjs findGrantedWorktreeForFeature — never throws.
pub(crate) fn find_granted_worktree_for_feature(main_root: &Path, feature: &str) -> Option<(String, String)> {
    if feature.is_empty() {
        return None;
    }
    let grants = read_grants(&main_root.join(".bee"));
    for (id, granted) in &grants {
        if !matches!(granted, Value::Bool(true)) {
            continue;
        }
        let Some(worktree_root) = resolve_worktree_by_id(main_root, id) else {
            continue;
        };
        let identity = read_worktree_feature(&worktree_root);
        if identity.as_deref() == Some(feature) {
            return Some((id.clone(), worktree_root));
        }
    }
    None
}

/// bee.mjs ungrantedWorktreeNotice (GH #30) — messaging only, and the ONE
/// status field whose presence depends on the checkout's grant state.
///
/// Node re-resolves `process.cwd()` here rather than reading `root`, because
/// an UNGRANTED linked worktree's `root` already fell back to the main store
/// (the P40 default) and is therefore indistinguishable from an ordinary
/// checkout. `ctx.linked` is that same resolution. Emitted only when the
/// checkout is linked-valid AND storeRoot === mainRoot; an ordinary checkout
/// and a GRANTED worktree both omit the key entirely, byte-identical to the
/// pre-flip output. `null` from the throw arm is unreachable here: a
/// WorktreeLinkInvalidError already delegated the whole command.
pub(crate) const UNGRANTED_WORKTREE_NOTICE: &str = "⚠ This linked worktree is UNGRANTED — it SHARES the main checkout's store (same feature/phase/claims; no isolation). To work an isolated feature: run \"bee worktree new --feature <slug>\" from the main checkout. To grant isolation to THIS existing worktree instead: run \"bee worktree register --feature <slug>\" from inside it.";

pub(crate) fn ungranted_worktree_notice(ctx: &Ctx) -> Option<String> {
    let linked = ctx.linked.as_ref()?;
    if !linked.ungranted() {
        return None;
    }
    Some(UNGRANTED_WORKTREE_NOTICE.to_string())
}

/// bee.mjs readWorktreeBranch — best-effort branch for a granted worktree's
/// orient block. The linked worktree's HEAD lives at
/// `<mainRoot>/.git/worktrees/<id>/HEAD`; null on any failure or detached
/// HEAD. The Node regex is `/^ref:\s*refs\/heads\/(.+)$/` over the TRIMMED
/// file: `.` never matches a line terminator and there is no `m` flag, so the
/// captured branch must be non-empty and run to the very end of the string.
pub(crate) fn read_worktree_branch(main_root: &Path, id: &str) -> Option<String> {
    let head = read_text_opt(
        &main_root
            .join(".git")
            .join("worktrees")
            .join(id)
            .join("HEAD"),
    )?;
    let head = js_trim(&head);
    let rest = head.strip_prefix("ref:")?;
    // \s* — JS whitespace, the same class js_trim strips.
    let rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    let branch = rest.strip_prefix("refs/heads/")?;
    if branch.is_empty() || branch.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
        return None;
    }
    Some(branch.to_string())
}

/// bee.mjs isCodeTouchingLane.
pub(crate) fn is_code_touching_lane(lane: Option<&Value>, other_live_session: bool) -> bool {
    if !opt_truthy(lane) || str_eq(lane, "docs") {
        return false;
    }
    if str_eq(lane, "tiny") && !other_live_session {
        return false;
    }
    true
}

/// bee.mjs otherLiveWorkPresent — D9a live-session signal; fails quiet.
pub(crate) fn other_live_work_present(ctx: &mut Ctx) -> R<bool> {
    let attempt = |ctx: &mut Ctx| -> R<bool> {
        let ctrl_root = control_root_for(ctx)?;
        let self_id = resolve_session_id(ctx, Some(&ctrl_root))?;
        let others = active_workers(ctx, &ctrl_root, self_id.as_deref())?;
        if others.is_empty() {
            return Ok(false);
        }
        for worker in &others {
            let record: Option<JMap> = match worker.get("lane") {
                Some(v) if truthy(v) => read_lane(ctx, &jsjson::js_to_string(v))?,
                _ => Some(read_state_full(ctx)?),
            };
            let phase = record.as_ref().and_then(|r| r.get("phase"));
            if opt_truthy(phase)
                && !str_eq(phase, "idle")
                && !str_eq(phase, "compounding-complete")
            {
                return Ok(true);
            }
        }
        Ok(false)
    };
    match attempt(ctx) {
        Ok(v) => Ok(v),
        Err(Ex::Thrown) => Ok(false), // Node's own catch -> false
        Err(e) => Err(e),
    }
}

// ─── lane rows / summary (bee.mjs buildLaneRows / buildLaneSummary) ────────

/// bee.mjs buildLaneRows — every lane record plus its bound session ids.
/// NOTE: the sessions come through bee.mjs's LOCAL listSessionRecords wrapper
/// (bee.mjs:3986), which re-roots through controlRootFor (msn-18c) — one
/// extra readConfig (and its warnings) per call, unlike claims.mjs's own
/// bare-root listSessionRecords.
pub(crate) fn build_lane_rows(ctx: &mut Ctx) -> R<Vec<JMap>> {
    let lanes = list_lanes(ctx)?;
    let ctrl_root = control_root_for(ctx)?;
    let sessions = list_session_records(ctx, &ctrl_root)?;
    let mut bound_by: HashMap<String, Vec<Value>> = HashMap::new();
    for session in &sessions {
        if let Some(Value::String(lane)) = session.get("lane") {
            if !lane.is_empty() {
                bound_by
                    .entry(lane.clone())
                    .or_default()
                    .push(session.get("id").cloned().unwrap_or(Value::Null));
            }
        }
    }
    Ok(lanes
        .into_iter()
        .map(|mut lane| {
            let key = lane.get("feature").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let bound = bound_by.get(&key).cloned().unwrap_or_default();
            lane.insert("bound_sessions".into(), Value::Array(bound));
            lane
        })
        .collect())
}

/// bee.mjs buildLaneSummary (lpsp-2): active lane in full + counts/ids.
pub(crate) fn build_lane_summary(ctx: &mut Ctx) -> R<JMap> {
    let lanes = build_lane_rows(ctx)?;
    let mut out = JMap::new();
    if lanes.is_empty() {
        out.insert("active".into(), Value::Null);
        out.insert("counts".into(), json!({}));
        out.insert("ids".into(), json!([]));
        return Ok(out);
    }
    let ctrl_root = control_root_for(ctx)?;
    let session_id = resolve_session_id(ctx, Some(&ctrl_root))?;
    let mut active: Option<JMap> = None;
    if let Some(session_id) = session_id {
        if let Some(session) = read_session(ctx, &ctrl_root, &session_id)? {
            if let Some(Value::String(lane)) = session.get("lane") {
                if !lane.is_empty() {
                    active = lanes
                        .iter()
                        .find(|l| str_eq(l.get("feature"), lane))
                        .cloned();
                }
            }
        }
    }
    let active_feature = active
        .as_ref()
        .and_then(|a| a.get("feature"))
        .cloned();
    let rest: Vec<&JMap> = lanes
        .iter()
        .filter(|l| match (&active, &active_feature) {
            (Some(_), Some(f)) => !strict_eq(l.get("feature"), Some(f)),
            _ => true,
        })
        .collect();
    let mut counts = JMap::new();
    for l in &rest {
        let key = tpl(l.get("phase"));
        let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
        counts.insert(key, json!(n));
    }
    out.insert(
        "active".into(),
        active.map(Value::Object).unwrap_or(Value::Null),
    );
    out.insert("counts".into(), Value::Object(counts));
    out.insert(
        "ids".into(),
        Value::Array(rest.iter().map(|l| l.get("feature").cloned().unwrap_or(Value::Null)).collect()),
    );
    Ok(out)
}
