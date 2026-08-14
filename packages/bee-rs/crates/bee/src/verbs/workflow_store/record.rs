// paths, gates, and the record read/write/create path
//
// Split out of the single 2.7k-line verbs/workflow_store.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, LockGuard, MAX_ATTEMPTS};
use crate::verbs::reservations::{
    date_parse_val, jget, js_disp, js_disp_opt, js_trim, now_iso, pseudo_uuid_v4,
    truthy, Err2, Ex, Exotic,
};
use crate::verbs::state_group::{
    adopt_claim, coerce_legacy_phase, default_gates, handoff_path, io_read_reason, parse_json_v8,
    read_claim, read_state_peek, spread_gates, write_state, AdoptOutcome, ParsedJson,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

// ─── paths ─────────────────────────────────────────────────────────────────

/// workflow-store.mjs runtimeDir/workflowsDir.
pub(crate) fn workflows_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("workflows")
}

/// workflow-store.mjs workflowDir (id already validated by the caller).
pub(crate) fn workflow_dir(root: &Path, id: &str) -> PathBuf {
    workflows_dir(root).join(id)
}

/// workflow-store.mjs workflowStatePath.
pub(crate) fn workflow_state_path(root: &Path, id: &str) -> PathBuf {
    workflow_dir(root, id).join("state.json")
}

/// Node path.relative(root, file) for a file under root — the shape every
/// "git checkout -- <rel>" FIX hint interpolates.
pub(crate) fn path_relative(root: &Path, file: &Path) -> String {
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

/// D3: the gate entry's own approval state — persisted, not derived. `off` is
/// deliberately excluded here (it is a config-level setting, not a per-gate
/// one) — see `GATE_BYPASS_LEVELS` for the level vocabulary.
pub(crate) const GATE_STATE_VALUES: [&str; 3] = ["pending", "approved", "rejected"];

/// D2: who moved the gate — a real user, or an auto-approval under
/// `gate_bypass`.
pub(crate) const GATE_ACTOR_VALUES: [&str; 2] = ["user", "auto"];

/// D2: the bypass level an auto-approval recorded as its reason for not
/// halting, mirroring `gate_bypass`'s own vocabulary
/// (`skills/bee-hive/references/gates-and-delegation.md`, "Gate bypass
/// mode").
pub(crate) const GATE_BYPASS_LEVELS: [&str; 4] = ["off", "normal", "full", "total"];

/// A gate entry's `state`/`actor`/`bypass_level` are typed vocabularies. An
/// unrecognized value refuses loudly rather than defaulting silently,
/// mirroring the WORKFLOW_MISSING/WORKFLOW_CORRUPT discipline elsewhere in
/// this module. Only fields PRESENT on `entry` are checked — an old-shape
/// entry that never set `state` at all is not a refusal, it is the case
/// `merge_gates` derives from `approved` instead.
fn check_gate_entry_fields(name: &str, entry: &Map<String, Value>) -> Result<(), Err2> {
    let check = |field: &str, allowed: &[&str], allow_null: bool| -> Result<(), Err2> {
        match entry.get(field) {
            None => Ok(()),
            Some(Value::Null) if allow_null => Ok(()),
            Some(Value::String(s)) if allowed.contains(&s.as_str()) => Ok(()),
            Some(other) => Err(Err2::Msg(format!(
                "gate \"{name}\": {field} must be one of {} (got {}).",
                allowed.join("/"),
                jsjson::stringify(other)
            ))),
        }
    };
    check("state", &GATE_STATE_VALUES, false)?;
    check("actor", &GATE_ACTOR_VALUES, true)?;
    check("bypass_level", &GATE_BYPASS_LEVELS, true)?;
    Ok(())
}

/// workflow-store.mjs defaultGateEntry, extended by D2/D3: `state` is the
/// persisted approval state (kept equal to `approved == "approved"`), and
/// `actor`/`at`/`reason`/`bypass_level` are D2's auto-approval trace. The
/// pre-existing `approved` boolean stays — 37 call sites still read it.
pub(crate) fn default_gate_entry() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("approved".into(), Value::Bool(false));
    m.insert("approved_for_plan_rev".into(), Value::Null);
    m.insert("state".into(), json!("pending"));
    m.insert("actor".into(), Value::Null);
    m.insert("at".into(), Value::Null);
    m.insert("reason".into(), Value::Null);
    m.insert("bypass_level".into(), Value::Null);
    m
}

/// workflow-store.mjs defaultGates.
pub(crate) fn default_wf_gates() -> Map<String, Value> {
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
///
/// D3 extension: a shallow per-key overlay alone would let a patch (or an
/// old-shape on-disk entry) that changes `approved` without also naming
/// `state` leave a STALE `state` behind — the exact desync
/// `gates_patch_from_record` would otherwise cause. So after the overlay, an
/// override that touched `approved` but never named `state` gets `state`
/// re-derived from the fresh boolean (`approved` → `"approved"`, otherwise
/// the persisted default `"pending"`) — this is also what makes an old-shape
/// entry (no `state` field at all, pre-migration) read back correctly. An
/// override that names `state` explicitly is always respected as-is. Every
/// entry an override touches is then validated (`check_gate_entry_fields`),
/// so an unrecognized `state`/`actor`/`bypass_level` refuses instead of
/// silently persisting.
pub(crate) fn merge_gates(base: Option<&Value>, overrides: Option<&Value>) -> Result<Value, Err2> {
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
            let (touches_approved, names_state) = match value {
                Value::Object(v) => (v.contains_key("approved"), v.contains_key("state")),
                _ => (false, false),
            };
            if let Value::Object(v) = value {
                for (k, val) in v {
                    entry.insert(k.clone(), val.clone());
                }
            }
            if touches_approved && !names_state {
                let approved = matches!(entry.get("approved"), Some(Value::Bool(true)));
                entry.insert("state".into(), json!(if approved { "approved" } else { "pending" }));
            }
            check_gate_entry_fields(name, &entry)?;
            merged.insert(name.clone(), Value::Object(entry));
        }
    }
    Ok(Value::Object(merged))
}

/// workflow-store.mjs baseWorkflowDefaults.
pub(crate) fn base_workflow_defaults() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("mode".into(), Value::Null);
    m.insert("phase".into(), json!("idle"));
    m.insert("plan_rev".into(), json!(0));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!(""));
    m.insert("status".into(), json!("active"));
    m.insert("route".into(), Value::Null);
    // D4: the run's own persisted lifecycle name. Null is the pre-migration
    // / not-yet-computed shape (see derive_run_state — every write recomputes
    // it fresh), and check_run_state_value accepts null for exactly that
    // reason.
    m.insert("run_state".into(), Value::Null);
    m
}

// ─── run state (D4) ─────────────────────────────────────────────────────────

/// D4: the closed vocabulary a workflow record's `run_state` is persisted
/// against — covering the lifecycle states a dashboard needs to name without
/// re-deriving them at read time: still being shaped, waiting on a human
/// decision, actively executing, stuck behind a refusal, or finished.
pub(crate) const RUN_STATE_VALUES: [&str; 5] =
    ["shaping", "awaiting-approval", "running", "blocked", "done"];

/// `null` (not yet computed / pre-migration) or one of RUN_STATE_VALUES.
/// Anything else is a corrupt or hand-edited record, refused loudly rather
/// than defaulted silently — the same discipline check_gate_entry_fields
/// already applies to a gate entry's own `state`.
pub(crate) fn valid_run_state_value(v: &Value) -> bool {
    matches!(v, Value::Null) || matches!(v, Value::String(s) if RUN_STATE_VALUES.contains(&s.as_str()))
}

/// A patch's `run_state`, if it names one at all, must be in the closed
/// vocabulary. In practice every write recomputes `run_state` itself
/// (`derive_run_state`, below) and ignores whatever the patch said — this
/// guard exists so a caller that DOES pass a bogus value refuses loudly
/// instead of that value briefly existing in the patch unnoticed.
pub(crate) fn check_patch_run_state(patch: &Map<String, Value>) -> Result<(), Err2> {
    match patch.get("run_state") {
        None => Ok(()),
        Some(v) if valid_run_state_value(v) => Ok(()),
        Some(v) => Err(Err2::Msg(format!(
            "updateWorkflowAssumingLock: run_state must be one of {} (got {}).",
            RUN_STATE_VALUES.join("/"),
            jsjson::stringify(v)
        ))),
    }
}

/// D4: per-status cell counts for one feature, gathered natively from the
/// same `.bee/cells/*.json` store `bee cells list` reads
/// (`crate::verbs::cells::list_cells`). An exotic JSON-array cell file
/// (`Delegate`) is never a run_state input — it counts toward nothing here,
/// exactly as it counts toward nothing in `list_cells`'s own callers today.
#[derive(Default)]
pub(crate) struct CellCounts {
    pub(crate) open: usize,
    pub(crate) claimed: usize,
    pub(crate) blocked: usize,
    pub(crate) capped: usize,
    pub(crate) dropped: usize,
}

impl CellCounts {
    pub(crate) fn total(&self) -> usize {
        self.open + self.claimed + self.blocked + self.capped + self.dropped
    }
}

pub(crate) fn cell_counts_for_feature(root: &Path, feature: &str) -> CellCounts {
    let mut counts = CellCounts::default();
    if feature.is_empty() {
        return counts;
    }
    let Ok(cells) = crate::verbs::cells::list_cells(root, Some(feature), None) else {
        return counts; // exotic array cell — fail open, same as list_cells's own tolerance
    };
    for cell in &cells {
        match cell.get("status").and_then(Value::as_str) {
            Some("open") => counts.open += 1,
            Some("claimed") => counts.claimed += 1,
            Some("blocked") => counts.blocked += 1,
            Some("capped") => counts.capped += 1,
            Some("dropped") => counts.dropped += 1,
            _ => {}
        }
    }
    counts
}

/// D4: the run's own persisted lifecycle name, derived from what the record
/// already knows (its own `status`, and the GATE_NAMES-ordered gate entries
/// slice 1 added) plus a fresh cell-count scan of the SAME feature — never
/// computed at read time, only written by `create_workflow` and
/// `update_workflow_assuming_lock_with`, which both call this and persist
/// the result unconditionally.
///
/// The one rule this feature exists to make visible (plan.md, must_haves):
/// `awaiting-approval` fires whenever any gate is `pending` and NO gate
/// later in the fixed sequence is `approved` — unconditionally, even if an
/// earlier gate in the sequence was `rejected`. A trailing, unresolved
/// `rejected` (nothing later approved, and no earlier pending gate already
/// claimed the state) reads `blocked` — the existing approved→rejected
/// revocation path, finally named on the run as a whole. Once every gate
/// clears, the cell counts distinguish `running` (something open or
/// claimed) from a stalled `blocked` (only blocked cells remain) from
/// `done` (every cell settled) from the pre-cells default of `shaping`.
pub(crate) fn derive_run_state(status: &str, gates: &Value, cells: &CellCounts) -> &'static str {
    if status == "closed" {
        return "done";
    }
    let mut approved_at: Vec<usize> = Vec::new();
    let mut pending_at: Vec<usize> = Vec::new();
    let mut rejected_at: Option<usize> = None;
    for (i, name) in GATE_NAMES.iter().enumerate() {
        let state = jget(gates, name).and_then(|e| jget(e, "state")).and_then(Value::as_str);
        match state {
            Some("approved") => approved_at.push(i),
            Some("rejected") => rejected_at = Some(i),
            _ => pending_at.push(i), // "pending", an old-shape entry, or missing entirely
        }
    }
    // The LAST (highest-index) pending gate is the one to check: anything
    // after it in the sequence is, by construction, not pending (it is
    // either approved or rejected), so it has the fewest "later" gates to
    // clear. If ITS condition holds — nothing later approved — the rule
    // ("a gate entry is pending and none later is approved") is satisfied,
    // and it is also the weakest possible case: if a LOWER pending index
    // failed this same check, the highest one would too, since every later
    // gate that defeats a lower pending gate's check also postdates the
    // higher one.
    if let Some(&p) = pending_at.iter().max() {
        if !approved_at.iter().any(|&a| a > p) {
            return "awaiting-approval";
        }
    }
    if let Some(r) = rejected_at {
        if !approved_at.iter().any(|&a| a > r) {
            return "blocked";
        }
    }
    if cells.open > 0 || cells.claimed > 0 {
        return "running";
    }
    if cells.blocked > 0 {
        return "blocked";
    }
    if cells.total() > 0 {
        return "done";
    }
    "shaping"
}

// ─── record read ───────────────────────────────────────────────────────────

/// Why listWorkflows would skip an entry — always a reason we can author.
///
/// CUTOVER: this used to carry a second variant, `Delegate`, for the two
/// classes whose Node reason embedded a V8 parse message or a libuv errno
/// string. Both are native now (see `read_workflow_record`), so every skip
/// is a `Reason` and nothing about this read can send a run to Node.
pub(crate) struct WfSkip(pub(crate) String);

/// typeof-style label for the not-an-object refusal.
pub(crate) fn found_kind(v: &Value) -> &'static str {
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
///
/// CUTOVER: the two refusals Node built out of engine text are ours now. The
/// WORKFLOW_CORRUPT sentence, its skip-vs-throw role, and the FIX line are
/// unchanged — only the parenthetical cause is reworded:
///   * `(${err.message})`, a V8 parse message → dropped; the sentence already
///     says "is not valid JSON", which is the whole truth we can tell, and it
///     matches readStateStrict's sibling refusal byte for byte in shape.
///   * `(${err.code})`, a libuv errno string → an `io_read_reason` category.
pub(crate) fn read_workflow_record(root: &Path, id: &str) -> Result<Map<String, Value>, WfSkip> {
    let file = workflow_state_path(root, id);
    let file_str = file.display().to_string();
    let rel = path_relative(root, &file);
    let bytes = match std::fs::read(&file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WfSkip(format!(
                "readWorkflow: no workflow record at \"{file_str}\". FIX: createWorkflow first, or check the id."
            )));
        }
        Err(e) => {
            return Err(WfSkip(format!(
                "readWorkflow: could not read \"{file_str}\" ({}). The bee CLI refuses to guess at a workflow record it cannot read — that could silently clobber real state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
                io_read_reason(&file, &e)
            )));
        }
        Ok(b) => b,
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed = match parse_json_v8(&text) {
        // Includes the lone-surrogate escapes V8 accepted and serde refuses:
        // unreadable here means unreadable, so it is reported as corrupt.
        Err(Exotic) | Ok(ParsedJson::Unparseable) => {
            return Err(WfSkip(format!(
                "readWorkflow: \"{file_str}\" exists but is not valid JSON. The bee CLI refuses to rebuild a workflow from defaults over a present-but-corrupt file — that would silently clobber real state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            )));
        }
        Ok(ParsedJson::Parsed(v)) => v,
    };
    let Value::Object(parsed_map) = parsed else {
        return Err(WfSkip(format!(
            "readWorkflow: \"{file_str}\" exists but is not a JSON object (found {}).",
            found_kind(&parsed)
        )));
    };
    if !matches!(parsed_map.get("id"), Some(Value::String(s)) if s == id) {
        return Err(WfSkip(format!(
            "readWorkflow: \"{file_str}\" exists but its id field (\"{}\") does not match the requested workflow \"{id}\" — never trusted. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
            js_disp_opt(parsed_map.get("id"))
        )));
    }
    let mut merged = base_workflow_defaults();
    for (k, v) in &parsed_map {
        merged.insert(k.clone(), v.clone());
    }
    merged.insert("id".into(), json!(id));
    let gates = merge_gates(None, parsed_map.get("gates")).map_err(|e| {
        WfSkip(format!(
            "readWorkflow: \"{file_str}\" exists but its gates are corrupt ({}). The bee CLI refuses to guess at a workflow record it cannot read — that could silently clobber real state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
            match e {
                Err2::Msg(m) => m,
                Err2::Ex => "unreadable value".to_string(),
            }
        ))
    })?;
    merged.insert("gates".into(), gates);
    // D4: `run_state` is a closed vocabulary too (RUN_STATE_VALUES, or the
    // pre-migration `null`) — a hand-edited or corrupt value refuses rather
    // than silently passing through, the same discipline the gates map just
    // applied to itself above.
    if let Some(rs) = merged.get("run_state") {
        if !valid_run_state_value(rs) {
            return Err(WfSkip(format!(
                "readWorkflow: \"{file_str}\" exists but its run_state is corrupt (must be one of {} or null, got {}). The bee CLI refuses to guess at a workflow record it cannot read — that could silently clobber real state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").",
                RUN_STATE_VALUES.join("/"),
                jsjson::stringify(rs)
            )));
        }
    }
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
/// SKIP TOLERANCE IS FULLY NATIVE. The reason bytes come straight from
/// `read_workflow_record`'s own refusals, so ALL FIVE skips — missing record,
/// unreadable file, unparseable JSON, not-a-JSON-object, id mismatch — warn
/// here in the same sentence Node used, one line per bad record, in
/// directory-read order. The repeat count needs no modelling: this function is
/// called from the same places `listWorkflows` is (the mutation-lock scope
/// read, the write-through, and each rebuild*), so a verb that calls it 2–4
/// times emits the stream 2–4 times by construction.
/// `state_lanes_over_a_broken_workflow_record_warns_once_per_call` pins that.
///
/// CUTOVER: two of those five used to be decided in a PRE-PASS that buffered
/// every warn until the whole directory was known warn-reproducible, so a
/// delegating run could emit zero bytes before returning None. With their
/// reasons native there is nothing left to decide, so the pre-pass is gone and
/// each warn fires at its own record, exactly as the ordinary skips always
/// did. The one surviving `Exotic` is a non-UTF-8 directory name, which is not
/// a corrupt-JSON matter at all.
pub(crate) fn list_workflows(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let Ok(rd) = std::fs::read_dir(workflows_dir(root)) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // `if (!entry.isDirectory()) continue` — silent, no warn
        }
        let Some(id) = entry.file_name().to_str().map(str::to_string) else {
            return Err(Exotic); // non-UTF-8 name: Node's id bytes are unmodelable here
        };
        match read_workflow_record(root, &id) {
            Ok(record) => out.push(record),
            Err(WfSkip(reason)) => eprintln!("{}", skip_warn_line(&id, &reason)),
        }
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
        w.get("feature").unwrap_or(&Value::Null) == &want
            && w.get("status").unwrap_or(&Value::Null) != &json!("closed")
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
pub(crate) fn protect_identity(next: &mut Map<String, Value>, current: &Map<String, Value>, key: &str) {
    match current.get(key) {
        Some(v) => {
            next.insert(key.to_string(), v.clone());
        }
        None => {
            next.shift_remove(key);
        }
    }
}

pub(crate) fn check_patch_status(patch: &Map<String, Value>) -> Result<(), Err2> {
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
        // Every WORKFLOW_MISSING/WORKFLOW_CORRUPT refusal is authorable now,
        // so the strict read throws for all five instead of two delegating.
        Err(WfSkip(msg)) => return Err(Err2::Msg(msg)),
    };
    let patch = updater(&current)?;
    check_patch_status(&patch)?;
    check_patch_run_state(&patch)?;
    let mut next = current.clone();
    for (k, v) in &patch {
        next.insert(k.clone(), v.clone());
    }
    protect_identity(&mut next, &current, "id");
    protect_identity(&mut next, &current, "feature");
    protect_identity(&mut next, &current, "created_at");
    next.insert(
        "gates".into(),
        merge_gates(current.get("gates"), patch.get("gates"))?,
    );
    // D4: run_state is derived and WRITTEN fresh on every update — never
    // taken from the patch — so it always reflects what this write just
    // produced (the status/gates fields resolved above) plus a live
    // cell-count scan of the same feature, and survives a restart.
    let status_str = next.get("status").and_then(Value::as_str).unwrap_or("active");
    let feature_str = next.get("feature").and_then(Value::as_str).unwrap_or("");
    let gates_val = next.get("gates").cloned().unwrap_or_else(|| Value::Object(default_wf_gates()));
    let counts = cell_counts_for_feature(root, feature_str);
    next.insert("run_state".into(), json!(derive_run_state(status_str, &gates_val, &counts)));
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
pub(crate) fn generate_workflow_id(feature: Option<&str>) -> String {
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
        record.insert("gates".into(), merge_gates(None, opts.gates.as_ref())?);
        record.insert("summary".into(), opts.summary.unwrap_or_else(|| json!("")));
        record.insert("next_action".into(), opts.next_action.unwrap_or_else(|| json!("")));
        record.insert("status".into(), Value::String(status.to_string()));
        record.insert("created_at".into(), Value::String(now_iso()));
        // D4/D1: a brand-new record's gates are all the default `pending`
        // (default_wf_gates via merge_gates above), so this reads
        // "awaiting-approval" on the very first write — the run is
        // traceable as waiting on a human from the moment it exists.
        let gates_val = record.get("gates").cloned().unwrap_or_else(|| Value::Object(default_wf_gates()));
        let feature_for_counts = record.get("feature").and_then(Value::as_str).unwrap_or("").to_string();
        let counts = cell_counts_for_feature(root, &feature_for_counts);
        record.insert("run_state".into(), json!(derive_run_state(status, &gates_val, &counts)));
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
