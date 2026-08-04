// lease_store — native port of packages/bee/lib/lease-store.mjs
// (multisession-native-11/12/13: per-resource epoch lease records).
//
// LIBRARY module (no `try_native`, no probe line in verbs/mod.rs) — the same
// shape workflow_store.rs has. It exists because three contracts the Node
// runtime owns had NO Rust implementation anywhere in this tree, so deleting
// `bee.mjs` would have LOST behavior rather than lost coverage:
//
//   * renewLease / renewLeasesBySession and the typed LEASE_MISSING refusal
//   * LEASE_FENCE_STALE on BOTH renew and release, including the "the lease
//     file is never removed on a fenced refusal" property
//   * multi-resource batch acquire: hash-sorted deterministic order, full
//     rollback of a partial acquire, and LEASE_INVALID_REQUEST as a typed
//     batch validation error
//
// Provenance, function by function (lease-store.mjs → this file):
//   runtimeDir/leasesRoot/leaseCellsDir/leasePathsDir → runtime_dir /
//     leases_root / lease_cells_dir / lease_paths_dir
//   requireCellId                → require_cell_id
//   canonicalizePath             → canonicalize_path
//   hashString/lockNameForFile   → sha256_hex / lock_name_for_file
//   resolveResourceFile          → resolve_resource_file
//   readLeaseSafe                → read_lease_safe
//   isLeaseExpired               → lease_expired
//   computeExpiresAt             → compute_expires_at
//   sameLeaseRecord              → same_lease_record
//   tryCreateLeaseFile           → try_create_lease_file
//   rollbackOne                  → rollback_one
//   normalizeAcquireRequest      → normalize_acquire_request
//   acquireLeases                → acquire_leases
//   releaseLease                 → release_lease
//   renewLeaseFile / renewLease  → renew_lease_file / renew_lease
//   renewLeasesBySession         → renew_leases_by_session
//   sweepExpiredLeases           → sweep_expired_leases
//   listAllLeaseFiles/listLeases → list_all_lease_files / list_leases
//
// SECOND-PORT NOTE (campaign rule "keep one behavior, not two"): two narrowed
// renew paths already exist in this tree — `renew_lease_path` in
// src/hooks/state_sync.rs and its twin in src/hooks/prompt_context.rs. Both
// are module-private and both files are outside this cell's touchable set, so
// they are NOT consolidated here; instead they are re-derived from the same
// .mjs source and `agrees_with_the_hook_ports_on_the_shared_renew_fixture`
// below pins this module against the hook's own on-disk outcome, so a future
// divergence fails a test instead of drifting silently.
//
// A THIRD partial port lives in verbs/reservations.rs: the single-resource
// path-lease arm that `reservations reserve|release|list|sweep` drives
// (`reserve_locked`'s inline O_EXCL create). That arm is left exactly as it
// is — it is byte-diffed against Node through four live verbs and rewiring it
// through this module would be a behavior risk with no behavior gain. This
// module is the home of the MULTI-resource batch, the renew half, and the
// fencing half, none of which reservations.rs has or needs.
//
// Locking: identical lock-name strings to Node, so both runtimes serialize
// against each other mid-campaign — `lease:<sha256(lease-file-path)>`
// (lease-store.mjs lockNameForFile), never a store-wide "leases" lock. The
// file path fed to the hash is the platform-native spelling `path.join`
// produces, matching hooks/state_sync.rs's existing lock-name derivation.
//
// Refusal posture: every refusal this module produces is a typed
// LeaseStoreError with DETERMINISTIC bytes and is reproduced natively —
// including the ones reached after a lock attempt (LEASE_MISSING,
// LEASE_FENCE_STALE), which campaign rule 2 required be native so a
// delegation could not double the lock-contention telemetry. LEASE_CORRUPT
// interpolated the V8/libuv error message in Node; this port interpolates the
// Rust error instead, exactly as hooks/state_sync.rs::renew_lease_path does —
// the two ports agree with each other, and at cutover there is no third
// answer to agree with.

// NOT YET WIRED TO A VERB — deliberately, and the reason is worth stating
// rather than silencing. `bee` has no CLI surface that acquires a multi-
// resource batch, renews a lease, or presents a fencing epoch: Node's own
// callers of these functions are `state start-feature` (still delegated) and
// the msn-16 write-guard shim (not yet built), so wiring one today would mean
// inventing a verb, not porting one. What this module removes is the
// DELETION blocker — the behavior exists natively now, tested against the
// live Node oracle, so `bee.mjs` can go without losing it.
#![allow(dead_code)]

use crate::fsutil::{ensure_dir, write_json_atomic};
use crate::jsjson;
use crate::lock::{self, LockBusy};
use crate::verbs::reservations::{iso_from_ms, jget, js_trim, truthy};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_TTL_SECONDS: f64 = 3600.0;
const RESOURCE_TYPES: [&str; 2] = ["cell", "path"];
const LEASE_KINDS: [&str; 2] = ["intent", "lease"];
const DEFAULT_LEASE_KIND: &str = "lease";

// ─── typed refusals ────────────────────────────────────────────────────────

/// lease-store.mjs LeaseStoreError: `{name:'LeaseStoreError', type:'refused',
/// code, message}` plus the optional `.resource` / `.holder` details the
/// LEASE_HELD and fencing throws attach.
#[derive(Debug, Clone)]
pub(crate) struct LeaseRefusal {
    pub code: &'static str,
    pub message: String,
    pub resource: Option<String>,
    pub holder: Option<Value>,
}

impl LeaseRefusal {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message, resource: None, holder: None }
    }
    fn with_details(mut self, resource: String, holder: Option<Value>) -> Self {
        self.resource = Some(resource);
        self.holder = holder;
        self
    }
}

#[derive(Debug)]
pub(crate) enum LeaseErr {
    /// A typed LeaseStoreError — deterministic bytes, reproduced natively.
    Refused(LeaseRefusal),
    /// lock.mjs LockBusyError — deterministic bytes (lock.rs reproduces them).
    LockBusy(LockBusy),
    /// An fs error other than the modeled ones, or a JS relational compare
    /// against a non-number epoch — the cases where Node surfaced raw
    /// V8/libuv bytes.
    ///
    /// CUTOVER: the instruction here used to be "a caller reached from a verb
    /// must DELEGATE on this". There is no runtime to delegate to. This module
    /// has no verb caller yet (see the not-yet-wired note above), so the first
    /// one that lands must map this to a native refusal — never to a bail.
    Exotic,
}

impl LeaseErr {
    fn refuse(code: &'static str, message: String) -> Self {
        LeaseErr::Refused(LeaseRefusal::new(code, message))
    }
    /// The refusal code, for callers that branch on it (renewLeasesBySession's
    /// LEASE_MISSING skip).
    pub(crate) fn code(&self) -> Option<&'static str> {
        match self {
            LeaseErr::Refused(r) => Some(r.code),
            _ => None,
        }
    }
    /// `${error.name}: ${error.message}` — the shape a bee.mjs catch renders.
    pub(crate) fn text(&self) -> String {
        match self {
            LeaseErr::Refused(r) => format!("LeaseStoreError: {}", r.message),
            LeaseErr::LockBusy(b) => format!("LockBusyError: {}", b.message()),
            LeaseErr::Exotic => "<exotic>".to_string(),
        }
    }
}

pub(crate) type LR<T> = Result<T, LeaseErr>;

// ─── paths ─────────────────────────────────────────────────────────────────

pub(crate) fn runtime_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime")
}

/// leasesRoot(root) — the ONE place this module builds the leases directory,
/// mirroring the .mjs's own one-line-re-root discipline.
pub(crate) fn leases_root(root: &Path) -> PathBuf {
    runtime_dir(root).join("leases")
}

pub(crate) fn lease_cells_dir(root: &Path) -> PathBuf {
    leases_root(root).join("cells")
}

pub(crate) fn lease_paths_dir(root: &Path) -> PathBuf {
    leases_root(root).join("paths")
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

/// lease-store.mjs lockNameForFile — `lease:<sha256(file)>`, hashed over the
/// platform-native path spelling `path.join` produces.
fn lock_name_for_file(file: &Path) -> String {
    format!("lease:{}", sha256_hex(&file.display().to_string()))
}

/// lease-store.mjs requireCellId.
fn require_cell_id(value: Option<&Value>) -> LR<String> {
    let s = match value {
        Some(Value::String(s)) if !js_trim(s).is_empty() => s,
        _ => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                "lease request: cell id is required.".to_string(),
            ))
        }
    };
    let id = js_trim(s).to_string();
    if id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(LeaseErr::refuse(
            "LEASE_INVALID_REQUEST",
            format!(
                "lease request: cell id \"{id}\" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/leases/cells/."
            ),
        ));
    }
    Ok(id)
}

/// lease-store.mjs canonicalizePath — backslash-normalize, strip ONE anchored
/// leading `./+`, strip trailing slashes. (Deliberate duplicate of
/// reservations.mjs's private normalizePath, exactly as the .mjs says.)
pub(crate) fn canonicalize_path(value: &str) -> String {
    let s = value.replace('\\', "/");
    let s = match s.strip_prefix('.') {
        Some(rest) if rest.starts_with('/') => rest.trim_start_matches('/').to_string(),
        _ => s,
    };
    s.trim_end_matches('/').to_string()
}

/// `String(value || '')` — the coercion canonicalizePath applies first.
fn string_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(v) if truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    }
}

/// `JSON.stringify(x)` where an ABSENT property renders as the literal
/// `undefined` in a template string (JSON.stringify(undefined) is undefined).
fn json_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::stringify(v),
    }
}

pub(crate) struct Resolved {
    pub rtype: &'static str,
    pub id: String,
    pub resource_key: String,
    pub hash: String,
    pub file: PathBuf,
}

/// lease-store.mjs resolveResourceFile — the SINGLE place a {type, id} pair
/// becomes a resource key + lock-name hash + on-disk file path.
pub(crate) fn resolve_resource_file(root: &Path, request: &Value) -> LR<Resolved> {
    let rtype = jget(request, "type");
    let type_name = match rtype {
        Some(Value::String(s)) if RESOURCE_TYPES.contains(&s.as_str()) => {
            if s == "cell" { "cell" } else { "path" }
        }
        _ => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                format!(
                    "lease request: type must be one of {} (got {}).",
                    RESOURCE_TYPES.join("/"),
                    json_or_undefined(rtype)
                ),
            ))
        }
    };
    let id = jget(request, "id");
    if type_name == "cell" {
        let cell_id = require_cell_id(id)?;
        let resource_key = format!("cell:{cell_id}");
        let hash = sha256_hex(&resource_key);
        let file = lease_cells_dir(root).join(format!("{cell_id}.json"));
        return Ok(Resolved { rtype: "cell", id: cell_id, resource_key, hash, file });
    }
    match id {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {}
        _ => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                "lease request: path id is required.".to_string(),
            ))
        }
    }
    let canonical = canonicalize_path(&string_or_empty(id));
    let resource_key = format!("path:{canonical}");
    let hash = sha256_hex(&resource_key);
    let file = lease_paths_dir(root).join(format!("{hash}.json"));
    Ok(Resolved { rtype: "path", id: canonical, resource_key, hash, file })
}

// ─── record helpers ────────────────────────────────────────────────────────

/// lease-store.mjs readLeaseSafe — gone, unreadable or mid-write elsewhere is
/// "no info", never a throw.
fn read_lease_safe(file: &Path) -> Option<Value> {
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    serde_json::from_str::<Value>(&text).ok()
}

/// lease-store.mjs isLeaseExpired — no record, or `expires_at == null`
/// ("never expires"), or an unparseable stamp, all read as NOT expired.
fn lease_expired(record: Option<&Value>, now_ms: f64) -> bool {
    let Some(record) = record else { return false };
    let expires = match jget(record, "expires_at") {
        None | Some(Value::Null) => return false, // `== null` covers undefined too
        Some(v) => v,
    };
    let Value::String(s) = expires else {
        return false; // Date.parse(non-string) -> NaN via String() in every bee shape
    };
    match crate::verbs::reservations::js_date_parse(s) {
        Ok(Some(ms)) => ms <= now_ms,
        _ => false,
    }
}

/// lease-store.mjs computeExpiresAt — non-positive/non-finite ttl means
/// "never expires" (`expires_at: null`).
fn compute_expires_at(ttl: f64, now_ms: f64) -> Value {
    if ttl.is_finite() && ttl > 0.0 {
        match iso_from_ms(now_ms + ttl * 1000.0) {
            Ok(s) => Value::String(s),
            Err(_) => Value::Null,
        }
    } else {
        Value::Null
    }
}

/// Native `Value` equality, except two Numbers compare by numeric value
/// (`as_f64`) rather than by serde_json's internal PosInt/NegInt/Float
/// variant — `epoch` here can arrive as an integer literal parsed from a
/// stored file or as a float produced by in-memory arithmetic, and those
/// are the same JSON number even though serde_json's derived `PartialEq`
/// treats them as different representations.
fn value_eq(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// lease-store.mjs sameLeaseRecord — exact-match acquisition identity, so a
/// rollback can never delete a file that changed underneath it.
fn same_lease_record(a: Option<&Value>, b: &Value) -> bool {
    let Some(a) = a else { return false };
    if !truthy(a) {
        return false;
    }
    for key in ["resource", "session_id", "workflow_id", "epoch", "acquired_at"] {
        match (jget(a, key), jget(b, key)) {
            (None, None) => {}            // undefined === undefined
            (Some(x), Some(y)) if value_eq(x, y) => {}
            _ => return false,
        }
    }
    true
}

/// lease-store.mjs tryCreateLeaseFile — O_EXCL create, the whole concurrency
/// primitive. EEXIST -> Ok(false); any other fs error is V8-worded -> Exotic.
fn try_create_lease_file(file: &Path, record: &Value) -> LR<bool> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir).map_err(|_| LeaseErr::Exotic)?;
    }
    let body = format!("{}\n", jsjson::stringify_pretty(record));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(file) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(body.as_bytes()).map_err(|_| LeaseErr::Exotic)?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(_) => Err(LeaseErr::Exotic),
    }
}

/// lease-store.mjs rollbackOne — delete only if the file is still THIS call's
/// own acquisition; a mismatch is left exactly as it is now.
fn rollback_one(file: &Path, record: &Value) {
    let current = read_lease_safe(file);
    if same_lease_record(current.as_ref(), record) {
        let _ = std::fs::remove_file(file); // best-effort, never over the original refusal
    }
}

// ─── acquireLeases (the multi-resource batch) ──────────────────────────────

struct Normalized {
    resolved: Resolved,
    mode: String,
    workflow_id: Value,
    session_id: Value,
    workspace_id: Value,
    epoch: Value,
    ttl: f64,
    kind: String,
}

/// lease-store.mjs normalizeAcquireRequest — validation order is load-bearing
/// (resolveResourceFile, then mode, then kind, then the three id fields in
/// declaration order, then epoch), because the FIRST failure is the message
/// the caller sees.
fn normalize_acquire_request(root: &Path, req: &Value) -> LR<Normalized> {
    // `!req || typeof req !== 'object'` — null/false/0/"" fail; an ARRAY
    // passes typeof and simply has none of the properties below.
    let is_object = matches!(req, Value::Object(_) | Value::Array(_));
    if !truthy(req) || !is_object {
        return Err(LeaseErr::refuse(
            "LEASE_INVALID_REQUEST",
            "acquireLeases: each request must be an object.".to_string(),
        ));
    }
    let resolved = resolve_resource_file(root, req)?;
    let mode = match jget(req, "mode") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                format!("lease request \"{}\": mode is required.", resolved.resource_key),
            ))
        }
    };
    // `kind = DEFAULT_LEASE_KIND` fires only on undefined (an absent key).
    let kind_value = jget(req, "kind");
    let kind = match kind_value {
        None => DEFAULT_LEASE_KIND.to_string(),
        Some(Value::String(s)) if LEASE_KINDS.contains(&s.as_str()) => s.clone(),
        Some(v) => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                format!(
                    "lease request \"{}\": kind must be one of {} (got {}).",
                    resolved.resource_key,
                    LEASE_KINDS.join("/"),
                    jsjson::stringify(v)
                ),
            ))
        }
    };
    let mut fields: Vec<(&str, Value)> = Vec::new();
    for key in ["workflow_id", "session_id", "workspace_id"] {
        match jget(req, key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                fields.push((key, Value::String(s.clone())))
            }
            _ => {
                return Err(LeaseErr::refuse(
                    "LEASE_INVALID_REQUEST",
                    format!("lease request \"{}\": {key} is required.", resolved.resource_key),
                ))
            }
        }
    }
    let epoch = match jget(req, "epoch") {
        Some(Value::Number(n)) if n.as_f64().map(f64::is_finite).unwrap_or(false) => {
            Value::Number(n.clone())
        }
        _ => {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                format!(
                    "lease request \"{}\": epoch must be a finite number.",
                    resolved.resource_key
                ),
            ))
        }
    };
    // `ttl = DEFAULT` on undefined, then `Number.isFinite(ttl) ? ttl : DEFAULT`
    // — so `null`, a string, or NaN all land back on the default.
    let ttl = match jget(req, "ttl") {
        Some(Value::Number(n)) => n.as_f64().filter(|v| v.is_finite()).unwrap_or(DEFAULT_TTL_SECONDS),
        _ => DEFAULT_TTL_SECONDS,
    };
    Ok(Normalized {
        resolved,
        mode,
        workflow_id: fields[0].1.clone(),
        session_id: fields[1].1.clone(),
        workspace_id: fields[2].1.clone(),
        epoch,
        ttl,
        kind,
    })
}

/// acquireLeases(root, requests, {now}) — acquire one or more leases as a
/// single all-or-nothing batch.
///
/// Deadlock-free regardless of caller order: requests are sorted by the
/// sha256 of their resource key BEFORE any file is created, so two callers
/// racing the same two resources in opposite request order still attempt
/// creation in the same globally-determined order.
///
/// On the first collision every lease already created by THIS call is rolled
/// back before the typed LEASE_HELD refusal — a partial acquire is never left
/// standing.
pub(crate) fn acquire_leases(root: &Path, requests: &[Value], now_ms: f64) -> LR<Vec<Value>> {
    if requests.is_empty() {
        return Err(LeaseErr::refuse(
            "LEASE_INVALID_REQUEST",
            "acquireLeases: requests must be a non-empty array.".to_string(),
        ));
    }
    let mut normalized: Vec<Normalized> = Vec::with_capacity(requests.len());
    for req in requests {
        normalized.push(normalize_acquire_request(root, req)?);
    }

    let mut seen: Vec<PathBuf> = Vec::new();
    for item in &normalized {
        if seen.contains(&item.resolved.file) {
            return Err(LeaseErr::refuse(
                "LEASE_INVALID_REQUEST",
                format!(
                    "acquireLeases: resource \"{}\" was requested more than once in the same call.",
                    item.resolved.resource_key
                ),
            ));
        }
        seen.push(item.resolved.file.clone());
    }

    // Array.prototype.sort is STABLE in V8 and the comparator only orders by
    // hash, so equal hashes (impossible after the duplicate check above) would
    // keep request order. sort_by is stable in Rust too.
    normalized.sort_by(|a, b| a.resolved.hash.cmp(&b.resolved.hash));

    let acquired_at = iso_from_ms(now_ms).map_err(|_| LeaseErr::Exotic)?;
    let mut acquired: Vec<(PathBuf, Value)> = Vec::new();
    for item in &normalized {
        let mut record = Map::new();
        record.insert("resource".into(), Value::String(item.resolved.resource_key.clone()));
        record.insert("mode".into(), Value::String(item.mode.clone()));
        record.insert("workflow_id".into(), item.workflow_id.clone());
        record.insert("session_id".into(), item.session_id.clone());
        record.insert("workspace_id".into(), item.workspace_id.clone());
        record.insert("epoch".into(), item.epoch.clone());
        record.insert("acquired_at".into(), Value::String(acquired_at.clone()));
        record.insert("expires_at".into(), compute_expires_at(item.ttl, now_ms));
        record.insert("kind".into(), Value::String(item.kind.clone()));
        let record = Value::Object(record);
        if try_create_lease_file(&item.resolved.file, &record)? {
            acquired.push((item.resolved.file.clone(), record));
            continue;
        }
        let holder = read_lease_safe(&item.resolved.file);
        for (file, rec) in &acquired {
            rollback_one(file, rec);
        }
        let by_session = match holder.as_ref().and_then(|h| jget(h, "session_id")) {
            Some(v) if truthy(v) => format!(" by session \"{}\"", jsjson::js_to_string(v)),
            _ => String::new(),
        };
        return Err(LeaseErr::Refused(
            LeaseRefusal::new(
                "LEASE_HELD",
                format!(
                    "acquireLeases: resource \"{}\" is already leased{by_session}.",
                    item.resolved.resource_key
                ),
            )
            .with_details(item.resolved.resource_key.clone(), holder),
        ));
    }
    Ok(acquired.into_iter().map(|(_, r)| r).collect())
}

// ─── releaseLease (legacy + fenced) ────────────────────────────────────────

/// The `{ok, released}` answer both release arms return.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Released(pub bool);

/// releaseLease(root, {type,id}, {presentedEpoch}) — idempotent delete of one
/// lease file.
///
/// `presented_epoch: None` is the LEGACY arm: byte-unchanged, a single
/// lock-free remove. `Some(v)` takes the SAME per-resource
/// `lease:<file-hash>` lock renewLease uses (a genuine read-then-maybe-delete
/// is not safely lock-free) and refuses typed LEASE_FENCE_STALE when the
/// presentation is behind the stored epoch — the file is left UNTOUCHED on
/// that refusal, never deleted on a stale presentation.
pub(crate) fn release_lease(
    root: &Path,
    request: &Value,
    presented_epoch: Option<&Value>,
    lock_attempts: u32,
) -> LR<Released> {
    let resolved = resolve_resource_file(root, request)?;
    let Some(presented) = presented_epoch else {
        return match std::fs::remove_file(&resolved.file) {
            Ok(()) => Ok(Released(true)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Released(false)),
            Err(_) => Err(LeaseErr::Exotic),
        };
    };
    let mut guard = lock::acquire_store_lock(root, &lock_name_for_file(&resolved.file), lock_attempts)
        .map_err(LeaseErr::LockBusy)?;
    let outcome = (|| -> LR<Released> {
        let Some(current) = read_lease_safe(&resolved.file).filter(truthy) else {
            return Ok(Released(false)); // nothing on disk — nothing to fence against
        };
        fence_check("releaseLease", presented, &current)?;
        match std::fs::remove_file(&resolved.file) {
            Ok(()) => Ok(Released(true)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Released(false)),
            Err(_) => Err(LeaseErr::Exotic),
        }
    })();
    guard.release();
    outcome
}

/// The shared `!Number.isFinite(presentedEpoch) || presentedEpoch <
/// current.epoch` guard behind BOTH fenced arms. `verb` is the message prefix
/// ("renewLease" / "releaseLease") — the only difference between the two
/// throws in the .mjs.
fn fence_check(verb: &str, presented: &Value, current: &Value) -> LR<()> {
    let stored = match jget(current, "epoch") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        // A record whose epoch is absent or non-numeric takes a JS relational
        // compare this port does not model (`5 < undefined` is false, `5 <
        // "3"` coerces) — delegate rather than guess a fencing verdict.
        _ => return Err(LeaseErr::Exotic),
    };
    let presented_num = match presented {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        _ => f64::NAN, // Number.isFinite(non-number) === false
    };
    if presented_num.is_finite() && presented_num >= stored {
        return Ok(());
    }
    let resource = jget(current, "resource").map(jsjson::js_to_string).unwrap_or_else(|| "undefined".into());
    Err(LeaseErr::Refused(
        LeaseRefusal::new(
            "LEASE_FENCE_STALE",
            format!(
                "{verb}: presented epoch {} is behind resource \"{resource}\"'s current epoch {} — a takeover already moved ownership forward.",
                jsjson::stringify(presented),
                jsjson::js_f64_to_string(stored)
            ),
        )
        .with_details(resource, Some(current.clone())),
    ))
}

// ─── renewLease ────────────────────────────────────────────────────────────

/// lease-store.mjs renewLeaseFile — read-modify-write ONE lease file under its
/// OWN per-file lock, never a store-wide one.
fn renew_lease_file(
    root: &Path,
    file: &Path,
    ttl: f64,
    now_ms: f64,
    presented_epoch: Option<&Value>,
    lock_attempts: u32,
) -> LR<Value> {
    let mut guard = lock::acquire_store_lock(root, &lock_name_for_file(file), lock_attempts)
        .map_err(LeaseErr::LockBusy)?;
    let outcome = (|| -> LR<Value> {
        let text = match std::fs::read(file) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(LeaseErr::refuse(
                    "LEASE_MISSING",
                    format!("renewLease: no lease record at \"{}\" — acquire first.", file.display()),
                ))
            }
            Err(e) => {
                return Err(LeaseErr::refuse(
                    "LEASE_CORRUPT",
                    format!(
                        "renewLease: could not read/parse lease record at \"{}\" ({e}).",
                        file.display()
                    ),
                ))
            }
        };
        let current: Value = serde_json::from_str(&text).map_err(|e| {
            LeaseErr::refuse(
                "LEASE_CORRUPT",
                format!(
                    "renewLease: could not read/parse lease record at \"{}\" ({e}).",
                    file.display()
                ),
            )
        })?;
        if let Some(presented) = presented_epoch {
            fence_check("renewLease", presented, &current)?;
        }
        // `{...current, expires_at}` — a primitive spreads to nothing.
        let mut next: Map<String, Value> = match current {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        next.insert("expires_at".into(), compute_expires_at(ttl, now_ms));
        let next = Value::Object(next);
        write_json_atomic(file, &next).map_err(|_| LeaseErr::Exotic)?;
        Ok(next)
    })();
    guard.release();
    outcome
}

/// renewLease(root, {type,id}, {ttl, now, presentedEpoch}) — extend one
/// lease's expiry. Throws typed LEASE_MISSING/LEASE_CORRUPT, never guesses at
/// a record it cannot read; refuses LEASE_FENCE_STALE on a stale
/// presentation, leaving the record on disk exactly as read.
pub(crate) fn renew_lease(
    root: &Path,
    request: &Value,
    ttl: f64,
    now_ms: f64,
    presented_epoch: Option<&Value>,
    lock_attempts: u32,
) -> LR<Value> {
    let resolved = resolve_resource_file(root, request)?;
    renew_lease_file(root, &resolved.file, ttl, now_ms, presented_epoch, lock_attempts)
}

/// renewLeasesBySession — renews every still-present lease owned by
/// `session_id`, one per-file lock at a time, never a store-wide lock. A lease
/// swept or released between the listing and its own renew attempt is treated
/// as already-gone (skipped), never resurrected.
pub(crate) fn renew_leases_by_session(
    root: &Path,
    session_id: &str,
    ttl: f64,
    now_ms: f64,
    lock_attempts: u32,
) -> LR<usize> {
    let session = js_trim(session_id);
    if session.is_empty() {
        return Ok(0);
    }
    let mut renewed = 0usize;
    for file in list_all_lease_files(root) {
        let Some(current) = read_lease_safe(&file) else { continue };
        match jget(&current, "session_id") {
            Some(Value::String(s)) if s == session => {}
            _ => continue,
        }
        match renew_lease_file(root, &file, ttl, now_ms, None, lock_attempts) {
            Ok(_) => renewed += 1,
            Err(e) if e.code() == Some("LEASE_MISSING") => continue, // swept concurrently
            Err(e) => return Err(e),
        }
    }
    Ok(renewed)
}

// ─── enumeration + sweep ───────────────────────────────────────────────────

/// lease-store.mjs listAllLeaseFiles — cells dir then paths dir, `*.json`
/// files only, in directory-read order.
fn list_all_lease_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [lease_cells_dir(root), lease_paths_dir(root)] {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_file && name.ends_with(".json") {
                files.push(dir.join(name));
            }
        }
    }
    files
}

/// sweepExpiredLeases — delete every lease file whose expires_at has passed.
/// Per-record, never a store-wide lock; a corrupt entry is skipped, never
/// guessed at; a delete racing a concurrent release is a harmless no-op.
pub(crate) fn sweep_expired_leases(root: &Path, now_ms: f64) -> usize {
    let mut swept = 0usize;
    for file in list_all_lease_files(root) {
        let record = read_lease_safe(&file);
        if record.is_none() || !lease_expired(record.as_ref(), now_ms) {
            continue;
        }
        if std::fs::remove_file(&file).is_ok() {
            swept += 1;
        }
    }
    swept
}

/// listLeases — fail-open enumeration: a corrupt entry is SKIPPED and
/// reported back. Returns `(leases, skipped)` in directory-read order.
pub(crate) fn list_leases(root: &Path) -> (Vec<Value>, Vec<Value>) {
    let mut leases = Vec::new();
    let mut skipped = Vec::new();
    for file in list_all_lease_files(root) {
        match read_lease_safe(&file) {
            Some(record) => leases.push(record),
            None => skipped.push(serde_json::json!({
                "file": file.display().to_string(),
                "reason": "unreadable or corrupt JSON",
            })),
        }
    }
    (leases, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::reservations::now_ms;
    use serde_json::json;

    fn fixture() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        // dunce-canonical root, so the lock-name hash is over the same
        // spelling a resolved root would produce.
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        tmp
    }

    fn req(rtype: &str, id: &str, session: &str, epoch: f64) -> Value {
        json!({
            "type": rtype, "id": id, "mode": "write",
            "workflow_id": "wf-1", "session_id": session, "workspace_id": "agent:a",
            "epoch": epoch,
        })
    }

    fn read(file: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
    }

    fn all_files(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = list_all_lease_files(root)
            .into_iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn refusal(e: LeaseErr) -> (&'static str, String) {
        match e {
            LeaseErr::Refused(r) => (r.code, r.message),
            other => panic!("expected a typed refusal, got {}", other.text()),
        }
    }

    // ══ contract 6: multi-resource batch acquire ═══════════════════════════

    /// Oracle (test_lease_store.mjs): "acquireLeases writes the full record
    /// shape for every requested resource, cells and paths alike".
    #[test]
    fn a_batch_acquires_every_resource_with_the_full_record_shape() {
        let tmp = fixture();
        let root = tmp.path();
        let records = acquire_leases(
            root,
            &[req("cell", "c-1", "sess-a", 3.0), req("path", "./src/a.ts/", "sess-a", 3.0)],
            1_700_000_000_000.0,
        )
        .unwrap();
        assert_eq!(records.len(), 2);
        // Field ORDER is part of the on-disk contract (C1: either runtime
        // reads/writes the same bytes).
        let keys: Vec<&String> = records[0].as_object().unwrap().keys().collect();
        assert_eq!(
            keys,
            vec![
                "resource", "mode", "workflow_id", "session_id", "workspace_id", "epoch",
                "acquired_at", "expires_at", "kind"
            ]
        );
        // canonicalizePath: backslash-normalized, "./" stripped, trailing "/" dropped.
        let by_key: Vec<String> =
            records.iter().map(|r| r["resource"].as_str().unwrap().to_string()).collect();

        let cell_file = lease_cells_dir(root).join("c-1.json");
        let on_disk = read(&cell_file);
        // The epoch is stored VERBATIM; JS renders 3 and 3.0 identically, so
        // the round-trip is compared numerically, not by JSON number kind.
        assert_eq!(on_disk["epoch"].as_f64(), Some(3.0));
        assert_eq!(on_disk["kind"], json!("lease"), "kind defaults to lease");
        assert_eq!(on_disk["acquired_at"], json!("2023-11-14T22:13:20.000Z"));
        assert_eq!(on_disk["expires_at"], json!("2023-11-14T23:13:20.000Z"), "default ttl 3600");
        // ORACLE-PINNED BYTES: captured from a live `node` run of
        // lease-store.mjs acquireLeases over this exact fixture
        // (`JSON.stringify(record, null, 2) + "\n"`), not from a reading of
        // the source.
        assert_eq!(
            std::fs::read_to_string(&cell_file).unwrap(),
            "{\n  \"resource\": \"cell:c-1\",\n  \"mode\": \"write\",\n  \"workflow_id\": \"wf-1\",\n  \"session_id\": \"sess-a\",\n  \"workspace_id\": \"agent:a\",\n  \"epoch\": 3,\n  \"acquired_at\": \"2023-11-14T22:13:20.000Z\",\n  \"expires_at\": \"2023-11-14T23:13:20.000Z\",\n  \"kind\": \"lease\"\n}\n"
        );
        // …and the path lease's file NAME, likewise oracle-pinned: a path
        // lease is keyed by the sha256 of its resource key.
        let path_file = lease_paths_dir(root)
            .join(format!("{}.json", sha256_hex("path:src/a.ts")));
        assert_eq!(
            path_file.file_name().unwrap().to_string_lossy(),
            "443405ed7318275198a27206d0ed174e8093c5c7bbb407fbaaf437222ebeb828.json"
        );
        assert!(path_file.exists());
        // The batch RETURNS records in the post-sort (hash) order, not the
        // caller's — the oracle answers ["path:src/a.ts", "cell:c-1"] here.
        assert_eq!(by_key, vec!["path:src/a.ts".to_string(), "cell:c-1".to_string()]);
    }

    /// Oracle: "acquireLeases is deadlock-free: requests are attempted in
    /// hash-sorted order regardless of the caller's own order".
    ///
    /// Node observes this with an `_orderSeam` test hook. This port proves the
    /// same property WITHOUT a production seam: the batch is run twice from
    /// opposite request orders against a pre-held middle resource, and the
    /// rollback set is identical both times — which is only true if both runs
    /// processed the resources in the same globally-determined order.
    #[test]
    fn a_batch_attempts_resources_in_hash_sorted_order_whatever_the_caller_asked_for() {
        // Establish the hash order of three cell resources.
        let mut ordered = vec!["c-a", "c-b", "c-c"];
        ordered.sort_by_key(|id| sha256_hex(&format!("cell:{id}")));
        let (first, middle) = (ordered[0], ordered[1]);

        for reversed in [false, true] {
            let tmp = fixture();
            let root = tmp.path();
            // Pre-hold the MIDDLE resource in hash order.
            acquire_leases(root, &[req("cell", middle, "holder", 1.0)], now_ms()).unwrap();

            let mut batch: Vec<Value> =
                ordered.iter().map(|id| req("cell", id, "sess-a", 1.0)).collect();
            if reversed {
                batch.reverse();
            }
            let (code, message) = refusal(acquire_leases(root, &batch, now_ms()).unwrap_err());
            assert_eq!(code, "LEASE_HELD");
            assert!(
                message.contains(&format!("resource \"cell:{middle}\"")),
                "the MIDDLE resource in hash order must be the collision, not the caller's second request: {message}"
            );
            assert!(message.contains("by session \"holder\""), "{message}");
            // Only the holder's own file survives: the one this call created
            // BEFORE the collision (hash-order `first`) was rolled back.
            assert_eq!(
                all_files(root),
                vec![format!("{middle}.json")],
                "reversed={reversed}: a partial acquire is never left standing"
            );
            assert!(!lease_cells_dir(root).join(format!("{first}.json")).exists());
        }
    }

    /// Oracle: "partial acquire rolls back fully — zero residue", plus the
    /// rollback's own identity guard: a file that CHANGED underneath this call
    /// since it created it is never deleted.
    #[test]
    fn rollback_deletes_only_this_calls_own_acquisitions() {
        let tmp = fixture();
        let root = tmp.path();
        let mine = json!({"resource":"cell:x","session_id":"s","workflow_id":"w","epoch":1,"acquired_at":"2020-01-01T00:00:00.000Z"});
        let file = lease_cells_dir(root).join("x.json");
        ensure_dir(file.parent().unwrap()).unwrap();

        // Identical record -> removed.
        std::fs::write(&file, jsjson::stringify_pretty(&mine)).unwrap();
        rollback_one(&file, &mine);
        assert!(!file.exists());

        // Released and re-acquired by someone else in the window: acquired_at
        // alone differs, and the file must survive untouched.
        let theirs = json!({"resource":"cell:x","session_id":"s","workflow_id":"w","epoch":1,"acquired_at":"2020-01-01T00:00:00.001Z"});
        std::fs::write(&file, jsjson::stringify_pretty(&theirs)).unwrap();
        rollback_one(&file, &mine);
        assert!(file.exists(), "a changed file is never touched by another call's rollback");
        assert_eq!(read(&file), theirs);

        // A corrupt/unreadable file is likewise never deleted.
        std::fs::write(&file, "{not json").unwrap();
        rollback_one(&file, &mine);
        assert!(file.exists());
    }

    /// Oracle: "LEASE_INVALID_REQUEST — missing fields, bad type, duplicate
    /// resource … refuses before any file is created".
    #[test]
    fn every_malformed_batch_request_refuses_before_any_file_exists() {
        let tmp = fixture();
        let root = tmp.path();
        let good = req("cell", "c-1", "sess-a", 1.0);
        let strip = |key: &str| {
            let mut m = good.as_object().unwrap().clone();
            m.shift_remove(key);
            Value::Object(m)
        };
        let with = |key: &str, v: Value| {
            let mut m = good.as_object().unwrap().clone();
            m.insert(key.into(), v);
            Value::Object(m)
        };

        let cases: Vec<(Vec<Value>, &str)> = vec![
            (vec![], "acquireLeases: requests must be a non-empty array."),
            (vec![Value::Null], "acquireLeases: each request must be an object."),
            (vec![json!("nope")], "acquireLeases: each request must be an object."),
            (vec![strip("type")], "lease request: type must be one of cell/path (got undefined)."),
            (vec![with("type", json!("workspace"))], "lease request: type must be one of cell/path (got \"workspace\")."),
            (vec![with("id", json!("  "))], "lease request: cell id is required."),
            (
                vec![with("id", json!("a/b"))],
                "lease request: cell id \"a/b\" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/leases/cells/.",
            ),
            (vec![json!({"type":"path"})], "lease request: path id is required."),
            (vec![strip("mode")], "lease request \"cell:c-1\": mode is required."),
            (
                vec![with("kind", json!("exclusive"))],
                "lease request \"cell:c-1\": kind must be one of intent/lease (got \"exclusive\").",
            ),
            (vec![strip("workflow_id")], "lease request \"cell:c-1\": workflow_id is required."),
            (vec![strip("session_id")], "lease request \"cell:c-1\": session_id is required."),
            (vec![strip("workspace_id")], "lease request \"cell:c-1\": workspace_id is required."),
            (vec![strip("epoch")], "lease request \"cell:c-1\": epoch must be a finite number."),
            (vec![with("epoch", json!("1"))], "lease request \"cell:c-1\": epoch must be a finite number."),
            (
                vec![good.clone(), req("cell", "c-1", "sess-b", 2.0)],
                "acquireLeases: resource \"cell:c-1\" was requested more than once in the same call.",
            ),
        ];
        for (batch, expected) in cases {
            let (code, message) = refusal(acquire_leases(root, &batch, now_ms()).unwrap_err());
            assert_eq!(code, "LEASE_INVALID_REQUEST", "{expected}");
            assert_eq!(message, expected);
            assert!(all_files(root).is_empty(), "a refused batch created a file: {expected}");
        }

        // Control: the same batch with every field valid succeeds, and an
        // explicit `intent` kind is stamped verbatim.
        let ok = acquire_leases(root, &[with("kind", json!("intent"))], now_ms()).unwrap();
        assert_eq!(ok[0]["kind"], json!("intent"));
        assert_eq!(all_files(root), vec!["c-1.json".to_string()]);
    }

    /// A non-positive or non-finite ttl means "never expires", matching the
    /// acquire/renew TTL semantics reservations.mjs's isExpired relies on.
    #[test]
    fn a_non_positive_ttl_stores_expires_at_null_and_never_sweeps() {
        let tmp = fixture();
        let root = tmp.path();
        let mut zero = req("cell", "c-0", "s", 1.0);
        zero["ttl"] = json!(0);
        let mut negative = req("cell", "c-neg", "s", 1.0);
        negative["ttl"] = json!(-5);
        let mut bogus = req("cell", "c-bogus", "s", 1.0);
        bogus["ttl"] = json!(null); // falls back to the 3600 default
        let mut short = req("cell", "c-short", "s", 1.0);
        short["ttl"] = json!(1);

        let now = 1_700_000_000_000.0;
        acquire_leases(root, &[zero, negative, bogus, short], now).unwrap();
        assert_eq!(read(&lease_cells_dir(root).join("c-0.json"))["expires_at"], Value::Null);
        assert_eq!(read(&lease_cells_dir(root).join("c-neg.json"))["expires_at"], Value::Null);
        assert_eq!(
            read(&lease_cells_dir(root).join("c-bogus.json"))["expires_at"],
            json!("2023-11-14T23:13:20.000Z")
        );

        assert_eq!(sweep_expired_leases(root, now + 2000.0), 1, "only the 1s lease is expired");
        assert_eq!(
            all_files(root),
            vec!["c-0.json".to_string(), "c-bogus.json".to_string(), "c-neg.json".to_string()]
        );
        // A corrupt entry is skipped by the sweep AND reported by listLeases.
        std::fs::write(lease_cells_dir(root).join("c-0.json"), "{oops").unwrap();
        assert_eq!(sweep_expired_leases(root, now + 1e12), 1, "corrupt is never guessed at");
        let (leases, skipped) = list_leases(root);
        assert_eq!(leases.len(), 1, "only the never-expiring c-neg survives as readable");
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0]["reason"], json!("unreadable or corrupt JSON"));
    }

    // ══ contract 2: renewLease + LEASE_MISSING ═════════════════════════════

    /// Oracle: "renewLease extends expires_at and leaves every other field —
    /// epoch included — exactly as it was".
    #[test]
    fn renew_moves_only_expires_at_and_never_the_epoch() {
        let tmp = fixture();
        let root = tmp.path();
        let t0 = 1_700_000_000_000.0;
        acquire_leases(root, &[req("cell", "c-1", "sess-a", 7.0)], t0).unwrap();
        let before = read(&lease_cells_dir(root).join("c-1.json"));

        let next = renew_lease(root, &json!({"type":"cell","id":"c-1"}), 60.0, t0 + 5000.0, None, 1)
            .unwrap();
        assert_eq!(next["expires_at"], json!("2023-11-14T22:14:25.000Z"));
        assert_eq!(next["epoch"].as_f64(), Some(7.0), "renewal never bumps the fence");
        assert_eq!(next["acquired_at"], before["acquired_at"], "the acquisition stamp is immutable");
        assert_eq!(read(&lease_cells_dir(root).join("c-1.json")), next, "written atomically");

        // ttl <= 0 resets the clock to "never expires", same as a fresh acquire.
        let forever =
            renew_lease(root, &json!({"type":"cell","id":"c-1"}), 0.0, t0, None, 1).unwrap();
        assert_eq!(forever["expires_at"], Value::Null);
    }

    /// Oracle: "renewLease on a resource with no record refuses typed
    /// LEASE_MISSING — never creates one". NEGATIVE test: the missing state is
    /// constructed and the refusal bytes are pinned exactly.
    #[test]
    fn renewing_an_absent_lease_refuses_lease_missing_and_creates_nothing() {
        let tmp = fixture();
        let root = tmp.path();
        let expected_file = lease_cells_dir(root).join("ghost.json");
        // The path arm's file name is oracle-pinned (a live `node` renewLease
        // of `./src/gone.ts` names this exact hash in its refusal).
        assert_eq!(
            sha256_hex("path:src/gone.ts"),
            "ac310810eed41b715f5313f907fc1f45d9e1fa5656815de8610f85a48a46abef"
        );

        let (code, message) =
            refusal(renew_lease(root, &json!({"type":"cell","id":"ghost"}), 60.0, now_ms(), None, 1).unwrap_err());
        assert_eq!(code, "LEASE_MISSING");
        assert_eq!(
            message,
            format!("renewLease: no lease record at \"{}\" — acquire first.", expected_file.display())
        );
        assert!(!expected_file.exists(), "a refused renew never materializes the record");
        assert!(all_files(root).is_empty());

        // The same shape for a PATH resource, whose file name is the hash.
        let path_file =
            lease_paths_dir(root).join(format!("{}.json", sha256_hex("path:src/gone.ts")));
        let (code, message) = refusal(
            renew_lease(root, &json!({"type":"path","id":"./src/gone.ts"}), 60.0, now_ms(), None, 1)
                .unwrap_err(),
        );
        assert_eq!(code, "LEASE_MISSING");
        assert_eq!(
            message,
            format!("renewLease: no lease record at \"{}\" — acquire first.", path_file.display())
        );

        // Control: once acquired, the very same call succeeds — so the refusal
        // above is about the missing record, not a malformed request.
        acquire_leases(root, &[req("cell", "ghost", "s", 1.0)], now_ms()).unwrap();
        assert!(renew_lease(root, &json!({"type":"cell","id":"ghost"}), 60.0, now_ms(), None, 1).is_ok());
    }

    /// Oracle: "renewLeasesBySession renews only that session's leases; one
    /// swept concurrently is skipped, never resurrected".
    #[test]
    fn renew_by_session_touches_only_that_sessions_leases() {
        let tmp = fixture();
        let root = tmp.path();
        let t0 = 1_700_000_000_000.0;
        acquire_leases(
            root,
            &[
                req("cell", "mine-1", "sess-a", 1.0),
                req("path", "src/mine.ts", "sess-a", 1.0),
                req("cell", "theirs", "sess-b", 1.0),
            ],
            t0,
        )
        .unwrap();
        let theirs_before = read(&lease_cells_dir(root).join("theirs.json"));

        assert_eq!(renew_leases_by_session(root, "  sess-a  ", 60.0, t0 + 1000.0, 1).unwrap(), 2);
        assert_eq!(
            read(&lease_cells_dir(root).join("mine-1.json"))["expires_at"],
            json!("2023-11-14T22:14:21.000Z")
        );
        assert_eq!(
            read(&lease_cells_dir(root).join("theirs.json")),
            theirs_before,
            "another session's lease is untouched"
        );
        // An empty/blank session id is a no-op, never a whole-store renew.
        assert_eq!(renew_leases_by_session(root, "   ", 60.0, t0, 1).unwrap(), 0);
        // A session that owns nothing renews nothing.
        assert_eq!(renew_leases_by_session(root, "sess-c", 60.0, t0, 1).unwrap(), 0);
    }

    /// The second-port pin required by the module header: this module's renew
    /// must leave the SAME bytes on disk as hooks/state_sync.rs's narrowed
    /// `renew_lease_path` does for the shape they share (a path lease, no
    /// fencing, try-once lock). Re-derived here because that file is private
    /// and outside this cell's touchable set.
    #[test]
    fn agrees_with_the_hook_ports_on_the_shared_renew_fixture() {
        let tmp = fixture();
        let root = tmp.path();
        let t0 = 1_700_000_000_000.0;
        acquire_leases(root, &[req("path", "src/app.ts", "sess-a", 0.0)], t0).unwrap();
        let file = lease_paths_dir(root).join(format!("{}.json", sha256_hex("path:src/app.ts")));

        // The hook resolves the same file from the resource key and takes the
        // same `lease:<sha256(file)>` lock name.
        assert_eq!(
            lock_name_for_file(&file),
            format!("lease:{}", sha256_hex(&file.display().to_string()))
        );
        let renewed = renew_lease(root, &json!({"type":"path","id":"src/app.ts"}), 3600.0, t0 + 1000.0, None, 1).unwrap();
        // The hook writes `{...current, expires_at}` through writeJsonAtomic —
        // key order preserved, expires_at rewritten in place.
        let keys: Vec<&String> = renewed.as_object().unwrap().keys().collect();
        assert_eq!(
            keys,
            vec![
                "resource", "mode", "workflow_id", "session_id", "workspace_id", "epoch",
                "acquired_at", "expires_at", "kind"
            ]
        );
        assert_eq!(renewed["expires_at"], json!("2023-11-14T23:13:21.000Z"));
    }

    // ══ contract 3: LEASE_FENCE_STALE ══════════════════════════════════════

    /// Oracle: "renewLease refuses typed LEASE_FENCE_STALE when the presented
    /// epoch is behind the stored one, and the record on disk is left exactly
    /// as read — no partial write". NEGATIVE test: the stale state is
    /// constructed and the refusal bytes are pinned.
    #[test]
    fn a_stale_presented_epoch_refuses_the_renew_and_writes_nothing() {
        let tmp = fixture();
        let root = tmp.path();
        let t0 = 1_700_000_000_000.0;
        // A takeover already moved ownership forward: the stored epoch is 5.
        acquire_leases(root, &[req("cell", "c-1", "sess-new", 5.0)], t0).unwrap();
        let file = lease_cells_dir(root).join("c-1.json");
        let before = std::fs::read_to_string(&file).unwrap();

        for stale in [json!(4), json!(0), json!(-1)] {
            let (code, message) = refusal(
                renew_lease(root, &json!({"type":"cell","id":"c-1"}), 60.0, t0 + 9000.0, Some(&stale), 1)
                    .unwrap_err(),
            );
            assert_eq!(code, "LEASE_FENCE_STALE");
            assert_eq!(
                message,
                format!(
                    "renewLease: presented epoch {stale} is behind resource \"cell:c-1\"'s current epoch 5 — a takeover already moved ownership forward."
                )
            );
            assert_eq!(
                std::fs::read_to_string(&file).unwrap(),
                before,
                "a fenced refusal must leave the record byte-identical"
            );
        }
        // A non-finite presentation is stale by definition (!Number.isFinite).
        let (code, message) = refusal(
            renew_lease(root, &json!({"type":"cell","id":"c-1"}), 60.0, t0, Some(&json!(null)), 1)
                .unwrap_err(),
        );
        assert_eq!(code, "LEASE_FENCE_STALE");
        assert!(message.starts_with("renewLease: presented epoch null is behind"), "{message}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);

        // Controls: the CURRENT epoch and any AHEAD epoch both renew.
        for fresh in [json!(5), json!(6)] {
            assert!(
                renew_lease(root, &json!({"type":"cell","id":"c-1"}), 60.0, t0, Some(&fresh), 1).is_ok(),
                "presenting {fresh} must renew"
            );
        }
        // …and omitting the presentation entirely is the legacy, unfenced arm.
        assert!(renew_lease(root, &json!({"type":"cell","id":"c-1"}), 60.0, t0, None, 1).is_ok());
    }

    /// Oracle: "releaseLease refuses typed LEASE_FENCE_STALE on a stale
    /// presentation and the FILE IS NEVER REMOVED" — the safety-critical half:
    /// a stale fence must refuse, never silently proceed.
    #[test]
    fn a_stale_presented_epoch_refuses_the_release_and_never_removes_the_file() {
        let tmp = fixture();
        let root = tmp.path();
        acquire_leases(root, &[req("cell", "c-1", "sess-new", 5.0)], now_ms()).unwrap();
        let file = lease_cells_dir(root).join("c-1.json");
        let before = std::fs::read_to_string(&file).unwrap();
        let request = json!({"type":"cell","id":"c-1"});

        let stale = json!(4);
        let (code, message) = refusal(release_lease(root, &request, Some(&stale), 1).unwrap_err());
        assert_eq!(code, "LEASE_FENCE_STALE");
        assert_eq!(
            message,
            "releaseLease: presented epoch 4 is behind resource \"cell:c-1\"'s current epoch 5 — a takeover already moved ownership forward."
        );
        assert!(file.exists(), "a fenced release must NEVER delete the lease file");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);

        // The refusal carries the holder so a caller can name the takeover.
        match release_lease(root, &request, Some(&stale), 1).unwrap_err() {
            LeaseErr::Refused(r) => {
                assert_eq!(r.resource.as_deref(), Some("cell:c-1"));
                assert_eq!(r.holder.as_ref().unwrap()["session_id"], json!("sess-new"));
            }
            other => panic!("{}", other.text()),
        }

        // Control: a CURRENT presentation releases for real.
        assert_eq!(release_lease(root, &request, Some(&json!(5)), 1).unwrap(), Released(true));
        assert!(!file.exists());
        // Idempotent: releasing an absent lease is `released: false`, fenced or
        // not — there is nothing on disk to fence against.
        assert_eq!(release_lease(root, &request, Some(&json!(0)), 1).unwrap(), Released(false));
        assert_eq!(release_lease(root, &request, None, 1).unwrap(), Released(false));
    }

    /// The LEGACY (no presentedEpoch) release arm stays byte-unchanged: a
    /// single lock-free remove that takes no lock at all. Proven by holding
    /// the per-lease lock externally — the legacy arm must sail straight
    /// through it, while the FENCED arm must report the busy holder.
    #[test]
    fn the_legacy_release_arm_takes_no_lock_and_the_fenced_arm_does() {
        let tmp = fixture();
        let root = tmp.path();
        acquire_leases(
            root,
            &[req("cell", "unfenced", "s", 1.0), req("cell", "fenced", "s", 1.0)],
            now_ms(),
        )
        .unwrap();
        let unfenced = lease_cells_dir(root).join("unfenced.json");
        let fenced = lease_cells_dir(root).join("fenced.json");

        let mut held_a = lock::acquire_store_lock(root, &lock_name_for_file(&unfenced), 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));
        let mut held_b = lock::acquire_store_lock(root, &lock_name_for_file(&fenced), 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));

        assert_eq!(
            release_lease(root, &json!({"type":"cell","id":"unfenced"}), None, 1).unwrap(),
            Released(true),
            "the legacy arm never contends on the per-lease lock"
        );
        match release_lease(root, &json!({"type":"cell","id":"fenced"}), Some(&json!(1)), 1) {
            Err(LeaseErr::LockBusy(b)) => {
                assert!(b.message().starts_with("lock \"lease:"), "{}", b.message())
            }
            other => panic!("the fenced arm must take the lease lock, got {:?}", other.map(|r| r.0)),
        }
        assert!(fenced.exists(), "a lock-busy release removed nothing");
        held_a.release();
        held_b.release();
    }

    /// Renewing lease A never contends with lease B — the structural
    /// difference from reservations.mjs's one shared store lock.
    #[test]
    fn renewing_one_lease_never_contends_with_another() {
        let tmp = fixture();
        let root = tmp.path();
        acquire_leases(root, &[req("cell", "a", "s", 1.0), req("cell", "b", "s", 1.0)], now_ms())
            .unwrap();
        let a = lease_cells_dir(root).join("a.json");
        let mut held = lock::acquire_store_lock(root, &lock_name_for_file(&a), 1)
            .unwrap_or_else(|e| panic!("precondition: {}", e.message()));
        assert!(
            renew_lease(root, &json!({"type":"cell","id":"b"}), 60.0, now_ms(), None, 1).is_ok(),
            "a sibling lease renews through a held lease:<A> lock"
        );
        assert!(
            matches!(
                renew_lease(root, &json!({"type":"cell","id":"a"}), 60.0, now_ms(), None, 1),
                Err(LeaseErr::LockBusy(_))
            ),
            "the negative control: A's own renew IS denied by that same lock"
        );
        held.release();
    }
}
