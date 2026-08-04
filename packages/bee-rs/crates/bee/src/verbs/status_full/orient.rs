// orient, and the entry point both verbs share
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

// ─── orient (bee.mjs ~1229-1373) ───────────────────────────────────────────

/// bee.mjs orientNextCommand.
pub(crate) fn orient_next_command(status: &JMap, ready_ids: &[Value]) -> Value {
    if opt_truthy(status.get("handoff")) {
        return json!("bee state handoff show --json");
    }
    if !ready_ids.is_empty() {
        return json!("bee cells ready --json");
    }
    Value::Null
}

/// bee.mjs orientDecisionLine — first line, UTF-16 160-cap with '...'.
pub(crate) fn orient_decision_line(decision: Option<&Value>) -> String {
    let s = tpl(decision);
    let first = s.split('\n').next().unwrap_or("");
    let line = js_trim(first);
    let units: Vec<u16> = line.encode_utf16().collect();
    if units.len() > 160 {
        // slice(0, 157) over UTF-16 units; bee decision text is BMP so the
        // lossy re-decode is exact in practice.
        let sliced = String::from_utf16_lossy(&units[..157]);
        format!("{sliced}...")
    } else {
        line.to_string()
    }
}

/// bee.mjs orientWorktreeContext — BOTH halves. Inside a GRANTED worktree the
/// packet carries the merge-back state; from the MAIN checkout with a
/// code-touching active feature that already has a granted worktree, it
/// carries "go there". Null everywhere else, so an ordinary orient with no
/// granted worktree is byte-unchanged. Never throws (Thrown -> None like
/// Node's catch).
pub(crate) fn orient_worktree_context(ctx: &mut Ctx, status: &JMap) -> R<Option<JMap>> {
    let attempt = |ctx: &mut Ctx| -> R<Option<JMap>> {
        // grantedWorktreeContext(): the current checkout when it is a GRANTED
        // linked worktree (its own storeRoot === its own worktreeRoot).
        if let Some(linked) = ctx.granted_worktree() {
            let id = linked.id.clone();
            let branch = read_worktree_branch(&linked.main_root, &id);
            let mut m = JMap::new();
            m.insert("location".into(), json!("worktree"));
            m.insert("id".into(), json!(id.clone()));
            m.insert(
                "feature".into(),
                match status.get("feature") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            m.insert("branch".into(), branch.map(Value::String).unwrap_or(Value::Null));
            m.insert(
                "merge_command".into(),
                json!(format!("bee worktree merge --id {id}")),
            );
            return Ok(Some(m));
        }
        // `resolution.worktreeResolution !== 'ordinary'` — an UNGRANTED linked
        // worktree stops here (it is neither "go to the worktree" nor "you
        // are in one that owns the feature").
        if ctx.linked.is_some() {
            return Ok(None);
        }
        let feature = match status.get("feature") {
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        };
        let lane = match status.get("route") {
            Some(route) if truthy(route) => vget(route, "lane").cloned(),
            _ => Some(Value::Null),
        };
        let lane_ref = lane.as_ref();
        if !truthy(&feature) {
            return Ok(None);
        }
        let other_live = if str_eq(lane_ref, "tiny") {
            other_live_work_present(ctx)?
        } else {
            true
        };
        if !is_code_touching_lane(lane_ref, other_live) {
            return Ok(None);
        }
        let feature_str = match &feature {
            Value::String(s) => s.clone(),
            other => jsjson::js_to_string(other),
        };
        let root = ctx.root.clone();
        let Some((id, worktree_root)) = find_granted_worktree_for_feature(&root, &feature_str)
        else {
            return Ok(None);
        };
        let mut m = JMap::new();
        m.insert("location".into(), json!("main"));
        m.insert("id".into(), json!(id));
        m.insert("feature".into(), feature);
        m.insert("path".into(), json!(worktree_root.clone()));
        m.insert("guidance".into(), json!(format!("open your session at {worktree_root}")));
        Ok(Some(m))
    };
    match attempt(ctx) {
        Ok(v) => Ok(v),
        Err(Ex::Thrown) => Ok(None),
        Err(e) => Err(e),
    }
}

// ─── capture-queue blocker escalation (counter-teeth D2) ───────────────────
//
// docs/history/counter-teeth/CONTEXT.md D2: `bee orient` escalates the
// capture-queue line from an offer into `work.blockers[]` once the queue
// holds this many pending stubs, OR the oldest pending stub is older than
// this many days. Constants for this batch; config keys are future work
// (out of scope per CONTEXT.md's Open questions).
const CAPTURE_QUEUE_BLOCKER_MIN_PENDING: u64 = 10;
const CAPTURE_QUEUE_BLOCKER_MAX_AGE_DAYS: f64 = 7.0;

/// The oldest pending stub's `at`, in epoch ms, or NaN when there is none
/// or it can't be resolved. `cq`'s `ids` array is already sorted
/// oldest-first by `capture_queue_summary` (records.rs) — this looks the
/// first id back up in the raw queue purely to read its timestamp; the
/// membership itself (which ids are "pending" — stub rows minus flush
/// rows) is computed exactly once, in `capture_queue_summary`, and is
/// never re-derived here.
fn capture_queue_oldest_pending_at_ms(ctx: &Ctx, cq: &Value) -> f64 {
    let Some(Value::Array(ids)) = vget(cq, "ids") else {
        return f64::NAN;
    };
    let Some(oldest_id) = ids.first() else {
        return f64::NAN;
    };
    let events = read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl"));
    events
        .iter()
        .find(|e| str_eq(vget(e, "kind"), "stub") && strict_eq(vget(e, "id"), Some(oldest_id)))
        .map(|e| to_ms(vget(e, "at")))
        .unwrap_or(f64::NAN)
}

/// D2's escalation test over `status.capture_queue` (already the
/// stub-minus-flush pending count/ids from `capture_queue_summary`):
/// >= CAPTURE_QUEUE_BLOCKER_MIN_PENDING pending stubs, or the oldest
/// pending stub older than CAPTURE_QUEUE_BLOCKER_MAX_AGE_DAYS days.
/// Returns the blocker line, or None when the queue stays an offer.
fn capture_queue_blocker_line(ctx: &Ctx, cq: &Value) -> Option<String> {
    let count = vget(cq, "count").and_then(|v| v.as_u64()).unwrap_or(0);
    if count == 0 {
        return None;
    }
    let over_count = count >= CAPTURE_QUEUE_BLOCKER_MIN_PENDING;
    let oldest_ms = capture_queue_oldest_pending_at_ms(ctx, cq);
    let over_age = !oldest_ms.is_nan()
        && now_ms() - oldest_ms > CAPTURE_QUEUE_BLOCKER_MAX_AGE_DAYS * 24.0 * 60.0 * 60.0 * 1000.0;
    if !over_count && !over_age {
        return None;
    }
    Some(format!(
        "capture queue: {count} pending stub(s) — run bee-capturing to drain it (decision c2a7bd4f item 2)."
    ))
}

/// bee.mjs buildOrient.
pub(crate) fn build_orient(ctx: &mut Ctx) -> R<JMap> {
    let status = build_status(ctx, false)?;
    let feature = match status.get("feature") {
        None | Some(Value::Null) => Value::Null,
        Some(v) => v.clone(),
    };
    let context_md: Value = if truthy(&feature) {
        // path.join(root, 'docs', 'history', feature) — a non-string feature
        // would throw in Node's path.join -> bail (Node re-run reproduces).
        let Value::String(feature_str) = &feature else {
            return Err(Ex::Bail);
        };
        if ctx
            .root
            .join("docs")
            .join("history")
            .join(feature_str)
            .join("CONTEXT.md")
            .exists()
        {
            json!(format!("docs/history/{feature_str}/CONTEXT.md"))
        } else {
            Value::Null
        }
    } else {
        Value::Null
    };
    let feature_arg = if truthy(&feature) { Some(feature.clone()) } else { None };
    let ready_ids: Vec<Value> = ready_cells(ctx, feature_arg.as_ref())?
        .iter()
        .take(5)
        .map(|c| vget(c, "id").cloned().unwrap_or(Value::Null))
        .collect();
    let mut blockers: Vec<Value> = Vec::new();
    if opt_truthy(status.get("handoff")) {
        blockers.push(json!("pending handoff — surface it to the user and wait"));
    }
    let sd = status.get("scribing_debt");
    if opt_truthy(sd) && vget(sd.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
        blockers.push(json!(format!(
            "scribing debt: {} behavior_change cell(s) uncaptured",
            tpl(vget(sd.unwrap(), "count"))
        )));
    }
    if let Some(cq) = status.get("capture_queue") {
        if let Some(line) = capture_queue_blocker_line(ctx, cq) {
            blockers.push(json!(line));
        }
    }
    if let Some(Value::Array(warnings)) = status.get("staleness_warnings") {
        for warning in warnings {
            if let Value::String(w) = warning {
                if w.contains("reservation(s) expired") {
                    blockers.push(warning.clone());
                }
            }
        }
    }
    let worktree = orient_worktree_context(ctx, &status)?;

    let mut packet = JMap::new();
    {
        let mut where_ = JMap::new();
        where_.insert("phase".into(), status.get("phase").cloned().unwrap_or(Value::Null));
        where_.insert("feature".into(), feature.clone());
        where_.insert(
            "mode".into(),
            match status.get("mode") {
                None | Some(Value::Null) => Value::Null,
                Some(v) => v.clone(),
            },
        );
        where_.insert("gates".into(), status.get("gates").cloned().unwrap_or(Value::Null));
        where_.insert(
            "gate_bypass_level".into(),
            status.get("gate_bypass_level").cloned().unwrap_or(Value::Null),
        );
        packet.insert("where".into(), Value::Object(where_));
    }
    {
        let mut decisions = JMap::new();
        decisions.insert("context_md".into(), context_md);
        decisions.insert("active_count".into(), json!(active_decisions(ctx, None).len()));
        let recent: Vec<Value> = match status.get("recent_decisions") {
            Some(Value::Array(rows)) => rows
                .iter()
                .map(|d| json!(orient_decision_line(vget(d, "decision"))))
                .collect(),
            _ => Vec::new(),
        };
        decisions.insert("recent".into(), Value::Array(recent));
        packet.insert("decisions".into(), Value::Object(decisions));
    }
    {
        let cells = status.get("cells").cloned().unwrap_or(Value::Null);
        let mut work = JMap::new();
        let mut counts = JMap::new();
        counts.insert("open".into(), vget(&cells, "open").cloned().unwrap_or(Value::Null));
        counts.insert("claimed".into(), vget(&cells, "claimed").cloned().unwrap_or(Value::Null));
        counts.insert("capped".into(), vget(&cells, "capped").cloned().unwrap_or(Value::Null));
        work.insert("cells".into(), Value::Object(counts));
        work.insert("ready".into(), Value::Array(ready_ids.clone()));
        work.insert("blockers".into(), Value::Array(blockers));
        packet.insert("work".into(), Value::Object(work));
    }
    if let Some(worktree) = &worktree {
        packet.insert("worktree".into(), Value::Object(worktree.clone()));
    }
    {
        let mut next = JMap::new();
        next.insert(
            "action".into(),
            status.get("recommended_next").cloned().unwrap_or(Value::Null),
        );
        let skill = status
            .get("phase")
            .and_then(|v| v.as_str())
            .and_then(|p| {
                ORIENT_PHASE_SKILL
                    .iter()
                    .find(|(phase, _)| *phase == p)
                    .map(|(_, s)| *s)
            })
            .unwrap_or("bee-hive");
        next.insert("skill".into(), json!(skill));
        let command = match &worktree {
            Some(w) if str_eq(w.get("location"), "main") => {
                w.get("guidance").cloned().unwrap_or(Value::Null)
            }
            _ => orient_next_command(&status, &ready_ids),
        };
        next.insert("command".into(), command);
        packet.insert("next".into(), Value::Object(next));
    }
    Ok(packet)
}

/// bee.mjs renderOrientText — at most six lines plus the conditional
/// blockers/worktree lines.
pub(crate) fn render_orient_text(packet: &JMap) -> String {
    let where_ = packet.get("where").cloned().unwrap_or(Value::Null);
    let gates = GATE_NAMES
        .iter()
        .map(|g| {
            if opt_truthy(vget(&where_, "gates").and_then(|gs| vget(gs, g))) {
                "true"
            } else {
                "false"
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    let worktree = packet.get("worktree");
    let worktree_line: Option<String> = worktree.map(|w| {
        if str_eq(vget(w, "location"), "main") {
            format!(
                "worktree: feature \"{}\" lives in worktree {} — {}",
                tpl(vget(w, "feature")),
                tpl(vget(w, "id")),
                tpl(vget(w, "guidance"))
            )
        } else {
            let branch = if opt_truthy(vget(w, "branch")) {
                format!(" (branch {})", tpl(vget(w, "branch")))
            } else {
                String::new()
            };
            format!(
                "worktree: {}{branch} — merge back from main with {}",
                tpl(vget(w, "id")),
                tpl(vget(w, "merge_command"))
            )
        }
    });
    let decisions = packet.get("decisions").cloned().unwrap_or(Value::Null);
    let work = packet.get("work").cloned().unwrap_or(Value::Null);
    let next = packet.get("next").cloned().unwrap_or(Value::Null);
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "where: phase={} feature={} mode={} gates={gates} bypass={}",
        tpl(vget(&where_, "phase")),
        if nullish(vget(&where_, "feature")) { "none".to_string() } else { tpl(vget(&where_, "feature")) },
        if nullish(vget(&where_, "mode")) { "none".to_string() } else { tpl(vget(&where_, "mode")) },
        tpl(vget(&where_, "gate_bypass_level"))
    ));
    let context_part = if opt_truthy(vget(&decisions, "context_md")) {
        format!(" | context: {}", tpl(vget(&decisions, "context_md")))
    } else {
        String::new()
    };
    lines.push(format!(
        "decisions: {} active{context_part}",
        tpl(vget(&decisions, "active_count"))
    ));
    let cells = vget(&work, "cells").cloned().unwrap_or(Value::Null);
    let ready_part = match vget(&work, "ready") {
        Some(Value::Array(ready)) if !ready.is_empty() => {
            format!(" | ready: {}", js_join(ready, ", "))
        }
        _ => String::new(),
    };
    lines.push(format!(
        "work: open={} claimed={} capped={}{ready_part}",
        tpl(vget(&cells, "open")),
        tpl(vget(&cells, "claimed")),
        tpl(vget(&cells, "capped"))
    ));
    if let Some(Value::Array(blockers)) = vget(&work, "blockers") {
        if !blockers.is_empty() {
            lines.push(format!("blockers: {}", js_join(blockers, "; ")));
        }
    }
    if let Some(line) = worktree_line {
        lines.push(line);
    }
    lines.push(format!("skill: {}", tpl(vget(&next, "skill"))));
    lines.push(format!("next: {}", tpl(vget(&next, "action"))));
    lines.join("\n")
}

// ─── entry point ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) enum Verb {
    Status,
    Orient,
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    // STRANGLER ROUTING RULE: exactly these six shapes; any --brief presence
    // was already served upstream by status_brief; everything else -> None.
    let (verb, lanes_full, use_json) = match strs.as_slice() {
        ["status"] => (Verb::Status, false, false),
        ["status", "--json"] => (Verb::Status, false, true),
        ["status", "--lanes-full"] => (Verb::Status, true, false),
        ["status", "--lanes-full", "--json"] => (Verb::Status, true, true),
        ["orient"] => (Verb::Orient, false, false),
        ["orient", "--json"] => (Verb::Orient, false, true),
        _ => return None,
    };
    run(verb, lanes_full, use_json, t0)
}

pub(crate) fn run(verb: Verb, lanes_full: bool, use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let cmd = match verb {
        Verb::Status => "status",
        Verb::Orient => "orient",
    };
    // WORKTREE-NATIVE (see roots.rs's header): status/orient serve linked
    // worktrees themselves. A BROKEN link still delegates.
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, use_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, use_json, t0)),
    };
    let root = roots.root;
    // Drift check first (its cache write is the one permitted pre-bail side
    // effect — Node performs it before routing too).
    let drift = check_manifest_drift(&root).ok()?;
    let mut ctx = Ctx { root, cwd, linked: roots.linked, stderr: RefCell::new(Vec::new()) };
    let (payload, text) = match verb {
        Verb::Status => {
            let status = build_status(&mut ctx, lanes_full).ok()?;
            let text = render_status_text(&status);
            (Value::Object(status), text)
        }
        Verb::Orient => {
            let packet = build_orient(&mut ctx).ok()?;
            let text = render_orient_text(&packet);
            (Value::Object(packet), text)
        }
    };
    // Emission order (per stream): handler warnings, then the drift line on
    // stderr; the payload on stdout; the timing line last.
    for line in ctx.stderr.borrow().iter() {
        eprintln!("{line}");
    }
    if drift.manifest_changed {
        eprintln!("manifest_changed: true — {}", drift.hint);
    }
    if use_json {
        println!("{}", jsjson::stringify_pretty(&payload));
    } else {
        println!("{text}");
    }
    record_timing(&ctx.root, cmd, t0, true);
    Some(ExitCode::SUCCESS)
}
