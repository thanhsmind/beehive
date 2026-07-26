//! reservations — readers for the same-session file-reservation stores,
//! ported from `.bee/bin/lib/reservations.mjs` and `.bee/bin/lib/
//! lease-store.mjs` (rust-port-8, CONTEXT.md D3), plus (rust-port-17) the
//! ONE write path bee-state-sync.mjs's hook call site exercises:
//! `renewHoldsBySession`, itself a thin per-record loop over lease-store.mjs's
//! `renewLease`. Every other lease-store mutation (`acquireLeases`,
//! `releaseLease`, epoch fencing) stays unported — the hook never presents
//! an epoch and never acquires/releases a lease.
//!
//! multisession-native-16 demoted `.bee/reservations.json` to a
//! rebuildable PROJECTION for legacy readers only — the live storage is
//! now sharded per-resource lease files under `.bee/runtime/leases/`
//! (`.bee/runtime/leases/cells/<cell-id>.json`,
//! `.bee/runtime/leases/paths/<path-hash>.json`). This module reads BOTH,
//! as the cell's panel named ("reservations.json plus sharded lease
//! records listing"): [`read_reservations_projection`] for the legacy
//! single-file shape, [`list_leases`] for the current sharded store.
//!
//! `.bee/bin/lib/reservations.mjs`/`lease-store.mjs` are FROZEN for the
//! duration of the rust-port feature (D1). Every struct round-trips
//! unknown fields via `extra` (D3).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::fsutil::{read_json, write_json_atomic};
use crate::jsdate::parse_iso_ms;
use crate::lock::{iso8601_millis, with_store_lock, LockOptions, WithLockError};

pub fn reservations_path(root: &Path) -> PathBuf {
    root.join(".bee").join("reservations.json")
}

pub fn leases_root(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("leases")
}

/// One row of the legacy `.bee/reservations.json` projection:
/// `{agent, cell, path, ttl_seconds, reserved_at, released_at, kind?}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub cell: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub ttl_seconds: Option<Value>,
    #[serde(default)]
    pub reserved_at: Option<String>,
    #[serde(default)]
    pub released_at: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReservationsStore {
    #[serde(default)]
    reservations: Vec<Reservation>,
}

/// Reads the legacy `.bee/reservations.json` projection. Fail-open: a
/// missing or malformed store reads as an empty list, matching
/// reservations.mjs's `readStore` posture (never a mutator's concern here —
/// this reader never rebuilds or writes the projection).
pub fn read_reservations_projection(root: &Path) -> Vec<Reservation> {
    let raw: Value = read_json(&reservations_path(root), Value::Null);
    if !raw.is_object() {
        return Vec::new();
    }
    serde_json::from_value::<ReservationsStore>(raw)
        .map(|s| s.reservations)
        .unwrap_or_default()
}

/// One `.bee/runtime/leases/{cells,paths}/<hash>.json` record:
/// `{resource, mode, workflow_id, session_id, workspace_id, epoch,
/// acquired_at, expires_at, kind}` (lease-store.mjs module header).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRecord {
    /// Type-prefixed resource key: `"cell:<cell-id>"` or
    /// `"path:<canonical-path>"`.
    pub resource: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub epoch: Option<Value>,
    #[serde(default)]
    pub acquired_at: Option<String>,
    /// ISO timestamp, or `null` meaning "never expires".
    #[serde(default)]
    pub expires_at: Option<Value>,
    /// `'intent'` (advisory broad/glob scope) or `'lease'` (an exact path a
    /// writer is about to touch); defaults to `'lease'` when the source
    /// record omits it, matching lease-store.mjs.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Lists every readable lease record under `.bee/runtime/leases/cells/`
/// and `.bee/runtime/leases/paths/` (both subdirectories, when present).
/// Fail-open per file and per missing subdirectory — a repo mid-migration
/// with only one of the two subdirs present reads the other as empty
/// rather than erroring.
pub fn list_leases(root: &Path) -> Vec<LeaseRecord> {
    let base = leases_root(root);
    let mut leases = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = base.join(sub);
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if let Ok(record) = serde_json::from_str::<LeaseRecord>(&text) {
                leases.push(record);
            }
        }
    }
    leases
}


// ─── current lease-backed listReservations (rust-port-20) ───────────────
// reservations.mjs's ACTUAL `listReservations` today (post multisession-
// native-16) reads the sharded lease store, not the legacy projection file
// above — `bee.mjs`'s `buildStatus` calls THIS shape. Distinct struct
// ([`LeaseReservationView`]) rather than reusing [`Reservation`]: that
// struct's job is deserializing the legacy on-disk projection (round-trip
// via `extra`), never constructing a fresh object with JS's
// undefined-key-drop semantics, which needs `skip_serializing_if` on every
// optional field instead.

pub const PATH_RESOURCE_PREFIX: &str = "path:";
const AGENT_WORKSPACE_PREFIX: &str = "agent:";
/// reservations.mjs `SESSIONLESS_SESSION_ID` — U+0000-delimited so no real
/// session id could ever collide with it.
const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";
const DEFAULT_RESERVATION_KIND: &str = "lease";

fn is_path_lease(record: &LeaseRecord) -> bool {
    record.resource.starts_with(PATH_RESOURCE_PREFIX)
}

/// `isLeaseRecordExpired(record, nowMs)`.
fn is_lease_record_expired(record: &LeaseRecord, now_ms: i64) -> bool {
    match &record.expires_at {
        None | Some(Value::Null) => false, // no record, or "never expires"
        Some(v) => match v.as_str().and_then(parse_iso_ms) {
            Some(ms) => ms <= now_ms,
            None => false,
        },
    }
}

/// `leaseAgent(record)`.
fn lease_agent(record: &LeaseRecord) -> Option<String> {
    record
        .workspace_id
        .as_deref()
        .map(|w| w.strip_prefix(AGENT_WORKSPACE_PREFIX).map(str::to_string).unwrap_or_else(|| w.to_string()))
}

/// `leaseTtlSeconds(record)` — 0 sentinel: never expires.
fn lease_ttl_seconds(record: &LeaseRecord) -> i64 {
    match &record.expires_at {
        Some(Value::String(s)) => {
            let expires = parse_iso_ms(s);
            let acquired = record.acquired_at.as_deref().and_then(parse_iso_ms);
            match (expires, acquired) {
                (Some(e), Some(a)) => ((e - a) as f64 / 1000.0).round().max(0.0) as i64,
                _ => 0,
            }
        }
        _ => 0,
    }
}

/// One row of the CURRENT lease-backed `listReservations(root, {activeOnly})`
/// shape (rust-port-20) — built fresh from a [`LeaseRecord`] via
/// [`lease_to_reservation`], never deserialized from disk directly, so
/// every optional field uses `skip_serializing_if` to mirror JS's
/// undefined-key-drop-on-JSON.stringify semantics exactly (mjs never
/// emits `"agent": null` — an unset agent is simply an absent key).
#[derive(Debug, Clone, Serialize)]
pub struct LeaseReservationView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<String>,
    pub path: String,
    pub ttl_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved_at: Option<String>,
    /// Always `null`: a released lease file is DELETED, never
    /// soft-deleted (`leaseToReservation`'s own comment) — every row this
    /// reader can still see is, by construction, not-yet-released.
    pub released_at: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    pub kind: String,
}

/// Port of `leaseToReservation(record)`.
pub fn lease_to_reservation(record: &LeaseRecord) -> LeaseReservationView {
    let session = record.session_id.clone().filter(|s| !s.is_empty() && s != SESSIONLESS_SESSION_ID);
    LeaseReservationView {
        agent: lease_agent(record),
        cell: record.workflow_id.clone(),
        path: record.resource.strip_prefix(PATH_RESOURCE_PREFIX).unwrap_or("").to_string(),
        ttl_seconds: lease_ttl_seconds(record),
        reserved_at: record.acquired_at.clone(),
        released_at: Value::Null,
        session,
        kind: record.kind.clone().filter(|k| !k.is_empty()).unwrap_or_else(|| DEFAULT_RESERVATION_KIND.to_string()),
    }
}

/// Port of `listReservations(root, {activeOnly, now})` — the CURRENT
/// sharded-lease-backed reader `bee.mjs`'s `buildStatus` actually calls
/// today (distinct from [`read_reservations_projection`]'s legacy-file
/// reader above). `root` is expected already control-root-resolved by the
/// caller (msn-18b: `listPathLeaseRecords` re-roots via `controlRootFor`
/// internally in mjs; this port takes the resolved root directly, matching
/// this cell's established caller-supplies-control-root pattern).
pub fn list_reservations(root: &Path, active_only: bool, now_ms: i64) -> Vec<LeaseReservationView> {
    list_leases(root)
        .into_iter()
        .filter(is_path_lease)
        .filter(|r| !active_only || !is_lease_record_expired(r, now_ms))
        .map(|r| lease_to_reservation(&r))
        .collect()
}

/// Port of the reservation-expiry staleness computation INLINE in
/// `bee.mjs`'s `buildStatus` (bee.mjs:739-746) — deliberately NOT a
/// bugfix. The mjs source computes:
/// ```js
/// const allReservations = listReservations(root);
/// const active = listReservations(root, { activeOnly: true });
/// const expiredUnreleased = allReservations.filter(
///   (r) => r.released_at == null && !active.includes(r),
/// );
/// ```
/// Two facts make this NOT what its name suggests: (1) `released_at` is
/// ALWAYS `null` in the current lease-backed shape (see
/// [`LeaseReservationView`]'s doc comment), so the first half of the
/// filter is always true; (2) `allReservations` and `active` are two
/// SEPARATE `listReservations()` calls, each producing freshly-`.map()`-
/// allocated objects — so `active.includes(r)` compares object IDENTITY
/// (JS `Array.includes` uses SameValueZero) across objects that can never
/// be `===` to one another, meaning `!active.includes(r)` is
/// unconditionally `true`. The net, OBSERVABLE (and D1-frozen) mjs
/// behavior is: EVERY currently-stored reservation counts as "expired
/// unreleased" — active leases included, not just the genuinely-expired-
/// but-unswept ones the name suggests. This port reproduces that behavior
/// exactly rather than "fixing" it to genuine value-equality exclusion,
/// which would silently diverge from the frozen mjs oracle (D1).
pub fn expired_unreleased_reservations(root: &Path, now_ms: i64) -> Vec<LeaseReservationView> {
    list_reservations(root, false, now_ms)
}

// ─── lease renewal by session (rust-port-17) ─────────────────────────────
// Port of reservations.mjs's `renewHoldsBySession` (itself a thin per-record
// loop over lease-store.mjs's `renewLease`/`renewLeaseFile`) — the ONE
// write path bee-state-sync.mjs's heartbeat companion calls. `control_root`
// is caller-supplied, already resolved (this crate's established pattern:
// `list_reservations`/`list_leases` above already take a resolved `root`
// directly) — mjs re-derives it internally via its own private
// `controlRootFor`.

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// lease-store.mjs's `lockNameForFile(file)` — `lease:<sha256(file)>`, the
/// same D9 store-lock name a real node `renewLease`/`releaseLease` call on
/// this exact file takes, so a genuine cross-runtime holder is denied (or
/// denies) correctly.
fn lease_lock_name_for_file(file: &Path) -> String {
    format!("lease:{}", sha256_hex(&file.to_string_lossy()))
}

/// lease-store.mjs's `computeExpiresAt(ttl, nowMs)` — `ttl <= 0` is the
/// never-expires sentinel (`null`/`Value::Null`), matching
/// [`is_lease_record_expired`]'s own `None`-reads-as-"never expires"
/// convention.
fn compute_expires_at(ttl: i64, now_ms: i64) -> Value {
    if ttl > 0 {
        Value::String(iso8601_millis(now_ms + ttl * 1000))
    } else {
        Value::Null
    }
}

/// Port of `renewLeaseFile`'s read-modify-write body, scoped to this cell's
/// call site: `presentedEpoch` fencing is never used here (the hook never
/// presents one), so it is not ported. `Ok(true)` = renewed; `Ok(false)` =
/// the record was missing or unreadable/corrupt at read time — mirrors the
/// mjs source's `LEASE_MISSING`/`LEASE_CORRUPT` typed failures, both of
/// which `renewHoldsBySession` treats as "skip, never block the caller"
/// (only `LEASE_MISSING` explicitly; a corrupt record is a hook-scoped
/// fail-open simplification — nothing this call path can safely renew
/// anyway). Runs under this file's own per-resource lock
/// ([`lease_lock_name_for_file`]), never a store-wide lock.
fn renew_lease_file(control_root: &Path, file: &Path, ttl: i64, now_ms: i64) -> Result<bool, WithLockError> {
    with_store_lock(control_root, &lease_lock_name_for_file(file), LockOptions::try_once(), || -> bool {
        let Ok(text) = fs::read_to_string(file) else {
            return false;
        };
        let Ok(mut current) = serde_json::from_str::<Value>(&text) else {
            return false;
        };
        if let Value::Object(ref mut m) = current {
            m.insert("expires_at".to_string(), compute_expires_at(ttl, now_ms));
        }
        write_json_atomic(file, &current).is_ok()
    })
}

/// Port of `renewHoldsBySession(root, sessionId, { now, lockOptions })`:
/// resets each of this session's still-live PATH leases' anchor clock
/// (`expires_at`) by the lease's OWN original ttl window (never
/// lease-store's `DEFAULT_TTL_SECONDS`) — "reset the anchor time, never
/// touch ttl_seconds", the exact contract the mjs source documents. Returns
/// the renewed count (informational; the hook itself never inspects this).
/// An empty/blank `session_id` is a no-op, matching the mjs early return.
pub fn renew_holds_by_session(control_root: &Path, session_id: &str, now_ms: i64) -> usize {
    let session = session_id.trim();
    if session.is_empty() {
        return 0;
    }
    let dir = leases_root(control_root).join("paths");
    let Ok(entries) = fs::read_dir(&dir) else {
        return 0;
    };
    let mut renewed = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_str::<LeaseRecord>(&text) else {
            continue;
        };
        if record.session_id.as_deref() != Some(session) {
            continue;
        }
        let ttl = lease_ttl_seconds(&record);
        if matches!(renew_lease_file(control_root, &path, ttl, now_ms), Ok(true)) {
            renewed += 1;
        }
    }
    renewed
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
//
// (rust-port-17's own lease-renewal write above is proved against the real
// mjs oracle by crates/queen-bee/tests/heavyhooks_conformance.rs, this
// cell's mandated single integration target — side-effect parity on
// seeded fixtures, not here.)
