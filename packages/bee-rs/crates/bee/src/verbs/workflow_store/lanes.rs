// the lane layer
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
    let gates = spread_gates(obj.get("approved_gates"));
    merged.insert("approved_gates".into(), Value::Object(gates));
    coerce_legacy_phase(&mut merged)?;
    Ok(Some(merged))
}

/// state.mjs readLane — the fail-open DISPLAY read. Every corrupt shape warns
/// and reads as "no lane". A JSON-corrupt file produces TWO lines, exactly as
/// Node did: readJson's own `could not parse JSON at …` (our wording) and
/// then readLane's `skipping corrupt lane record …`, because `readJson`'s
/// `null` fallback is what makes `laneRecordFrom` answer null.
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
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&file);
            warn();
            Ok(None)
        }
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
            // CUTOVER: `(${err.code})` was a libuv errno string, so only
            // EISDIR/EPERM were reproduced and every other class delegated.
            // Same refusal, same exit code — an engine-free category now, for
            // EVERY read failure.
            return Err(Err2::Msg(format!(
                "readLaneStrict: could not read lane record \"{file_str}\" ({}). The bee CLI refuses to mutate a lane it cannot read — that could silently clobber real lane state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.",
                io_read_reason(&file, &e)
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
