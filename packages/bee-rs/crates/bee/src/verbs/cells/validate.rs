// cell validators, cycle detection and budgets
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::textutil::{code_unit_cmp, js_default_sort};
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── cell validators (lib/cells.mjs validateNewCell / updateCell) ──────────

pub(crate) const LANES: [&str; 5] = ["tiny", "small", "standard", "high-risk", "spike"];

pub(crate) const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];

pub(crate) const CHANGE_CLASSES: [&str; 8] =
    ["formatting", "bugfix", "behavior", "api", "security", "migration", "refactor", "test"];
const BUDGET_KEYS: [&str; 3] = ["max_claims", "max_failed_attempts", "max_same_signature"];

pub(crate) const BUDGET_DEFAULTS: [f64; 3] = [3.0, 4.0, 2.0];

pub(crate) const BUDGET_HARD_MAX: [f64; 3] = [9.0, 12.0, 6.0];

/// assertVerifySentinelAllowed (no-test-repos D1/D2).
pub(crate) fn assert_verify_sentinel_allowed(root: &Path, verb: &str, verify: &Value) -> MR<()> {
    if !matches!(verify, Value::String(s) if s == NO_TEST_SENTINEL) {
        return Ok(());
    }
    if is_no_test_repo(&read_commands_slice(root)?) {
        return Ok(());
    }
    Err(Fail::Thrown(format!(
        "{verb}: verify \"{NO_TEST_SENTINEL}\" is refused — this repo has not declared itself a no-test repo. FIX: use a real, runnable verify command, or declare the repo no-test first by setting commands.test to \"{NO_TEST_SENTINEL}\" in .bee/config.json (decision 55b951e1)."
    )))
}

pub(crate) fn nonblank_string(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::String(s)) if !js_trim(s).is_empty())
}

pub(crate) fn is_string_array(v: &Value) -> bool {
    matches!(v, Value::Array(items) if items.iter().all(|i| matches!(i, Value::String(_))))
}

/// JS Number.isInteger for a JSON value.
pub(crate) fn js_is_integer(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => {
            let f = n.as_f64()?;
            if f.is_finite() && f.fract() == 0.0 {
                Some(f)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `<name>` -> `skills/<name>/SKILL.md` when that file really exists under
/// `root`. Used to turn a bare skill name into the exact path that belongs
/// in `affects_skills`; None for anything already carrying a `/`, and for a
/// name that names no skill.
pub(crate) fn bare_skill_name_path(root: &Path, name: &str) -> Option<String> {
    if name.is_empty() || name.contains('/') {
        return None;
    }
    if root.join("skills").join(name).join("SKILL.md").is_file() {
        Some(format!("skills/{name}/SKILL.md"))
    } else {
        None
    }
}

/// One problem sentence for an `affects_skills` entry that is not a
/// repo-relative path under `skills/` — None when the entry is well formed.
///
/// `affects_skills` holds PATHS: the cap-time sync door (sync_door.rs check
/// (c)) compares the prediction against the touched `skills/**` paths, so a
/// bare skill name can never match and only ever explodes at cap. The
/// refusal names the entry and, for a bare name that resolves to an
/// existing `skills/<name>/SKILL.md`, the exact replacement path.
pub(crate) fn affects_skills_entry_problem(root: &Path, verb: &str, raw: &str) -> Option<String> {
    let norm = normalize_cell_path(raw);
    if !norm.is_empty() && norm != "skills" && path_under_root(&norm, "skills") {
        return None;
    }
    let fix = match bare_skill_name_path(root, &norm) {
        Some(path) => format!("\"{raw}\" is a bare skill name; use \"{path}\" instead."),
        None => "use the skill file's repo-relative path (e.g. \"skills/<skill-name>/SKILL.md\"), or drop the entry.".to_string(),
    };
    Some(format!(
        "{verb}: \"affects_skills\" entry \"{raw}\" is not a repo-relative path under \"skills/\" — affects_skills holds paths, never skill names (the cap-time sync door compares them against touched skills/** paths). FIX: {fix}"
    ))
}

/// Every bad entry in one `affects_skills` array, in order — whole-batch
/// validation rules are unchanged: nothing stops at the first problem.
pub(crate) fn affects_skills_path_problems(root: &Path, verb: &str, value: &Value) -> Vec<String> {
    let Value::Array(items) = value else { return Vec::new() };
    items
        .iter()
        .filter_map(|item| match item {
            Value::String(raw) => affects_skills_entry_problem(root, verb, raw),
            _ => None,
        })
        .collect()
}

/// lib/cells.mjs validateNewCell, as a collector — gathers EVERY schema
/// problem (in the original check order) into one list instead of throwing
/// on the first. A dependent check is skipped when its prerequisite already
/// failed (id pattern/already-exists need a non-blank id; must_haves.truths
/// needs a valid lane; the verify-sentinel check needs a non-blank verify) —
/// skipping never manufactures a second problem for the same root cause. A
/// real IO/Delegate failure (config read, cell-store read, guard discovery)
/// still propagates as Err immediately; it is never collected as a problem.
pub(crate) fn validate_new_cell_problems(root: &Path, cell: &Value) -> MR<Vec<String>> {
    let map = match cell {
        Value::Object(m) => m,
        _ => return Ok(vec!["addCell: cell must be a JSON object.".into()]),
    };
    let mut problems: Vec<String> = Vec::new();
    for field in ["id", "feature", "title", "action", "verify"] {
        if !nonblank_string(map.get(field)) {
            problems.push(format!(
                "addCell: cell is missing required field \"{field}\" (non-empty string)."
            ));
        }
    }
    for key in ["affects_skills", "affects_specs"] {
        match map.get(key) {
            None => {
                problems.push(format!(
                    "addCell: cell is missing required field \"{key}\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none)."
                ));
            }
            Some(v) if !is_string_array(v) => {
                problems.push(format!(
                    "addCell: \"{key}\" must be an array of strings."
                ));
            }
            // Shape is good: `affects_skills` also owes its FORMAT here
            // (paths, not names) so a bare name never survives to the
            // cap-time sync door. `affects_specs` has no cap-time door and
            // keeps its shape-only check.
            Some(v) if key == "affects_skills" => {
                problems.extend(affects_skills_path_problems(root, "addCell", v));
            }
            _ => {}
        }
    }
    if nonblank_string(map.get("verify")) {
        match assert_verify_sentinel_allowed(root, "addCell", map.get("verify").unwrap()) {
            Ok(()) => {}
            Err(Fail::Thrown(message)) => problems.push(message),
            Err(Fail::Delegate) => return Err(Fail::Delegate),
        }
    }
    let id = if nonblank_string(map.get("id")) {
        match map.get("id") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => unreachable!("nonblank_string only matches Value::String"),
        }
    } else {
        None
    };
    if let Some(id) = &id {
        if !id_pattern_ok(id) {
            problems.push(format!(
                "addCell: invalid id \"{id}\" — use letters, digits, dot, dash, underscore (e.g. \"auth-3\")."
            ));
        }
    }
    let lane_ok = matches!(map.get("lane"), Some(Value::String(s)) if LANES.contains(&s.as_str()));
    if !lane_ok {
        problems.push(format!(
            "addCell: invalid lane \"{}\" — must be one of: {}.",
            js_string_or_undefined(map.get("lane")),
            LANES.join(", ")
        ));
    }
    if lane_ok {
        let lane = match map.get("lane") {
            Some(Value::String(s)) => s.clone(),
            _ => unreachable!(),
        };
        if lane == "standard" || lane == "high-risk" {
            let truths = map
                .get("must_haves")
                .filter(|m| js_truthy(m))
                .and_then(|m| m.get("truths"));
            let ok = matches!(truths, Some(Value::Array(a)) if !a.is_empty());
            if !ok {
                problems.push(format!(
                    "addCell: lane \"{lane}\" requires non-empty must_haves.truths (observable truths to verify)."
                ));
            }
        }
    }
    if let Some(pbi) = map.get("pbi") {
        if !matches!(pbi, Value::Null | Value::String(_)) {
            problems.push("addCell: optional \"pbi\" must be a string backlog id when present.".into());
        }
    }
    if let Some(tier) = map.get("tier") {
        let ok = matches!(tier, Value::Null)
            || matches!(tier, Value::String(s) if MODEL_TIERS.contains(&s.as_str()));
        if !ok {
            problems.push(format!(
                "addCell: optional \"tier\" must be one of {} when present.",
                MODEL_TIERS.join(", ")
            ));
        }
    }
    if let Some(class) = map.get("change_class") {
        let ok = matches!(class, Value::Null)
            || matches!(class, Value::String(s) if CHANGE_CLASSES.contains(&s.as_str()));
        if !ok {
            problems.push(format!(
                "addCell: optional \"change_class\" must be one of {} when present.",
                CHANGE_CLASSES.join(", ")
            ));
        }
    }
    if let Some(budgets) = map.get("budgets") {
        if !matches!(budgets, Value::Null) {
            match budgets {
                Value::Object(budget_map) => {
                    let unknown_key = budget_map.keys().find(|key| !BUDGET_KEYS.contains(&key.as_str()));
                    if let Some(key) = unknown_key {
                        problems.push(format!(
                            "addCell: unknown \"budgets\" key \"{key}\" — must be one of: {}.",
                            BUDGET_KEYS.join(", ")
                        ));
                    } else {
                        for (idx, key) in BUDGET_KEYS.iter().enumerate() {
                            let Some(value) = budget_map.get(*key) else { continue };
                            let hard_max = BUDGET_HARD_MAX[idx];
                            let ok = js_is_integer(value).map(|f| f >= 1.0 && f <= hard_max).unwrap_or(false);
                            if !ok {
                                problems.push(format!(
                                    "addCell: \"budgets.{key}\" must be an integer in [1, {}] when present, got {}.",
                                    jsjson::js_f64_to_string(hard_max),
                                    jsjson::stringify(value)
                                ));
                                break;
                            }
                        }
                    }
                }
                _ => problems.push(
                    "addCell: optional \"budgets\" must be a plain object when present.".into(),
                ),
            }
        }
    }
    if let Some(ack) = map.get(REGEN_ACK_FIELD) {
        if !matches!(ack, Value::Null) && !nonblank_string(Some(ack)) {
            problems.push(format!(
                "addCell: optional \"{REGEN_ACK_FIELD}\" must be a non-empty string (the one-line reason the derived regen obligation is being skipped)."
            ));
        }
    }
    if let Some(id) = &id {
        if read_cell_norm(root, id)?.map(|v| js_truthy(&v)).unwrap_or(false) {
            problems.push(format!("addCell: cell \"{id}\" already exists."));
        }
    }
    match assert_regen_obligation(map, "addCell") {
        Ok(()) => {}
        Err(Fail::Thrown(message)) => problems.push(message),
        Err(Fail::Delegate) => return Err(Fail::Delegate),
    }
    match assert_judge_obligation(map, "addCell") {
        Ok(()) => {}
        Err(Fail::Thrown(message)) => problems.push(message),
        Err(Fail::Delegate) => return Err(Fail::Delegate),
    }
    Ok(problems)
}

/// lib/cells.mjs validateNewCell — throws (Fail::Thrown) every problem the
/// collector found, joined into one message; each collected sentence keeps
/// its own exact wording.
pub(crate) fn validate_new_cell(root: &Path, cell: &Value) -> MR<()> {
    let problems = validate_new_cell_problems(root, cell)?;
    if problems.is_empty() {
        Ok(())
    } else {
        Err(Fail::Thrown(problems.join(" ")))
    }
}

/// The shared behavior-door default (E6 / B-P2-8): a call that sets
/// `change_class` to `"behavior"` arms `trace.behavior_change = true`
/// UNLESS that same call already declared its own `behavior_change` value —
/// an explicit `false` is a deliberate opt-out, honored as-is. Both
/// `addCell` (`normalize_new_cell`, below — the payload's nested
/// `trace.behavior_change`) and `updateCell` (`run_update`, handlers_write.rs
/// — the patch's flat `behavior_change` field) call this ONE door rather
/// than each re-deriving the rule, so the two shapes can never drift.
///
/// Never fires the other direction: a call that changes `change_class` AWAY
/// from `"behavior"` (or never touches it) changes nothing here — the door
/// only ever ARMS, it never disarms an already-armed cell.
pub(crate) fn arms_behavior_door(change_class: Option<&Value>, explicit_behavior_change: bool) -> bool {
    !explicit_behavior_change && matches!(change_class, Some(Value::String(s)) if s == "behavior")
}

/// lib/cells.mjs normalizeNewCell — key order: existing keys keep position,
/// the literal's fields (status, deps, decisions, files, read_first, affects_skills, affects_specs, trace)
/// append where absent.
pub(crate) fn normalize_new_cell(cell: &Value) -> MR<Value> {
    let Value::Object(map) = cell else { return Err(Fail::Delegate) };
    let mut out = map.clone();
    let status = match map.get("status") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => Value::String("open".into()),
    };
    out.insert("status".into(), status);
    for key in ["deps", "decisions", "files", "read_first", "affects_skills", "affects_specs"] {
        let value = match map.get(key) {
            Some(Value::Array(a)) => Value::Array(a.clone()),
            _ => Value::Array(vec![]),
        };
        out.insert(key.into(), value);
    }
    let mut trace = merge_trace(map.get("trace"))?;
    // A cell authored with change_class "behavior" defaults to a behavior
    // change unless the payload explicitly sets trace.behavior_change — an
    // explicit false is a deliberate opt-out and is respected as-is.
    let explicit_behavior_change =
        matches!(map.get("trace"), Some(Value::Object(t)) if t.contains_key("behavior_change"));
    if arms_behavior_door(map.get("change_class"), explicit_behavior_change) {
        trace.insert("behavior_change".into(), Value::Bool(true));
    }
    out.insert("trace".into(), Value::Object(trace));
    Ok(Value::Object(out))
}

// ─── cycle detection (lib/schedule.mjs detectCycles, Tarjan) ───────────────

pub(crate) fn schedule_deps_of(cell: &Value) -> Vec<String> {
    match cell.get("deps") {
        Some(Value::Array(deps)) => deps
            .iter()
            .filter_map(|d| match d {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn schedule_files_of(cell: &Value) -> Vec<String> {
    match cell.get("files") {
        Some(Value::Array(files)) => files
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn ids_by_id(cells: &[Value]) -> Vec<(String, &Value)> {
    let mut by_id: Vec<(String, &Value)> = Vec::new();
    for cell in cells {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = by_id.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell; // Map.set: last value, first position
            } else {
                by_id.push((id.clone(), cell));
            }
        }
    }
    by_id
}

/// detectCycles — iterative Tarjan SCC; output normalized by the same sorts
/// Node applies (members sorted, cycles sorted by first member), so SCC
/// emission order never shows.
pub(crate) fn detect_cycles(cells: &[Value]) -> Vec<Vec<String>> {
    let by_id = ids_by_id(cells);
    let index_of: std::collections::HashMap<&str, usize> =
        by_id.iter().enumerate().map(|(i, (k, _))| (k.as_str(), i)).collect();
    let n = by_id.len();
    let adj: Vec<Vec<usize>> = by_id
        .iter()
        .map(|(_, cell)| {
            schedule_deps_of(cell)
                .into_iter()
                .filter_map(|d| index_of.get(d.as_str()).copied())
                .collect()
        })
        .collect();

    let mut indices = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut sccs: Vec<Vec<usize>> = Vec::new();
    let mut counter = 0usize;

    for start in 0..n {
        if indices[start] != usize::MAX {
            continue;
        }
        // Iterative DFS frame: (node, next-edge-index).
        let mut call: Vec<(usize, usize)> = vec![(start, 0)];
        indices[start] = counter;
        lowlink[start] = counter;
        counter += 1;
        stack.push(start);
        on_stack[start] = true;
        while !call.is_empty() {
            let (v, ei) = *call.last().unwrap();
            if ei < adj[v].len() {
                call.last_mut().unwrap().1 += 1;
                let w = adj[v][ei];
                if indices[w] == usize::MAX {
                    indices[w] = counter;
                    lowlink[w] = counter;
                    counter += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    call.push((w, 0));
                } else if on_stack[w] {
                    lowlink[v] = lowlink[v].min(indices[w]);
                }
            } else {
                if lowlink[v] == indices[v] {
                    let mut component = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        component.push(w);
                        if w == v {
                            break;
                        }
                    }
                    sccs.push(component);
                }
                call.pop();
                if let Some(&(parent, _)) = call.last() {
                    lowlink[parent] = lowlink[parent].min(lowlink[v]);
                }
            }
        }
    }

    let mut cycles: Vec<Vec<String>> = Vec::new();
    for component in sccs {
        if component.len() > 1 {
            let mut members: Vec<String> =
                component.iter().map(|&i| by_id[i].0.clone()).collect();
            js_default_sort(&mut members);
            cycles.push(members);
            continue;
        }
        let idx = component[0];
        if adj[idx].contains(&idx) {
            cycles.push(vec![by_id[idx].0.clone()]);
        }
    }
    cycles.sort_by(|a, b| code_unit_cmp(&a[0], &b[0]));
    cycles
}

/// lib/cells.mjs computeIncomingCycles: on-disk cells overlaid by incoming,
/// filtered to cycles touching an incoming id.
pub(crate) fn compute_incoming_cycles(root: &Path, incoming: &[Value]) -> MR<Vec<Vec<String>>> {
    let disk = list_cells(root, None, None).map_err(|_| Fail::Delegate)?;
    let mut union: Vec<(String, Value)> = Vec::new();
    for cell in &disk {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = union.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell.clone();
            } else {
                union.push((id.clone(), cell.clone()));
            }
        }
    }
    let mut incoming_ids: Vec<String> = Vec::new();
    for cell in incoming {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = union.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell.clone();
            } else {
                union.push((id.clone(), cell.clone()));
            }
            if !incoming_ids.contains(id) {
                incoming_ids.push(id.clone());
            }
        }
    }
    let values: Vec<Value> = union.into_iter().map(|(_, v)| v).collect();
    Ok(detect_cycles(&values)
        .into_iter()
        .filter(|cycle| cycle.iter().any(|id| incoming_ids.contains(id)))
        .collect())
}

pub(crate) fn format_cycle_refusal(verb: &str, cycles: &[Vec<String>]) -> String {
    let named = cycles
        .iter()
        .map(|c| c.join(" -> "))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "{verb}: dependency cycle refused — {named}. Cycles are illegal at every dep-mutating write (D2); file overlap stays legal and is never refused."
    )
}

pub(crate) fn assert_no_cycle(root: &Path, verb: &str, incoming: &[Value]) -> MR<()> {
    let cycles = compute_incoming_cycles(root, incoming)?;
    if cycles.is_empty() {
        Ok(())
    } else {
        Err(Fail::Thrown(format_cycle_refusal(verb, &cycles)))
    }
}

// ─── budgets (lib/cells.mjs D2/D-GHF) ──────────────────────────────────────

pub(crate) struct Budgets {
    pub(crate) max_claims: f64,
    pub(crate) max_failed_attempts: f64,
    pub(crate) max_same_signature: f64,
}

/// resolveCellBudgets: forgiving runtime fallback + hard-max clamp.
pub(crate) fn resolve_cell_budgets(cell: &Map<String, Value>) -> Budgets {
    let declared = match cell.get("budgets") {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    };
    let pick = |idx: usize| -> f64 {
        let value = declared.and_then(|m| m.get(BUDGET_KEYS[idx]));
        match value.and_then(js_is_integer) {
            Some(v) if v >= 1.0 => v.min(BUDGET_HARD_MAX[idx]),
            _ => BUDGET_DEFAULTS[idx],
        }
    };
    Budgets { max_claims: pick(0), max_failed_attempts: pick(1), max_same_signature: pick(2) }
}

pub(crate) const FAILED_ATTEMPT_VERDICTS: [&str; 3] = ["fail", "blocked", "tests-red"];

/// attemptsSinceBudgetReset — lexical ISO comparison, per the .mjs.
pub(crate) fn attempts_since_budget_reset(cell: &Map<String, Value>) -> MR<Vec<Value>> {
    let trace = match cell.get("trace") {
        Some(Value::Object(t)) => Some(t),
        _ => None,
    };
    let attempts: Vec<Value> = match trace.and_then(|t| t.get("attempts")) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    // Null entries would crash Node's later `a.claim_session` access.
    if attempts.iter().any(|a| a.is_null()) {
        return Err(Fail::Delegate);
    }
    let resets = match trace.and_then(|t| t.get("budget_resets")) {
        Some(Value::Array(r)) => r.clone(),
        _ => Vec::new(),
    };
    let marker = resets
        .last()
        .and_then(|r| r.get("reset_at"))
        .and_then(|v| match v {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        });
    let Some(marker) = marker else { return Ok(attempts) };
    Ok(attempts
        .into_iter()
        .filter(|a| matches!(a.get("at"), Some(Value::String(at)) if at.as_str() > marker.as_str()))
        .collect())
}

pub(crate) enum BudgetCheck {
    Ok,
    Refused { code: &'static str, reason: String },
}

/// checkCellBudgets — the structural loop-safety check (never reads bypass).
pub(crate) fn check_cell_budgets(cell: &Map<String, Value>) -> MR<BudgetCheck> {
    let budgets = resolve_cell_budgets(cell);
    let relevant = attempts_since_budget_reset(cell)?;
    let id_disp = js_string_or_undefined(cell.get("id"));

    let coerce = |v: Option<&Value>| -> String {
        match v {
            None | Some(Value::Null) => String::new(), // ?? ''
            Some(other) => jsjson::js_to_string(other),
        }
    };
    let mut pairs: Vec<String> = Vec::new();
    for a in &relevant {
        let acquired = match a.get("acquired_at") {
            None | Some(Value::Null) => a.get("claimed_at"),
            other => other,
        };
        let key = format!("{} {}", coerce(a.get("claim_session")), coerce(acquired));
        if !pairs.contains(&key) {
            pairs.push(key);
        }
    }
    let claims_used = pairs.len() as f64 + 1.0;
    if claims_used > budgets.max_claims {
        return Ok(BudgetCheck::Refused {
            code: "CELL_BUDGET_EXHAUSTED",
            reason: format!(
                "cell \"{id_disp}\" exhausted its \"max_claims\" budget (limit {}, used {}) — the claim door is closed until an audited reset.",
                jsjson::js_f64_to_string(budgets.max_claims),
                jsjson::js_f64_to_string(claims_used)
            ),
        });
    }

    let is_failed =
        |a: &Value| matches!(a.get("verdict"), Some(Value::String(v)) if FAILED_ATTEMPT_VERDICTS.contains(&v.as_str()));
    let failed = relevant.iter().filter(|a| is_failed(a)).count() as f64;
    if failed >= budgets.max_failed_attempts {
        return Ok(BudgetCheck::Refused {
            code: "CELL_BUDGET_EXHAUSTED",
            reason: format!(
                "cell \"{id_disp}\" exhausted its \"max_failed_attempts\" budget (limit {}, used {}) — the claim door is closed until an audited reset.",
                jsjson::js_f64_to_string(budgets.max_failed_attempts),
                jsjson::js_f64_to_string(failed)
            ),
        });
    }

    // Same-signature refusal — insertion-ordered Map, first offender wins.
    let mut signature_counts: Vec<(String, f64)> = Vec::new();
    for a in &relevant {
        if !is_failed(a) {
            continue;
        }
        let Some(Value::String(sig)) = a.get("failure_signature") else { continue };
        if sig.is_empty() {
            continue;
        }
        if let Some(slot) = signature_counts.iter_mut().find(|(s, _)| s == sig) {
            slot.1 += 1.0;
        } else {
            signature_counts.push((sig.clone(), 1.0));
        }
    }
    for (signature, count) in &signature_counts {
        if *count >= budgets.max_same_signature {
            return Ok(BudgetCheck::Refused {
                code: "REPEATED_FAILURE",
                reason: format!(
                    "cell \"{id_disp}\" failed {} time(s) with the identical signature \"{signature}\" — change approach or escalate, this is not a re-run.",
                    jsjson::js_f64_to_string(*count)
                ),
            });
        }
    }
    Ok(BudgetCheck::Ok)
}
