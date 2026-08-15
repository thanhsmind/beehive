// `reservations release` and `reservations sweep`
//
// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── reservations release ──────────────────────────────────────────────────

pub(crate) fn run_release(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["agent", "cell"]) {
        return None;
    }
    let agent = flags.req_str("agent")?.to_string();
    let cell = flags.truthy_str("cell").map(str::to_string);

    let (ctx, roots) = match prelude_worktree("reservations release", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    let topology = roots.hold_topology();
    let topo = topology.as_ref().map(|(m, h)| Topo { main_root: m, holder: h });
    let root_s = ctx.root.to_str()?.to_string();

    // Pre-checks: every store this verb will read or mutate.
    let precheck = (|| -> Ex<()> {
        let now = now_ms();
        for rec in list_path_lease_records(&root_s)? {
            lease_record_expired(&rec, now)?;
            lease_to_reservation(&rec)?;
        }
        // Without a topology the ledger is never opened (releaseHolds is
        // skipped), so it must not be probed either.
        if let Some(t) = topo {
            read_holds_store(t.main_root)?;
        }
        Ok(())
    })();
    if precheck.is_err() {
        return None;
    }

    let out = release_exec(topo, &root_s, &agent, cell.as_deref(), lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// releaseReservationsForAgent — pub(crate) since the `dispatch prepare
/// --claim` port, whose conflict unwind releases exactly the reservations the
/// same call had just taken (`agent` = the worker, `cell` = the claimed cell).
pub(crate) fn release_reservations_for_agent(
    topo: Option<(&Path, &str)>,
    root_s: &str,
    agent: &str,
    cell: Option<&str>,
) -> R2<Out> {
    let t = topo.map(|(m, h)| Topo { main_root: m, holder: h });
    release_exec(t, root_s, agent, cell, lock::MAX_ATTEMPTS)
}

pub(crate) fn release_exec(
    topo: Option<Topo>,
    root_s: &str,
    agent: &str,
    cell: Option<&str>,
    max_attempts: u32,
) -> R2<Out> {
    // releaseReservationsForAgent: matched rows FIRST (before release marks
    // them), to derive the ledger's {cell, session} scoping pairs.
    let matched: Vec<Resv> = list_reservations(root_s, true, now_ms())?
        .into_iter()
        .filter(|r| {
            let agent_match = matches!(&r.agent, Some(Value::String(s)) if s == agent);
            let cell_match = match cell {
                None => true,
                Some(c) => {
                    matches!(&r.cell, Some(v) if v == &Value::String(c.to_string()))
                }
            };
            agent_match && cell_match
        })
        .collect();
    let mut pairs: Vec<(Value, Option<Value>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in &matched {
        let Some(cell_v) = r.cell.as_ref().filter(|c| truthy(c)) else {
            continue;
        };
        let session_v = r.session.as_ref().filter(|s| truthy(s)).cloned();
        let key = format!(
            "{}::{}",
            js_disp(cell_v),
            session_v.as_ref().map(js_disp).unwrap_or_default()
        );
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v.clone(), session_v));
        }
    }
    // hha-2 (docs/history/hold-holder-attribution/plan.md): those pairs are
    // derived from LIVE reservations, so a cell that is already capped or
    // unclaimed has no lease left to derive from — the pairs come out empty
    // and its ledger rows become unreachable by any release at all. That is
    // why ghost rows outlive the cells that made them. An EXPLICIT `--cell`
    // therefore scopes the ledger pass by that cell id directly, in addition
    // to whatever the live leases contributed: a session-less pair, which
    // covers every session's rows for that one cell. Without `--cell` the
    // scoping is untouched, so a whole-agent release clears exactly what it
    // cleared before. Rows already marked by an earlier pair are skipped by
    // the `unreleased` guard below, so nothing is counted twice.
    if let Some(c) = cell {
        let cell_v = Value::String(c.to_string());
        let key = format!("{}::", js_disp(&cell_v));
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v, None));
        }
    }

    // release() (lib/reservations.mjs).
    if js_trim(agent).is_empty() {
        return Ok(Out::Thrown("release: agent is required.".into()));
    }
    let control_root = control_root_for(root_s)?;
    let trimmed_agent = js_trim(agent);
    let mut released: u64 = 0;
    for rec in list_path_lease_records(root_s)? {
        let lease_agent = match rec.get("workspace_id") {
            Some(Value::String(s)) if s.starts_with("agent:") => {
                Value::String(s["agent:".len()..].to_string())
            }
            Some(other) => other.clone(),
            None => continue, // undefined !== an agent string — never a match
        };
        if !v_is_str(&lease_agent, trimmed_agent) {
            continue;
        }
        if let Some(c) = cell {
            let matches_cell =
                matches!(rec.get("workflow_id"), Some(v) if v == &Value::String(c.to_string()));
            if !matches_cell {
                continue;
            }
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control_root, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => released += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // released: false
            Err(_) => return Err(Err2::Ex),
        }
    }

    // xwh-2/gfb-1: clear the mirrored ledger entries these cells own (hha-2 —
    // no longer "this checkout's"), one locked
    // releaseHolds per {cell, session} pair — the same topology gate the
    // reserve side uses, so `if (topology)` skipping it entirely inside an
    // ungranted worktree leaves holds_released at 0 and takes no lock.
    let mut holds_released: u64 = 0;
    for (cell_v, session_v) in &pairs {
        let Some(t) = topo else { break };
        // hha-2: whichever checkout types the release clears the rows the
        // CELL's owner holds, never the rows the acting checkout happens to
        // be stamped with. bee's control plane runs from main while hha-1
        // stamps the granted worktree that owns the cell, so an acting-holder
        // filter here would leave those rows unclearable by anyone — the same
        // deadlock, mirrored. Resolved per pair, before the lock: plain
        // filesystem reads, no ledger touch.
        let owner = cell_hold_owner(t.main_root, t.holder, &js_disp(cell_v))?;
        let guard =
            match lock::acquire_store_lock(t.main_root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts) {
                Ok(g) => g,
                Err(busy) => return Err(Err2::Msg(busy.message())),
            };
        let mut store = read_holds_store(t.main_root)?;
        let released_at = now_iso();
        let mut count: u64 = 0;
        if let Some(Value::Array(holds)) = store.get_mut("holds") {
            for hold in holds.iter_mut() {
                let unreleased = matches!(jget(hold, "released_at"), None | Some(Value::Null));
                if !unreleased {
                    continue;
                }
                if !matches!(jget(hold, "holder"), Some(Value::String(s)) if s == &owner.holder) {
                    continue;
                }
                if let Some(s) = session_v {
                    let sess_match = matches!(jget(hold, "session"), Some(v) if v == s);
                    if !sess_match {
                        continue;
                    }
                }
                let cell_match = matches!(jget(hold, "cell"), Some(v) if v == cell_v);
                if !cell_match {
                    continue;
                }
                if let Value::Object(m) = hold {
                    m.insert("released_at".into(), Value::String(released_at.clone()));
                }
                count += 1;
            }
        }
        if count > 0 {
            write_json_atomic(&holds_ledger_path(t.main_root), &store).map_err(|_| Err2::Ex)?;
        }
        holds_released += count;
        drop(guard);
    }

    let result = json!({
        "released": released as f64,
        "holds_released": holds_released as f64,
    });
    let text = format!(
        "Released {released} reservation(s){}.",
        if holds_released > 0 {
            format!(" and {holds_released} cross-worktree hold(s)")
        } else {
            String::new()
        }
    );
    Ok(Out::Emit(result, text, 0))
}

// ─── reservations sweep ────────────────────────────────────────────────────

pub(crate) fn run_sweep(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let (ctx, roots) = match prelude_worktree("reservations sweep", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    // sweep uses resolveMainRoot, NOT resolveHoldTopology: sweepExpiredHolds
    // resolves its own empty/missing ledger, so bee.mjs calls it
    // unconditionally — even from an ungranted worktree, which prunes MAIN's
    // ledger exactly as running it from main would.
    let ledger_root = roots.main_root();
    let root_s = ctx.root.to_str()?.to_string();

    let precheck = (|| -> Ex<()> {
        let now = now_ms();
        for rec in list_path_lease_records(&root_s)? {
            lease_record_expired(&rec, now)?;
        }
        let store = read_holds_store(&ledger_root)?;
        for hold in holds_of(&store) {
            hold_expired(hold, now)?;
        }
        Ok(())
    })();
    if precheck.is_err() {
        return None;
    }

    let out = sweep_exec(&ledger_root, &root_s, lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// `root` here is resolveMainRoot(root) — where the shared holds ledger and
/// its lock live, which is NOT the store root inside a linked worktree.
pub(crate) fn sweep_exec(root: &Path, root_s: &str, max_attempts: u32) -> R2<Out> {
    // sweepExpired (lib/reservations.mjs): per-record, lock-free.
    let control_root = control_root_for(root_s)?;
    let now = now_ms();
    let mut released: u64 = 0;
    for rec in list_path_lease_records(root_s)? {
        if !lease_record_expired(&rec, now)? {
            continue;
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control_root, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => released += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Err2::Ex),
        }
    }

    // sweepExpiredHolds (worktree-holds.mjs): whole-ledger, locked.
    let guard = match lock::acquire_store_lock(root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts) {
        Ok(g) => g,
        Err(busy) => return Err(Err2::Msg(busy.message())),
    };
    let mut store = read_holds_store(root)?;
    let now2 = now_ms();
    let released_at = now_iso();
    let mut holds_released: u64 = 0;
    if let Some(Value::Array(holds)) = store.get_mut("holds") {
        for hold in holds.iter_mut() {
            let unreleased = matches!(jget(hold, "released_at"), None | Some(Value::Null));
            if !unreleased {
                continue;
            }
            if !hold_expired(hold, now2)? {
                continue;
            }
            if let Value::Object(m) = hold {
                m.insert("released_at".into(), Value::String(released_at.clone()));
            }
            holds_released += 1;
        }
    }
    if holds_released > 0 {
        write_json_atomic(&holds_ledger_path(root), &store).map_err(|_| Err2::Ex)?;
    }
    drop(guard);

    let result = json!({
        "released": released as f64,
        "holds_released": holds_released as f64,
    });
    let text = format!(
        "Swept {released} expired reservation(s) and {holds_released} expired cross-worktree hold(s)."
    );
    Ok(Out::Emit(result, text, 0))
}
