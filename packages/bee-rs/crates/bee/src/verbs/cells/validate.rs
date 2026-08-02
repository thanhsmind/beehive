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

/// lib/cells.mjs validateNewCell — throws (Fail::Thrown) the FIRST problem.
pub(crate) fn validate_new_cell(root: &Path, cell: &Value) -> MR<()> {
    let map = match cell {
        Value::Object(m) => m,
        _ => return Err(Fail::Thrown("addCell: cell must be a JSON object.".into())),
    };
    for field in ["id", "feature", "title", "action", "verify"] {
        if !nonblank_string(map.get(field)) {
            return Err(Fail::Thrown(format!(
                "addCell: cell is missing required field \"{field}\" (non-empty string)."
            )));
        }
    }
    assert_verify_sentinel_allowed(root, "addCell", map.get("verify").unwrap_or(&Value::Null))?;
    let id = match map.get("id") {
        Some(Value::String(s)) => s.clone(),
        _ => unreachable!("checked above"),
    };
    if !id_pattern_ok(&id) {
        return Err(Fail::Thrown(format!(
            "addCell: invalid id \"{id}\" — use letters, digits, dot, dash, underscore (e.g. \"auth-3\")."
        )));
    }
    let lane_ok = matches!(map.get("lane"), Some(Value::String(s)) if LANES.contains(&s.as_str()));
    if !lane_ok {
        return Err(Fail::Thrown(format!(
            "addCell: invalid lane \"{}\" — must be one of: {}.",
            js_string_or_undefined(map.get("lane")),
            LANES.join(", ")
        )));
    }
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
            return Err(Fail::Thrown(format!(
                "addCell: lane \"{lane}\" requires non-empty must_haves.truths (observable truths to verify)."
            )));
        }
    }
    if let Some(pbi) = map.get("pbi") {
        if !matches!(pbi, Value::Null | Value::String(_)) {
            return Err(Fail::Thrown(
                "addCell: optional \"pbi\" must be a string backlog id when present.".into(),
            ));
        }
    }
    if let Some(tier) = map.get("tier") {
        let ok = matches!(tier, Value::Null)
            || matches!(tier, Value::String(s) if MODEL_TIERS.contains(&s.as_str()));
        if !ok {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"tier\" must be one of {} when present.",
                MODEL_TIERS.join(", ")
            )));
        }
    }
    if let Some(class) = map.get("change_class") {
        let ok = matches!(class, Value::Null)
            || matches!(class, Value::String(s) if CHANGE_CLASSES.contains(&s.as_str()));
        if !ok {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"change_class\" must be one of {} when present.",
                CHANGE_CLASSES.join(", ")
            )));
        }
    }
    if let Some(budgets) = map.get("budgets") {
        if !matches!(budgets, Value::Null) {
            let Value::Object(budget_map) = budgets else {
                return Err(Fail::Thrown(
                    "addCell: optional \"budgets\" must be a plain object when present.".into(),
                ));
            };
            for key in budget_map.keys() {
                if !BUDGET_KEYS.contains(&key.as_str()) {
                    return Err(Fail::Thrown(format!(
                        "addCell: unknown \"budgets\" key \"{key}\" — must be one of: {}.",
                        BUDGET_KEYS.join(", ")
                    )));
                }
            }
            for (idx, key) in BUDGET_KEYS.iter().enumerate() {
                let Some(value) = budget_map.get(*key) else { continue };
                let hard_max = BUDGET_HARD_MAX[idx];
                let ok = js_is_integer(value).map(|f| f >= 1.0 && f <= hard_max).unwrap_or(false);
                if !ok {
                    return Err(Fail::Thrown(format!(
                        "addCell: \"budgets.{key}\" must be an integer in [1, {}] when present, got {}.",
                        jsjson::js_f64_to_string(hard_max),
                        jsjson::stringify(value)
                    )));
                }
            }
        }
    }
    if let Some(ack) = map.get(REGEN_ACK_FIELD) {
        if !matches!(ack, Value::Null) && !nonblank_string(Some(ack)) {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"{REGEN_ACK_FIELD}\" must be a non-empty string (the one-line reason the derived regen obligation is being skipped)."
            )));
        }
    }
    if read_cell_norm(root, &id)?.map(|v| js_truthy(&v)).unwrap_or(false) {
        return Err(Fail::Thrown(format!("addCell: cell \"{id}\" already exists.")));
    }
    assert_regen_obligation(map, "addCell")
}

/// lib/cells.mjs normalizeNewCell — key order: existing keys keep position,
/// the literal's fields (status, deps, decisions, files, read_first, trace)
/// append where absent.
pub(crate) fn normalize_new_cell(cell: &Value) -> MR<Value> {
    let Value::Object(map) = cell else { return Err(Fail::Delegate) };
    let mut out = map.clone();
    let status = match map.get("status") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => Value::String("open".into()),
    };
    out.insert("status".into(), status);
    for key in ["deps", "decisions", "files", "read_first"] {
        let value = match map.get(key) {
            Some(Value::Array(a)) => Value::Array(a.clone()),
            _ => Value::Array(vec![]),
        };
        out.insert(key.into(), value);
    }
    out.insert("trace".into(), Value::Object(merge_trace(map.get("trace"))?));
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
            js_default_str_sort(&mut members);
            cycles.push(members);
            continue;
        }
        let idx = component[0];
        if adj[idx].contains(&idx) {
            cycles.push(vec![by_id[idx].0.clone()]);
        }
    }
    cycles.sort_by(|a, b| {
        let au: Vec<u16> = a[0].encode_utf16().collect();
        let bu: Vec<u16> = b[0].encode_utf16().collect();
        au.cmp(&bu)
    });
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
