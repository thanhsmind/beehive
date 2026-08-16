// the lease store, worktree holds and the sessions slice
//
// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::textutil::code_unit_cmp;
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── reservations.mjs / lease-store.mjs ports ──────────────────────────────

pub(crate) const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

pub(crate) const LIST_ALL_HOLDS_SENTINEL: &str = "\u{0}bee-reservations-list-all\u{0}";

pub(crate) const RESERVATION_KINDS: [&str; 2] = ["intent", "lease"];

pub(crate) const DEFAULT_TTL_SECONDS: f64 = 3600.0;

pub(crate) const CROSS_WORKTREE_HOLDS_LOCK: &str = "cross-worktree-holds";

/// provenance: reservations.mjs normalizePath == lease-store.mjs
/// canonicalizePath (kept in sync by hand there; one copy here).
pub(crate) fn res_normalize_path(v: &str) -> String {
    let mut s = v.replace('\\', "/");
    if s.starts_with("./") {
        // /^\.\/+/ — one dot, then one-or-more slashes
        let rest = s[1..].trim_start_matches('/');
        s = rest.to_string();
    }
    while s.ends_with('/') {
        s.pop();
    }
    s
}

/// String(value || '') for a possibly-absent JSON value, then normalizePath.
pub(crate) fn res_normalize_value(v: Option<&Value>) -> String {
    match v {
        Some(val) if truthy(val) => res_normalize_path(&js_disp(val)),
        _ => String::new(),
    }
}

/// provenance: reservations.mjs pathsOverlap.
pub(crate) fn paths_overlap(a: &str, b: &str) -> bool {
    let left = res_normalize_path(a);
    let right = res_normalize_path(b);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }
    let strip = |s: &str| -> String {
        if s.ends_with('*') {
            let mut t = s.trim_end_matches('*').to_string();
            while t.ends_with('/') {
                t.pop();
            }
            t
        } else {
            s.to_string()
        }
    };
    let lb = strip(&left);
    let rb = strip(&right);
    if lb == rb {
        return true;
    }
    if lb.is_empty() || rb.is_empty() {
        return true; // bare "*" covers everything
    }
    lb.starts_with(&format!("{}/", rb)) || rb.starts_with(&format!("{}/", lb))
}

pub(crate) fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// provenance: reservations.mjs findMainRoot/controlRootFor — the
/// self-contained, never-throwing git-common main-root walk (fails open to
/// `root` itself on every malformed shape).
pub(crate) fn control_root_for(root: &str) -> Ex<String> {
    let walked = (|| -> Ex<Option<String>> {
        // locateGitRootForRoot
        let mut dir = np_resolve1(root)?;
        let (work_root, marker) = loop {
            let m = Path::new(&dir).join(".git");
            if m.exists() {
                break (dir.clone(), m);
            }
            let parent = np_dirname(&dir);
            if parent == dir {
                return Ok(None);
            }
            dir = parent;
        };
        let is_file = match std::fs::metadata(&marker) {
            Ok(m) => m.is_file(),
            Err(_) => return Ok(None),
        };
        if !is_file {
            return Ok(Some(work_root)); // ordinary checkout: mainRoot === workRoot
        }
        let read_ptr = |file: &Path, base: &str| -> Ex<Option<String>> {
            let raw = match std::fs::read_to_string(file) {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };
            let mut raw = js_trim(&raw);
            if raw.is_empty() {
                return Ok(None);
            }
            if let Some(rest) = raw.strip_prefix("gitdir:") {
                raw = js_trim(rest);
            }
            // raw.replace(/\\/g, path.sep) — a no-op on Windows.
            let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
            np_resolve2(base, &fixed).map(Some)
        };
        let Some(gitdir) = read_ptr(&marker, &work_root)? else {
            return Ok(None); // malformed — fail open
        };
        let worktrees_root = np_resolve2(&gitdir, "..")?;
        let common_git_dir = np_resolve2(&worktrees_root, "..")?;
        if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
            return Ok(None);
        }
        let id = np_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return Ok(None);
        }
        let marker_s = marker.to_string_lossy().into_owned();
        let Some(reverse) = read_ptr(&Path::new(&gitdir).join("gitdir"), &gitdir)? else {
            return Ok(None);
        };
        // Identity, not spelling (roots.rs `same_path`). This copy fails OPEN,
        // so a byte-compare miss does not error — it quietly answers with the
        // worktree when main is correct, which is how a granted worktree came
        // to read its own empty holds ledger on Windows and release nobody.
        if !crate::roots::same_path(&reverse, &marker_s) {
            return Ok(None);
        }
        Ok(Some(np_dirname(&common_git_dir)))
    })()?;
    Ok(walked.unwrap_or_else(|| root.to_string()))
}

/// provenance: lease-store.mjs listAllLeaseFiles + readLeaseSafe (silent skip
/// on corrupt — Node never warns there) filtered to path-type leases
/// (reservations.mjs listPathLeaseRecords), numbers reshaped like JSON.parse.
pub(crate) fn list_path_lease_records(root: &str) -> Ex<Vec<Map<String, Value>>> {
    let control = control_root_for(root)?;
    let leases_root = Path::new(&control).join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue, // directory not created yet
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(dir.join(&name)) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    if let Value::Object(m) = js_numberify(&parsed)? {
                        let is_path = matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                        if is_path {
                            out.push(m);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// On-disk file for a path resource key (lease-store.mjs resolveResourceFile,
/// path branch): the resource is re-canonicalized from its own key, never
/// trusted from the source filename.
pub(crate) fn path_lease_file(control_root: &str, raw_path_id: &str) -> PathBuf {
    let canonical = res_normalize_path(raw_path_id);
    let resource_key = format!("path:{canonical}");
    Path::new(control_root)
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)))
}

/// The reservation-shape view of a path lease (reservations.mjs
/// leaseToReservation / leaseAgent / leaseTtlSeconds). Option = absent key
/// (dropped by JSON.stringify, shows as "undefined" in text templates).
pub(crate) struct Resv {
    pub(crate) agent: Option<Value>,
    pub(crate) cell: Option<Value>,
    pub(crate) path: String,
    pub(crate) ttl_seconds: Option<f64>, // None = NaN → JSON null
    pub(crate) reserved_at: Option<Value>,
    pub(crate) session: Option<Value>,
    pub(crate) kind: Value,
    /// D3 (wtf-2) — a Rust-side addition with no JS parity: the checkout
    /// that wrote this lease ("main", or a granted worktree's git-verified
    /// id), absent on a lease written before this change or from an
    /// ungranted linked worktree (no topology to report).
    pub(crate) holder: Option<Value>,
}

pub(crate) fn lease_to_reservation(rec: &Map<String, Value>) -> Ex<Resv> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(Exotic), // callers filter to path leases; defensive
    };
    let ttl = match rec.get("expires_at") {
        None | Some(Value::Null) => Some(0.0), // never-expires sentinel
        Some(exp) => {
            let e = date_parse_val(Some(exp))?;
            let a = date_parse_val(rec.get("acquired_at"))?;
            match (e, a) {
                (Some(e), Some(a)) => Some(js_round((e - a) / 1000.0).max(0.0)),
                _ => None, // NaN flows through Math.round/max
            }
        }
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => {
            Value::String(s["agent:".len()..].to_string())
        }
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if truthy(v) && !v_is_str(v, SESSIONLESS_SESSION_ID) => Some(v.clone()),
        _ => None,
    };
    let kind = match rec.get("kind") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("lease".into()),
    };
    // D3 (wtf-2): absent on any record written before this change, and on a
    // reserve from an ungranted linked worktree — a plain key-absent read,
    // same Option-and-omit pattern every other optional field here uses.
    let holder = rec.get("holder").cloned();
    Ok(Resv {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        ttl_seconds: ttl,
        reserved_at: rec.get("acquired_at").cloned(),
        session,
        kind,
        holder,
    })
}

/// Serialization order pinned by leaseToReservation's object literal: agent,
/// cell, path, ttl_seconds, reserved_at, released_at, [session], kind —
/// undefined-valued keys drop out of JSON.stringify.
pub(crate) fn resv_to_value(r: &Resv) -> Value {
    let mut m = Map::new();
    if let Some(a) = &r.agent {
        m.insert("agent".into(), a.clone());
    }
    if let Some(c) = &r.cell {
        m.insert("cell".into(), c.clone());
    }
    m.insert("path".into(), Value::String(r.path.clone()));
    m.insert(
        "ttl_seconds".into(),
        r.ttl_seconds
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null), // JSON.stringify(NaN) → null
    );
    if let Some(ra) = &r.reserved_at {
        m.insert("reserved_at".into(), ra.clone());
    }
    m.insert("released_at".into(), Value::Null);
    if let Some(s) = &r.session {
        m.insert("session".into(), s.clone());
    }
    m.insert("kind".into(), r.kind.clone());
    // D3 (wtf-2): emitted only when present, so a pre-cell or ungranted-
    // worktree reservation round-trips with no key at all.
    if let Some(h) = &r.holder {
        m.insert("holder".into(), h.clone());
    }
    Value::Object(m)
}

/// provenance: reservations.mjs isLeaseRecordExpired (raw record, never the
/// translated shape).
pub(crate) fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> Ex<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match date_parse_val(Some(v))? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

/// provenance: reservations.mjs listReservations.
///
/// pub(crate) since R6: verbs/state_group.rs's `state rebuild-projections`
/// needs it for rebuildReservationsProjection, and verbs/cells.rs's
/// `cells claim-next` needs it for findSessionConflicts.
pub(crate) fn list_reservations(root: &str, active_only: bool, now: f64) -> Ex<Vec<Resv>> {
    let mut out = Vec::new();
    for rec in list_path_lease_records(root)? {
        if active_only && lease_record_expired(&rec, now)? {
            continue;
        }
        out.push(lease_to_reservation(&rec)?);
    }
    Ok(out)
}

/// provenance: reservations.mjs reservationsPath.
pub(crate) fn reservations_path(root: &Path) -> PathBuf {
    root.join(".bee").join("reservations.json")
}

/// provenance: reservations.mjs rebuildReservationsProjection (msn-16/msn-18b).
/// Returns the row count (`{ authoritative: true, count }`'s count — the
/// authoritative flag is an unconditional `true` at the call site, because this
/// projection is never gated on workflow records).
///
/// The READ is control-rooted inside listReservations → listPathLeaseRecords;
/// the WRITE deliberately stays on the caller's own workspace root (msn-18b:
/// `.bee/reservations.json` is a legacy single-checkout DISPLAY projection).
///
/// A row whose `reserved_at` is not a string would make Node's comparator
/// inconsistent (`undefined < undefined` is false, so it answers 1 both ways)
/// and V8's TimSort order becomes implementation detail — delegate instead.
pub(crate) fn rebuild_reservations_projection(root: &Path) -> Ex<usize> {
    let root_s = root.to_str().ok_or(Exotic)?;
    let rows = list_reservations(root_s, true, now_ms())?;
    // `a.reserved_at !== b.reserved_at ? (a.reserved_at < b.reserved_at ? -1 : 1)
    //  : a.path !== b.path ? (a.path < b.path ? -1 : 1) : 0` — JS string
    // relational comparison is UTF-16 code-unit lexicographic.
    let mut keyed: Vec<(String, String, Resv)> = Vec::with_capacity(rows.len());
    for r in rows {
        let Some(Value::String(ra)) = r.reserved_at.clone() else {
            return Err(Exotic);
        };
        let path_key = r.path.clone();
        keyed.push((ra, path_key, r));
    }
    keyed.sort_by(|a, b| code_unit_cmp(&a.0, &b.0).then_with(|| code_unit_cmp(&a.1, &b.1)));
    let rows: Vec<Value> = keyed.iter().map(|(_, _, r)| resv_to_value(r)).collect();
    let count = rows.len();
    write_json_atomic(
        &reservations_path(root),
        &json!({ "reservations": Value::Array(rows) }),
    )
    .map_err(|_| Exotic)?;
    Ok(count)
}

/// provenance: reservations.mjs findConflicts (agent-keyed, activeOnly).
pub(crate) fn find_conflicts(root: &str, agent: &str, path: &str, now: f64) -> Ex<Vec<Resv>> {
    let mut out = Vec::new();
    for resv in list_reservations(root, true, now)? {
        let same_agent = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        if !same_agent && paths_overlap(&resv.path, path) {
            out.push(resv);
        }
    }
    Ok(out)
}

/// provenance: reservations.mjs isHardConflict.
pub(crate) fn is_hard_conflict(resv: &Resv, target_path: &str) -> bool {
    let is_intent = v_is_str(&resv.kind, "intent");
    !(is_intent && res_normalize_path(&resv.path) != res_normalize_path(target_path))
}

// ─── worktree-holds.mjs ports ──────────────────────────────────────────────

pub(crate) fn holds_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// provenance: worktree-holds.mjs readStore — a missing, corrupt or
/// shape-less file is an empty ledger. A corrupt file WARNS once and then
/// takes `readJson(..., null)`'s fallback, which Node's `!store` guard turned
/// into `{ holds: [] }` — identical result, one line of explanation.
/// `null` entries (a JS property-access crash downstream) stay Exotic.
pub(crate) fn read_holds_store(root: &Path) -> Ex<Value> {
    let store = match read_json(&holds_ledger_path(root)) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&holds_ledger_path(root));
            None
        }
        ReadJson::Parsed(v) => Some(js_numberify(&v)?),
    };
    let ok_shape = store
        .as_ref()
        .map(|s| matches!(jget(s, "holds"), Some(Value::Array(_))))
        .unwrap_or(false);
    if !ok_shape {
        return Ok(json!({ "holds": [] }));
    }
    let store = store.unwrap();
    if let Some(Value::Array(holds)) = jget(&store, "holds") {
        if holds.iter().any(|h| h.is_null()) {
            return Err(Exotic); // `hold.released_at` on null throws in Node
        }
    }
    Ok(store)
}

pub(crate) fn holds_of(store: &Value) -> &Vec<Value> {
    match jget(store, "holds") {
        Some(Value::Array(a)) => a,
        _ => unreachable!("read_holds_store guarantees the shape"),
    }
}

/// provenance: worktree-holds.mjs isExpired.
pub(crate) fn hold_expired(hold: &Value, now: f64) -> Ex<bool> {
    let ttl = match jget(hold, "ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        _ => return Ok(false), // Number.isFinite(non-number) → false
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return Ok(false);
    }
    match date_parse_val(jget(hold, "mirrored_at"))? {
        None => Ok(false),
        Some(m) => Ok(m + ttl * 1000.0 <= now),
    }
}

/// provenance: worktree-holds.mjs isActive.
pub(crate) fn hold_active(hold: &Value, now: f64) -> Ex<bool> {
    let released = jget(hold, "released_at");
    let unreleased = matches!(released, None | Some(Value::Null)); // == null
    Ok(unreleased && !hold_expired(hold, now)?)
}

/// provenance: worktree-holds.mjs findForeignHolds (read-only, unlocked).
pub(crate) fn find_foreign_holds<'a>(
    store: &'a Value,
    acting: &str,
    request_path: &str,
    now: f64,
) -> Ex<Vec<&'a Value>> {
    let mut out = Vec::new();
    for hold in holds_of(store) {
        if !hold_active(hold, now)? {
            continue;
        }
        // hold.holder !== acting (strict) — non-strings can never equal it.
        let same = matches!(jget(hold, "holder"), Some(Value::String(s)) if s == acting);
        if same {
            continue;
        }
        let hold_path = res_normalize_value(jget(hold, "path"));
        if paths_overlap(&hold_path, request_path) {
            out.push(hold);
        }
    }
    Ok(out)
}

/// Who owns this cell's holds — the answer every hold-reading site asks for
/// instead of "is this row mine?" (docs/history/hold-holder-attribution/plan.md).
///
/// A hold belongs to the work stream that owns the CELL, never to the checkout
/// that happened to type the command. bee's control plane MUST run from main
/// (drivers/prepare.rs), so a reserve made from main on behalf of a granted
/// worktree's cell used to stamp `holder: "main"` on a row the worktree alone
/// can ever satisfy — the guard then denied that worktree's own writes.
pub(crate) struct CellHoldOwner {
    /// The granted worktree's id when the cell's feature owns one, else the
    /// ACTING topology holder, unchanged from before this rule existed.
    pub(crate) holder: String,
    /// The cell's own feature, resolved so a refusal can name it instead of
    /// reading "feature unknown" (hooks/write_guard/checks.rs).
    pub(crate) feature: Option<String>,
}

/// `cell -> feature -> granted worktree`, composed out of the two lookups that
/// already answer each half (`verbs/cells/finish_support.rs` composes the same
/// pair for the commit-trailer history root). Both halves are plain filesystem
/// reads — `.bee/cells/<id>.json` plus `worktree-grants.json` and the gitdir
/// pointers — with no store lock and no read of the holds ledger, so this is
/// safe to call from INSIDE the CROSS_WORKTREE_HOLDS_LOCK section.
///
/// `read_cell` answers `Delegate` (unreachable today: a corrupt cell file
/// warns and reads as absent), mapped here to this module's own Exotic channel
/// so a caller's `?` delegates the whole command exactly as it already does.
pub(crate) fn cell_hold_owner(main_root: &Path, acting: &str, cell: &str) -> Ex<CellHoldOwner> {
    let id = js_trim(cell);
    let mine = || CellHoldOwner { holder: acting.to_string(), feature: None };
    if id.is_empty() {
        return Ok(mine());
    }
    let Some(record) = crate::verbs::cells::read_cell(main_root, id).map_err(|_| Exotic)? else {
        return Ok(mine());
    };
    let feature = match jget(&record, "feature") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return Ok(mine()),
    };
    let holder = crate::verbs::status_full::find_granted_worktree_for_feature(main_root, &feature)
        .map(|(id, _root)| id)
        .unwrap_or_else(|| acting.to_string());
    Ok(CellHoldOwner { holder, feature: Some(feature) })
}

/// provenance: bee.mjs holdForeignExpiry.
pub(crate) fn hold_foreign_expiry(hold: &Value) -> Ex<String> {
    let mirrored = date_parse_val(jget(hold, "mirrored_at"))?;
    let ttl = match jget(hold, "ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    match (mirrored, ttl) {
        (Some(m), Some(t)) if t > 0.0 => Ok(format!("expires {}", iso_from_ms(m + t * 1000.0)?)),
        _ => Ok("no expiry".to_string()),
    }
}

// ─── claims.mjs ports (sessions slice) ─────────────────────────────────────

pub(crate) const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

/// provenance: claims.mjs listSessionRecords (fail-open flavor). A corrupt
/// record WARNS once and is skipped — `readJson(file, null)`'s fallback fails
/// readSession's `!session` guard, so the scan continues over the rest.
pub(crate) fn list_session_records(control_root: &str) -> Ex<Vec<Map<String, Value>>> {
    let dir = Path::new(control_root).join(".bee").join("sessions");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let stem = &name[..name.len() - ".json".len()];
        // requireId inside sessionPath: trimmed; separators/'..' throw → the
        // catch in readSession reads them as "no session".
        let id = js_trim(stem);
        if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
            continue;
        }
        let file = dir.join(format!("{id}.json"));
        match read_json(&file) {
            ReadJson::Missing => continue,
            // readJson's `null` fallback fails readSession's `!session`
            // guard, so the record is skipped and the scan continues.
            ReadJson::Corrupt => {
                crate::fsutil::warn_corrupt_json(&file);
                continue;
            }
            ReadJson::Parsed(v) => {
                if let Value::Object(m) = js_numberify(&v)? {
                    if matches!(m.get("id"), Some(Value::String(s)) if s == id) {
                        out.push(m);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// provenance: claims.mjs heartbeatStale. A session marked "closed" (clean
/// SessionEnd) or "dead" (crash-swept) reads as stale unconditionally,
/// matching cells::heartbeat_stale.
pub(crate) fn heartbeat_stale(session: &Map<String, Value>, now: f64) -> Ex<bool> {
    if matches!(session.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
        return Ok(true);
    }
    match date_parse_val(session.get("last_heartbeat"))? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

pub(crate) fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// provenance: claims.mjs resolveSessionId (flag → BEE_SESSION_ID →
/// CLAUDE_CODE_SESSION_ID → single-live-session adoption → null).
///
/// pub(crate) since the `worktree new` port: bee.mjs's handleWorktreeNew
/// resolves the acting session the same way, against controlRootFor(mainRoot).
pub(crate) fn resolve_session_id(flag: Option<&str>, control_root: &str) -> Ex<Option<String>> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Ok(Some(js_trim(f).to_string()));
        }
    }
    if let Some(v) = env_nonempty("BEE_SESSION_ID") {
        return Ok(Some(v));
    }
    if let Some(v) = env_nonempty("CLAUDE_CODE_SESSION_ID") {
        return Ok(Some(v));
    }
    let records = list_session_records(control_root)?;
    let mut fresh: Vec<&Map<String, Value>> = Vec::new();
    for r in &records {
        if !heartbeat_stale(r, now_ms())? {
            fresh.push(r);
        }
    }
    if fresh.len() == 1 {
        if let Some(Value::String(id)) = fresh[0].get("id") {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

/// provenance: claims.mjs isConcurrentMode (no exclusion — the sessionless
/// reserve caller has no id of its own).
pub(crate) fn is_concurrent_mode(control_root: &str) -> Ex<bool> {
    is_concurrent_mode_excluding(control_root, None)
}

/// provenance: claims.mjs isConcurrentMode with `excludeSessionId` — the
/// acting session's OWN heartbeat is never "another" live session.
///
/// `strict` has no separate arm here. CUTOVER: `list_session_records` used to
/// answer `Exotic` for an unreadable record, which made this port
/// strict-equivalent by construction. It now warns and SKIPS that record,
/// matching Node's own non-strict readSession — so an unreadable record reads
/// as "not a live session" and is loudly reported, rather than silently
/// disappearing. (guards.mjs's strict mode, which throws instead, is a
/// separate caller and is not reached from here.)
///
/// pub(crate) since the `worktree new` port (bee.mjs's wcg-3 guard).
pub(crate) fn is_concurrent_mode_excluding(
    control_root: &str,
    exclude_session_id: Option<&str>,
) -> Ex<bool> {
    let exclude = exclude_session_id.map(js_trim).unwrap_or("");
    let now = now_ms();
    for session in list_session_records(control_root)? {
        let is_excluded = matches!(session.get("id"), Some(Value::String(s)) if s == exclude);
        if !is_excluded && !heartbeat_stale(&session, now)? {
            return Ok(true);
        }
    }
    Ok(false)
}
