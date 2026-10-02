use super::{control_root, list_claimed_cells, read_json_failopen};
use crate::fsutil::ReadJson;
use crate::hooks::adapter::{now_iso, HookContext};
use crate::verbs::state_group::{advisor_ref_anchors, advisor_ref_stale};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

pub(crate) fn answer(root: &Path, ctx: &HookContext) -> Option<String> {
    let skip_key = ctx.payload.get("skip_key").and_then(Value::as_str);
    let obligations_only = ctx.payload.get("obligations_only") == Some(&Value::Bool(true));
    if skip_key.is_none() && !obligations_only {
        return None;
    }
    let off = crate::hooks::herding_worker_marker_set()
        || crate::state::read_config_raw(root).get("pi_harness_workflow") == Some(&Value::Bool(false));
    let file = control_root(root, ctx).join(".bee").join("runtime").join("settle-obligations.json");
    if let Some(key) = skip_key {
        if !off {
            let _ = mark_served(&file, &[key.to_string()]);
        }
        return Some(String::new());
    }
    let owed = if off { Vec::new() } else { unserved(root, &file) };
    let keys: Vec<String> = owed.iter().filter_map(|o| o["key"].as_str().map(String::from)).collect();
    let owed = if keys.is_empty() || mark_served(&file, &keys).is_ok() { owed } else { Vec::new() };
    Some(json!({ "obligations": owed }).to_string())
}

fn read_served(file: &Path) -> Map<String, Value> {
    match read_json_failopen(file) {
        ReadJson::Parsed(Value::Object(m)) => match m.get("served") {
            Some(Value::Object(served)) => served.clone(),
            _ => Map::new(),
        },
        _ => Map::new(),
    }
}

fn mark_served(file: &Path, keys: &[String]) -> std::io::Result<()> {
    let mut served = read_served(file);
    let at = now_iso();
    for key in keys {
        served.insert(key.clone(), json!(at));
    }
    crate::fsutil::write_json_atomic(file, &json!({ "served": served }))
}

fn unserved(root: &Path, file: &Path) -> Vec<Value> {
    let served = read_served(file);
    let mut owed: Vec<Value> = Vec::new();
    for cell in list_claimed_cells(root).unwrap_or_default() {
        let Some(id) = cell.get("id").and_then(Value::as_str) else { continue };
        let feature = cell.get("feature").and_then(Value::as_str).unwrap_or("");
        owed.push(json!({
            "key": format!("{feature}:cap:{id}"),
            "kind": "cap",
            "cell": id,
            "message": format!(
                "bee: cell {id} is claimed and not capped. Cap it now with its proof line: \
bee cells finish --id {id} --outcome \"<one line>\" --files <a,b> --report '<json>'. \
If the work is not done, write .bee/HANDOFF.json and release the claim instead."
            ),
            "user_notice": format!(
                "bee: work is still owed: cell {id} is claimed but not capped. One extra turn starts now to cap it. \
(Esc stops it. This message will not repeat for {id}.)"
            ),
        }));
    }
    for record in planning_records(root) {
        let Some(feature) = record.get("feature").and_then(Value::as_str).filter(|f| !f.is_empty()) else {
            continue;
        };
        let staleness = advisor_ref_stale(root, record.get("advisor_ref"), &record);
        if !staleness.stale {
            continue;
        }
        let sha = advisor_ref_anchors(root, &json!(feature))["plan_sha256"].as_str().unwrap_or("").to_string();
        owed.push(json!({
            "key": format!("{feature}:advisor:{sha}"),
            "kind": "advisor",
            "message": format!(
                "bee: the high-risk plan for {feature} is gate-ready but has no fresh advisor consult ({}). \
Run the plan-step hat wave now: bee_advisor (bee dispatch prepare --kind advisor), synthesize it, then record it: \
bee state advisor-ref record --advisor \"<identity>\" --digest-file <path>. Do not approve the gate; the user does.",
                staleness.reasons.join("; ")
            ),
            "user_notice": format!(
                "bee: the plan for {feature} is ready for its gate but has no fresh advisor review. \
One extra turn starts now to run the advisor (this costs extra model calls). \
(Esc stops it. This message will not repeat for this plan.)"
            ),
        }));
    }
    let mut seen = std::collections::HashSet::new();
    owed.retain(|o| {
        let key = o["key"].as_str().unwrap_or("").to_string();
        !served.contains_key(&key) && seen.insert(key)
    });
    owed
}

fn planning_records(root: &Path) -> Vec<Map<String, Value>> {
    let bee = root.join(".bee");
    let mut files: Vec<PathBuf> = vec![bee.join("state.json")];
    if let Ok(entries) = std::fs::read_dir(bee.join("lanes")) {
        let mut lanes: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        lanes.sort();
        files.extend(lanes);
    }
    files
        .iter()
        .filter_map(|f| match read_json_failopen(f) {
            ReadJson::Parsed(Value::Object(m)) => Some(m),
            _ => None,
        })
        .filter(|m| {
            matches!(m.get("mode"), Some(Value::String(s)) if s == "high-risk")
                && matches!(m.get("phase"), Some(Value::String(s)) if s == "planning" || s == "validating")
                && matches!(m.get("gate_preview"), Some(Value::Object(p)) if !p.is_empty())
        })
        .collect()
}
