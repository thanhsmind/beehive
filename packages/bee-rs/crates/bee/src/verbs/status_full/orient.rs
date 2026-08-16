// orient, and the entry point both verbs share
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::textutil::{char_len, truncate_chars_head};
// Aliased (not `use crate::verbs::cells::*`): `use super::*` above already
// binds the name `cells` to THIS module's own sibling `status_full::cells`
// (verbs/status_full/cells.rs) — a different module entirely. sweep-at-
// every-door's D1/D6 door reuses the crate::verbs::cells sweep functions
// without unifying the two crates' error types (cells::MR<T> = Result<T,
// Fail>; status_full::R<T> = Result<T, Ex> just below) — see
// `bridge_sweep_fail`.
use crate::verbs::cells as sweep_cells;
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

/// bee.mjs orientDecisionLine — first line, 160-CHAR cap with '...' (decision
/// D3: char-based, not the historical UTF-16-unit count).
pub(crate) fn orient_decision_line(decision: Option<&Value>) -> String {
    let s = tpl(decision);
    let first = s.split('\n').next().unwrap_or("");
    let line = js_trim(first);
    if char_len(line) > 160 {
        format!("{}...", truncate_chars_head(line, 157))
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

/// D2's trigger-registry surfacing (`verbs/triggers`): a due predicate
/// trigger or an unresolved manual trigger is a blocker — the whole point
/// of the registry is that a deferred condition cannot sink silently.
/// `None` when nothing is due and nothing awaits confirmation.
fn trigger_queue_blocker_line(control: &Path) -> Option<String> {
    let (due, awaiting) = crate::verbs::triggers::due_and_manual_counts(control);
    if due == 0 && awaiting == 0 {
        return None;
    }
    Some(format!("{due} trigger(s) due, {awaiting} awaiting confirmation"))
}

// ─── sweep door (sweep-at-every-door D1/D6) ────────────────────────────────
//
// `bee cells claim-next` (handlers_select.rs:620) and `bee orient` (below)
// were the sweep's only production callers until `bee recovery scan`
// (`recovery_verb.rs`) became a third (sweep-recovery-door D7/srd-2) —
// `bee status` stays report-only, no other verb gains one.

/// Bridges `sweep_cells::Fail` (`MR<T> = Result<T, Fail>`) into this module's
/// own `Ex` (`R<T> = Result<T, Ex>`, mod.rs:157) rather than unifying the two
/// error types. `Ex` carries no message (mod.rs's error-plumbing note: `Bail`
/// = JS-exotic input, `Thrown` = a caught-or-escaping exception) — both
/// `Fail::Delegate` (JS-exotic claim/session data) and `Fail::Thrown` (an I/O
/// failure inside the sweep's own write) become `Ex::Thrown`, the same
/// "an exception escaped the attempt" outcome `orient_worktree_context`'s
/// fail-open wrapper already uses for everything unexpected.
fn bridge_sweep_fail(_: sweep_cells::Fail) -> Ex {
    Ex::Thrown
}

/// D6 preview: counts, without removing, the claims `sweep_expired_claims`
/// would take — TTL expired AND owning session heartbeat-stale — with no
/// `.adopting` gate ever acquired. Used only on the decline path below, when
/// `bee orient` cannot resolve its own caller session: it reports what it
/// would have swept instead of sweeping with no self-exclusion, which would
/// defeat D6 in exactly the multi-agent case this feature targets.
fn count_expired_claims(control: &Path, now: f64) -> sweep_cells::MR<usize> {
    let dir = sweep_cells::claims_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(0) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    let mut count = 0usize;
    for entry in names {
        let Some(cell) = entry.strip_suffix(".json") else { continue };
        let Some(claim) = sweep_cells::read_claim(control, cell)? else { continue };
        if !sweep_cells::claim_expired(&claim, now)? {
            continue;
        }
        if !sweep_cells::heartbeat_stale(sweep_cells::read_session_of_claim(control, &claim)?.as_ref(), now)? {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

/// `bee orient`'s sweep door (D1) — the second call site added to the
/// existing sweep, `bee cells claim-next`'s (`handlers_select.rs:620`) being
/// first; `bee recovery scan` (`recovery_verb.rs`) is now a third
/// (sweep-recovery-door D7/srd-2). Resolves orient's OWN caller session
/// exactly the way `claim-next`
/// does: `resolve_session_flag_env` first (`orient` takes no `--session-id`
/// flag — `orient.rs`'s six-shape arg match gains none — so the `flag`
/// argument is always `None`, leaving the `BEE_SESSION_ID` /
/// `CLAUDE_CODE_SESSION_ID` env chain), then the durable single-live-session
/// fallback `resolve_session_adopt`. The resolved id is passed to
/// `sweep_expired_claims` as `caller_session`, so orient's own caller's claim
/// is never swept (D6), no matter how stale its TTL or heartbeat read.
///
/// D6: when NEITHER resolves — precisely the multi-agent case
/// `resolve_session_adopt` returns `None` for by construction — this
/// declines to sweep at all rather than sweep anonymously, and returns a
/// blocker line naming the count of claims that qualified, untouched.
///
/// Called from `build_orient` BEFORE it reads cell counts (`build_status`),
/// so a claim this pass frees — and the cell it parks `blocked` — shows up
/// in the SAME orient packet, not a follow-up call. `bee status` never calls
/// this function.
fn sweep_on_orient(ctx: &Ctx) -> R<Option<String>> {
    let control = sweep_cells::control_root(&ctx.root).map_err(bridge_sweep_fail)?;
    let now = now_ms();
    // D3 (dirty-main-conflicts dmc-4): reservations get the same sweep door
    // claims already have, so a dead session's stale lease is the rare,
    // cleared-away case rather than the normal thing a new reserve on the
    // same path has to take over. Expiry-only — unlike the claim sweep below,
    // no caller-session exclusion is needed: an expired lease is expired
    // regardless of who is asking.
    crate::lease_store::sweep_expired_leases(&control, now);
    let caller = sweep_cells::resolve_session_flag_env(None)
        .or_else(|| sweep_cells::resolve_session_adopt(&control).ok().flatten());
    let Some(caller) = caller else {
        let count = count_expired_claims(&control, now).map_err(bridge_sweep_fail)?;
        if count == 0 {
            return Ok(None);
        }
        return Ok(Some(format!(
            "sweep declined: {count} expired claim(s) detected, but bee orient could not resolve its own caller session (BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID unset, and more than one live session exists to adopt) — set BEE_SESSION_ID and run bee recovery scan from an identified session to release them."
        )));
    };
    sweep_cells::sweep_expired_claims(&control, now, Some(caller.as_str())).map_err(bridge_sweep_fail)?;
    Ok(None)
}

/// bee.mjs buildOrient.
pub(crate) fn build_orient(ctx: &mut Ctx) -> R<JMap> {
    // D1 (sweep-at-every-door): orient's own sweep door, before anything
    // below reads cell counts — see `sweep_on_orient`'s header.
    let sweep_blocker = sweep_on_orient(ctx)?;
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
    if let Some(line) = sweep_blocker {
        blockers.push(json!(line));
    }
    if opt_truthy(status.get("handoff")) {
        blockers.push(json!("pending handoff — surface it to the user and wait"));
    }
    // D1 (awaiting-human): a live wait is a BLOCKER here — the whole point
    // of `bee orient` is that a reader routing work can tell the run is
    // stopped on a person rather than running. Named after the handoff
    // blocker just above (a handoff is also a wait on the human, just a
    // different shape of one) and before the report-only lines below.
    if opt_truthy(status.get("waiting_on")) {
        let w = status.get("waiting_on").unwrap();
        blockers.push(json!(format!(
            "awaiting human — {}: {}",
            tpl(vget(w, "kind")),
            tpl(vget(w, "subject")),
        )));
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
    // D2 (knowledge-distill-trigger C2): same control-root resolution
    // sweep_on_orient already performed above — a deferred condition's
    // registry entry surfaces here the moment it is due or awaiting a
    // human's confirmation.
    if let Ok(control) = sweep_cells::control_root(&ctx.root) {
        if let Some(line) = trigger_queue_blocker_line(&control) {
            blockers.push(json!(line));
        }
    }
    // kf-2 (D3): report-only, same voice as scribing debt / capture queue
    // just above — never gates a phase or changes an exit code.
    let proposals = unapplied_promote_proposals(&ctx.root);
    if !proposals.is_empty() {
        let named = proposals
            .iter()
            .map(|p| format!("{} ({}, {})", p.feature, p.counts, p.path))
            .collect::<Vec<_>>()
            .join("; ");
        blockers.push(json!(format!(
            "promote proposal unapplied: {} feature(s) — {named}",
            proposals.len()
        )));
    }
    // D4 (D4a): a granted worktree `bee worktree merge` never reached and
    // nobody pruned — same report-only voice as the promote proposal just
    // above, reading the ONE scan (`reclaimable_worktree_ids`) the session
    // preamble reads too. No git call, no size walk (D4a): `bee orient`
    // never pays that cost.
    let reclaimable = reclaimable_worktree_ids(&ctx.root);
    if reclaimable.len() > RECLAIMABLE_WORKTREES_SHOWN_FLOOR {
        blockers.push(json!(format!(
            "{} worktree(s) reclaimable — merged, clean, and idle past the age threshold: run `bee worktree prune --dry-run` to see what it would remove.",
            reclaimable.len()
        )));
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
        // D1 (awaiting-human): the structured mark beside the text blocker
        // above — a reader that wants the subject/kind without parsing the
        // blocker string reads it here, same additive shape `status` itself
        // carries it in.
        where_.insert("waiting_on".into(), status.get("waiting_on").cloned().unwrap_or(Value::Null));
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
    // ROUTING RULE: exactly these six shapes; any --brief presence was
    // already served upstream by status_brief; everything else -> None.
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
    let drift = check_manifest_drift(&root);
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
