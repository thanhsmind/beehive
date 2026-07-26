//! claims — `readSession`/`heartbeatStale` port over `.bee/sessions/*.json`,
//! from `.bee/bin/lib/claims.mjs` (rust-port-8, CONTEXT.md D3), plus
//! (rust-port-17) the ONE write path bee-state-sync.mjs's `heartbeatTouch`
//! composes: a throttled `heartbeatSession` write plus a `renewClaimTTL`
//! sweep. The rest of claims.mjs's mutating surface (create/adopt/release/
//! sweep, epoch fencing, lane binding) stays unported — this cell's action
//! names only the hook-driven call site, which never presents an epoch and
//! never touches session/claim creation.
//!
//! `.bee/bin/lib/claims.mjs` is FROZEN for the duration of the rust-port
//! feature (D1).

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::fsutil::{read_json, write_json_atomic};
use crate::jsdate::parse_iso_ms;
use crate::lock::{iso8601_millis, with_store_lock, LockOptions, WithLockError};

/// `DEFAULT_HEARTBEAT_STALE_SECONDS`.
pub const DEFAULT_HEARTBEAT_STALE_SECONDS: i64 = 900;

pub fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

pub fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// `sessionPath(root, sessionId)`. Note this reader does not replicate
/// `requireId`'s validation-and-throw (`readSession` itself catches that
/// and returns `null` on a malformed id — the "fail-open: a malformed id
/// reads as 'no session'" comment on `readSession` below) — a caller here
/// gets `None` from [`read_session`] for the same malformed-id case rather
/// than a panic.
pub fn session_path(root: &Path, session_id: &str) -> PathBuf {
    sessions_dir(root).join(format!("{session_id}.json"))
}

pub fn claim_path(root: &Path, cell_id: &str) -> PathBuf {
    claims_dir(root).join(format!("{cell_id}.json"))
}

/// `.bee/sessions/<id>.json`: `{id, started_at, last_heartbeat, ...}`.
///
/// `lane`/`transcript_path` are named explicitly (rust-port-20:
/// `buildLaneRows`/`buildLaneSummary` read `session.lane`;
/// `detectCrashCandidates` reads `session.transcript_path` as its
/// authoritative, checked-first transcript location, D5 hardening-1-7-10)
/// — both previously only reachable via `extra`. Adding a named field never
/// loses round-trip: `#[serde(flatten)]` only collects keys NOT already
/// claimed by a named field, so existing consumers that read
/// `extra.get("lane")` must move to the named field (done: `state::
/// resolve_pipeline`) rather than silently seeing an empty `extra` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub last_heartbeat: Option<String>,
    #[serde(default)]
    pub lane: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `readSession(root, sessionId)`: `None` when the record is missing,
/// malformed, or its own `id` field doesn't match the requested id
/// (mirrors mjs's `session.id !== String(sessionId).trim()` guard, which
/// catches a stale/misnamed file). A blank/empty `session_id` is likewise
/// `None` (mirrors the malformed-id fail-open branch).
pub fn read_session(root: &Path, session_id: &str) -> Option<Session> {
    let trimmed = session_id.trim();
    if trimmed.is_empty() {
        return None;
    }
    let raw: Value = read_json(&session_path(root, trimmed), Value::Null);
    if !raw.is_object() {
        return None;
    }
    let session: Session = serde_json::from_value(raw).ok()?;
    if session.id != trimmed {
        return None;
    }
    Some(session)
}

/// `listSessionRecords(root)`: every readable `.bee/sessions/*.json`
/// record. Fail-open per file and for a missing directory (empty list),
/// same posture as `readSession`.
pub fn list_session_records(root: &Path) -> Vec<Session> {
    let dir = sessions_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut sessions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if let Ok(session) = serde_json::from_str::<Session>(&text) {
            sessions.push(session);
        }
    }
    sessions
}

/// `heartbeatStale(session, nowMs, staleSeconds)`: `true` when the
/// session is absent/malformed, has no parseable `last_heartbeat`, or the
/// heartbeat is older than `stale_seconds`.
pub fn heartbeat_stale(session: Option<&Session>, now_ms: i64, stale_seconds: i64) -> bool {
    let session = match session {
        Some(s) => s,
        None => return true,
    };
    let beat_ms = match session.last_heartbeat.as_deref().and_then(parse_iso_ms) {
        Some(ms) => ms,
        None => return true,
    };
    beat_ms + stale_seconds * 1000 <= now_ms
}

// ─── claims (rust-port-20) ───────────────────────────────────────────────
// Port of claims.mjs `readClaim`/`isClaimActive` — `detectCrashCandidates`'s
// `sessionHasActiveClaim` work-signal check reads these. Read-only; the
// mutating claim/adopt/release surface stays out of scope here (bee-swarming
// owns that verb group).

/// `.bee/claims/<cellId>.json`: `{session, claimed_at, ttl_seconds, ...}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub claimed_at: Option<String>,
    #[serde(default)]
    pub ttl_seconds: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `readClaim(root, cellId)`: `None` when the record is missing or not a
/// JSON object — fail-open, same posture as [`read_session`].
pub fn read_claim(root: &Path, cell_id: &str) -> Option<Claim> {
    let raw: Value = read_json(&claim_path(root, cell_id), Value::Null);
    if !raw.is_object() {
        return None;
    }
    serde_json::from_value(raw).ok()
}

/// `isClaimExpired`: TTL semantics mirror reservations.mjs — a
/// non-positive/invalid/unparseable `ttl_seconds` or `claimed_at` never
/// expires (fail-open: an unreadable expiry looks like "still held").
fn is_claim_expired(claim: &Claim, now_ms: i64) -> bool {
    let ttl = match claim.ttl_seconds.as_ref().and_then(Value::as_f64) {
        Some(t) if t.is_finite() && t > 0.0 => t,
        _ => return false,
    };
    let claimed_ms = match claim.claimed_at.as_deref().and_then(parse_iso_ms) {
        Some(ms) => ms,
        None => return false,
    };
    claimed_ms as f64 + ttl * 1000.0 <= now_ms as f64
}

/// `isClaimActive(claim, nowMs)`.
pub fn is_claim_active(claim: &Claim, now_ms: i64) -> bool {
    !is_claim_expired(claim, now_ms)
}

// ─── heartbeat + claim-TTL renewal (rust-port-17) ────────────────────────
// Port of the ONE write path bee-state-sync.mjs's `heartbeatTouch` composes:
// a throttled session-heartbeat write (`heartbeatSession`) plus a
// same-session claim-TTL sweep (`renewClaimTTL`). Scoped deliberately to
// what the hook actually exercises — `presentedEpoch` fencing is never
// used on this call path (the hook never presents one), so it is not
// ported here; see this module's header for the fuller scope note.

/// claims.mjs's `HEARTBEAT_TOUCH_THROTTLE_SECONDS` — `heartbeat_touch` below
/// no-ops unless the stored heartbeat is older than this.
pub const HEARTBEAT_TOUCH_THROTTLE_SECONDS: i64 = 60;

/// claims.mjs's `SESSIONS_LOCK_NAME` — the same D9 store lock name
/// `heartbeatSession`'s write and `bindSessionLane`/`unbindSessionLane`
/// (unported: CLI-only) all share, so a real node holder of this lock
/// denies a concurrent Rust write and vice versa (genuine cross-runtime
/// interop, not a same-runtime-only approximation).
const SESSIONS_LOCK_NAME: &str = "sessions";

fn claim_gate_path(root: &Path, cell_id: &str) -> PathBuf {
    claims_dir(root).join(format!("{cell_id}.adopting"))
}

/// `acquireGate(root, cellId, nowMs)` — an exclusive `wx`-flag create
/// (Rust's `create_new`): `false` on `EEXIST` (gate already held), `true`
/// once this call's own body is written. `renewClaimTTL` calls this ONCE per
/// claim, never through the retrying `acquireGateWithRetry` wrapper (that
/// wrapper is CLI-only, unported here) — a held gate is simply skipped.
fn acquire_gate(root: &Path, cell_id: &str, now_ms: i64) -> bool {
    let path = claim_gate_path(root, cell_id);
    let Some(parent) = path.parent() else { return false };
    if fs::create_dir_all(parent).is_err() {
        return false;
    }
    match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut f) => {
            let body = json!({ "pid": std::process::id(), "at": iso8601_millis(now_ms) });
            let _ = writeln!(f, "{body}");
            true
        }
        Err(_) => false,
    }
}

/// `releaseGate(root, cellId)` — best-effort remove; a missing gate file is
/// not an error (mirrors the mjs source's `{ force: true }`).
fn release_gate(root: &Path, cell_id: &str) {
    let _ = fs::remove_file(claim_gate_path(root, cell_id));
}

/// Port of `heartbeatSession(root, sessionId, { now, lockAttempts })`'s
/// write step: renews `session.last_heartbeat` under [`SESSIONS_LOCK_NAME`].
/// `Ok(Ok(false))` mirrors the mjs `SESSION_MISSING` typed failure (no
/// record to heartbeat — never written); `Err(WithLockError::Busy { .. })`
/// mirrors the mjs `LOCK_BUSY` typed failure. `lock_attempts` mirrors the
/// mjs source's caller-supplied budget — the hook call site below always
/// passes `1` (Δ3: "hooks never wait").
///
/// D3 byte-compatibility: the rewrite patches the RAW `serde_json::Value`
/// in place rather than round-tripping through the typed [`Session`]
/// struct. `Session` names `lane`/`transcript_path` as `Option<String>`
/// WITHOUT `skip_serializing_if` (so a typed round-trip always emits the
/// key, `null` when absent) — but the mjs source's own session records
/// OMIT those keys entirely whenever unset (`createSession`'s conditional
/// spread), and `heartbeatSession`'s real write is `session.last_heartbeat
/// = ...; writeJsonAtomic(file, session)` on the object `readSession`
/// handed back, which never introduces a key that was not already there. A
/// typed round-trip here would silently inject `"lane": null,
/// "transcript_path": null` into every session record this hook ever
/// touches — caught by this cell's own oracle diff
/// (`heavyhooks_conformance.rs`), fixed by staying in `Value` space for the
/// write.
pub fn heartbeat_session(root: &Path, session_id: &str, now_ms: i64, lock_attempts: u32) -> Result<io::Result<bool>, WithLockError> {
    let options = LockOptions { max_attempts: lock_attempts, ..LockOptions::default() };
    with_store_lock(root, SESSIONS_LOCK_NAME, options, || -> io::Result<bool> {
        let trimmed = session_id.trim();
        let raw: Value = read_json(&session_path(root, trimmed), Value::Null);
        let Value::Object(mut map) = raw else {
            return Ok(false); // SESSION_MISSING (or malformed) — never written
        };
        if map.get("id").and_then(Value::as_str) != Some(trimmed) {
            return Ok(false); // stale/misnamed file — same fail-open guard readSession applies
        }
        map.insert("last_heartbeat".to_string(), Value::String(iso8601_millis(now_ms)));
        write_json_atomic(&session_path(root, trimmed), &Value::Object(map))?;
        Ok(true)
    })
}

/// Port of `renewClaimTTL(root, sessionId, { now })` (no `presentedEpoch` —
/// unused by this call path): bumps `claimed_at` (the expiry clock) on
/// every LIVE claim file this session owns, under that claim's own
/// single-attempt gate — a held gate is skipped, never waited on. Returns
/// the renewed cell ids (informational; the hook itself never inspects
/// this).
pub fn renew_claim_ttl(root: &Path, session_id: &str, now_ms: i64) -> Vec<String> {
    let dir = claims_dir(root);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut renewed = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(cell) = path.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
            continue;
        };
        let Some(preview) = read_claim(root, &cell) else {
            continue; // not ours (or sessionless): never touched
        };
        if preview.session.as_deref() != Some(session_id) {
            continue;
        }
        if !acquire_gate(root, &cell, now_ms) {
            continue; // gate held elsewhere — skipped, never waited on
        }
        // Re-verify ownership under the gate (mirrors the mjs source's own
        // re-read) before writing — patching the RAW value, same
        // byte-compatibility reasoning as `heartbeat_session` above: a
        // typed `Claim` round-trip would emit `null` for any of its
        // explicitly-named optional fields the on-disk record happens to
        // omit, where the mjs source's own `{ ...claim, claimed_at }`
        // spread never introduces a key that was not already there.
        let raw: Value = read_json(&claim_path(root, &cell), Value::Null);
        if let Value::Object(mut map) = raw {
            if map.get("session").and_then(Value::as_str) == Some(session_id) {
                map.insert("claimed_at".to_string(), Value::String(iso8601_millis(now_ms)));
                if write_json_atomic(&claim_path(root, &cell), &Value::Object(map)).is_ok() {
                    renewed.push(cell.clone());
                }
            }
        }
        release_gate(root, &cell);
    }
    renewed
}

/// `heartbeatTouch(root, sessionId)` result — only the field
/// bee-state-sync.mjs's caller actually branches on (`touch.touched`); the
/// nested `heartbeat`/`claims` sub-results are never inspected downstream
/// of that gate, so they are not modeled here (the WRITES they'd describe
/// already happened as real side effects by the time this returns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartbeatTouchResult {
    pub touched: bool,
}

/// Port of `heartbeatTouch(root, sessionId)`: throttled (no-op unless the
/// stored heartbeat is older than [`HEARTBEAT_TOUCH_THROTTLE_SECONDS`]) —
/// once through the throttle, `touched` is `true` regardless of whether the
/// underlying write actually lands (mirrors the mjs source exactly: a
/// `SESSION_MISSING` or `LOCK_BUSY` typed failure from `heartbeatSession`
/// still leaves `touched: true`, since the caller only gates on "did we
/// attempt", never "did the write succeed").
pub fn heartbeat_touch(root: &Path, session_id: &str, now_ms: i64) -> HeartbeatTouchResult {
    let session = session_id.trim();
    if session.is_empty() {
        return HeartbeatTouchResult { touched: false };
    }
    let record = read_session(root, session);
    if !heartbeat_stale(record.as_ref(), now_ms, HEARTBEAT_TOUCH_THROTTLE_SECONDS) {
        return HeartbeatTouchResult { touched: false };
    }
    let _ = heartbeat_session(root, session, now_ms, 1);
    let _ = renew_claim_ttl(root, session, now_ms);
    HeartbeatTouchResult { touched: true }
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
//
// (rust-port-17's own heartbeat/claim-renewal writes above are proved
// against the real mjs oracle by crates/queen-bee/tests/
// heavyhooks_conformance.rs, this cell's mandated single integration
// target — side-effect parity on seeded fixtures, not here.)
