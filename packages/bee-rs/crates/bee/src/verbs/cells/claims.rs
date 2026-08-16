// claims.mjs — sessions, the O_EXCL claim file, fencing and adoption
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

// Provenance markers for later sections are appended below in order:
// claims-store protocol, trace helpers, text scanners, decision log,
// regen guards, validators, budgets, judge, schedule, test runner,
// reservations-release subset, impact registry, and the verb handlers.
// ─── claims.mjs port — sessions + per-cell O_EXCL claim files ──────────────
// All claim/session stores are CONTROL-PLANE (msn-18b): resolved through
// controlRootFor (rsv::control_root_for — the same git-common-dir walk the
// reservations port already carries). For the ordinary checkouts this native
// path serves, control root == root.

pub(crate) const DEFAULT_CLAIM_TTL_SECONDS: f64 = 3600.0;

pub(crate) const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

pub(crate) const GATE_RETRY_ATTEMPTS: u32 = 15;

pub(crate) const GATE_RETRY_DELAY_MS: u64 = 20;

pub(crate) fn control_root(root: &Path) -> MR<PathBuf> {
    let s = root.to_str().ok_or(Fail::Delegate)?;
    Ok(PathBuf::from(rsv::control_root_for(s).map_err(|_| Fail::Delegate)?))
}

pub(crate) fn claims_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("claims")
}

pub(crate) fn sessions_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("sessions")
}

/// claims.mjs requireId.
pub(crate) fn require_id(value: &str, label: &str) -> MR<String> {
    if js_trim(value).is_empty() {
        return Err(Fail::Thrown(format!("{label} is required.")));
    }
    let id = js_trim(value).to_string();
    if id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(Fail::Thrown(format!("{label} must be a plain id (no path separators).")));
    }
    Ok(id)
}

pub(crate) fn claim_path(control: &Path, cell_id: &str) -> MR<PathBuf> {
    Ok(claims_dir(control).join(format!("{}.json", require_id(cell_id, "cell id")?)))
}

pub(crate) fn claim_gate_path(control: &Path, cell_id: &str) -> MR<PathBuf> {
    Ok(claims_dir(control).join(format!("{}.adopting", require_id(cell_id, "cell id")?)))
}

/// claims.mjs readClaim: readJson fallback null; falsy/non-object -> null
/// (a corrupt file warns and lands in that same null). A JSON-array claim
/// file would take JS property paths this port does not model — Delegate.
pub(crate) fn read_claim(control: &Path, cell_id: &str) -> MR<Option<Map<String, Value>>> {
    let file = claim_path(control, cell_id)?;
    match read_store_json(&file)? {
        None => Ok(None),
        Some(Value::Object(m)) => Ok(Some(m)),
        Some(Value::Array(_)) => Err(Fail::Delegate),
        Some(_) => Ok(None), // primitives fail `typeof === 'object'` (null falls earlier)
    }
}

/// claims.mjs isClaimExpired: non-finite/non-positive TTL never expires;
/// unparseable claimed_at never expires.
pub(crate) fn claim_expired(claim: &Map<String, Value>, now: f64) -> MR<bool> {
    let ttl = match claim.get("ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        _ => return Ok(false), // Number.isFinite(non-number) -> false
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return Ok(false);
    }
    match rsv::date_parse_val(claim.get("claimed_at")).map_err(|_| Fail::Delegate)? {
        None => Ok(false),
        Some(ms) => Ok(ms + ttl * 1000.0 <= now),
    }
}

pub(crate) fn claim_active(claim: Option<&Map<String, Value>>, now: f64) -> MR<bool> {
    match claim {
        None => Ok(false),
        Some(c) => Ok(!claim_expired(c, now)?),
    }
}

/// claims.mjs claimExpiry: 'no expiry' | 'expires <iso>'.
pub(crate) fn claim_expiry(claim: Option<&Map<String, Value>>) -> MR<String> {
    let Some(claim) = claim else { return Ok("no expiry".to_string()) };
    let claimed = rsv::date_parse_val(claim.get("claimed_at")).map_err(|_| Fail::Delegate)?;
    let ttl = match claim.get("ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    match (claimed, ttl) {
        (Some(c), Some(t)) if t.is_finite() && t > 0.0 => {
            Ok(format!("expires {}", rsv::iso_from_ms(c + t * 1000.0).map_err(|_| Fail::Delegate)?))
        }
        _ => Ok("no expiry".to_string()),
    }
}

/// claims.mjs withTransientFsRetry (EBUSY/EPERM/ENOTEMPTY/EMFILE/ENFILE class,
/// 15 x 20ms) — the same Windows sharing-violation classes lock.rs retries.
pub(crate) fn transient_fs_retry<T>(mut f: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    #[cfg(windows)]
    fn transient(e: &std::io::Error) -> bool {
        matches!(e.raw_os_error(), Some(4 | 5 | 32 | 33 | 145))
    }
    #[cfg(unix)]
    fn transient(e: &std::io::Error) -> bool {
        matches!(e.raw_os_error(), Some(code) if {
            code == libc::EBUSY || code == libc::EPERM || code == libc::ENOTEMPTY
                || code == libc::EMFILE || code == libc::ENFILE
        })
    }
    let mut attempt = 0u32;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if !transient(&e) || attempt >= GATE_RETRY_ATTEMPTS {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
            }
        }
    }
}

/// claims.mjs acquireGate: single 'wx' attempt writing {pid, at}. EEXIST ->
/// false. Any other fs error throws in Node with libuv bytes — residual
/// divergence (Rust io message), documented in the header.
pub(crate) fn acquire_gate(control: &Path, cell_id: &str) -> MR<bool> {
    let file = claim_gate_path(control, cell_id)?;
    let body = format!(
        "{}\n",
        jsjson::stringify(&json!({ "pid": std::process::id(), "at": utc_now() }))
    );
    let result = transient_fs_retry(|| {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&file) {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(body.as_bytes())
            }
            Err(e) => Err(e),
        }
    });
    match result {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(Fail::Thrown(format!("{e}"))),
    }
}

pub(crate) fn release_gate(control: &Path, cell_id: &str) {
    if let Ok(file) = claim_gate_path(control, cell_id) {
        let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
    }
}

pub(crate) fn acquire_gate_with_retry(control: &Path, cell_id: &str) -> MR<bool> {
    for attempt in 0..GATE_RETRY_ATTEMPTS {
        if acquire_gate(control, cell_id)? {
            return Ok(true);
        }
        if attempt + 1 < GATE_RETRY_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
        }
    }
    Ok(false)
}

/// claims.mjs clearClaim — unconditional claim-file removal for the
/// claim-clearing cell transitions (cap/unclaim/block/drop/reopen). The
/// production call sites wrap it best-effort (releaseClaimFileBestEffort
/// swallows every failure), so this returns () and never fails the caller.
pub(crate) fn release_claim_file_best_effort(root: &Path, id: &str) {
    let Ok(control) = control_root(root) else { return };
    let _ = (|| -> MR<()> {
        if read_claim(&control, id)?.is_none() {
            return Ok(());
        }
        if !acquire_gate(&control, id)? {
            return Ok(()); // GATE_HELD — best-effort caller never retries
        }
        let still_there = read_claim(&control, id);
        if let Ok(Some(_)) = still_there {
            if let Ok(file) = claim_path(&control, id) {
                let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    other => other,
                });
            }
        }
        release_gate(&control, id);
        Ok(())
    })();
}

// ─── claims: fencing, adoption and same-session renewal ───────────────────
//
// multisession-native-12 (D4/D9 invariant 10). `fence_epoch` is a CAS-style
// token: claimCellFile stamps 1, adoptClaim bumps it by exactly 1 in the SAME
// atomic write as the ownership change, and renewClaimTTL/releaseClaim refuse
// typed CLAIM_FENCE_STALE when a caller presents an epoch BEHIND the stored
// one. Before this cell nothing in the Rust tree consumed the token at all —
// claim_cell_file stamped it and no code path ever compared it, so a stale
// holder's write would have silently proceeded. Fence semantics are
// safety-critical: a stale fence REFUSES, never silently proceeds.
//
// SECOND-PORT NOTE: verbs/state_group.rs carries a NARROWED adopt_claim (the
// `state handoff adopt` path: no typed codes, no `now` injection). That file
// is outside this cell's touchable set, so adoptClaim is re-derived here from
// claims.mjs rather than imported, and
// `adopt_agrees_with_the_state_group_port_on_the_shared_fixture` pins the two
// against one fixture so a future divergence fails a test.

/// claims.mjs `fail(code, reason, extra)` — the typed refusal shape every
/// claims mutator returns instead of throwing.
#[allow(dead_code)]
pub(crate) struct ClaimRefusal {
    pub code: &'static str,
    pub reason: String,
    pub extra: Map<String, Value>,
}

impl ClaimRefusal {
    fn new(code: &'static str, reason: String) -> Self {
        Self { code, reason, extra: Map::new() }
    }
}

#[allow(dead_code)]
pub(crate) enum AdoptClaimOutcome {
    Ok { claim: Value, previous_owner: Option<Value> },
    Refused(ClaimRefusal),
}

#[allow(dead_code)]
pub(crate) enum RenewClaimOutcome {
    Ok { renewed: Vec<String>, skipped: Vec<String> },
    Refused(ClaimRefusal),
}

#[allow(dead_code)]
pub(crate) enum ReleaseClaimOutcome {
    Ok { released: Value },
    Refused(ClaimRefusal),
}

/// `Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1` — a legacy
/// claim written before msn-12 reads as epoch 1, the same default a fresh
/// claimCellFile stamps.
pub(crate) fn current_fence_epoch(claim: &Map<String, Value>) -> f64 {
    match claim.get("fence_epoch") {
        Some(Value::Number(n)) => match n.as_f64() {
            Some(v) if v.is_finite() => v,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

/// The shared `!Number.isFinite(presentedEpoch) || presentedEpoch <
/// currentEpoch` guard behind renewClaimTTL and releaseClaim. `verb` is the
/// only difference between the two refusal texts ("renew" / "release").
pub(crate) fn claim_fence_refusal(
    verb: &str,
    cell: &str,
    presented: &Value,
    claim: &Map<String, Value>,
) -> Option<ClaimRefusal> {
    let current = current_fence_epoch(claim);
    let presented_num = match presented {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        _ => f64::NAN, // Number.isFinite(non-number) === false
    };
    if presented_num.is_finite() && presented_num >= current {
        return None;
    }
    let mut refusal = ClaimRefusal::new(
        "CLAIM_FENCE_STALE",
        format!(
            "cell \"{cell}\" {verb} refused: presented epoch {} is behind current fence_epoch {} — a takeover already moved ownership forward; re-adopt before writing again.",
            jsjson::stringify(presented),
            jsjson::js_f64_to_string(current)
        ),
    );
    refusal.extra.insert("cell".into(), Value::String(cell.to_string()));
    refusal.extra.insert(
        "current_epoch".into(),
        Number::from_f64(current).map(Value::Number).unwrap_or(Value::Null),
    );
    refusal.extra.insert("presented_epoch".into(), presented.clone());
    Some(refusal)
}

/// claims.mjs adoptClaim — transfer ownership IN PLACE under the exclusive
/// gate: the claim file is atomically REWRITTEN, never deleted, so concurrent
/// 'wx' claimers keep getting EEXIST throughout the adoption. Every adoption
/// bumps `fence_epoch` by exactly 1 in the same atomic write as the ownership
/// change, which is what makes a stale holder's later renew/release
/// detectable at all.
///
/// Not called from a verb in THIS module: the one native caller today is
/// `state handoff adopt`, which lives in verbs/state_group.rs (owned by
/// another in-flight cell) and drives its own narrowed twin. This is the full
/// claims-module contract, byte-pinned against the live Node oracle.
#[allow(dead_code)]
pub(crate) fn adopt_claim(
    control: &Path,
    cell_id: &str,
    new_session_id: &str,
) -> MR<AdoptClaimOutcome> {
    let cell = require_id(cell_id, "cell id")?;
    let session = require_id(new_session_id, "session id")?;
    let _ = std::fs::create_dir_all(claims_dir(control));
    if !acquire_gate(control, &cell)? {
        return Ok(AdoptClaimOutcome::Refused(ClaimRefusal::new(
            "GATE_HELD",
            format!(
                "claim \"{cell}\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
            ),
        )));
    }
    let outcome = (|| -> MR<AdoptClaimOutcome> {
        let Some(claim) = read_claim(control, &cell)? else {
            return Ok(AdoptClaimOutcome::Refused(ClaimRefusal::new(
                "NOT_FOUND",
                format!("cell \"{cell}\" has no claim to adopt."),
            )));
        };
        let previous = claim.get("session").cloned();
        let previous_epoch = current_fence_epoch(&claim);
        let now = utc_now();
        // `{...claim, session, claimed_at, adopted_from, adopted_at,
        // fence_epoch}` — a re-assigned key keeps its ORIGINAL position, and
        // `adopted_from: undefined` (no previous owner) is dropped wholesale
        // by JSON.stringify rather than written as null.
        let mut adopted = claim.clone();
        adopted.insert("session".into(), Value::String(session));
        adopted.insert("claimed_at".into(), Value::String(now.clone())); // fresh ownership renews the TTL clock
        match &previous {
            Some(prev) => {
                adopted.insert("adopted_from".into(), prev.clone());
            }
            None => {
                adopted.shift_remove("adopted_from");
            }
        }
        adopted.insert("adopted_at".into(), Value::String(now));
        adopted.insert(
            "fence_epoch".into(),
            Value::Number(Number::from_f64(previous_epoch + 1.0).ok_or(Fail::Delegate)?),
        );
        let adopted = Value::Object(adopted);
        let file = claim_path(control, &cell)?;
        transient_fs_retry(|| write_json_atomic(&file, &adopted))
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
        Ok(AdoptClaimOutcome::Ok { claim: adopted, previous_owner: previous })
    })();
    release_gate(control, &cell); // `finally` — the gate is never leaked
    outcome
}

/// claims.mjs renewClaimTTL — same-session-only TTL renewal: refreshes
/// `claimed_at` for every claim owned by `session`, never touching
/// adopted_from/adopted_at and never bumping `fence_epoch`. A claim whose gate
/// is held by another in-flight adopt/sweep is SKIPPED, never waited on. The
/// session match is RE-VERIFIED under the gate, so a renewal racing an
/// adoption can never revert ownership.
///
/// `presented_epoch` is OPTIONAL and OFF by default (the shape every
/// production caller uses today). Presented and behind a reached claim's
/// current epoch, the WHOLE call refuses typed CLAIM_FENCE_STALE rather than
/// silently completing a partial renewal of the others.
///
/// Node's production caller is `heartbeatTouch` (claims.mjs), reached from
/// the bee-state-sync hook — whose own narrowed heartbeat path lives in
/// src/hooks/state_sync.rs, outside this cell's touchable set. This is the
/// full contract including the fencing arm that hook does not present.
#[allow(dead_code)]
pub(crate) fn renew_claim_ttl(
    control: &Path,
    session_id: &str,
    presented_epoch: Option<&Value>,
) -> MR<RenewClaimOutcome> {
    let session = require_id(session_id, "session id")?;
    let Ok(entries) = std::fs::read_dir(claims_dir(control)) else {
        return Ok(RenewClaimOutcome::Ok { renewed: Vec::new(), skipped: Vec::new() });
    };
    // readdirSync yields directory order; collect first so the gate work does
    // not hold the directory handle open (a Windows sharing hazard).
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.retain(|n| n.ends_with(".json"));
    let mut renewed = Vec::new();
    let mut skipped = Vec::new();
    for name in names {
        let cell = name[..name.len() - ".json".len()].to_string();
        // readClaim → claimPath → requireId: a file name that is not a plain
        // id throws in Node (uncaught, so the whole call throws) — mirrored.
        let preview = read_claim(control, &cell)?;
        match preview.as_ref().and_then(|c| c.get("session")) {
            Some(Value::String(s)) if *s == session => {}
            _ => continue, // not ours (or sessionless): never touched
        }
        if !acquire_gate(control, &cell)? {
            skipped.push(cell);
            continue;
        }
        let step = (|| -> MR<RenewStep> {
            let Some(claim) = read_claim(control, &cell)? else { return Ok(RenewStep::Untouched) };
            match claim.get("session") {
                Some(Value::String(s)) if *s == session => {}
                _ => return Ok(RenewStep::Untouched), // adopted away between listing and gating
            }
            if let Some(presented) = presented_epoch {
                if let Some(refusal) = claim_fence_refusal("renew", &cell, presented, &claim) {
                    return Ok(RenewStep::Refused(refusal));
                }
            }
            // The `...claim` spread carries acquired_at AND fence_epoch
            // forward untouched — only claimed_at, the expiry clock, advances.
            let mut next = claim.clone();
            next.insert("claimed_at".into(), Value::String(utc_now()));
            let file = claim_path(control, &cell)?;
            transient_fs_retry(|| write_json_atomic(&file, &Value::Object(next.clone())))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            Ok(RenewStep::Renewed)
        })();
        release_gate(control, &cell); // `finally`, even on the fenced refusal
        match step? {
            RenewStep::Refused(refusal) => return Ok(RenewClaimOutcome::Refused(refusal)),
            RenewStep::Renewed => renewed.push(cell),
            RenewStep::Untouched => {}
        }
    }
    Ok(RenewClaimOutcome::Ok { renewed, skipped })
}

#[allow(dead_code)] // Refused is only constructed on the fencing arm
pub(crate) enum RenewStep {
    Renewed,
    Untouched,
    Refused(ClaimRefusal),
}

/// claims.mjs releaseClaim — owner-matched removal under the same exclusive
/// gate as adopt/sweep, with the msn-12 fencing guard checked AFTER the owner
/// check (epoch fencing is an additional, orthogonal guard, never a
/// substitute for ownership). A fenced refusal leaves the claim file
/// untouched.
pub(crate) fn release_claim_typed(
    control: &Path,
    session: Option<&str>,
    cell_id: &str,
    presented_epoch: Option<&Value>,
) -> MR<ReleaseClaimOutcome> {
    let cell = require_id(cell_id, "cell id")?;
    let not_found = || {
        ReleaseClaimOutcome::Refused(ClaimRefusal::new(
            "NOT_FOUND",
            format!("cell \"{cell}\" has no claim to release."),
        ))
    };
    if read_claim(control, &cell)?.is_none() {
        return Ok(not_found());
    }
    if !acquire_gate_with_retry(control, &cell)? {
        return Ok(ReleaseClaimOutcome::Refused(ClaimRefusal::new(
            "GATE_HELD",
            format!(
                "claim \"{cell}\" is gated by another in-flight adopt/sweep/release after {GATE_RETRY_ATTEMPTS} bounded retries — never waited unboundedly."
            ),
        )));
    }
    let outcome = (|| -> MR<ReleaseClaimOutcome> {
        let Some(claim) = read_claim(control, &cell)? else { return Ok(not_found()) };
        let owner: Option<String> = match claim.get("session") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Null) | None => None,
            Some(_) => return Err(Fail::Delegate), // non-string session — JS-exotic compare
        };
        if owner.as_deref() != session {
            return Ok(ReleaseClaimOutcome::Refused(ClaimRefusal::new(
                "NOT_OWNER",
                format!(
                    "cell \"{cell}\" is owned by session \"{}\", not \"{}\".",
                    owner.as_deref().unwrap_or("none (sessionless)"),
                    session.unwrap_or("none (sessionless)")
                ),
            )));
        }
        if let Some(presented) = presented_epoch {
            if let Some(refusal) = claim_fence_refusal("release", &cell, presented, &claim) {
                return Ok(ReleaseClaimOutcome::Refused(refusal));
            }
        }
        let file = claim_path(control, &cell)?;
        let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
        Ok(ReleaseClaimOutcome::Ok { released: Value::Object(claim) })
    })();
    release_gate(control, &cell);
    outcome
}

/// The claim-unwind path of claimCellCrossSession: the caller ignores the
/// typed result (only the disk effect must match Node), and never fences.
pub(crate) fn release_claim(control: &Path, session: Option<&str>, cell_id: &str) -> MR<()> {
    release_claim_typed(control, session, cell_id, None).map(|_| ())
}

// ─── sessions (claims.mjs) ─────────────────────────────────────────────────

/// claims.mjs readSession (fail-open flavor): malformed id -> None; corrupt
/// record -> warn + None (readJson's fallback); id-mismatch -> None.
pub(crate) fn read_session(control: &Path, session_id: &str) -> MR<Option<Map<String, Value>>> {
    let trimmed = js_trim(session_id);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Ok(None); // sessionPath's requireId throw, caught -> null
    }
    let file = sessions_dir(control).join(format!("{trimmed}.json"));
    match read_store_json(&file)? {
        None => Ok(None),
        Some(Value::Object(m)) => {
            if matches!(m.get("id"), Some(Value::String(s)) if s == trimmed) {
                Ok(Some(m))
            } else {
                Ok(None)
            }
        }
        Some(Value::Array(_)) => Err(Fail::Delegate), // typeof 'object' — JS-exotic
        Some(_) => Ok(None),
    }
}

/// claims.mjs listSessionRecords (fail-open): missing dir -> empty.
pub(crate) fn list_session_records(control: &Path) -> MR<Vec<Map<String, Value>>> {
    let dir = sessions_dir(control);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    // fs.readdirSync returns names sorted per OS; both runtimes read the same
    // order — only the record set matters to every consumer here.
    names.sort();
    let mut out = Vec::new();
    for name in names {
        if !name.ends_with(".json") {
            continue;
        }
        let stem = &name[..name.len() - ".json".len()];
        if let Some(record) = read_session(control, stem)? {
            out.push(record);
        }
    }
    Ok(out)
}

/// claims.mjs heartbeatStale. A session record marked "closed" (clean
/// SessionEnd) or "dead" (crash-swept) reads as stale unconditionally — its
/// heartbeat recency no longer matters once the record itself says the
/// session is over.
pub(crate) fn heartbeat_stale(session: Option<&Map<String, Value>>, now: f64) -> MR<bool> {
    let Some(session) = session else { return Ok(true) };
    if matches!(session.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
        return Ok(true);
    }
    match rsv::date_parse_val(session.get("last_heartbeat")).map_err(|_| Fail::Delegate)? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

/// claims.mjs isConcurrentMode (no exclusion).
pub(crate) fn is_concurrent_mode(control: &Path) -> MR<bool> {
    let now = rsv::now_ms();
    for record in list_session_records(control)? {
        if !heartbeat_stale(Some(&record), now)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// claims.mjs resolveSessionId flag -> BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID
/// (no `root`: the durable single-live-session fallback stays with callers
/// that pass one).
pub(crate) fn resolve_session_flag_env(flag: Option<&str>) -> Option<String> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Some(js_trim(f).to_string());
        }
    }
    env_nonempty("BEE_SESSION_ID").or_else(|| env_nonempty("CLAUDE_CODE_SESSION_ID"))
}

/// resolveSessionId's durable fallback half ({root, audit}) — exactly one
/// fresh live session record adopts; anything else resolves None.
pub(crate) fn resolve_session_adopt(control: &Path) -> MR<Option<String>> {
    let now = rsv::now_ms();
    let mut fresh: Vec<Map<String, Value>> = Vec::new();
    for record in list_session_records(control)? {
        if !heartbeat_stale(Some(&record), now)? {
            fresh.push(record);
        }
    }
    if fresh.len() == 1 {
        if let Some(Value::String(id)) = fresh[0].get("id") {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

/// claimCellFile's typed outcome. The Ok claim payload mirrors Node's
/// {ok:true, claim} return; the CLI handler never reads it (tests do).
#[allow(dead_code)]
pub(crate) enum ClaimFileOutcome {
    Ok { claim: Value },
    Refused { code: &'static str, reason: String },
}

/// claims.mjs claimCellFile — the O_EXCL one-winner primitive: fence_epoch
/// stamped 1 at creation, acquired_at == claimed_at, session/workspace_id/
/// adopted omitted (never null) when absent.
pub(crate) fn claim_cell_file(
    control: &Path,
    session_in: Option<&str>,
    cell_id: &str,
    ttl: Option<f64>,
) -> MR<ClaimFileOutcome> {
    let explicit = match session_in {
        None => None,
        Some(s) => Some(require_id(s, "session id")?),
    };
    let cell = require_id(cell_id, "cell id")?;
    let mut session = explicit;
    let mut adopted = false;
    if session.is_none() {
        if let Some(candidate) = resolve_session_adopt(control)? {
            session = Some(candidate);
            adopted = true;
        } else if is_concurrent_mode(control)? {
            return Ok(ClaimFileOutcome::Refused {
                code: "SESSION_REQUIRED",
                reason: format!(
                    "cell \"{cell}\" cannot be claimed without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored)."
                ),
            });
        }
    }
    let _ = std::fs::create_dir_all(claims_dir(control));
    // msn-19: workspace_id auto-looked-up from the acting session's record.
    let mut workspace_id: Option<String> = None;
    if let Some(s) = &session {
        if let Some(record) = read_session(control, s)? {
            if let Some(Value::String(w)) = record.get("workspace_id") {
                if !w.is_empty() {
                    workspace_id = Some(w.clone());
                }
            }
        }
    }
    let mut claim = Map::new();
    claim.insert("cell".into(), Value::String(cell.clone()));
    if let Some(s) = &session {
        claim.insert("session".into(), Value::String(s.clone()));
    }
    if let Some(w) = &workspace_id {
        claim.insert("workspace_id".into(), Value::String(w.clone()));
    }
    let ttl_value = match ttl {
        Some(t) if t.is_finite() && t > 0.0 => t.floor(),
        _ => DEFAULT_CLAIM_TTL_SECONDS,
    };
    claim.insert(
        "ttl_seconds".into(),
        Value::Number(Number::from_f64(ttl_value).ok_or(Fail::Delegate)?),
    );
    let now = utc_now();
    claim.insert("claimed_at".into(), Value::String(now.clone()));
    claim.insert("acquired_at".into(), Value::String(now));
    claim.insert("fence_epoch".into(), Value::Number(Number::from_f64(1.0).unwrap()));
    if adopted {
        claim.insert("adopted".into(), Value::Bool(true));
    }
    let claim = Value::Object(claim);
    let file = claim_path(control, &cell)?;
    let body = format!("{}\n", jsjson::stringify_pretty(&claim));
    let write = std::fs::OpenOptions::new().write(true).create_new(true).open(&file).and_then(|mut f| {
        use std::io::Write;
        f.write_all(body.as_bytes())
    });
    match write {
        Ok(()) => Ok(ClaimFileOutcome::Ok { claim }),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let holder = read_claim(control, &cell)?;
            let owner = match holder.as_ref().and_then(|h| h.get("session")) {
                Some(Value::Null) | None => "no session (sessionless claim)".to_string(),
                Some(v) => jsjson::js_to_string(v),
            };
            Ok(ClaimFileOutcome::Refused {
                code: "CLAIMED",
                reason: format!(
                    "cell \"{cell}\" is already claimed by session \"{owner}\" ({}).",
                    claim_expiry(holder.as_ref())?
                ),
            })
        }
        Err(e) => Err(Fail::Thrown(format!("{e}"))), // Node rethrows raw fs errors
    }
}
