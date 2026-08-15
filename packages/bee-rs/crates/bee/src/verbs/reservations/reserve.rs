// `reservations reserve`
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

// ─── reservations reserve ──────────────────────────────────────────────────

pub(crate) struct ReserveParams {
    pub(crate) agent: String,
    pub(crate) cell: String,
    pub(crate) path: String,
    pub(crate) ttl: Option<f64>,
    pub(crate) session: Option<String>,
    pub(crate) kind: Option<String>,
}

pub(crate) fn run_reserve(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["agent", "cell", "path", "ttl", "session", "kind"]) {
        return None;
    }
    let agent = flags.req_str("agent")?.to_string();
    let cell = flags.req_str("cell")?.to_string();
    let path = flags.req_str("path")?.to_string();
    let ttl_flag = match flags.get("ttl") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
    };
    let session = flags.truthy_str("session").map(str::to_string);
    let kind = flags.truthy_str("kind").map(str::to_string);

    let (ctx, roots) = match prelude_worktree("reservations reserve", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    let topology = roots.hold_topology();
    // handleReservationsReserve's own --ttl gate runs before everything else.
    let ttl: Option<f64> = match &ttl_flag {
        None => None,
        Some(raw) => match js_number_flag(raw) {
            Err(_) => return None, // Node's validate() owns the message
            Ok(parsed) => match parsed {
                Some(v) if v.is_finite() && v > 0.0 => Some(v),
                _ => return Some(ctx.fail("--ttl must be a positive integer (seconds).")),
            },
        },
    };
    let root_s = ctx.root.to_str()?.to_string();
    let params = ReserveParams { agent, cell, path, ttl, session, kind };

    let topo = topology.as_ref().map(|(m, h)| Topo { main_root: m, holder: h });

    // Pre-checks (pure reads) — any Exotic delegates before the lock, so the
    // Node re-run owns the whole command including its own lock telemetry.
    if reserve_prechecks(topo, &root_s, &params).is_err() {
        return None;
    }

    let out = reserve_exec(topo, &root_s, &params, lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// bee.mjs resolveHoldTopology's `{mainRoot, holder}`, borrowed for one
/// command. `None` (an UNGRANTED linked worktree) means the cross-worktree
/// section is skipped entirely — no lock, no ledger read, no mirror row.
#[derive(Clone, Copy)]
pub(crate) struct Topo<'a> {
    pub(crate) main_root: &'a Path,
    pub(crate) holder: &'a str,
}

pub(crate) fn reserve_prechecks(topo: Option<Topo>, root_s: &str, p: &ReserveParams) -> Ex<()> {
    let now = now_ms();
    if let Some(t) = topo {
        if res_normalize_path(&p.path).is_empty() {
            // insertHold would throw AFTER the lease write — a state Node must
            // own. Without a topology insertHold never runs, so an empty
            // normalized path is reserve()'s own plain "path is required."
            return Err(Exotic);
        }
        let store = read_holds_store(t.main_root)?;
        for hold in holds_of(&store) {
            hold_active(hold, now)?; // date-modelability probe
            hold_foreign_expiry(hold)?;
        }
    }
    let control_root = control_root_for(root_s)?;
    let flag_or_env_session = p
        .session
        .as_deref()
        .map(|s| !js_trim(s).is_empty())
        .unwrap_or(false)
        || env_nonempty("BEE_SESSION_ID").is_some()
        || env_nonempty("CLAUDE_CODE_SESSION_ID").is_some();
    if !flag_or_env_session {
        for r in list_session_records(&control_root)? {
            heartbeat_stale(&r, now)?;
        }
    }
    for rec in list_path_lease_records(root_s)? {
        lease_record_expired(&rec, now)?;
        lease_to_reservation(&rec)?;
    }
    // computeExpiresAt / toISOString range for the record about to be built.
    let ttl_eff = p.ttl.unwrap_or(DEFAULT_TTL_SECONDS);
    iso_from_ms(now + ttl_eff * 1000.0 + 60_000.0)?;
    Ok(())
}

/// The whole reservePathAtomic section — separated from run_reserve so tests
/// can drive it against a fixture root (max_attempts lets contention tests
/// refuse instantly like a hook would).
///
/// bee.mjs reservePathAtomic: WITH a topology the foreign-hold check, the
/// local reserve and the mirror-insert are ONE atomic section under
/// withHoldsLock(topology.mainRoot) (hardening-1-7-10 D3). WITHOUT one — an
/// ungranted linked worktree — Node runs the bare `doReserve()` and takes no
/// shared lock at all, so this port must not take one either: a LOCK_BUSY
/// refusal Node could never emit would be a C2 break of its own.
pub(crate) fn reserve_exec(
    topo: Option<Topo>,
    root_s: &str,
    p: &ReserveParams,
    max_attempts: u32,
) -> R2<Out> {
    let Some(t) = topo else {
        return reserve_locked(None, root_s, p);
    };
    let guard = match lock::acquire_store_lock(t.main_root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts)
    {
        Ok(g) => g,
        Err(busy) => return Err(Err2::Msg(busy.message())),
    };
    let out = reserve_locked(Some(t), root_s, p);
    drop(guard); // Node releases in withStoreLock's finally, before emit
    out
}

pub(crate) fn reserve_locked(topo: Option<Topo>, root_s: &str, p: &ReserveParams) -> R2<Out> {
    // findForeignHolds runs its own fresh readStore inside the section.
    let mut store = match topo {
        Some(t) => Some(read_holds_store(t.main_root)?),
        None => None,
    };
    // hha-1 (docs/history/hold-holder-attribution/plan.md): the holds this
    // command reads AND writes belong to whoever owns the CELL, not to the
    // checkout typing it. Resolved ONCE, from inside the section (plain
    // filesystem reads only — no ledger read, no second lock), and used by
    // both the foreign-hold check below and the mirrored row further down:
    // stamping the owner without also asking on the read side would make main
    // refuse its own control-plane reserve with FOREIGN_HOLD.
    let owner = match topo {
        Some(t) => Some(cell_hold_owner(t.main_root, t.holder, &p.cell)?),
        None => None,
    };
    // `store` and `owner` are Some exactly when `topo` is, so this is the same
    // "with a topology" arm it has always been.
    if let (Some(store), Some(owner)) = (store.as_ref(), owner.as_ref()) {
        let foreign = find_foreign_holds(store, &owner.holder, &p.path, now_ms())?;
        if let Some(hold) = foreign.first() {
            let hold = (*hold).clone();
            let mut m = Map::new();
            m.insert("ok".into(), Value::Bool(false));
            m.insert("code".into(), Value::String("FOREIGN_HOLD".into()));
            if let Some(v) = jget(&hold, "holder") {
                m.insert("holder".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "feature") {
                m.insert("feature".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "cell") {
                m.insert("cell".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "path") {
                m.insert("path".into(), v.clone());
            }
            let expiry = hold_foreign_expiry(&hold)?;
            m.insert("expires".into(), Value::String(expiry.clone()));
            let or_unknown = |k: &str| match jget(&hold, k) {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            let text = format!(
                "bee cross-worktree hold: \"{}\" is held by checkout \"{}\" (feature {}, cell {}), {}. Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.",
                js_disp_opt(jget(&hold, "path")),
                js_disp_opt(jget(&hold, "holder")),
                or_unknown("feature"),
                or_unknown("cell"),
                expiry
            );
            return Ok(Out::Emit(Value::Object(m), text, 1));
        }
    }

    // ── reserve() (lib/reservations.mjs) ───────────────────────────────────
    if js_trim(&p.agent).is_empty() {
        return Ok(Out::Thrown("reserve: agent is required.".into()));
    }
    if js_trim(&p.cell).is_empty() {
        return Ok(Out::Thrown("reserve: cell id is required.".into()));
    }
    if js_trim(&p.path).is_empty() {
        return Ok(Out::Thrown("reserve: path is required.".into()));
    }
    let kind = p.kind.clone().unwrap_or_else(|| "lease".to_string());
    if !RESERVATION_KINDS.contains(&kind.as_str()) {
        return Ok(Out::Thrown(format!(
            "reserve: kind must be one of intent/lease (got {}).",
            js_quote(&kind)
        )));
    }
    let control_root = control_root_for(root_s)?;
    let resolved_session = resolve_session_id(p.session.as_deref(), &control_root)?;
    if resolved_session.is_none() && is_concurrent_mode(&control_root)? {
        let reason = format!(
            "reserve: cannot reserve \"{}\" without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored).",
            js_trim(&p.path)
        );
        let result = json!({
            "ok": false,
            "code": "SESSION_REQUIRED",
            "reason": reason,
            "conflicts": [],
        });
        let text = "Reservation CONFLICT — return [BLOCKED] to the orchestrator:".to_string();
        return Ok(Out::Emit(result, text, 1));
    }

    let trimmed_agent = js_trim(&p.agent).to_string();
    let trimmed_cell = js_trim(&p.cell).to_string();
    let now = now_ms(); // reserve()'s own `now = Date.now()` default

    let overlap_conflicts: Vec<Resv> = find_conflicts(&control_root, &trimmed_agent, &p.path, now)?
        .into_iter()
        .filter(|c| is_hard_conflict(c, &p.path))
        .collect();
    if !overlap_conflicts.is_empty() {
        return Ok(conflict_out(&overlap_conflicts));
    }

    // acquireLeases (lease-store.mjs): O_EXCL create of the one path lease.
    let canonical = res_normalize_path(&p.path);
    let resource_key = format!("path:{canonical}");
    let ttl_eff = p.ttl.unwrap_or(DEFAULT_TTL_SECONDS);
    let acquired_at = iso_from_ms(now)?;
    let expires_at: Option<String> = if ttl_eff.is_finite() && ttl_eff > 0.0 {
        Some(iso_from_ms(now + ttl_eff * 1000.0)?)
    } else {
        None // computeExpiresAt: non-positive ttl never expires
    };
    let session_for_lease = resolved_session
        .clone()
        .unwrap_or_else(|| SESSIONLESS_SESSION_ID.to_string());
    let mut record = Map::new();
    record.insert("resource".into(), Value::String(resource_key.clone()));
    record.insert("mode".into(), Value::String("write".into()));
    record.insert("workflow_id".into(), Value::String(trimmed_cell.clone()));
    record.insert("session_id".into(), Value::String(session_for_lease));
    record.insert(
        "workspace_id".into(),
        Value::String(format!("agent:{trimmed_agent}")),
    );
    record.insert("epoch".into(), Value::Number(Number::from_f64(0.0).unwrap()));
    record.insert("acquired_at".into(), Value::String(acquired_at));
    record.insert(
        "expires_at".into(),
        expires_at.clone().map(Value::String).unwrap_or(Value::Null),
    );
    record.insert("kind".into(), Value::String(kind));
    // Reporting-only addition (D3, wtf-2): the same holder string the
    // mirrored ledger row carries below — the cell's owning worktree when it
    // has one (hha-1), else "main" for an ordinary checkout or the
    // git-verified worktree id for a granted one. Omitted entirely for an
    // ungranted linked worktree (topo is None there), so an old reader never
    // sees a key it does not expect.
    if let Some(owner) = owner.as_ref() {
        record.insert("holder".into(), Value::String(owner.holder.clone()));
    }

    let lease_file = path_lease_file(&control_root, &p.path);
    if let Some(dir) = lease_file.parent() {
        std::fs::create_dir_all(dir).map_err(|_| Err2::Ex)?;
    }
    let body = format!("{}\n", jsjson::stringify_pretty(&Value::Object(record.clone())));
    let create = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lease_file);
    match create {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(body.as_bytes()).map_err(|_| Err2::Ex)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lost an exact-path race — report like a pre-check conflict.
            let holder_resv: Option<Resv> = match std::fs::read_to_string(&lease_file) {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(v) => {
                        let v = js_numberify(&v)?;
                        match v {
                            Value::Object(m)
                                if matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:")) =>
                            {
                                Some(lease_to_reservation(&m)?)
                            }
                            _ => None,
                        }
                    }
                    Err(_) => None, // readLeaseSafe: unparseable → null holder
                },
                Err(_) => None,
            };
            let conflicts: Vec<Resv> = holder_resv.into_iter().collect();
            return Ok(conflict_out(&conflicts));
        }
        Err(_) => return Err(Err2::Ex),
    }

    let reservation = lease_to_reservation(&record)?;

    // insertHold (worktree-holds.mjs), called from inside the held section —
    // the ledger read from this same section is reused (see module header).
    // Skipped wholesale without a topology, exactly like Node.
    if let (Some(t), Some(store), Some(owner)) = (topo, store.as_mut(), owner.as_ref()) {
        let ttl_secs = reservation.ttl_seconds.unwrap_or(f64::NAN);
        let hold_ttl = if ttl_secs.is_finite() && ttl_secs > 0.0 {
            ttl_secs.floor()
        } else {
            DEFAULT_TTL_SECONDS
        };
        let mut hold = Map::new();
        hold.insert(
            "path".into(),
            Value::String(res_normalize_path(&reservation.path)),
        );
        // hha-1: the cell's owning worktree when its feature has one — the
        // work stream this hold actually belongs to. Falls back to
        // topology.holder ("main" from an ordinary checkout, the git-verified
        // worktree id from a granted one) for a cell with no feature or a
        // feature with no granted worktree.
        hold.insert("holder".into(), Value::String(owner.holder.clone()));
        // Resolved from the cell instead of hardcoded null, so a refusal can
        // name the feature rather than reading "feature unknown".
        hold.insert(
            "feature".into(),
            owner.feature.clone().map(Value::String).unwrap_or(Value::Null),
        );
        hold.insert(
            "session".into(),
            match &reservation.session {
                Some(Value::String(s)) if !js_trim(s).is_empty() => {
                    Value::String(js_trim(s).to_string())
                }
                _ => Value::Null,
            },
        );
        hold.insert("cell".into(), Value::String(trimmed_cell.clone()));
        hold.insert(
            "ttl_seconds".into(),
            Value::Number(Number::from_f64(hold_ttl).ok_or(Err2::Ex)?),
        );
        hold.insert("mirrored_at".into(), Value::String(now_iso()));
        hold.insert("released_at".into(), Value::Null);
        if let Some(Value::Array(holds)) = store.get_mut("holds") {
            holds.push(Value::Object(hold));
        }
        write_json_atomic(&holds_ledger_path(t.main_root), store).map_err(|_| Err2::Ex)?;
    }

    let resv_value = resv_to_value(&reservation);
    let text = format!(
        "Reserved \"{}\" for {} (cell {}, ttl {}s).",
        reservation.path,
        js_disp_opt(reservation.agent.as_ref()),
        js_disp_opt(reservation.cell.as_ref()),
        reservation
            .ttl_seconds
            .map(jsjson::js_f64_to_string)
            .unwrap_or_else(|| "NaN".to_string()),
    );
    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("reservation".into(), resv_value);
    Ok(Out::Emit(Value::Object(result), text, 0))
}

/// reservePathAtomic's TYPED verdict — bee.mjs returns `{refusal}` (the
/// foreign hold) or `{reserveResult}` (reserve()'s own ok/conflict answer)
/// and leaves presentation to each caller: `reservations reserve` renders the
/// FOREIGN_HOLD / CONFLICT text, `dispatch prepare --claim` renders its own
/// `- checkout "…" holds "…"` lines and then unwinds.
///
/// Derived from the SAME `Out` `run_reserve` emits, never from a second copy
/// of the section — the values inside are the identical JS values Node's
/// `section.refusal` / `section.reserveResult` carry.
pub(crate) enum ReserveOutcome {
    /// `{refusal}` — the cross-worktree hold, with `holder`/`feature`/`cell`/
    /// `path` copied verbatim off the ledger row.
    ForeignHold(Map<String, Value>),
    /// `reserveResult.conflicts` — each row has `agent`/`path`/`cell`.
    Conflicts(Vec<Value>),
    /// `reserveResult.reservation` — note `.path` is the NORMALIZED path the
    /// lease record carries, not the raw `files[]` entry.
    Reserved(Value),
    /// reserve()'s own argument refusals (thrown at the CLI boundary).
    Thrown(String),
}

/// bee.mjs's `reservePathAtomic(root, {agent, cell, path})` — the ONE reserve
/// door `reservations reserve` and `dispatch prepare --claim` share, so the
/// foreign-hold check, the local reserve and the mirror-insert cannot diverge
/// between them.
///
/// pub(crate) since the `dispatch prepare --claim` port. `ttl`/`session`/
/// `kind` are structurally absent because the dispatch call site passes none:
/// reserve() then defaults to DEFAULT_TTL_SECONDS, `resolveSessionId` from the
/// environment only, and kind `'lease'` (hard conflicts).
pub(crate) fn reserve_path_atomic(
    topo: Option<(&Path, &str)>,
    root_s: &str,
    agent: &str,
    cell: &str,
    path: &str,
) -> R2<ReserveOutcome> {
    let params = ReserveParams {
        agent: agent.to_string(),
        cell: cell.to_string(),
        path: path.to_string(),
        ttl: None,
        session: None,
        kind: None,
    };
    let t = topo.map(|(m, h)| Topo { main_root: m, holder: h });
    // Every delegate-trigger front-loaded, before the cross-worktree lock.
    reserve_prechecks(t, root_s, &params)?;
    Ok(match reserve_exec(t, root_s, &params, lock::MAX_ATTEMPTS)? {
        Out::Thrown(m) => ReserveOutcome::Thrown(m),
        Out::Emit(Value::Object(m), _, _) => {
            if matches!(m.get("code"), Some(Value::String(c)) if c == "FOREIGN_HOLD") {
                ReserveOutcome::ForeignHold(m)
            } else if m.get("ok") == Some(&Value::Bool(true)) {
                ReserveOutcome::Reserved(m.get("reservation").cloned().unwrap_or(Value::Null))
            } else {
                ReserveOutcome::Conflicts(match m.get("conflicts") {
                    Some(Value::Array(a)) => a.clone(),
                    _ => Vec::new(),
                })
            }
        }
        Out::Emit(..) => return Err(Err2::Ex), // unreachable: always an object
    })
}

pub(crate) fn conflict_out(conflicts: &[Resv]) -> Out {
    let mut lines =
        vec!["Reservation CONFLICT — return [BLOCKED] to the orchestrator:".to_string()];
    for c in conflicts {
        // D3 (wtf-2): a lease carrying a holder names the checkout that holds
        // it, same as the cross-worktree FOREIGN_HOLD refusal already does.
        // No holder (pre-cell lease, or an ungranted linked worktree) — the
        // line stays byte-for-byte what it always was.
        let line = match &c.holder {
            Some(h) => format!(
                "- {} holds \"{}\" (cell {}, agent {})",
                js_disp(h),
                c.path,
                js_disp_opt(c.cell.as_ref()),
                js_disp_opt(c.agent.as_ref())
            ),
            None => format!(
                "- {} holds \"{}\" (cell {})",
                js_disp_opt(c.agent.as_ref()),
                c.path,
                js_disp_opt(c.cell.as_ref())
            ),
        };
        lines.push(line);
    }
    let result = json!({
        "ok": false,
        "conflicts": conflicts.iter().map(resv_to_value).collect::<Vec<_>>(),
    });
    Out::Emit(result, lines.join("\n"), 1)
}
