// the config, state and lane layers
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;

// ─── config (state.mjs readConfig / mergeConfigOverlay / normalizeCommands) ─
//
// provenance: lib/state.mjs readConfig (l. 1947) + mergeConfigOverlay
// (l. 1919); Rust lift of crate::state::{merge_config_overlay,
// read_config_raw} (state.rs:109-156) with the ONE change the preamble
// needs: read_config_raw bails on a corrupt config file, and a preamble may
// not bail — the corrupt file warns and reads as absent instead.

pub(crate) fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {
    match overlay {
        Value::Array(items) => Value::Array(items.clone()),
        Value::Object(over) => {
            let base_obj = match base {
                Value::Object(m) => m.clone(),
                _ => JMap::new(),
            };
            let mut out = base_obj.clone();
            for (key, value) in over {
                let merged = match (base_obj.get(key), value) {
                    (Some(b @ Value::Object(_)), Value::Object(_)) => merge_config_overlay(b, value),
                    _ => match value {
                        Value::Array(items) => Value::Array(items.clone()),
                        other => other.clone(),
                    },
                };
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => base.clone(),
    }
}

/// The merged tracked+overlay config object, advisor key stripped. Fail-open.
pub(crate) fn read_config_raw_open(root: &Path) -> JMap {
    let tracked = read_json_object(&root.join(".bee").join("config.json")).unwrap_or_default();
    let overlay = read_json_object(&root.join(".bee").join("config.local.json"));
    let mut merged = match overlay {
        Some(over) => match merge_config_overlay(&Value::Object(tracked), &Value::Object(over)) {
            Value::Object(m) => m,
            _ => JMap::new(),
        },
        None => tracked,
    };
    merged.shift_remove("advisor");
    merged
}

/// provenance: state.mjs normalizeCommands; Rust lift of
/// verbs/status_full.rs:800-830.
pub(crate) fn normalize_commands(raw: Option<&Value>) -> JMap {
    let mut commands = JMap::new();
    let Some(Value::Object(obj)) = raw else { return commands };
    for key in COMMAND_KEYS.iter().chain(WORKTREE_COMPANION_COMMAND_KEYS.iter()) {
        match obj.get(*key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                commands.insert((*key).into(), json!(js_trim(s)));
            }
            Some(Value::Array(items)) if *key == "test" => {
                let list: Vec<Value> = items
                    .iter()
                    .filter_map(|c| c.as_str())
                    .filter(|c| !js_trim(c).is_empty())
                    .map(|c| json!(js_trim(c)))
                    .collect();
                if !list.is_empty() {
                    commands.insert((*key).into(), Value::Array(list));
                }
            }
            _ => {}
        }
    }
    commands
}

/// provenance: state.mjs resolveProductRoot (l. 1065); Rust lift of
/// verbs/status_full.rs:1381-1420 with its buffered warns printed straight to
/// stderr (this module has no emit-time buffer — nothing here can bail, so a
/// warning can never leak alongside partial output).
pub(crate) fn resolve_product_root(root: &Path) -> PathBuf {
    let config = read_config_raw_open(root);
    match config.get("product_root") {
        None | Some(Value::Null) => root.to_path_buf(),
        Some(Value::String(s)) if s.is_empty() => root.to_path_buf(),
        Some(Value::String(s)) => {
            let resolved = if Path::new(s).is_absolute() {
                PathBuf::from(s)
            } else {
                root.join(s)
            };
            let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
            if !is_dir {
                eprintln!(
                    "bee: config product_root \"{s}\" -> \"{}\" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)",
                    resolved.display()
                );
            }
            resolved
        }
        Some(other) => {
            let ty = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => "object",
            };
            eprintln!(
                "bee: .bee/config.json product_root must be a string path (got {ty}); ignoring it and using the bee root."
            );
            root.to_path_buf()
        }
    }
}

/// state.mjs bypassBanner — one canonical loud line per active level.
pub(crate) fn bypass_banner(level: &str) -> &'static str {
    match level {
        "total" => "⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-hive bypass off",
        "full" => "⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-3 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-hive bypass off",
        "normal" => "⚡ GATE BYPASS: NORMAL — Gates 1-3 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop. Turn off: bee-hive bypass off",
        _ => "",
    }
}

// ─── state layer (state.mjs readState / readOnboarding / readHandoff) ──────
//
// provenance: lib/state.mjs defaultState/readState (l. 1097-1126); Rust lift
// of hooks/chain_nudge.rs:246-288, with the corrupt-file and JS-exotic
// approved_gates arms turned fail-open (warn + defaults) instead of Delegate.

pub(crate) fn default_gates() -> JMap {
    let mut m = JMap::new();
    for g in GATE_NAMES {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

pub(crate) fn default_state() -> JMap {
    let mut m = JMap::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!("No active bee work — awaiting a user request."));
    m
}

/// Merge `{...defaults, ...overlay}` for a gates-shaped field: falsy overlays
/// (and any non-object) leave the defaults, JS-spread exotica included — the
/// preamble only ever READS gate booleans, so an index-keyed spread of a
/// string could not change a rendered gate either way.
pub(crate) fn merge_gates(overlay: Option<&Value>) -> JMap {
    match overlay {
        Some(Value::Object(o)) => {
            let mut g = default_gates();
            for (k, v) in o {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        _ => default_gates(),
    }
}

/// state.mjs readState — fail-open merge over defaultState() with the D13
/// legacy 'validating' -> 'planning' coercion.
pub(crate) fn read_state(root: &Path) -> JMap {
    let file_state = read_json_object(&root.join(".bee").join("state.json"));
    let mut merged = default_state();
    let Some(state) = file_state else { return merged };
    for (k, v) in &state {
        merged.insert(k.clone(), v.clone()); // existing keys keep position (JS spread)
    }
    merged.insert("approved_gates".into(), Value::Object(merge_gates(state.get("approved_gates"))));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning"));
    }
    merged
}

/// state.mjs readOnboarding — `readJson(.bee/onboarding.json, null)`.
pub(crate) fn read_onboarding(root: &Path) -> Option<Value> {
    read_json_open(&root.join(".bee").join("onboarding.json")).filter(truthy_ref)
}

pub(crate) fn truthy_ref(v: &Value) -> bool {
    truthy(v)
}

/// state.mjs readHandoff (l. 1217) — the fail-open DISPLAY read, with `kind`
/// normalized for objects (missing/unknown reads as 'pause', the safe
/// surface-and-wait side). `None` models every falsy result Node's
/// `readJson(file, null)` could hand back, which is exactly what
/// `handoffBlockLines`' `if (!handoff)` guard tests.
///
/// Exported because lib/compaction.mjs's capsule reads the same record — one
/// truth for the handoff block, never two copies of it.
pub fn read_handoff(root: &Path) -> Option<Value> {
    let parsed = read_json_open(&root.join(".bee").join("HANDOFF.json"))?;
    if !truthy(&parsed) {
        return None;
    }
    match parsed {
        Value::Object(m) => {
            let kind = if str_eq(m.get("kind"), "planned-next") { "planned-next" } else { "pause" };
            let mut out = m;
            out.insert("kind".into(), json!(kind)); // JS {...handoff, kind}: key keeps its position
            Some(Value::Object(out))
        }
        other => Some(other),
    }
}

// ─── lanes (state.mjs listLanes / readLane / lanePath) ─────────────────────
//
// provenance: lib/state.mjs l. 1698-1829; Rust lift of
// hooks/chain_nudge.rs:427-505 (read_lane_record, warn_corrupt_lane,
// path_relative) and verbs/status_full.rs:1612-1636 (list_lanes).

pub(crate) fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — a throw reads as "no lane" at every
/// fail-open call site.
pub(crate) fn require_lane_feature(value: Option<&Value>) -> Option<String> {
    let Some(Value::String(s)) = value else { return None };
    let feature = js_trim(s);
    if feature.is_empty() {
        return None;
    }
    if feature.contains('/') || feature.contains('\\') || feature.contains("..") {
        return None;
    }
    Some(feature.to_string())
}

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

pub(crate) fn warn_corrupt_lane(root: &Path, file: &Path) {
    let rel = path_relative(root, file);
    eprintln!(
        "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
    );
}

/// state.mjs readLane — fail-open display read; a corrupt or mismatched
/// record warns (byte-identical line) and reads as absent.
pub(crate) fn read_lane(root: &Path, feature: Option<&Value>) -> Option<JMap> {
    let name = require_lane_feature(feature)?;
    let file = lanes_dir(root).join(format!("{name}.json"));
    if !file.exists() {
        return None;
    }
    let parsed = read_json_open(&file);
    let Some(Value::Object(parsed)) = parsed else {
        warn_corrupt_lane(root, &file);
        return None;
    };
    if parsed.get("feature") != Some(&Value::String(name.clone())) {
        warn_corrupt_lane(root, &file);
        return None;
    }
    let mut merged = JMap::new();
    merged.insert("schema_version".into(), json!("1.0"));
    merged.insert("feature".into(), Value::String(name));
    merged.insert("mode".into(), Value::Null);
    merged.insert("phase".into(), json!("idle"));
    merged.insert("approved_gates".into(), Value::Object(default_gates()));
    merged.insert("summary".into(), json!(""));
    merged.insert("next_action".into(), json!(""));
    merged.insert("created_at".into(), Value::Null);
    for (k, v) in &parsed {
        merged.insert(k.clone(), v.clone());
    }
    merged.insert("approved_gates".into(), Value::Object(merge_gates(parsed.get("approved_gates"))));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning"));
    }
    Some(merged)
}

/// state.mjs listLanes — fail-open enumeration in directory order.
pub(crate) fn list_lanes(root: &Path) -> Vec<JMap> {
    let Ok(entries) = std::fs::read_dir(lanes_dir(root)) else { return Vec::new() };
    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let mut lanes = Vec::new();
    for entry in names {
        let Some(stem) = entry.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane(root, Some(&json!(stem))) {
            lanes.push(record);
        }
    }
    lanes
}

// ─── sessions + roots (claims.mjs readSession, state.mjs controlRootFor) ───
//
// provenance: Rust lift of hooks/chain_nudge.rs:290-440, with every Delegate
// and Crash arm collapsed into the fail-open direction (read as "no session"
// / "the root you were given").

pub(crate) fn well_formed_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// claims.mjs readSession (strict=false) — fail-open display read.
pub(crate) fn read_session(control_root: &Path, session_id: &str) -> Option<JMap> {
    let id = js_trim(session_id);
    if id.is_empty() || !well_formed_id(id) {
        return None;
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{id}.json"));
    let session = read_json_object(&file)?;
    if session.get("id") != Some(&Value::String(id.to_string())) {
        return None;
    }
    Some(session)
}

pub(crate) fn js_absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

/// state.mjs readGitdirFile.
pub(crate) fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
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

pub(crate) enum RootsCore {
    None,
    Ordinary(PathBuf),
    LinkedValid(PathBuf),
}

/// state.mjs resolveRootsCore — an invalid link reads as "no linked main",
/// never the WorktreeLinkInvalidError throw (this renderer is total).
pub(crate) fn resolve_roots_core(start: &Path) -> RootsCore {
    let mut nearest = js_absolute(start);
    loop {
        if nearest.join(".bee").join("onboarding.json").exists() && !nearest.join(".git").exists() {
            return RootsCore::Ordinary(nearest);
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
                return RootsCore::Ordinary(dir);
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return RootsCore::None,
            }
        }
    };
    let marker = work_root.join(".git");
    let Ok(stat) = std::fs::metadata(&marker) else { return RootsCore::Ordinary(work_root) };
    if !stat.is_file() {
        return RootsCore::Ordinary(work_root);
    }
    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else {
        return RootsCore::Ordinary(work_root);
    };
    let worktrees_root = gitdir.parent().map(Path::to_path_buf).unwrap_or_default();
    let common_git_dir = worktrees_root.parent().map(Path::to_path_buf).unwrap_or_default();
    if !(common_git_dir.file_name().is_some_and(|n| n == ".git")
        && worktrees_root.file_name().is_some_and(|n| n == "worktrees"))
    {
        return RootsCore::Ordinary(work_root);
    }
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if id.is_empty() || id == "." || id == ".." {
        return RootsCore::Ordinary(work_root);
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    if reverse.as_deref() != Some(js_absolute(&marker).as_path()) {
        return RootsCore::Ordinary(work_root);
    }
    match common_git_dir.parent().map(Path::to_path_buf) {
        Some(main_root) => RootsCore::LinkedValid(main_root),
        None => RootsCore::Ordinary(work_root),
    }
}

/// state.mjs controlRootFor(root).
pub(crate) fn control_root_for(root: &Path) -> PathBuf {
    match resolve_roots_core(root) {
        RootsCore::None => root.to_path_buf(),
        RootsCore::Ordinary(work_root) => work_root,
        RootsCore::LinkedValid(main_root) => main_root,
    }
}

/// state.mjs resolvePipeline's return, narrowed to what the preamble reads.
pub(crate) struct Pipeline {
    pub(crate) ok: bool,
    pub(crate) source: &'static str,
    pub(crate) feature: Option<String>,
    pub(crate) record: JMap,
}

/// state.mjs resolvePipeline (l. 1854): session record -> bound lane ->
/// default state.json. A binding that names an invalid/missing/corrupt lane
/// is a typed refusal (`ok:false`), never a silent fallback — the caller
/// then renders the DEFAULT record, exactly as inject.mjs does.
pub(crate) fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Pipeline {
    let defaults = || Pipeline {
        ok: true,
        source: "default",
        feature: None,
        record: read_state(root),
    };
    let refusal = || Pipeline {
        ok: false,
        source: "default",
        feature: None,
        record: JMap::new(),
    };
    let Some(sid) = session_id.filter(|s| !js_trim(s).is_empty()) else { return defaults() };
    let control_root = control_root_for(root);
    let Some(session) = read_session(&control_root, sid) else { return defaults() };
    let bound = match session.get("lane") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if bound.is_empty() {
        return defaults();
    }
    if !well_formed_id(&bound) {
        return refusal(); // LANE_INVALID
    }
    let file = lanes_dir(&control_root).join(format!("{bound}.json"));
    if !file.exists() {
        return refusal(); // LANE_MISSING
    }
    match read_lane(&control_root, Some(&json!(bound.clone()))) {
        Some(record) => Pipeline { ok: true, source: "lane", feature: Some(bound), record },
        None => refusal(), // LANE_CORRUPT
    }
}
