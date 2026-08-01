// workspace_store — Rust port of lib/workspace-store.mjs (the workspace
// registry + the single-write-owner lock, multisession-native-19).
//
// Library module (no `try_native`): it has no verb of its own. Its consumer
// is lib/state.mjs's applyWritePolicy — the write-capable door
// `state start-feature` and `cells claim`/`claim-next` all run through.
//
// Structural isolation is preserved from the .mjs: this module imports ONLY
// crate::{fsutil, jsjson, lock}. It never reaches into claims/state/
// worktree/reservations code, so `workspace:<id>` can never nest inside — or
// be nested inside — the 'sessions' store lock. Liveness is a caller-supplied
// predicate (`is_owner_live`), exactly as in Node, and its default is the
// SAFE assumption ("the current owner is live"), never an accidental takeover.
//
// INTEROP (contract C1). Everything observable is byte-identical:
//   - record path: <root>/.bee/runtime/workspaces/<id>.json
//   - lock name: `workspace:<id>` through crate::lock (Node's withStoreLock),
//     so the two runtimes serialize against each other mid-campaign
//   - file bytes: writeJsonAtomic's `JSON.stringify(v, null, 2) + "\n"`
//   - KEY ORDER. This is the subtle one and it is load-bearing for C1:
//     readWorkspaceRecord returns
//       `{write_owner_session, fence_epoch, attached_sessions, branch,
//         base_sha, ...parsed, id}`
//     so those five default keys take the FRONT of the object and every
//     other key keeps its file order behind them. A record written by
//     registerWorkspace (id, type, root, branch, base_sha,
//     write_owner_session, fence_epoch, attached_sessions, created_at) is
//     therefore REORDERED the first time an ownership change rewrites it —
//     and Node does exactly that. `spread_read_defaults` below reproduces the
//     spread, so the on-disk bytes match Node's after every mutation, not
//     just after the create.
//
// DELEGATION (Ex) — shapes whose Node bytes embed a V8 message and so cannot
// be reproduced:
//   - a workspace record that exists but does not parse (the refusal text
//     interpolates `err.message` from JSON.parse)
//   - a non-ENOENT read error (the refusal interpolates `err.code`, and the
//     errno set is platform-open)
//   - a record whose `id` field is an object/array (the refusal interpolates
//     `${parsed.id}`, i.e. V8's own ToString for exotic values)
//   - any fs write failure (a V8-worded throw in Node)
// Every one of these is PROBED BEFORE the lock is taken (`probe_readable`),
// so the normal path never delegates from inside a lock hold — delegating
// there would double lock.rs's contention.jsonl telemetry (campaign rule 2).
// The one accepted residual, identical in kind to verbs/worktree.rs's: a
// record that becomes unreadable BETWEEN the probe and the in-lock read (or a
// write that fails) still delegates late. Every step taken by then is
// idempotent — registerWorkspace is idempotent by construction, and an
// ownership decision that never wrote leaves the record untouched — so the
// Node re-run reproduces the same answer over the same store.
//
// Provenance: lib/workspace-store.mjs WorkspaceStoreError / TYPE_VALUES /
// runtimeDir / workspacesDir / requireWorkspaceId / requireType /
// requireWorkspaceRoot / workspacePath / withWorkspaceLock /
// readWorkspaceRecord / readWorkspace / writeWorkspaceFileAtomic /
// registerWorkspace / unregisterWorkspace / listWorkspaces / decideOwnership /
// applyOwnershipTakeover / claimWriteOwnership / attachWorkspace /
// releaseWriteOwnership.

#![allow(dead_code)] // the applyWritePolicy consumer lands with start-feature

use crate::fsutil::write_json_atomic;
use crate::jsjson;
use crate::lock;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// workspace-store.mjs TYPE_VALUES.
pub(crate) const TYPE_VALUES: [&str; 2] = ["main", "worktree"];

/// Every failure this module can produce.
///
/// `Ex` = "Node's bytes here embed a V8 message" — the caller must return
/// None before emitting anything and let the whole command re-run under Node.
/// `Err` = a reproducible thrown Error: a typed WorkspaceStoreError (with its
/// `code`, which callers switch on) or a LockBusyError (code `None`). bee.mjs
/// surfaces both through the same `emitError(error.message)` seam.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WsErr {
    Ex,
    Err {
        code: Option<&'static str>,
        message: String,
        /// WorkspaceStoreError's optional `holder` detail (WORKSPACE_OWNED).
        holder: Option<String>,
    },
}

impl WsErr {
    fn refuse(code: &'static str, message: String) -> Self {
        WsErr::Err { code: Some(code), message, holder: None }
    }
    fn refuse_holder(code: &'static str, message: String, holder: String) -> Self {
        WsErr::Err { code: Some(code), message, holder: Some(holder) }
    }
    pub(crate) fn code(&self) -> Option<&'static str> {
        match self {
            WsErr::Ex => None,
            WsErr::Err { code, .. } => *code,
        }
    }
    pub(crate) fn message(&self) -> &str {
        match self {
            WsErr::Ex => "",
            WsErr::Err { message, .. } => message,
        }
    }
    pub(crate) fn holder(&self) -> Option<&str> {
        match self {
            WsErr::Ex => None,
            WsErr::Err { holder, .. } => holder.as_deref(),
        }
    }
}

type W<T> = Result<T, WsErr>;

// ─── paths ────────────────────────────────────────────────────────────────

pub(crate) fn runtime_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime")
}

pub(crate) fn workspaces_dir(root: &Path) -> PathBuf {
    runtime_dir(root).join("workspaces")
}

/// requireWorkspaceId: an id becomes a filename, so separators and '..' are
/// refused rather than allowed to escape the directory. `value` arrives here
/// already known to be a JS string (every call site passes one).
pub(crate) fn require_workspace_id(value: &str) -> W<String> {
    if value.trim().is_empty() {
        return Err(WsErr::refuse(
            "WORKSPACE_INVALID_ID",
            "workspace id is required.".to_string(),
        ));
    }
    let id = value.trim().to_string();
    if id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(WsErr::refuse(
            "WORKSPACE_INVALID_ID",
            format!(
                "workspace id \"{id}\" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/workspaces/."
            ),
        ));
    }
    Ok(id)
}

/// requireType — `TYPE_VALUES.includes(value)`, else a typed refusal that
/// interpolates `JSON.stringify(value)`.
pub(crate) fn require_type(value: &str) -> W<String> {
    if TYPE_VALUES.contains(&value) {
        return Ok(value.to_string());
    }
    Err(WsErr::refuse(
        "WORKSPACE_INVALID_TYPE",
        format!(
            "workspace type must be one of {} (got {}).",
            TYPE_VALUES.join("/"),
            jsjson::stringify(&Value::String(value.to_string()))
        ),
    ))
}

fn require_workspace_root(value: &str) -> W<String> {
    if value.trim().is_empty() {
        return Err(WsErr::refuse(
            "WORKSPACE_INVALID_ROOT",
            "workspace root (physical checkout path) is required.".to_string(),
        ));
    }
    Ok(value.to_string())
}

pub(crate) fn workspace_path(root: &Path, id: &str) -> W<PathBuf> {
    Ok(workspaces_dir(root).join(format!("{}.json", require_workspace_id(id)?)))
}

/// withWorkspaceLock's name, byte-identical to Node's `workspace:<id>`.
pub(crate) fn workspace_lock_name(id: &str) -> W<String> {
    Ok(format!("workspace:{}", require_workspace_id(id)?))
}

/// withWorkspaceLock(root, id, ...) — the RAII half of Node's closure form.
/// A LockBusyError escapes as a plain thrown Error with `code: None`, exactly
/// how bee.mjs surfaces it.
fn acquire_workspace_lock(root: &Path, id: &str) -> W<lock::LockGuard> {
    let name = workspace_lock_name(id)?;
    lock::acquire_store_lock(root, &name, lock::MAX_ATTEMPTS)
        .map_err(|busy| WsErr::Err { code: None, message: busy.message(), holder: None })
}

// ─── read ─────────────────────────────────────────────────────────────────

/// `path.relative(root, file)` as the refusal's FIX hint spells it. Node
/// emits forward or backslashes per platform (path.relative is native), which
/// is what `strip_prefix` + the platform separator reproduces.
fn path_relative(root: &Path, file: &Path) -> String {
    match file.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    }
}

/// `{write_owner_session: null, fence_epoch: 0, attached_sessions: [],
///   branch: null, base_sha: null, ...parsed, id: workspaceId}` — the spread
/// whose KEY ORDER the on-disk bytes depend on (see the module header).
fn spread_read_defaults(parsed: &Map<String, Value>, workspace_id: &str) -> Map<String, Value> {
    let mut out = Map::new();
    out.insert("write_owner_session".into(), Value::Null);
    out.insert("fence_epoch".into(), json!(0));
    out.insert("attached_sessions".into(), Value::Array(vec![]));
    out.insert("branch".into(), Value::Null);
    out.insert("base_sha".into(), Value::Null);
    for (k, v) in parsed {
        out.insert(k.clone(), v.clone()); // existing key keeps its position
    }
    out.insert("id".into(), Value::String(workspace_id.to_string()));
    out
}

/// Would `read_workspace_record` be able to answer without delegating?
/// Run BEFORE every lock acquire so the in-lock read is Ex-free (module
/// header). `Ok(())` covers both "missing" and "readable and modelable".
fn probe_readable(root: &Path, workspace_id: &str) -> W<()> {
    let file = workspace_path(root, workspace_id)?;
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(WsErr::Ex), // WORKSPACE_CORRUPT interpolates err.code
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed: Value = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(v) => v,
        Err(_) => return Err(WsErr::Ex), // the refusal interpolates err.message
    };
    // Only the `${parsed.id}` interpolation of an exotic value is unmodelable;
    // every other shape below is deterministic.
    if let Value::Object(m) = &parsed {
        match m.get("id") {
            Some(Value::Object(_)) | Some(Value::Array(_)) => return Err(WsErr::Ex),
            _ => {}
        }
    }
    Ok(())
}

fn read_workspace_record(root: &Path, workspace_id: &str) -> W<Map<String, Value>> {
    let file = workspace_path(root, workspace_id)?;
    let file_str = file.to_string_lossy().into_owned();
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WsErr::refuse(
                "WORKSPACE_MISSING",
                format!(
                    "readWorkspace: no workspace record at \"{file_str}\". FIX: registerWorkspace first, or check the id."
                ),
            ))
        }
        Err(_) => return Err(WsErr::Ex),
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed: Value = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(v) => v,
        Err(_) => return Err(WsErr::Ex),
    };
    let obj = match &parsed {
        Value::Object(m) => m,
        other => {
            // `!parsed || typeof parsed !== 'object' || Array.isArray(parsed)`
            let found = match other {
                Value::Array(_) => "an array".to_string(),
                Value::Null => "object".to_string(), // `typeof null === 'object'`
                Value::Bool(_) => "boolean".to_string(),
                Value::Number(_) => "number".to_string(),
                Value::String(_) => "string".to_string(),
                Value::Object(_) => unreachable!(),
            };
            return Err(WsErr::refuse(
                "WORKSPACE_CORRUPT",
                format!(
                    "readWorkspace: \"{file_str}\" exists but is not a JSON object (found {found})."
                ),
            ));
        }
    };
    let id_matches = matches!(obj.get("id"), Some(Value::String(s)) if s == workspace_id);
    if !id_matches {
        let shown = match obj.get("id") {
            Some(Value::Object(_)) | Some(Value::Array(_)) => return Err(WsErr::Ex),
            Some(v) => jsjson::js_to_string(v),
            None => "undefined".to_string(),
        };
        let _ = path_relative(root, &file); // (the FIX hint below spells it out)
        return Err(WsErr::refuse(
            "WORKSPACE_CORRUPT",
            format!(
                "readWorkspace: \"{file_str}\" exists but its id field (\"{shown}\") does not match the requested workspace \"{workspace_id}\" — never trusted."
            ),
        ));
    }
    Ok(spread_read_defaults(obj, workspace_id))
}

/// readWorkspace(root, id).
pub(crate) fn read_workspace(root: &Path, id: &str) -> W<Map<String, Value>> {
    let workspace_id = require_workspace_id(id)?;
    read_workspace_record(root, &workspace_id)
}

fn write_workspace_file_atomic(root: &Path, id: &str, record: &Map<String, Value>) -> W<()> {
    let file = workspace_path(root, id)?;
    std::fs::create_dir_all(workspaces_dir(root)).map_err(|_| WsErr::Ex)?;
    write_json_atomic(&file, &Value::Object(record.clone())).map_err(|_| WsErr::Ex)
}

// ─── register / unregister / list ─────────────────────────────────────────

pub(crate) struct RegisterSpec<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub root: &'a str,
    pub branch: Option<&'a str>,
    pub base_sha: Option<&'a str>,
}

/// registerWorkspace — IDEMPOTENT create: an existing record for `id` is
/// returned UNCHANGED (never overwritten, never an error). See the .mjs
/// header for why this must not be an O_EXCL refuse-on-exists.
///
/// `now_iso` is injected so a caller (and the tests) can pin `created_at`;
/// production passes `chrono::Utc::now()`'s ISO form.
pub(crate) fn register_workspace(
    root: &Path,
    spec: RegisterSpec<'_>,
    now_iso: &str,
) -> W<Map<String, Value>> {
    let workspace_id = require_workspace_id(spec.id)?;
    let workspace_type = require_type(spec.kind)?;
    let resolved_root = require_workspace_root(spec.root)?;
    probe_readable(root, &workspace_id)?;

    let mut guard = acquire_workspace_lock(root, &workspace_id)?;
    let out = (|| -> W<Map<String, Value>> {
        let file = workspace_path(root, &workspace_id)?;
        if file.exists() {
            return read_workspace_record(root, &workspace_id);
        }
        let mut record = Map::new();
        record.insert("id".into(), json!(workspace_id));
        record.insert("type".into(), json!(workspace_type));
        record.insert("root".into(), json!(resolved_root));
        // `typeof branch === 'string' && branch ? branch : null`
        record.insert(
            "branch".into(),
            spec.branch.filter(|s| !s.is_empty()).map_or(Value::Null, |s| json!(s)),
        );
        record.insert(
            "base_sha".into(),
            spec.base_sha.filter(|s| !s.is_empty()).map_or(Value::Null, |s| json!(s)),
        );
        record.insert("write_owner_session".into(), Value::Null);
        record.insert("fence_epoch".into(), json!(0));
        record.insert("attached_sessions".into(), Value::Array(vec![]));
        record.insert("created_at".into(), json!(now_iso));
        write_workspace_file_atomic(root, &workspace_id, &record)?;
        Ok(record)
    })();
    guard.release();
    out
}

/// unregisterWorkspace — idempotent delete: `{ok: true, removed: false}` when
/// there was nothing on disk (lease-store.mjs's releaseLease tolerance).
pub(crate) fn unregister_workspace(root: &Path, id: &str) -> W<Value> {
    let workspace_id = require_workspace_id(id)?;
    let mut guard = acquire_workspace_lock(root, &workspace_id)?;
    let out = (|| -> W<Value> {
        let file = workspace_path(root, &workspace_id)?;
        match std::fs::remove_file(&file) {
            Ok(()) => Ok(json!({ "ok": true, "removed": true })),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(json!({ "ok": true, "removed": false }))
            }
            // Node rethrows a non-ENOENT rmSync error (V8-worded).
            Err(_) => Err(WsErr::Ex),
        }
    })();
    guard.release();
    out
}

/// listWorkspaces — fail-open enumeration returning `(workspaces, skipped)`.
///
/// DIVERGENCE, deliberate and identical in kind to workflow_store.rs's
/// `list_workflows`: Node SKIPS an unreadable entry, pushes `{id, reason}`,
/// and `console.warn`s the reason — and that reason can embed a V8 message.
/// So an entry this port cannot model turns the whole call into `Ex`
/// (delegate) rather than a silently different `skipped` list. When every
/// entry is modelable the two agree exactly, including the skip list.
pub(crate) fn list_workspaces(root: &Path) -> W<(Vec<Map<String, Value>>, Vec<Value>)> {
    let entries = match std::fs::read_dir(workspaces_dir(root)) {
        Ok(e) => e,
        Err(_) => return Ok((vec![], vec![])),
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        if !is_file {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        names.push(name);
    }
    // readdirSync order is the OS's; std::fs::read_dir matches it on both
    // NTFS and ext4 for the same directory, and the caller sorts when order
    // matters (no caller today depends on it).
    let mut workspaces = Vec::new();
    let mut skipped = Vec::new();
    for name in names {
        let id = &name[..name.len() - ".json".len()];
        match read_workspace_record(root, id) {
            Ok(record) => workspaces.push(record),
            Err(WsErr::Ex) => return Err(WsErr::Ex),
            Err(e) => skipped.push(json!({ "id": id, "reason": e.message() })),
        }
    }
    Ok((workspaces, skipped))
}

// ─── ownership ────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum Outcome {
    AlreadyOwner,
    BecomeOwner,
    Reclaim,
    Blocked(String),
}

/// decideOwnership — the PURE decision core shared by claimWriteOwnership and
/// attachWorkspace, run under the SAME lock hold both acquire.
fn decide_ownership(
    record: &Map<String, Value>,
    session_id: &str,
    now: f64,
    is_owner_live: Option<&dyn Fn(&str, f64) -> bool>,
) -> Outcome {
    let owner = record.get("write_owner_session");
    let owner_str = match owner {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        // `!owner` — null/undefined/''/0/false are all falsy in JS.
        Some(Value::Null) | None => None,
        Some(Value::Bool(false)) => None,
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => None,
        Some(Value::String(_)) => None, // '' is falsy
        // A truthy non-string owner can never `=== sessionId`; it is passed to
        // the predicate as-is in Node. Model it by its string form.
        Some(other) => Some(jsjson::js_to_string(other)),
    };
    let Some(owner) = owner_str else {
        return Outcome::BecomeOwner;
    };
    if owner == session_id {
        return Outcome::AlreadyOwner;
    }
    // No predicate => the SAFE default: "the owner is live".
    let live = is_owner_live.map_or(true, |f| f(&owner, now));
    if !live {
        return Outcome::Reclaim;
    }
    Outcome::Blocked(owner)
}

/// applyOwnershipTakeover — `{...record, write_owner_session, fence_epoch,
/// owner_claimed_at, attached_sessions}`. Every key but `owner_claimed_at`
/// already exists (readWorkspaceRecord's spread guarantees it), so only
/// `owner_claimed_at` appends; the rest keep their positions.
fn apply_ownership_takeover(
    record: &Map<String, Value>,
    session_id: &str,
    now_iso: &str,
) -> Map<String, Value> {
    let attached: Vec<Value> = match record.get("attached_sessions") {
        Some(Value::Array(a)) => a
            .iter()
            .filter(|s| !matches!(s, Value::String(x) if x == session_id))
            .cloned()
            .collect(),
        _ => vec![],
    };
    let next_epoch = match record.get("fence_epoch") {
        // `Number.isFinite(record.fence_epoch) ? record.fence_epoch : 0` + 1
        Some(Value::Number(n)) => n.as_f64().filter(|f| f.is_finite()).unwrap_or(0.0) + 1.0,
        _ => 1.0,
    };
    let mut next = record.clone();
    next.insert("write_owner_session".into(), json!(session_id));
    next.insert("fence_epoch".into(), json!(js_num(next_epoch)));
    next.insert("owner_claimed_at".into(), json!(now_iso));
    next.insert("attached_sessions".into(), Value::Array(attached));
    next
}

/// JSON.stringify writes an integral double without a fractional part.
fn js_num(f: f64) -> Value {
    if f.fract() == 0.0 && f.abs() < 9.007_199_254_740_992e15 {
        json!(f as i64)
    } else {
        json!(f)
    }
}

pub(crate) struct OwnershipOpts<'a> {
    pub now: f64,
    pub now_iso: &'a str,
    pub is_owner_live: Option<&'a dyn Fn(&str, f64) -> bool>,
}

fn require_session(verb: &str, session_id: Option<&str>) -> W<String> {
    match session_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => Ok(s.to_string()),
        None => Err(WsErr::refuse(
            "WORKSPACE_INVALID_SESSION",
            format!("{verb}: sessionId is required."),
        )),
    }
}

/// The WORKSPACE_MISSING → WORKSPACE_NOT_REGISTERED remap both ownership
/// verbs share ("no unregistered workspace gains write ownership").
fn read_for_ownership(root: &Path, workspace_id: &str, verb: &str) -> W<Map<String, Value>> {
    match read_workspace_record(root, workspace_id) {
        Ok(r) => Ok(r),
        Err(e) if e.code() == Some("WORKSPACE_MISSING") => {
            let tail = if verb == "claimWriteOwnership" {
                " — no unregistered workspace may gain write ownership. FIX: registerWorkspace first (worktree create/merge and the main-checkout first-touch path both call this automatically)."
            } else {
                " — attach never auto-registers. FIX: registerWorkspace first."
            };
            Err(WsErr::refuse(
                "WORKSPACE_NOT_REGISTERED",
                format!("{verb}: workspace \"{workspace_id}\" has never been registered{tail}"),
            ))
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
pub(crate) struct ClaimOutcome {
    pub reclaimed: bool,
    pub record: Map<String, Value>,
}

/// claimWriteOwnership — the STRICT primitive: become the write owner or
/// throw a typed refusal. Never falls back to a read-only attach.
pub(crate) fn claim_write_ownership(
    root: &Path,
    id: &str,
    session_id: Option<&str>,
    opts: OwnershipOpts<'_>,
) -> W<ClaimOutcome> {
    let workspace_id = require_workspace_id(id)?;
    let session = require_session("claimWriteOwnership", session_id)?;
    probe_readable(root, &workspace_id)?;

    let mut guard = acquire_workspace_lock(root, &workspace_id)?;
    let out = (|| -> W<ClaimOutcome> {
        let record = read_for_ownership(root, &workspace_id, "claimWriteOwnership")?;
        match decide_ownership(&record, &session, opts.now, opts.is_owner_live) {
            Outcome::AlreadyOwner => Ok(ClaimOutcome { reclaimed: false, record }),
            Outcome::Blocked(owner) => Err(WsErr::refuse_holder(
                "WORKSPACE_OWNED",
                format!(
                    "claimWriteOwnership: workspace \"{workspace_id}\" is already write-owned by session \"{owner}\"."
                ),
                owner,
            )),
            other => {
                let next = apply_ownership_takeover(&record, &session, opts.now_iso);
                write_workspace_file_atomic(root, &workspace_id, &next)?;
                Ok(ClaimOutcome { reclaimed: other == Outcome::Reclaim, record: next })
            }
        }
    })();
    guard.release();
    out
}

#[derive(Debug)]
pub(crate) enum AttachRole {
    Owner { reclaimed: bool },
    ReadOnly { write_owner_session: String },
}

#[derive(Debug)]
pub(crate) struct AttachOutcome {
    pub role: AttachRole,
    pub record: Map<String, Value>,
}

/// attachWorkspace — the FORGIVING wrapper: becomes owner exactly like
/// claimWriteOwnership when ownership is free (or reclaimable), but records a
/// READ-ONLY ATTACH (deduplicated, idempotent) instead of throwing when a
/// DIFFERENT session already holds live ownership. Still refuses
/// WORKSPACE_NOT_REGISTERED on an unregistered id.
pub(crate) fn attach_workspace(
    root: &Path,
    id: &str,
    session_id: Option<&str>,
    opts: OwnershipOpts<'_>,
) -> W<AttachOutcome> {
    let workspace_id = require_workspace_id(id)?;
    let session = require_session("attachWorkspace", session_id)?;
    probe_readable(root, &workspace_id)?;

    let mut guard = acquire_workspace_lock(root, &workspace_id)?;
    let out = (|| -> W<AttachOutcome> {
        let record = read_for_ownership(root, &workspace_id, "attachWorkspace")?;
        match decide_ownership(&record, &session, opts.now, opts.is_owner_live) {
            Outcome::AlreadyOwner => Ok(AttachOutcome {
                role: AttachRole::Owner { reclaimed: false },
                record,
            }),
            Outcome::BecomeOwner | Outcome::Reclaim => {
                let reclaimed = matches!(
                    decide_ownership(&record, &session, opts.now, opts.is_owner_live),
                    Outcome::Reclaim
                );
                let next = apply_ownership_takeover(&record, &session, opts.now_iso);
                write_workspace_file_atomic(root, &workspace_id, &next)?;
                Ok(AttachOutcome { role: AttachRole::Owner { reclaimed }, record: next })
            }
            Outcome::Blocked(owner) => {
                // A read-only attach rather than a refusal.
                let existing: Vec<Value> = match record.get("attached_sessions") {
                    Some(Value::Array(a)) => a.clone(),
                    _ => vec![],
                };
                let already = existing
                    .iter()
                    .any(|s| matches!(s, Value::String(x) if *x == session));
                let attached = if already {
                    existing
                } else {
                    let mut v = existing;
                    v.push(json!(session));
                    v
                };
                let mut next = record.clone();
                next.insert("attached_sessions".into(), Value::Array(attached));
                write_workspace_file_atomic(root, &workspace_id, &next)?;
                Ok(AttachOutcome {
                    role: AttachRole::ReadOnly { write_owner_session: owner },
                    record: next,
                })
            }
        }
    })();
    guard.release();
    out
}

/// releaseWriteOwnership — same-session-only release; a non-owner calling
/// this is a no-op (`released: false`), never an error.
pub(crate) fn release_write_ownership(
    root: &Path,
    id: &str,
    session_id: Option<&str>,
) -> W<(bool, Map<String, Value>)> {
    let workspace_id = require_workspace_id(id)?;
    let session = require_session("releaseWriteOwnership", session_id)?;
    probe_readable(root, &workspace_id)?;

    let mut guard = acquire_workspace_lock(root, &workspace_id)?;
    let out = (|| -> W<(bool, Map<String, Value>)> {
        let record = read_workspace_record(root, &workspace_id)?;
        let is_owner = matches!(record.get("write_owner_session"), Some(Value::String(s)) if *s == session);
        if !is_owner {
            return Ok((false, record));
        }
        let next_epoch = match record.get("fence_epoch") {
            Some(Value::Number(n)) => n.as_f64().filter(|f| f.is_finite()).unwrap_or(0.0) + 1.0,
            _ => 1.0,
        };
        let mut next = record.clone();
        next.insert("write_owner_session".into(), Value::Null);
        next.insert("fence_epoch".into(), js_num(next_epoch));
        write_workspace_file_atomic(root, &workspace_id, &next)?;
        Ok((true, next))
    })();
    guard.release();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: &str = "2026-08-01T00:00:00.000Z";

    fn spec<'a>(id: &'a str, root: &'a str) -> RegisterSpec<'a> {
        RegisterSpec { id, kind: "main", root, branch: None, base_sha: None }
    }

    fn read_bytes(root: &Path, id: &str) -> String {
        std::fs::read_to_string(workspace_path(root, id).unwrap()).unwrap()
    }

    /// The id validator's two refusals, byte-for-byte.
    #[test]
    fn require_workspace_id_matches_node() {
        assert_eq!(require_workspace_id("  main  ").unwrap(), "main");
        for bad in ["", "   "] {
            let e = require_workspace_id(bad).unwrap_err();
            assert_eq!(e.code(), Some("WORKSPACE_INVALID_ID"));
            assert_eq!(e.message(), "workspace id is required.");
        }
        for bad in ["a/b", "a\\b", "..", "x..y"] {
            let e = require_workspace_id(bad).unwrap_err();
            assert_eq!(e.code(), Some("WORKSPACE_INVALID_ID"));
            assert_eq!(
                e.message(),
                format!(
                    "workspace id \"{}\" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/workspaces/.",
                    bad.trim()
                )
            );
        }
    }

    #[test]
    fn require_type_matches_node() {
        assert_eq!(require_type("worktree").unwrap(), "worktree");
        let e = require_type("bogus").unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_INVALID_TYPE"));
        assert_eq!(
            e.message(),
            "workspace type must be one of main/worktree (got \"bogus\")."
        );
    }

    /// registerWorkspace's fresh-record BYTES and key order, plus its
    /// idempotence (a second register never overwrites, never throws).
    #[test]
    fn register_is_idempotent_and_writes_nodes_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let record = register_workspace(root, spec("main", "D:\\repo"), T0).unwrap();
        assert_eq!(
            record.keys().collect::<Vec<_>>(),
            vec![
                "id", "type", "root", "branch", "base_sha", "write_owner_session",
                "fence_epoch", "attached_sessions", "created_at"
            ]
        );
        let text = read_bytes(root, "main");
        assert_eq!(
            text,
            "{\n  \"id\": \"main\",\n  \"type\": \"main\",\n  \"root\": \"D:\\\\repo\",\n  \"branch\": null,\n  \"base_sha\": null,\n  \"write_owner_session\": null,\n  \"fence_epoch\": 0,\n  \"attached_sessions\": [],\n  \"created_at\": \"2026-08-01T00:00:00.000Z\"\n}\n"
        );
        // Second register for the same id: the EXISTING record, untouched.
        let again = register_workspace(
            root,
            RegisterSpec {
                id: "main",
                kind: "worktree",
                root: "D:\\elsewhere",
                branch: Some("wt/x"),
                base_sha: Some("deadbeef"),
            },
            "2026-09-09T09:09:09.000Z",
        )
        .unwrap();
        assert_eq!(again.get("root"), Some(&json!("D:\\repo")));
        assert_eq!(again.get("type"), Some(&json!("main")));
        assert_eq!(read_bytes(root, "main"), text, "an existing record is never rewritten");
        // ...and it comes back through readWorkspaceRecord's spread, so the
        // five default keys lead.
        assert_eq!(
            again.keys().take(5).collect::<Vec<_>>(),
            vec!["write_owner_session", "fence_epoch", "attached_sessions", "branch", "base_sha"]
        );
    }

    /// The load-bearing one (module header): an ownership change REORDERS the
    /// record on disk, exactly as Node's spread does, and appends
    /// `owner_claimed_at` last.
    #[test]
    fn takeover_rewrites_in_nodes_spread_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let out = claim_write_ownership(
            root,
            "main",
            Some("sess-a"),
            OwnershipOpts { now: 0.0, now_iso: "2026-08-02T00:00:00.000Z", is_owner_live: None },
        )
        .unwrap();
        assert!(!out.reclaimed);
        assert_eq!(
            read_bytes(root, "main"),
            "{\n  \"write_owner_session\": \"sess-a\",\n  \"fence_epoch\": 1,\n  \"attached_sessions\": [],\n  \"branch\": null,\n  \"base_sha\": null,\n  \"id\": \"main\",\n  \"type\": \"main\",\n  \"root\": \"/repo\",\n  \"created_at\": \"2026-08-01T00:00:00.000Z\",\n  \"owner_claimed_at\": \"2026-08-02T00:00:00.000Z\"\n}\n"
        );
    }

    /// Idempotent re-claim by the SAME session: no epoch bump, no rewrite.
    #[test]
    fn same_session_reclaim_is_a_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let opts = || OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: None };
        claim_write_ownership(root, "main", Some("sess-a"), opts()).unwrap();
        let before = read_bytes(root, "main");
        let again = claim_write_ownership(root, "main", Some("sess-a"), opts()).unwrap();
        assert!(!again.reclaimed);
        assert_eq!(again.record.get("fence_epoch"), Some(&json!(1)));
        assert_eq!(read_bytes(root, "main"), before);
    }

    /// A DIFFERENT live owner: claim refuses WORKSPACE_OWNED naming the
    /// holder; attach records a read-only attach instead.
    #[test]
    fn live_owner_blocks_claim_and_downgrades_attach() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let live: &dyn Fn(&str, f64) -> bool = &|_, _| true;
        let opts = || OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: Some(live) };
        claim_write_ownership(root, "main", Some("sess-a"), opts()).unwrap();

        let e = claim_write_ownership(root, "main", Some("sess-b"), opts()).unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_OWNED"));
        assert_eq!(e.holder(), Some("sess-a"));
        assert_eq!(
            e.message(),
            "claimWriteOwnership: workspace \"main\" is already write-owned by session \"sess-a\"."
        );

        let attach = attach_workspace(root, "main", Some("sess-b"), opts()).unwrap();
        match attach.role {
            AttachRole::ReadOnly { write_owner_session } => assert_eq!(write_owner_session, "sess-a"),
            _ => panic!("a live owner must downgrade attach to read-only"),
        }
        assert_eq!(attach.record.get("attached_sessions"), Some(&json!(["sess-b"])));
        assert_eq!(attach.record.get("fence_epoch"), Some(&json!(1)), "an attach never bumps the epoch");
        // Idempotent: a second attach does not duplicate the session.
        let again = attach_workspace(root, "main", Some("sess-b"), opts()).unwrap();
        assert_eq!(again.record.get("attached_sessions"), Some(&json!(["sess-b"])));
    }

    /// A DEAD owner is reclaimable, the epoch bumps, and the reclaiming
    /// session is dropped from attached_sessions.
    #[test]
    fn dead_owner_is_reclaimed_and_bumps_the_epoch() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let live: &dyn Fn(&str, f64) -> bool = &|_, _| true;
        claim_write_ownership(
            root,
            "main",
            Some("sess-a"),
            OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: Some(live) },
        )
        .unwrap();
        attach_workspace(
            root,
            "main",
            Some("sess-b"),
            OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: Some(live) },
        )
        .unwrap();

        let dead: &dyn Fn(&str, f64) -> bool = &|_, _| false;
        let out = attach_workspace(
            root,
            "main",
            Some("sess-b"),
            OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: Some(dead) },
        )
        .unwrap();
        match out.role {
            AttachRole::Owner { reclaimed } => assert!(reclaimed, "a dead owner is a reclaim"),
            _ => panic!("a dead owner must be reclaimable"),
        }
        assert_eq!(out.record.get("fence_epoch"), Some(&json!(2)));
        assert_eq!(out.record.get("write_owner_session"), Some(&json!("sess-b")));
        assert_eq!(
            out.record.get("attached_sessions"),
            Some(&json!([])),
            "the new owner is removed from attached_sessions"
        );
    }

    /// The prohibition: neither ownership verb ever registers.
    #[test]
    fn ownership_refuses_an_unregistered_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let opts = || OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: None };
        let e = claim_write_ownership(root, "ghost", Some("s"), opts()).unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_NOT_REGISTERED"));
        assert_eq!(
            e.message(),
            "claimWriteOwnership: workspace \"ghost\" has never been registered — no unregistered workspace may gain write ownership. FIX: registerWorkspace first (worktree create/merge and the main-checkout first-touch path both call this automatically)."
        );
        let e = attach_workspace(root, "ghost", Some("s"), opts()).unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_NOT_REGISTERED"));
        assert_eq!(
            e.message(),
            "attachWorkspace: workspace \"ghost\" has never been registered — attach never auto-registers. FIX: registerWorkspace first."
        );
        assert!(!workspaces_dir(root).join("ghost.json").exists());
    }

    #[test]
    fn a_missing_session_id_is_a_typed_refusal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for (verb, e) in [
            (
                "claimWriteOwnership",
                claim_write_ownership(
                    root,
                    "main",
                    Some("   "),
                    OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: None },
                )
                .unwrap_err(),
            ),
            (
                "attachWorkspace",
                attach_workspace(
                    root,
                    "main",
                    None,
                    OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: None },
                )
                .unwrap_err(),
            ),
            (
                "releaseWriteOwnership",
                release_write_ownership(root, "main", None).unwrap_err(),
            ),
        ] {
            assert_eq!(e.code(), Some("WORKSPACE_INVALID_SESSION"));
            assert_eq!(e.message(), format!("{verb}: sessionId is required."));
        }
    }

    /// releaseWriteOwnership: same-session-only, epoch bumps, non-owner no-op.
    #[test]
    fn release_is_same_session_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        claim_write_ownership(
            root,
            "main",
            Some("sess-a"),
            OwnershipOpts { now: 0.0, now_iso: T0, is_owner_live: None },
        )
        .unwrap();
        let (released, _) = release_write_ownership(root, "main", Some("sess-b")).unwrap();
        assert!(!released, "a non-owner release is a no-op, never an error");

        let (released, record) = release_write_ownership(root, "main", Some("sess-a")).unwrap();
        assert!(released);
        assert_eq!(record.get("write_owner_session"), Some(&Value::Null));
        assert_eq!(record.get("fence_epoch"), Some(&json!(2)));
    }

    #[test]
    fn unregister_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("wt-1", "/repo/wt"), T0).unwrap();
        assert_eq!(
            unregister_workspace(root, "wt-1").unwrap(),
            json!({ "ok": true, "removed": true })
        );
        assert_eq!(
            unregister_workspace(root, "wt-1").unwrap(),
            json!({ "ok": true, "removed": false })
        );
    }

    /// readWorkspace's deterministic corrupt shapes stay native; the two
    /// V8-worded ones delegate.
    #[test]
    fn corrupt_records_split_native_from_delegated() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(workspaces_dir(root)).unwrap();
        let file = workspace_path(root, "main").unwrap();
        let shown = file.to_string_lossy().into_owned();

        // MISSING.
        let e = read_workspace(root, "main").unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_MISSING"));
        assert_eq!(
            e.message(),
            format!("readWorkspace: no workspace record at \"{shown}\". FIX: registerWorkspace first, or check the id.")
        );

        // Not an object — deterministic (`typeof`), so native.
        for (body, found) in [("[1]", "an array"), ("null", "object"), ("5", "number"), ("\"x\"", "string"), ("true", "boolean")] {
            std::fs::write(&file, body).unwrap();
            let e = read_workspace(root, "main").unwrap_err();
            assert_eq!(e.code(), Some("WORKSPACE_CORRUPT"));
            assert_eq!(
                e.message(),
                format!("readWorkspace: \"{shown}\" exists but is not a JSON object (found {found}).")
            );
        }

        // id mismatch — deterministic.
        std::fs::write(&file, "{\"id\":\"other\"}").unwrap();
        let e = read_workspace(root, "main").unwrap_err();
        assert_eq!(e.code(), Some("WORKSPACE_CORRUPT"));
        assert_eq!(
            e.message(),
            format!("readWorkspace: \"{shown}\" exists but its id field (\"other\") does not match the requested workspace \"main\" — never trusted.")
        );

        // Unparseable — the refusal embeds JSON.parse's V8 message.
        std::fs::write(&file, "{oops").unwrap();
        assert_eq!(read_workspace(root, "main").unwrap_err(), WsErr::Ex);
        // ...and the probe catches it BEFORE any lock is taken.
        assert_eq!(probe_readable(root, "main").unwrap_err(), WsErr::Ex);
        assert!(
            !lock::lock_file_path(root, "workspace:main").exists(),
            "no lock file is created on the delegated path"
        );
    }

    /// listWorkspaces: fail-open per entry, and Ex for an unmodelable one.
    #[test]
    fn list_reports_skips_and_delegates_on_a_v8_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(list_workspaces(root).unwrap(), (vec![], vec![]));
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let (workspaces, skipped) = list_workspaces(root).unwrap();
        assert_eq!(workspaces.len(), 1);
        assert!(skipped.is_empty());

        // A deterministic corrupt entry is skipped with its reason.
        std::fs::write(workspaces_dir(root).join("bad.json"), "{\"id\":\"nope\"}").unwrap();
        let (workspaces, skipped) = list_workspaces(root).unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0]["id"], json!("bad"));

        // A V8-worded one turns the whole call into a delegation.
        std::fs::write(workspaces_dir(root).join("worse.json"), "{oops").unwrap();
        assert_eq!(list_workspaces(root).unwrap_err(), WsErr::Ex);
    }

    /// The lock file this module contends on is Node's, by name — proven by
    /// holding `workspace:wt-1` externally and watching register(wt-1) block
    /// on it while register(main) sails through (the lock is PER WORKSPACE,
    /// never merely likely-distinct). The hold is released by the guard, so
    /// existence of the file is not the assertion — contention is.
    #[test]
    fn ownership_uses_nodes_lock_name_per_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(workspace_lock_name("wt-1").unwrap(), "workspace:wt-1");
        assert_ne!(
            lock::lock_file_path(root, "workspace:wt-1"),
            lock::lock_file_path(root, "workspace:main")
        );
        let mut held = lock::acquire_store_lock(root, "workspace:wt-1", 1)
            .unwrap_or_else(|b| panic!("precondition: the lock must be free — {}", b.message()));
        // A DIFFERENT workspace is untouched by that hold.
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        assert!(workspaces_dir(root).join("main.json").exists());
        // The SAME workspace contends and reports Node's LockBusyError.
        let e = release_write_ownership(root, "wt-1", Some("s")).unwrap_err();
        assert!(e.message().starts_with("lock \"workspace:wt-1\" busy:"), "{}", e.message());
        held.release();
        // Released: the same id now registers cleanly.
        register_workspace(root, spec("wt-1", "/repo/wt"), T0).unwrap();
        assert!(workspaces_dir(root).join("wt-1.json").exists());
    }

    /// A held `workspace:<id>` lock denies the mutators, with Node's
    /// LockBusyError message and no `code`.
    #[test]
    fn a_held_lock_surfaces_lock_busy_natively() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        register_workspace(root, spec("main", "/repo"), T0).unwrap();
        let mut held = lock::acquire_store_lock(root, "workspace:main", 1)
            .unwrap_or_else(|b| panic!("precondition: the lock must be free — {}", b.message()));
        let e = release_write_ownership(root, "main", Some("sess-a")).unwrap_err();
        assert_eq!(e.code(), None);
        assert!(
            e.message().starts_with("lock \"workspace:main\" busy: held by pid="),
            "unexpected: {}",
            e.message()
        );
        held.release();
    }
}
