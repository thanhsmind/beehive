// bee state — native port of the `state` verb group (strangler subset).
//
// R6 coverage debt "the lane/workflow world" — CLOSED for the record-mutating
// verbs. Through R3 wave 2 these verbs served natively ONLY in the "C1 world"
// (no --lane flag, no session-bound lane, zero records under
// .bee/runtime/workflows/). That gate is GONE: verbs/workflow_store.rs now
// carries the lane store, the workflow store, the handoff mailbox, and the
// projection builders, so lane-targeted and workflow-carrying repos take the
// same native path a bare repo does — same lock names, same projection
// write-through, same bytes.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   state set / gate / scribing-run / compounding-run / plan-rev bump
//     — native in EVERY repo shape (explicit --lane, a session-bound lane, or
//       the default record; with or without live workflow records). The full
//       Node seam is reproduced: resolveMutationLockScope's fail-open peek,
//       withMutationLock's `workflow:<id>` → {'state' | lane:<f>} nesting,
//       resolveMutationTarget's strict reads, and
//       writeLaneRecordThroughProjection / writeStateRecordThroughProjection
//       (updateWorkflowAssumingLock + rebuild). Deterministic refusals — arg
//       validation, the chain-integrity gate doors, owner checks,
//       readStateStrict/readLaneStrict's typed errors, the LANE_MISSING
//       refusals, the scribing-run/compounding-run phase doors, plan-rev
//       bump's four refusals — are all native.
//       Still delegated INSIDE these verbs (each is a different R6 debt, not
//       this one): a PASSING `--phase compounding-complete` close (its
//       scribing-debt door + waiver decision-logging live in cells.mjs /
//       decisions.mjs — default AND lane branches), a real --feature swap
//       (same door), and a high-risk execution/merge approval
//       (advisorRefStale, lib/state.mjs).
//   state worker add / update / remove / clear / prune — always native for
//     known flag shapes (they never consult lanes/sessions/workflows).
//   state lanes / session list / session bind / session unbind — native.
//   state scribing-run --show — native (read-only ledger/lane/state query).
//   state handoff write / adopt / show — native in every repo shape: the
//     legacy .bee/HANDOFF.json path when resolveHandoffWorkflowId answers
//     null (C1), and the per-workflow MAILBOX path
//     (.bee/runtime/handoffs/<workflow-id>/NNNN.json + the legacy-file
//     projection rebuild) when a workflow resolves.
//   state workflows list / close — native (listWorkflowRecords /
//     closeWorkflowsForFeature + the three mutually exclusive close modes).
//
// DELEGATED whole verbs (unprovable here, by design): state.route (its
// worktree-grant block reaches guards/worktree machinery), state.start-feature
// (startFeature's precondition sweep + write-policy/isolate redirect + claims
// + reservations), state.rebuild-projections (needs reservations.mjs's
// rebuildReservationsProjection, whose Rust twin `list_reservations` is
// private to verbs/reservations.rs), state.advisor-ref.*, state.compact-*.
//
// Provenance: bee.mjs handleStateSet/handleStateGate/handleStatePlanRevBump/
// stateWorkerMutate + worker handlers/readPruneKeepSet/keptByPruneKeepSet/
// handleStateScribingRun/handleStateCompoundingRun/handleStateLanes/
// handleStateSessionList/Bind/Unbind/handleStateHandoffWrite/Adopt/Show/
// resolveHandoffWorkflowId/mutationLaneSelector/optionalLaneFlag/
// resolveMutationTarget/resolveMutationLockScope/requireFlag/requireFlags/
// exampleFor/splitList/WORKER_TRANSIENT_SUFFIX; lib/state.mjs readStateStrict/
// writeState/defaultState/coerceLegacyPhase/checkPhaseTransition/
// checkScribingRunPhase/checkCompoundingRunPhase/isKnownPhase/readState/
// readHandoff/writeHandoff/adoptHandoff/normalizeHandoffKind/readLane/
// laneRecordFrom/defaultLaneRecord/listLanes; lib/claims.mjs requireId/
// sessionsDir/readSession/listSessionRecords/heartbeatStale/resolveSessionId/
// bindSessionLane/unbindSessionLane/readClaim/adoptClaim (gate file +
// fence_epoch); lib/cells.mjs scribingLedgerPath/readScribingLedger/
// appendScribingLedger/bestScribingStampMs/scribingRunStampMs.
//
// Locking: identical lock-name strings — the mutation verbs follow bee.mjs's
// global order `workflow:<id>` → {'state' | lane:<feature>} (withMutationLock),
// falling back to a single "state" hold when no live workflow names the
// target; the worker verbs hold "state" alone; the handoff mailbox holds
// `handoff:<workflow-id>`; session bind/unbind hold "sessions" through
// claims.mjs's bounded 15×20ms acquire-once loop; adoptClaim uses the
// per-claim `<cell>.adopting` gate file (no store lock). worker prune takes
// no lock (read-only on state). All waits are lock.rs's 100×50ms
// withStoreLock, with LockBusyError's bytes reproduced natively.
//
// Known accepted approximations (documented, delegation guards the rest):
// unreadable-file errno strings map only EISDIR/EPERM (others delegate);
// serde-vs-V8 JSON grammar gaps (lone-surrogate escapes) delegate via the
// "\u"-escape heuristic; prune's mid-loop rmSync failure message is
// reconstructed from the errno class; the scribing-ledger append-failure
// warning (embeds a Node error message) is not replicated — the append
// virtually never fails and the verb's own success output is unaffected.

use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt, js_numberify, js_strict_eq,
    js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy, Ctx, Err2, Ex, Exotic,
    FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::reservations::rebuild_reservations_projection;
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, find_live_workflow,
    gates_patch_from_record, lane_lock_name, lane_path, list_lanes, list_workflows,
    newest_open_handoff_mailbox_record, projection_lock_name, read_lane_display, read_lane_strict,
    rebuild_handoff_projection, rebuild_handoff_projection_reporting, rebuild_lane_projection,
    rebuild_lane_projection_reporting, rebuild_state_projection,
    rebuild_state_projection_reporting, update_workflow, update_workflow_assuming_lock,
    update_workflow_assuming_lock_with, wf_id, workflows_list_sort, write_lane,
    write_mailbox_handoff, MailboxAdopt,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── enums (state.mjs) ─────────────────────────────────────────────────────

const KNOWN_PHASES: [&str; 9] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding",
    "grooming", "compounding-complete",
];
const KNOWN_PHASES_JOINED: &str =
    "idle, exploring, planning, swarming, reviewing, scribing, compounding, grooming, compounding-complete";
const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];
const SCRIBING_RUN_FROM: [&str; 3] = ["swarming", "reviewing", "scribing"];
const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

fn is_known_phase(p: &str) -> bool {
    KNOWN_PHASES.contains(&p)
}

// exampleFor(command) — registry examples[0] for the requireFlags callers.
const EXAMPLE_GATE: &str = "bee state gate --name execution --approved true --json";
const EXAMPLE_SCRIBING: &str =
    "bee state scribing-run --feature newf --areas auth --next-action bee-capturing --json";
const EXAMPLE_WORKFLOWS_CLOSE: &str = "bee state workflows close --feature stale-feature --json";
const EXAMPLE_COMPOUNDING: &str =
    "bee state compounding-run --feature newf --learnings docs/history/newf/reports/learnings.md --json";

// ─── tiny JS-semantics helpers ─────────────────────────────────────────────

/// `a === b` where either side may be undefined (None). undefined === undefined
/// is true in JS.
fn opt_strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => js_strict_eq(x, y),
        _ => false,
    }
}

/// `${from || 'idle'}` — the phase-door display value.
fn disp_or_idle(from: Option<&Value>) -> String {
    match from {
        Some(v) if truthy(v) => js_disp(v),
        _ => "idle".to_string(),
    }
}

/// JS Array.prototype.sort default comparator (UTF-16 code units).
fn js_sort(v: &mut [String]) {
    v.sort_by(|a, b| {
        a.encode_utf16()
            .collect::<Vec<_>>()
            .cmp(&b.encode_utf16().collect::<Vec<_>>())
    });
}

/// splitList (bee.mjs): split on ',', JS-trim, drop empties.
fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `.bee<sep>rest...` display path (path.relative(root, file) for files under
/// root/.bee/) — Node uses the platform separator.
fn rel_bee(parts: &[&str]) -> String {
    let mut out = String::from(".bee");
    for p in parts {
        out.push(MAIN_SEPARATOR);
        out.push_str(p);
    }
    out
}

pub(crate) enum ParsedJson {
    Parsed(Value),
    Unparseable,
}

/// JSON.parse modeled: serde parse + js_numberify. When serde fails but the
/// text carries "\u" escapes, V8 might still parse it (lone surrogates) —
/// delegate instead of guessing.
pub(crate) fn parse_json_v8(text: &str) -> Ex<ParsedJson> {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => Ok(ParsedJson::Parsed(js_numberify(&v)?)),
        Err(_) => {
            if text.contains("\\u") {
                Err(Exotic)
            } else {
                Ok(ParsedJson::Unparseable)
            }
        }
    }
}

// ─── flag plumbing (bee.mjs requireFlag / requireFlags / lane selectors) ───

/// The raw string value of a flag, or None for absent/boolean-true — the
/// shape `typeof flag === 'string'` accepts (resolveSessionId's own guard).
fn flag_value(flags: &Flags, name: &str) -> Option<String> {
    match flags.get(name) {
        Some(FlagV::S(s)) => Some(s.clone()),
        _ => None,
    }
}

/// `${sessionId}` where resolveSessionId answers `string | null`.
fn sid_disp(sid: &Option<String>) -> String {
    match sid {
        Some(s) => s.clone(),
        None => "null".to_string(),
    }
}

/// `String(flags[name])` when the flag is present (Present → "true").
fn flag_string(flags: &Flags, name: &str) -> Option<String> {
    match flags.get(name) {
        None => None,
        Some(FlagV::Present) => Some("true".to_string()),
        Some(FlagV::S(s)) => Some(s.clone()),
    }
}

/// requireFlag: undefined | '' | true → the thrown "Missing required flag".
fn require_flag(flags: &Flags, name: &str) -> Result<String, Err2> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Err2::Msg(format!("Missing required flag --{name}."))),
    }
}

/// requireFlags (ce-1 batch refusal): every missing/invalid flag in one Error.
fn require_flags(
    flags: &Flags,
    spec: &[(&str, Option<&[&str]>)],
    example: &str,
) -> Result<Vec<String>, Err2> {
    let mut missing: Vec<String> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    for (name, allowed) in spec {
        match flags.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => {
                if let Some(a) = allowed {
                    if !a.contains(&s.as_str()) {
                        invalid.push(format!("--{name} \"{s}\" (must be one of {})", a.join(", ")));
                        values.push(String::new());
                        continue;
                    }
                }
                values.push(s.clone());
            }
            _ => {
                missing.push(format!("--{name}"));
                values.push(String::new());
            }
        }
    }
    if missing.is_empty() && invalid.is_empty() {
        return Ok(values);
    }
    let mut parts: Vec<String> = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing required flag(s): {}", missing.join(", ")));
    }
    if !invalid.is_empty() {
        parts.push(format!("invalid flag(s): {}", invalid.join("; ")));
    }
    Err(Err2::Msg(format!("{}. Example: {example}", parts.join("; "))))
}

/// optionalLaneFlag: `--lane` bare/empty is a malformed call, never "no lane".
fn optional_lane_flag(flags: &Flags, verb: &str) -> Result<Option<String>, Err2> {
    match flags.get("lane") {
        None => Ok(None),
        Some(FlagV::Present) => Err(Err2::Msg(format!(
            "{verb}: --lane requires a value (the lane's feature name)."
        ))),
        Some(FlagV::S(s)) if s.is_empty() => Err(Err2::Msg(format!(
            "{verb}: --lane requires a value (the lane's feature name)."
        ))),
        Some(FlagV::S(s)) => Ok(Some(s.clone())),
    }
}

/// mutationLaneSelector (i54-closeout-7 D7).
fn mutation_lane_selector(flags: &Flags, verb: &str) -> Result<(Option<String>, bool), Err2> {
    let lane_feature = optional_lane_flag(flags, verb)?;
    let no_lane = flags.get("no-lane").is_some();
    if no_lane && lane_feature.is_some() {
        return Err(Err2::Msg(format!(
            "{verb}: --no-lane cannot be combined with --lane — --no-lane forces the default record, --lane names a lane record. Pick one."
        )));
    }
    Ok((lane_feature, no_lane))
}

/// validate() type check for a boolean-schema flag: bare, "true", or "false"
/// pass; any other =value is validate()'s own STDOUT refusal → delegate.
fn bool_flag_ok(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        None | Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true" || s == "false",
    }
}

// ─── state store (lib/state.mjs readStateStrict / readState / writeState) ──

fn state_path(root: &Path) -> PathBuf {
    root.join(".bee").join("state.json")
}

pub(crate) fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("context".into(), json!(false));
    m.insert("shape".into(), json!(false));
    m.insert("execution".into(), json!(false));
    m.insert("review".into(), json!(false));
    m
}

pub(crate) fn default_state() -> Map<String, Value> {
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
        json!("No active bee work — awaiting a user request."),
    );
    m
}

/// `{ ...defaults, ...(overlay || {}) }` over the gates map. Falsy and
/// no-own-props truthy primitives (true, numbers) yield the defaults; string/
/// array spreads add JS exotica → delegate.
pub(crate) fn spread_gates(overlay: Option<&Value>) -> Ex<Map<String, Value>> {
    match overlay {
        Some(Value::Object(over)) => {
            let mut merged = default_gates();
            for (k, v) in over {
                merged.insert(k.clone(), v.clone());
            }
            Ok(merged)
        }
        Some(Value::String(s)) if !s.is_empty() => Err(Exotic),
        Some(Value::Array(_)) => Err(Exotic),
        _ => Ok(default_gates()),
    }
}

/// coerceLegacyPhase applied to the merged map's phase slot (D13).
pub(crate) fn coerce_legacy_phase(m: &mut Map<String, Value>) -> Ex<()> {
    let Some(phase) = m.get("phase") else { return Ok(()) };
    if matches!(phase, Value::String(s) if s == "validating") {
        m.insert("phase".into(), json!("planning"));
    } else if !matches!(phase, Value::String(_)) && js_disp(phase) == "validating" {
        // hasOwnProperty coerces the property key — an exotic value whose JS
        // string form is "validating" would coerce in Node; delegate.
        return Err(Exotic);
    }
    Ok(())
}

/// `{ ...defaultState(), ...parsed }` + gates merge + phase coercion — shared
/// by readState and readStateStrict.
fn merge_state_with_defaults(parsed: &Map<String, Value>) -> Ex<Map<String, Value>> {
    let mut merged = default_state();
    for (k, v) in parsed {
        merged.insert(k.clone(), v.clone());
    }
    let gates = spread_gates(parsed.get("approved_gates"))?;
    merged.insert("approved_gates".into(), Value::Object(gates));
    coerce_legacy_phase(&mut merged)?;
    Ok(merged)
}

/// readStateStrict — the three typed refuse-to-rebuild errors are DETERMINISTIC
/// bytes (they embed only path + code strings) and are served natively.
pub(crate) fn read_state_strict(root: &Path) -> Result<Map<String, Value>, Err2> {
    let file = state_path(root);
    let file_disp = file.display().to_string();
    let rel = rel_bee(&["state.json"]);
    let tail_read = "The bee CLI refuses to rebuild state from defaults when it cannot read the existing file — that could silently clobber real state (gates, workers, feature).";
    let tail_corrupt_a = "The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently clobber real state (gates, workers, feature) while reporting success.";
    let fix = format!("FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.");
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(default_state()),
        Err(e) => {
            // err.code interpolation: map the two codes seen in practice; any
            // other errno class delegates rather than guessing Node's string.
            let code = if file.is_dir() {
                "EISDIR"
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                "EPERM"
            } else {
                return Err(Err2::Ex);
            };
            return Err(Err2::Msg(format!(
                "readStateStrict: could not read \"{file_disp}\" ({code}). {tail_read} {fix}"
            )));
        }
    };
    // Node reads utf8 (lossy) and parses WITHOUT a BOM strip (unlike readJson).
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed = match parse_json_v8(&text)? {
        ParsedJson::Parsed(v) => v,
        ParsedJson::Unparseable => {
            return Err(Err2::Msg(format!(
                "readStateStrict: \"{file_disp}\" exists but is not valid JSON. {tail_corrupt_a} {fix}"
            )));
        }
    };
    let obj = match &parsed {
        Value::Object(m) => m.clone(),
        other => {
            let found = match other {
                Value::Array(_) => "an array",
                Value::Null => "object", // typeof null
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Bool(_) => "boolean",
                Value::Object(_) => unreachable!(),
            };
            return Err(Err2::Msg(format!(
                "readStateStrict: \"{file_disp}\" exists but is not a JSON object (found {found}). {tail_corrupt_a} {fix}"
            )));
        }
    };
    merge_state_with_defaults(&obj).map_err(|_| Err2::Ex)
}

/// readState — the fail-open peek (corrupt → Node's V8-worded warn → delegate).
pub(crate) fn read_state_peek(root: &Path) -> Ex<Map<String, Value>> {
    match read_json(&state_path(root)) {
        ReadJson::Missing => Ok(default_state()),
        ReadJson::Corrupt => Err(Exotic),
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            Value::Object(m) => merge_state_with_defaults(&m),
            _ => Ok(default_state()),
        },
    }
}

pub(crate) fn write_state(root: &Path, state: &Map<String, Value>) -> Result<(), Err2> {
    write_json_atomic(&state_path(root), &Value::Object(state.clone())).map_err(|_| Err2::Ex)
}

// ─── phase rules (state.mjs, chain-integrity door — PURE) ──────────────────

struct Transition {
    ok: bool,
    reason: String,
    /// transition.waivedCompounding — production reads it only on the close
    /// path, which this port delegates (the waiver's decision-logging lives in
    /// decisions.mjs); kept so the door tests pin the full Node contract.
    #[cfg_attr(not(test), allow(dead_code))]
    waived_compounding: bool,
}

/// checkPhaseTransition (state.mjs:116-166) — the chain-integrity tail guard.
fn check_phase_transition(
    from: Option<&Value>,
    to: &str,
    rec: &Map<String, Value>,
    waive_compounding: bool,
) -> Ex<Transition> {
    let refuse = |reason: String| Transition { ok: false, reason, waived_compounding: false };
    if to == "compounding" {
        return Ok(refuse(
            "set: phase \"compounding\" is not settable directly — it is produced only by RECORDING a real scribing run, never by asserting one. FIX: run `bee state scribing-run --feature <f> --areas \"<a,b>\" --next-action \"<n>\"`, which stamps last_scribing_run and advances the phase for you.".to_string(),
        ));
    }
    // `current !== 'compounding'` — strict, so only the exact string passes.
    let current_is_compounding = matches!(from, Some(Value::String(s)) if s == "compounding");
    if to == "compounding-complete" && !current_is_compounding {
        let current = disp_or_idle(from);
        return Ok(refuse(format!(
            "set: phase \"compounding-complete\" may only be entered from \"compounding\" (current: \"{current}\"). That name asserts scribing ran, compounding ran, AND the compounding run was RECORDED — not merely asserted; setting it from \"{current}\" claims work that did not happen and shuts the intake gate on a feature that never closed. FIX: close the chain in order — bee-capturing (`state scribing-run`), then bee-capturing (`state compounding-run`)."
        )));
    }
    if to == "compounding-complete" {
        let run = rec.get("last_compounding_run");
        let scribing = rec.get("last_scribing_run");
        let parse_at = |v: Option<&Value>| -> Ex<Option<f64>> {
            match v {
                Some(v) if truthy(v) => match jget(v, "at") {
                    Some(Value::String(_)) => date_parse_val(jget(v, "at")),
                    _ => Ok(None), // typeof run.at !== 'string' → NaN
                },
                _ => Ok(None),
            }
        };
        let run_at = parse_at(run)?;
        let scribing_at = parse_at(scribing)?;
        let run_truthy = run.map(truthy).unwrap_or(false);
        let scribing_truthy = scribing.map(truthy).unwrap_or(false);
        let fresh = run_truthy
            && scribing_truthy
            && matches!((run_at, scribing_at), (Some(r), Some(s)) if r >= s)
            && opt_strict_eq(
                run.and_then(|v| jget(v, "feature")),
                scribing.and_then(|v| jget(v, "feature")),
            );
        if !fresh && !waive_compounding {
            // (scribing && scribing.feature) || rec.feature || '<f>'
            let cand1 = if scribing_truthy {
                scribing.and_then(|v| jget(v, "feature"))
            } else {
                None
            };
            let feature_name = [cand1, rec.get("feature")]
                .into_iter()
                .flatten()
                .find(|v| truthy(v))
                .map(js_disp)
                .unwrap_or_else(|| "<f>".to_string());
            return Ok(refuse(format!(
                "set: phase \"compounding-complete\" refused — no fresh compounding run recorded for feature \"{feature_name}\" (last_compounding_run must exist, with an \"at\" timestamp at or after last_scribing_run.at, for the same feature). FIX: run `bee state compounding-run --feature {feature_name} --learnings <path>`, then retry. If compounding genuinely needs no separate recorded run here, retry with --waive-compounding — it is permitted, but it logs a decision naming the feature."
            )));
        }
        return Ok(Transition { ok: true, reason: String::new(), waived_compounding: !fresh && waive_compounding });
    }
    Ok(Transition { ok: true, reason: String::new(), waived_compounding: false })
}

/// checkScribingRunPhase — None = ok, Some(reason) = refuse.
fn check_scribing_run_phase(from: Option<&Value>) -> Option<String> {
    if matches!(from, Some(Value::String(s)) if SCRIBING_RUN_FROM.contains(&s.as_str())) {
        return None;
    }
    let current = disp_or_idle(from);
    Some(format!(
        "scribing-run: refused from phase \"{current}\" — a scribing run records the spec sync for work that has been EXECUTED. Legal from: swarming, reviewing, scribing. FIX: if execution really is done, the phase should say so; if it is not, there is nothing to scribe yet."
    ))
}

/// checkCompoundingRunPhase — legal only from "compounding".
fn check_compounding_run_phase(from: Option<&Value>) -> Option<String> {
    if matches!(from, Some(Value::String(s)) if s == "compounding") {
        return None;
    }
    let current = disp_or_idle(from);
    Some(format!(
        "compounding-run: refused from phase \"{current}\" — a compounding run records the durable-learnings sync for a feature whose scribing has already run. Legal from: compounding. FIX: if compounding really is underway, the phase should already say so (`state scribing-run` is what produces it); if it is not, there is nothing to compound yet."
    ))
}

// ─── sessions (lib/claims.mjs — control root == root on the native path) ───

fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

/// claims.mjs requireId — thrown Errors, byte-identical.
fn require_id(value: &str, label: &str) -> Result<String, Err2> {
    let id = js_trim(value);
    if id.is_empty() {
        return Err(Err2::Msg(format!("{label} is required.")));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Err2::Msg(format!("{label} must be a plain id (no path separators).")));
    }
    Ok(id.to_string())
}

/// readSession — fail-open (malformed id / missing → None); corrupt JSON
/// delegates (Node warns with the V8 message).
fn read_session(root: &Path, session_id: &str) -> Ex<Option<Map<String, Value>>> {
    let id = js_trim(session_id);
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Ok(None);
    }
    match read_json(&sessions_dir(root).join(format!("{id}.json"))) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Exotic),
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            Value::Object(m) => {
                // session.id !== String(sessionId).trim() → null
                if matches!(m.get("id"), Some(Value::String(s)) if s == id) {
                    Ok(Some(m))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        },
    }
}

fn list_session_records(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let entries = match std::fs::read_dir(sessions_dir(root)) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_session(root, stem)? {
            out.push(record);
        }
    }
    Ok(out)
}

/// claims.mjs heartbeatStale (default 900s window).
fn heartbeat_stale(session: &Map<String, Value>, now: f64) -> Ex<bool> {
    match date_parse_val(session.get("last_heartbeat"))? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// claims.mjs resolveSessionId({flag, root}) — the explicit flag wins, then
/// the env chain, then single-live-session adoption.
fn resolve_session_id(flag: Option<&str>, root: &Path) -> Ex<Option<String>> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Ok(Some(js_trim(f).to_string()));
        }
    }
    resolve_session_id_no_flag(root)
}

/// resolveSessionId({root}) — env chain, then single-live-session adoption.
fn resolve_session_id_no_flag(root: &Path) -> Ex<Option<String>> {
    if let Some(v) = env_nonempty("BEE_SESSION_ID") {
        return Ok(Some(v));
    }
    if let Some(v) = env_nonempty("CLAUDE_CODE_SESSION_ID") {
        return Ok(Some(v));
    }
    let now = now_ms();
    let mut fresh: Vec<Map<String, Value>> = Vec::new();
    for r in list_session_records(root)? {
        if !heartbeat_stale(&r, now)? {
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

/// The `(sessionId, boundLane)` pair every resolution seam shares:
/// `session && typeof session.lane === 'string' ? session.lane.trim() : ''`.
fn session_binding(flag: Option<&str>, root: &Path) -> Ex<(Option<String>, Option<String>)> {
    let Some(sid) = resolve_session_id(flag, root)? else { return Ok((None, None)) };
    let Some(sess) = read_session(root, &sid)? else { return Ok((Some(sid), None)) };
    let bound = match sess.get("lane") {
        Some(Value::String(l)) if !js_trim(l).is_empty() => Some(js_trim(l).to_string()),
        _ => None,
    };
    Ok((Some(sid), bound))
}

/// claims.mjs SESSIONS_LOCK_NAME bounded acquire (15 × 20ms, acquire-once).
fn acquire_sessions_lock(root: &Path) -> Option<lock::LockGuard> {
    for attempt in 0..15u32 {
        match lock::acquire_store_lock_once(root, "sessions") {
            AcquireOnce::Acquired(guard) => return Some(guard),
            AcquireOnce::Busy { .. } => {
                if attempt + 1 < 15 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }
    None
}

// ─── claims (readClaim / adoptClaim + the per-claim gate file) ─────────────

fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// readClaim — `!claim || typeof claim !== 'object'` → null. Arrays pass
/// typeof in JS; the callers here treat them as Exotic when spread.
pub(crate) fn read_claim(root: &Path, cell: &str) -> Ex<Option<Value>> {
    match read_json(&claims_dir(root).join(format!("{cell}.json"))) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Exotic),
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            v @ (Value::Object(_) | Value::Array(_)) => Ok(Some(v)),
            _ => Ok(None),
        },
    }
}

/// Removes the `<cell>.adopting` gate file on drop (releaseGate, force:true).
struct GateGuard(PathBuf);
impl Drop for GateGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(crate) enum AdoptOutcome {
    Fail { reason: String },
    Adopted { claim: Map<String, Value>, previous_owner: Option<Value> },
}

/// adoptClaim (claims.mjs) — in-place atomic rewrite under the exclusive
/// gate; fence_epoch bumps by exactly 1 in the same write.
pub(crate) fn adopt_claim(root: &Path, cell_id: &str, new_session_id: &str) -> Result<AdoptOutcome, Err2> {
    let cell = require_id(cell_id, "cell id")?;
    let session = require_id(new_session_id, "session id")?;
    ensure_dir(&claims_dir(root)).map_err(|_| Err2::Ex)?;
    let gate_path = claims_dir(root).join(format!("{cell}.adopting"));
    let now = now_iso();
    let gate_body = format!(
        "{}\n",
        jsjson::stringify(&json!({"pid": std::process::id(), "at": now}))
    );
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&gate_path) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(gate_body.as_bytes()).map_err(|_| Err2::Ex)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(AdoptOutcome::Fail {
                reason: format!(
                    "claim \"{cell}\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
                ),
            });
        }
        Err(_) => return Err(Err2::Ex),
    }
    let _gate = GateGuard(gate_path); // releaseGate via finally, incl. delegate paths
    let claim = read_claim(root, &cell)?;
    let m = match claim {
        None => {
            return Ok(AdoptOutcome::Fail {
                reason: format!("cell \"{cell}\" has no claim to adopt."),
            });
        }
        Some(Value::Array(_)) => return Err(Err2::Ex), // JS array spread exotica
        Some(Value::Object(m)) => m,
        Some(_) => unreachable!("read_claim yields only objects/arrays"),
    };
    let previous = m.get("session").cloned();
    // Number.isFinite(claim.fence_epoch) ? it : 1 — js_numberify keeps numbers finite.
    let previous_epoch = match m.get("fence_epoch") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(1.0),
        _ => 1.0,
    };
    let mut adopted = m.clone();
    adopted.insert("session".into(), json!(session));
    adopted.insert("claimed_at".into(), json!(now)); // fresh ownership renews the TTL clock
    match &previous {
        // adopted_from: undefined is DROPPED by JSON.stringify — remove the key.
        None => {
            adopted.shift_remove("adopted_from");
        }
        Some(prev) => {
            adopted.insert("adopted_from".into(), prev.clone());
        }
    }
    adopted.insert("adopted_at".into(), json!(now));
    let epoch = serde_json::Number::from_f64(previous_epoch + 1.0).ok_or(Err2::Ex)?;
    adopted.insert("fence_epoch".into(), Value::Number(epoch));
    write_json_atomic(&claims_dir(root).join(format!("{cell}.json")), &Value::Object(adopted.clone()))
        .map_err(|_| Err2::Ex)?;
    Ok(AdoptOutcome::Adopted { claim: adopted, previous_owner: previous })
}

// ─── handoff (lib/state.mjs legacy single-file path — C1 only) ─────────────

pub(crate) fn handoff_path(root: &Path) -> PathBuf {
    root.join(".bee").join("HANDOFF.json")
}

/// readHandoff — fail-open read; an object gets `kind` normalized
/// (missing/unknown → 'pause'). Non-object values return as-is in Node; the
/// callers here delegate on truthy non-objects.
fn read_handoff(root: &Path) -> Ex<Option<Value>> {
    match read_json(&handoff_path(root)) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Exotic),
        ReadJson::Parsed(v) => {
            let v = js_numberify(&v)?;
            match v {
                Value::Object(mut m) => {
                    let normalized = if matches!(m.get("kind"), Some(Value::String(s)) if s == "planned-next")
                    {
                        "planned-next"
                    } else {
                        "pause"
                    };
                    m.insert("kind".into(), json!(normalized));
                    Ok(Some(Value::Object(m)))
                }
                other => Ok(Some(other)),
            }
        }
    }
}

/// A previous-cell id that Node's path.join would treat as a relative
/// filename but Rust's Path::join could interpret differently (drive letters,
/// absolute prefixes, NULs) delegates instead of risking a different read.
fn cell_path_modelable(id: &str) -> bool {
    !(id.contains(':') || id.starts_with('/') || id.starts_with('\\') || id.contains('\0'))
}

/// writeHandoff (state.mjs) — the strict, guarded legacy writer.
fn write_handoff(root: &Path, input: &Map<String, Value>, kind: &str) -> Result<Map<String, Value>, Err2> {
    let now = now_iso();
    if kind == "pause" {
        let mut record = Map::new();
        for (k, v) in input {
            if k != "kind" {
                record.insert(k.clone(), v.clone());
            }
        }
        record.insert("kind".into(), json!("pause"));
        record.insert("written_at".into(), json!(now));
        write_json_atomic(&handoff_path(root), &Value::Object(record.clone())).map_err(|_| Err2::Ex)?;
        return Ok(record);
    }
    // planned-next: every precondition is READ before the single write.
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
            "writeHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.".to_string(),
        ));
    }
    if !cell_path_modelable(&previous_cell) {
        return Err(Err2::Ex);
    }
    let previous = match read_json(&root.join(".bee").join("cells").join(format!("{previous_cell}.json"))) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => return Err(Err2::Ex), // Node's readJson warns (V8 bytes)
        ReadJson::Parsed(v) => Some(js_numberify(&v)?),
    };
    let prev_capped = previous
        .as_ref()
        .map(|v| truthy(v) && matches!(jget(v, "status"), Some(Value::String(s)) if s == "capped"))
        .unwrap_or(false);
    if !prev_capped {
        // `${previous?.status ?? 'missing'}` — nullish only.
        let status_disp = match previous.as_ref().and_then(|v| jget(v, "status")) {
            None | Some(Value::Null) => "missing".to_string(),
            Some(v) => js_disp(v),
        };
        return Err(Err2::Msg(format!(
            "writeHandoff: refused — previous cell \"{previous_cell}\" is not capped (found status \"{status_disp}\"). A planned-next handoff may only follow a capped cell. FIX: finish \"{previous_cell}\" first (bee.mjs cells finish), then retry."
        )));
    }
    // readClaim(controlRootFor(root), nextCell) — claimPath's requireId throws.
    let next_id = require_id(&next_cell, "cell id")?;
    let claim = read_claim(root, &next_id)?;
    let claim_owned = claim
        .as_ref()
        .map(|c| opt_strict_eq(jget(c, "session"), Some(&Value::String(writer_session.clone()))))
        .unwrap_or(false);
    if !claim_owned {
        let found = match &claim {
            None => "no claim".to_string(),
            Some(c) => format!(
                "owner \"{}\"",
                match jget(c, "session") {
                    Some(v) => js_disp(v),
                    None => "undefined".to_string(),
                }
            ),
        };
        return Err(Err2::Msg(format!(
            "writeHandoff: refused — next cell \"{next_cell}\" has no claim owned by writer session \"{writer_session}\" (found {found}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim \"{next_cell}\" as session \"{writer_session}\" first (claims.mjs claimCellFile), then retry."
        )));
    }
    let mut record = input.clone();
    record.insert("kind".into(), json!("planned-next"));
    record.insert("writer_session".into(), json!(writer_session));
    record.insert("previous_cell".into(), json!(previous_cell));
    record.insert("next_cell".into(), json!(next_cell));
    record.insert("written_at".into(), json!(now));
    write_json_atomic(&handoff_path(root), &Value::Object(record.clone())).map_err(|_| Err2::Ex)?;
    Ok(record)
}

enum HandoffAdopt {
    Fail { reason: String },
    Ok { claim: Map<String, Value>, previous_owner: Option<Value>, next_cell: String },
}

/// adoptHandoff (state.mjs) — clear-after-adopt with idempotent recovery.
fn adopt_handoff(root: &Path, session_id: &str) -> Result<HandoffAdopt, Err2> {
    let handoff = read_handoff(root)?;
    let handoff = match handoff {
        None => {
            return Ok(HandoffAdopt::Fail { reason: "no .bee/HANDOFF.json to adopt.".to_string() })
        }
        Some(v) if !truthy(&v) => {
            return Ok(HandoffAdopt::Fail { reason: "no .bee/HANDOFF.json to adopt.".to_string() })
        }
        Some(Value::Object(m)) => m,
        Some(_) => return Err(Err2::Ex), // truthy non-object — JS property exotica
    };
    // read_handoff normalized kind, so a non-planned-next always reads "pause".
    if !matches!(handoff.get("kind"), Some(Value::String(s)) if s == "planned-next") {
        return Ok(HandoffAdopt::Fail {
            reason: format!(
                "handoff kind \"{}\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).",
                handoff.get("kind").map(js_disp).unwrap_or_default()
            ),
        });
    }
    let next_cell = match handoff.get("next_cell") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if next_cell.is_empty() {
        return Ok(HandoffAdopt::Fail {
            reason: "planned-next handoff has no next_cell to adopt.".to_string(),
        });
    }
    match adopt_claim(root, &next_cell, session_id)? {
        AdoptOutcome::Fail { reason } => Ok(HandoffAdopt::Fail { reason }),
        AdoptOutcome::Adopted { claim, previous_owner } => {
            let _ = std::fs::remove_file(handoff_path(root)); // rmSync force:true
            Ok(HandoffAdopt::Ok { claim, previous_owner, next_cell })
        }
    }
}

// ─── lanes ────────────────────────────────────────────────────────────────
// The lane store (lanesDir/lanePath/requireLaneFeature/defaultLaneRecord/
// laneRecordFrom/readLane/readLaneStrict/writeLane/listLanes) lives in
// verbs/workflow_store.rs — ONE port, shared by the display verbs here and
// by the mutation/projection seams below.

// ─── scribing ledger (lib/cells.mjs) ───────────────────────────────────────

fn scribing_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("scribing-runs.jsonl")
}

/// readJsonl — corrupt lines are skipped silently in Node; a line serde can't
/// parse but V8 might ("\u" escapes) delegates.
fn read_scribing_ledger(root: &Path) -> Ex<Vec<Value>> {
    let bytes = match std::fs::read(scribing_ledger_path(root)) {
        Ok(b) => b,
        Err(_) => return Ok(Vec::new()),
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        match parse_json_v8(trimmed)? {
            ParsedJson::Parsed(v) => events.push(v),
            ParsedJson::Unparseable => continue, // Node skips corrupt lines
        }
    }
    Ok(events)
}

/// cells.mjs scribingRunStampMs: Date.parse(run.at || run.date).
fn scribing_run_stamp_ms(run: Option<&Value>) -> Ex<Option<f64>> {
    let Some(run) = run else { return Ok(None) };
    if !truthy(run) {
        return Ok(None);
    }
    // run.at || run.date — first truthy.
    let at = jget(run, "at").filter(|v| truthy(v)).or_else(|| jget(run, "date"));
    match date_parse_val(at)? {
        Some(ms) => Ok(Some(ms)),
        None => Ok(None),
    }
}

/// bestScribingStampMs — ledger max, then the lane stamp, then the default
/// record's own stamp when it names this same feature.
fn best_scribing_stamp_ms(
    root: &Path,
    feature: &str,
    state: &Map<String, Value>,
) -> Ex<Option<f64>> {
    let mut best: Option<f64> = None;
    let mut consider = |ms: Option<f64>| {
        if let Some(v) = ms {
            if best.map(|b| v > b).unwrap_or(true) {
                best = Some(v);
            }
        }
    };
    for entry in read_scribing_ledger(root)? {
        if !truthy(&entry) {
            continue;
        }
        if !opt_strict_eq(jget(&entry, "feature"), Some(&Value::String(feature.to_string()))) {
            continue;
        }
        consider(date_parse_val(jget(&entry, "ts"))?);
    }
    let lane = read_lane_display(root, feature)?;
    if let Some(lane) = &lane {
        consider(scribing_run_stamp_ms(lane.get("last_scribing_run"))?);
    }
    if let Some(run) = state.get("last_scribing_run") {
        if truthy(run)
            && opt_strict_eq(jget(run, "feature"), Some(&Value::String(feature.to_string())))
        {
            consider(scribing_run_stamp_ms(Some(run))?);
        }
    }
    Ok(best)
}

fn acquire_state_lock(root: &Path) -> Result<lock::LockGuard, Err2> {
    lock::acquire_store_lock(root, "state", lock::MAX_ATTEMPTS).map_err(|b| Err2::Msg(b.message()))
}

// ─── the mutation seam (bee.mjs resolveMutationLockScope / withMutationLock /
//     resolveMutationTarget / write*RecordThroughProjection) ────────────────
//
// This block is what retired the C1 gate. Every record-mutating state verb now
// runs the SAME three steps Node runs, in the same order, against the same
// lock names:
//   1. resolve_mutation_lock_scope — a fail-open PEEK (never the *Strict
//      readers) at which feature/record a mutation will land on.
//   2. acquire_mutation_locks — `workflow:<id>` then the projection lock when
//      a live workflow names that feature; a single 'state' hold otherwise.
//      The global order workflow:<id> → {'state' | lane:<f>} is never inverted.
//   3. resolve_mutation_target — the authoritative strict read, then
//      write_through_projection: updateWorkflowAssumingLock (D1 fields +
//      gates, identity fields protected), the caller's FULL record
//      (writeState/writeLane — GH #86), then the rebuild.

/// bee.mjs resolveMutationLockScope's `{feature, lane}`.
struct Scope {
    feature: Option<String>,
    lane: bool,
}

fn resolve_mutation_lock_scope(
    root: &Path,
    lane_feature: Option<&str>,
    no_lane: bool,
) -> Ex<Scope> {
    if let Some(f) = lane_feature {
        return Ok(Scope { feature: Some(f.to_string()), lane: true });
    }
    if no_lane {
        return Ok(Scope { feature: None, lane: false });
    }
    let (_sid, bound) = session_binding(None, root)?;
    if let Some(bound) = bound {
        return Ok(Scope { feature: Some(bound), lane: true });
    }
    // Fail-open peek; readStateStrict's throw still happens in the target.
    let state = read_state_peek(root)?;
    let feature = match state.get("feature") {
        Some(v) if truthy(v) => Some(js_disp(v)),
        _ => None,
    };
    Ok(Scope { feature, lane: false })
}

/// bee.mjs resolveMutationTarget's return, minus the `write` closure (which
/// becomes `write_through_projection` below).
enum Target {
    /// The default `.bee/state.json` record. `target_feature` is captured at
    /// resolution time, BEFORE any caller mutation (the `--feature` swap
    /// carve-out depends on that).
    Default { record: Map<String, Value>, target_feature: Option<String> },
    Lane { record: Map<String, Value>, lane: String },
}

impl Target {
    fn record(&self) -> &Map<String, Value> {
        match self {
            Target::Default { record, .. } | Target::Lane { record, .. } => record,
        }
    }
    fn record_mut(&mut self) -> &mut Map<String, Value> {
        match self {
            Target::Default { record, .. } | Target::Lane { record, .. } => record,
        }
    }
    fn lane(&self) -> Option<&str> {
        match self {
            Target::Lane { lane, .. } => Some(lane),
            Target::Default { .. } => None,
        }
    }
    /// `target.source === 'lane' ? `lane "${target.lane}"` : 'default state'`.
    fn selected_record(&self) -> String {
        match self {
            Target::Lane { lane, .. } => format!("lane \"{lane}\""),
            Target::Default { .. } => "default state".to_string(),
        }
    }
    /// `${targetLane ? ` (lane "${targetLane}")` : ''}` — every verb's text tail.
    fn lane_note(&self) -> String {
        match self.lane() {
            Some(l) => format!(" (lane \"{l}\")"),
            None => String::new(),
        }
    }
}

fn lane_missing_refusal(verb: &str, lane_feature: &str) -> String {
    format!(
        "{verb}: refused — lane \"{lane_feature}\" does not exist (no .bee/lanes/{lane_feature}.json). FIX: start it first (\"state start-feature --feature {lane_feature} --as-lane\"), then retry."
    )
}

fn bound_lane_missing_refusal(verb: &str, session_id: &str, bound: &str) -> String {
    format!(
        "{verb}: refused — calling session \"{session_id}\" is bound to lane \"{bound}\" but no .bee/lanes/{bound}.json exists; resolution never guesses back to the default record. FIX: start the lane (\"state start-feature --feature {bound} --as-lane\"), unbind the session, or pass --no-lane to target the default record explicitly."
    )
}

fn resolve_mutation_target(
    root: &Path,
    lane_feature: Option<&str>,
    verb: &str,
    no_lane: bool,
) -> Result<Target, Err2> {
    let default_target = |root: &Path| -> Result<Target, Err2> {
        let record = read_state_strict(root)?;
        let target_feature = match record.get("feature") {
            Some(v) if truthy(v) => Some(js_disp(v)),
            _ => None,
        };
        Ok(Target::Default { record, target_feature })
    };
    if let Some(f) = lane_feature {
        let Some(record) = read_lane_strict(root, f)? else {
            return Err(Err2::Msg(lane_missing_refusal(verb, f)));
        };
        return Ok(Target::Lane { record, lane: f.to_string() });
    }
    if no_lane {
        return default_target(root);
    }
    let (sid, bound) = session_binding(None, root)?;
    let Some(bound) = bound else { return default_target(root) };
    let Some(record) = read_lane_strict(root, &bound)? else {
        return Err(Err2::Msg(bound_lane_missing_refusal(verb, &sid_disp(&sid), &bound)));
    };
    Ok(Target::Lane { record, lane: bound })
}

/// The five D1 fields writeLaneRecordThroughProjection /
/// writeStateRecordThroughProjection patch onto the live workflow record.
fn workflow_patch_from_record(
    updated: &Map<String, Value>,
    stamps: &[(String, Value)],
) -> Map<String, Value> {
    let mut patch = Map::new();
    patch.insert("phase".into(), updated.get("phase").cloned().unwrap_or(Value::Null));
    // `updated.mode == null ? null : String(updated.mode)` — loose null check.
    let mode = match updated.get("mode") {
        None | Some(Value::Null) => Value::Null,
        Some(v) => json!(js_disp(v)),
    };
    patch.insert("mode".into(), mode);
    patch.insert("summary".into(), updated.get("summary").cloned().unwrap_or(Value::Null));
    patch.insert(
        "next_action".into(),
        updated.get("next_action").cloned().unwrap_or(Value::Null),
    );
    patch.insert("gates".into(), gates_patch_from_record(updated, stamps));
    patch
}

/// bee.mjs writeLaneRecordThroughProjection + writeStateRecordThroughProjection.
/// Runs INSIDE the caller's `workflow:<id>` hold, so it uses
/// updateWorkflowAssumingLock (never the self-locking form).
fn write_through_projection(
    root: &Path,
    target: &Target,
    updated: &Map<String, Value>,
    stamps: &[(String, Value)],
) -> Result<(), Err2> {
    let workflows = list_workflows(root)?;
    let (routable_feature, is_lane) = match target {
        Target::Lane { lane, .. } => (Some(lane.clone()), true),
        Target::Default { target_feature, .. } => {
            // `routable = targetFeature && updated.feature === targetFeature`
            // — a --feature SWAP deliberately falls to the direct writeState.
            let routable = target_feature.as_ref().is_some_and(|tf| {
                matches!(updated.get("feature"), Some(Value::String(f)) if f == tf)
            });
            (if routable { target_feature.clone() } else { None }, false)
        }
    };
    let wf = routable_feature
        .as_deref()
        .and_then(|f| find_live_workflow(&workflows, f));
    let Some(wf) = wf else {
        // C1 fallback: no live workflow names this target — the direct write.
        return if is_lane { write_lane(root, updated) } else { write_state(root, updated) };
    };
    let id = wf_id(wf);
    update_workflow_assuming_lock(root, &id, workflow_patch_from_record(updated, stamps))?;
    // GH #86: land the caller's FULL record before the rebuild re-reads disk.
    match target {
        Target::Lane { lane, .. } => {
            write_lane(root, updated)?;
            rebuild_lane_projection(root, lane)?;
        }
        Target::Default { .. } => {
            write_state(root, updated)?;
            rebuild_state_projection(root)?;
        }
    }
    Ok(())
}

/// bee.mjs withMutationLock's acquisition, as RAII guards. Dropping the struct
/// releases in reverse order (projection lock first), matching the .mjs's
/// nested `withStoreLock` unwind.
struct MutationLocks {
    _projection: LockGuard,
    _workflow: Option<LockGuard>,
}

fn acquire_mutation_locks(
    root: &Path,
    scope: &Scope,
    workflows: &[Map<String, Value>],
) -> Result<MutationLocks, Err2> {
    let wf = scope
        .feature
        .as_deref()
        .and_then(|f| find_live_workflow(workflows, f));
    match wf {
        Some(wf) => {
            let workflow = acquire_workflow_lock(root, &wf_id(wf))?;
            let projection = acquire_named_lock(
                root,
                &projection_lock_name(scope.lane, scope.feature.as_deref()),
            )?;
            Ok(MutationLocks { _projection: projection, _workflow: Some(workflow) })
        }
        // C1 fallback — deliberately 'state' even for a lane (see the .mjs).
        None => Ok(MutationLocks { _projection: acquire_state_lock(root)?, _workflow: None }),
    }
}

/// The record a mutation will land on, read WITHOUT any warn — used only for
/// the pre-lock delegation decisions (the scribing-debt doors and the
/// high-risk advisor precondition, which live in cells.mjs/state.mjs and are
/// separate R6 debts). A `None` answer means "cannot tell yet"; the same
/// check re-runs under the lock and delegates there instead.
fn peek_target_record(
    root: &Path,
    scope: &Scope,
    lane_feature: Option<&str>,
) -> Ex<Option<Map<String, Value>>> {
    if scope.lane {
        let feature = lane_feature.map(str::to_string).or_else(|| scope.feature.clone());
        let Some(feature) = feature else { return Ok(None) };
        // Silent: read_lane_display would warn on a mismatched record, and a
        // warn before a delegate would double up under Node.
        let Ok(file) = lane_path(root, &feature) else { return Ok(None) };
        let bytes = match std::fs::read(&file) {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };
        let text = String::from_utf8_lossy(&bytes).into_owned();
        return match parse_json_v8(&text)? {
            ParsedJson::Unparseable => Ok(None),
            ParsedJson::Parsed(v) => {
                crate::verbs::workflow_store::lane_record_from(js_trim(&feature), &v)
            }
        };
    }
    Ok(Some(read_state_peek(root)?))
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "state" {
        return None;
    }
    let toks: Vec<&str> = args[1..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // main()'s group-scoped help path
    }
    // splitCommandTokens: leading command tokens end at the first "--" token.
    let split = toks.iter().position(|t| t.starts_with("--")).unwrap_or(toks.len());
    let (leading, rest) = toks.split_at(split);
    let (verb, consumed): (&str, usize) = match leading {
        ["set", ..] => ("set", 1),
        ["gate", ..] => ("gate", 1),
        ["plan-rev", "bump", ..] => ("plan-rev.bump", 2),
        ["worker", "add", ..] => ("worker.add", 2),
        ["worker", "update", ..] => ("worker.update", 2),
        ["worker", "remove", ..] => ("worker.remove", 2),
        ["worker", "clear", ..] => ("worker.clear", 2),
        ["worker", "prune", ..] => ("worker.prune", 2),
        ["scribing-run", ..] => ("scribing-run", 1),
        ["compounding-run", ..] => ("compounding-run", 1),
        ["lanes", ..] => ("lanes", 1),
        ["session", "list", ..] => ("session.list", 2),
        ["session", "bind", ..] => ("session.bind", 2),
        ["session", "unbind", ..] => ("session.unbind", 2),
        ["handoff", "write", ..] => ("handoff.write", 2),
        ["handoff", "adopt", ..] => ("handoff.adopt", 2),
        ["handoff", "show", ..] => ("handoff.show", 2),
        ["workflows", "list", ..] => ("workflows.list", 2),
        ["workflows", "close", ..] => ("workflows.close", 2),
        ["rebuild-projections", ..] => ("rebuild-projections", 1),
        _ => return None, // route/start-feature/advisor-ref/compact-*/unknown → Node
    };
    if leading.len() != consumed {
        return None; // "Unexpected argument" — Node's own refusal path
    }
    let (flags, use_json) = parse_flags(rest)?;
    match verb {
        "set" => run_set(flags, use_json, t0),
        "gate" => run_gate(flags, use_json, t0),
        "plan-rev.bump" => run_plan_rev_bump(flags, use_json, t0),
        "worker.add" => run_worker_add(flags, use_json, t0),
        "worker.update" => run_worker_update(flags, use_json, t0),
        "worker.remove" => run_worker_remove(flags, use_json, t0),
        "worker.clear" => run_worker_clear(flags, use_json, t0),
        "worker.prune" => run_worker_prune(flags, use_json, t0),
        "scribing-run" => run_scribing_run(flags, use_json, t0),
        "compounding-run" => run_compounding_run(flags, use_json, t0),
        "lanes" => run_lanes(flags, use_json, t0),
        "session.list" => run_session_list(flags, use_json, t0),
        "session.bind" => run_session_bind(flags, use_json, t0),
        "session.unbind" => run_session_unbind(flags, use_json, t0),
        "handoff.write" => run_handoff_write(flags, use_json, t0),
        "handoff.adopt" => run_handoff_adopt(flags, use_json, t0),
        "handoff.show" => run_handoff_show(flags, use_json, t0),
        "workflows.list" => run_workflows_list(flags, use_json, t0),
        "workflows.close" => run_workflows_close(flags, use_json, t0),
        "rebuild-projections" => run_rebuild_projections(flags, use_json, t0),
        _ => None,
    }
}

fn go(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Result<Ctx, ExitCode>> {
    match prelude(cmd, use_json, t0)? {
        Pre::Go(c) => Some(Ok(c)),
        Pre::Emitted(code) => Some(Err(code)),
    }
}

// ─── state set ─────────────────────────────────────────────────────────────

fn run_set(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "phase", "mode", "feature", "next-action", "summary", "owner", "lane", "no-lane",
            "waive-scribing-debt", "waive-compounding",
        ],
    ) {
        return None;
    }
    for b in ["no-lane", "waive-scribing-debt", "waive-compounding"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state set", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let phase_flag = flag_string(&flags, "phase");
        if let Some(p) = &phase_flag {
            if !is_known_phase(p) {
                return Ok(Out::Thrown(format!(
                    "set: invalid phase \"{p}\" \u{2014} not in the known-phase enum (isKnownPhase, not the bare PHASES array \u{2014} the terminal alias \"compounding-complete\" must pass). FIX: use one of {KNOWN_PHASES_JOINED}."
                )));
            }
        }
        if ["phase", "mode", "feature", "next-action", "summary"]
            .iter()
            .all(|n| flags.get(n).is_none())
        {
            return Ok(Out::Thrown(
                "set: at least one of --phase, --mode, --feature, --next-action, --summary is required."
                    .to_string(),
            ));
        }
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "set") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        if lane_feature.is_some() && flags.get("feature").is_some() {
            return Ok(Out::Thrown(
                "set: --feature cannot be combined with --lane \u{2014} a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or start a new lane instead.".to_string(),
            ));
        }
        let waive = matches!(flags.get("waive-compounding"), Some(FlagV::Present));

        // withMutationLock's own pre-lock reads: the fail-open scope peek, then
        // the workflow listing that picks the lock names.
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;

        // Strangler: the two scribing-debt doors (cells.mjs scribingDebt +
        // decisions.mjs logDecision — a DIFFERENT R6 debt, for BOTH the lane
        // and default branches) are decided here, BEFORE any lock or output,
        // off a silent peek at the record the strict read will land on.
        if let Some(peek) = peek_target_record(&ctx.root, &scope, lane_feature.as_deref())?
        {
            if let Some(p) = &phase_flag {
                let t = check_phase_transition(peek.get("phase"), p, &peek, waive)?;
                if t.ok && p == "compounding-complete" {
                    return Err(Err2::Ex); // passing close -> the debt door on Node
                }
            }
            if !scope.lane {
                if let Some(f) = flag_string(&flags, "feature") {
                    let current = peek.get("feature");
                    if current.map(truthy).unwrap_or(false)
                        && !opt_strict_eq(current, Some(&Value::String(f.clone())))
                    {
                        return Err(Err2::Ex); // feature swap -> the debt door on Node
                    }
                }
            }
        }

        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "set", no_lane)?;
        // i54-closeout-7 (D7): a session-AUTO-resolved lane refuses --feature too
        // (the flag-level guard above only ever sees an EXPLICIT --lane).
        if flags.get("feature").is_some() {
            if let Some(lane) = target.lane() {
                return Ok(Out::Thrown(format!(
                    "set: --feature cannot target lane \"{lane}\" (auto-resolved from this session's lane binding) \u{2014} a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or pass --no-lane to address the default record."
                )));
            }
        }
        if let Some(p) = &phase_flag {
            let record = target.record();
            let t = check_phase_transition(record.get("phase"), p, record, waive)?;
            if !t.ok {
                return Ok(Out::Thrown(t.reason));
            }
            if p == "compounding-complete" {
                return Err(Err2::Ex); // race: the strict read now passes the door
            }
        }
        if target.lane().is_none() {
            if let Some(f) = flag_string(&flags, "feature") {
                let current = target.record().get("feature");
                if current.map(truthy).unwrap_or(false)
                    && !opt_strict_eq(current, Some(&Value::String(f.clone())))
                {
                    return Err(Err2::Ex);
                }
            }
        }
        let selected = target.selected_record();
        let lane_note = target.lane_note();
        let phase_known =
            matches!(target.record().get("phase"), Some(Value::String(s)) if is_known_phase(s));
        if !phase_known {
            // `${state.phase ?? ''}` — nullish coalescing.
            let disp = match target.record().get("phase") {
                None | Some(Value::Null) => String::new(),
                Some(v) => js_disp(v),
            };
            return Ok(Out::Thrown(format!(
                "set: refused \u{2014} selected {selected} has missing or invalid pre-mutation phase \"{disp}\". Ownership cannot be derived from a corrupt routing record, so nothing was written. FIX: restore a valid phase before retrying."
            )));
        }
        let phase_str = js_disp(target.record().get("phase").unwrap());
        let owner = match flags.get("owner") {
            Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
            _ => {
                return Ok(Out::Thrown(format!(
                    "set: missing --owner \u{2014} selected {selected}'s pre-mutation phase is \"{phase_str}\". FIX: retry with --owner {phase_str}."
                )));
            }
        };
        if owner != phase_str {
            return Ok(Out::Thrown(format!(
                "set: owner mismatch \u{2014} selected {selected}'s pre-mutation phase is \"{phase_str}\", not \"{owner}\". FIX: retry with --owner {phase_str}."
            )));
        }
        let mut changed: Vec<String> = Vec::new();
        {
            let state = target.record_mut();
            if let Some(p) = &phase_flag {
                state.insert("phase".into(), json!(p));
                changed.push(format!("phase={p}"));
            }
            if let Some(m) = flag_string(&flags, "mode") {
                state.insert("mode".into(), json!(m));
                changed.push(format!("mode={m}"));
            }
            if let Some(f) = flag_string(&flags, "feature") {
                state.insert("feature".into(), json!(f));
                changed.push(format!("feature={f}"));
            }
            if let Some(n) = flag_string(&flags, "next-action") {
                state.insert("next_action".into(), json!(n));
                changed.push("next_action".to_string());
            }
            if let Some(s) = flag_string(&flags, "summary") {
                state.insert("summary".into(), json!(s));
                changed.push("summary".to_string());
            }
        }
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &[])?;
        drop(locks);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Updated state: {}.{lane_note}", changed.join(" ")),
            0,
        ))
    })();
    finish(&ctx, out)
}

// ─── state gate ────────────────────────────────────────────────────────────

fn run_gate(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["name", "merge", "approved", "lane", "no-lane", "owner"]) {
        return None;
    }
    for b in ["merge", "no-lane"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state gate", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        if flags.get("owner").is_some() {
            return Ok(Out::Thrown(
                "gate: --owner is not accepted \u{2014} routing ownership protects generic `state set` fields only. FIX: omit --owner and use the dedicated gate command.".to_string(),
            ));
        }
        let merge = matches!(flags.get("merge"), Some(FlagV::Present));
        if merge && flags.get("name").is_some() {
            return Ok(Out::Thrown(
                "gate: --merge cannot be combined with --name \u{2014} --merge always addresses BOTH shape and execution in one call. FIX: pass --merge alone, or drop --merge and use --name to approve a single gate.".to_string(),
            ));
        }
        let spec: Vec<(&str, Option<&[&str]>)> = if merge {
            vec![("approved", Some(&["true", "false"][..]))]
        } else {
            vec![
                ("name", Some(&GATE_NAMES[..])),
                ("approved", Some(&["true", "false"][..])),
            ]
        };
        let values = match require_flags(&flags, &spec, EXAMPLE_GATE) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (name, approved_raw) = if merge {
            (String::new(), values[0].clone())
        } else {
            (values[0].clone(), values[1].clone())
        };
        let approved = approved_raw == "true";
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "gate") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let exec_component = merge || name == "execution";

        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        // requireFreshAdvisorForHighRisk (advisorRefStale, lib/state.mjs) is a
        // separate R6 debt — decided off a silent peek, before any lock.
        if exec_component && approved {
            if let Some(peek) =
                peek_target_record(&ctx.root, &scope, lane_feature.as_deref())?
            {
                if matches!(peek.get("mode"), Some(Value::String(s)) if s == "high-risk") {
                    return Err(Err2::Ex);
                }
            }
        }
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "gate", no_lane)?;
        if exec_component
            && approved
            && matches!(target.record().get("mode"), Some(Value::String(s)) if s == "high-risk")
        {
            return Err(Err2::Ex); // race: the peek missed the high-risk mode
        }
        let lane_note = target.lane_note();
        // multisession-native-9 D7 / validation-diet D15 — the plan-rev stamp is
        // LANE-ONLY and reads the live workflow's CURRENT plan_rev.
        let mut stamps: Vec<(String, Value)> = Vec::new();
        if exec_component {
            if let Some(lane) = target.lane() {
                let live = list_workflows(&ctx.root)?;
                if let Some(wf) = find_live_workflow(&live, lane) {
                    let rev = if approved {
                        wf.get("plan_rev").cloned().unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    if merge {
                        stamps.push(("shape".to_string(), rev.clone()));
                    }
                    stamps.push(("execution".to_string(), rev));
                }
            }
        }
        {
            let state = target.record_mut();
            // Revocation tracking (AO13) — execution component only.
            if exec_component && !approved {
                let mut revoked = match state.get("gate_revoked_at") {
                    Some(Value::Object(m)) => m.clone(),
                    None | Some(Value::Null) | Some(Value::Bool(_)) | Some(Value::Number(_)) => {
                        Map::new()
                    }
                    Some(_) => return Err(Err2::Ex), // string/array spread exotica
                };
                revoked.insert("execution".into(), json!(now_iso()));
                state.insert("gate_revoked_at".into(), Value::Object(revoked));
            }
            let mut gates = match state.get("approved_gates") {
                Some(Value::Object(m)) => m.clone(),
                _ => Map::new(), // both strict readers always merge an object
            };
            if merge {
                gates.insert("shape".into(), json!(approved));
                gates.insert("execution".into(), json!(approved));
            } else {
                gates.insert(name.clone(), json!(approved));
            }
            state.insert("approved_gates".into(), Value::Object(gates));
        }
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &stamps)?;
        drop(locks);
        let text = if merge {
            format!("Gates \"shape\" and \"execution\" set to {approved}.{lane_note}")
        } else {
            format!("Gate \"{name}\" set to {approved}.{lane_note}")
        };
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state plan-rev bump ───────────────────────────────────────────────────

fn run_plan_rev_bump(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state plan-rev bump", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "plan-rev bump") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        // Read-only PEEK outside both locks (splr-1's canonical order:
        // workflow:<id> FIRST, then the projection lock — never the reverse).
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        let peeked_id = scope
            .feature
            .as_deref()
            .and_then(|f| find_live_workflow(&workflows, f))
            .map(wf_id);
        let _workflow_guard = match &peeked_id {
            Some(id) => Some(acquire_workflow_lock(&ctx.root, id)?),
            None => None,
        };
        let _projection_guard = acquire_named_lock(
            &ctx.root,
            &projection_lock_name(scope.lane, scope.feature.as_deref()),
        )?;
        let target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "plan-rev bump", no_lane)?;
        let Some(lane) = target.lane() else {
            return Ok(Out::Thrown(
                "plan-rev bump: refused \u{2014} resolution landed on the default (non-lane) record. plan_rev bumping is scoped to lanes only, by design (nothing else ever reads or bumps the default pipeline's plan_rev, so stamping it would be meaningless). FIX: target a lane explicitly with --lane <feature>, or bind the calling session to one first (\"state session bind --session-id <id> --lane <feature>\").".to_string(),
            ));
        };
        let live = list_workflows(&ctx.root)?;
        let Some(wf) = find_live_workflow(&live, lane) else {
            return Ok(Out::Thrown(format!(
                "plan-rev bump: no live workflow record found for lane \"{lane}\" \u{2014} nothing to bump. FIX: start the lane first (\"state start-feature --feature {lane} --as-lane\")."
            )));
        };
        let id = wf_id(wf);
        // The peek picked which workflow lock is held right now; a different
        // resolution means that lock protects the wrong record.
        if peeked_id.as_deref() != Some(id.as_str()) {
            return Ok(Out::Thrown(format!(
                "plan-rev bump: the target lane's workflow changed while this call was starting (expected \"{}\", resolved \"{id}\"), so the workflow lock this call holds does not protect it. Nothing was written. FIX: re-run the bump.",
                peeked_id.as_deref().unwrap_or("none")
            )));
        }
        let updated = update_workflow_assuming_lock_with(&ctx.root, &id, |current| {
            // `(current.plan_rev || 0) + 1` — a non-numeric plan_rev would take
            // JS's own coercion path (string concat), which this port delegates.
            let base = match current.get("plan_rev") {
                Some(Value::Number(n)) => n.as_f64().ok_or(Err2::Ex)?,
                None | Some(Value::Null) | Some(Value::Bool(false)) => 0.0,
                Some(Value::String(s)) if s.is_empty() => 0.0,
                Some(_) => return Err(Err2::Ex),
            };
            let mut patch = Map::new();
            patch.insert(
                "plan_rev".into(),
                Value::Number(serde_json::Number::from_f64(base + 1.0).ok_or(Err2::Ex)?),
            );
            Ok(patch)
        })?;
        let rebuilt = rebuild_lane_projection(&ctx.root, lane)?;
        let plan_rev = updated.get("plan_rev").cloned().unwrap_or(Value::Null);
        let mut result = Map::new();
        result.insert("feature".into(), json!(lane));
        result.insert("plan_rev".into(), plan_rev.clone());
        result.insert(
            "lane".into(),
            rebuilt.map(Value::Object).unwrap_or(Value::Null),
        );
        let text = format!(
            "Bumped plan_rev to {} for lane \"{lane}\" (workflow); lane projection rebuilt.",
            js_disp(&plan_rev)
        );
        Ok(Out::Emit(Value::Object(result), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state worker add/update/remove/clear ──────────────────────────────────

/// stateWorkerMutate — the shared lock + strict-read + write frame.
fn worker_mutate(
    root: &Path,
    mutate: impl FnOnce(&mut Vec<Value>) -> Result<String, Err2>,
) -> R2<Out> {
    let guard = acquire_state_lock(root)?;
    let mut state = read_state_strict(root)?;
    let mut workers: Vec<Value> = match state.get("workers") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let text = mutate(&mut workers)?;
    state.insert("workers".into(), Value::Array(workers));
    write_state(root, &state)?;
    drop(guard);
    Ok(Out::Emit(Value::Object(state), text, 0))
}

/// A thrown-Error inside the mutate closure must surface as emitError, not a
/// delegate — collapse Err2::Msg into Out::Thrown at the boundary.
fn thrown_ok(out: R2<Out>) -> R2<Out> {
    match out {
        Err(Err2::Msg(m)) => Ok(Out::Thrown(m)),
        other => other,
    }
}

fn run_worker_add(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname", "cell", "tier", "status"]) {
        return None;
    }
    let ctx = match go("state worker add", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let cell = require_flag(&flags, "cell")?;
        let tier: Value = match flag_string(&flags, "tier") {
            None => Value::Null,
            Some(t) => {
                if !MODEL_TIERS.contains(&t.as_str()) {
                    return Err(Err2::Msg(format!(
                        "worker add: invalid tier \"{t}\" — must be one of extraction, generation, ceiling."
                    )));
                }
                json!(t)
            }
        };
        let status: Value = match flag_string(&flags, "status") {
            None => Value::Null,
            Some(s) => json!(s),
        };
        workers.push(json!({"nickname": nickname, "cell": cell, "tier": tier, "status": status}));
        Ok(format!("Added worker \"{nickname}\" (cell {cell})."))
    }));
    finish(&ctx, out)
}

fn run_worker_update(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname", "cell", "tier", "status"]) {
        return None;
    }
    let ctx = match go("state worker update", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let idx = workers.iter().position(|w| {
            truthy(w) && opt_strict_eq(jget(w, "nickname"), Some(&Value::String(nickname.clone())))
        });
        let Some(idx) = idx else {
            return Err(Err2::Msg(format!(
                "worker update: nickname \"{nickname}\" not found — use \"worker add\" to create it first."
            )));
        };
        // const worker = { ...workers[idx] } — always an object once matched.
        let mut worker = match &workers[idx] {
            Value::Object(m) => m.clone(),
            _ => return Err(Err2::Ex),
        };
        if let Some(c) = flag_string(&flags, "cell") {
            worker.insert("cell".into(), json!(c));
        }
        if let Some(t) = flag_string(&flags, "tier") {
            if !MODEL_TIERS.contains(&t.as_str()) {
                return Err(Err2::Msg(format!(
                    "worker update: invalid tier \"{t}\" — must be one of extraction, generation, ceiling."
                )));
            }
            worker.insert("tier".into(), json!(t));
        }
        if let Some(s) = flag_string(&flags, "status") {
            worker.insert("status".into(), json!(s));
        }
        workers[idx] = Value::Object(worker);
        Ok(format!("Updated worker \"{nickname}\"."))
    }));
    finish(&ctx, out)
}

fn run_worker_remove(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname"]) {
        return None;
    }
    let ctx = match go("state worker remove", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let before = workers.len();
        workers.retain(|w| {
            !(truthy(w)
                && opt_strict_eq(jget(w, "nickname"), Some(&Value::String(nickname.clone()))))
        });
        if workers.len() == before {
            return Err(Err2::Msg(format!("worker remove: nickname \"{nickname}\" not found.")));
        }
        Ok(format!("Removed worker \"{nickname}\"."))
    }));
    finish(&ctx, out)
}

fn run_worker_clear(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state worker clear", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let removed = workers.len();
        workers.clear();
        Ok(format!("Cleared {removed} worker(s)."))
    }));
    finish(&ctx, out)
}

// ─── state worker prune ────────────────────────────────────────────────────

/// WORKER_TRANSIENT_SUFFIX — leftmost match of
/// /\.(prompt\.md|result\.md|result\.json|out\d*\.log|log)$/, returning the
/// matched suffix length.
fn worker_transient_suffix_len(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] != b'.' {
            continue;
        }
        let tail = &name[i..];
        let matched = tail == ".prompt.md"
            || tail == ".result.md"
            || tail == ".result.json"
            || tail == ".log"
            || tail
                .strip_prefix(".out")
                .map(|rest| {
                    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                    &rest[digits..] == ".log"
                })
                .unwrap_or(false);
        if matched {
            return Some(name.len() - i);
        }
    }
    None
}

/// keptByPruneKeepSet — "<id>" or "<id>.<anything>" is protected.
fn kept_by_keep_set(name: &str, keep: &[String]) -> bool {
    keep.iter()
        .any(|id| name == id || name.starts_with(&format!("{id}.")))
}

/// readPruneKeepSet — strict state read + non-capped/corrupt cell stems.
fn read_prune_keep_set(root: &Path) -> Result<Vec<String>, Err2> {
    let state = read_state_strict(root)?;
    let workers = match state.get("workers") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) => a.clone(),
        Some(_) => {
            return Err(Err2::Msg(
                "worker prune: state.workers is not an array — refusing to prune against a malformed keep set (a destructive verb fails closed). FIX: repair .bee/state.json via the bee_state.mjs worker verbs first.".to_string(),
            ));
        }
    };
    let mut keep: Vec<String> = Vec::new();
    let mut push_unique = |s: String| {
        if !keep.contains(&s) {
            keep.push(s);
        }
    };
    for w in &workers {
        if !truthy(w) {
            continue;
        }
        match jget(w, "cell") {
            None | Some(Value::Null) => {}
            Some(cell) => push_unique(js_disp(cell)),
        }
    }
    let cells_dir = root.join(".bee").join("cells");
    if cells_dir.exists() {
        let entries = std::fs::read_dir(&cells_dir).map_err(|_| Err2::Ex)?;
        for entry in entries.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = file.strip_suffix(".json") else { continue };
            let capped = match std::fs::read(cells_dir.join(&file)) {
                Err(_) => false, // JSON.parse throws → cell null → keep
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    match parse_json_v8(&text).map_err(Err2::from)? {
                        ParsedJson::Unparseable => false,
                        ParsedJson::Parsed(v) => {
                            truthy(&v)
                                && matches!(jget(&v, "status"), Some(Value::String(s)) if s == "capped")
                        }
                    }
                }
            };
            if !capped {
                push_unique(stem.to_string());
            }
        }
    }
    Ok(keep)
}

fn run_worker_prune(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["dry-run"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "dry-run") {
        return None;
    }
    let ctx = match go("state worker prune", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let dry_run = flags.get("dry-run").is_some();
        let workers_dir = ctx.root.join(".bee").join("workers");
        let keep = match read_prune_keep_set(&ctx.root) {
            Ok(k) => k,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let mut candidates: Vec<String> = Vec::new();
        let mut kept: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&workers_dir) {
            for entry in entries.flatten() {
                let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
                if !is_file {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                let Some(suffix_len) = worker_transient_suffix_len(&name) else { continue };
                if name.len() == suffix_len {
                    continue; // empty stem is not a transient
                }
                if kept_by_keep_set(&name, &keep) {
                    kept.push(name);
                    continue;
                }
                candidates.push(name);
            }
        }
        let mut pruned: Vec<String> = Vec::new();
        if dry_run {
            pruned.extend(candidates);
        } else if !candidates.is_empty() {
            // C1: re-read the keep set immediately before the destructive loop.
            let keep2 = match read_prune_keep_set(&ctx.root) {
                Ok(k) => k,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            for name in candidates {
                if kept_by_keep_set(&name, &keep2) {
                    kept.push(name);
                    continue;
                }
                let path = workers_dir.join(&name);
                if let Err(e) = std::fs::remove_file(&path) {
                    // fs.rmSync throws — reconstruct the Node errno message for
                    // the two realistic classes (documented approximation).
                    let msg = match e.kind() {
                        std::io::ErrorKind::NotFound => format!(
                            "ENOENT: no such file or directory, rm '{}'",
                            path.display()
                        ),
                        _ => format!("EPERM: operation not permitted, rm '{}'", path.display()),
                    };
                    return Ok(Out::Thrown(msg));
                }
                pruned.push(name);
            }
        }
        js_sort(&mut pruned);
        js_sort(&mut kept);
        let verb = if dry_run { "Would prune" } else { "Pruned" };
        let text = format!(
            "{verb} {} worker transient(s) from .bee/workers/ (kept {} still-active).",
            pruned.len(),
            kept.len()
        );
        Ok(Out::Emit(json!({"dry_run": dry_run, "pruned": pruned, "kept": kept}), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state scribing-run ────────────────────────────────────────────────────

fn run_scribing_run(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "areas", "next-action", "lane", "no-lane", "show"]) {
        return None;
    }
    for b in ["no-lane", "show"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state scribing-run", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        // sqs-b3: --show is a READ-ONLY query mode, above every write-side check.
        if matches!(flags.get("show"), Some(FlagV::Present)) {
            let show_feature = flag_string(&flags, "feature");
            let stamp_ms: Option<f64> = match &show_feature {
                Some(f) => {
                    let state = read_state_peek(&ctx.root)?;
                    best_scribing_stamp_ms(&ctx.root, f, &state)?
                }
                None => {
                    let mut best: Option<f64> = None;
                    for entry in read_scribing_ledger(&ctx.root)? {
                        if !truthy(&entry) {
                            continue;
                        }
                        if let Some(ms) = date_parse_val(jget(&entry, "ts"))? {
                            if best.map(|b| ms > b).unwrap_or(true) {
                                best = Some(ms);
                            }
                        }
                    }
                    best
                }
            };
            let stamp_iso = match stamp_ms {
                Some(ms) => Some(iso_from_ms(ms)?),
                None => None,
            };
            let feature_v = show_feature.as_ref().map(|f| json!(f)).unwrap_or(Value::Null);
            let for_note = show_feature
                .as_ref()
                .map(|f| format!(" for \"{f}\""))
                .unwrap_or_default();
            let text = match &stamp_iso {
                Some(iso) => format!("Last scribing run{for_note}: {iso}"),
                None => format!("No scribing run recorded{for_note}."),
            };
            let stamp_v = stamp_iso.map(|s| json!(s)).unwrap_or(Value::Null);
            return Ok(Out::Emit(json!({"feature": feature_v, "stamp": stamp_v}), text, 0));
        }
        // write path
        let values = match require_flags(
            &flags,
            &[("feature", None), ("areas", None), ("next-action", None)],
            EXAMPLE_SCRIBING,
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (feature, areas_raw, next_action) =
            (values[0].clone(), values[1].clone(), values[2].clone());
        let areas: Vec<Value> = split_list(&areas_raw).into_iter().map(|s| json!(s)).collect();
        let at = now_iso();
        let date = at[..10].to_string();
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "scribing-run") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "scribing-run", no_lane)?;
        let lane_note = target.lane_note();
        let is_lane = target.lane().is_some();
        let active_feature_at_call = target.record().get("feature").cloned().unwrap_or(Value::Null);
        // tst-1: only a call that ACTUALLY produces a phase transition on the
        // record it targets passes the D3 door — a lane call always does; a
        // default-record call only when it stamps its OWN active feature (or
        // none). A mismatch is the si-1 ledger-only repair path.
        let stamped_active = is_lane
            || !truthy(&active_feature_at_call)
            || opt_strict_eq(
                Some(&active_feature_at_call),
                Some(&Value::String(feature.clone())),
            );
        if stamped_active {
            if let Some(reason) = check_scribing_run_phase(target.record().get("phase")) {
                return Ok(Out::Thrown(reason));
            }
            {
                let state = target.record_mut();
                let mut run = Map::new();
                run.insert("feature".into(), json!(feature));
                run.insert("date".into(), json!(date));
                run.insert("at".into(), json!(at));
                run.insert("areas_synced".into(), Value::Array(areas.clone()));
                run.insert("next_action".into(), json!(next_action));
                state.insert("last_scribing_run".into(), Value::Object(run));
                state.insert("phase".into(), json!("compounding"));
                state.insert("next_action".into(), json!(next_action));
            }
            let record = target.record().clone();
            write_through_projection(&ctx.root, &target, &record, &[])?;
        }
        let record = target.record().clone();
        drop(locks);
        // si-1: the durable ledger append — ALWAYS, even on the repair path.
        // Fail-open; the Node warning embeds a Node error message, not replicated.
        let _ = append_jsonl(
            &scribing_ledger_path(&ctx.root),
            &json!({"ts": at, "feature": feature, "areas": areas}),
        );
        let repair_note = if stamped_active {
            String::new()
        } else {
            format!(
                " \u{2014} recorded in the durable ledger only: the default record tracks feature \"{}\", not \"{feature}\", so its phase/last_scribing_run were left untouched (repair path for an orphaned feature; `bee status --json`'s scribing_debt.orphaned names it).",
                js_disp(&active_feature_at_call)
            )
        };
        let text = format!("Recorded scribing run for \"{feature}\" at {at}.{lane_note}{repair_note}");
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state compounding-run ─────────────────────────────────────────────────

fn run_compounding_run(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "learnings", "next-action", "lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state compounding-run", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let values = match require_flags(
            &flags,
            &[("feature", None), ("learnings", None)],
            EXAMPLE_COMPOUNDING,
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (feature, learnings) = (values[0].clone(), values[1].clone());
        let next_action = flag_string(&flags, "next-action");
        let at = now_iso();
        let date = at[..10].to_string();
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "compounding-run") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target = resolve_mutation_target(
            &ctx.root,
            lane_feature.as_deref(),
            "compounding-run",
            no_lane,
        )?;
        let lane_note = target.lane_note();
        if let Some(reason) = check_compounding_run_phase(target.record().get("phase")) {
            return Ok(Out::Thrown(reason));
        }
        {
            let state = target.record_mut();
            let mut run = Map::new();
            run.insert("feature".into(), json!(feature));
            run.insert("date".into(), json!(date));
            run.insert("at".into(), json!(at));
            run.insert("learnings".into(), json!(learnings));
            run.insert(
                "next_action".into(),
                next_action.as_ref().map(|n| json!(n)).unwrap_or(Value::Null),
            );
            state.insert("last_compounding_run".into(), Value::Object(run));
            if let Some(n) = &next_action {
                state.insert("next_action".into(), json!(n));
            }
        }
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &[])?;
        drop(locks);
        let text = format!("Recorded compounding run for \"{feature}\" at {at}.{lane_note}");
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state lanes / session list ────────────────────────────────────────────

fn run_lanes(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state lanes", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lanes = list_lanes(&ctx.root)?;
        let sessions = list_session_records(&ctx.root)?;
        // boundBy: lane feature -> bound session ids (session.lane, string+truthy).
        let mut bound_by: Vec<(String, Vec<Value>)> = Vec::new();
        for session in &sessions {
            let Some(Value::String(lane)) = session.get("lane") else { continue };
            if lane.is_empty() {
                continue;
            }
            let id = session.get("id").cloned().unwrap_or(Value::Null);
            match bound_by.iter_mut().find(|(k, _)| k == lane) {
                Some((_, ids)) => ids.push(id),
                None => bound_by.push((lane.clone(), vec![id])),
            }
        }
        let mut rows: Vec<Value> = Vec::new();
        let mut lines: Vec<String> = Vec::new();
        for lane in &lanes {
            let feature = match lane.get("feature") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            let bound: Vec<Value> = bound_by
                .iter()
                .find(|(k, _)| *k == feature)
                .map(|(_, ids)| ids.clone())
                .unwrap_or_default();
            let mut row = lane.clone();
            row.insert("bound_sessions".into(), Value::Array(bound.clone()));
            let gates_obj = lane.get("approved_gates");
            let gates = GATE_NAMES
                .iter()
                .map(|g| {
                    let approved = gates_obj
                        .and_then(|v| jget(v, g))
                        .map(truthy)
                        .unwrap_or(false);
                    format!("{g}={}", if approved { "approved" } else { "pending" })
                })
                .collect::<Vec<_>>()
                .join(" ");
            let bindings_note = if bound.is_empty() {
                String::new()
            } else {
                format!(
                    " sessions={}",
                    bound.iter().map(js_disp).collect::<Vec<_>>().join(",")
                )
            };
            lines.push(format!(
                "{} [{}] {gates}{bindings_note}",
                js_disp(lane.get("feature").unwrap_or(&Value::Null)),
                js_disp(lane.get("phase").unwrap_or(&Value::Null)),
            ));
            rows.push(Value::Object(row));
        }
        let text = if rows.is_empty() {
            "No lane records.".to_string()
        } else {
            lines.join("\n")
        };
        Ok(Out::Emit(Value::Array(rows), text, 0))
    })();
    finish(&ctx, out)
}

fn run_session_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state session list", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let sessions = list_session_records(&ctx.root)?;
        let lines: Vec<String> = sessions
            .iter()
            .map(|s| {
                let lane_note = match s.get("lane") {
                    Some(Value::String(l)) if !l.is_empty() => format!("-> lane \"{l}\""),
                    _ => "(unbound)".to_string(),
                };
                let disp = |key: &str| match s.get(key) {
                    Some(v) => js_disp(v),
                    None => "undefined".to_string(),
                };
                format!(
                    "{} {lane_note} | started {} | heartbeat {}",
                    disp("id"),
                    disp("started_at"),
                    disp("last_heartbeat")
                )
            })
            .collect();
        let text = if sessions.is_empty() {
            "No session records.".to_string()
        } else {
            lines.join("\n")
        };
        let result = Value::Array(sessions.into_iter().map(Value::Object).collect());
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

// ─── state session bind / unbind ───────────────────────────────────────────

fn run_session_bind(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id", "lane"]) {
        return None;
    }
    let ctx = match go("state session bind", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id_raw = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane_raw = match require_flag(&flags, "lane") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        // bindSessionLane: requireId runs BEFORE the lock is touched at all.
        let session = match require_id(&session_id_raw, "session id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane = match require_id(&lane_raw, "lane feature") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let Some(guard) = acquire_sessions_lock(&ctx.root) else {
            return Ok(Out::Thrown(format!(
                "session bind: session \"{session}\" bind to lane \"{lane}\" could not acquire the sessions lock after 15 bounded attempts — never waited unboundedly."
            )));
        };
        let record = read_session(&ctx.root, &session)?;
        let Some(mut record) = record else {
            return Ok(Out::Thrown(format!(
                "session bind: session \"{session}\" has no record to bind to lane \"{lane}\"."
            )));
        };
        record.insert("lane".into(), json!(lane));
        write_json_atomic(
            &sessions_dir(&ctx.root).join(format!("{session}.json")),
            &Value::Object(record.clone()),
        )
        .map_err(|_| Err2::Ex)?;
        drop(guard);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Session \"{session_id_raw}\" bound to lane \"{lane_raw}\"."),
            0,
        ))
    })();
    finish(&ctx, out)
}

fn run_session_unbind(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id"]) {
        return None;
    }
    let ctx = match go("state session unbind", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id_raw = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let session = match require_id(&session_id_raw, "session id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let Some(guard) = acquire_sessions_lock(&ctx.root) else {
            return Ok(Out::Thrown(format!(
                "session unbind: session \"{session}\" unbind could not acquire the sessions lock after 15 bounded attempts — never waited unboundedly."
            )));
        };
        let record = read_session(&ctx.root, &session)?;
        let Some(mut record) = record else {
            return Ok(Out::Thrown(format!(
                "session unbind: session \"{session}\" has no record to unbind."
            )));
        };
        // `const { lane, ...unbound } = record` — the key is OMITTED entirely.
        record.shift_remove("lane");
        write_json_atomic(
            &sessions_dir(&ctx.root).join(format!("{session}.json")),
            &Value::Object(record.clone()),
        )
        .map_err(|_| Err2::Ex)?;
        drop(guard);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Session \"{session_id_raw}\" unbound from its lane."),
            0,
        ))
    })();
    finish(&ctx, out)
}

// ─── state handoff write / adopt / show ────────────────────────────────────
//
// multisession-native-15 (D5): each verb first resolves WHICH workflow it
// targets and, when one resolves, reads/writes/adopts THAT workflow's own
// mailbox (.bee/runtime/handoffs/<workflow-id>/NNNN.json) instead of the
// single legacy .bee/HANDOFF.json — every mailbox mutation then rebuilds the
// legacy file as a display projection. A repo with zero workflow records (C1),
// or a call where nothing resolves, keeps the legacy single-file path.

/// bee.mjs resolveHandoffWorkflowId — explicit --lane > the calling session's
/// bound lane > the DEFAULT record's own live workflow > null. A --lane or a
/// bound session naming NO live workflow refuses loudly (never guesses back).
fn resolve_handoff_workflow_id(
    root: &Path,
    lane_feature: Option<&str>,
    session_id_flag: Option<&str>,
) -> Result<Option<String>, Err2> {
    let workflows = list_workflows(root)?;
    if workflows.is_empty() {
        return Ok(None); // C1: no workflow records anywhere.
    }
    if let Some(f) = lane_feature {
        return match find_live_workflow(&workflows, f) {
            Some(wf) => Ok(Some(wf_id(wf))),
            None => Err(Err2::Msg(format!(
                "state handoff: refused \u{2014} --lane \"{f}\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"{f}\" and status !== closed). FIX: start it first (\"state start-feature --feature {f} --as-lane\"), or omit --lane."
            ))),
        };
    }
    let (sid, bound) = session_binding(session_id_flag, root)?;
    if let Some(bound) = bound {
        return match find_live_workflow(&workflows, &bound) {
            Some(wf) => Ok(Some(wf_id(wf))),
            None => Err(Err2::Msg(format!(
                "state handoff: refused \u{2014} calling session \"{}\" is bound to lane \"{bound}\" but no live workflow names it. FIX: start the lane, unbind the session, or pass --lane explicitly.",
                sid_disp(&sid)
            ))),
        };
    }
    let default_record = read_state_strict(root)?;
    if let Some(v) = default_record.get("feature") {
        if truthy(v) {
            if let Some(wf) = find_live_workflow(&workflows, &js_disp(v)) {
                return Ok(Some(wf_id(wf)));
            }
        }
    }
    Ok(None) // nothing resolves — the legacy single-file path handles this call
}

fn run_handoff_write(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "kind", "writer-session", "previous-cell", "next-cell", "cell", "files", "done",
            "remaining", "feature", "phase", "mode", "next-action", "lane", "target-role",
            "session-id",
        ],
    ) {
        return None;
    }
    // validate(): `kind` is REQUIRED (and therefore enum-enforced) — a
    // missing/out-of-enum kind is the generic STDOUT refusal → delegate.
    let kind = match flags.get("kind") {
        Some(FlagV::S(s)) if s == "planned-next" || s == "pause" => s.clone(),
        _ => return None,
    };
    let ctx = match go("state handoff write", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lane = match optional_lane_flag(&flags, "state handoff write") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let mut input = Map::new();
        input.insert("kind".into(), json!(kind));
        if let Some(v) = flag_string(&flags, "feature") {
            input.insert("feature".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "phase") {
            input.insert("phase".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "mode") {
            input.insert("mode".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "next-action") {
            input.insert("next_action".into(), json!(v));
        }
        if kind == "planned-next" {
            for (flag, key) in [
                ("writer-session", "writer_session"),
                ("previous-cell", "previous_cell"),
                ("next-cell", "next_cell"),
            ] {
                match require_flag(&flags, flag) {
                    Ok(v) => {
                        input.insert(key.into(), json!(v));
                    }
                    Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                    Err(Err2::Ex) => return Err(Err2::Ex),
                }
            }
        } else {
            if let Some(v) = flag_string(&flags, "cell") {
                input.insert("cell".into(), json!(v));
            }
            for (flag, key) in [("files", "files"), ("done", "done"), ("remaining", "remaining")] {
                if let Some(v) = flag_string(&flags, flag) {
                    let list: Vec<Value> = split_list(&v).into_iter().map(|s| json!(s)).collect();
                    input.insert(key.into(), Value::Array(list));
                }
            }
        }
        let workflow_id = match resolve_handoff_workflow_id(
            &ctx.root,
            lane.as_deref(),
            flag_value(&flags, "session-id").as_deref(),
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        if let Some(wid) = workflow_id {
            let record = match write_mailbox_handoff(
                &ctx.root,
                &wid,
                &input,
                target_role.as_deref(),
            ) {
                Ok(r) => r,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            rebuild_handoff_projection(&ctx.root)?;
            let kind_disp = js_disp_opt(record.get("kind"));
            let seq_disp = js_disp_opt(record.get("seq"));
            let text = format!(
                "Wrote \"{kind_disp}\" handoff to workflow \"{wid}\" mailbox (seq {seq_disp})."
            );
            return Ok(Out::Emit(Value::Object(record), text, 0));
        }
        // Legacy single-file path (C1).
        let record = match write_handoff(&ctx.root, &input, &kind) {
            Ok(r) => r,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let kind_disp = js_disp(record.get("kind").unwrap_or(&Value::Null));
        Ok(Out::Emit(
            Value::Object(record),
            format!("Wrote \"{kind_disp}\" handoff."),
            0,
        ))
    })();
    finish(&ctx, out)
}

fn run_handoff_adopt(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id", "lane", "target-role"]) {
        return None;
    }
    // validate(): `session-id` is REQUIRED — missing/empty delegates.
    if !matches!(flags.get("session-id"), Some(FlagV::S(s)) if !s.is_empty()) {
        return None;
    }
    let ctx = match go("state handoff adopt", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane = match optional_lane_flag(&flags, "state handoff adopt") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let workflow_id =
            match resolve_handoff_workflow_id(&ctx.root, lane.as_deref(), Some(&session_id)) {
                Ok(v) => v,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
        if let Some(wid) = workflow_id {
            let adopted = match adopt_mailbox_handoff(
                &ctx.root,
                &wid,
                &session_id,
                target_role.as_deref(),
            ) {
                Ok(v) => v,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            return match adopted {
                MailboxAdopt::Fail { reason } => {
                    Ok(Out::Thrown(format!("state handoff adopt: {reason}")))
                }
                MailboxAdopt::Ok { claim, previous_owner, next_cell, workflow_id, seq } => {
                    rebuild_handoff_projection(&ctx.root)?;
                    let mut result = Map::new();
                    result.insert("ok".into(), json!(true));
                    result.insert("claim".into(), claim.unwrap_or(Value::Null));
                    if let Some(prev) = previous_owner {
                        // undefined is dropped by JSON.stringify.
                        result.insert("previous_owner".into(), prev);
                    }
                    result.insert("next_cell".into(), json!(next_cell));
                    result.insert("workflow_id".into(), json!(workflow_id));
                    result.insert("seq".into(), json!(seq));
                    let text = format!(
                        "Adopted the handoff's carried claim on \"{next_cell}\" into session \"{session_id}\" (workflow \"{wid}\"); handoff cleared."
                    );
                    Ok(Out::Emit(Value::Object(result), text, 0))
                }
            };
        }
        // Legacy single-file path (C1).
        match adopt_handoff(&ctx.root, &session_id) {
            Err(Err2::Msg(m)) => Ok(Out::Thrown(m)), // requireId's own throws, unprefixed
            Err(Err2::Ex) => Err(Err2::Ex),
            Ok(HandoffAdopt::Fail { reason }) => {
                Ok(Out::Thrown(format!("state handoff adopt: {reason}")))
            }
            Ok(HandoffAdopt::Ok { claim, previous_owner, next_cell }) => {
                let mut result = Map::new();
                result.insert("ok".into(), json!(true));
                result.insert("claim".into(), Value::Object(claim));
                if let Some(prev) = previous_owner {
                    result.insert("previous_owner".into(), prev);
                }
                result.insert("next_cell".into(), json!(next_cell));
                let text = format!(
                    "Adopted the handoff's carried claim on \"{next_cell}\" into session \"{session_id}\"; handoff cleared."
                );
                Ok(Out::Emit(Value::Object(result), text, 0))
            }
        }
    })();
    finish(&ctx, out)
}

fn run_handoff_show(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "target-role", "session-id"]) {
        return None;
    }
    let ctx = match go("state handoff show", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lane = match optional_lane_flag(&flags, "state handoff show") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let workflow_id = match resolve_handoff_workflow_id(
            &ctx.root,
            lane.as_deref(),
            flag_value(&flags, "session-id").as_deref(),
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let handoff: Option<Value> = match &workflow_id {
            Some(wid) => {
                newest_open_handoff_mailbox_record(&ctx.root, wid, target_role.as_deref())?
                    .map(Value::Object)
            }
            None => read_handoff(&ctx.root)?,
        };
        let m = match handoff {
            None => return Ok(Out::Emit(Value::Null, "No handoff.".to_string(), 0)),
            Some(v) if !truthy(&v) => {
                return Ok(Out::Emit(Value::Null, "No handoff.".to_string(), 0))
            }
            Some(Value::Object(m)) => m,
            Some(_) => return Err(Err2::Ex), // truthy non-object — JS property exotica
        };
        // `${h.feature ?? 'unknown'}` — nullish coalescing per field.
        let field = |key: &str| match m.get(key) {
            None | Some(Value::Null) => "unknown".to_string(),
            Some(v) => js_disp(v),
        };
        let text = format!(
            "kind={} feature={} phase={} mode={}",
            js_disp(m.get("kind").unwrap_or(&Value::Null)),
            field("feature"),
            field("phase"),
            field("mode"),
        );
        Ok(Out::Emit(Value::Object(m), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state workflows list / close (workflow-lifecycle wl-2) ────────────────

fn run_workflows_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state workflows list", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let mut records = list_workflows(&ctx.root)?;
        workflows_list_sort(&mut records)?;
        let text = if records.is_empty() {
            "No workflow records.".to_string()
        } else {
            records
                .iter()
                .map(|r| {
                    format!(
                        "{} feature={} status={} phase={} created_at={}",
                        js_disp_opt(r.get("id")),
                        js_disp_opt(r.get("feature")),
                        js_disp_opt(r.get("status")),
                        js_disp_opt(r.get("phase")),
                        js_disp_opt(r.get("created_at")),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let result = Value::Array(records.into_iter().map(Value::Object).collect());
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

/// bee.mjs resolveActiveFeatureForWorkflowsClose (review-p1-fixes p1-3, F5):
/// `Ok(feature)` on a real resolution (None is a definite "idle" answer),
/// `Err(reason)` when resolution ITSELF failed.
fn resolve_active_feature_for_workflows_close(
    root: &Path,
) -> Ex<Result<Option<String>, String>> {
    match resolve_mutation_target(root, None, "workflows close", false) {
        Ok(t) => Ok(Ok(match t.record().get("feature") {
            Some(v) if truthy(v) => Some(js_disp(v)),
            _ => None,
        })),
        Err(Err2::Msg(m)) => Ok(Err(m)),
        Err(Err2::Ex) => Err(Exotic),
    }
}

/// The shared tail of both unresolved-active refusals (F5).
fn workflows_close_unresolved_active_tail(reason: &str) -> String {
    format!(
        "Underlying resolution failure: {reason}\nA guard that cannot establish its precondition refuses \u{2014} it never proceeds on a null active feature.\nFIX: repair the routing record named above (restore .bee/state.json, start or unbind the session-bound lane), then retry \u{2014} or close one record explicitly with `bee state workflows close --id <id>`, the one mode that never consults the active feature."
    )
}

fn closed_row(record: &Map<String, Value>) -> Value {
    let mut row = Map::new();
    row.insert("id".into(), record.get("id").cloned().unwrap_or(Value::Null));
    row.insert(
        "feature".into(),
        record.get("feature").cloned().unwrap_or(Value::Null),
    );
    Value::Object(row)
}

fn run_workflows_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "id", "all-but-active"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "all-but-active") {
        return None;
    }
    let ctx = match go("state workflows close", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let has_feature = matches!(flags.get("feature"), Some(FlagV::S(s)) if !s.is_empty());
        let has_id = matches!(flags.get("id"), Some(FlagV::S(s)) if !s.is_empty());
        let has_all_but_active = matches!(flags.get("all-but-active"), Some(FlagV::Present));
        let mode_count =
            [has_feature, has_id, has_all_but_active].iter().filter(|b| **b).count();
        if mode_count != 1 {
            return Ok(Out::Thrown(format!(
                "workflows close: requires exactly one of --feature <feature>, --id <id>, or --all-but-active. Example: {EXAMPLE_WORKFLOWS_CLOSE}"
            )));
        }
        // F5: computed for every mode, CONSULTED only by the two it protects.
        let active = resolve_active_feature_for_workflows_close(&ctx.root)?;
        let active_feature = match &active {
            Ok(f) => f.clone(),
            Err(_) => None,
        };

        if has_id {
            let id = flag_string(&flags, "id").unwrap_or_default();
            let records = list_workflows(&ctx.root)?;
            let live = records.iter().find(|r| {
                js_strict_eq(r.get("id").unwrap_or(&Value::Null), &Value::String(id.clone()))
                    && !js_strict_eq(r.get("status").unwrap_or(&Value::Null), &json!("closed"))
            });
            if live.is_none() {
                return Ok(Out::Thrown(format!(
                    "workflows close --id: no live workflow record found with id \"{id}\"."
                )));
            }
            let mut patch = Map::new();
            patch.insert("status".into(), json!("closed"));
            let closed = update_workflow(&ctx.root, &id, patch)?;
            let text = format!(
                "Closed 1 workflow record: {} (feature \"{}\").",
                js_disp_opt(closed.get("id")),
                js_disp_opt(closed.get("feature"))
            );
            let mut result = Map::new();
            result.insert("closed".into(), Value::Array(vec![closed_row(&closed)]));
            return Ok(Out::Emit(Value::Object(result), text, 0));
        }

        if has_feature {
            let feature = flag_string(&flags, "feature").unwrap_or_default();
            let Ok(_) = &active else {
                let reason = active.unwrap_err();
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: refused \u{2014} the currently active feature could not be resolved, so the guard that keeps \"{feature}\" from being closed while it IS the active feature cannot be evaluated. Nothing was closed.\n{}",
                    workflows_close_unresolved_active_tail(&reason)
                )));
            };
            if active_feature.as_deref() == Some(feature.as_str()) {
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: refused \u{2014} \"{feature}\" is the currently active feature; use --id <id> to close its record explicitly."
                )));
            }
            let records = list_workflows(&ctx.root)?;
            let matches: Vec<String> = records
                .iter()
                .filter(|r| {
                    js_strict_eq(
                        r.get("feature").unwrap_or(&Value::Null),
                        &Value::String(feature.clone()),
                    ) && !js_strict_eq(
                        r.get("status").unwrap_or(&Value::Null),
                        &json!("closed"),
                    )
                })
                .map(wf_id)
                .collect();
            if matches.is_empty() {
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: no live workflow record found for feature \"{feature}\"."
                )));
            }
            let mut closed: Vec<Map<String, Value>> = Vec::new();
            for id in matches {
                let mut patch = Map::new();
                patch.insert("status".into(), json!("closed"));
                closed.push(update_workflow(&ctx.root, &id, patch)?);
            }
            let ids: Vec<String> = closed.iter().map(|r| js_disp_opt(r.get("id"))).collect();
            let text = format!(
                "Closed {} workflow record(s) for feature \"{feature}\": {}.",
                closed.len(),
                ids.join(", ")
            );
            let mut result = Map::new();
            result.insert(
                "closed".into(),
                Value::Array(closed.iter().map(closed_row).collect()),
            );
            return Ok(Out::Emit(Value::Object(result), text, 0));
        }

        // --all-but-active
        let Ok(_) = &active else {
            let reason = active.unwrap_err();
            return Ok(Out::Thrown(format!(
                "workflows close --all-but-active: refused \u{2014} the currently active feature could not be resolved, so \"all but active\" would silently degrade into \"all\": every live workflow record, in-flight work included, would be closed. Nothing was closed.\n{}",
                workflows_close_unresolved_active_tail(&reason)
            )));
        };
        // state.mjs closeWorkflowsForFeature({keepFeature}).
        let keep = active_feature
            .as_deref()
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let records = list_workflows(&ctx.root)?;
        let mut closed: Vec<Value> = Vec::new();
        for record in &records {
            if js_strict_eq(record.get("status").unwrap_or(&Value::Null), &json!("closed")) {
                continue;
            }
            if let Some(keep) = &keep {
                if js_strict_eq(
                    record.get("feature").unwrap_or(&Value::Null),
                    &Value::String(keep.clone()),
                ) {
                    continue;
                }
            }
            let id = wf_id(record);
            let mut patch = Map::new();
            patch.insert("status".into(), json!("closed"));
            update_workflow(&ctx.root, &id, patch)?;
            closed.push(closed_row(record));
        }
        if closed.is_empty() {
            return Ok(Out::Thrown(
                "workflows close --all-but-active: nothing to close \u{2014} no live workflow record other than the active feature.".to_string(),
            ));
        }
        let ids: Vec<String> = closed
            .iter()
            .map(|r| js_disp_opt(jget(r, "id")))
            .collect();
        let text = format!(
            "Closed {} workflow record(s), kept active feature \"{}\": {}.",
            closed.len(),
            active_feature.as_deref().unwrap_or("(none)"),
            ids.join(", ")
        );
        let mut result = Map::new();
        result.insert("closed".into(), Value::Array(closed));
        Ok(Out::Emit(Value::Object(result), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state rebuild-projections (R6 coverage debt) ──────────────────────────

/// bee.mjs withStoreLocks as RAII: the array IS the acquisition order, and the
/// unwind releases innermost-first exactly like the nested `withStoreLock`
/// closures it replaces (Vec's own Drop would release outermost-first).
struct LockStack(Vec<LockGuard>);

impl Drop for LockStack {
    fn drop(&mut self) {
        while self.0.pop().is_some() {}
    }
}

/// state-projection.mjs rebuildAllProjections(root). Returns the `{state,
/// handoff, reservations, lanes}` literal in its own key order.
fn rebuild_all_projections(root: &Path) -> R2<Value> {
    let state = rebuild_state_projection_reporting(root)?;
    let handoff = rebuild_handoff_projection_reporting(root)?;
    let count = rebuild_reservations_projection(root)?;

    let mut state_out = Map::new();
    state_out.insert("authoritative".into(), json!(state.authoritative));
    state_out.insert("source".into(), state.source);
    state_out.insert("state".into(), Value::Object(state.record));

    let mut handoff_out = Map::new();
    handoff_out.insert("authoritative".into(), json!(handoff.authoritative));
    handoff_out.insert("source".into(), handoff.source);

    // rebuildReservationsProjection is never gated on workflow records — see
    // its own doc comment; `authoritative` is an unconditional literal `true`.
    let mut reservations_out = Map::new();
    reservations_out.insert("authoritative".into(), json!(true));
    reservations_out.insert("count".into(), json!(count));

    let workflows = list_workflows(root)?;
    let mut lanes = Vec::new();
    for wf in workflows.iter().filter(is_active_workflow) {
        let feature = js_disp_opt(wf.get("feature"));
        let lane = rebuild_lane_projection_reporting(root, &feature)?;
        let mut row = Map::new();
        row.insert("authoritative".into(), json!(lane.authoritative));
        row.insert("source".into(), lane.source);
        row.insert(
            "lane".into(),
            lane.record.map(Value::Object).unwrap_or(Value::Null),
        );
        lanes.push(Value::Object(row));
    }

    let mut result = Map::new();
    result.insert("state".into(), Value::Object(state_out));
    result.insert("handoff".into(), Value::Object(handoff_out));
    result.insert("reservations".into(), Value::Object(reservations_out));
    result.insert("lanes".into(), Value::Array(lanes));
    Ok(Value::Object(result))
}

/// `wf.status === 'active'` — the filter both the lane-lock peek and
/// rebuildAllProjections's own lane pass use.
fn is_active_workflow(wf: &&Map<String, Value>) -> bool {
    js_strict_eq(wf.get("status").unwrap_or(&Value::Null), &json!("active"))
}

/// bee.mjs handleStateRebuildProjections. The ONE seam that holds more than one
/// projection lock: 'state' then every active workflow's `lane:<feature>`, lane
/// names SORTED and de-duplicated so two concurrent rebuilds acquire in the same
/// sequence and can never deadlock. The lane set is peeked immediately before
/// the acquire, exactly as the .mjs does — rebuildAllProjections re-lists inside.
fn run_rebuild_projections(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state rebuild-projections", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let workflows = list_workflows(&ctx.root)?;
        // DELEGATION GUARD (pre-lock, pre-write): Node's lane pass maps over
        // every ACTIVE record — including one whose `feature` is absent or not
        // a plain non-empty string, where rebuildLaneProjection(root, undefined)
        // reaches requireLaneFeature/String(feature) with bytes this port does
        // not model. No bee-created record has that shape; delegate rather than
        // guess.
        for wf in workflows.iter().filter(is_active_workflow) {
            match wf.get("feature") {
                Some(Value::String(s)) if !s.is_empty() => {}
                _ => return Err(Err2::Ex),
            }
        }
        let mut lane_locks: Vec<String> = workflows
            .iter()
            .filter(is_active_workflow)
            .filter(|wf| wf.get("feature").is_some_and(truthy))
            .map(|wf| lane_lock_name(&js_disp_opt(wf.get("feature"))))
            .collect();
        js_sort(&mut lane_locks); // `.sort()` then `new Set(...)` — sorted, so
        lane_locks.dedup(); // duplicates are adjacent and Set order is preserved
        let mut names: Vec<String> = vec!["state".to_string()];
        names.extend(lane_locks);

        let mut stack = LockStack(Vec::new());
        for name in &names {
            stack.0.push(acquire_named_lock(&ctx.root, name)?);
        }
        let result = rebuild_all_projections(&ctx.root)?;
        drop(stack);

        let lane_count = match jget(&result, "lanes") {
            Some(Value::Array(rows)) => rows
                .iter()
                .filter(|r| jget(r, "authoritative").is_some_and(truthy))
                .count(),
            _ => 0,
        };
        let state_authoritative = jget(&result, "state")
            .and_then(|s| jget(s, "authoritative"))
            .is_some_and(truthy);
        let state_note = if state_authoritative {
            format!(
                "rebuilt .bee/state.json from workflow {}",
                js_disp_opt(jget(&result, "state").and_then(|s| jget(s, "source")))
            )
        } else {
            "state.json left untouched (no workflow records yet, or a live non-idle default feature \u{2014} see D1 field scoping)".to_string()
        };
        let reservations_note = format!(
            "reservations.json rebuilt ({} active)",
            js_disp_opt(jget(&result, "reservations").and_then(|r| jget(r, "count")))
        );
        let text =
            format!("{state_note}; {lane_count} lane projection(s) rebuilt; {reservations_note}.");
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::workflow_store::{
        lanes_dir, list_handoff_mailbox, workflow_state_path, workflows_dir,
    };

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Err2/Exotic carry no Debug impl (reservations.rs owns them) — a local
    /// expect keeps `Result` unwrapping panics readable without editing it.
    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn obj(s: &str) -> Map<String, Value> {
        match serde_json::from_str::<Value>(s).unwrap() {
            Value::Object(m) => m,
            _ => panic!("expected object"),
        }
    }

    // ── gate-door rules (checkPhaseTransition) ────────────────────────────

    #[test]
    fn compounding_is_never_settable_directly() {
        let t = check_phase_transition(Some(&json!("swarming")), "compounding", &Map::new(), false)
            .ok()
            .unwrap();
        assert!(!t.ok);
        assert!(t.reason.starts_with(
            "set: phase \"compounding\" is not settable directly — it is produced only by RECORDING"
        ));
    }

    #[test]
    fn compounding_complete_requires_compounding_phase() {
        let t = check_phase_transition(
            Some(&json!("swarming")),
            "compounding-complete",
            &Map::new(),
            false,
        )
        .ok()
        .unwrap();
        assert!(!t.ok);
        assert!(t.reason.contains(
            "may only be entered from \"compounding\" (current: \"swarming\")"
        ));
        // Falsy phase reads as idle.
        let t2 = check_phase_transition(None, "compounding-complete", &Map::new(), false)
            .ok()
            .unwrap();
        assert!(t2.reason.contains("(current: \"idle\")"));
    }

    #[test]
    fn compounding_complete_requires_fresh_recorded_run() {
        let rec = obj(
            r#"{"feature":"f1","last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec, false)
            .ok()
            .unwrap();
        assert!(!t.ok);
        assert!(t.reason.contains("no fresh compounding run recorded for feature \"f1\""));
        // Waived: passes with waived_compounding flagged.
        let t2 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec, true)
            .ok()
            .unwrap();
        assert!(t2.ok && t2.waived_compounding);
        // Stale run (before the scribing stamp) is not fresh.
        let rec3 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-02T00:00:00.000Z"},"last_compounding_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t3 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec3, false)
            .ok()
            .unwrap();
        assert!(!t3.ok);
        // Fresh same-feature run at-or-after the scribing stamp passes clean.
        let rec4 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"},"last_compounding_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t4 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec4, false)
            .ok()
            .unwrap();
        assert!(t4.ok && !t4.waived_compounding);
        // A mismatched feature is never fresh.
        let rec5 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"},"last_compounding_run":{"feature":"other","at":"2026-07-03T00:00:00.000Z"}}"#,
        );
        let t5 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec5, false)
            .ok()
            .unwrap();
        assert!(!t5.ok);
    }

    #[test]
    fn backward_moves_and_idle_stay_permissive() {
        for to in ["idle", "exploring", "planning", "swarming", "grooming"] {
            let t = check_phase_transition(Some(&json!("reviewing")), to, &Map::new(), false)
                .ok()
                .unwrap();
            assert!(t.ok, "transition to {to} must be permissive");
        }
    }

    #[test]
    fn scribing_and_compounding_run_doors() {
        assert!(check_scribing_run_phase(Some(&json!("swarming"))).is_none());
        let refuse = check_scribing_run_phase(Some(&json!("idle"))).unwrap();
        assert!(refuse.contains("refused from phase \"idle\""));
        assert!(refuse.contains("Legal from: swarming, reviewing, scribing."));
        assert!(check_compounding_run_phase(Some(&json!("compounding"))).is_none());
        let refuse2 = check_compounding_run_phase(Some(&json!("swarming"))).unwrap();
        assert!(refuse2.contains("compounding-run: refused from phase \"swarming\""));
    }

    // ── readStateStrict's typed errors ────────────────────────────────────

    #[test]
    fn read_state_strict_missing_yields_defaults() {
        let tmp = tmp_root();
        let state = read_state_strict(tmp.path()).ok().unwrap();
        assert_eq!(
            jsjson::stringify(&Value::Object(state)),
            r#"{"schema_version":"1.0","phase":"idle","feature":null,"mode":null,"approved_gates":{"context":false,"shape":false,"execution":false,"review":false},"workers":[],"summary":"","next_action":"No active bee work — awaiting a user request."}"#
        );
    }

    #[test]
    fn read_state_strict_unparseable_message_is_exact() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), "{broken");
        let file = tmp.path().join(".bee").join("state.json");
        match read_state_strict(tmp.path()) {
            Err(Err2::Msg(m)) => {
                assert_eq!(
                    m,
                    format!(
                        "readStateStrict: \"{}\" exists but is not valid JSON. The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently clobber real state (gates, workers, feature) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- .bee{}state.json\"), then retry.",
                        file.display(),
                        MAIN_SEPARATOR
                    )
                );
            }
            _ => panic!("expected the unparseable refusal"),
        }
    }

    #[test]
    fn read_state_strict_non_object_names_the_found_type() {
        let cases = [
            ("[1,2]", "an array"),
            ("null", "object"),
            ("42", "number"),
            ("\"x\"", "string"),
            ("true", "boolean"),
        ];
        for (content, found) in cases {
            let tmp = tmp_root();
            write_state_file(tmp.path(), content);
            match read_state_strict(tmp.path()) {
                Err(Err2::Msg(m)) => {
                    assert!(
                        m.contains(&format!("exists but is not a JSON object (found {found})")),
                        "content {content}: {m}"
                    );
                }
                _ => panic!("expected the non-object refusal for {content}"),
            }
        }
    }

    #[test]
    fn read_state_strict_merges_defaults_and_coerces_legacy_phase() {
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"phase":"validating","feature":"f1","approved_gates":{"shape":true},"extra":1}"#,
        );
        let state = read_state_strict(tmp.path()).ok().unwrap();
        assert_eq!(state.get("phase"), Some(&json!("planning")));
        assert_eq!(state.get("feature"), Some(&json!("f1")));
        assert_eq!(
            jsjson::stringify(state.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        // File keys override in the default slot order; extras append.
        let keys: Vec<&String> = state.keys().collect();
        assert_eq!(keys.last().unwrap().as_str(), "extra");
    }

    // ── worker add / prune ────────────────────────────────────────────────

    #[test]
    fn worker_mutate_add_writes_node_shaped_state() {
        let tmp = tmp_root();
        let out = worker_mutate(tmp.path(), |workers| {
            workers.push(json!({"nickname": "w1", "cell": "c1", "tier": Value::Null, "status": Value::Null}));
            Ok("Added worker \"w1\" (cell c1).".to_string())
        });
        match out {
            Ok(Out::Emit(result, text, 0)) => {
                assert_eq!(text, "Added worker \"w1\" (cell c1).");
                let workers = result.get("workers").unwrap();
                assert_eq!(
                    jsjson::stringify(workers),
                    r#"[{"nickname":"w1","cell":"c1","tier":null,"status":null}]"#
                );
            }
            _ => panic!("expected emit"),
        }
        let bytes = std::fs::read_to_string(tmp.path().join(".bee").join("state.json")).unwrap();
        assert!(bytes.ends_with("\n"));
        assert!(bytes.contains("\"nickname\": \"w1\""));
        // The store lock was released.
        assert!(!lock::lock_file_path(tmp.path(), "state").exists());
    }

    #[test]
    fn worker_transient_suffix_and_keep_rules() {
        assert_eq!(worker_transient_suffix_len("c1.prompt.md"), Some(10));
        assert_eq!(worker_transient_suffix_len("c1.result.json"), Some(12));
        assert_eq!(worker_transient_suffix_len("c1.out12.log"), Some(10));
        assert_eq!(worker_transient_suffix_len("c1.out.log"), Some(8));
        assert_eq!(worker_transient_suffix_len("c1.log"), Some(4));
        assert_eq!(worker_transient_suffix_len("c1.txt"), None);
        // Leftmost match: the whole dotted tail is the suffix for a dotted id.
        assert_eq!(worker_transient_suffix_len("a.result.md"), Some(10));
        // Empty stem: the name IS the suffix.
        assert_eq!(worker_transient_suffix_len(".log"), Some(4));
        // Prefix keep-check: "<id>" or "<id>.<anything>", never a mis-stem.
        let keep = vec!["cell.a".to_string()];
        assert!(kept_by_keep_set("cell.a.log", &keep));
        assert!(kept_by_keep_set("cell.a", &keep));
        assert!(!kept_by_keep_set("cell.ab.log", &keep));
    }

    #[test]
    fn prune_keep_set_protects_non_capped_and_corrupt_cells() {
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"workers":[{"nickname":"w1","cell":"c-keep"},null,"junk"]}"#,
        );
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("open.json"), r#"{"status":"open"}"#).unwrap();
        std::fs::write(cells.join("capped.json"), r#"{"status":"capped"}"#).unwrap();
        std::fs::write(cells.join("corrupt.json"), "{nope").unwrap();
        let keep = ok(read_prune_keep_set(tmp.path()));
        assert!(keep.contains(&"c-keep".to_string()));
        assert!(keep.contains(&"open".to_string()));
        assert!(keep.contains(&"corrupt".to_string()));
        assert!(!keep.contains(&"capped".to_string()));
    }

    #[test]
    fn prune_refuses_malformed_workers() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"workers":"not-an-array"}"#);
        match read_prune_keep_set(tmp.path()) {
            Err(Err2::Msg(m)) => assert!(m.starts_with("worker prune: state.workers is not an array")),
            _ => panic!("expected the fails-closed refusal"),
        }
    }

    // ── handoff kinds ─────────────────────────────────────────────────────

    #[test]
    fn pause_handoff_round_trips_with_kind_and_written_at() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("feature".into(), json!("f1"));
        input.insert("cell".into(), json!("wip-1"));
        let record = ok(write_handoff(tmp.path(), &input, "pause"));
        let keys: Vec<&String> = record.keys().collect();
        assert_eq!(keys, ["feature", "cell", "kind", "written_at"]);
        assert_eq!(record.get("kind"), Some(&json!("pause")));
        // readHandoff normalizes kind (already pause) and reads it back.
        let read = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&read, "cell"), Some(&json!("wip-1")));
        // A kindless record on disk reads as pause (the fail-safe).
        std::fs::write(
            handoff_path(tmp.path()),
            "{\n  \"cell\": \"x\"\n}\n",
        )
        .unwrap();
        let read2 = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&read2, "kind"), Some(&json!("pause")));
    }

    #[test]
    fn planned_next_refuses_uncapped_previous_and_unowned_claim() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("planned-next"));
        input.insert("writer_session".into(), json!("sess-w"));
        input.insert("previous_cell".into(), json!("prev"));
        input.insert("next_cell".into(), json!("next"));
        // No previous cell record at all → "missing".
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => assert!(m.contains(
                "previous cell \"prev\" is not capped (found status \"missing\")"
            )),
            _ => panic!("expected the uncapped refusal"),
        }
        // Capped previous but no claim on next → "no claim".
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("next cell \"next\" has no claim owned by writer session \"sess-w\" (found no claim)"))
            }
            _ => panic!("expected the unowned-claim refusal"),
        }
        // Claim owned by someone else → owner "..." in the refusal.
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"other"}"#).unwrap();
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => assert!(m.contains("(found owner \"other\")")),
            _ => panic!("expected the owner refusal"),
        }
        // Correctly owned claim → the record writes with the stamp fields.
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"sess-w"}"#).unwrap();
        let record = ok(write_handoff(tmp.path(), &input, "planned-next"));
        assert_eq!(record.get("kind"), Some(&json!("planned-next")));
        assert!(record.contains_key("written_at"));
    }

    #[test]
    fn adopt_refuses_pause_and_adopts_planned_next_with_fence_bump() {
        let tmp = tmp_root();
        // Pause handoff is never adopted.
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            handoff_path(tmp.path()),
            r#"{"kind":"pause","cell":"x"}"#,
        )
        .unwrap();
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Fail { reason } => assert_eq!(
                reason,
                "handoff kind \"pause\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1)."
            ),
            _ => panic!("expected the pause refusal"),
        }
        // planned-next with an owned claim adopts: fence bumps, handoff clears.
        std::fs::write(
            handoff_path(tmp.path()),
            r#"{"kind":"planned-next","writer_session":"sess-w","previous_cell":"prev","next_cell":"next"}"#,
        )
        .unwrap();
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join("next.json"),
            r#"{"cell":"next","session":"sess-w","fence_epoch":1}"#,
        )
        .unwrap();
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Ok { claim, previous_owner, next_cell } => {
                assert_eq!(next_cell, "next");
                assert_eq!(previous_owner, Some(json!("sess-w")));
                assert_eq!(claim.get("session"), Some(&json!("sess-new")));
                assert_eq!(claim.get("adopted_from"), Some(&json!("sess-w")));
                assert_eq!(claim.get("fence_epoch"), Some(&json!(2.0)));
            }
            HandoffAdopt::Fail { reason } => panic!("unexpected refusal: {reason}"),
        }
        assert!(!handoff_path(tmp.path()).exists(), "handoff cleared after adopt");
        // The gate file was released.
        assert!(!claims.join("next.adopting").exists());
        // No handoff left → NO_HANDOFF.
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Fail { reason } => assert_eq!(reason, "no .bee/HANDOFF.json to adopt."),
            _ => panic!("expected NO_HANDOFF"),
        }
    }

    #[test]
    fn adopt_gate_held_is_a_typed_refusal() {
        let tmp = tmp_root();
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"a"}"#).unwrap();
        std::fs::write(claims.join("next.adopting"), "{}").unwrap();
        match ok(adopt_claim(tmp.path(), "next", "sess-new")) {
            AdoptOutcome::Fail { reason } => assert!(reason.contains("gated by another in-flight adopt/sweep")),
            _ => panic!("expected GATE_HELD"),
        }
        // The pre-existing (foreign) gate file must survive our failed attempt.
        assert!(claims.join("next.adopting").exists());
    }

    // ── the lane/workflow seam (the R6 "C1 gate" is gone) ─────────────────

    fn write_workflow(root: &Path, id: &str, body: Value) {
        let dir = workflows_dir(root).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), serde_json::to_string(&body).unwrap()).unwrap();
    }

    fn read_workflow_file(root: &Path, id: &str) -> Value {
        serde_json::from_str(&std::fs::read_to_string(workflow_state_path(root, id)).unwrap())
            .unwrap()
    }

    fn write_lane_file(root: &Path, feature: &str, content: &str) {
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        std::fs::write(lanes_dir(root).join(format!("{feature}.json")), content).unwrap();
    }

    /// The session id the resolver will actually look for. resolveSessionId's
    /// env chain (BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID) OUTRANKS single-live
    /// -session adoption, and a Claude Code test runner really does export
    /// CLAUDE_CODE_SESSION_ID — so a fixture that hard-codes "sess-1" would be
    /// invisible to the very code under test. Ask the resolver instead.
    fn fixture_session_id(root: &Path) -> String {
        ok(resolve_session_id_no_flag(root)).unwrap_or_else(|| "sess-1".to_string())
    }

    fn write_session(root: &Path, id: &str, lane: Option<&str>) {
        let dir = sessions_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut rec = Map::new();
        rec.insert("id".into(), json!(id));
        rec.insert("last_heartbeat".into(), json!(now_iso()));
        if let Some(l) = lane {
            rec.insert("lane".into(), json!(l));
        }
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify(&Value::Object(rec)),
        )
        .unwrap();
    }

    #[test]
    fn mutation_scope_follows_lane_then_session_binding_then_default_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"default-f"}"#);
        // Explicit --lane always wins.
        let s = ok(resolve_mutation_lock_scope(tmp.path(), Some("lane-a"), false));
        assert_eq!(s.feature.as_deref(), Some("lane-a"));
        assert!(s.lane);
        assert_eq!(projection_lock_name(s.lane, s.feature.as_deref()), "lane:lane-a");
        // --no-lane forces the default record AND skips session resolution.
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, true));
        assert!(s.feature.is_none() && !s.lane);
        // A bound session targets its lane.
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("lane-b"));
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, false));
        assert_eq!(s.feature.as_deref(), Some("lane-b"));
        assert!(s.lane);
        // Unbound: the default record's own feature, lane = false.
        write_session(tmp.path(), &sid, None);
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, false));
        assert_eq!(s.feature.as_deref(), Some("default-f"));
        assert!(!s.lane);
        assert_eq!(projection_lock_name(s.lane, s.feature.as_deref()), "state");
    }

    #[test]
    fn lane_resolution_refusals_are_byte_exact_and_never_guess_back() {
        let tmp = tmp_root();
        match resolve_mutation_target(tmp.path(), Some("ghost"), "set", false) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "set: refused — lane \"ghost\" does not exist (no .bee/lanes/ghost.json). FIX: start it first (\"state start-feature --feature ghost --as-lane\"), then retry."
            ),
            _ => panic!("expected the LANE_MISSING refusal"),
        }
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("ghost"));
        match resolve_mutation_target(tmp.path(), None, "gate", false) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                format!("gate: refused — calling session \"{sid}\" is bound to lane \"ghost\" but no .bee/lanes/ghost.json exists; resolution never guesses back to the default record. FIX: start the lane (\"state start-feature --feature ghost --as-lane\"), unbind the session, or pass --no-lane to target the default record explicitly.")
            ),
            _ => panic!("expected the bound-lane refusal"),
        }
        // --no-lane is the documented escape back to the default record.
        assert!(matches!(
            ok(resolve_mutation_target(tmp.path(), None, "gate", true)),
            Target::Default { .. }
        ));
        // A present-but-corrupt lane record refuses instead of defaulting.
        write_lane_file(tmp.path(), "ghost", "{nope");
        assert!(matches!(
            resolve_mutation_target(tmp.path(), Some("ghost"), "set", false),
            Err(Err2::Msg(_))
        ));
    }

    #[test]
    fn lane_mutation_writes_through_the_live_workflow_record() {
        let tmp = tmp_root();
        write_lane_file(
            tmp.path(),
            "f1",
            r#"{"feature":"f1","phase":"planning","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning",
                   "plan_rev":2,"created_at":"2026-01-02T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), Some("f1"), "gate", false));
        assert_eq!(target.selected_record(), "lane \"f1\"");
        assert_eq!(target.lane_note(), " (lane \"f1\")");
        {
            let rec = target.record_mut();
            let mut gates = default_gates();
            gates.insert("execution".into(), json!(true));
            rec.insert("approved_gates".into(), Value::Object(gates));
            rec.insert("phase".into(), json!("swarming"));
        }
        let record = target.record().clone();
        let stamps = vec![("execution".to_string(), json!(2))];
        ok(write_through_projection(tmp.path(), &target, &record, &stamps));
        // The WORKFLOW record took the D1 fields and the plan-rev stamp…
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["phase"], json!("swarming"));
        assert_eq!(wf["feature"], json!("f1"), "identity never patched");
        assert_eq!(
            wf["gates"]["execution"],
            json!({"approved":true,"approved_for_plan_rev":2})
        );
        assert_eq!(
            wf["gates"]["context"],
            json!({"approved":false,"approved_for_plan_rev":null})
        );
        // …and the lane projection was rebuilt FROM it ("record wins").
        let lane = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":false,"execution":true,"review":false}"#
        );
        assert_eq!(lane.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        // A lane mutation never touches .bee/state.json.
        assert!(!state_path(tmp.path()).exists());
    }

    #[test]
    fn a_plan_rev_bump_flips_the_stamped_gate_in_the_same_projection() {
        let tmp = tmp_root();
        write_lane_file(tmp.path(), "f1", r#"{"feature":"f1","phase":"planning"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning","plan_rev":1,
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":1},
                            "context":{"approved":true,"approved_for_plan_rev":null}}}),
        );
        // Before the bump, execution projects effective.
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":true,"shape":false,"execution":true,"review":false}"#
        );
        let updated = ok(update_workflow_assuming_lock_with(tmp.path(), "wf-1", |current| {
            let base = current.get("plan_rev").and_then(Value::as_f64).unwrap_or(0.0);
            let mut patch = Map::new();
            patch.insert(
                "plan_rev".into(),
                Value::Number(serde_json::Number::from_f64(base + 1.0).unwrap()),
            );
            Ok(patch)
        }));
        assert_eq!(jsjson::stringify(updated.get("plan_rev").unwrap()), "2");
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        // execution was stamped for rev 1 → ineffective at rev 2; context is
        // rev-immune (never stamped) and survives.
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":true,"shape":false,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn default_mutation_routes_through_its_own_live_workflow() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1","workers":[]}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning",
                   "plan_rev":0,"summary":"","next_action":"",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), None, "set", true));
        assert_eq!(target.selected_record(), "default state");
        assert_eq!(target.lane_note(), "");
        target.record_mut().insert("phase".into(), json!("swarming"));
        target.record_mut().insert("summary".into(), json!("S"));
        let record = target.record().clone();
        ok(write_through_projection(tmp.path(), &target, &record, &[]));
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["phase"], json!("swarming"));
        assert_eq!(wf["summary"], json!("S"));
        // state.json is the rebuilt projection of that same record.
        let st = ok(read_state_strict(tmp.path()));
        assert_eq!(st.get("phase"), Some(&json!("swarming")));
        assert_eq!(st.get("summary"), Some(&json!("S")));
    }

    #[test]
    fn a_feature_swap_bypasses_workflow_routing_and_writes_state_directly() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"old"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"old","status":"active","phase":"planning",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), None, "set", true));
        target.record_mut().insert("feature".into(), json!("new"));
        let record = target.record().clone();
        ok(write_through_projection(tmp.path(), &target, &record, &[]));
        // state.json took the swap…
        assert_eq!(ok(read_state_strict(tmp.path())).get("feature"), Some(&json!("new")));
        // …and the OLD feature's workflow record is completely untouched.
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["feature"], json!("old"));
        assert_eq!(wf["phase"], json!("planning"));
    }

    #[test]
    fn mutation_locks_follow_the_global_order_and_the_projection_scope() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let workflows = ok(list_workflows(tmp.path()));
        let scope = Scope { feature: Some("f1".to_string()), lane: true };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        assert!(lock::lock_file_path(tmp.path(), "lane:f1").exists());
        // A lane mutation must NOT serialize against .bee/state.json's writers.
        assert!(!lock::lock_file_path(tmp.path(), "state").exists());
        drop(locks);
        assert!(!lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        // A default-record mutation with a live workflow: workflow:<id> + 'state'.
        let scope = Scope { feature: Some("f1".to_string()), lane: false };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        assert!(lock::lock_file_path(tmp.path(), "state").exists());
        drop(locks);
        // C1 fallback (no live workflow): the single 'state' hold, lane or not.
        let scope = Scope { feature: Some("nolane".to_string()), lane: true };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "state").exists());
        assert!(!lock::lock_file_path(tmp.path(), "lane:nolane").exists());
        drop(locks);
    }

    #[test]
    fn handoff_workflow_resolution_covers_c1_lane_session_and_default() {
        let tmp = tmp_root();
        // C1: zero workflow records → the legacy single-file path.
        assert!(ok(resolve_handoff_workflow_id(tmp.path(), None, None)).is_none());
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        assert_eq!(
            ok(resolve_handoff_workflow_id(tmp.path(), Some("f1"), None)).as_deref(),
            Some("wf-1")
        );
        match resolve_handoff_workflow_id(tmp.path(), Some("ghost"), None) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "state handoff: refused — --lane \"ghost\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"ghost\" and status !== closed). FIX: start it first (\"state start-feature --feature ghost --as-lane\"), or omit --lane."
            ),
            _ => panic!("expected the --lane refusal"),
        }
        // The default record's own feature resolves last.
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        assert_eq!(
            ok(resolve_handoff_workflow_id(tmp.path(), None, None)).as_deref(),
            Some("wf-1")
        );
        // A bound session naming no live workflow refuses loudly (the
        // --session-id FLAG outranks the env chain, so "sess-1" is safe here).
        write_session(tmp.path(), "sess-1", Some("ghost"));
        match resolve_handoff_workflow_id(tmp.path(), None, Some("sess-1")) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "state handoff: refused — calling session \"sess-1\" is bound to lane \"ghost\" but no live workflow names it. FIX: start the lane, unbind the session, or pass --lane explicitly."
            ),
            _ => panic!("expected the bound-session refusal"),
        }
        // A CLOSED workflow is not live.
        write_workflow(
            tmp.path(),
            "wf-2",
            json!({"id":"wf-2","feature":"ghost","status":"closed",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        assert!(resolve_handoff_workflow_id(tmp.path(), Some("ghost"), None).is_err());
    }

    #[test]
    fn a_workflow_carrying_repo_routes_handoffs_to_the_mailbox() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        let wid = ok(resolve_handoff_workflow_id(tmp.path(), None, None)).unwrap();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("feature".into(), json!("f1"));
        let record = ok(write_mailbox_handoff(tmp.path(), &wid, &input, None));
        assert_eq!(record.get("workflow_id"), Some(&json!("wf-1")));
        assert_eq!(record.get("seq"), Some(&json!(1)));
        // The mailbox is the source of truth…
        assert_eq!(ok(list_handoff_mailbox(tmp.path(), "wf-1")).len(), 1);
        // …and the legacy file is its projection.
        ok(rebuild_handoff_projection(tmp.path()));
        let legacy = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&legacy, "kind"), Some(&json!("pause")));
        assert!(jget(&legacy, "workflow_id").is_none(), "mailbox-only field stripped");
    }

    #[test]
    fn workflows_close_guards_the_active_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"active-f"}"#);
        write_workflow(
            tmp.path(),
            "wf-active",
            json!({"id":"wf-active","feature":"active-f","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        write_workflow(
            tmp.path(),
            "wf-stale",
            json!({"id":"wf-stale","feature":"stale-f","status":"active",
                   "created_at":"2026-01-02T00:00:00.000Z"}),
        );
        let active = ok(resolve_active_feature_for_workflows_close(tmp.path()));
        assert_eq!(active.unwrap().as_deref(), Some("active-f"));
        // A resolution FAILURE is distinguishable from "idle" (F5).
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("ghost"));
        let failed = ok(resolve_active_feature_for_workflows_close(tmp.path()));
        assert!(failed.is_err(), "a bound-but-missing lane is a failure, not null");
        assert!(workflows_close_unresolved_active_tail("R").starts_with("Underlying resolution failure: R\n"));
    }

    // ── lanes ─────────────────────────────────────────────────────────────

    #[test]
    fn lane_records_merge_defaults_and_reject_mismatched_features() {
        let tmp = tmp_root();
        let lanes = lanes_dir(tmp.path());
        std::fs::create_dir_all(&lanes).unwrap();
        std::fs::write(
            lanes.join("f1.json"),
            r#"{"feature":"f1","phase":"swarming","approved_gates":{"shape":true}}"#,
        )
        .unwrap();
        std::fs::write(lanes.join("f2.json"), r#"{"feature":"OTHER"}"#).unwrap();
        let rows = list_lanes(tmp.path()).ok().unwrap();
        assert_eq!(rows.len(), 1, "mismatched-feature record is skipped (warned)");
        let lane = &rows[0];
        assert_eq!(lane.get("feature"), Some(&json!("f1")));
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        assert_eq!(lane.get("created_at"), Some(&Value::Null));
    }
}
