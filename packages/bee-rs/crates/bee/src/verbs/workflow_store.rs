// workflow_store — the lane/workflow store shared by the `state` verb group.
// LIBRARY module (no `try_native`, no probe line in verbs/mod.rs).
//
// This is the Rust home of everything the Node runtime keeps in
// lib/workflow-store.mjs, the lanes half of lib/state.mjs, the handoff
// mailbox half of lib/state.mjs, and lib/state-projection.mjs. Porting it is
// what lets verbs/state_group.rs drop its "C1 gate" (native only in a repo
// with no --lane, no lane-bound session, and zero `.bee/runtime/workflows/`
// records) and serve those verbs in EVERY repo shape.
//
// Provenance, function by function (source → this file):
//   lib/workflow-store.mjs
//     runtimeDir/workflowsDir/workflowDir/workflowStatePath → workflows_dir /
//       workflow_dir / workflow_state_path
//     requireWorkflowId              → require_workflow_id
//     defaultGateEntry/defaultGates  → default_gate_entry / default_wf_gates
//     mergeGates                     → merge_gates
//     baseWorkflowDefaults           → base_workflow_defaults
//     readWorkflowRecord             → read_workflow_record
//     listWorkflows                  → list_workflows
//     updateWorkflowAssumingLock     → update_workflow_assuming_lock(_with)
//     updateWorkflow                 → update_workflow
//     withWorkflowLock               → acquire_workflow_lock
//   lib/state.mjs (lanes)
//     lanesDir/requireLaneFeature/lanePath/defaultLaneRecord/laneRecordFrom →
//       lanes_dir / require_lane_feature / lane_path / default_lane_record /
//       lane_record_from
//     readLane / readLaneStrict / writeLane / listLanes → read_lane_display /
//       read_lane_strict / write_lane / list_lanes
//   lib/state.mjs (handoff mailbox)
//     requireHandoffWorkflowId/normalizeTargetRole/handoffMailboxDir/
//     handoffRecordPath/listHandoffMailbox/newestOpenHandoffMailboxRecord/
//     nextHandoffSeq/writeMailboxHandoff/adoptMailboxHandoff →
//       require_handoff_workflow_id / normalize_target_role /
//       handoff_mailbox_dir / handoff_record_path / list_handoff_mailbox /
//       newest_open_handoff_mailbox_record / write_mailbox_handoff /
//       adopt_mailbox_handoff
//     normalizeHandoffKind           → normalize_handoff_kind
//   lib/state-projection.mjs
//     workflowGatesToApprovedGates   → workflow_gates_to_approved_gates
//     pickNewestActiveWorkflow       → pick_newest_active_workflow
//     rebuildStateProjection         → rebuild_state_projection
//     rebuildLaneProjection          → rebuild_lane_projection
//     rebuildHandoffProjection       → rebuild_handoff_projection
//   bee.mjs
//     findGateStamp                  → find_gate_stamp
//     laneLockName/projectionLockName→ lane_lock_name / projection_lock_name
//     workflowsListSort              → workflows_list_sort
//
// SECOND-PORT NOTE (required by the campaign rule "keep one behavior, not
// two"): src/hooks/state_sync.rs already carries a faithful port of
// listWorkflows / readWorkflowRecord / mergeGates / baseWorkflowDefaults /
// workflowGatesToApprovedGates / pickNewestActiveWorkflow /
// rebuildStateProjection for the state-sync hook. Those functions are MODULE-
// PRIVATE there (`fn`, not `pub(crate)`) and state_sync.rs is outside this
// cell's touchable file set, so they are re-derived here from the same .mjs
// sources rather than imported. `agrees_with_state_sync_port_on_shared_fixtures`
// below pins the two against the exact fixtures state_sync.rs's own tests use,
// so a future divergence fails a test instead of drifting silently. Two
// deliberate refinements over the hook's copy are called out inline:
//   * a workflow record whose key is ABSENT projects as an absent key (JS
//     `{...current, feature: undefined}` is dropped by JSON.stringify), where
//     the hook writes `null`. Unreachable for any bee-created record.
//   * list_workflows REPRODUCES the skip warn natively (the hook's copy still
//     delegates on any skip). Only the two arms whose reason embeds a V8 parse
//     message or a libuv errno string route back to Node — see its own comment.
//
// Locking: identical lock-name strings to Node so both runtimes interoperate
// mid-campaign — `workflow:<id>` (workflow-store.mjs withWorkflowLock),
// `lane:<feature>` (bee.mjs laneLockName), `handoff:<workflow-id>`
// (state.mjs writeMailboxHandoff/adoptMailboxHandoff), and plain `state` for
// the default projection record. crate::lock's sanitizeLockName twin hashes
// the ':' forms into distinct lock files exactly as lock.mjs does.
//
// Control root: every caller reaches here through verbs/reservations.rs's
// `prelude`, whose resolve_store_root answers NeedsNode for a linked
// worktree — so on the native path controlRootFor(root) === root and the
// msn-18c re-rooting is the identity. Callers pass plain `root`.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, LockGuard, MAX_ATTEMPTS};
use crate::verbs::reservations::{
    date_parse_val, jget, js_disp, js_disp_opt, js_strict_eq, js_trim, now_iso, pseudo_uuid_v4,
    truthy, Err2, Ex, Exotic,
};
use crate::verbs::state_group::{
    adopt_claim, coerce_legacy_phase, default_gates, handoff_path, parse_json_v8, read_claim,
    read_state_peek, spread_gates, write_state, AdoptOutcome, ParsedJson,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

pub(crate) const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
const STATUS_VALUES: [&str; 3] = ["active", "paused", "closed"];
const HANDOFF_SEQ_WIDTH: usize = 4;

// ─── paths ─────────────────────────────────────────────────────────────────

/// workflow-store.mjs runtimeDir/workflowsDir.
pub(crate) fn workflows_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("workflows")
}

/// workflow-store.mjs workflowDir (id already validated by the caller).
fn workflow_dir(root: &Path, id: &str) -> PathBuf {
    workflows_dir(root).join(id)
}

/// workflow-store.mjs workflowStatePath.
pub(crate) fn workflow_state_path(root: &Path, id: &str) -> PathBuf {
    workflow_dir(root, id).join("state.json")
}

/// Node path.relative(root, file) for a file under root — the shape every
/// "git checkout -- <rel>" FIX hint interpolates.
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

/// workflow-store.mjs requireWorkflowId — a typed WorkflowStoreError whose
/// bytes are deterministic (no V8 text), so it is served natively.
pub(crate) fn require_workflow_id(value: &str) -> Result<String, Err2> {
    let id = js_trim(value);
    if id.is_empty() {
        return Err(Err2::Msg("workflow id is required.".to_string()));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Err2::Msg(format!(
            "workflow id \"{id}\" must be a plain id (no path separators) — it becomes a directory name under .bee/runtime/workflows/."
        )));
    }
    Ok(id.to_string())
}

// ─── gates ─────────────────────────────────────────────────────────────────

/// workflow-store.mjs defaultGateEntry.
fn default_gate_entry() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("approved".into(), Value::Bool(false));
    m.insert("approved_for_plan_rev".into(), Value::Null);
    m
}

/// workflow-store.mjs defaultGates.
fn default_wf_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for name in GATE_NAMES {
        m.insert(name.to_string(), Value::Object(default_gate_entry()));
    }
    m
}

/// workflow-store.mjs mergeGates(base, overrides) — `{...defaultGates(),
/// ...(base||{})}` then a one-level-deep per-gate-name overlay. Unknown gate
/// names in `overrides` are kept (forward-compat), every GATE_NAMES entry is
/// always present.
pub(crate) fn merge_gates(base: Option<&Value>, overrides: Option<&Value>) -> Value {
    let mut merged = default_wf_gates();
    match base {
        Some(Value::Object(b)) => {
            for (k, v) in b {
                merged.insert(k.clone(), v.clone());
            }
        }
        // `{...base}` of a falsy or property-less primitive adds nothing.
        _ => {}
    }
    if let Some(Value::Object(over)) = overrides {
        for (name, value) in over {
            // `merged[name] || defaultGateEntry()` then `{...baseEntry, ...value}`.
            let mut entry = match merged.get(name) {
                Some(Value::Object(e)) => e.clone(),
                Some(v) if truthy(v) => Map::new(), // spread of a truthy primitive yields {}
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

/// workflow-store.mjs baseWorkflowDefaults.
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

// ─── record read ───────────────────────────────────────────────────────────

/// Why listWorkflows would skip an entry. `Reason` bytes are deterministic;
/// `Delegate` covers the two classes whose warn embeds a V8/errno string.
enum WfSkip {
    Reason(String),
    Delegate,
}

/// typeof-style label for the not-an-object refusal.
fn found_kind(v: &Value) -> &'static str {
    match v {
        Value::Array(_) => "an array",
        Value::Null | Value::Object(_) => "object", // typeof null === 'object'
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
    }
}

/// workflow-store.mjs readWorkflowRecord. Raw read + JSON.parse (no BOM
/// strip), then `{...baseWorkflowDefaults(), ...parsed, id, gates}`.
fn read_workflow_record(root: &Path, id: &str) -> Result<Map<String, Value>, WfSkip> {
    let file = workflow_state_path(root, id);
    let file_str = file.display().to_string();
    let rel = path_relative(root, &file);
    let bytes = match std::fs::read(&file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WfSkip::Reason(format!(
                "readWorkflow: no workflow record at \"{file_str}\". FIX: createWorkflow first, or check the id."
            )));
        }
        // `(${err.code})` — an errno string this port does not reproduce.
        Err(_) => return Err(WfSkip::Delegate),
        Ok(b) => b,
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed = match parse_json_v8(&text) {
        Err(Exotic) => return Err(WfSkip::Delegate),
        // `(${err.message})` — a V8 parse message.
        Ok(ParsedJson::Unparseable) => return Err(WfSkip::Delegate),
        Ok(ParsedJson::Parsed(v)) => v,
    };
    let Value::Object(parsed_map) = parsed else {
        return Err(WfSkip::Reason(format!(
            "readWorkflow: \"{file_str}\" exists but is not a JSON object (found {}).",
            found_kind(&parsed)
        )));
    };
    if !matches!(parsed_map.get("id"), Some(Value::String(s)) if s == id) {
        return Err(WfSkip::Reason(format!(
            "readWorkflow: \"{file_str}\" exists but its id field (\"{}\") does not match the requested workflow \"{id}\" — never trusted. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
            js_disp_opt(parsed_map.get("id"))
        )));
    }
    let mut merged = base_workflow_defaults();
    for (k, v) in &parsed_map {
        merged.insert(k.clone(), v.clone());
    }
    merged.insert("id".into(), json!(id));
    merged.insert("gates".into(), merge_gates(None, parsed_map.get("gates")));
    Ok(merged)
}

/// The `listWorkflows: skipping unreadable workflow "<id>" — <reason>` line
/// Node's `console.warn` puts on stderr for every skipped record. Kept as a
/// function so the tests assert the same bytes the runtime emits.
pub(crate) fn skip_warn_line(id: &str, reason: &str) -> String {
    format!("listWorkflows: skipping unreadable workflow \"{id}\" — {reason}")
}

/// workflow-store.mjs listWorkflows — fail-open enumeration in directory-read
/// order. A corrupt or unreadable entry is SKIPPED (never guessed at, never
/// thrown for the whole list) and reported with a `console.warn` per skip.
///
/// SKIP TOLERANCE IS NATIVE (R6 blocker closed). The reason bytes come
/// straight from `read_workflow_record`'s own refusals, so the three ordinary
/// skips — missing record, not-a-JSON-object, id mismatch — warn here exactly
/// as Node does. The repeat count needs no modelling: this function is called
/// from the same places `listWorkflows` is (the mutation-lock scope read, the
/// write-through, and each rebuild*), so a verb that calls it 2–4 times emits
/// the stream 2–4 times by construction. `state_lanes_over_a_broken_workflow_
/// record_warns_once_per_call` pins that.
///
/// WHAT STILL DELEGATES — two arms, both because the reason embeds bytes this
/// port cannot author (rust-port.md campaign rule 2, "refusals delegate unless
/// their bytes are deterministic"):
///   1. `readWorkflow: "<file>" exists but is not valid JSON (${err.message})`
///      — a V8 parse message.
///   2. `readWorkflow: could not read "<file>" (${err.code})` — a libuv errno
///      string for a non-ENOENT read failure (EISDIR/EACCES/EPERM). Node's
///      mapping of a Win32 error to that code is not reproducible from
///      `std::io::Error` (a directory read is ACCESS_DENIED on Win32 and
///      libuv rewrites it to EISDIR only in some paths).
/// Both are decided in a PRE-PASS: the whole directory is classified before a
/// single warn is written, so a delegating run still emits zero bytes first
/// (`Outcome`-equivalent for verbs: `try_native` must produce no output before
/// returning None).
pub(crate) fn list_workflows(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let Ok(rd) = std::fs::read_dir(workflows_dir(root)) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    let mut warns: Vec<String> = Vec::new();
    for entry in rd.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // `if (!entry.isDirectory()) continue` — silent, no warn
        }
        let Some(id) = entry.file_name().to_str().map(str::to_string) else {
            return Err(Exotic); // non-UTF-8 name: Node's id bytes are unmodelable here
        };
        match read_workflow_record(root, &id) {
            Ok(record) => out.push(record),
            Err(WfSkip::Reason(reason)) => warns.push(skip_warn_line(&id, &reason)),
            Err(WfSkip::Delegate) => return Err(Exotic), // see the doc comment
        }
    }
    // Emitted only once the whole scan is known warn-reproducible.
    for line in warns {
        eprintln!("{line}");
    }
    Ok(out)
}

/// `workflows.find((w) => w.feature === feature && w.status !== 'closed')` —
/// the "live workflow naming this feature" lookup every write path shares.
pub(crate) fn find_live_workflow<'a>(
    workflows: &'a [Map<String, Value>],
    feature: &str,
) -> Option<&'a Map<String, Value>> {
    let want = Value::String(feature.to_string());
    workflows.iter().find(|w| {
        js_strict_eq(w.get("feature").unwrap_or(&Value::Null), &want)
            && !js_strict_eq(w.get("status").unwrap_or(&Value::Null), &json!("closed"))
    })
}

/// `wf.id` as a plain String (readWorkflowRecord always stamps it).
pub(crate) fn wf_id(wf: &Map<String, Value>) -> String {
    match wf.get("id") {
        Some(Value::String(s)) => s.clone(),
        other => js_disp_opt(other),
    }
}

// ─── locks ─────────────────────────────────────────────────────────────────

/// workflow-store.mjs withWorkflowLock — `workflow:<id>` via withStoreLock.
pub(crate) fn acquire_workflow_lock(root: &Path, id: &str) -> Result<LockGuard, Err2> {
    let workflow_id = require_workflow_id(id)?;
    lock::acquire_store_lock(root, &format!("workflow:{workflow_id}"), MAX_ATTEMPTS)
        .map_err(|b| Err2::Msg(b.message()))
}

/// bee.mjs laneLockName.
pub(crate) fn lane_lock_name(feature: &str) -> String {
    format!("lane:{feature}")
}

/// bee.mjs projectionLockName(scope).
pub(crate) fn projection_lock_name(lane: bool, feature: Option<&str>) -> String {
    match (lane, feature) {
        (true, Some(f)) if !f.is_empty() => lane_lock_name(f),
        _ => "state".to_string(),
    }
}

pub(crate) fn acquire_named_lock(root: &Path, name: &str) -> Result<LockGuard, Err2> {
    lock::acquire_store_lock(root, name, MAX_ATTEMPTS).map_err(|b| Err2::Msg(b.message()))
}

// ─── record write ──────────────────────────────────────────────────────────

/// `{...current, ...patch, id: current.id}` — an identity key ABSENT on
/// `current` becomes `undefined` in JS and is dropped by JSON.stringify, so a
/// patch can never smuggle one in.
fn protect_identity(next: &mut Map<String, Value>, current: &Map<String, Value>, key: &str) {
    match current.get(key) {
        Some(v) => {
            next.insert(key.to_string(), v.clone());
        }
        None => {
            next.shift_remove(key);
        }
    }
}

fn check_patch_status(patch: &Map<String, Value>) -> Result<(), Err2> {
    match patch.get("status") {
        None => Ok(()),
        Some(Value::String(s)) if STATUS_VALUES.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(Err2::Msg(format!(
            "updateWorkflowAssumingLock: status must be one of active/paused/closed (got {}).",
            jsjson::stringify(v)
        ))),
    }
}

/// workflow-store.mjs updateWorkflowAssumingLock with a FUNCTION updater —
/// `updater({...current})` produces the patch. NEVER call without already
/// holding `workflow:<id>` (lock.mjs is a non-reentrant O_EXCL primitive).
pub(crate) fn update_workflow_assuming_lock_with(
    root: &Path,
    id: &str,
    updater: impl FnOnce(&Map<String, Value>) -> Result<Map<String, Value>, Err2>,
) -> Result<Map<String, Value>, Err2> {
    let workflow_id = require_workflow_id(id)?;
    let current = match read_workflow_record(root, &workflow_id) {
        Ok(c) => c,
        Err(WfSkip::Reason(msg)) => return Err(Err2::Msg(msg)),
        Err(WfSkip::Delegate) => return Err(Err2::Ex),
    };
    let patch = updater(&current)?;
    check_patch_status(&patch)?;
    let mut next = current.clone();
    for (k, v) in &patch {
        next.insert(k.clone(), v.clone());
    }
    protect_identity(&mut next, &current, "id");
    protect_identity(&mut next, &current, "feature");
    protect_identity(&mut next, &current, "created_at");
    next.insert(
        "gates".into(),
        merge_gates(current.get("gates"), patch.get("gates")),
    );
    write_json_atomic(&workflow_state_path(root, &workflow_id), &Value::Object(next.clone()))
        .map_err(|_| Err2::Ex)?;
    Ok(next)
}

// ─── record creation (workflow-store.mjs createWorkflow) ───────────────────

/// workflow-store.mjs generateWorkflowId — `wf-${randomBytes(4).hex}`, with
/// the one defensive regeneration the .mjs carries: a generated id that
/// happens to equal the caller's own feature slug is rerolled, so D1's
/// "workflow_id is never the feature slug" never has to fire on the default
/// generator. Randomness is derived the same way reservations.rs's
/// pseudo_uuid_v4 and lock.rs's fresh_token derive theirs (pid + counter +
/// clock nanos through sha256) rather than adding an RNG dependency — the
/// store needs uniqueness, and the value is never compared to Node's.
#[allow(dead_code)] // see create_workflow's own note on wiring
fn generate_workflow_id(feature: Option<&str>) -> String {
    let mut id = format!("wf-{}", &pseudo_uuid_v4().replace('-', "")[..8]);
    while matches!(feature, Some(f) if id == f) {
        id = format!("wf-{}", &pseudo_uuid_v4().replace('-', "")[..8]);
    }
    id
}

/// The caller-supplied half of createWorkflow's options object. `None` means
/// the JS parameter was `undefined`, i.e. its default applies.
#[allow(dead_code)]
pub(crate) struct NewWorkflow<'a> {
    pub feature: Option<&'a str>,
    pub phase: Option<Value>,
    pub mode: Option<Value>,
    pub plan_rev: Option<Value>,
    pub gates: Option<Value>,
    pub summary: Option<Value>,
    pub next_action: Option<Value>,
    pub status: Option<&'a str>,
    pub id: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> NewWorkflow<'a> {
    pub(crate) fn for_feature(feature: &'a str) -> Self {
        Self {
            feature: Some(feature),
            phase: None,
            mode: None,
            plan_rev: None,
            gates: None,
            summary: None,
            next_action: None,
            status: None,
            id: None,
        }
    }
}

/// createWorkflow(root, {...}) — create a new workflow record under its own
/// `workflow:<id>` lock, so two callers racing the same explicit id can never
/// both "win" a create. Never overwrites an existing record.
///
/// Every refusal here has deterministic bytes (no V8 text), so all four are
/// reproduced natively: WORKFLOW_INVALID_ID (from requireWorkflowId or the D1
/// id-equals-feature invariant), WORKFLOW_INVALID_FEATURE,
/// WORKFLOW_INVALID_STATUS, and WORKFLOW_ALREADY_EXISTS — the last of which is
/// reached AFTER the lock is taken and therefore MUST be native (campaign rule
/// 2), or a delegation would double the contention telemetry.
///
/// NOT YET WIRED TO A VERB, deliberately: Node's only production callers are
/// `state.mjs`'s `ensureWorkflowRecordForFeature` / `startFeature` /
/// `seedLegacyWorkflows`, and `state start-feature` is still a documented
/// delegation (its `applyWritePolicy` + workspace-store half is unported).
/// What this closes is the DELETION blocker: record creation exists natively
/// now, byte-pinned against the live Node oracle, so `bee.mjs` can go without
/// losing it. `verbs/state_group.rs` — the file that will call it — is owned
/// by another in-flight cell, so the call site is left to that cell.
#[allow(dead_code)]
pub(crate) fn create_workflow(
    root: &Path,
    opts: NewWorkflow<'_>,
) -> Result<Map<String, Value>, Err2> {
    // `id = generateWorkflowId(feature)` is a DEFAULT PARAMETER: it is
    // evaluated before requireFeature runs, which is why an invalid feature
    // still reports the feature refusal, not an id one.
    let generated;
    let raw_id = match opts.id {
        Some(id) => id,
        None => {
            generated = generate_workflow_id(opts.feature.map(js_trim).filter(|f| !f.is_empty()));
            &generated
        }
    };
    let workflow_id = require_workflow_id(raw_id)?;
    let feature_name = match opts.feature.map(js_trim) {
        Some(f) if !f.is_empty() => f.to_string(),
        _ => return Err(Err2::Msg("createWorkflow: feature is required.".to_string())),
    };
    if workflow_id == feature_name {
        return Err(Err2::Msg(format!(
            "createWorkflow: workflow id \"{workflow_id}\" must not equal the feature slug \"{feature_name}\" — ids are \
generated identifiers, never feature slugs (CONTEXT.md D1: a feature can reopen or run competing \
attempts, so identity must never collide with the human-chosen name). FIX: pass an explicit id distinct \
from the feature, or omit id to let one be generated."
        )));
    }
    let status = opts.status.unwrap_or("active");
    if !STATUS_VALUES.contains(&status) {
        return Err(Err2::Msg(format!(
            "createWorkflow: status must be one of active/paused/closed (got {}).",
            jsjson::stringify(&Value::String(status.to_string()))
        )));
    }

    let guard = acquire_workflow_lock(root, &workflow_id)?;
    let out = (|| -> Result<Map<String, Value>, Err2> {
        let file = workflow_state_path(root, &workflow_id);
        if file.exists() {
            return Err(Err2::Msg(format!(
                "createWorkflow: a workflow record already exists at \"{}\" — createWorkflow never overwrites an \
existing record. FIX: use updateWorkflow, or generate a fresh id.",
                file.display()
            )));
        }
        // Key ORDER is `{...baseWorkflowDefaults(), id, feature, phase, mode,
        // plan_rev, gates, summary, next_action, status, created_at}` — a JS
        // re-assignment keeps the key's ORIGINAL position, so the six defaults
        // stay first and only id/feature/gates/created_at are appended. This
        // is C1 surface: readWorkflowRecord must read back what create wrote.
        let mut record = base_workflow_defaults();
        record.insert("id".into(), Value::String(workflow_id.clone()));
        record.insert("feature".into(), Value::String(feature_name));
        record.insert("phase".into(), opts.phase.unwrap_or_else(|| json!("idle")));
        record.insert("mode".into(), opts.mode.unwrap_or(Value::Null));
        record.insert("plan_rev".into(), opts.plan_rev.unwrap_or_else(|| json!(0)));
        // `mergeGates(defaultGates(), gates)` — merge_gates already seeds the
        // full default map, so a None base is the same value, not a shortcut.
        record.insert("gates".into(), merge_gates(None, opts.gates.as_ref()));
        record.insert("summary".into(), opts.summary.unwrap_or_else(|| json!("")));
        record.insert("next_action".into(), opts.next_action.unwrap_or_else(|| json!("")));
        record.insert("status".into(), Value::String(status.to_string()));
        record.insert("created_at".into(), Value::String(now_iso()));
        write_json_atomic(&file, &Value::Object(record.clone())).map_err(|_| Err2::Ex)?;
        Ok(record)
    })();
    drop(guard);
    out
}

/// workflow-store.mjs updateWorkflowAssumingLock with a plain object patch.
pub(crate) fn update_workflow_assuming_lock(
    root: &Path,
    id: &str,
    patch: Map<String, Value>,
) -> Result<Map<String, Value>, Err2> {
    update_workflow_assuming_lock_with(root, id, move |_| Ok(patch))
}

/// workflow-store.mjs updateWorkflow — the SELF-LOCKING form (takes
/// `workflow:<id>` itself). Only for callers holding no workflow lock.
pub(crate) fn update_workflow(
    root: &Path,
    id: &str,
    patch: Map<String, Value>,
) -> Result<Map<String, Value>, Err2> {
    let workflow_id = require_workflow_id(id)?;
    let guard = acquire_workflow_lock(root, &workflow_id)?;
    let out = update_workflow_assuming_lock(root, &workflow_id, patch);
    drop(guard);
    out
}

// ─── lanes (lib/state.mjs) ─────────────────────────────────────────────────

pub(crate) fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — deterministic throws, served natively.
pub(crate) fn require_lane_feature(value: &str) -> Result<String, Err2> {
    let feature = js_trim(value);
    if feature.is_empty() {
        return Err(Err2::Msg("lane feature is required.".to_string()));
    }
    if feature.contains('/') || feature.contains('\\') || feature.contains("..") {
        return Err(Err2::Msg(
            "lane feature must be a plain id (no path separators).".to_string(),
        ));
    }
    Ok(feature.to_string())
}

pub(crate) fn lane_path(root: &Path, feature: &str) -> Result<PathBuf, Err2> {
    Ok(lanes_dir(root).join(format!("{}.json", require_lane_feature(feature)?)))
}

/// state.mjs defaultLaneRecord.
pub(crate) fn default_lane_record(feature: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("feature".into(), json!(feature));
    m.insert("mode".into(), Value::Null);
    m.insert("phase".into(), json!("idle"));
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!(""));
    m.insert("created_at".into(), Value::Null);
    m
}

/// state.mjs laneRecordFrom — null unless the parsed content is a lane record
/// for THIS feature; merged over the per-feature defaults.
pub(crate) fn lane_record_from(feature: &str, parsed: &Value) -> Ex<Option<Map<String, Value>>> {
    let obj = match parsed {
        Value::Object(m) => m,
        _ => return Ok(None),
    };
    if !matches!(obj.get("feature"), Some(Value::String(s)) if s == feature) {
        return Ok(None);
    }
    let mut merged = default_lane_record(feature);
    for (k, v) in obj {
        merged.insert(k.clone(), v.clone());
    }
    let gates = spread_gates(obj.get("approved_gates"))?;
    merged.insert("approved_gates".into(), Value::Object(gates));
    coerce_legacy_phase(&mut merged)?;
    Ok(Some(merged))
}

/// state.mjs readLane — the fail-open DISPLAY read. Corrupt-but-valid-JSON
/// records get the deterministic warn + skip; JSON-corrupt files delegate
/// (Node's readJson warns first with the V8 message).
pub(crate) fn read_lane_display(root: &Path, raw_feature: &str) -> Ex<Option<Map<String, Value>>> {
    let feature = js_trim(raw_feature);
    if feature.is_empty()
        || feature.contains('/')
        || feature.contains('\\')
        || feature.contains("..")
    {
        return Ok(None); // lanePath's requireLaneFeature throw is caught → "no lane"
    }
    let file = lanes_dir(root).join(format!("{feature}.json"));
    if !file.exists() {
        return Ok(None);
    }
    let warn = || {
        let rel = format!(".bee{0}lanes{0}{feature}.json", MAIN_SEPARATOR);
        eprintln!(
            "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        );
    };
    match read_json(&file) {
        ReadJson::Missing => {
            // existsSync raced a delete: readJson falls back → record null → warn.
            warn();
            Ok(None)
        }
        ReadJson::Corrupt => Err(Exotic),
        ReadJson::Parsed(v) => {
            let v = crate::verbs::reservations::js_numberify(&v)?;
            match lane_record_from(feature, &v)? {
                Some(rec) => Ok(Some(rec)),
                None => {
                    warn();
                    Ok(None)
                }
            }
        }
    }
}

/// state.mjs listLanes.
pub(crate) fn list_lanes(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let entries = match std::fs::read_dir(lanes_dir(root)) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut lanes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane_display(root, stem)? {
            lanes.push(record);
        }
    }
    Ok(lanes)
}

/// state.mjs readLaneStrict — the MUTATION read. Missing reads as None
/// (creation is start-feature's job); present-but-corrupt THROWS with the
/// file untouched. Both refusals are deterministic (the corrupt arm never
/// embeds the V8 parse message) and therefore native.
pub(crate) fn read_lane_strict(
    root: &Path,
    feature: &str,
) -> Result<Option<Map<String, Value>>, Err2> {
    let id = require_lane_feature(feature)?;
    let file = lane_path(root, &id)?;
    let file_str = file.display().to_string();
    let rel = path_relative(root, &file);
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            // `(${err.code})` — only the two errno classes seen in practice are
            // reproduced; anything else delegates rather than guessing.
            let code = if file.is_dir() {
                "EISDIR"
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                "EPERM"
            } else {
                return Err(Err2::Ex);
            };
            return Err(Err2::Msg(format!(
                "readLaneStrict: could not read lane record \"{file_str}\" ({code}). The bee CLI refuses to mutate a lane it cannot read — that could silently clobber real lane state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry."
            )));
        }
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let record = match parse_json_v8(&text)? {
        ParsedJson::Unparseable => None,
        ParsedJson::Parsed(v) => lane_record_from(&id, &v)?,
    };
    match record {
        Some(r) => Ok(Some(r)),
        None => Err(Err2::Msg(format!(
            "readLaneStrict: lane record \"{file_str}\" exists but is corrupt (not a JSON object naming feature \"{id}\"). The bee CLI refuses to rebuild a lane from defaults over a present-but-corrupt file — that would silently clobber real lane state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry."
        ))),
    }
}

/// state.mjs writeLane.
pub(crate) fn write_lane(root: &Path, lane: &Map<String, Value>) -> Result<(), Err2> {
    let feature = match lane.get("feature") {
        Some(Value::String(s)) => s.clone(),
        // lanePath(root, lane.feature) coerces via requireLaneFeature, which
        // throws for every non-string — deterministic, but the coercion of an
        // exotic value is not modeled here.
        Some(_) => return Err(Err2::Ex),
        None => return Err(Err2::Msg("lane feature is required.".to_string())),
    };
    let file = lane_path(root, &feature)?;
    write_json_atomic(&file, &Value::Object(lane.clone())).map_err(|_| Err2::Ex)
}

// ─── projections (lib/state-projection.mjs) ────────────────────────────────

/// state-projection.mjs workflowGatesToApprovedGates(gates, planRev) — the
/// PLAN-REV-EFFECTIVE approval, in the fixed GATE_NAMES key order.
pub(crate) fn workflow_gates_to_approved_gates(gates: Option<&Value>, plan_rev: Option<&Value>) -> Value {
    let mut approved = Map::new();
    for name in GATE_NAMES {
        let entry = gates.and_then(|g| jget(g, name));
        let entry_truthy = entry.map(truthy).unwrap_or(false);
        let is_approved = entry_truthy
            && matches!(entry, Some(Value::Object(e)) if e.get("approved") == Some(&Value::Bool(true)));
        // `entry ? entry.approved_for_plan_rev : undefined`; property access on
        // a truthy primitive also yields undefined.
        let rev = if entry_truthy {
            match entry {
                Some(Value::Object(e)) => e.get("approved_for_plan_rev"),
                _ => None,
            }
        } else {
            None
        };
        let rev_effective = match rev {
            None | Some(Value::Null) => true,
            Some(v) => match plan_rev {
                Some(p) => js_strict_eq(v, p),
                None => false, // `rev === undefined` already handled; a real rev never === undefined
            },
        };
        approved.insert(name.to_string(), Value::Bool(is_approved && rev_effective));
    }
    Value::Object(approved)
}

/// state-projection.mjs pickNewestActiveWorkflow — active only, never
/// compounding-complete; created_at descending, then id descending.
pub(crate) fn pick_newest_active_workflow(
    workflows: &[Map<String, Value>],
) -> Ex<Option<&Map<String, Value>>> {
    let mut active: Vec<&Map<String, Value>> = Vec::new();
    for wf in workflows {
        if wf.get("status") == Some(&json!("active"))
            && !js_strict_eq(
                wf.get("phase").unwrap_or(&Value::Null),
                &json!("compounding-complete"),
            )
        {
            active.push(wf);
        }
    }
    if active.is_empty() {
        return Ok(None);
    }
    // `Date.parse(x.created_at) || 0` — NaN and 0 both collapse to 0.
    let stamp = |wf: &Map<String, Value>| -> Ex<f64> {
        Ok(date_parse_val(wf.get("created_at"))?.filter(|v| *v != 0.0).unwrap_or(0.0))
    };
    let mut keyed: Vec<(f64, String, &Map<String, Value>)> = Vec::new();
    for wf in active {
        keyed.push((stamp(wf)?, js_disp_opt(wf.get("id")), wf));
    }
    keyed.sort_by(|a, b| {
        if a.0 != b.0 {
            return b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal);
        }
        if a.1 == b.1 {
            return std::cmp::Ordering::Equal;
        }
        b.1.cmp(&a.1) // `a.id < b.id ? 1 : -1` — id descending
    });
    Ok(Some(keyed[0].2))
}

/// `next[key] = wf[key]` with JS undefined semantics: an absent source key
/// makes the destination key `undefined`, which JSON.stringify drops.
fn set_from(next: &mut Map<String, Value>, key: &str, wf: &Map<String, Value>) {
    match wf.get(key) {
        Some(v) => {
            next.insert(key.to_string(), v.clone());
        }
        None => {
            next.shift_remove(key);
        }
    }
}

fn apply_workflow_d1_fields(next: &mut Map<String, Value>, wf: &Map<String, Value>) {
    set_from(next, "phase", wf);
    set_from(next, "feature", wf);
    set_from(next, "mode", wf);
    next.insert(
        "approved_gates".into(),
        workflow_gates_to_approved_gates(wf.get("gates"), wf.get("plan_rev")),
    );
    set_from(next, "summary", wf);
    set_from(next, "next_action", wf);
}

/// state-projection.mjs rebuildStateProjection(root) with NO overrides — the
/// CLI's own shape (bee-state-sync is the only caller that passes
/// cellCounts/lastActivity, and that path lives in hooks/state_sync.rs). With
/// no overrides `applyOverridesOnly()` never writes, so every non-authoritative
/// branch here is a pure no-op.
pub(crate) fn rebuild_state_projection(root: &Path) -> Result<(), Err2> {
    rebuild_state_projection_reporting(root).map(|_| ())
}

/// The `{ authoritative, source, state|lane }` triple state-projection.mjs's
/// rebuild* functions return. Only `state rebuild-projections` reports it —
/// every other caller writes for effect, so `rebuild_state_projection` /
/// `rebuild_lane_projection` stay the thin shapes they already had.
pub(crate) struct Proj<T> {
    pub(crate) authoritative: bool,
    pub(crate) source: Value,
    pub(crate) record: T,
}

/// provenance: state-projection.mjs rebuildStateProjection's own return value
/// (see `rebuild_state_projection` above for the body's provenance).
pub(crate) fn rebuild_state_projection_reporting(
    root: &Path,
) -> Result<Proj<Map<String, Value>>, Err2> {
    let workflows = list_workflows(root)?;
    let current = read_state_peek(root)?; // read in Node's own order
    // applyOverridesOnly() with no overrides — never writes, `state: current`.
    let no_op = |state: Map<String, Value>| Proj {
        authoritative: false,
        source: Value::Null,
        record: state,
    };
    if workflows.is_empty() {
        return Ok(no_op(current)); // C1: zero workflow records — no write at all
    }
    let feature = current.get("feature").cloned().unwrap_or(Value::Null);
    if truthy(&feature) {
        // Branch (1) — feature-matched (msn-10).
        // A non-string truthy feature never `===` a record's string feature.
        if let Value::String(f) = &feature {
            if let Some(wf) = find_live_workflow(&workflows, f) {
                let source = wf.get("id").cloned().unwrap_or(Value::Null);
                let mut next = current.clone();
                apply_workflow_d1_fields(&mut next, wf);
                write_state(root, &next)?;
                return Ok(Proj { authoritative: true, source, record: next });
            }
        }
        // feature set, no live workflow names it → the idle-bootstrap branch
        // below requires `!current.feature`, so this is always a no-op.
        return Ok(no_op(current));
    }
    // Branch (2) — idle bootstrap (msn-7).
    let phase = current.get("phase").cloned().unwrap_or(Value::Null);
    let current_is_idle = js_strict_eq(&phase, &json!("idle"))
        || js_strict_eq(&phase, &json!("compounding-complete"))
        || !truthy(&phase);
    if !current_is_idle {
        return Ok(no_op(current));
    }
    let active = pick_newest_active_workflow(&workflows)?;
    let source = match active {
        Some(wf) => wf.get("id").cloned().unwrap_or(Value::Null),
        None => Value::Null,
    };
    let mut next = current.clone();
    match active {
        Some(wf) => apply_workflow_d1_fields(&mut next, wf),
        None => {
            next.insert("phase".into(), json!("idle"));
            next.insert("feature".into(), Value::Null);
            next.insert("mode".into(), Value::Null);
            next.insert("approved_gates".into(), Value::Object(default_gates()));
            next.insert("summary".into(), json!(""));
            next.insert(
                "next_action".into(),
                json!("No active bee work \u{2014} awaiting a user request."),
            );
        }
    }
    write_state(root, &next)?;
    Ok(Proj { authoritative: true, source, record: next })
}

/// state-projection.mjs rebuildLaneProjection(root, feature). Returns the
/// rebuilt lane record when the projection took authority, otherwise the
/// existing (fail-open) lane read — exactly `rebuilt.lane`.
pub(crate) fn rebuild_lane_projection(
    root: &Path,
    feature: &str,
) -> Result<Option<Map<String, Value>>, Err2> {
    Ok(rebuild_lane_projection_reporting(root, feature)?.record)
}

/// provenance: state-projection.mjs rebuildLaneProjection's own return value.
pub(crate) fn rebuild_lane_projection_reporting(
    root: &Path,
    feature: &str,
) -> Result<Proj<Option<Map<String, Value>>>, Err2> {
    let workflows = list_workflows(root)?;
    let no_op = |lane| Proj { authoritative: false, source: Value::Null, record: lane };
    if workflows.is_empty() {
        return Ok(no_op(read_lane_display(root, feature)?));
    }
    let Some(wf) = find_live_workflow(&workflows, feature) else {
        return Ok(no_op(read_lane_display(root, feature)?));
    };
    let source = wf.get("id").cloned().unwrap_or(Value::Null);
    let existing = read_lane_display(root, feature)?;
    let mut next = existing.clone().unwrap_or_default();
    next.insert("schema_version".into(), json!("1.0"));
    set_from(&mut next, "feature", wf);
    set_from(&mut next, "mode", wf);
    set_from(&mut next, "phase", wf);
    next.insert(
        "approved_gates".into(),
        workflow_gates_to_approved_gates(wf.get("gates"), wf.get("plan_rev")),
    );
    set_from(&mut next, "summary", wf);
    set_from(&mut next, "next_action", wf);
    // `(existing && existing.created_at) || wf.created_at || new Date()...`
    let created_at = existing
        .as_ref()
        .and_then(|e| e.get("created_at"))
        .filter(|v| truthy(v))
        .or_else(|| wf.get("created_at").filter(|v| truthy(v)))
        .cloned()
        .unwrap_or_else(|| json!(now_iso()));
    next.insert("created_at".into(), created_at);
    write_lane(root, &next)?;
    Ok(Proj { authoritative: true, source, record: Some(next) })
}

/// state-projection.mjs rebuildHandoffProjection(root) — the legacy
/// .bee/HANDOFF.json as a projection of the newest OPEN mailbox record across
/// every workflow. No-op at zero workflow records (C1); removes the legacy
/// file when workflows exist but none carries an open handoff.
pub(crate) fn rebuild_handoff_projection(root: &Path) -> Result<(), Err2> {
    rebuild_handoff_projection_reporting(root).map(|_| ())
}

/// provenance: state-projection.mjs rebuildHandoffProjection's own return value.
pub(crate) fn rebuild_handoff_projection_reporting(root: &Path) -> Result<Proj<()>, Err2> {
    let workflows = list_workflows(root)?;
    if workflows.is_empty() {
        return Ok(Proj { authoritative: false, source: Value::Null, record: () });
    }
    let mut newest: Option<(Map<String, Value>, String)> = None;
    for wf in &workflows {
        let id = wf_id(wf);
        let open: Vec<Map<String, Value>> = list_handoff_mailbox(root, &id)?
            .into_iter()
            .filter(|r| matches!(r.get("status"), Some(Value::String(s)) if s == "open"))
            .collect();
        let Some(candidate) = open.last().cloned() else { continue };
        match &newest {
            None => newest = Some((candidate, id)),
            Some((cur, cur_id)) => {
                let a = date_parse_val(candidate.get("written_at"))?
                    .filter(|v| *v != 0.0)
                    .unwrap_or(0.0);
                let b = date_parse_val(cur.get("written_at"))?
                    .filter(|v| *v != 0.0)
                    .unwrap_or(0.0);
                if a > b || (a == b && id > *cur_id) {
                    newest = Some((candidate, id));
                }
            }
        }
    }
    let Some((record, source_id)) = newest else {
        let _ = std::fs::remove_file(handoff_path(root)); // rmSync force:true
        return Ok(Proj { authoritative: true, source: Value::Null, record: () });
    };
    // Drop the mailbox-only fields so a legacy reader sees writeHandoff's shape.
    let mut projected = record;
    for key in ["seq", "status", "id", "workflow_id", "target_role", "from_session"] {
        projected.shift_remove(key);
    }
    write_json_atomic(&handoff_path(root), &Value::Object(projected)).map_err(|_| Err2::Ex)?;
    Ok(Proj { authoritative: true, source: json!(source_id), record: () })
}

// ─── handoff mailbox (lib/state.mjs) ───────────────────────────────────────

/// state.mjs normalizeHandoffKind.
fn normalize_handoff_kind(kind: Option<&Value>) -> &'static str {
    match kind {
        Some(Value::String(s)) if s == "planned-next" => "planned-next",
        _ => "pause",
    }
}

/// state.mjs normalizeTargetRole.
pub(crate) fn normalize_target_role(role: Option<&str>) -> Option<String> {
    match role {
        Some(r) if !js_trim(r).is_empty() => Some(js_trim(r).to_string()),
        _ => None,
    }
}

fn normalize_target_role_val(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

/// state.mjs requireHandoffWorkflowId.
fn require_handoff_workflow_id(value: &str) -> Result<String, Err2> {
    let id = js_trim(value);
    if id.is_empty() {
        return Err(Err2::Msg("handoff mailbox: workflow id is required.".to_string()));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Err2::Msg(format!(
            "handoff mailbox: workflow id \"{id}\" must be a plain id (no path separators)."
        )));
    }
    Ok(id.to_string())
}

pub(crate) fn handoff_mailbox_dir(root: &Path, workflow_id: &str) -> Result<PathBuf, Err2> {
    Ok(root
        .join(".bee")
        .join("runtime")
        .join("handoffs")
        .join(require_handoff_workflow_id(workflow_id)?))
}

fn handoff_record_path(root: &Path, workflow_id: &str, seq: i64) -> Result<PathBuf, Err2> {
    Ok(handoff_mailbox_dir(root, workflow_id)?
        .join(format!("{:0width$}.json", seq, width = HANDOFF_SEQ_WIDTH)))
}

/// state.mjs listHandoffMailbox — oldest→newest by seq; `seq` attached
/// in-memory from the filename, `kind` normalized on read.
pub(crate) fn list_handoff_mailbox(root: &Path, workflow_id: &str) -> Ex<Vec<Map<String, Value>>> {
    let Ok(dir) = handoff_mailbox_dir(root, workflow_id) else {
        return Ok(Vec::new()); // the throw is not caught in Node — but every
        // caller here passes an id straight off a workflow record's directory
        // name, which already satisfies requireHandoffWorkflowId.
    };
    let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(Vec::new()) };
    let mut records: Vec<Map<String, Value>> = Vec::new();
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        let Some(seq) = js_parse_int(stem) else { continue };
        let file = dir.join(format!("{:0width$}.json", seq, width = HANDOFF_SEQ_WIDTH));
        let raw = match read_json(&file) {
            ReadJson::Missing => continue,
            ReadJson::Corrupt => return Err(Exotic), // readJson warns with the V8 message
            ReadJson::Parsed(v) => crate::verbs::reservations::js_numberify(&v)?,
        };
        let Value::Object(mut m) = raw else { continue };
        let kind = normalize_handoff_kind(m.get("kind"));
        m.insert("kind".into(), json!(kind));
        m.insert("seq".into(), json!(seq));
        records.push(m);
    }
    records.sort_by_key(|r| r.get("seq").and_then(Value::as_i64).unwrap_or(0));
    Ok(records)
}

/// Number.parseInt(s, 10) restricted to the plain-decimal shapes a mailbox
/// filename can carry; NaN (→ skip) for everything else.
fn js_parse_int(s: &str) -> Option<i64> {
    let t = js_trim(s);
    let (sign, rest) = match t.strip_prefix('-') {
        Some(r) => (-1i64, r),
        None => (1i64, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok().map(|n| sign * n)
}

fn record_seq(record: &Map<String, Value>) -> i64 {
    record.get("seq").and_then(Value::as_i64).unwrap_or(0)
}

/// state.mjs newestOpenHandoffMailboxRecord.
pub(crate) fn newest_open_handoff_mailbox_record(
    root: &Path,
    workflow_id: &str,
    target_role: Option<&str>,
) -> Ex<Option<Map<String, Value>>> {
    let role = normalize_target_role(target_role);
    let records = list_handoff_mailbox(root, workflow_id)?;
    for record in records.iter().rev() {
        if !matches!(record.get("status"), Some(Value::String(s)) if s == "open") {
            continue;
        }
        if normalize_target_role_val(record.get("target_role")) != role {
            continue;
        }
        return Ok(Some(record.clone()));
    }
    Ok(None)
}

/// A previous-cell id Node's path.join treats as a plain filename.
fn cell_path_modelable(id: &str) -> bool {
    !(id.contains(':') || id.starts_with('/') || id.starts_with('\\') || id.contains('\0'))
}

/// state.mjs writeMailboxHandoff. `input` is the CLI-built record (kind first,
/// then the optional fields, in bee.mjs's own insertion order); `target_role`
/// is passed separately because bee.mjs spreads it on last. Every precondition
/// is READ before the single write; the whole write runs under
/// `handoff:<workflow-id>`.
pub(crate) fn write_mailbox_handoff(
    root: &Path,
    workflow_id: &str,
    input: &Map<String, Value>,
    target_role: Option<&str>,
) -> Result<Map<String, Value>, Err2> {
    let wf_id_s = require_handoff_workflow_id(workflow_id)?;
    let kind = match input.get("kind") {
        Some(Value::String(s)) if s == "planned-next" || s == "pause" => s.clone(),
        other => {
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: --kind must be \"planned-next\" or \"pause\" (got {}) — D1 requires an explicit kind, never guessed. FIX: pass one of the two handoff kinds.",
                match other {
                    Some(v) => jsjson::stringify(v),
                    None => "undefined".to_string(),
                }
            )));
        }
    };
    let role = normalize_target_role(target_role);
    let now = now_iso();

    // `fields` — the per-kind body, in Node's exact key order.
    let mut fields: Map<String, Value> = Map::new();
    if kind == "pause" {
        for (k, v) in input {
            if k != "kind" && k != "target_role" && k != "from_session" {
                fields.insert(k.clone(), v.clone());
            }
        }
        fields.insert("kind".into(), json!("pause"));
        // `from_session` is never supplied by the CLI, so this is always null.
        let from = match input.get("from_session") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => json!(js_trim(s)),
            _ => Value::Null,
        };
        fields.insert("from_session".into(), from);
    } else {
        let get_trim = |key: &str| -> String {
            match input.get(key) {
                Some(Value::String(s)) => js_trim(s).to_string(),
                _ => String::new(),
            }
        };
        let writer_session = get_trim("writer_session");
        let previous_cell = get_trim("previous_cell");
        let next_cell = get_trim("next_cell");
        if writer_session.is_empty() || previous_cell.is_empty() || next_cell.is_empty() {
            return Err(Err2::Msg(
                "writeMailboxHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.".to_string(),
            ));
        }
        if !cell_path_modelable(&previous_cell) {
            return Err(Err2::Ex);
        }
        let previous = match read_json(
            &root.join(".bee").join("cells").join(format!("{previous_cell}.json")),
        ) {
            ReadJson::Missing => None,
            ReadJson::Corrupt => return Err(Err2::Ex), // readJson warns (V8 bytes)
            ReadJson::Parsed(v) => Some(crate::verbs::reservations::js_numberify(&v)?),
        };
        let capped = previous
            .as_ref()
            .map(|v| truthy(v) && matches!(jget(v, "status"), Some(Value::String(s)) if s == "capped"))
            .unwrap_or(false);
        if !capped {
            let status_disp = match previous.as_ref().and_then(|v| jget(v, "status")) {
                None | Some(Value::Null) => "missing".to_string(),
                Some(v) => js_disp(v),
            };
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: refused — previous cell \"{previous_cell}\" is not capped (found status \"{status_disp}\"). A planned-next handoff may only follow a capped cell. FIX: finish \"{previous_cell}\" first (bee.mjs cells finish), then retry."
            )));
        }
        let claim = read_claim(root, &next_cell)?;
        let owned = claim
            .as_ref()
            .map(|c| {
                matches!(jget(c, "session"), Some(Value::String(s)) if *s == writer_session)
            })
            .unwrap_or(false);
        if !owned {
            let found = match &claim {
                None => "no claim".to_string(),
                Some(c) => format!("owner \"{}\"", js_disp_opt(jget(c, "session"))),
            };
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: refused — next cell \"{next_cell}\" has no claim owned by writer session \"{writer_session}\" (found {found}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim \"{next_cell}\" as session \"{writer_session}\" first (claims.mjs claimCellFile), then retry."
            )));
        }
        let claim_epoch = match claim.as_ref().and_then(|c| jget(c, "fence_epoch")) {
            Some(Value::Number(n)) => Value::Number(n.clone()),
            _ => json!(1),
        };
        for (k, v) in input {
            if !matches!(
                k.as_str(),
                "kind" | "target_role" | "writer_session" | "from_session" | "previous_cell" | "next_cell"
            ) {
                fields.insert(k.clone(), v.clone());
            }
        }
        fields.insert("kind".into(), json!("planned-next"));
        fields.insert("writer_session".into(), json!(writer_session));
        fields.insert("from_session".into(), json!(writer_session));
        fields.insert("previous_cell".into(), json!(previous_cell));
        fields.insert("next_cell".into(), json!(next_cell));
        fields.insert("claim_epoch".into(), claim_epoch);
    }

    let guard = acquire_named_lock(root, &format!("handoff:{wf_id_s}"))?;
    let out = (|| -> Result<Map<String, Value>, Err2> {
        let existing = list_handoff_mailbox(root, &wf_id_s)?;
        let seq = existing.last().map(|r| record_seq(r) + 1).unwrap_or(1);
        // Auto-clear the previous OPEN record for this SAME (workflow, role).
        if let Some(prior) = newest_open_handoff_mailbox_record(root, &wf_id_s, target_role)? {
            let prior_seq = record_seq(&prior);
            let mut cleared = prior;
            cleared.shift_remove("seq");
            cleared.insert("status".into(), json!("cleared"));
            cleared.insert("cleared_at".into(), json!(now));
            write_json_atomic(
                &handoff_record_path(root, &wf_id_s, prior_seq)?,
                &Value::Object(cleared),
            )
            .map_err(|_| Err2::Ex)?;
        }
        let mut record: Map<String, Value> = Map::new();
        record.insert(
            "id".into(),
            json!(format!("{wf_id_s}-{:0width$}", seq, width = HANDOFF_SEQ_WIDTH)),
        );
        record.insert("workflow_id".into(), json!(wf_id_s));
        record.insert(
            "target_role".into(),
            role.as_ref().map(|r| json!(r)).unwrap_or(Value::Null),
        );
        record.insert("status".into(), json!("open"));
        record.insert("written_at".into(), json!(now));
        for (k, v) in &fields {
            record.insert(k.clone(), v.clone());
        }
        write_json_atomic(&handoff_record_path(root, &wf_id_s, seq)?, &Value::Object(record.clone()))
            .map_err(|_| Err2::Ex)?;
        let mut returned = record;
        returned.insert("seq".into(), json!(seq));
        Ok(returned)
    })();
    drop(guard);
    out
}

/// `record[key] = value` where `None` is JS `undefined` — the key is dropped
/// by JSON.stringify rather than written as null.
fn set_or_drop(record: &mut Map<String, Value>, key: &str, value: &Option<Value>) {
    match value {
        Some(v) => {
            record.insert(key.to_string(), v.clone());
        }
        None => {
            record.shift_remove(key);
        }
    }
}

pub(crate) enum MailboxAdopt {
    Fail { reason: String },
    Ok {
        claim: Option<Value>,
        previous_owner: Option<Value>,
        next_cell: String,
        workflow_id: String,
        seq: i64,
    },
}

/// state.mjs adoptMailboxHandoff — newest open-OR-adopted record for
/// (workflow, role); adoptClaim then mark 'cleared', with the crash-between
/// self-heal. Runs under `handoff:<workflow-id>`.
pub(crate) fn adopt_mailbox_handoff(
    root: &Path,
    workflow_id: &str,
    session_id: &str,
    target_role: Option<&str>,
) -> Result<MailboxAdopt, Err2> {
    let wf_id_s = require_handoff_workflow_id(workflow_id)?;
    let role = normalize_target_role(target_role);
    let guard = acquire_named_lock(root, &format!("handoff:{wf_id_s}"))?;
    let out = (|| -> Result<MailboxAdopt, Err2> {
        let records = list_handoff_mailbox(root, &wf_id_s)?;
        let mut candidate: Option<Map<String, Value>> = None;
        for record in records.iter().rev() {
            if normalize_target_role_val(record.get("target_role")) != role {
                continue;
            }
            match record.get("status") {
                Some(Value::String(s)) if s == "open" || s == "adopted" => {
                    candidate = Some(record.clone());
                    break;
                }
                _ => {}
            }
        }
        let Some(candidate) = candidate else {
            let role_note = role.as_ref().map(|r| format!(" (role \"{r}\")")).unwrap_or_default();
            return Ok(MailboxAdopt::Fail {
                reason: format!(
                    "no open handoff in workflow \"{wf_id_s}\"'s mailbox{role_note} to adopt."
                ),
            });
        };
        if !matches!(candidate.get("kind"), Some(Value::String(s)) if s == "planned-next") {
            return Ok(MailboxAdopt::Fail {
                reason: format!(
                    "handoff kind \"{}\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).",
                    js_disp_opt(candidate.get("kind"))
                ),
            });
        }
        let next_cell = match candidate.get("next_cell") {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        if next_cell.is_empty() {
            return Ok(MailboxAdopt::Fail {
                reason: "planned-next handoff has no next_cell to adopt.".to_string(),
            });
        }
        let seq = record_seq(&candidate);
        let mut rest = candidate.clone();
        rest.shift_remove("seq");
        let was_open = matches!(candidate.get("status"), Some(Value::String(s)) if s == "open");
        // `candidate.adopted_previous_owner ?? null` — a definite null (the key
        // IS emitted). The open branch below overwrites it with adoptClaim's
        // own `previous_owner`, which may be undefined (key DROPPED), hence
        // Option<Value> rather than a bare Value.
        let mut previous_owner: Option<Value> =
            Some(candidate.get("adopted_previous_owner").cloned().unwrap_or(Value::Null));
        let claim: Option<Value>;
        if was_open {
            match adopt_claim(root, &next_cell, session_id)? {
                AdoptOutcome::Fail { reason } => return Ok(MailboxAdopt::Fail { reason }),
                AdoptOutcome::Adopted { claim: c, previous_owner: prev } => {
                    previous_owner = prev;
                    let epoch = c.get("fence_epoch").cloned().unwrap_or(Value::Null);
                    claim = Some(Value::Object(c));
                    let mut adopted = rest.clone();
                    adopted.insert("status".into(), json!("adopted"));
                    adopted.insert("claim_epoch".into(), epoch);
                    adopted.insert("adopted_by".into(), json!(session_id));
                    adopted.insert("adopted_at".into(), json!(now_iso()));
                    set_or_drop(&mut adopted, "adopted_previous_owner", &previous_owner);
                    write_json_atomic(
                        &handoff_record_path(root, &wf_id_s, seq)?,
                        &Value::Object(adopted),
                    )
                    .map_err(|_| Err2::Ex)?;
                }
            }
        } else {
            // Self-heal: the claim already moved on a crashed-before-clear call.
            claim = read_claim(root, &next_cell)?;
        }
        let claim_epoch = match claim.as_ref().and_then(|c| jget(c, "fence_epoch")) {
            Some(Value::Number(n)) => Value::Number(n.clone()),
            _ => candidate.get("claim_epoch").cloned().unwrap_or(Value::Null),
        };
        let adopted_at = match candidate.get("adopted_at") {
            None | Some(Value::Null) => json!(now_iso()),
            Some(v) => v.clone(),
        };
        let mut cleared = rest;
        cleared.insert("status".into(), json!("cleared"));
        cleared.insert("claim_epoch".into(), claim_epoch);
        cleared.insert("adopted_by".into(), json!(session_id));
        cleared.insert("adopted_at".into(), adopted_at);
        set_or_drop(&mut cleared, "adopted_previous_owner", &previous_owner);
        cleared.insert("cleared_at".into(), json!(now_iso()));
        write_json_atomic(&handoff_record_path(root, &wf_id_s, seq)?, &Value::Object(cleared))
            .map_err(|_| Err2::Ex)?;
        Ok(MailboxAdopt::Ok {
            claim,
            previous_owner,
            next_cell,
            workflow_id: wf_id_s.clone(),
            seq,
        })
    })();
    drop(guard);
    out
}

// ─── gate stamps + workflow listing order (bee.mjs) ────────────────────────

/// bee.mjs findGateStamp — normalizes the single/array/absent shapes to the
/// one entry (if any) naming `name`.
pub(crate) fn find_gate_stamp<'a>(
    stamps: &'a [(String, Value)],
    name: &str,
) -> Option<&'a (String, Value)> {
    stamps.iter().find(|(n, _)| n == name)
}

/// The `gates` half of writeLaneRecordThroughProjection /
/// writeStateRecordThroughProjection's updateWorkflowAssumingLock patch.
pub(crate) fn gates_patch_from_record(
    updated: &Map<String, Value>,
    stamps: &[(String, Value)],
) -> Value {
    let mut gates = Map::new();
    for name in GATE_NAMES {
        let mut entry = Map::new();
        let approved = updated
            .get("approved_gates")
            .filter(|v| truthy(v))
            .and_then(|v| jget(v, name))
            .map(|v| v == &Value::Bool(true))
            .unwrap_or(false);
        entry.insert("approved".into(), Value::Bool(approved));
        if let Some((_, rev)) = find_gate_stamp(stamps, name) {
            entry.insert("approved_for_plan_rev".into(), rev.clone());
        }
        gates.insert(name.to_string(), Value::Object(entry));
    }
    Value::Object(gates)
}

/// bee.mjs workflowsListSort — created_at descending (newest first), stable.
pub(crate) fn workflows_list_sort(records: &mut [Map<String, Value>]) -> Ex<()> {
    let mut keyed: Vec<f64> = Vec::with_capacity(records.len());
    for r in records.iter() {
        let stamp = match r.get("created_at") {
            Some(v) if truthy(v) => date_parse_val(Some(v))?.unwrap_or(f64::NAN),
            _ => 0.0,
        };
        keyed.push(stamp);
    }
    let mut idx: Vec<usize> = (0..records.len()).collect();
    idx.sort_by(|&a, &b| {
        let d = keyed[b] - keyed[a];
        if d.is_nan() || d == 0.0 {
            // A comparator returning NaN is treated as 0 by V8's sort.
            std::cmp::Ordering::Equal
        } else if d < 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    let sorted: Vec<Map<String, Value>> = idx.into_iter().map(|i| records[i].clone()).collect();
    records.clone_from_slice(&sorted);
    Ok(())
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    fn write_workflow(root: &Path, id: &str, body: Value) {
        let dir = workflows_dir(root).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), serde_json::to_string(&body).unwrap()).unwrap();
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn write_lane_file(root: &Path, feature: &str, content: &str) {
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        std::fs::write(lanes_dir(root).join(format!("{feature}.json")), content).unwrap();
    }

    fn read_back(file: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
    }

    // ── workflow-store.mjs ────────────────────────────────────────────────

    #[test]
    fn merge_gates_defaults_overlays_and_keeps_unknown_names() {
        let merged = merge_gates(
            None,
            Some(&json!({"execution": {"approved": true, "approved_for_plan_rev": 2},
                         "fifth": {"approved": true}})),
        );
        assert_eq!(
            jsjson::stringify(&merged),
            r#"{"context":{"approved":false,"approved_for_plan_rev":null},"shape":{"approved":false,"approved_for_plan_rev":null},"execution":{"approved":true,"approved_for_plan_rev":2},"review":{"approved":false,"approved_for_plan_rev":null},"fifth":{"approved":true,"approved_for_plan_rev":null}}"#
        );
        // A patch carrying only `approved` PRESERVES the base's rev stamp.
        let base = merge_gates(None, Some(&json!({"execution": {"approved": true, "approved_for_plan_rev": 7}})));
        let next = merge_gates(Some(&base), Some(&json!({"execution": {"approved": true}})));
        assert_eq!(jget(&next, "execution").unwrap()["approved_for_plan_rev"], json!(7));
    }

    #[test]
    fn read_workflow_record_merges_defaults_and_refuses_id_mismatch() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let rec = read_workflow_record(tmp.path(), "wf-1").ok().unwrap();
        assert_eq!(rec.get("phase"), Some(&json!("idle")));
        assert_eq!(rec.get("plan_rev"), Some(&json!(0)));
        assert_eq!(rec.get("status"), Some(&json!("active")));
        assert_eq!(rec.get("route"), Some(&Value::Null));
        assert!(rec.contains_key("gates"));
        write_workflow(tmp.path(), "wf-2", json!({"id":"other","feature":"f2"}));
        match read_workflow_record(tmp.path(), "wf-2") {
            Err(WfSkip::Reason(m)) => assert!(m.contains("does not match the requested workflow \"wf-2\"")),
            _ => panic!("expected the id-mismatch skip"),
        }
        // Missing record: WORKFLOW_MISSING reason.
        std::fs::create_dir_all(workflows_dir(tmp.path()).join("wf-3")).unwrap();
        match read_workflow_record(tmp.path(), "wf-3") {
            Err(WfSkip::Reason(m)) => assert!(m.starts_with("readWorkflow: no workflow record at")),
            _ => panic!("expected WORKFLOW_MISSING"),
        }
    }

    // ── listWorkflows skip tolerance (R6 blocker, now native) ─────────────

    /// The three ordinary skips, each with the reason bytes `read_workflow_
    /// record` hands `console.warn`. Named here so the warn-stream tests and
    /// the reason-shape tests cannot drift apart.
    fn seed_the_three_ordinary_skips(root: &Path) -> Vec<String> {
        let dir = workflows_dir(root);
        // (1) directory present, no state.json → WORKFLOW_MISSING
        std::fs::create_dir_all(dir.join("wf-missing")).unwrap();
        // (2) present but not a JSON object
        std::fs::create_dir_all(dir.join("wf-array")).unwrap();
        std::fs::write(dir.join("wf-array").join("state.json"), "[1,2]").unwrap();
        // (3) present, an object, but its id names someone else
        write_workflow(root, "wf-wrongid", json!({"id":"somebody-else","feature":"f"}));
        vec![
            format!(
                "readWorkflow: no workflow record at \"{}\". FIX: createWorkflow first, or check the id.",
                workflow_state_path(root, "wf-missing").display()
            ),
            format!(
                "readWorkflow: \"{}\" exists but is not a JSON object (found an array).",
                workflow_state_path(root, "wf-array").display()
            ),
            format!(
                "readWorkflow: \"{}\" exists but its id field (\"somebody-else\") does not match the \
requested workflow \"wf-wrongid\" — never trusted. FIX: inspect/restore the file (e.g. \"git \
checkout -- {}\").",
                workflow_state_path(root, "wf-wrongid").display(),
                format!(".bee{0}runtime{0}workflows{0}wf-wrongid{0}state.json", MAIN_SEPARATOR)
            ),
        ]
    }

    #[test]
    fn list_workflows_skips_the_three_ordinary_shapes_and_keeps_the_readable_ones() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        assert_eq!(ok(list_workflows(tmp.path())).len(), 1);

        let reasons = seed_the_three_ordinary_skips(tmp.path());
        // Every skip is tolerated: the listing still returns the good record.
        let listed = ok(list_workflows(tmp.path()));
        assert_eq!(listed.len(), 1, "the readable record survives every skip");
        assert_eq!(wf_id(&listed[0]), "wf-1");

        // ...and each skip reason is the exact WorkflowStoreError message.
        for (id, reason) in ["wf-missing", "wf-array", "wf-wrongid"].iter().zip(&reasons) {
            match read_workflow_record(tmp.path(), id) {
                Err(WfSkip::Reason(m)) => assert_eq!(&m, reason, "reason bytes for {id}"),
                _ => panic!("expected an ordinary (native) skip for {id}"),
            }
        }

        // A non-directory entry is skipped SILENTLY by Node — no warn at all.
        std::fs::write(workflows_dir(tmp.path()).join("README"), "x").unwrap();
        assert_eq!(ok(list_workflows(tmp.path())).len(), 1);
    }

    #[test]
    fn the_warn_line_is_console_warns_own_shape() {
        let tmp = tmp_root();
        let reasons = seed_the_three_ordinary_skips(tmp.path());
        assert_eq!(
            skip_warn_line("wf-missing", &reasons[0]),
            format!(
                "listWorkflows: skipping unreadable workflow \"wf-missing\" — readWorkflow: no \
workflow record at \"{}\". FIX: createWorkflow first, or check the id.",
                workflow_state_path(tmp.path(), "wf-missing").display()
            )
        );
    }

    #[test]
    fn only_the_two_v8_worded_arms_still_delegate() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        // (1) unparseable JSON — the reason embeds V8's own parse message.
        let dir = workflows_dir(tmp.path()).join("wf-badjson");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), "{not json").unwrap();
        assert!(
            matches!(read_workflow_record(tmp.path(), "wf-badjson"), Err(WfSkip::Delegate)),
            "a V8 parse message must stay Node's"
        );
        assert!(list_workflows(tmp.path()).is_err(), "and it routes the whole call back");

        // (2) present-but-unreadable — the reason embeds a libuv errno string.
        // A directory in place of state.json is the portable way to reach it.
        std::fs::remove_dir_all(&dir).unwrap();
        let d2 = workflows_dir(tmp.path()).join("wf-eisdir");
        std::fs::create_dir_all(d2.join("state.json")).unwrap();
        assert!(
            matches!(read_workflow_record(tmp.path(), "wf-eisdir"), Err(WfSkip::Delegate)),
            "an errno-worded refusal must stay Node's"
        );
        assert!(list_workflows(tmp.path()).is_err());
    }

    #[test]
    fn a_delegating_scan_emits_no_warn_before_it_bails() {
        // The pre-pass contract: a directory holding BOTH an ordinary skip and
        // a delegating one must produce zero output, or the Node re-run would
        // print that warn a second time. Proven structurally — the classifier
        // is run to completion and only then are the lines emitted — and
        // observably by the child-process byte-diff in
        // scripts (see the cell's harness), which sees an empty stderr here.
        let tmp = tmp_root();
        seed_the_three_ordinary_skips(tmp.path()); // three warnable entries
        let bad = workflows_dir(tmp.path()).join("wf-badjson");
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(bad.join("state.json"), "{not json").unwrap();
        assert!(list_workflows(tmp.path()).is_err());
    }

    #[test]
    fn update_assuming_lock_protects_identity_and_merges_gates() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","created_at":"2026-01-01T00:00:00.000Z",
                   "phase":"planning","plan_rev":2,
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":2}}}),
        );
        let mut patch = Map::new();
        patch.insert("id".into(), json!("hacked"));
        patch.insert("feature".into(), json!("hacked"));
        patch.insert("created_at".into(), json!("hacked"));
        patch.insert("phase".into(), json!("swarming"));
        patch.insert("gates".into(), json!({"execution":{"approved":false}}));
        let next = ok(update_workflow_assuming_lock(tmp.path(), "wf-1", patch));
        assert_eq!(next.get("id"), Some(&json!("wf-1")));
        assert_eq!(next.get("feature"), Some(&json!("f1")));
        assert_eq!(next.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        assert_eq!(next.get("phase"), Some(&json!("swarming")));
        // approved flipped; the rev stamp survived (mergeGates one level deep).
        let gates = next.get("gates").unwrap();
        assert_eq!(jget(gates, "execution").unwrap()["approved"], json!(false));
        // JSON.parse yields JS numbers, so js_numberify makes every parsed
        // number an f64 — jsjson prints 2.0 back as "2", byte-identically.
        assert_eq!(jget(gates, "execution").unwrap()["approved_for_plan_rev"], json!(2.0));
        // …and it landed on disk.
        let on_disk = read_back(&workflow_state_path(tmp.path(), "wf-1"));
        assert_eq!(on_disk["phase"], json!("swarming"));
    }

    #[test]
    fn update_assuming_lock_refuses_bad_status() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let mut patch = Map::new();
        patch.insert("status".into(), json!("zombie"));
        match update_workflow_assuming_lock(tmp.path(), "wf-1", patch) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "updateWorkflowAssumingLock: status must be one of active/paused/closed (got \"zombie\")."
            ),
            _ => panic!("expected the status refusal"),
        }
    }

    #[test]
    fn workflow_lock_name_matches_node_and_is_per_id() {
        let tmp = tmp_root();
        let g = ok(acquire_workflow_lock(tmp.path(), "wf-a"));
        // A DIFFERENT id is a distinct lock file (sanitizeLockName hashes ':').
        let g2 = ok(acquire_workflow_lock(tmp.path(), "wf-b"));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-a").exists());
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-b").exists());
        assert_ne!(
            lock::lock_file_path(tmp.path(), "workflow:wf-a"),
            lock::lock_file_path(tmp.path(), "workflow:wf-b")
        );
        drop(g);
        drop(g2);
        assert!(!lock::lock_file_path(tmp.path(), "workflow:wf-a").exists());
        assert_eq!(projection_lock_name(true, Some("f1")), "lane:f1");
        assert_eq!(projection_lock_name(false, Some("f1")), "state");
        assert_eq!(projection_lock_name(true, None), "state");
    }

    // ── projections ───────────────────────────────────────────────────────

    #[test]
    fn gates_project_plan_rev_effective_approval() {
        let gates = json!({
            "context": {"approved": true, "approved_for_plan_rev": null},
            "shape": {"approved": true},
            "execution": {"approved": true, "approved_for_plan_rev": 3},
            "review": {"approved": false},
        });
        // plan_rev 3: the stamped execution gate is effective.
        assert_eq!(
            jsjson::stringify(&workflow_gates_to_approved_gates(Some(&gates), Some(&json!(3)))),
            r#"{"context":true,"shape":true,"execution":true,"review":false}"#
        );
        // plan_rev 4 (a bump): execution goes ineffective, the rest are immune.
        assert_eq!(
            jsjson::stringify(&workflow_gates_to_approved_gates(Some(&gates), Some(&json!(4)))),
            r#"{"context":true,"shape":true,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn newest_active_workflow_skips_closed_and_terminal_and_breaks_ties_by_id() {
        let mk = |id: &str, status: &str, phase: &str, at: &str| -> Map<String, Value> {
            match json!({"id":id,"feature":id,"status":status,"phase":phase,"created_at":at}) {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        };
        let wfs = vec![
            mk("wf-a", "active", "planning", "2026-01-01T00:00:00.000Z"),
            mk("wf-z", "active", "planning", "2026-01-01T00:00:00.000Z"),
            mk("wf-newer", "closed", "planning", "2026-05-01T00:00:00.000Z"),
            mk("wf-term", "active", "compounding-complete", "2026-06-01T00:00:00.000Z"),
        ];
        let picked = ok(pick_newest_active_workflow(&wfs)).unwrap();
        // Same created_at → id DESCENDING wins.
        assert_eq!(picked.get("id"), Some(&json!("wf-z")));
        let none: Vec<Map<String, Value>> = vec![mk("wf-c", "closed", "idle", "2026-01-01T00:00:00.000Z")];
        assert!(ok(pick_newest_active_workflow(&none)).is_none());
    }

    /// The C1 fallback and the two authoritative branches, pinned against the
    /// EXACT fixtures src/hooks/state_sync.rs's own projection tests use
    /// (`rebuild_with_zero_workflows_is_overrides_only`,
    /// `rebuild_idle_bootstrap_adopts_newest_active_workflow`,
    /// `rebuild_feature_match_projects_gates_with_plan_rev`). state_sync.rs's
    /// copies of these functions are module-private and that file is outside
    /// this cell's touchable set, so this test is the standing proof that the
    /// two ports agree — the only difference being the overrides the hook
    /// always passes and the CLI never does (with none, a non-authoritative
    /// branch writes nothing at all).
    #[test]
    fn agrees_with_state_sync_port_on_shared_fixtures() {
        // (1) zero workflow records — pure no-op, D1 fields untouched.
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"schema_version":"1.0","phase":"swarming","feature":"f1","extra":42}"#,
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["extra"], json!(42));
        assert!(out.get("cells").is_none(), "no overrides → nothing added");

        // (2) idle bootstrap adopts the newest ACTIVE workflow; a gate stamped
        // for a rev the record is not at projects false.
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"idle"}"#);
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
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["feature"], json!("f-new"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(
            out["approved_gates"],
            json!({"context":false,"shape":false,"execution":false,"review":false})
        );

        // (3) feature-matched branch, pass-through fields survive.
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"phase":"planning","feature":"f1","workers":["w"]}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"lane","plan_rev":3,"summary":"sum","next_action":"next",
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"context":{"approved":true,"approved_for_plan_rev":null},
                            "execution":{"approved":true,"approved_for_plan_rev":3}}}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["mode"], json!("lane"));
        assert_eq!(out["summary"], json!("sum"));
        assert_eq!(out["workers"], json!(["w"]));
        assert_eq!(
            out["approved_gates"],
            json!({"context":true,"shape":false,"execution":true,"review":false})
        );
    }

    #[test]
    fn state_projection_is_a_noop_when_no_live_workflow_names_the_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"swarming","feature":"f1"}"#);
        write_workflow(
            tmp.path(),
            "wf-other",
            json!({"id":"wf-other","feature":"other","status":"active","phase":"idle",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"), "untouched");
        // A CLOSED workflow naming the feature is also no authority.
        write_workflow(
            tmp.path(),
            "wf-closed",
            json!({"id":"wf-closed","feature":"f1","status":"closed","phase":"idle",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        ok(rebuild_state_projection(tmp.path()));
        assert_eq!(
            read_back(&tmp.path().join(".bee").join("state.json"))["phase"],
            json!("swarming")
        );
    }

    #[test]
    fn lane_projection_rebuilds_from_the_record_and_keeps_ad_hoc_fields() {
        let tmp = tmp_root();
        write_lane_file(
            tmp.path(),
            "f1",
            r#"{"schema_version":"1.0","feature":"f1","phase":"idle","created_at":"2026-01-01T00:00:00.000Z","last_scribing_run":{"feature":"f1"}}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"S","next_action":"N",
                   "created_at":"2026-03-03T00:00:00.000Z",
                   "gates":{"shape":{"approved":true,"approved_for_plan_rev":1}}}),
        );
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(lane.get("summary"), Some(&json!("S")));
        // created_at keeps the LANE's original identity timestamp.
        assert_eq!(lane.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        // Ad hoc lane-only fields pass through.
        assert!(lane.contains_key("last_scribing_run"));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        // No live workflow → no-op, existing record returned.
        let none = ok(rebuild_lane_projection(tmp.path(), "nolane"));
        assert!(none.is_none());
    }

    #[test]
    fn lane_projection_seeds_created_at_from_the_record_when_no_file_exists() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"fresh","status":"active","phase":"planning",
                   "created_at":"2026-04-04T00:00:00.000Z"}),
        );
        let lane = ok(rebuild_lane_projection(tmp.path(), "fresh")).unwrap();
        let keys: Vec<&str> = lane.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "schema_version",
                "feature",
                "mode",
                "phase",
                "approved_gates",
                "summary",
                "next_action",
                "created_at"
            ]
        );
        assert_eq!(lane.get("created_at"), Some(&json!("2026-04-04T00:00:00.000Z")));
    }

    // ── lanes ─────────────────────────────────────────────────────────────

    #[test]
    fn lane_strict_refuses_corrupt_and_returns_none_for_missing() {
        let tmp = tmp_root();
        assert!(ok(read_lane_strict(tmp.path(), "nope")).is_none());
        write_lane_file(tmp.path(), "bad", "{not json");
        match read_lane_strict(tmp.path(), "bad") {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("exists but is corrupt (not a JSON object naming feature \"bad\")"));
                assert!(!m.contains("Unexpected token"), "no V8 text in the refusal");
            }
            _ => panic!("expected the corrupt refusal"),
        }
        // A record naming a DIFFERENT feature is corrupt too.
        write_lane_file(tmp.path(), "mismatch", r#"{"feature":"other"}"#);
        assert!(matches!(read_lane_strict(tmp.path(), "mismatch"), Err(Err2::Msg(_))));
        // Bad names throw requireLaneFeature's own message.
        match read_lane_strict(tmp.path(), "a/b") {
            Err(Err2::Msg(m)) => assert_eq!(m, "lane feature must be a plain id (no path separators)."),
            _ => panic!("expected the name refusal"),
        }
        // Healthy record merges the per-feature defaults.
        write_lane_file(tmp.path(), "f1", r#"{"feature":"f1","phase":"swarming"}"#);
        let rec = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(rec.get("phase"), Some(&json!("swarming")));
        assert_eq!(rec.get("schema_version"), Some(&json!("1.0")));
        assert_eq!(
            jsjson::stringify(rec.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn write_lane_round_trips_through_the_feature_filename() {
        let tmp = tmp_root();
        let mut lane = default_lane_record("f1");
        lane.insert("phase".into(), json!("reviewing"));
        ok(write_lane(tmp.path(), &lane));
        let back = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(back.get("phase"), Some(&json!("reviewing")));
    }

    // ── handoff mailbox ───────────────────────────────────────────────────

    fn capped_cell_and_claim(root: &Path, cell: &str, session: &str) {
        let cells = root.join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        let claims = root.join(".bee").join("claims");
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join(format!("{cell}.json")),
            format!(r#"{{"cell":"{cell}","session":"{session}","fence_epoch":1}}"#),
        )
        .unwrap();
    }

    fn planned_next_input() -> Map<String, Value> {
        let mut input = Map::new();
        input.insert("kind".into(), json!("planned-next"));
        input.insert("feature".into(), json!("f1"));
        input.insert("writer_session".into(), json!("sess-w"));
        input.insert("previous_cell".into(), json!("prev"));
        input.insert("next_cell".into(), json!("next"));
        input
    }

    #[test]
    fn mailbox_write_assigns_seq_clears_the_prior_open_record_and_scopes_by_role() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        let input = planned_next_input();
        let r1 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        assert_eq!(r1.get("seq"), Some(&json!(1)));
        assert_eq!(r1.get("id"), Some(&json!("wf-1-0001")));
        assert_eq!(r1.get("status"), Some(&json!("open")));
        assert_eq!(r1.get("claim_epoch"), Some(&json!(1.0))); // parsed → f64, prints "1"
        assert_eq!(r1.get("from_session"), Some(&json!("sess-w")));
        // Node's exact key order for a planned-next mailbox record.
        let keys: Vec<&str> = r1.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "id", "workflow_id", "target_role", "status", "written_at", "feature", "kind",
                "writer_session", "from_session", "previous_cell", "next_cell", "claim_epoch",
                "seq"
            ]
        );
        // A second write to the SAME (workflow, role) clears the first.
        let r2 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        assert_eq!(r2.get("seq"), Some(&json!(2)));
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].get("status"), Some(&json!("cleared")));
        assert_eq!(all[1].get("status"), Some(&json!("open")));
        // A DIFFERENT role gets its own slot and never touches the other.
        let r3 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, Some("reviewer")));
        assert_eq!(r3.get("target_role"), Some(&json!("reviewer")));
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all[1].get("status"), Some(&json!("open")), "unscoped slot survives");
        let newest_unscoped = ok(newest_open_handoff_mailbox_record(tmp.path(), "wf-1", None)).unwrap();
        assert_eq!(record_seq(&newest_unscoped), 2);
        let newest_reviewer =
            ok(newest_open_handoff_mailbox_record(tmp.path(), "wf-1", Some("reviewer"))).unwrap();
        assert_eq!(record_seq(&newest_reviewer), 3);
    }

    #[test]
    fn mailbox_write_refuses_uncapped_previous_and_unowned_claim() {
        let tmp = tmp_root();
        let input = planned_next_input();
        match write_mailbox_handoff(tmp.path(), "wf-1", &input, None) {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("previous cell \"prev\" is not capped (found status \"missing\")"))
            }
            _ => panic!("expected the uncapped refusal"),
        }
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        match write_mailbox_handoff(tmp.path(), "wf-1", &input, None) {
            Err(Err2::Msg(m)) => assert!(m.contains("(found no claim)")),
            _ => panic!("expected the unowned-claim refusal"),
        }
        // Nothing was written on either refusal.
        assert!(ok(list_handoff_mailbox(tmp.path(), "wf-1")).is_empty());
    }

    #[test]
    fn mailbox_adopt_moves_the_claim_bumps_the_fence_and_clears() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Ok { claim, previous_owner, next_cell, workflow_id, seq } => {
                assert_eq!(next_cell, "next");
                assert_eq!(workflow_id, "wf-1");
                assert_eq!(seq, 1);
                assert_eq!(previous_owner, Some(json!("sess-w")));
                let claim = claim.unwrap();
                assert_eq!(jget(&claim, "session"), Some(&json!("sess-new")));
                assert_eq!(jget(&claim, "fence_epoch"), Some(&json!(2.0)));
            }
            MailboxAdopt::Fail { reason } => panic!("unexpected refusal: {reason}"),
        }
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all[0].get("status"), Some(&json!("cleared")));
        assert_eq!(all[0].get("adopted_by"), Some(&json!("sess-new")));
        // A second adopt finds nothing open/adopted left.
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Fail { reason } => {
                assert_eq!(reason, "no open handoff in workflow \"wf-1\"'s mailbox to adopt.")
            }
            _ => panic!("expected NO_HANDOFF"),
        }
    }

    #[test]
    fn mailbox_adopt_refuses_a_pause_record() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("cell".into(), json!("wip"));
        let rec = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        // Node's exact key order for a pause mailbox record.
        let keys: Vec<&str> = rec.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["id", "workflow_id", "target_role", "status", "written_at", "cell", "kind", "from_session", "seq"]
        );
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "s", None)) {
            MailboxAdopt::Fail { reason } => assert!(reason.starts_with("handoff kind \"pause\" is not \"planned-next\"")),
            _ => panic!("expected NOT_PLANNED_NEXT"),
        }
    }

    #[test]
    fn handoff_projection_picks_the_newest_open_record_and_strips_mailbox_fields() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        write_workflow(tmp.path(), "wf-2", json!({"id":"wf-2","feature":"f2"}));
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        ok(rebuild_handoff_projection(tmp.path()));
        let projected = read_back(&handoff_path(tmp.path()));
        assert_eq!(projected["kind"], json!("planned-next"));
        assert_eq!(projected["next_cell"], json!("next"));
        for stripped in ["seq", "status", "id", "workflow_id", "target_role", "from_session"] {
            assert!(projected.get(stripped).is_none(), "{stripped} must be stripped");
        }
        // Adopting clears the only open record → the legacy file is removed.
        ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None));
        ok(rebuild_handoff_projection(tmp.path()));
        assert!(!handoff_path(tmp.path()).exists());
    }

    #[test]
    fn handoff_projection_is_a_noop_at_zero_workflow_records() {
        let tmp = tmp_root();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(handoff_path(tmp.path()), r#"{"kind":"pause"}"#).unwrap();
        ok(rebuild_handoff_projection(tmp.path()));
        assert!(handoff_path(tmp.path()).exists(), "C1: legacy file untouched");
    }

    #[test]
    fn workflows_list_sort_is_newest_created_first() {
        let mk = |id: &str, at: Value| -> Map<String, Value> {
            match json!({"id": id, "created_at": at}) {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        };
        let mut records = vec![
            mk("a", json!("2026-01-01T00:00:00.000Z")),
            mk("b", json!("2026-05-01T00:00:00.000Z")),
            mk("c", Value::Null),
        ];
        ok(workflows_list_sort(&mut records));
        let ids: Vec<String> = records.iter().map(|r| js_disp_opt(r.get("id"))).collect();
        assert_eq!(ids, ["b", "a", "c"]);
    }

    #[test]
    fn gates_patch_carries_stamps_only_for_the_named_gates() {
        let mut updated = Map::new();
        updated.insert(
            "approved_gates".into(),
            json!({"context": true, "shape": true, "execution": true, "review": false}),
        );
        let stamps = vec![
            ("shape".to_string(), json!(4)),
            ("execution".to_string(), json!(4)),
        ];
        assert_eq!(
            jsjson::stringify(&gates_patch_from_record(&updated, &stamps)),
            r#"{"context":{"approved":true},"shape":{"approved":true,"approved_for_plan_rev":4},"execution":{"approved":true,"approved_for_plan_rev":4},"review":{"approved":false}}"#
        );
        // No stamps at all: every gate carries `approved` only, so mergeGates
        // preserves whatever rev each entry already had.
        assert_eq!(
            jsjson::stringify(&gates_patch_from_record(&updated, &[])),
            r#"{"context":{"approved":true},"shape":{"approved":true},"execution":{"approved":true},"review":{"approved":false}}"#
        );
    }

    // ── R5 test migration: createWorkflow + locking + absent-store ─────────
    //
    // The createWorkflow rows of test_workflow_store.mjs (full-schema write,
    // refusal to overwrite an existing id, refusal when id === feature) now
    // have a Rust counterpart — see § createWorkflow below.
    //
    // The oracle's "listWorkflows … tolerant of an unreadable entry
    // (skip+report, never throws)" row is now ported too — see § listWorkflows
    // skip tolerance above. Only the two V8/errno-worded reasons still
    // delegate; `only_the_two_v8_worded_arms_still_delegate` pins exactly
    // which, so the residue can never quietly widen.

    // ══ createWorkflow (workflow-store.mjs) ════════════════════════════════

    /// Oracle: "createWorkflow writes the full schema and readWorkflow reads
    /// it back unchanged". Create and read must be BYTE-symmetric (mv-4), so
    /// the key order is asserted, not just the values.
    #[test]
    fn create_writes_the_full_schema_and_reads_back_identical() {
        let tmp = tmp_root();
        let root = tmp.path();
        let record = create_workflow(
            root,
            NewWorkflow {
                feature: Some("  billing-refunds  "),
                phase: Some(json!("planning")),
                mode: Some(json!("swarm")),
                plan_rev: Some(json!(2)),
                gates: Some(json!({"shape": {"approved": true}})),
                summary: Some(json!("s")),
                next_action: Some(json!("n")),
                status: Some("paused"),
                id: Some("wf-explicit"),
            },
        )
        .expect("create");

        let keys: Vec<&str> = record.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec![
                "mode", "phase", "plan_rev", "summary", "next_action", "status", "route", "id",
                "feature", "gates", "created_at"
            ],
            "a JS re-assignment keeps a key's original position — only id/feature/gates/created_at append"
        );
        assert_eq!(record["id"], json!("wf-explicit"));
        assert_eq!(record["feature"], json!("billing-refunds"), "the feature slug is trimmed");
        assert_eq!(record["phase"], json!("planning"));
        assert_eq!(record["mode"], json!("swarm"));
        assert_eq!(record["plan_rev"], json!(2));
        assert_eq!(record["status"], json!("paused"));
        assert_eq!(record["route"], Value::Null, "baseWorkflowDefaults' route survives");
        assert!(record["created_at"].as_str().unwrap().ends_with('Z'));
        // mergeGates(defaultGates(), overrides): the override is one level
        // deep over the default entry, and every GATE_NAME is still present.
        assert_eq!(
            jsjson::stringify(&record["gates"]),
            r#"{"context":{"approved":false,"approved_for_plan_rev":null},"shape":{"approved":true,"approved_for_plan_rev":null},"execution":{"approved":false,"approved_for_plan_rev":null},"review":{"approved":false,"approved_for_plan_rev":null}}"#
        );

        // The record is on disk at .bee/runtime/workflows/<id>/state.json,
        // byte-for-byte as a live `node` run of workflow-store.mjs
        // createWorkflow writes it over this exact fixture (ORACLE-PINNED).
        let file = workflow_state_path(root, "wf-explicit");
        let on_disk = std::fs::read_to_string(&file)
            .unwrap()
            .replace(record["created_at"].as_str().unwrap(), "<now>");
        assert_eq!(
            on_disk,
            "{\n  \"mode\": \"swarm\",\n  \"phase\": \"planning\",\n  \"plan_rev\": 2,\n  \"summary\": \"s\",\n  \"next_action\": \"n\",\n  \"status\": \"paused\",\n  \"route\": null,\n  \"id\": \"wf-explicit\",\n  \"feature\": \"billing-refunds\",\n  \"gates\": {\n    \"context\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null\n    },\n    \"shape\": {\n      \"approved\": true,\n      \"approved_for_plan_rev\": null\n    },\n    \"execution\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null\n    },\n    \"review\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null\n    }\n  },\n  \"created_at\": \"<now>\"\n}\n"
        );
        // … and readWorkflowRecord round-trips it with no drift at all.
        let read_back = read_workflow_record(root, "wf-explicit").ok().expect("readable");
        assert_eq!(jsjson::stringify(&Value::Object(read_back)), jsjson::stringify(&Value::Object(record)));
    }

    /// Oracle: "createWorkflow defaults every optional field" — and the
    /// generated id is never the feature slug.
    #[test]
    fn create_defaults_every_optional_field_and_generates_a_wf_prefixed_id() {
        let tmp = tmp_root();
        let root = tmp.path();
        let record = create_workflow(root, NewWorkflow::for_feature("f1")).expect("create");
        assert_eq!(record["phase"], json!("idle"));
        assert_eq!(record["mode"], Value::Null);
        assert_eq!(record["plan_rev"], json!(0));
        assert_eq!(record["summary"], json!(""));
        assert_eq!(record["next_action"], json!(""));
        assert_eq!(record["status"], json!("active"));
        let id = record["id"].as_str().unwrap();
        assert!(id.starts_with("wf-"), "{id}");
        assert_eq!(id.len(), 11, "wf- plus 4 bytes of hex: {id}");
        assert_ne!(id, "f1");
        assert!(record["gates"]["context"]["approved"] == json!(false));

        // Two creates never collide, and both are listed.
        let second = create_workflow(root, NewWorkflow::for_feature("f2")).expect("create");
        assert_ne!(second["id"], record["id"]);
        assert_eq!(ok(list_workflows(root)).len(), 2);
    }

    /// Oracle: "createWorkflow refuses to overwrite an existing record", "…
    /// refuses when id === feature (D1)", plus the id/feature/status
    /// validation ladder. Every refusal's bytes are pinned.
    #[test]
    fn create_refuses_on_every_invalid_shape_and_never_overwrites() {
        let tmp = tmp_root();
        let root = tmp.path();

        let msg = |r: Result<Map<String, Value>, Err2>| match r {
            Err(Err2::Msg(m)) => m,
            Err(Err2::Ex) => panic!("expected a typed refusal, got Exotic"),
            Ok(_) => panic!("expected a refusal"),
        };

        // feature is required — checked AFTER requireWorkflowId, so a valid
        // explicit id plus a blank feature reports the feature refusal.
        for feature in [None, Some(""), Some("   ")] {
            let mut opts = NewWorkflow::for_feature("x");
            opts.feature = feature;
            opts.id = Some("wf-a");
            assert_eq!(msg(create_workflow(root, opts)), "createWorkflow: feature is required.");
        }
        // requireWorkflowId fires first for a path-shaped explicit id.
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("  ");
        assert_eq!(msg(create_workflow(root, opts)), "workflow id is required.");
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("a/b");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "workflow id \"a/b\" must be a plain id (no path separators) — it becomes a directory name under .bee/runtime/workflows/."
        );
        // D1: the id may never be the feature slug.
        let mut opts = NewWorkflow::for_feature("wf-thing");
        opts.id = Some("wf-thing");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "createWorkflow: workflow id \"wf-thing\" must not equal the feature slug \"wf-thing\" — ids are \
generated identifiers, never feature slugs (CONTEXT.md D1: a feature can reopen or run competing \
attempts, so identity must never collide with the human-chosen name). FIX: pass an explicit id distinct \
from the feature, or omit id to let one be generated."
        );
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-a");
        opts.status = Some("archived");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "createWorkflow: status must be one of active/paused/closed (got \"archived\")."
        );
        // Not one of those refusals wrote anything.
        assert!(!workflows_dir(root).exists(), "a refused create never touches the store");

        // The overwrite refusal — reached AFTER the `workflow:<id>` lock, so
        // it must be native (campaign rule 2).
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-dup");
        create_workflow(root, opts).expect("first create");
        let before = std::fs::read_to_string(workflow_state_path(root, "wf-dup")).unwrap();
        let mut opts = NewWorkflow::for_feature("other-feature");
        opts.id = Some("wf-dup");
        opts.status = Some("closed");
        assert_eq!(
            msg(create_workflow(root, opts)),
            format!(
                "createWorkflow: a workflow record already exists at \"{}\" — createWorkflow never overwrites an \
existing record. FIX: use updateWorkflow, or generate a fresh id.",
                workflow_state_path(root, "wf-dup").display()
            )
        );
        assert_eq!(
            std::fs::read_to_string(workflow_state_path(root, "wf-dup")).unwrap(),
            before,
            "the existing record is byte-identical after the refusal"
        );
    }

    /// createWorkflow takes `workflow:<id>` for its whole body — the same lock
    /// name updateWorkflow uses, so a racing create and update on one id
    /// serialize. Proven by holding the lock externally.
    #[test]
    fn create_takes_the_workflow_id_lock_for_its_whole_body() {
        let tmp = tmp_root();
        let root = tmp.path();
        let held = lock::acquire_store_lock(root, "workflow:wf-locked", 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-locked");
        match create_workflow(root, opts) {
            Err(Err2::Msg(m)) => assert!(
                m.starts_with("lock \"workflow:wf-locked\" busy: held by "),
                "expected LOCK_BUSY, got {m}"
            ),
            other => panic!(
                "create must be denied under a held workflow lock, got {}",
                match other {
                    Ok(_) => "a record".to_string(),
                    Err(Err2::Ex) => "Exotic".to_string(),
                    Err(Err2::Msg(m)) => m,
                }
            ),
        }
        assert!(!workflow_state_path(root, "wf-locked").exists());
        drop(held);
        // Control: with the lock free the very same create succeeds.
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-locked");
        assert!(create_workflow(root, opts).is_ok());
        // A DIFFERENT id is never blocked by another id's lock.
        let other = lock::acquire_store_lock(root, "workflow:wf-locked", 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));
        let mut opts = NewWorkflow::for_feature("g");
        opts.id = Some("wf-sibling");
        assert!(create_workflow(root, opts).is_ok(), "sibling ids hash to distinct lock files");
        drop(other);
    }

    /// Oracle: "listWorkflows on an absent .bee/runtime/workflows/ directory
    /// returns an empty, non-throwing result".
    #[test]
    fn list_workflows_over_an_absent_store_is_empty_and_creates_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        assert!(!workflows_dir(root).exists());
        assert!(ok(list_workflows(root)).is_empty(), "no workflows dir -> empty list");
        assert!(
            !workflows_dir(root).exists(),
            "listWorkflows never creates the directory as a side effect of listing"
        );
        // An EXISTING but empty store is the same answer, still without
        // inventing entries.
        std::fs::create_dir_all(workflows_dir(root)).unwrap();
        assert!(ok(list_workflows(root)).is_empty());
        // Control: the enumeration is not simply always-empty.
        write_workflow(root, "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let listed = ok(list_workflows(root));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].get("id"), Some(&json!("wf-1")));
    }

    /// Oracle: "updateWorkflowAssumingLock (multisession-native-10, C4):
    /// succeeds through an externally-held workflow:<id> lock that DENIES
    /// updateWorkflow itself — proves it takes no lock of its own".
    ///
    /// The negative control runs the real self-locking form against the real
    /// retry loop, so it spends MAX_ATTEMPTS × RETRY_DELAY (~5s) before
    /// reporting busy; Node's oracle short-circuits it with {maxAttempts: 1},
    /// which this port does not expose.
    #[test]
    fn update_assuming_lock_writes_through_an_externally_held_workflow_lock() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_workflow(root, "wf-1", json!({"id":"wf-1","feature":"assuming-lock-deadlock-proof"}));

        let held = lock::acquire_store_lock(root, "workflow:wf-1", 1)
            .unwrap_or_else(|b| panic!("precondition: the test must hold workflow:wf-1 — {}", b.message()));

        // Negative control: the SELF-LOCKING form is denied while that same
        // lock is held — the shape that would deadlock a caller already
        // inside its own withWorkflowLock hold.
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("planning"));
        match update_workflow(root, "wf-1", patch) {
            Err(Err2::Msg(m)) => assert!(
                m.starts_with("lock \"workflow:wf-1\" busy: held by "),
                "expected the LOCK_BUSY refusal, got {m}"
            ),
            other => panic!("updateWorkflow must be denied under a held lock, got {}", match other {
                Ok(_) => "Ok".to_string(),
                Err(_) => "a non-message error".to_string(),
            }),
        }
        // …and the denial wrote nothing.
        assert_eq!(read_back(&workflow_state_path(root, "wf-1")).get("phase"), None);

        // The real proof: the assuming-lock form succeeds THROUGH the same
        // held lock, because it never tries to acquire it.
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("planning"));
        let updated = ok(update_workflow_assuming_lock(root, "wf-1", patch));
        assert_eq!(updated.get("phase"), Some(&json!("planning")));
        assert_eq!(read_back(&workflow_state_path(root, "wf-1"))["phase"], json!("planning"));

        // Release, and the self-locking form works again — so the denial
        // above was the lock, not a broken record.
        drop(held);
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("swarming"));
        assert_eq!(ok(update_workflow(root, "wf-1", patch)).get("phase"), Some(&json!("swarming")));
    }

    /// Oracle: "withWorkflowLock is a thin named wrapper: two ids run their
    /// bodies without either blocking the other".
    ///
    /// Node proves independence by interleaving two async bodies; the Rust
    /// wrapper is an RAII guard, so the same property is proved without a
    /// scheduler: with `workflow:wf-p` held, a DIFFERENT id is granted on its
    /// very first attempt while the SAME id is denied on its first attempt.
    #[test]
    fn workflow_locks_for_two_ids_never_block_each_other() {
        let tmp = tmp_root();
        let root = tmp.path();
        let held_p = ok(acquire_workflow_lock(root, "wf-p"));

        // Independent name: granted with zero retries, while wf-p is held.
        let held_q = lock::acquire_store_lock(root, "workflow:wf-q", 1)
            .unwrap_or_else(|b| panic!("a distinct id must not queue behind wf-p — {}", b.message()));
        // Control: the SAME name is refused on that same single attempt, so
        // the grant above is independence and not a disabled lock.
        let same = lock::acquire_store_lock(root, "workflow:wf-p", 1);
        assert!(same.is_err(), "workflow:wf-p must be denied while it is held");
        // A third, still-distinct id is likewise granted with both held.
        let held_r = lock::acquire_store_lock(root, "workflow:wf-r", 1)
            .unwrap_or_else(|b| panic!("a third distinct id must not queue — {}", b.message()));

        assert!(lock::lock_file_path(root, "workflow:wf-p").exists());
        assert!(lock::lock_file_path(root, "workflow:wf-q").exists());
        assert!(lock::lock_file_path(root, "workflow:wf-r").exists());
        drop(held_q);
        drop(held_r);
        drop(held_p);
        // Every guard released its own file and only its own.
        for id in ["wf-p", "wf-q", "wf-r"] {
            assert!(!lock::lock_file_path(root, &format!("workflow:{id}")).exists());
        }
        // …and wf-p is takeable again once released.
        ok(lock::acquire_store_lock(root, "workflow:wf-p", 1));
    }
}
