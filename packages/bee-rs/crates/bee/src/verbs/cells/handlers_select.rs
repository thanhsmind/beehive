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
// files removed, claimed->open cell resets, one decision row per reset)
// BEFORE selection reads anything, so the usual "return None and let Node
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

/// lib/claims.mjs sweepExpiredClaims (hardening-4b sweep-reset, rel180-2).
/// TTL expired AND owner heartbeat stale, both re-verified under the claim's
/// exclusive `<cell>.adopting` gate and — for a session-owned claim — under
/// the same `sessions` store lock heartbeatSession itself holds. Every
/// removal is followed by the claimed->open cell reset under
/// `cells:<id>` and one best-effort decision row.
pub(crate) fn sweep_expired_claims(control: &Path, now: f64) -> MR<()> {
    let dir = claims_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(()) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    for entry in names {
        let Some(cell) = entry.strip_suffix(".json") else { continue };
        let Some(preview) = read_claim(control, cell)? else { continue }; // corrupt: never touch
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
        let swept_session = nullish(swept.get("session"));
        let was_reset = sweep_reset_cell(control, cell, &swept_session, now)?;
        if was_reset {
            // Best-effort: the cell write above already committed, so a
            // decision-log failure must never read as the reset having failed.
            let owner_disp = if swept_session.is_null() {
                "none (sessionless)".to_string()
            } else {
                jsjson::js_to_string(&swept_session)
            };
            let _ = log_decision(
                control,
                &format!(
                    "\u{ab}sweep: cell \"{cell}\" reset claimed -> open \u{2014} swept session \"{owner_disp}\"'s expired, stale claim\u{bb}"
                ),
                "sweepExpiredClaims (hardening-4b) removed the abandoned claim file; the cell was still \"claimed\" by that exact session (trace.claim_session matched), so it is returned to open rather than left claimed-but-unclaimable forever.",
                &["claims", "sweep"],
            );
        }
    }
    Ok(())
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

/// The sweep's claimed->open reset, under the SAME `cells:<id>` store lock
/// every other cells.mjs mutator uses. `readCellForSweepReset` is claims.mjs's
/// own minimal `.bee/cells/<id>.json` read/write (never cells.mjs's readCell —
/// that would cycle), so it never consults the archive.
pub(crate) fn sweep_reset_cell(
    control: &Path,
    cell: &str,
    swept_session: &Value,
    now: f64,
) -> MR<bool> {
    let mut guard = acquire_named_lock(control, &format!("cells:{cell}"))?;
    let outcome = (|| -> MR<bool> {
        let file = cells_dir(control).join(format!("{cell}.json"));
        let record = match read_store_json(&file)? {
            Some(Value::Object(m)) => m,
            _ => return Ok(false), // !cellRecord (or a non-object: .status is undefined)
        };
        if !matches!(record.get("status"), Some(Value::String(s)) if s == "claimed") {
            return Ok(false);
        }
        // `(cellRecord.trace && cellRecord.trace.claim_session) ?? null`
        let current_session = match record.get("trace") {
            None => Value::Null,
            Some(t) if !js_truthy(t) => nullish(Some(t)),
            Some(Value::Object(t)) => nullish(t.get("claim_session")),
            Some(_) => Value::Null, // truthy non-object: .claim_session is undefined
        };
        if !rsv::js_strict_eq(&current_session, swept_session) {
            return Ok(false); // a fresher claim already owns it
        }
        let mut record = record;
        record.insert("status".into(), Value::String("open".into()));
        // `{ ...(cellRecord.trace || {}), worker: null, claimed_at: null,
        //    claim_session: null, swept_at, swept_from_session }`
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
        record.insert("trace".into(), Value::Object(trace));
        let value = Value::Object(record);
        transient_fs_retry(|| crate::fsutil::write_json_atomic(&file, &value))
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
        Ok(true)
    })();
    guard.release();
    outcome
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

/// lib/worktree-holds.mjs findForeignHolds over resolveHoldTopology's
/// ORDINARY arm (`{mainRoot: root, holder: 'main'}` — see the section header).
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
        // resolveHoldTopology(root) is the ordinary constant here.
        if has_foreign_hold(root, "main", &requested, now)? {
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
        sweep_expired_claims(&control, rsv::now_ms())?;

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
            if let Some(f) = &state_feature {
                if own_feature.as_deref() != Some(f.as_str()) {
                    pipelines.push((
                        f.clone(),
                        matches!(state.gates.get("execution"), Some(Value::Bool(true))),
                        Value::Null,
                    ));
                }
            }
            // GH#20: lanes actively owned by ANOTHER live session are never pooled.
            let mut live_owned: Vec<String> = Vec::new();
            for record in list_session_records(&control)? {
                if matches!(record.get("id"), Some(Value::String(s)) if *s == session) {
                    continue;
                }
                let bound = match record.get("lane") {
                    Some(Value::String(l)) => js_trim(l).to_string(),
                    _ => String::new(),
                };
                if bound.is_empty() || heartbeat_stale(Some(&record), now)? {
                    continue;
                }
                live_owned.push(bound);
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
