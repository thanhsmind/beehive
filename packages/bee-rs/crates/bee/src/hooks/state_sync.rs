// bee hook state-sync — port of hooks/bee-state-sync.mjs: PostToolUse +
// SubagentStop + Stop. Refreshes cell status counts and last_activity into
// .bee/state.json (via the state-projection full rebuild) and renews the
// session heartbeat / claim TTLs / lease holds / cross-worktree holds, all
// with the try-once/never-wait lock posture. ALWAYS silent on stdout.
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).
//
// Strangler rule (contract C2): stdout is always empty, so the only
// observable divergence risk is stderr. Every branch where Node would emit a
// V8-specific message (fsutil.mjs readJson's corrupt-JSON warn; a corrupt
// workflow record's JSON.parse message inside listWorkflows' skip warn)
// delegates via a PRE-FLIGHT scan that runs BEFORE any write, so a delegated
// run never double-writes. Deterministic warns (listWorkflows' shape-error
// skips) are reproduced natively. Log-file lines match in shape/field-order;
// `error` text for crash logs approximates Node's `String(err.stack || err)`
// without the V8 stack frames (documented divergence, log-only).
//
// Ported lib functions (source file → private fn here):
//   claims.mjs heartbeatTouch                     → heartbeat_block (inlined)
//   claims.mjs readSession/sessionPath/requireId  → read_session / well_formed_id
//   claims.mjs heartbeatStale                     → heartbeat_stale
//   claims.mjs heartbeatSession (lockAttempts: 1) → heartbeat_session
//   claims.mjs renewClaimTTL/readClaim/acquireGate/releaseGate → renew_claim_ttl
//   claims.mjs withTransientFsRetry               → write_json_atomic_retry/remove_file_retry
//   reservations.mjs renewHoldsBySession/controlRootFor/findMainRoot/leaseTtlSeconds
//                                                 → renew_holds_by_session / find_main_root / lease_ttl_seconds
//   lease-store.mjs renewLease/renewLeaseFile/resolveResourceFile/canonicalizePath/
//                   hashString/lockNameForFile/computeExpiresAt/listAllLeaseFiles/readLeaseSafe
//                                                 → renew_lease_path / list_lease_files / read_lease_safe
//   worktree-holds.mjs renewHolds/readStore/writeStore/isActive/isExpired → renew_holds
//   cells.mjs listCells (status counting slice)   → count_cells
//   state-projection.mjs rebuildStateProjection/workflowGatesToApprovedGates/
//                        pickNewestActiveWorkflow → rebuild_state_projection etc.
//   workflow-store.mjs listWorkflows/readWorkflowRecord/mergeGates/baseWorkflowDefaults
//                                                 → list_workflows / read_workflow_record
//   state.mjs readState/defaultState/writeState/coerceLegacyPhase → read_state_failopen/default_state
//   state.mjs controlRootFor/resolveContext/resolveRootsCore → control_root_for/resolve_roots_core
//   state.mjs hookEnabled → crate::state::hook_enabled (already ported)
//   lock.mjs withStoreLock/acquireStoreLockOnceSync/LockBusyError → crate::lock (already ported)

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::adapter::{log_crash, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::lock::{acquire_store_lock, acquire_store_lock_once, AcquireOnce, LockBusy};
use crate::state::hook_enabled;
use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "state-sync";
const HEARTBEAT_TOUCH_THROTTLE_SECONDS: f64 = 60.0; // claims.mjs D5 throttle
const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

/// The native path cannot reproduce Node's observable bytes for this input —
/// re-run the .mjs wrapper with the same stdin (decided BEFORE any write).
struct Delegate;

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Outcome::Done(ExitCode::SUCCESS);
    };
    // Vendored-lib presence gate, byte-parity with the .mjs's existsSync check.
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return Outcome::Done(ExitCode::SUCCESS);
    }
    match run_gated(&ctx, &root) {
        Ok(()) => Outcome::Done(ExitCode::SUCCESS),
        Err(Delegate) => Outcome::Delegate,
    }
}

fn run_gated(ctx: &HookContext, root: &Path) -> Result<(), Delegate> {
    match hook_enabled(root, HOOK_NAME) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        // Corrupt config: Node's readJson warns to stderr with the V8 message.
        Err(_) => return Err(Delegate),
    }

    let plan = preflight(ctx, root)?;

    // D5 — throttled heartbeat + claim/hold lease renewal, own try/catch so a
    // throw here never blocks the state counts/last_activity refresh below.
    if let Some(sid) = &plan.sid {
        if let Err(msg) = heartbeat_block(ctx, root, sid) {
            log_crash(Some(root), HOOK_NAME, &msg, ctx.source);
        }
    }

    // `await import(libModuleUrl(root, "cells.mjs"))` — a missing vendored
    // module throws into the outer catch: logCrash, exit 0, nothing further.
    if !lib_exists(root, "cells.mjs") {
        log_crash(Some(root), HOOK_NAME, &module_not_found(root, "cells.mjs"), ctx.source);
        return Ok(());
    }
    let counts = count_cells(root);

    for module in ["lock.mjs", "state-projection.mjs"] {
        if !lib_exists(root, module) {
            log_crash(Some(root), HOOK_NAME, &module_not_found(root, module), ctx.source);
            return Ok(());
        }
    }

    // D3-amended: the state read-modify-write runs under the 'state' store
    // lock, try-once — LOCK_BUSY skips the sync silently.
    match acquire_store_lock(root, "state", 1) {
        Err(_busy) => {} // `if (!(error instanceof LockBusyError)) throw` — busy is swallowed
        Ok(mut guard) => {
            let last_activity = iso_ms(&Utc::now());
            let result = rebuild_state_projection(root, &plan.ctrl, &counts, &last_activity);
            guard.release();
            if let Err(msg) = result {
                // Non-LockBusy errors rethrow into the outer catch: logCrash, exit 0.
                log_crash(Some(root), HOOK_NAME, &msg, ctx.source);
                return Ok(());
            }
        }
    }
    Ok(())
}

// ── pre-flight: native/delegate decision before any write ──────────────────

struct Plan {
    /// controlRootFor(root) — Ok(control root) or the WorktreeLinkInvalidError
    /// message the .mjs would throw (and crash-log) at projection time.
    ctrl: Result<PathBuf, String>,
    sid: Option<String>,
}

fn preflight(ctx: &HookContext, root: &Path) -> Result<Plan, Delegate> {
    // rebuildStateProjection's readState: corrupt state.json → Node warns with
    // the V8 message; a truthy non-object approved_gates spreads exotic keys.
    read_state_strict_for_delegate(root)?;
    // listCells (counts): every corrupt cell .json would warn.
    scan_dir_for_corrupt_json(&root.join(".bee").join("cells"))?;

    let ctrl = match control_root_for(root) {
        Ok(p) => Ok(p),
        Err(CtrlErr::Delegate) => return Err(Delegate),
        Err(CtrlErr::Crash(msg)) => Err(msg),
    };
    if let Ok(ctrl_root) = &ctrl {
        scan_workflows_for_corrupt(ctrl_root)?;
    }

    let sid = get_session_id(&ctx.payload);
    if let Some(sid) = &sid {
        if well_formed_id(sid) {
            let ctrl_hb = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
            // heartbeatTouch's readSession: corrupt session record would warn.
            let record = read_session(&ctrl_hb, sid)?;
            let now_ms = Utc::now().timestamp_millis() as f64;
            if heartbeat_stale(record.as_ref(), now_ms, HEARTBEAT_TOUCH_THROTTLE_SECONDS) {
                // renewClaimTTL previews EVERY claims/*.json via readClaim.
                scan_dir_for_corrupt_json(&ctrl_hb.join(".bee").join("claims"))?;
                // worktree-holds readStore reads the MAIN checkout's ledger.
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                if let Ok(core) = resolve_roots_core(&cwd) {
                    let main_root = match core {
                        RootsCore::LinkedValid { main_root, .. } => main_root,
                        _ => root.to_path_buf(),
                    };
                    let ledger = main_root
                        .join(".bee")
                        .join("runtime")
                        .join("cross-worktree-holds.json");
                    if matches!(read_json(&ledger), ReadJson::Corrupt) {
                        return Err(Delegate);
                    }
                }
            }
        }
    }
    Ok(Plan { ctrl, sid })
}

/// readState's delegate conditions only (result discarded; the projection
/// re-reads inside the lock like the .mjs does).
fn read_state_strict_for_delegate(root: &Path) -> Result<(), Delegate> {
    match read_json(&root.join(".bee").join("state.json")) {
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(Value::Object(m)) => match m.get("approved_gates") {
            // Truthy non-object spreads exotic keys (string chars / array
            // indices) in JS — those states delegate; everything else is safe.
            Some(Value::String(s)) if !s.is_empty() => Err(Delegate),
            Some(Value::Array(a)) if !a.is_empty() => Err(Delegate),
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn scan_dir_for_corrupt_json(dir: &Path) -> Result<(), Delegate> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Ok(()) };
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        if matches!(read_json(&entry.path()), ReadJson::Corrupt) {
            return Err(Delegate);
        }
    }
    Ok(())
}

/// workflow-store.mjs readWorkflowRecord parses RAW file content (no BOM
/// strip); a JSON-corrupt record's listWorkflows skip-warn embeds the V8
/// parse message → delegate. Shape errors warn deterministically → native.
fn scan_workflows_for_corrupt(ctrl_root: &Path) -> Result<(), Delegate> {
    let dir = ctrl_root.join(".bee").join("runtime").join("workflows");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(()) };
    for entry in rd.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let file = entry.path().join("state.json");
        match std::fs::read(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Delegate), // read-error warn embeds an errno string
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                if serde_json::from_str::<Value>(&text).is_err() {
                    return Err(Delegate);
                }
            }
        }
    }
    Ok(())
}

// ── payload / vendored-module helpers ──────────────────────────────────────

fn get_session_id(payload: &Map<String, Value>) -> Option<String> {
    match payload.get("session_id") {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

fn lib_exists(root: &Path, name: &str) -> bool {
    root.join(".bee").join("bin").join("lib").join(name).exists()
}

/// Log-shape approximation of Node's ERR_MODULE_NOT_FOUND for a missing
/// vendored module (crash-log text only; stdout/stderr are unaffected).
fn module_not_found(root: &Path, name: &str) -> String {
    format!(
        "Error [ERR_MODULE_NOT_FOUND]: Cannot find module '{}'",
        root.join(".bee").join("bin").join("lib").join(name).display()
    )
}

// ── JS semantics helpers ────────────────────────────────────────────────────

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn js_strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false, // two separately-parsed objects are never === in JS
    }
}

/// JS Date.parse for the shapes bee writes (RFC3339, or date-only UTC).
fn date_parse_ms(v: &Value) -> Option<f64> {
    let Value::String(s) = v else { return None };
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis() as f64);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0)?;
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    None
}

/// new Date(ms).toISOString() — millisecond-precision ISO, always 3 digits.
fn iso_ms(t: &DateTime<Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn well_formed_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// String(value) for a possibly-absent JSON value (undefined → "undefined").
fn js_string_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => crate::jsjson::js_to_string(v),
    }
}

/// Node path.relative for the shape this hook needs (file under root).
fn path_relative(root: &Path, file: &Path) -> String {
    match file.strip_prefix(root) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR),
        Err(_) => file.display().to_string(),
    }
}

// ── transient-FS retry (claims.mjs rel1710rc-5 / lock.mjs rel180-4 class) ──

#[cfg(windows)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(4 | 5 | 32 | 33 | 145))
}

#[cfg(unix)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(code) if {
        code == libc::EBUSY || code == libc::EPERM || code == libc::ENOTEMPTY
            || code == libc::EMFILE || code == libc::ENFILE
    })
}

fn with_transient_fs_retry<T>(mut f: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut attempt = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if !is_transient_fs_error(&e) || attempt >= 15 {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
}

fn write_json_atomic_retry(file: &Path, value: &Value) -> Result<(), String> {
    with_transient_fs_retry(|| write_json_atomic(file, value)).map_err(|e| format!("Error: {e}"))
}

fn remove_file_retry(file: &Path) -> Result<(), String> {
    with_transient_fs_retry(|| match std::fs::remove_file(file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // rmSync force
        other => other,
    })
    .map_err(|e| format!("Error: {e}"))
}

// ── state.mjs readState / defaultState ─────────────────────────────────────

fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for name in GATE_NAMES {
        m.insert(name.into(), Value::Bool(false));
    }
    m
}

fn default_state() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert(
        "next_action".into(),
        json!("No active bee work \u{2014} awaiting a user request."),
    );
    m
}

/// state.mjs readState, fail-open flavor for the execution phase: corrupt and
/// exotic-gates inputs were already delegated in pre-flight, so here they
/// collapse to defaults (a race-window-only divergence, never the plan).
fn read_state_failopen(root: &Path) -> Map<String, Value> {
    let file_state = match read_json(&root.join(".bee").join("state.json")) {
        ReadJson::Parsed(Value::Object(m)) => m,
        _ => return default_state(),
    };
    let mut merged = default_state();
    for (k, v) in &file_state {
        merged.insert(k.clone(), v.clone());
    }
    let gates = match file_state.get("approved_gates") {
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        _ => default_gates(),
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning")); // D13 legacy coercion
    }
    merged
}

// ── state.mjs resolveRootsCore / controlRootFor ────────────────────────────

enum RootsCore {
    None,
    Ordinary { work_root: PathBuf },
    LinkedValid { work_root: PathBuf, main_root: PathBuf },
}

fn js_absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

/// state.mjs readGitdirFile: trim, strip "gitdir:", backslash-normalize,
/// path.resolve against `base`. (Byte-identical logic backs reservations.mjs's
/// readGitdirFileForRoot, reused by find_main_root below.)
fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(file).ok()?;
    let mut raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = rest.trim();
    }
    let sep_fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
    Some(js_absolute(&base.join(sep_fixed)))
}

/// state.mjs resolveRootsCore. Err(message) mirrors the thrown
/// WorktreeLinkInvalidError's `${reason} (${marker})`.
fn resolve_roots_core(start: &Path) -> Result<RootsCore, String> {
    let mut nearest = js_absolute(start);
    loop {
        if nearest.join(".bee").join("onboarding.json").exists() && !nearest.join(".git").exists() {
            return Ok(RootsCore::Ordinary { work_root: nearest });
        }
        match nearest.parent() {
            Some(p) => nearest = p.to_path_buf(),
            None => break,
        }
    }
    let mut dir = js_absolute(start);
    let located = loop {
        if dir.join(".git").exists() {
            break Some(dir.clone());
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break None,
        }
    };
    let Some(work_root) = located else {
        let mut dir = js_absolute(start);
        loop {
            if dir.join(".bee").join("onboarding.json").exists() {
                return Ok(RootsCore::Ordinary { work_root: dir });
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return Ok(RootsCore::None),
            }
        }
    };
    let marker = work_root.join(".git");
    let marker_str = marker.display().to_string();
    let stat =
        std::fs::metadata(&marker).map_err(|e| format!("Error: {e}, stat '{marker_str}'"))?;
    if !stat.is_file() {
        return Ok(RootsCore::Ordinary { work_root });
    }
    let invalid = |reason: &str| Err(format!("{reason} ({marker_str})"));
    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else {
        return invalid("linked worktree gitdir is missing or malformed");
    };
    let worktrees_root = gitdir.parent().map(Path::to_path_buf).unwrap_or_default();
    let common_git_dir = worktrees_root.parent().map(Path::to_path_buf).unwrap_or_default();
    if !(common_git_dir.file_name().is_some_and(|n| n == ".git")
        && worktrees_root.file_name().is_some_and(|n| n == "worktrees"))
    {
        return invalid("linked worktree gitdir is outside the expected .git/worktrees namespace");
    }
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if id.is_empty() || id == "." || id == ".." {
        return invalid("linked worktree id is empty");
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    if reverse.as_deref() != Some(js_absolute(&marker).as_path()) {
        return invalid("linked worktree reverse gitdir pointer is missing or mismatched");
    }
    let Some(main_root) = common_git_dir.parent().map(Path::to_path_buf) else {
        return invalid("linked worktree gitdir is outside the expected .git/worktrees namespace");
    };
    Ok(RootsCore::LinkedValid { work_root, main_root })
}

enum CtrlErr {
    Delegate,
    Crash(String),
}

/// state.mjs resolveProductRoot's warn conditions, as a delegate gate (see
/// chain_nudge.rs's twin for the reasoning).
fn check_product_root(work_root: &Path) -> Result<(), CtrlErr> {
    let config = crate::state::read_config_raw(work_root).map_err(|_| CtrlErr::Delegate)?;
    match config.get("product_root") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(s)) if s.is_empty() => Ok(()),
        Some(Value::String(s)) => {
            let resolved = work_root.join(s);
            match std::fs::metadata(&resolved) {
                Ok(m) if m.is_dir() => Ok(()),
                _ => Err(CtrlErr::Delegate),
            }
        }
        Some(_) => Err(CtrlErr::Delegate),
    }
}

/// state.mjs controlRootFor(root) = resolveContext(root).controlRoot ?? root.
fn control_root_for(root: &Path) -> Result<PathBuf, CtrlErr> {
    match resolve_roots_core(root) {
        Err(msg) => Err(CtrlErr::Crash(format!("WorktreeLinkInvalidError: {msg}"))),
        Ok(RootsCore::None) => Ok(root.to_path_buf()),
        Ok(RootsCore::Ordinary { work_root }) => {
            check_product_root(&work_root)?;
            Ok(work_root)
        }
        Ok(RootsCore::LinkedValid { work_root, main_root }) => {
            check_product_root(&work_root)?;
            Ok(main_root)
        }
    }
}

// ── claims.mjs: sessions, heartbeats, claim TTL renewal ────────────────────

/// claims.mjs readSession (fail-open display read). Delegate on a corrupt
/// record — Node's readJson warns with the V8 message.
fn read_session(
    control_root: &Path,
    session_id: &str,
) -> Result<Option<Map<String, Value>>, Delegate> {
    if !well_formed_id(session_id) {
        return Ok(None); // sessionPath's requireId throw reads as "no session"
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{session_id}.json"));
    let session = match read_json(&file) {
        ReadJson::Missing => return Ok(None),
        ReadJson::Corrupt => return Err(Delegate),
        ReadJson::Parsed(Value::Object(m)) => m,
        ReadJson::Parsed(_) => return Ok(None),
    };
    if session.get("id") != Some(&Value::String(session_id.to_string())) {
        return Ok(None);
    }
    Ok(Some(session))
}

fn read_session_failopen(control_root: &Path, session_id: &str) -> Option<Map<String, Value>> {
    read_session(control_root, session_id).unwrap_or(None) // corrupt was pre-delegated; race-only
}

/// claims.mjs heartbeatStale.
fn heartbeat_stale(record: Option<&Map<String, Value>>, now_ms: f64, stale_seconds: f64) -> bool {
    let Some(rec) = record else { return true };
    let Some(beat) = rec.get("last_heartbeat").and_then(date_parse_ms) else { return true };
    beat + stale_seconds * 1000.0 <= now_ms
}

/// The heartbeat/lease-renewal block of bee-state-sync.mjs main(), i.e.
/// claims.mjs heartbeatTouch composed with reservations.renewHoldsBySession
/// and worktree-holds.renewHolds. Err(text) is the crash the .mjs's own catch
/// would logCrash (silently — never stdout/stderr).
fn heartbeat_block(ctx: &HookContext, root: &Path, sid: &str) -> Result<(), String> {
    let ctrl = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
    if !lib_exists(root, "claims.mjs") {
        return Err(module_not_found(root, "claims.mjs"));
    }
    let now = Utc::now();
    let now_ms = now.timestamp_millis() as f64;

    // heartbeatTouch: read → throttle → heartbeatSession + renewClaimTTL.
    let record = read_session_failopen(&ctrl, sid);
    if !heartbeat_stale(record.as_ref(), now_ms, HEARTBEAT_TOUCH_THROTTLE_SECONDS) {
        return Ok(()); // { touched: false, reason: 'throttled' } — nothing else runs
    }
    heartbeat_session(&ctrl, sid, &now)?;
    renew_claim_ttl(&ctrl, sid, &now)?;
    // touched: true → renew this session's path-lease reservations…
    if !lib_exists(root, "reservations.mjs") {
        return Err(module_not_found(root, "reservations.mjs"));
    }
    renew_holds_by_session(root, sid)?;
    // …and (hardening-1-7-10 D3, own try/catch) its cross-worktree holds.
    if let Err(msg) = renew_cross_worktree_holds(root, sid) {
        log_crash(Some(root), HOOK_NAME, &msg, ctx.source);
    }
    Ok(())
}

/// claims.mjs heartbeatSession with lockAttempts: 1. LOCK_BUSY and
/// SESSION_MISSING are typed results (not throws) — both read as Ok here.
fn heartbeat_session(ctrl: &Path, sid: &str, now: &DateTime<Utc>) -> Result<(), String> {
    match acquire_store_lock_once(ctrl, "sessions") {
        AcquireOnce::Busy { .. } => Ok(()),
        AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> Result<(), String> {
                let Some(mut session) = read_session_failopen(ctrl, sid) else {
                    return Ok(()); // SESSION_MISSING typed result
                };
                session.insert("last_heartbeat".to_string(), Value::String(iso_ms(now)));
                let file = ctrl.join(".bee").join("sessions").join(format!("{sid}.json"));
                write_json_atomic_retry(&file, &Value::Object(session))
            })();
            guard.release();
            result
        }
    }
}

/// claims.mjs renewClaimTTL (no fencing — the hook never presents an epoch).
fn renew_claim_ttl(ctrl: &Path, sid: &str, now: &DateTime<Utc>) -> Result<(), String> {
    // requireId(sessionId, 'session id') — malformed ids throw.
    if !well_formed_id(sid) {
        return Err("Error: session id must be a plain id (no path separators).".to_string());
    }
    let dir = ctrl.join(".bee").join("claims");
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Ok(()); // readdir failure: { ok: true, renewed: [] }
    };
    let names: Vec<String> =
        rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    let sid_value = Value::String(sid.to_string());
    for name in names {
        let Some(cell) = name.strip_suffix(".json") else { continue };
        let claim_file = dir.join(&name);
        let owned = matches!(
            read_claim(&claim_file),
            Some(claim) if claim.get("session") == Some(&sid_value)
        );
        if !owned {
            continue; // not ours (or sessionless): never touched
        }
        // acquireGate: exclusive-create claims/<cell>.adopting.
        let gate = dir.join(format!("{cell}.adopting"));
        let gate_body = format!(
            "{}\n",
            crate::jsjson::stringify(&json!({ "pid": std::process::id(), "at": iso_ms(now) }))
        );
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&gate) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue, // skipped
            Err(e) => return Err(format!("Error: {e}")),
            Ok(mut f) => {
                use std::io::Write;
                if let Err(e) = f.write_all(gate_body.as_bytes()) {
                    let _ = remove_file_retry(&gate);
                    return Err(format!("Error: {e}"));
                }
            }
        }
        let result = (|| -> Result<(), String> {
            // Re-verify ownership under the gate.
            if let Some(mut claim) = read_claim(&claim_file) {
                if claim.get("session") == Some(&sid_value) {
                    // `{ ...claim, claimed_at }` — only the expiry clock advances.
                    claim.insert("claimed_at".to_string(), Value::String(iso_ms(now)));
                    write_json_atomic_retry(&claim_file, &Value::Object(claim))?;
                }
            }
            Ok(())
        })();
        let released = remove_file_retry(&gate); // finally: releaseGate
        released?;
        result?;
    }
    Ok(())
}

/// claims.mjs readClaim: fail-open readJson; non-objects read as "no claim".
fn read_claim(file: &Path) -> Option<Map<String, Value>> {
    match read_json(file) {
        ReadJson::Parsed(Value::Object(m)) => Some(m),
        _ => None, // corrupt claims were pre-delegated; race-only here
    }
}

// ── reservations.mjs + lease-store.mjs: path-lease renewal ─────────────────

/// reservations.mjs findMainRoot (fail-open, never throws) — the cycle-safe
/// controlRootFor replica reservations.mjs carries.
fn find_main_root(root: &Path) -> Option<PathBuf> {
    let mut dir = js_absolute(root);
    let work_root = loop {
        if dir.join(".git").exists() {
            break dir;
        }
        dir = dir.parent()?.to_path_buf();
    };
    let marker = work_root.join(".git");
    let stat = std::fs::metadata(&marker).ok()?;
    if !stat.is_file() {
        return Some(work_root); // ordinary checkout: mainRoot === workRoot
    }
    let gitdir = read_gitdir_file(&marker, &work_root)?;
    let worktrees_root = gitdir.parent()?.to_path_buf();
    let common_git_dir = worktrees_root.parent()?.to_path_buf();
    if !(common_git_dir.file_name().is_some_and(|n| n == ".git")
        && worktrees_root.file_name().is_some_and(|n| n == "worktrees"))
    {
        return None;
    }
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if id.is_empty() || id == "." || id == ".." {
        return None;
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    if reverse.as_deref() != Some(js_absolute(&marker).as_path()) {
        return None;
    }
    common_git_dir.parent().map(Path::to_path_buf)
}

/// lease-store.mjs listAllLeaseFiles: cells dir then paths dir, *.json files.
fn list_lease_files(root: &Path) -> Vec<PathBuf> {
    let leases = root.join(".bee").join("runtime").join("leases");
    let mut files = Vec::new();
    for dir in [leases.join("cells"), leases.join("paths")] {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) && name.ends_with(".json") {
                files.push(entry.path());
            }
        }
    }
    files
}

/// lease-store.mjs readLeaseSafe (raw parse, no BOM strip, never warns).
fn read_lease_safe(file: &Path) -> Option<Map<String, Value>> {
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Some(m),
        _ => None, // non-object leases fail isPathLease's `.resource` probe anyway
    }
}

/// reservations.mjs leaseTtlSeconds: renew by the lease's OWN window; the
/// 0 sentinel (and NaN math) mean "never expires".
fn lease_ttl_seconds(record: &Map<String, Value>) -> f64 {
    match record.get("expires_at") {
        None | Some(Value::Null) => 0.0,
        Some(exp) => {
            let e = date_parse_ms(exp);
            let a = record.get("acquired_at").and_then(date_parse_ms);
            match (e, a) {
                (Some(e), Some(a)) => (((e - a) / 1000.0).round()).max(0.0),
                _ => f64::NAN, // Date.parse NaN propagates; computeExpiresAt maps it to null
            }
        }
    }
}

/// lease-store.mjs computeExpiresAt.
fn compute_expires_at(ttl: f64, now: &DateTime<Utc>) -> Value {
    if ttl.is_finite() && ttl > 0.0 {
        let expires = chrono::DateTime::from_timestamp_millis(
            now.timestamp_millis() + (ttl as i64) * 1000,
        );
        match expires {
            Some(dt) => Value::String(iso_ms(&dt)),
            None => Value::Null,
        }
    } else {
        Value::Null
    }
}

/// reservations.mjs renewHoldsBySession(root, sessionId, {lockOptions:
/// {maxAttempts: 1}}) over lease-store.mjs renewLease. Err(text) mirrors the
/// throw that escapes into the hook's heartbeat catch (crash-logged).
fn renew_holds_by_session(root: &Path, sid: &str) -> Result<(), String> {
    let ctrl2 = find_main_root(root).unwrap_or_else(|| root.to_path_buf());
    let now = Utc::now();
    let sid_value = Value::String(sid.to_string());
    for file in list_lease_files(&ctrl2) {
        let Some(record) = read_lease_safe(&file) else { continue };
        // isPathLease: resource must be a string starting with "path:".
        let Some(Value::String(resource)) = record.get("resource").cloned() else {
            continue;
        };
        if !resource.starts_with("path:") {
            continue; // not a path lease
        }
        if record.get("session_id") != Some(&sid_value) {
            continue;
        }
        let ttl = lease_ttl_seconds(&record);
        renew_lease_path(&ctrl2, &resource["path:".len()..], ttl, &now)?;
    }
    Ok(())
}

/// lease-store.mjs canonicalizePath: backslash-normalize, strip one anchored
/// leading "./+", strip trailing slashes.
fn canonicalize_path(v: &str) -> String {
    let s = v.replace('\\', "/");
    let s = match s.strip_prefix('.') {
        Some(rest) if rest.starts_with('/') => rest.trim_start_matches('/').to_string(),
        _ => s,
    };
    s.trim_end_matches('/').to_string()
}

/// lease-store.mjs renewLease → renewLeaseFile for one path resource, under
/// the per-file `lease:<sha256(file)>` store lock, try-once. LEASE_MISSING is
/// skipped (Ok); LEASE_CORRUPT / LockBusy propagate as crash text.
fn renew_lease_path(ctrl2: &Path, id: &str, ttl: f64, now: &DateTime<Utc>) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("LeaseStoreError: lease request: path id is required.".to_string());
    }
    let canonical = canonicalize_path(id);
    let resource_key = format!("path:{canonical}");
    let file = ctrl2
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)));
    let lock_name = format!("lease:{}", sha256_hex(&file.display().to_string()));
    match acquire_store_lock_once(ctrl2, &lock_name) {
        AcquireOnce::Busy { holder } => {
            Err(format!("LockBusyError: {}", LockBusy { lock_name, holder }.message()))
        }
        AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> Result<(), String> {
                let text = match std::fs::read(&file) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()), // LEASE_MISSING — swept concurrently, skip
                    Err(e) => {
                        return Err(format!(
                            "LeaseStoreError: renewLease: could not read/parse lease record at \"{}\" ({e}).",
                            file.display()
                        ));
                    }
                    Ok(b) => String::from_utf8_lossy(&b).into_owned(),
                };
                let current: Value = serde_json::from_str(&text).map_err(|e| {
                    format!(
                        "LeaseStoreError: renewLease: could not read/parse lease record at \"{}\" ({e}).",
                        file.display()
                    )
                })?;
                let mut next: Map<String, Value> = match current {
                    Value::Object(m) => m,
                    _ => Map::new(), // `{...current}` of a primitive spreads to nothing
                };
                next.insert("expires_at".to_string(), compute_expires_at(ttl, now));
                write_json_atomic(&file, &Value::Object(next)).map_err(|e| format!("Error: {e}"))
            })();
            guard.release();
            result
        }
    }
}

// ── worktree-holds.mjs renewHolds (cross-worktree ledger) ──────────────────

/// The hook's own D3 inner try/catch body: main-root resolution via
/// state.mjs's THROWING resolveRoots(process.cwd()), then
/// worktree-holds.renewHolds(mainRoot, sessionId, {maxAttempts: 1}).
fn renew_cross_worktree_holds(root: &Path, sid: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let main_root = match resolve_roots_core(&cwd) {
        Ok(RootsCore::LinkedValid { main_root, .. }) => main_root,
        Ok(_) => root.to_path_buf(),
        Err(msg) => return Err(format!("WorktreeLinkInvalidError: {msg}")),
    };
    if !lib_exists(root, "worktree-holds.mjs") {
        return Err(module_not_found(root, "worktree-holds.mjs"));
    }
    let lock_name = "cross-worktree-holds";
    match acquire_store_lock_once(&main_root, lock_name) {
        AcquireOnce::Busy { holder } => Err(format!(
            "LockBusyError: {}",
            LockBusy { lock_name: lock_name.to_string(), holder }.message()
        )),
        AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> Result<(), String> {
                let ledger = main_root
                    .join(".bee")
                    .join("runtime")
                    .join("cross-worktree-holds.json");
                // readStore: missing/invalid shape reads as an empty ledger.
                let mut store = match read_json(&ledger) {
                    ReadJson::Parsed(Value::Object(m))
                        if matches!(m.get("holds"), Some(Value::Array(_))) =>
                    {
                        m
                    }
                    _ => {
                        let mut m = Map::new();
                        m.insert("holds".to_string(), json!([]));
                        m
                    }
                };
                let now_ms = Utc::now().timestamp_millis() as f64;
                let now_iso = iso_ms(&Utc::now());
                let sid_value = Value::String(sid.to_string());
                let mut renewed = 0usize;
                if let Some(Value::Array(holds)) = store.get_mut("holds") {
                    for hold in holds.iter_mut() {
                        let Value::Object(h) = hold else { continue };
                        // isActive: `released_at == null` (loose — absent counts) && !isExpired.
                        let released = !matches!(h.get("released_at"), None | Some(Value::Null));
                        if released || hold_expired(h, now_ms) {
                            continue;
                        }
                        if h.get("session") != Some(&sid_value) {
                            continue;
                        }
                        h.insert("mirrored_at".to_string(), Value::String(now_iso.clone()));
                        renewed += 1;
                    }
                }
                if renewed > 0 {
                    write_json_atomic(&ledger, &Value::Object(store))
                        .map_err(|e| format!("Error: {e}"))?;
                }
                Ok(())
            })();
            guard.release();
            result
        }
    }
}

/// worktree-holds.mjs isExpired.
fn hold_expired(hold: &Map<String, Value>, now_ms: f64) -> bool {
    let ttl = match hold.get("ttl_seconds").and_then(Value::as_f64) {
        Some(t) if t.is_finite() && t > 0.0 => t,
        _ => return false, // non-positive/invalid TTL never expires
    };
    let Some(mirrored) = hold.get("mirrored_at").and_then(date_parse_ms) else {
        return false;
    };
    mirrored + ttl * 1000.0 <= now_ms
}

// ── cells.mjs listCells → status counts ────────────────────────────────────

/// The hook's counts loop over cells.mjs listCells(root, {}): active-dir scan
/// only, non-objects dropped, `counts[cell.status] !== undefined` guard.
fn count_cells(root: &Path) -> Map<String, Value> {
    let mut open = 0u64;
    let mut claimed = 0u64;
    let mut capped = 0u64;
    let mut blocked = 0u64;
    if let Ok(rd) = std::fs::read_dir(root.join(".bee").join("cells")) {
        for entry in rd.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue; // the `archive` child (or any dir) is never a cell
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let ReadJson::Parsed(Value::Object(cell)) = read_json(&entry.path()) else {
                continue;
            };
            if let Some(Value::String(status)) = cell.get("status") {
                match status.as_str() {
                    "open" => open += 1,
                    "claimed" => claimed += 1,
                    "capped" => capped += 1,
                    "blocked" => blocked += 1,
                    _ => {}
                }
            }
        }
    }
    let mut counts = Map::new();
    counts.insert("open".to_string(), json!(open));
    counts.insert("claimed".to_string(), json!(claimed));
    counts.insert("capped".to_string(), json!(capped));
    counts.insert("blocked".to_string(), json!(blocked));
    counts
}

// ── workflow-store.mjs listWorkflows / readWorkflowRecord ──────────────────

fn base_workflow_defaults() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("mode".into(), Value::Null);
    m.insert("phase".into(), json!("idle"));
    m.insert("plan_rev".into(), json!(0));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!(""));
    m.insert("status".into(), json!("active"));
    m.insert("route".into(), Value::Null);
    m
}

fn default_gate_entry() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("approved".into(), Value::Bool(false));
    m.insert("approved_for_plan_rev".into(), Value::Null);
    m
}

/// workflow-store.mjs mergeGates(defaultGates(), parsed.gates).
fn merge_gates(overrides: Option<&Value>) -> Value {
    let mut merged = Map::new();
    for name in GATE_NAMES {
        merged.insert(name.to_string(), Value::Object(default_gate_entry()));
    }
    if let Some(Value::Object(over)) = overrides {
        for (name, value) in over {
            let mut entry = match merged.get(name) {
                Some(Value::Object(e)) => e.clone(),
                _ => default_gate_entry(),
            };
            if let Value::Object(v) = value {
                for (k, val) in v {
                    entry.insert(k.clone(), val.clone());
                }
            }
            merged.insert(name.clone(), Value::Object(entry));
        }
    }
    Value::Object(merged)
}

/// typeof-style label for the not-an-object refusal message.
fn found_kind(v: &Value) -> &'static str {
    match v {
        Value::Array(_) => "an array",
        Value::Null | Value::Object(_) => "object", // typeof null === 'object'
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
    }
}

/// workflow-store.mjs readWorkflowRecord. Err(reason) carries the exact
/// WorkflowStoreError message listWorkflows quotes in its skip warn (the
/// JSON-corrupt/read-error variants were delegated in pre-flight).
fn read_workflow_record(ctrl_root: &Path, id: &str) -> Result<Map<String, Value>, String> {
    let file = ctrl_root
        .join(".bee")
        .join("runtime")
        .join("workflows")
        .join(id)
        .join("state.json");
    let file_str = file.display().to_string();
    let rel = path_relative(ctrl_root, &file);
    let bytes = match std::fs::read(&file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "readWorkflow: no workflow record at \"{file_str}\". FIX: createWorkflow first, or check the id."
            ));
        }
        Err(e) => {
            // Pre-flight delegates this class; race-window-only approximation.
            return Err(format!(
                "readWorkflow: could not read \"{file_str}\" ({e}). The bee CLI refuses to guess at a workflow record it cannot read \u{2014} that could silently clobber real state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            ));
        }
        Ok(b) => b,
    };
    let text = String::from_utf8_lossy(&bytes);
    let parsed: Value = serde_json::from_str(&text).map_err(|e| {
        // Pre-flight delegates this class; race-window-only approximation.
        format!(
            "readWorkflow: \"{file_str}\" exists but is not valid JSON ({e}). The bee CLI refuses to rebuild a workflow from defaults over a present-but-corrupt file \u{2014} that would silently clobber real state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        )
    })?;
    let Value::Object(parsed_map) = parsed else {
        return Err(format!(
            "readWorkflow: \"{file_str}\" exists but is not a JSON object (found {}).",
            found_kind(&parsed)
        ));
    };
    if parsed_map.get("id") != Some(&Value::String(id.to_string())) {
        return Err(format!(
            "readWorkflow: \"{file_str}\" exists but its id field (\"{}\") does not match the requested workflow \"{id}\" \u{2014} never trusted. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
            js_string_or_undefined(parsed_map.get("id"))
        ));
    }
    // { ...baseWorkflowDefaults(), ...parsed, id, gates: mergeGates(...) }
    let mut merged = base_workflow_defaults();
    for (k, v) in &parsed_map {
        merged.insert(k.clone(), v.clone());
    }
    merged.insert("id".to_string(), Value::String(id.to_string()));
    merged.insert("gates".to_string(), merge_gates(parsed_map.get("gates")));
    Ok(merged)
}

/// workflow-store.mjs listWorkflows: fail-open enumeration; unreadable
/// entries are skipped with the byte-identical console.warn line.
fn list_workflows(ctrl_root: &Path) -> Vec<Map<String, Value>> {
    let dir = ctrl_root.join(".bee").join("runtime").join("workflows");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        match read_workflow_record(ctrl_root, &id) {
            Ok(record) => out.push(record),
            Err(reason) => {
                eprintln!("listWorkflows: skipping unreadable workflow \"{id}\" \u{2014} {reason}");
            }
        }
    }
    out
}

// ── state-projection.mjs rebuildStateProjection ────────────────────────────

/// state-projection.mjs workflowGatesToApprovedGates(gates, planRev).
fn workflow_gates_to_approved(gates: &Value, plan_rev: &Value) -> Value {
    let mut approved = Map::new();
    for name in GATE_NAMES {
        let entry = match gates {
            Value::Object(m) => m.get(name),
            _ => None,
        };
        let entry_truthy = entry.map(truthy).unwrap_or(false);
        let is_approved = entry_truthy
            && matches!(entry, Some(Value::Object(e)) if e.get("approved") == Some(&Value::Bool(true)));
        let rev = if entry_truthy {
            match entry {
                Some(Value::Object(e)) => e.get("approved_for_plan_rev"),
                _ => None, // property access on a truthy primitive yields undefined
            }
        } else {
            None // entry falsy → rev is undefined
        };
        let rev_effective = match rev {
            None | Some(Value::Null) => true,
            Some(v) => js_strict_eq(v, plan_rev),
        };
        approved.insert(name.to_string(), Value::Bool(is_approved && rev_effective));
    }
    Value::Object(approved)
}

/// state-projection.mjs pickNewestActiveWorkflow.
fn pick_newest_active(workflows: &[Map<String, Value>]) -> Option<&Map<String, Value>> {
    let mut active: Vec<&Map<String, Value>> = workflows
        .iter()
        .filter(|wf| {
            wf.get("status") == Some(&json!("active"))
                && !js_strict_eq(
                    wf.get("phase").unwrap_or(&Value::Null),
                    &json!("compounding-complete"),
                )
        })
        .collect();
    active.sort_by(|a, b| {
        let ta = a.get("created_at").and_then(date_parse_ms).unwrap_or(0.0);
        let tb = b.get("created_at").and_then(date_parse_ms).unwrap_or(0.0);
        if ta != tb {
            return tb.partial_cmp(&ta).unwrap_or(std::cmp::Ordering::Equal);
        }
        let ia = js_string_or_undefined(a.get("id"));
        let ib = js_string_or_undefined(b.get("id"));
        ib.cmp(&ia) // `a.id < b.id ? 1 : -1` — id descending
    });
    active.first().copied()
}

/// state-projection.mjs rebuildStateProjection(root, {cellCounts,
/// lastActivity}) — the hook always passes both overrides, so every branch
/// writes. Err(text) is the throw the .mjs's outer catch would crash-log.
fn rebuild_state_projection(
    root: &Path,
    ctrl: &Result<PathBuf, String>,
    counts: &Map<String, Value>,
    last_activity: &str,
) -> Result<(), String> {
    // listWorkflows(controlRootFor(root)) — an invalid linked worktree throws
    // WorktreeLinkInvalidError right here in the .mjs.
    let ctrl_root = match ctrl {
        Ok(p) => p.clone(),
        Err(msg) => return Err(msg.clone()),
    };
    let workflows = list_workflows(&ctrl_root);
    let current = read_state_failopen(root);
    let state_file = root.join(".bee").join("state.json");

    let apply_overrides = |next: &mut Map<String, Value>| {
        next.insert("cells".to_string(), Value::Object(counts.clone()));
        next.insert("last_activity".to_string(), Value::String(last_activity.to_string()));
    };
    let write_state = |next: Map<String, Value>| -> Result<(), String> {
        write_json_atomic(&state_file, &Value::Object(next)).map_err(|e| format!("Error: {e}"))
    };

    if workflows.is_empty() {
        // C1: zero workflow records — overrides-only write.
        let mut next = current;
        apply_overrides(&mut next);
        return write_state(next);
    }

    let feature = current.get("feature").cloned().unwrap_or(Value::Null);
    if truthy(&feature) {
        // Branch (1) — feature-matched, msn-10.
        let wf = workflows.iter().find(|w| {
            js_strict_eq(w.get("feature").unwrap_or(&Value::Null), &feature)
                && !js_strict_eq(w.get("status").unwrap_or(&Value::Null), &json!("closed"))
        });
        let mut next = current;
        if let Some(wf) = wf {
            for key in ["phase", "feature", "mode"] {
                next.insert(key.to_string(), wf.get(key).cloned().unwrap_or(Value::Null));
            }
            next.insert(
                "approved_gates".to_string(),
                workflow_gates_to_approved(
                    wf.get("gates").unwrap_or(&Value::Null),
                    wf.get("plan_rev").unwrap_or(&Value::Null),
                ),
            );
            for key in ["summary", "next_action"] {
                next.insert(key.to_string(), wf.get(key).cloned().unwrap_or(Value::Null));
            }
        }
        // Feature set but no live workflow naming it → overrides-only.
        apply_overrides(&mut next);
        return write_state(next);
    }

    // Branch (2) — idle bootstrap, unchanged since msn-7.
    let phase = current.get("phase").cloned().unwrap_or(Value::Null);
    let current_is_idle =
        phase == json!("idle") || phase == json!("compounding-complete") || !truthy(&phase);
    if !current_is_idle {
        let mut next = current;
        apply_overrides(&mut next);
        return write_state(next);
    }
    let active = pick_newest_active(&workflows);
    let mut next = current;
    match active {
        Some(wf) => {
            for key in ["phase", "feature", "mode"] {
                next.insert(key.to_string(), wf.get(key).cloned().unwrap_or(Value::Null));
            }
            next.insert(
                "approved_gates".to_string(),
                workflow_gates_to_approved(
                    wf.get("gates").unwrap_or(&Value::Null),
                    wf.get("plan_rev").unwrap_or(&Value::Null),
                ),
            );
            for key in ["summary", "next_action"] {
                next.insert(key.to_string(), wf.get(key).cloned().unwrap_or(Value::Null));
            }
        }
        None => {
            next.insert("phase".to_string(), json!("idle"));
            next.insert("feature".to_string(), Value::Null);
            next.insert("mode".to_string(), Value::Null);
            next.insert("approved_gates".to_string(), Value::Object(default_gates()));
            next.insert("summary".to_string(), json!(""));
            next.insert(
                "next_action".to_string(),
                json!("No active bee work \u{2014} awaiting a user request."),
            );
        }
    }
    apply_overrides(&mut next);
    write_state(next)
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("cells")).unwrap();
        tmp
    }

    fn write_cell(root: &Path, id: &str, status: &str) {
        std::fs::write(
            root.join(".bee").join("cells").join(format!("{id}.json")),
            serde_json::to_string(&json!({"id": id, "status": status})).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn counts_cells_by_status() {
        let tmp = setup_root();
        write_cell(tmp.path(), "a", "open");
        write_cell(tmp.path(), "b", "capped");
        write_cell(tmp.path(), "c", "capped");
        write_cell(tmp.path(), "d", "weird"); // unknown status: not counted
        std::fs::create_dir_all(tmp.path().join(".bee/cells/archive/f")).unwrap(); // dirs skipped
        let counts = count_cells(tmp.path());
        assert_eq!(
            crate::jsjson::stringify(&Value::Object(counts)),
            r#"{"open":1,"claimed":0,"capped":2,"blocked":0}"#
        );
    }

    #[test]
    fn rebuild_with_zero_workflows_is_overrides_only() {
        let tmp = setup_root();
        std::fs::write(
            tmp.path().join(".bee/state.json"),
            r#"{"schema_version":"1.0","phase":"swarming","feature":"f1","extra":42}"#,
        )
        .unwrap();
        let counts = count_cells(tmp.path());
        rebuild_state_projection(tmp.path(), &Ok(tmp.path().to_path_buf()), &counts, "T")
            .unwrap();
        let out: Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".bee/state.json")).unwrap())
                .unwrap();
        // D1 fields untouched, extras preserved, cells/last_activity refreshed.
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["extra"], json!(42));
        assert_eq!(out["cells"], json!({"open":0,"claimed":0,"capped":0,"blocked":0}));
        assert_eq!(out["last_activity"], json!("T"));
    }

    fn write_workflow(root: &Path, id: &str, body: Value) {
        let dir = root.join(".bee/runtime/workflows").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), serde_json::to_string(&body).unwrap()).unwrap();
    }

    #[test]
    fn rebuild_idle_bootstrap_adopts_newest_active_workflow() {
        let tmp = setup_root();
        std::fs::write(tmp.path().join(".bee/state.json"), r#"{"phase":"idle"}"#).unwrap();
        write_workflow(
            tmp.path(),
            "wf-old",
            json!({"id":"wf-old","feature":"f-old","status":"active","phase":"planning",
                   "mode":"standard","plan_rev":0,"summary":"s","next_action":"n",
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"shape":{"approved":true,"approved_for_plan_rev":0}}}),
        );
        write_workflow(
            tmp.path(),
            "wf-new",
            json!({"id":"wf-new","feature":"f-new","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"s2","next_action":"n2",
                   "created_at":"2026-02-01T00:00:00.000Z",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":2}}}),
        );
        let counts = count_cells(tmp.path());
        rebuild_state_projection(tmp.path(), &Ok(tmp.path().to_path_buf()), &counts, "T")
            .unwrap();
        let out: Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".bee/state.json")).unwrap())
                .unwrap();
        assert_eq!(out["feature"], json!("f-new"));
        assert_eq!(out["phase"], json!("swarming"));
        // execution gate stamped for plan_rev 2 but record is at plan_rev 1 →
        // projects false (plan-rev-effective approval, msn-9 D7).
        assert_eq!(
            out["approved_gates"],
            json!({"context":false,"shape":false,"execution":false,"review":false})
        );
    }

    #[test]
    fn rebuild_feature_match_projects_gates_with_plan_rev() {
        let tmp = setup_root();
        std::fs::write(
            tmp.path().join(".bee/state.json"),
            r#"{"phase":"planning","feature":"f1","workers":["w"]}"#,
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"lane","plan_rev":3,"summary":"sum","next_action":"next",
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"context":{"approved":true,"approved_for_plan_rev":null},
                            "execution":{"approved":true,"approved_for_plan_rev":3}}}),
        );
        let counts = count_cells(tmp.path());
        rebuild_state_projection(tmp.path(), &Ok(tmp.path().to_path_buf()), &counts, "T")
            .unwrap();
        let out: Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".bee/state.json")).unwrap())
                .unwrap();
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["mode"], json!("lane"));
        assert_eq!(out["summary"], json!("sum"));
        assert_eq!(out["workers"], json!(["w"])); // pass-through field
        assert_eq!(
            out["approved_gates"],
            json!({"context":true,"shape":false,"execution":true,"review":false})
        );
    }

    #[test]
    fn heartbeat_throttles_and_renews() {
        let tmp = setup_root();
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let fresh = iso_ms(&Utc::now());
        std::fs::write(
            sessions.join("s1.json"),
            serde_json::to_string(&json!({"id":"s1","last_heartbeat": fresh})).unwrap(),
        )
        .unwrap();
        // Fresh heartbeat -> throttled -> stale predicate false.
        let rec = read_session_failopen(tmp.path(), "s1");
        let now_ms = Utc::now().timestamp_millis() as f64;
        assert!(!heartbeat_stale(rec.as_ref(), now_ms, 60.0));
        // Stale (old) heartbeat -> heartbeat_session rewrites it.
        std::fs::write(
            sessions.join("s1.json"),
            serde_json::to_string(&json!({"id":"s1","last_heartbeat":"2020-01-01T00:00:00.000Z","lane":"l1"}))
                .unwrap(),
        )
        .unwrap();
        let rec = read_session_failopen(tmp.path(), "s1");
        assert!(heartbeat_stale(rec.as_ref(), now_ms, 60.0));
        heartbeat_session(tmp.path(), "s1", &Utc::now()).unwrap();
        let after: Value =
            serde_json::from_str(&std::fs::read_to_string(sessions.join("s1.json")).unwrap()).unwrap();
        assert_eq!(after["id"], json!("s1"));
        assert_eq!(after["lane"], json!("l1")); // untouched fields survive
        assert_ne!(after["last_heartbeat"], json!("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn renew_claim_ttl_touches_only_owned_claims() {
        let tmp = setup_root();
        let claims = tmp.path().join(".bee").join("claims");
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join("c1.json"),
            r#"{"cell":"c1","session":"s1","claimed_at":"2020-01-01T00:00:00.000Z","ttl_seconds":3600}"#,
        )
        .unwrap();
        std::fs::write(
            claims.join("c2.json"),
            r#"{"cell":"c2","session":"other","claimed_at":"2020-01-01T00:00:00.000Z"}"#,
        )
        .unwrap();
        renew_claim_ttl(tmp.path(), "s1", &Utc::now()).unwrap();
        let c1: Value =
            serde_json::from_str(&std::fs::read_to_string(claims.join("c1.json")).unwrap()).unwrap();
        let c2: Value =
            serde_json::from_str(&std::fs::read_to_string(claims.join("c2.json")).unwrap()).unwrap();
        assert_ne!(c1["claimed_at"], json!("2020-01-01T00:00:00.000Z"));
        assert_eq!(c2["claimed_at"], json!("2020-01-01T00:00:00.000Z"));
        // Gate files are released.
        assert!(!claims.join("c1.adopting").exists());
    }

    #[test]
    fn renew_cross_worktree_holds_renews_active_session_rows_only() {
        let tmp = setup_root();
        // The vendored-module presence gate must pass for the tmp root.
        let lib = tmp.path().join(".bee").join("bin").join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(lib.join("worktree-holds.mjs"), "// vendored").unwrap();
        let runtime = tmp.path().join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        // Row 0: active + owned (mirrored 30min ago, 1h TTL) → renewed.
        // Row 1: released → untouched. Row 2: other session → untouched.
        // Row 3: owned but TTL-expired → never resurrected.
        let active_at =
            iso_ms(&(Utc::now() - chrono::Duration::minutes(30)));
        let expired_at = "2026-01-01T00:00:00.000Z";
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            serde_json::to_string(&json!({"holds": [
                {"path":"a","holder":"main","session":"s1","ttl_seconds":3600,
                 "mirrored_at": active_at, "released_at":null},
                {"path":"b","holder":"main","session":"s1","ttl_seconds":3600,
                 "mirrored_at": active_at, "released_at":"2026-01-02T00:00:00.000Z"},
                {"path":"c","holder":"main","session":"other","ttl_seconds":3600,
                 "mirrored_at": active_at, "released_at":null},
                {"path":"d","holder":"main","session":"s1","ttl_seconds":3600,
                 "mirrored_at": expired_at, "released_at":null}
            ]}))
            .unwrap(),
        )
        .unwrap();
        // The test process's cwd is an ordinary checkout, so main_root falls
        // back to the `root` argument (tmp) — the solo/main no-op branch.
        renew_cross_worktree_holds(tmp.path(), "s1").unwrap();
        let after: Value = serde_json::from_str(
            &std::fs::read_to_string(runtime.join("cross-worktree-holds.json")).unwrap(),
        )
        .unwrap();
        let renewed = after["holds"][0]["mirrored_at"].as_str().unwrap();
        assert!(renewed > active_at.as_str(), "active owned row renews forward");
        assert_eq!(after["holds"][1]["mirrored_at"], json!(active_at)); // released: untouched
        assert_eq!(after["holds"][2]["mirrored_at"], json!(active_at)); // other session
        assert_eq!(after["holds"][3]["mirrored_at"], json!(expired_at)); // expired: untouched
    }

    #[test]
    fn lease_ttl_and_expiry_semantics() {
        let rec: Map<String, Value> = serde_json::from_str(
            r#"{"resource":"path:src/a.rs","session_id":"s1",
                "acquired_at":"2026-01-01T00:00:00.000Z","expires_at":"2026-01-01T01:00:00.000Z"}"#,
        )
        .unwrap();
        assert_eq!(lease_ttl_seconds(&rec), 3600.0);
        let no_expiry: Map<String, Value> =
            serde_json::from_str(r#"{"resource":"path:x","expires_at":null}"#).unwrap();
        assert_eq!(lease_ttl_seconds(&no_expiry), 0.0);
        let now = Utc::now();
        assert_eq!(compute_expires_at(0.0, &now), Value::Null);
        assert!(matches!(compute_expires_at(60.0, &now), Value::String(_)));
        assert_eq!(canonicalize_path("./src\\lib/"), "src/lib");
    }
}
