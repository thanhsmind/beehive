// computeSchedule and the state/lane gate reads it consults
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

// ─── schedule.mjs computeSchedule ──────────────────────────────────────────

pub(crate) struct Schedule {
    pub(crate) waves: Vec<Vec<String>>,
    pub(crate) cycles: Vec<Vec<String>>,
    pub(crate) unsatisfiable: Vec<(String, String, &'static str)>, // (cell, dep, reason)
    pub(crate) empty_files: Vec<String>,
}

pub(crate) fn compute_schedule(cells: &[Value]) -> Schedule {
    let by_id = ids_by_id(cells);
    let cycles = detect_cycles(cells);

    let mut empty_files: Vec<String> = cells
        .iter()
        .filter(|c| matches!(c.get("id"), Some(Value::String(_))))
        .filter(|c| schedule_files_of(c).is_empty())
        .map(|c| match c.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => unreachable!(),
        })
        .collect();
    js_default_str_sort(&mut empty_files);

    let status_of = |cell: &Value| -> Option<String> {
        match cell.get("status") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let schedulable: Vec<&Value> = cells
        .iter()
        .filter(|c| matches!(status_of(c).as_deref(), Some("open") | Some("claimed")))
        .collect();

    let lookup = |id: &str| by_id.iter().find(|(k, _)| k == id).map(|(_, v)| *v);
    let classify = |dep: &str| -> &'static str {
        match lookup(dep) {
            None => "missing",
            Some(cell) => match status_of(cell).as_deref() {
                Some("capped") => "satisfied",
                Some("blocked") => "blocked",
                Some("dropped") => "dropped",
                _ => "pending",
            },
        }
    };

    let mut unsatisfiable: Vec<(String, String, &'static str)> = Vec::new();
    let mut excluded: Vec<String> = Vec::new();
    for cell in &schedulable {
        let Some(Value::String(cid)) = cell.get("id") else { continue };
        for dep in schedule_deps_of(cell) {
            let kind = classify(&dep);
            if matches!(kind, "missing" | "blocked" | "dropped") {
                unsatisfiable.push((cid.clone(), dep, kind));
                if !excluded.contains(cid) {
                    excluded.push(cid.clone());
                }
            }
        }
    }
    unsatisfiable.sort_by(|a, b| {
        let cell_cmp = {
            let au: Vec<u16> = a.0.encode_utf16().collect();
            let bu: Vec<u16> = b.0.encode_utf16().collect();
            au.cmp(&bu)
        };
        if cell_cmp != Ordering::Equal {
            return cell_cmp;
        }
        let au: Vec<u16> = a.1.encode_utf16().collect();
        let bu: Vec<u16> = b.1.encode_utf16().collect();
        au.cmp(&bu)
    });

    // Propagate exclusion.
    let mut changed = true;
    while changed {
        changed = false;
        for cell in &schedulable {
            let Some(Value::String(cid)) = cell.get("id") else { continue };
            if excluded.contains(cid) {
                continue;
            }
            for dep in schedule_deps_of(cell) {
                if excluded.contains(&dep) {
                    excluded.push(cid.clone());
                    changed = true;
                    break;
                }
            }
        }
    }

    let nodes: Vec<&Value> = schedulable
        .iter()
        .filter(|c| match c.get("id") {
            Some(Value::String(id)) => !excluded.contains(id),
            _ => true, // an id-less cell never entered `excluded`
        })
        .copied()
        .collect();
    let node_ids: Vec<String> = nodes
        .iter()
        .filter_map(|c| match c.get("id") {
            Some(Value::String(id)) => Some(id.clone()),
            _ => None,
        })
        .collect();

    let mut in_degree: Vec<(String, usize)> = node_ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut dependents: Vec<(String, Vec<String>)> =
        node_ids.iter().map(|id| (id.clone(), Vec::new())).collect();
    for cell in &nodes {
        let Some(Value::String(cid)) = cell.get("id") else { continue };
        for dep in schedule_deps_of(cell) {
            if !node_ids.contains(&dep) {
                continue;
            }
            if let Some(slot) = in_degree.iter_mut().find(|(k, _)| k == cid) {
                slot.1 += 1;
            }
            if let Some(slot) = dependents.iter_mut().find(|(k, _)| k == &dep) {
                slot.1.push(cid.clone());
            }
        }
    }

    let mut remaining = in_degree;
    let mut placed: Vec<String> = Vec::new();
    let mut waves: Vec<Vec<String>> = Vec::new();
    loop {
        let mut ready: Vec<String> = nodes
            .iter()
            .filter_map(|c| match c.get("id") {
                Some(Value::String(id)) => Some(id.clone()),
                _ => None,
            })
            .filter(|id| {
                !placed.contains(id)
                    && remaining.iter().find(|(k, _)| k == id).map(|(_, d)| *d == 0).unwrap_or(false)
            })
            .collect();
        js_default_str_sort(&mut ready);
        if ready.is_empty() {
            break;
        }
        let mut wave: Vec<String> = Vec::new();
        for id in &ready {
            let cell_files = lookup(id).map(schedule_files_of).unwrap_or_default();
            let overlaps = wave.iter().any(|placed_id| {
                let placed_files = lookup(placed_id).map(schedule_files_of).unwrap_or_default();
                placed_files
                    .iter()
                    .any(|a| cell_files.iter().any(|b| rsv::paths_overlap(a, b)))
            });
            if !overlaps {
                wave.push(id.clone());
            }
        }
        for id in &wave {
            placed.push(id.clone());
        }
        for id in &wave {
            let deps_list = dependents
                .iter()
                .find(|(k, _)| k == id)
                .map(|(_, d)| d.clone())
                .unwrap_or_default();
            for dependent in deps_list {
                if let Some(slot) = remaining.iter_mut().find(|(k, _)| *k == dependent) {
                    slot.1 = slot.1.saturating_sub(1);
                }
            }
        }
        waves.push(wave);
    }

    Schedule { waves, cycles, unsatisfiable, empty_files }
}

// ─── state/lane gate reads (state.mjs readState / readLane*) ───────────────

/// gateApproved(readState(root), gate) over the brief state slice.
pub(crate) fn default_gate_approved(root: &Path, gate: &str) -> MR<bool> {
    let state = bstate::read_state_brief(root).map_err(|_| Fail::Delegate)?;
    Ok(matches!(state.gates.get(gate), Some(Value::Bool(true))))
}

pub(crate) fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — Ok(trimmed) | Err(()) for the throw path.
pub(crate) fn lane_feature_ok(feature: &str) -> Option<String> {
    let trimmed = js_trim(feature);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return None;
    }
    Some(trimmed.to_string())
}

pub(crate) fn lane_rel_path(feature: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!(".bee{sep}lanes{sep}{feature}.json")
}

/// laneRecordFrom's approved_gates merge (defaults ...spread). Truthy
/// non-object approved_gates spreads exotic keys — Delegate.
pub(crate) fn merged_lane_gates(parsed: &Map<String, Value>) -> MR<Map<String, Value>> {
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    match parsed.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => {}
        Some(Value::Object(overlay)) => spread_into(&mut gates, overlay),
        Some(Value::Array(a)) if a.is_empty() => {}
        Some(Value::Number(_)) | Some(Value::Bool(true)) => {}
        Some(_) => return Err(Fail::Delegate),
    }
    Ok(gates)
}

/// lib/cells.mjs laneRecordForFeature — None (no lane record: default gate
/// governs) | Some(approved_gates). Both of readLaneStrict's refusals are
/// deterministic thrown messages now; the unreadable-file one carries the
/// Rust io error where Node interpolated the libuv errno.
pub(crate) fn lane_record_gates(root: &Path, feature: Option<&Value>) -> MR<Option<Map<String, Value>>> {
    let Some(Value::String(feature)) = feature else { return Ok(None) };
    if js_trim(feature).is_empty() {
        return Ok(None);
    }
    let Some(id) = lane_feature_ok(feature) else { return Ok(None) }; // lanePath throw, caught
    let file = lanes_dir(root).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    let text = match std::fs::read(&file) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        // readLaneStrict's unreadable branch. Node interpolated the libuv
        // err.code; the sentence and the refusal are otherwise unchanged.
        Err(e) => {
            return Err(Fail::Thrown(format!(
                "readLaneStrict: could not read lane record \"{}\" ({e}). The bee CLI refuses to mutate a lane it cannot read — that could silently clobber real lane state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {}\"), then retry.",
                file.display(),
                lane_rel_path(&id)
            )))
        }
    };
    let corrupt = || {
        Fail::Thrown(format!(
            "readLaneStrict: lane record \"{}\" exists but is corrupt (not a JSON object naming feature \"{id}\"). The bee CLI refuses to rebuild a lane from defaults over a present-but-corrupt file — that would silently clobber real lane state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {}\"), then retry.",
            file.display(),
            lane_rel_path(&id)
        ))
    };
    // A lone-surrogate escape lands in NotJson now and takes this same
    // deterministic corrupt refusal — the non-surrogate corrupt path.
    let parsed = match parse_json_js(&text, false) {
        JsParse::Value(v) => v,
        JsParse::NotJson => return Err(corrupt()),
    };
    let map = match parsed {
        Value::Object(m) => m,
        _ => return Err(corrupt()),
    };
    if !matches!(map.get("feature"), Some(Value::String(f)) if *f == id) {
        return Err(corrupt());
    }
    Ok(Some(merged_lane_gates(&map)?))
}

/// state.mjs readLane (fail-open display read) — only `route` truthiness is
/// consumed here (claimedFeatureHasRoute).
///
/// CUTOVER: a corrupt/mismatched record used to delegate, NOT because
/// readLane's own warning needed V8 (it is deterministic) but because it
/// would have stacked on top of readJson's V8-worded one. Both warnings are
/// ours now, so both are printed, in Node's order, and the read still fails
/// open to null.
pub(crate) fn read_lane_route(root: &Path, feature: &str) -> MR<Option<bool>> {
    let Some(id) = lane_feature_ok(feature) else { return Ok(None) };
    let file = lanes_dir(root).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    match read_store_json(&file)? {
        Some(Value::Object(m)) if matches!(m.get("feature"), Some(Value::String(f)) if *f == id) => {
            Ok(Some(m.get("route").map(js_truthy).unwrap_or(false)))
        }
        // laneRecordFrom returned falsy — a record that does not name this
        // feature, or readJson's null fallback after a corrupt file (a
        // MISSING file already returned above, so None here means corrupt).
        // readLane warns and reads as "no lane".
        _ => {
            let rel = lane_rel_path(&id);
            eprintln!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            );
            Ok(None)
        }
    }
}

/// bee.mjs claimedFeatureHasRoute (explicit-triage D3).
pub(crate) fn claimed_feature_has_route(root: &Path, feature: Option<&Value>) -> MR<bool> {
    let Some(feature) = feature else { return Ok(true) };
    if !js_truthy(feature) {
        return Ok(true);
    }
    let Value::String(feature_s) = feature else { return Err(Fail::Delegate) }; // non-string feature — JS-exotic path math
    if let Some(route) = read_lane_route(root, feature_s)? {
        return Ok(route);
    }
    let state = bstate::read_state_brief(root).map_err(|_| Fail::Delegate)?;
    if matches!(&state.feature, Value::String(f) if f == feature_s) {
        return Ok(js_truthy(&state.route));
    }
    Ok(true)
}
