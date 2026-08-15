// verb handler: claim-next — the selection half and the sweep
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
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── cells claim-next (R6 coverage debt — the SELECTION half + the sweep) ──
//
// Provenance: bee.mjs handleCellsClaimNext, lib/cells.mjs claimNextCell /
// resolveHoldTopology, lib/claims.mjs sweepExpiredClaims, lib/state.mjs
// resolvePipeline / applyWritePolicy, lib/reservations.mjs
// findSessionConflicts, lib/worktree-holds.mjs findForeignHolds /
// isActive / isExpired, lib/backlog.mjs featureBacklogRank (ported in
// verbs/backlog.rs and imported here).
//
// WHY THE SWEEP IS SAFE TO RUN NATIVELY. sweepExpiredClaims mutates (claim
// files removed, claimed->blocked cell verdicts — D4 — one decision row per
// verdict) BEFORE selection reads anything, so the usual "return None and let Node
// re-run" escape would ordinarily double-write. It does not here, because the
// sweep removes its own trigger: every row it writes is gated on a claim FILE
// that it then deletes, so a Node re-run finds `readClaim` null for exactly
// those cells and writes nothing a second time. The pre-scan below still
// front-loads every delegation trigger it can, so a mid-flight delegate is a
// concurrent-writer race, not a routine path — but when one does happen the
// end state and the emitted bytes are Node's own.
//
// applyWritePolicy is a NO-OP for this verb by construction: claim-next
// passes `paths: []` and `enforceIsolation: false`, so 'observe' returns
// immediately, 'shared-disjoint' short-circuits on the empty declared list,
// and 'isolated' takes the `!enforceIsolation` passthrough. `policy.redirect`
// can never be true, so the redirect branch of handleCellsClaimNext is
// unreachable — only readConfig's own corrupt-file delegation survives.
//
// Root topology: rsv::prelude serves ORDINARY checkouts only, so
// resolveHoldTopology(root) is the constant `{mainRoot: root, holder:'main'}`
// (the same constant verbs/reservations.rs already documents), and
// controlRootFor(root) === root.

/// lib/claims.mjs sweepExpiredClaims (hardening-4b sweep-reset, rel180-2;
/// sweep-at-every-door D4/D5/D6). TTL expired AND owner heartbeat stale, both
/// re-verified under the claim's exclusive `<cell>.adopting` gate and — for a
/// session-owned claim — under the same `sessions` store lock
/// heartbeatSession itself holds.
///
/// `caller_session` (D6) is checked FIRST, before the gate is ever acquired:
/// a claim whose own `session` field equals it is never swept, no matter how
/// stale its TTL or heartbeat read — a live session mid a long tool call must
/// never lose its own claim to its own sweep. `None` excludes nothing (a
/// caller that cannot resolve its own identity is expected to decline to
/// call this function at all, per D6 — that is each door's own decision).
///
/// Every removal is followed by the claimed->blocked verdict (D4) under
/// `cells:<id>` when the cell is readable in THIS store, or — per D5, the
/// sweep never writes across a store boundary — left untouched with the
/// cell id and its worktree named on stderr and in a decision row when it is
/// not. Either way, one best-effort decision row per removal.
///
/// The three sets a caller needs to report what the sweep did (srd-1,
/// sweep-recovery-door): `released` is every cell id whose claim file was
/// actually removed, regardless of what the claimed->blocked verdict then
/// found; `parked` is the subset the verdict reset to `blocked` (D4,
/// `SweepResetOutcome::Blocked`); `unreachable` is the subset whose cell
/// record could not be read in this store (D5, `SweepResetOutcome::
/// Unreachable`). A `released` id absent from both `parked` and
/// `unreachable` was left untouched by the verdict — a fresher claim already
/// owned the cell (`SweepResetOutcome::Untouched`) — exactly as before this
/// cell, silently and without a decision row.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SweepSummary {
    pub(crate) released: std::collections::BTreeSet<String>,
    pub(crate) parked: std::collections::BTreeSet<String>,
    pub(crate) unreachable: std::collections::BTreeSet<String>,
}

pub(crate) fn sweep_expired_claims(
    control: &Path,
    now: f64,
    caller_session: Option<&str>,
) -> MR<SweepSummary> {
    let mut summary = SweepSummary::default();
    let dir = claims_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(summary) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    for entry in names {
        let Some(cell) = entry.strip_suffix(".json") else { continue };
        let Some(preview) = read_claim(control, cell)? else { continue }; // corrupt: never touch
        // D6 self-exclusion — before ANY other gate (claim_expired,
        // heartbeat, .adopting).
        if let Some(caller) = caller_session {
            if matches!(preview.get("session"), Some(Value::String(s)) if s == caller) {
                continue;
            }
        }
        if !claim_expired(&preview, now)? {
            continue;
        }
        if !heartbeat_stale(read_session_of_claim(control, &preview)?.as_ref(), now)? {
            continue;
        }
        if !acquire_gate(control, cell)? {
            continue; // gate held by another in-flight adopt/sweep — skipped
        }
        let mut swept_claim: Option<Map<String, Value>> = None;
        let gated = (|| -> MR<()> {
            let Some(claim) = read_claim(control, cell)? else { return Ok(()) };
            if !claim_expired(&claim, now)? {
                return Ok(());
            }
            // `claim.session ?? null`; a sessionless claim has no heartbeat to
            // race against and skips the lock entirely (rel180-2).
            let owner_session = nullish(claim.get("session"));
            let _sessions_lock = if js_truthy(&owner_session) {
                match acquire_sessions_lock_bounded(control) {
                    Some(guard) => Some(guard),
                    None => return Ok(()), // never steal on contention — skipped
                }
            } else {
                None
            };
            if heartbeat_stale(read_session_of_claim(control, &claim)?.as_ref(), now)? {
                let file = claim_path(control, cell)?;
                let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    other => other,
                });
                swept_claim = Some(claim);
            }
            Ok(())
        })();
        release_gate(control, cell);
        gated?;
        let Some(swept) = swept_claim else { continue };
        summary.released.insert(cell.to_string());
        let swept_session = nullish(swept.get("session"));
        let owner_disp = owner_display(&swept_session);
        // Best-effort below: the claim file above is already gone either
        // way, so a decision-log or stderr failure must never read as the
        // sweep itself having failed.
        match sweep_reset_cell(control, cell, &swept, now)? {
            SweepResetOutcome::Blocked => {
                summary.parked.insert(cell.to_string());
                let _ = log_decision(
                    control,
                    &format!(
                        "\u{ab}sweep: cell \"{cell}\" reset claimed -> blocked \u{2014} swept session \"{owner_disp}\"'s expired, stale claim\u{bb}"
                    ),
                    "sweepExpiredClaims (D4, sweep-at-every-door) removed the abandoned claim file; the cell was still \"claimed\" by that exact session (trace.claim_session matched), so it is parked \"blocked\" — trace.blocked_reason names the dead session and its worktree — rather than reopened, which would invite the next agent to redo half-finished work blind.",
                    &["claims", "sweep"],
                );
            }
            SweepResetOutcome::Unreachable => {
                summary.unreachable.insert(cell.to_string());
                let worktree = worktree_clause(control, &swept);
                eprintln!(
                    "sweep: cell \"{cell}\"'s claim was removed, but its cell record is not readable in this store — its half-finished work, if any, may be at {worktree}. Run \"bee cells reopen\" from a session that can reach it, once its store is reachable."
                );
                let _ = log_decision(
                    control,
                    &format!(
                        "\u{ab}sweep: cell \"{cell}\"'s claim removed, cell left untouched \u{2014} not readable in this store \u{2014} swept session \"{owner_disp}\"'s expired, stale claim; its worktree may be {worktree}\u{bb}"
                    ),
                    "sweepExpiredClaims (D5, sweep-at-every-door) never writes a cell record across a store boundary: the claim is control-plane and freed here, but the cell record itself lives in a store this process cannot read (most likely a granted worktree's own .bee/cells), so parking it \"blocked\" is left to a session — or a human — that can reach it.",
                    &["claims", "sweep"],
                );
            }
            SweepResetOutcome::Untouched => {} // status/claim_session mismatch — a fresher claim already owns it, silently
        }
    }
    Ok(summary)
}

/// `claim.session ?? null` rendered for a decision row or a blocked reason:
/// `"none (sessionless)"` for a null session (rel180-2's own spelling,
/// pinned by `sweep_of_a_sessionless_claim_names_none_in_its_decision_row`),
/// otherwise the session id's JS string coercion.
pub(crate) fn owner_display(session: &Value) -> String {
    if session.is_null() {
        "none (sessionless)".to_string()
    } else {
        jsjson::js_to_string(session)
    }
}

/// The worktree half of D4's blocked reason and D5's unreachable report,
/// resolved from the claim's OPTIONAL `workspace_id` (auto-looked-up at
/// claim time, claims.rs:692-711) through the workspace registry
/// (`workspace_store::read_workspace`) to its `root`. Every arm returns a
/// full clause naming what is and is not known — never silently omitted, per
/// the feature's "Agent's Discretion" note on the blocked-reason wording.
pub(crate) fn worktree_clause(control: &Path, claim: &Map<String, Value>) -> String {
    let workspace_id = match claim.get("workspace_id") {
        Some(Value::String(w)) if !w.is_empty() => w.clone(),
        _ => return "an unknown worktree (the claim carries no workspace_id)".to_string(),
    };
    match ws::read_workspace(control, &workspace_id) {
        Ok(record) => match record.get("root") {
            Some(Value::String(r)) if !r.is_empty() => {
                format!("worktree \"{r}\" (workspace \"{workspace_id}\")")
            }
            _ => format!("an unknown worktree (workspace \"{workspace_id}\" has no root recorded)"),
        },
        Err(e) => format!(
            "an unknown worktree (workspace \"{workspace_id}\" has no readable record: {})",
            e.message()
        ),
    }
}

/// `value ?? null` for an optional JSON field (undefined AND null collapse).
pub(crate) fn nullish(v: Option<&Value>) -> Value {
    match v {
        None | Some(Value::Null) => Value::Null,
        Some(other) => other.clone(),
    }
}

/// `readSession(root, claim.session)` — a non-string session makes
/// sessionPath's requireId throw, which readSession catches as "no session".
pub(crate) fn read_session_of_claim(
    control: &Path,
    claim: &Map<String, Value>,
) -> MR<Option<Map<String, Value>>> {
    match claim.get("session") {
        Some(Value::String(s)) => read_session(control, s),
        _ => Ok(None),
    }
}

/// claims.mjs SESSIONS_LOCK_NAME bounded acquire (15 × 20ms, acquire-once) —
/// the exact `sessions` lock heartbeatSession/bindSessionLane hold.
pub(crate) fn acquire_sessions_lock_bounded(root: &Path) -> Option<lock::LockGuard> {
    for attempt in 0..15u32 {
        match lock::acquire_store_lock_once(root, "sessions") {
            lock::AcquireOnce::Acquired(guard) => return Some(guard),
            lock::AcquireOnce::Busy { .. } => {
                if attempt + 1 < 15 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }
    None
}

/// The result of the sweep's per-cell verdict (D4/D5).
pub(crate) enum SweepResetOutcome {
    /// The cell was readable here, still `claimed` by the swept session, and
    /// is now `blocked` with `trace.blocked_reason` naming the dead session
    /// and its worktree.
    Blocked,
    /// The cell record exists in this store but does not match (not
    /// `claimed`, or a fresher claim already owns it) — left exactly as
    /// found. Pre-existing, silent behavior, unrelated to the store-boundary
    /// case below.
    Untouched,
    /// D5: the cell record is not readable in THIS store — most likely a
    /// granted worktree's own `.bee/cells` holds it instead. The claim is
    /// already gone (the caller removed it before calling this function);
    /// nothing is written here.
    Unreachable,
}

/// The sweep's claimed->blocked verdict (D4), under the SAME `cells:<id>`
/// store lock every other cells.mjs mutator uses. `readCellForSweepReset` is
/// claims.mjs's own minimal `.bee/cells/<id>.json` read/write (never
/// cells.mjs's readCell — that would cycle), so it never consults the
/// archive — and, per D5, it never reaches past THIS store's own cells
/// directory to find one.
pub(crate) fn sweep_reset_cell(
    control: &Path,
    cell: &str,
    swept_claim: &Map<String, Value>,
    now: f64,
) -> MR<SweepResetOutcome> {
    let swept_session = nullish(swept_claim.get("session"));
    let mut guard = acquire_named_lock(control, &format!("cells:{cell}"))?;
    let outcome = (|| -> MR<SweepResetOutcome> {
        let file = cells_dir(control).join(format!("{cell}.json"));
        let record = match read_store_json(&file)? {
            Some(Value::Object(m)) => m,
            _ => return Ok(SweepResetOutcome::Unreachable), // D5: not readable in this store
        };
        if !matches!(record.get("status"), Some(Value::String(s)) if s == "claimed") {
            return Ok(SweepResetOutcome::Untouched);
        }
        // `(cellRecord.trace && cellRecord.trace.claim_session) ?? null`
        let current_session = match record.get("trace") {
            None => Value::Null,
            Some(t) if !js_truthy(t) => nullish(Some(t)),
            Some(Value::Object(t)) => nullish(t.get("claim_session")),
            Some(_) => Value::Null, // truthy non-object: .claim_session is undefined
        };
        if current_session != swept_session {
            return Ok(SweepResetOutcome::Untouched); // a fresher claim already owns it
        }
        let mut record = record;
        record.insert("status".into(), Value::String("blocked".into()));
        // `{ ...(cellRecord.trace || {}), worker: null, claimed_at: null,
        //    claim_session: null, swept_at, swept_from_session, blocked_reason }`
        let mut trace = Map::new();
        if let Some(Value::Object(old)) = record.get("trace") {
            spread_into(&mut trace, old);
        }
        trace.insert("worker".into(), Value::Null);
        trace.insert("claimed_at".into(), Value::Null);
        trace.insert("claim_session".into(), Value::Null);
        trace.insert(
            "swept_at".into(),
            Value::String(rsv::iso_from_ms(now).map_err(|_| Fail::Delegate)?),
        );
        trace.insert("swept_from_session".into(), swept_session.clone());
        trace.insert(
            "blocked_reason".into(),
            Value::String(format!(
                "swept by bee: session \"{}\"'s claim on this cell expired and its heartbeat went stale; its half-finished work, if any, may be at {}.",
                owner_display(&swept_session),
                worktree_clause(control, swept_claim)
            )),
        );
        record.insert("trace".into(), Value::Object(trace));
        let value = Value::Object(record);
        transient_fs_retry(|| crate::fsutil::write_json_atomic(&file, &value))
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
        Ok(SweepResetOutcome::Blocked)
    })();
    guard.release();
    outcome
}

/// GH#20 + D1 (default-pipeline-liveness, docs/history/default-pipeline-liveness):
/// the SINGLE session-record walk `claimNextCell`'s fallback pool needs,
/// computing both facts liveness protects from one read with one error
/// propagation:
///   - `live_owned`: lanes actively bound to another LIVE session (GH#20) —
///     never pooled as a fallback candidate lane.
///   - the returned `bool`: whether some OTHER live session is unbound. An
///     unbound session record is, by definition, working the default
///     pipeline (`.bee/state.json`) right now, so while one exists the
///     default pipeline itself must not be pushed into the fallback pool.
/// A record whose heartbeat is stale (`heartbeat_stale`) counts as neither —
/// a dead session must never park work forever. The acting session's own
/// record is skipped outright: it can never be its own peer (D2), and the
/// existing `own_feature` comparisons at each push site already keep an
/// unbound acting session claiming its own default pipeline unblocked.
pub(crate) fn live_session_facts(
    control: &Path,
    session: &str,
    now: f64,
) -> MR<(Vec<String>, bool)> {
    let mut live_owned: Vec<String> = Vec::new();
    let mut live_unbound_peer = false;
    for record in list_session_records(control)? {
        if matches!(record.get("id"), Some(Value::String(s)) if *s == session) {
            continue;
        }
        if heartbeat_stale(Some(&record), now)? {
            continue;
        }
        let bound = match record.get("lane") {
            Some(Value::String(l)) => js_trim(l).to_string(),
            _ => String::new(),
        };
        if bound.is_empty() {
            live_unbound_peer = true;
        } else {
            live_owned.push(bound);
        }
    }
    Ok((live_owned, live_unbound_peer))
}

/// lib/state.mjs resolvePipeline's answer, reduced to the two fields
/// claimNextCell consumes: `resolved.record.feature || null` and
/// `gateApproved(resolved.record, 'execution')`.
pub(crate) enum Pipeline {
    Ok { feature: Option<String>, execution_approved: bool },
    Refused { code: &'static str, reason: String },
}

/// lib/state.mjs resolvePipeline — session record → bound lane → default
/// state.json, with the four typed refusals. Sessions and lanes are
/// control-plane (msn-18a); the default record stays on the caller's own root.
pub(crate) fn resolve_pipeline(root: &Path, control: &Path, session_id: &str) -> MR<Pipeline> {
    let defaults = || -> MR<Pipeline> {
        let state = bstate::read_state_brief(root);
        Ok(Pipeline::Ok {
            feature: match &state.feature {
                v if js_truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
            execution_approved: matches!(state.gates.get("execution"), Some(Value::Bool(true))),
        })
    };
    if js_trim(session_id).is_empty() {
        return defaults();
    }
    let Some(session) = read_session(control, session_id)? else { return defaults() };
    let bound = match session.get("lane") {
        Some(Value::String(l)) => js_trim(l).to_string(),
        _ => String::new(),
    };
    if bound.is_empty() {
        return defaults();
    }
    let session_disp = js_string_or_undefined(session.get("id"));
    // lanePath's requireLaneFeature throw → LANE_INVALID, message embedded.
    let Some(lane_id) = lane_feature_ok(&bound) else {
        let detail = if js_trim(&bound).is_empty() {
            "lane feature is required."
        } else {
            "lane feature must be a plain id (no path separators)."
        };
        return Ok(Pipeline::Refused {
            code: "LANE_INVALID",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\", which is not a valid lane name ({detail}) \u{2014} never guessed back to the default pipeline. FIX: rebind or unbind the session (claims bindSessionLane/unbindSessionLane)."
            ),
        });
    };
    let file = lanes_dir(control).join(format!("{lane_id}.json"));
    let rel = lane_rel_path(&lane_id);
    if !file.exists() {
        return Ok(Pipeline::Refused {
            code: "LANE_MISSING",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\" but {rel} does not exist \u{2014} resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session."
            ),
        });
    }
    let record = crate::verbs::workflow_store::read_lane_display(control, &bound)?;
    let Some(record) = record else {
        return Ok(Pipeline::Refused {
            code: "LANE_CORRUPT",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\" but its record is corrupt \u{2014} display never guesses and mutations must refuse. FIX: inspect/restore {rel}, then retry."
            ),
        });
    };
    let approved = record
        .get("approved_gates")
        .and_then(|g| g.get("execution"))
        .map(|v| matches!(v, Value::Bool(true)))
        .unwrap_or(false);
    Ok(Pipeline::Ok {
        feature: match record.get("feature") {
            Some(v) if js_truthy(v) => Some(jsjson::js_to_string(v)),
            _ => None,
        },
        execution_approved: approved,
    })
}

/// lib/reservations.mjs findSessionConflicts — active path leases owned by a
/// DIFFERENT session overlapping any requested path. `true` = at least one.
pub(crate) fn has_session_conflict(root: &Path, acting: &str, requested: &[String], now: f64) -> MR<bool> {
    if requested.is_empty() {
        return Ok(false);
    }
    let acting = js_trim(acting);
    for rec in list_path_lease_records(root)? {
        if lease_record_expired(&rec, now)? {
            continue;
        }
        let resv = lease_to_resv_lite(&rec)?;
        let owner = match &resv.session {
            Some(Value::String(s)) if !js_trim(s).is_empty() => s.clone(),
            _ => continue, // a legacy/sessionless row never conflicts here
        };
        if owner == acting {
            continue;
        }
        if requested.iter().any(|p| rsv::paths_overlap(&resv.path, p)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// lib/worktree-holds.mjs findForeignHolds over the ledger at `root`
/// (resolveHoldTopology's ORDINARY arm, `{mainRoot: root, …}` — see the
/// section header). `holder` is whoever the caller decided owns the rows it
/// is asking about: hha-3 hands it the CELL's owner, not the acting checkout.
pub(crate) fn has_foreign_hold(root: &Path, holder: &str, requested: &[String], now: f64) -> MR<bool> {
    if requested.is_empty() {
        return Ok(false);
    }
    let acting = js_trim(holder);
    let store = read_holds_store(root)?;
    let Some(Value::Array(holds)) = store.get("holds") else { return Ok(false) };
    for hold in holds {
        // isActive: released_at == null && !isExpired
        if !matches!(hold.get("released_at"), None | Some(Value::Null)) {
            continue;
        }
        let expired = match hold.get("ttl_seconds") {
            Some(Value::Number(n)) => {
                let ttl = n.as_f64().unwrap_or(f64::NAN);
                if !ttl.is_finite() || ttl <= 0.0 {
                    false
                } else {
                    match rsv::date_parse_val(hold.get("mirrored_at")).map_err(|_| Fail::Delegate)? {
                        None => false,
                        Some(m) => m + ttl * 1000.0 <= now,
                    }
                }
            }
            _ => false,
        };
        if expired {
            continue;
        }
        if matches!(hold.get("holder"), Some(Value::String(s)) if s == acting) {
            continue;
        }
        let hold_path = match hold.get("path") {
            Some(v) if js_truthy(v) => jsjson::js_to_string(v),
            _ => String::new(),
        };
        if requested.iter().any(|p| rsv::paths_overlap(&hold_path, p)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Array.isArray(cell.files) ? cell.files : []`, then `.filter(Boolean)` —
/// the request list both hold checks take.
pub(crate) fn declared_files(cell: &Value) -> (usize, Vec<String>) {
    let Some(Value::Array(files)) = cell.get("files") else { return (0, Vec::new()) };
    let requested = files
        .iter()
        .filter(|f| js_truthy(f))
        .map(jsjson::js_to_string)
        .collect();
    (files.len(), requested)
}

/// claimNextCell's `candidateOk` = holdFree && checkCellBudgets().ok.
pub(crate) fn candidate_ok(root: &Path, control: &Path, session: &str, cell: &Value, now: f64) -> MR<bool> {
    let (raw_len, requested) = declared_files(cell);
    if raw_len > 0 {
        if has_session_conflict(control, session, &requested, now)? {
            return Ok(false);
        }
        // A hold belongs to the work stream that owns the CELL, never to the
        // checkout that typed the command
        // (docs/history/hold-holder-attribution/plan.md). claim-next is a
        // control-plane command, so it runs from MAIN — and after hha-1 the
        // mirrored row for a cell whose feature owns a granted worktree names
        // that WORKTREE. resolveHoldTopology(root)'s ordinary `'main'` is
        // still the ACTING holder and still the fallback; asking it directly
        // here would make main read a cell's own holds as foreign and skip
        // the very cell it exists to hand out. A hold owned by a DIFFERENT
        // work stream is unaffected and still skips the candidate.
        let cell_id = match cell.get("id") {
            Some(v) if js_truthy(v) => jsjson::js_to_string(v),
            _ => String::new(), // no id → the helper answers with the acting holder
        };
        let owner = rsv::cell_hold_owner(root, "main", &cell_id).map_err(|_| Fail::Delegate)?;
        if has_foreign_hold(root, &owner.holder, &requested, now)? {
            return Ok(false);
        }
    }
    let Value::Object(map) = cell else { return Err(Fail::Delegate) };
    Ok(matches!(check_cell_budgets(map)?, BudgetCheck::Ok))
}

/// readyCells(root, feature) — listCells({feature, status:'open'}) filtered to
/// cells whose depsAllCapped list is empty (lib/cells.mjs).
pub(crate) fn ready_cells(root: &Path, feature: Option<&str>) -> MR<Vec<Value>> {
    let mut out = Vec::new();
    for cell in list_cells(root, feature, Some("open"))? {
        if deps_all_capped_is_empty(root, &cell)? {
            out.push(cell);
        }
    }
    Ok(out)
}

/// Everything claim-next reads that could route the command back to Node,
/// probed BEFORE the sweep's first write. See the section header for why a
/// residual post-sweep delegate is still byte-safe.
pub(crate) fn prescan_claim_next(root: &Path, control: &Path) -> MR<()> {
    bstate::read_state_brief(root);
    delegate_only(list_session_records(control))?;
    delegate_only(read_holds_store(root))?;
    // CUTOVER: the lane-record walk that used to live here probed for exactly
    // two things — corrupt JSON and |n| >= 1e21 numbers. Both are native now,
    // so the walk has nothing left to decide, and keeping it would warn about
    // lane files this command never reads. Deleted; resolvePipeline warns at
    // its own read, once, like Node did.
    if crate::verbs::backlog::feature_backlog_rank(root).is_none() {
        return Err(Fail::Delegate);
    }
    for rec in list_path_lease_records(root)? {
        delegate_only(lease_to_resv_lite(&rec))?;
    }
    if let Ok(entries) = std::fs::read_dir(claims_dir(control)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(cell) = name.strip_suffix(".json") else { continue };
            let claim = match read_claim(control, cell) {
                Err(Fail::Delegate) => return Err(Fail::Delegate),
                Err(_) => continue, // requireId throws are Node's own, per-cell
                Ok(c) => c,
            };
            if let Some(claim) = claim {
                delegate_only(read_session_of_claim(control, &claim))?;
            }
        }
    }
    // The sweep's reset spreads `...(cellRecord.trace || {})`; a truthy
    // NON-object trace would spread JS-exotic index keys. Delegate up front
    // rather than guess (no bee-written cell has that shape). Read RAW here,
    // not through read_store_json: this walks every cell file, while the
    // sweep only reads the ones it resets, so a corrupt file must not warn
    // from the probe — its own read will warn if the sweep gets there.
    if let Ok(entries) = std::fs::read_dir(cells_dir(control)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            if let ReadJson::Parsed(Value::Object(m)) = read_json(&entry.path()) {
                match m.get("trace") {
                    None | Some(Value::Null) | Some(Value::Object(_)) => {}
                    Some(t) if !js_truthy(t) => {}
                    Some(_) => return Err(Fail::Delegate),
                }
            }
        }
    }
    Ok(())
}

/// bee.mjs handleCellsClaimNext + lib/cells.mjs claimNextCell.
pub(crate) fn run_claim_next(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["worker", "session-id", "ttl", "isolate"]) {
        return None;
    }
    let worker = flags.req_str("worker")?.to_string();
    let session_flag = opt_string_flag(&flags, "session-id")?;
    let _isolate = bool_flag(&flags, "isolate")?;
    let ttl: Option<f64> = match flags.get("ttl") {
        None => None,
        Some(FlagV::Present) => return None,
        Some(FlagV::S(s)) => match rsv::js_number_flag(s) {
            Err(_) => return None, // validate() refuses the shape — Node's message
            Ok(parsed) => Some(parsed.unwrap_or(f64::NAN)),
        },
    };
    dispatch("cells claim-next", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let control = control_root(&root)?;
        // resolveSessionId({flag, root: controlRootFor(root)}) — flag ->
        // BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID -> the durable
        // single-live-session adoption (hardening-1-7-10 D5/1710-10).
        let session_id = match resolve_session_flag_env(session_flag.as_deref()) {
            Some(s) => Some(s),
            None => resolve_session_adopt(&control)?,
        };
        let Some(session_id) = session_id else {
            return Err(Fail::Thrown(
                "claim-next: --session-id or CLAUDE_CODE_SESSION_ID env is required.".into(),
            ));
        };
        if let Some(t) = ttl {
            if !t.is_finite() || t <= 0.0 {
                return Err(Fail::Thrown("--ttl must be a positive integer (seconds).".into()));
            }
        }
        // applyWritePolicy — a no-op for this verb (see the section header);
        // readConfig's corrupt-file case now warns and reads as absent.
        bstate::read_config_raw(&root);

        prescan_claim_next(&root, &control)?;

        // ── claimNextCell ──────────────────────────────────────────────────
        let session = js_trim(&session_id).to_string();
        // Unconditional, first thing — the production sweep trigger (C10).
        // The just-resolved session is the sweep's own D6 self-exclusion:
        // this claim-next call can never sweep its own claims.
        sweep_expired_claims(&control, rsv::now_ms(), Some(session.as_str()))?;

        let (own_feature, own_approved) = match resolve_pipeline(&root, &control, &session)? {
            Pipeline::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim-next: {code} — {reason}")));
            }
            Pipeline::Ok { feature, execution_approved } => (feature, execution_approved),
        };

        let now = rsv::now_ms();
        let mut candidate: Option<Value> = None;
        if let Some(feature) = &own_feature {
            if own_approved {
                for cell in ready_cells(&root, Some(feature))? {
                    if candidate_ok(&root, &control, &session, &cell, now)? {
                        candidate = Some(cell);
                        break;
                    }
                }
            }
        }

        if candidate.is_none() {
            let state = bstate::read_state_brief(&root);
            // feature -> (approved, created_at); insertion-ordered like the Map.
            let mut pipelines: Vec<(String, bool, Value)> = Vec::new();
            let state_feature = match &state.feature {
                v if js_truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            };
            // GH#20 + D1 (default-pipeline-liveness): ONE walk of every session
            // record produces both facts liveness protects — lanes actively
            // owned by ANOTHER live session, and whether some OTHER live
            // session is unbound, which by definition means it is working the
            // default pipeline right now (docs/history/default-pipeline-liveness).
            let (live_owned, default_pipeline_live_peer) =
                live_session_facts(&control, &session, now)?;
            if let Some(f) = &state_feature {
                if own_feature.as_deref() != Some(f.as_str()) && !default_pipeline_live_peer {
                    pipelines.push((
                        f.clone(),
                        matches!(state.gates.get("execution"), Some(Value::Bool(true))),
                        Value::Null,
                    ));
                }
            }
            for lane in crate::verbs::workflow_store::list_lanes(&root)? {
                let feature = match lane.get("feature") {
                    Some(v) if js_truthy(v) => jsjson::js_to_string(v),
                    _ => continue,
                };
                if own_feature.as_deref() == Some(feature.as_str())
                    || pipelines.iter().any(|(f, _, _)| *f == feature)
                {
                    continue;
                }
                if live_owned.iter().any(|l| *l == feature) {
                    continue;
                }
                let approved = lane
                    .get("approved_gates")
                    .and_then(|g| g.get("execution"))
                    .map(|v| matches!(v, Value::Bool(true)))
                    .unwrap_or(false);
                let created_at = match lane.get("created_at") {
                    Some(v) if js_truthy(v) => v.clone(),
                    _ => Value::Null, // `lane.created_at || null`
                };
                pipelines.push((feature, approved, created_at));
            }

            let rank = crate::verbs::backlog::feature_backlog_rank(&root)
                .ok_or(Fail::Delegate)?;
            // (cell, rank, created_at_ms) — the sort keys, built in pool order.
            let mut pool: Vec<(Value, f64, Option<f64>)> = Vec::new();
            for (feature, approved, created_at) in &pipelines {
                if !approved {
                    continue; // D2: an unapproved lane is never touched
                }
                let rank_of = rank.get(feature).map(|r| *r as f64).unwrap_or(f64::INFINITY);
                let created = match created_at {
                    v if js_truthy(v) => rsv::date_parse_val(Some(v))
                        .map_err(|_| Fail::Delegate)?
                        .filter(|ms| ms.is_finite()),
                    _ => None, // `a.meta.created_at ? Date.parse(...) : NaN`
                };
                for cell in ready_cells(&root, Some(feature))? {
                    if candidate_ok(&root, &control, &session, &cell, now)? {
                        pool.push((cell, rank_of, created));
                    }
                }
            }
            // rank asc, then a KNOWN created_at asc, then a known one before an
            // unknown one; V8's sort is stable and so is Rust's.
            pool.sort_by(|a, b| {
                if a.1 != b.1 {
                    return a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal);
                }
                match (a.2, b.2) {
                    (Some(x), Some(y)) if x != y => {
                        x.partial_cmp(&y).unwrap_or(Ordering::Equal)
                    }
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            });
            candidate = pool.into_iter().next().map(|(cell, _, _)| cell);
        }

        let Some(candidate) = candidate else {
            return Err(Fail::Thrown(
                "claim-next: NO_APPROVED_WORK \u{2014} no claimable cell: the acting session's own pipeline has none ready, and no other execution-approved pipeline has a ready cell free of another session's hold.".into(),
            ));
        };
        let cell_id = js_string_or_undefined(candidate.get("id"));
        let (cell, claim) = match claim_cell_cross_session(
            &root,
            &control,
            Some(session.as_str()),
            &worker,
            &cell_id,
            ttl,
            Some(&candidate),
        )? {
            CrossClaim::Ok { cell, claim } => (cell, claim),
            CrossClaim::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim-next: {code} — {reason}")));
            }
        };
        let text = format!(
            "Claimed {} for {worker} (session {session}).",
            js_string_or_undefined(cell.get("id"))
        );
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("cell".into(), cell);
        result.insert("claim".into(), claim);
        Ok(Out::Emit(Value::Object(result), text, 0))
    })
}
