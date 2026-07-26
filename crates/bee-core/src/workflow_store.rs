//! workflow_store — per-workflow state records, ported from
//! `.bee/bin/lib/workflow-store.mjs` (rust-port-16, CONTEXT.md D3/D9).
//!
//! A WORKFLOW replaces the single `.bee/state.json` / `.bee/lanes/<feature>.json`
//! pipeline record as the unit of coordination state. Each workflow lives at
//! its own path, keyed by a GENERATED id — never the feature slug:
//!
//!   `.bee/runtime/workflows/<workflow-id>/state.json`
//!   `{ id, feature, phase, mode, plan_rev, gates, summary, next_action,
//!     status, created_at }`
//!
//! `gates` is a map of gate names to `{approved, approved_for_plan_rev}`
//! (gate approval is scoped to a plan revision). `status` is one of
//! active|paused|closed.
//!
//! Locking: every mutating verb here takes ONLY the per-workflow lock
//! `workflow:<id>` via `crate::lock::with_store_lock` — the SAME per-id-lock
//! shape `cells.mjs` uses for `cells:<id>`. This is a LEAF module: it never
//! reads or touches a session/claim record.
//!
//! `.bee/bin/lib/workflow-store.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is a faithful port, never an
//! "improvement" on it.
//!
//! Gates are kept as a raw `Map<String, Value>` (rather than a fixed
//! 4-field struct) so an unknown/future gate name round-trips untouched —
//! the same forward-compat guarantee `mergeGates` documents in the mjs
//! source ("a future cell adding a fifth gate must not be silently
//! dropped here").

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::fsutil::write_json_atomic;
use crate::lock::{with_store_lock, LockOptions, WithLockError};
use crate::workspace::runtime_dir;

pub const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
pub const STATUS_VALUES: [&str; 3] = ["active", "paused", "closed"];
const DEFAULT_STATUS: &str = "active";

pub fn workflows_dir(root: &Path) -> PathBuf {
    runtime_dir(root).join("workflows")
}

/// Typed refusal thrown by `read_workflow`/`update_workflow`/`create_workflow`
/// — never a silent fallback. Mirrors `WorkflowStoreError`'s `{type: 'refused',
/// code, message}` shape.
#[derive(Debug, Clone)]
pub enum WorkflowStoreError {
    InvalidId(String),
    InvalidFeature(String),
    InvalidStatus(String),
    InvalidPatch(String),
    AlreadyExists(String),
    Missing(String),
    Corrupt(String),
    Io(String),
}

impl WorkflowStoreError {
    pub fn code(&self) -> &'static str {
        match self {
            WorkflowStoreError::InvalidId(_) => "WORKFLOW_INVALID_ID",
            WorkflowStoreError::InvalidFeature(_) => "WORKFLOW_INVALID_FEATURE",
            WorkflowStoreError::InvalidStatus(_) => "WORKFLOW_INVALID_STATUS",
            WorkflowStoreError::InvalidPatch(_) => "WORKFLOW_INVALID_PATCH",
            WorkflowStoreError::AlreadyExists(_) => "WORKFLOW_ALREADY_EXISTS",
            WorkflowStoreError::Missing(_) => "WORKFLOW_MISSING",
            WorkflowStoreError::Corrupt(_) => "WORKFLOW_CORRUPT",
            WorkflowStoreError::Io(_) => "WORKFLOW_IO",
        }
    }
}

impl fmt::Display for WorkflowStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            WorkflowStoreError::InvalidId(m)
            | WorkflowStoreError::InvalidFeature(m)
            | WorkflowStoreError::InvalidStatus(m)
            | WorkflowStoreError::InvalidPatch(m)
            | WorkflowStoreError::AlreadyExists(m)
            | WorkflowStoreError::Missing(m)
            | WorkflowStoreError::Corrupt(m)
            | WorkflowStoreError::Io(m) => m,
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for WorkflowStoreError {}

/// A workflow's lock-then-mutate error surface: either the operation was
/// refused outright ([`WorkflowStoreError`]) or the per-workflow lock could
/// not be acquired ([`WithLockError`]).
#[derive(Debug)]
pub enum WorkflowLockError {
    Store(WorkflowStoreError),
    Lock(WithLockError),
}

impl fmt::Display for WorkflowLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowLockError::Store(e) => write!(f, "{e}"),
            WorkflowLockError::Lock(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WorkflowLockError {}

impl From<WorkflowStoreError> for WorkflowLockError {
    fn from(e: WorkflowStoreError) -> Self {
        WorkflowLockError::Store(e)
    }
}

impl From<WithLockError> for WorkflowLockError {
    fn from(e: WithLockError) -> Self {
        WorkflowLockError::Lock(e)
    }
}

/// `.bee/runtime/workflows/<id>/state.json`'s shape. Every field this
/// struct does not name explicitly survives round-trip via `extra` (D3
/// storage-compat contract) — the "unknown fields survive workflow-record
/// round-trips" must-have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRecord {
    pub id: String,
    pub feature: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default = "default_phase")]
    pub phase: String,
    #[serde(default)]
    pub plan_rev: i64,
    #[serde(default)]
    pub gates: Map<String, Value>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub next_action: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_phase() -> String {
    "idle".to_string()
}

fn default_status() -> String {
    DEFAULT_STATUS.to_string()
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn now_iso() -> String {
    crate::lock::iso8601_millis(now_ms())
}

fn random_hex4() -> io::Result<String> {
    let mut buf = [0u8; 4];
    getrandom::fill(&mut buf).map_err(|e| io::Error::other(format!("entropy source failed: {e}")))?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

/// `requireWorkflowId` — the id becomes a directory name under
/// `workflowsDir`, so path separators and `..` are bad arguments.
fn require_workflow_id(value: &str) -> Result<String, WorkflowStoreError> {
    let id = value.trim();
    if id.is_empty() {
        return Err(WorkflowStoreError::InvalidId("workflow id is required.".to_string()));
    }
    if id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(WorkflowStoreError::InvalidId(format!(
            "workflow id \"{id}\" must be a plain id (no path separators) — it becomes a directory name under .bee/runtime/workflows/."
        )));
    }
    Ok(id.to_string())
}

fn require_feature(value: &str) -> Result<String, WorkflowStoreError> {
    let feature = value.trim();
    if feature.is_empty() {
        return Err(WorkflowStoreError::InvalidFeature(
            "createWorkflow: feature is required.".to_string(),
        ));
    }
    Ok(feature.to_string())
}

pub fn workflow_dir(root: &Path, id: &str) -> Result<PathBuf, WorkflowStoreError> {
    Ok(workflows_dir(root).join(require_workflow_id(id)?))
}

pub fn workflow_state_path(root: &Path, id: &str) -> Result<PathBuf, WorkflowStoreError> {
    Ok(workflow_dir(root, id)?.join("state.json"))
}

/// `generateWorkflowId(feature)` — `wf-${hex}`, regenerated on the single
/// foreseeable collision (a generated id equal to the caller's own feature
/// slug).
fn generate_workflow_id(feature: Option<&str>) -> io::Result<String> {
    loop {
        let id = format!("wf-{}", random_hex4()?);
        if feature.map(|f| f.trim() != id).unwrap_or(true) {
            return Ok(id);
        }
    }
}

fn default_gate_entry() -> Value {
    json!({"approved": false, "approved_for_plan_rev": Value::Null})
}

fn default_gates() -> Map<String, Value> {
    let mut gates = Map::new();
    for name in GATE_NAMES {
        gates.insert(name.to_string(), default_gate_entry());
    }
    gates
}

/// `mergeGates(base, overrides)` — overlays `overrides` onto
/// `{...defaultGates(), ...(base||{})}`, per-gate-name, one level deep.
/// `base = None` is exactly `mergeGates(defaultGates(), overrides)` (the
/// create/read path); `base = Some(current.gates)` is the update path.
fn merge_gates(base: Option<&Map<String, Value>>, overrides: Option<&Value>) -> Map<String, Value> {
    let mut merged = default_gates();
    if let Some(b) = base {
        for (k, v) in b {
            merged.insert(k.clone(), v.clone());
        }
    }
    if let Some(Value::Object(over)) = overrides {
        for (name, value) in over {
            let base_entry = merged.get(name).cloned().unwrap_or_else(default_gate_entry);
            let mut entry_obj = match base_entry {
                Value::Object(m) => m,
                _ => Map::new(),
            };
            if let Value::Object(vobj) = value {
                for (k, v) in vobj {
                    entry_obj.insert(k.clone(), v.clone());
                }
            }
            merged.insert(name.clone(), Value::Object(entry_obj));
        }
    }
    merged
}

/// `withWorkflowLock(root, id, fn, options)` — run `f` holding the
/// per-workflow lock `workflow:<id>`.
pub fn with_workflow_lock<T>(
    root: &Path,
    id: &str,
    options: LockOptions,
    f: impl FnOnce() -> Result<T, WorkflowStoreError>,
) -> Result<T, WorkflowLockError> {
    let workflow_id = require_workflow_id(id)?;
    let name = format!("workflow:{workflow_id}");
    match with_store_lock(root, &name, options, f) {
        Ok(inner) => inner.map_err(WorkflowLockError::Store),
        Err(lock_err) => Err(WorkflowLockError::Lock(lock_err)),
    }
}

/// The shared read/parse/validate body for `read_workflow` and
/// `update_workflow_assuming_lock`'s read-modify-write. `workflow_id` is
/// assumed already validated by the caller.
fn read_workflow_record(root: &Path, workflow_id: &str) -> Result<WorkflowRecord, WorkflowStoreError> {
    let file = workflow_dir(root, workflow_id)?.join("state.json");
    let text = fs::read_to_string(&file).map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            WorkflowStoreError::Missing(format!(
                "readWorkflow: no workflow record at \"{}\". FIX: createWorkflow first, or check the id.",
                file.display()
            ))
        } else {
            WorkflowStoreError::Corrupt(format!(
                "readWorkflow: could not read \"{}\" ({err}). The bee CLI refuses to guess at a workflow record it cannot read — that could silently clobber real state (gates, phase).",
                file.display()
            ))
        }
    })?;
    let raw: Value = serde_json::from_str(&text).map_err(|err| {
        WorkflowStoreError::Corrupt(format!(
            "readWorkflow: \"{}\" exists but is not valid JSON ({err}). The bee CLI refuses to rebuild a workflow from defaults over a present-but-corrupt file.",
            file.display()
        ))
    })?;
    if !raw.is_object() {
        return Err(WorkflowStoreError::Corrupt(format!(
            "readWorkflow: \"{}\" exists but is not a JSON object.",
            file.display()
        )));
    }
    if raw.get("id").and_then(Value::as_str) != Some(workflow_id) {
        return Err(WorkflowStoreError::Corrupt(format!(
            "readWorkflow: \"{}\" exists but its id field does not match the requested workflow \"{workflow_id}\" — never trusted.",
            file.display()
        )));
    }
    let gates_override = raw.get("gates").cloned();
    let mut record: WorkflowRecord = serde_json::from_value(raw)
        .map_err(|err| WorkflowStoreError::Corrupt(format!("readWorkflow: \"{}\" has an unexpected shape ({err}).", file.display())))?;
    record.id = workflow_id.to_string();
    record.gates = merge_gates(None, gates_override.as_ref());
    Ok(record)
}

/// `readWorkflow(root, id)` — read a single workflow record. Typed refusal
/// on every failure mode.
pub fn read_workflow(root: &Path, id: &str) -> Result<WorkflowRecord, WorkflowStoreError> {
    let workflow_id = require_workflow_id(id)?;
    read_workflow_record(root, &workflow_id)
}

/// Input to [`create_workflow`] — mirrors `createWorkflow`'s destructured
/// options object; every field but `feature` is optional and defaults the
/// same way the mjs source's default params do.
#[derive(Debug, Clone, Default)]
pub struct CreateWorkflowInput {
    pub phase: Option<String>,
    pub mode: Option<String>,
    pub plan_rev: Option<i64>,
    pub gates: Option<Value>,
    pub summary: Option<String>,
    pub next_action: Option<String>,
    pub status: Option<String>,
    pub id: Option<String>,
}

/// `createWorkflow(root, {feature, ...})` — create a new workflow record.
/// `id` defaults to a fresh `generate_workflow_id(feature)` but may be
/// passed explicitly, EXCEPT equal to `feature` (refused outright — D1
/// requires workflow_id to never be the feature slug). Runs under
/// `with_workflow_lock` so two callers racing the same explicit id can
/// never both "win" a create.
pub fn create_workflow(
    root: &Path,
    feature: &str,
    input: CreateWorkflowInput,
    options: LockOptions,
) -> Result<WorkflowRecord, WorkflowLockError> {
    let feature_name = require_feature(feature)?;
    let workflow_id = match &input.id {
        Some(v) => require_workflow_id(v)?,
        None => generate_workflow_id(Some(&feature_name)).map_err(|e| WorkflowStoreError::Io(e.to_string()))?,
    };
    if workflow_id == feature_name {
        return Err(WorkflowLockError::Store(WorkflowStoreError::InvalidId(format!(
            "createWorkflow: workflow id \"{workflow_id}\" must not equal the feature slug \"{feature_name}\" — ids are generated identifiers, never feature slugs (CONTEXT.md D1)."
        ))));
    }
    let status = input.status.clone().unwrap_or_else(|| DEFAULT_STATUS.to_string());
    if !STATUS_VALUES.contains(&status.as_str()) {
        return Err(WorkflowLockError::Store(WorkflowStoreError::InvalidStatus(format!(
            "createWorkflow: status must be one of {} (got {status:?}).",
            STATUS_VALUES.join("/")
        ))));
    }
    let id_for_lock = workflow_id.clone();
    with_workflow_lock(root, &id_for_lock, options, move || {
        let dir = workflow_dir(root, &workflow_id)?;
        let file = dir.join("state.json");
        if file.exists() {
            return Err(WorkflowStoreError::AlreadyExists(format!(
                "createWorkflow: a workflow record already exists at \"{}\" — createWorkflow never overwrites an existing record.",
                file.display()
            )));
        }
        let record = WorkflowRecord {
            id: workflow_id.clone(),
            feature: feature_name.clone(),
            mode: input.mode.clone(),
            phase: input.phase.clone().unwrap_or_else(default_phase),
            plan_rev: input.plan_rev.unwrap_or(0),
            gates: merge_gates(None, input.gates.as_ref()),
            summary: input.summary.clone().unwrap_or_default(),
            next_action: input.next_action.clone().unwrap_or_default(),
            status: status.clone(),
            created_at: now_iso(),
            extra: Map::new(),
        };
        write_json_atomic(&file, &record).map_err(|e| WorkflowStoreError::Io(e.to_string()))?;
        Ok(record)
    })
}

/// `updateWorkflowAssumingLock(root, id, updater)` — the SAME
/// read-modify-write body `update_workflow` uses, but WITHOUT acquiring
/// `workflow:<id>` itself. NEVER call this without already holding that
/// lock for this exact `id`. `updater` receives the current record and
/// returns a patch object (a plain JSON object, shallow-merged onto the
/// current record; `gates` is merged per-gate-name via `merge_gates`).
/// `id`, `feature`, and `created_at` are identity fields and are never
/// touched by a patch, even if the patch happens to name them.
pub fn update_workflow_assuming_lock(
    root: &Path,
    id: &str,
    updater: impl FnOnce(&WorkflowRecord) -> Value,
) -> Result<WorkflowRecord, WorkflowStoreError> {
    let workflow_id = require_workflow_id(id)?;
    let current = read_workflow_record(root, &workflow_id)?;
    let patch = updater(&current);
    let Value::Object(patch_obj) = &patch else {
        return Err(WorkflowStoreError::InvalidPatch(
            "updateWorkflowAssumingLock: updater must be (or return) a plain object patch.".to_string(),
        ));
    };
    if let Some(status_val) = patch_obj.get("status") {
        let ok = status_val.as_str().map(|s| STATUS_VALUES.contains(&s)).unwrap_or(false);
        if !ok {
            return Err(WorkflowStoreError::InvalidStatus(format!(
                "updateWorkflowAssumingLock: status must be one of {} (got {status_val}).",
                STATUS_VALUES.join("/")
            )));
        }
    }
    let mut next_value = serde_json::to_value(&current).map_err(|e| WorkflowStoreError::Io(e.to_string()))?;
    if let Value::Object(next_obj) = &mut next_value {
        for (k, v) in patch_obj {
            next_obj.insert(k.clone(), v.clone());
        }
        next_obj.insert("id".to_string(), Value::String(current.id.clone()));
        next_obj.insert("feature".to_string(), Value::String(current.feature.clone()));
        next_obj.insert("created_at".to_string(), Value::String(current.created_at.clone()));
        let merged_gates = merge_gates(Some(&current.gates), patch_obj.get("gates"));
        next_obj.insert("gates".to_string(), Value::Object(merged_gates));
    }
    let next: WorkflowRecord = serde_json::from_value(next_value)
        .map_err(|e| WorkflowStoreError::InvalidPatch(format!("updateWorkflowAssumingLock: patch produced an invalid record ({e}).")))?;
    write_json_atomic(&workflow_dir(root, &workflow_id)?.join("state.json"), &next).map_err(|e| WorkflowStoreError::Io(e.to_string()))?;
    Ok(next)
}

/// `updateWorkflow(root, id, updater, options)` — read-modify-write a
/// workflow record under `with_workflow_lock`.
pub fn update_workflow(
    root: &Path,
    id: &str,
    updater: impl FnOnce(&WorkflowRecord) -> Value,
    options: LockOptions,
) -> Result<WorkflowRecord, WorkflowLockError> {
    let workflow_id = require_workflow_id(id)?;
    let id_for_body = workflow_id.clone();
    with_workflow_lock(root, &workflow_id, options, move || {
        update_workflow_assuming_lock(root, &id_for_body, updater)
    })
}

/// An unreadable workflow directory, skipped by [`list_workflows`].
#[derive(Debug, Clone)]
pub struct SkippedWorkflow {
    pub id: String,
    pub reason: String,
}

/// `listWorkflows(root)` — fail-open enumeration for display: a corrupt or
/// unreadable entry is SKIPPED (never guessed at, never thrown for the
/// whole list) and reported back in `skipped` (plus a stderr warning,
/// matching `readLane`'s own convention).
pub struct ListWorkflowsResult {
    pub workflows: Vec<WorkflowRecord>,
    pub skipped: Vec<SkippedWorkflow>,
}

pub fn list_workflows(root: &Path) -> ListWorkflowsResult {
    let entries = match fs::read_dir(workflows_dir(root)) {
        Ok(e) => e,
        Err(_) => return ListWorkflowsResult { workflows: Vec::new(), skipped: Vec::new() },
    };
    let mut workflows = Vec::new();
    let mut skipped = Vec::new();
    for entry in entries.flatten() {
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        match read_workflow_record(root, &id) {
            Ok(record) => workflows.push(record),
            Err(err) => {
                eprintln!("listWorkflows: skipping unreadable workflow \"{id}\" — {err}");
                skipped.push(SkippedWorkflow { id, reason: err.to_string() });
            }
        }
    }
    ListWorkflowsResult { workflows, skipped }
}

// Tests live in crates/bee-core/tests/projection_parity.rs (this cell's
// single integration target — cargo test -p bee-core --test
// projection_parity), oracle-checked against the real
// workflow-store.mjs/state-projection.mjs via a file-based node driver.
