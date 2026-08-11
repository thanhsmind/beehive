// lanes, cells, claims/sessions and the reservation leases
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

// ─── lanes (state.mjs) ─────────────────────────────────────────────────────

pub(crate) fn lanes_dir(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — Err = the JS throw (bad name).
pub(crate) fn require_lane_feature(value: &str) -> Result<String, ()> {
    let feature = js_trim(value);
    if feature.is_empty() {
        return Err(());
    }
    if feature.contains('\\') || feature.contains('/') || feature.contains("..") {
        return Err(());
    }
    Ok(feature.to_string())
}

pub(crate) fn default_lane_record(feature: &str) -> JMap {
    let mut m = JMap::new();
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

/// state.mjs laneRecordFrom — None when not an object naming THIS feature.
/// Truthy non-object approved_gates -> bail (JS-exotic spread).
pub(crate) fn lane_record_from(feature: &str, parsed: Option<&Value>) -> R<Option<JMap>> {
    let Some(Value::Object(obj)) = parsed else { return Ok(None) };
    if !str_eq(obj.get("feature"), feature) {
        return Ok(None);
    }
    let mut merged = default_lane_record(feature);
    for (k, v) in obj {
        merged.insert(k.clone(), v.clone());
    }
    let gates = match obj.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        Some(_) => return Err(Ex::Bail),
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if str_eq(merged.get("phase"), "validating") {
        merged.insert("phase".into(), json!("planning"));
    }
    Ok(Some(merged))
}

/// state.mjs readLane — fail-open display read. A present-but-corrupt record
/// produces BOTH lines Node produced, in Node's order: readJson's own
/// could-not-parse warning (our wording) and then readLane's
/// skipping-corrupt-lane-record line, because readJson's `null` fallback is
/// what makes `laneRecordFrom` answer null. A record that parses but
/// mismatches feature warns readLane's line only.
pub(crate) fn read_lane(ctx: &mut Ctx, feature: &str) -> R<Option<JMap>> {
    let Ok(id) = require_lane_feature(feature) else {
        return Ok(None);
    };
    let file = lanes_dir(ctx).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    // Corrupt: rj warns, then laneRecordFrom(null) is null, so readLane's own
    // line follows — both of the lines Node printed, in Node's order.
    let parsed = rj(ctx, &file)?;
    let trimmed = js_trim(feature).to_string();
    let record = lane_record_from(&trimmed, parsed.as_ref())?;
    if record.is_none() {
        // Node: console.warn with path.relative(root, file) — POSIX-ish only
        // when file sits under root (always true here).
        let rel = format!(".bee{sep}lanes{sep}{id}.json", sep = std::path::MAIN_SEPARATOR);
        ctx.warn(format!(
            "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        ));
        return Ok(None);
    }
    Ok(record)
}

/// state.mjs listLanes — fail-open enumeration in directory order.
pub(crate) fn list_lanes(ctx: &mut Ctx) -> R<Vec<JMap>> {
    let Ok(entries) = std::fs::read_dir(lanes_dir(ctx)) else {
        return Ok(Vec::new());
    };
    // Node readdirSync returns the OS enumeration order; Rust read_dir uses
    // the same OS API, so the order is preserved rather than re-sorted.
    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let mut lanes = Vec::new();
    for entry in names {
        let Some(stem) = entry.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane(ctx, stem)? {
            lanes.push(record);
        }
    }
    Ok(lanes)
}

// ─── cells (cells.mjs) ─────────────────────────────────────────────────────

pub(crate) fn cells_dir(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("cells")
}

pub(crate) const ARCHIVE_DIR_NAME: &str = "archive";

/// cells.mjs ID_PATTERN /^[A-Za-z0-9][A-Za-z0-9._-]*$/ over String(id).
pub(crate) fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// PBI p-9c48a67c / ips-1 read-side residue (irf-1): mirrors
/// `verbs::cells::read::island_feature_scope` for this ctx-based
/// enumerator — same grant test, already resolved ONCE into `ctx.linked`
/// (see `Ctx::granted_worktree`), same creation-identity read
/// (`read_worktree_feature`, this module). `None` for anything that is not
/// a granted worktree island; reads on those checkouts stay byte-identical.
fn island_feature_scope(ctx: &Ctx) -> Option<String> {
    ctx.granted_worktree()?;
    read_worktree_feature(&ctx.root.to_string_lossy())
}

/// cells.mjs listCells (includeArchived always false on the status path).
/// feature/status filters use JS strict !==; sort by id, numeric 'en'.
pub(crate) fn list_cells(ctx: &Ctx, feature: Option<&Value>, status: Option<&str>) -> R<Vec<Value>> {
    let island_feature = island_feature_scope(ctx);
    let dir = cells_dir(ctx);
    let mut cells: Vec<Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue; // the `archive` child (or any dir) is never a cell
            }
            if !name.ends_with(".json") {
                continue;
            }
            let Some(cell) = rj(ctx, &entry.path())? else { continue };
            if !matches!(cell, Value::Object(_) | Value::Array(_)) {
                continue; // `typeof cell !== 'object'` (null already skipped)
            }
            if let Some(scope) = island_feature.as_deref() {
                if !str_eq(vget(&cell, "feature"), scope) {
                    continue; // foreign-feature residue in a granted island — never surfaced
                }
            }
            if let Some(f) = feature {
                if truthy(f) && !strict_eq(vget(&cell, "feature"), Some(f)) {
                    continue;
                }
            }
            if let Some(s) = status {
                if !str_eq(vget(&cell, "status"), s) {
                    continue;
                }
            }
            cells.push(cell);
        }
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

/// Archive-aware sibling of `list_cells`, for debt-door-archive dda-2: `bee
/// close` archives a feature's cells on a green close
/// (`.bee/cells/archive/<feature>/*.json`), so a debt counter that only
/// walks the live store the way `list_cells` does goes structurally silent
/// the moment its own feature closes. This reads the live store (exactly as
/// `list_cells` does) THEN every archived `.json` under
/// `.bee/cells/archive/<feature>/` (a truthy `feature` filter) or under
/// every `.bee/cells/archive/*/` subdirectory (no filter — the global orphan
/// sweep needs every feature's archive slot), deduplicating by id with the
/// LIVE copy winning on a duplicate — the same live-copy-wins pattern
/// `verbs/drivers/guard.rs:218 list_cells_including_archive` already uses
/// for `bee close`'s own door. `list_cells` itself is untouched and stays
/// active-only: every other caller (`bee cells list`, `bee cells ready`,
/// reviews, compaction attribution, prompt-context, …) keeps its current
/// behavior. Only `scribing_debt` and `global_scribing_debt` below call
/// this variant.
pub(crate) fn list_cells_including_archive(
    ctx: &Ctx,
    feature: Option<&Value>,
    status: Option<&str>,
) -> R<Vec<Value>> {
    let island_feature = island_feature_scope(ctx);
    let mut cells = list_cells(ctx, feature, status)?;
    let mut seen_ids: HashSet<String> = cells.iter().map(|c| tpl(vget(c, "id"))).collect();
    let archive_root = cells_dir(ctx).join(ARCHIVE_DIR_NAME);
    let feature_dirs: Vec<PathBuf> = match feature.filter(|f| truthy(f)) {
        Some(f) => vec![archive_root.join(jsjson::js_to_string(f))],
        None => {
            let Ok(entries) = std::fs::read_dir(&archive_root) else {
                return Ok(cells);
            };
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.path())
                .collect()
        }
    };
    for dir in feature_dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let Some(cell) = rj(ctx, &entry.path())? else { continue };
            if !matches!(cell, Value::Object(_) | Value::Array(_)) {
                continue; // `typeof cell !== 'object'` (null already skipped)
            }
            if let Some(scope) = island_feature.as_deref() {
                if !str_eq(vget(&cell, "feature"), scope) {
                    continue; // foreign-feature residue in a granted island — never surfaced
                }
            }
            if let Some(f) = feature {
                if truthy(f) && !strict_eq(vget(&cell, "feature"), Some(f)) {
                    continue;
                }
            }
            if let Some(s) = status {
                if !str_eq(vget(&cell, "status"), s) {
                    continue;
                }
            }
            let id = tpl(vget(&cell, "id"));
            if !seen_ids.insert(id) {
                continue; // the live copy above already claimed this id
            }
            cells.push(cell);
        }
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

/// cells.mjs readCell — active file first, then the archive fallback.
pub(crate) fn read_cell(ctx: &Ctx, id: &Value) -> R<Option<Value>> {
    let id_str = jsjson::js_to_string(id);
    if !truthy(id) || !id_pattern_ok(&id_str) {
        return Ok(None);
    }
    let active = rj(ctx, &cells_dir(ctx).join(format!("{id_str}.json")))?;
    if active.is_some() {
        return Ok(active);
    }
    let archive_root = cells_dir(ctx).join(ARCHIVE_DIR_NAME);
    let Ok(entries) = std::fs::read_dir(&archive_root) else {
        return Ok(None);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(format!("{id_str}.json"));
        if let Some(v) = rj(ctx, &candidate)? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// cells.mjs archivedTotals over the archive summary ledger.
pub(crate) fn archived_totals(ctx: &Ctx) -> R<JMap> {
    let file = cells_dir(ctx).join(ARCHIVE_DIR_NAME).join("summary.json");
    let summary = match rj(ctx, &file)? {
        Some(Value::Object(m)) => m,
        _ => JMap::new(),
    };
    let (mut capped, mut dropped) = (0f64, 0f64);
    for entry in summary.values() {
        let Value::Object(e) = entry else { continue };
        if let Some(n) = e.get("capped").and_then(|v| v.as_f64()) {
            if n.is_finite() {
                capped += n;
            }
        }
        if let Some(n) = e.get("dropped").and_then(|v| v.as_f64()) {
            if n.is_finite() {
                dropped += n;
            }
        }
    }
    let mut out = JMap::new();
    out.insert("capped".into(), json_num(capped));
    out.insert("dropped".into(), json_num(dropped));
    out.insert("total".into(), json_num(capped + dropped));
    Ok(out)
}

/// JS number -> Value (whole f64 collapses to integer like JSON.stringify).
pub(crate) fn json_num(n: f64) -> Value {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 9.007_199_254_740_992e15 {
        json!(n as i64)
    } else if n.is_finite() {
        json!(n)
    } else {
        Value::Null // JSON.stringify(NaN/Infinity) -> null
    }
}

/// cells.mjs readyCells: open cells whose deps are all capped.
pub(crate) fn ready_cells(ctx: &Ctx, feature: Option<&Value>) -> R<Vec<Value>> {
    let open = list_cells(ctx, feature, Some("open"))?;
    let mut ready = Vec::new();
    for cell in open {
        let mut all_capped = true;
        if let Some(Value::Array(deps)) = vget(&cell, "deps") {
            for dep in deps {
                let dep_cell = read_cell(ctx, dep)?;
                let capped = dep_cell
                    .as_ref()
                    .map(|c| str_eq(vget(c, "status"), "capped"))
                    .unwrap_or(false);
                if !capped {
                    all_capped = false;
                }
            }
        }
        if all_capped {
            ready.push(cell);
        }
    }
    Ok(ready)
}

/// cells.mjs readScribingLedger.
pub(crate) fn read_scribing_ledger(ctx: &Ctx) -> Vec<Value> {
    read_jsonl(&ctx.root.join(".bee").join("logs").join("scribing-runs.jsonl"))
}

/// cells.mjs scribingRunStampMs.
pub(crate) fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    // Date.parse(run.at || run.date) — JS ||.
    let at = vget(run, "at").filter(|v| truthy(v));
    let chosen = at.or_else(|| vget(run, "date"));
    let parsed = date_parse_val(chosen);
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

/// cells.mjs bestScribingStampMs — ledger max, then the feature's lane stamp,
/// then the default record's stamp when it names this feature.
pub(crate) fn best_scribing_stamp_ms(
    ctx: &mut Ctx,
    feature: &Value,
    ledger: &[Value],
    state: &JMap,
) -> R<Option<f64>> {
    let mut best: Option<f64> = None;
    for entry in ledger {
        if !truthy(entry) || !strict_eq(vget(entry, "feature"), Some(feature)) {
            continue;
        }
        let parsed = date_parse_val(vget(entry, "ts"));
        if parsed.is_finite() && best.map(|b| parsed > b).unwrap_or(true) {
            best = Some(parsed);
        }
    }
    let feature_str = jsjson::js_to_string(feature);
    let lane = read_lane(ctx, &feature_str)?;
    if let Some(lane) = lane {
        if let Some(stamp) = scribing_run_stamp_ms(lane.get("last_scribing_run")) {
            if best.map(|b| stamp > b).unwrap_or(true) {
                best = Some(stamp);
            }
        }
    }
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(feature)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    Ok(best)
}

/// cells.mjs scribingDebt(root) — no opts on the status path.
pub(crate) fn scribing_debt(ctx: &mut Ctx) -> R<JMap> {
    let state = read_state_full(ctx)?;
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let mut out = JMap::new();
    if !truthy(&feature) {
        out.insert("count".into(), json!(0));
        out.insert("cells".into(), json!([]));
        return Ok(out);
    }
    let ledger = read_scribing_ledger(ctx);
    let threshold = best_scribing_stamp_ms(ctx, &feature, &ledger, &state)?.unwrap_or(0.0);
    // dda-2: archive-aware, so a feature that just closed (and got archived)
    // still shows its unpaid debt instead of going structurally silent.
    let capped = list_cells_including_archive(ctx, Some(&feature), Some("capped"))?;
    let mut ids = Vec::new();
    for cell in capped {
        let trace = vget(&cell, "trace").cloned().unwrap_or(json!({}));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        if capped_at.is_finite() && capped_at > threshold {
            ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        }
    }
    out.insert("count".into(), json!(ids.len()));
    out.insert("cells".into(), Value::Array(ids));
    Ok(out)
}

/// cells.mjs globalScribingDebt — the orphan sweep across every feature.
pub(crate) fn global_scribing_debt(ctx: &mut Ctx) -> R<JMap> {
    // dda-2: archive-aware. An orphaned feature's cells are exactly the ones
    // most likely to have been archived already, so the sweep must see them.
    let capped = list_cells_including_archive(ctx, None, Some("capped"))?;
    let cells: Vec<Value> = capped
        .into_iter()
        .filter(|cell| {
            let trace = vget(cell, "trace");
            matches!(trace.and_then(|t| vget(t, "behavior_change")), Some(Value::Bool(true)))
        })
        .collect();
    let mut out = JMap::new();
    if cells.is_empty() {
        out.insert("count".into(), json!(0));
        out.insert("features".into(), json!([]));
        return Ok(out);
    }
    let state = read_state_full(ctx)?;
    let ledger = read_scribing_ledger(ctx);
    let mut stamp_cache: HashMap<String, Option<f64>> = HashMap::new();
    // Insertion-ordered feature -> ids map (JS Map).
    let mut order: Vec<String> = Vec::new();
    let mut by_feature: HashMap<String, Vec<Value>> = HashMap::new();
    for cell in &cells {
        let feature_v = vget(cell, "feature");
        if !opt_truthy(feature_v) {
            continue;
        }
        let feature_v = feature_v.unwrap().clone();
        let feature_key = jsjson::js_to_string(&feature_v);
        let trace = vget(cell, "trace").cloned().unwrap_or(json!({}));
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        let stamp = match stamp_cache.get(&feature_key) {
            Some(s) => *s,
            None => {
                let s = best_scribing_stamp_ms(ctx, &feature_v, &ledger, &state)?;
                stamp_cache.insert(feature_key.clone(), s);
                s
            }
        };
        let orphaned = match stamp {
            None => true,
            Some(s) => capped_at.is_finite() && capped_at > s,
        };
        if !orphaned {
            continue;
        }
        if !by_feature.contains_key(&feature_key) {
            order.push(feature_key.clone());
            by_feature.insert(feature_key.clone(), Vec::new());
        }
        by_feature
            .get_mut(&feature_key)
            .unwrap()
            .push(vget(cell, "id").cloned().unwrap_or(Value::Null));
    }
    // sort((a,b) => a.feature.localeCompare(b.feature, 'en')) — non-numeric.
    order.sort_by(|a, b| locale_cmp(a, b, false));
    let mut features = Vec::new();
    let mut count = 0usize;
    for feature in order {
        let ids = by_feature.remove(&feature).unwrap_or_default();
        count += ids.len();
        let mut row = JMap::new();
        row.insert("feature".into(), json!(feature));
        row.insert("cells".into(), Value::Array(ids));
        features.push(Value::Object(row));
    }
    out.insert("count".into(), json!(count));
    out.insert("features".into(), Value::Array(features));
    Ok(out)
}

pub(crate) struct TierMix {
    pub(crate) counts: JMap,
    pub(crate) tiered: i64,
    pub(crate) ceiling: i64,
    pub(crate) ceiling_share: f64,
}

/// cells.mjs tierMix.
pub(crate) fn tier_mix(ctx: &Ctx, feature: Option<&Value>) -> R<TierMix> {
    // tierMix passes {} (no filter) when feature is null.
    let filter = feature.filter(|f| truthy(f));
    let cells = list_cells(ctx, filter, None)?;
    let (mut extraction, mut generation, mut ceiling, mut untiered) = (0i64, 0i64, 0i64, 0i64);
    for cell in &cells {
        match vget(cell, "tier").and_then(|t| t.as_str()) {
            Some(t) if MODEL_TIERS.contains(&t) => match t {
                "extraction" => extraction += 1,
                "generation" => generation += 1,
                _ => ceiling += 1,
            },
            _ => untiered += 1,
        }
    }
    let tiered = extraction + generation + ceiling;
    let ceiling_share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    let mut counts = JMap::new();
    counts.insert("extraction".into(), json!(extraction));
    counts.insert("generation".into(), json!(generation));
    counts.insert("ceiling".into(), json!(ceiling));
    counts.insert("untiered".into(), json!(untiered));
    Ok(TierMix { counts, tiered, ceiling, ceiling_share })
}

/// cells.mjs ceilingScarcityWarning.
pub(crate) fn ceiling_scarcity_warning(ctx: &mut Ctx) -> R<Option<JMap>> {
    let state = read_state_full(ctx)?;
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let feature_arg = if truthy(&feature) { Some(feature) } else { None };
    let mix = tier_mix(ctx, feature_arg.as_ref())?;
    if mix.tiered < SCARCITY_MIN_TIERED {
        return Ok(None);
    }
    if mix.ceiling_share <= CEILING_MAX_SHARE {
        return Ok(None);
    }
    let mut out = JMap::new();
    out.insert("pct".into(), json_num(js_round(mix.ceiling_share * 100.0)));
    out.insert("ceiling".into(), json!(mix.ceiling));
    out.insert("tiered".into(), json!(mix.tiered));
    Ok(Some(out))
}

// ─── claims / sessions (claims.mjs) ────────────────────────────────────────

pub(crate) fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

pub(crate) fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// claims.mjs requireId — Err = the JS throw.
pub(crate) fn require_id(value: &str) -> Result<String, ()> {
    let id = js_trim(value);
    if id.is_empty() || id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(());
    }
    Ok(id.to_string())
}

/// claims.mjs readSession (strict=false).
pub(crate) fn read_session(ctx: &Ctx, root: &Path, session_id: &str) -> R<Option<JMap>> {
    let Ok(id) = require_id(session_id) else {
        return Ok(None);
    };
    let file = sessions_dir(root).join(format!("{id}.json"));
    let Some(session) = rj(ctx, &file)? else { return Ok(None) };
    let Value::Object(m) = session else { return Ok(None) };
    if !str_eq(m.get("id"), js_trim(session_id)) {
        return Ok(None);
    }
    Ok(Some(m))
}

/// claims.mjs listSessionRecords (strict=false), directory order.
pub(crate) fn list_session_records(ctx: &Ctx, root: &Path) -> R<Vec<JMap>> {
    let Ok(entries) = std::fs::read_dir(sessions_dir(root)) else {
        return Ok(Vec::new());
    };
    let mut sessions = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_session(ctx, root, stem)? {
            sessions.push(record);
        }
    }
    Ok(sessions)
}

/// claims.mjs heartbeatStale.
pub(crate) fn heartbeat_stale(session: &JMap, now_ms_v: f64) -> bool {
    let beat = date_parse_val(session.get("last_heartbeat"));
    if !beat.is_finite() {
        return true;
    }
    beat + DEFAULT_HEARTBEAT_STALE_SECONDS * 1000.0 <= now_ms_v
}

/// claims.mjs resolveSessionId — flag(unused here)/env/env-legacy, then the
/// D5 single-live-session adoption when `root` is supplied.
pub(crate) fn resolve_session_id(ctx: &Ctx, root: Option<&Path>) -> R<Option<String>> {
    for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(var) {
            if !js_trim(&v).is_empty() {
                return Ok(Some(js_trim(&v).to_string()));
            }
        }
    }
    if let Some(root) = root {
        let now = now_ms();
        let fresh: Vec<JMap> = list_session_records(ctx, root)?
            .into_iter()
            .filter(|s| !heartbeat_stale(s, now))
            .collect();
        if fresh.len() == 1 {
            return Ok(fresh[0].get("id").and_then(|v| v.as_str()).map(str::to_string));
        }
    }
    Ok(None)
}

/// claims.mjs readClaim.
pub(crate) fn read_claim(ctx: &Ctx, root: &Path, cell_id: &str) -> R<Option<JMap>> {
    let Ok(id) = require_id(cell_id) else {
        return Err(Ex::Thrown); // claimPath's requireId throw
    };
    let Some(claim) = rj(ctx, &claims_dir(root).join(format!("{id}.json")))? else {
        return Ok(None);
    };
    match claim {
        Value::Object(m) => Ok(Some(m)),
        _ => Ok(None), // `typeof claim !== 'object'` / null
    }
}

/// claims.mjs isClaimExpired/isClaimActive.
pub(crate) fn is_claim_active(claim: &JMap, now_ms_v: f64) -> bool {
    let ttl = claim.get("ttl_seconds").and_then(|v| v.as_f64());
    let Some(ttl) = ttl else { return true };
    if !ttl.is_finite() || ttl <= 0.0 {
        return true;
    }
    let claimed = date_parse_val(claim.get("claimed_at"));
    if !claimed.is_finite() {
        return true;
    }
    claimed + ttl * 1000.0 > now_ms_v
}

/// claims.mjs activeWorkers — live-heartbeat sessions joined with their
/// first active claim, one row per session.
pub(crate) fn active_workers(ctx: &Ctx, root: &Path, exclude_session_id: Option<&str>) -> R<Vec<JMap>> {
    let exclude = exclude_session_id.map(js_trim).unwrap_or("");
    let now = now_ms();
    let live: Vec<JMap> = list_session_records(ctx, root)?
        .into_iter()
        .filter(|s| !str_eq(s.get("id"), exclude) && !heartbeat_stale(s, now))
        .collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let mut claim_cell_by_session: HashMap<String, Value> = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(claims_dir(root)) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(".json") else { continue };
            let claim = match read_claim(ctx, root, stem) {
                Ok(c) => c,
                Err(Ex::Thrown) => continue, // "not a plain cell id" filename
                Err(e) => return Err(e),
            };
            let Some(claim) = claim else { continue };
            let session = claim.get("session");
            if !opt_truthy(session) || !is_claim_active(&claim, now) {
                continue;
            }
            // JS Map keyed by claim.session — only string keys are ever
            // retrievable against a session id.
            if let Some(Value::String(s)) = session {
                claim_cell_by_session
                    .entry(s.clone())
                    .or_insert_with(|| claim.get("cell").cloned().unwrap_or(Value::Null));
            }
        }
    }
    let mut rows = Vec::new();
    for session in live {
        let mut row = JMap::new();
        // { session_id: session.id } — undefined would be dropped by
        // JSON.stringify; readSession guarantees a string id.
        row.insert("session_id".into(), session.get("id").cloned().unwrap_or(Value::Null));
        let lane = match session.get("lane") {
            Some(Value::String(s)) if !s.is_empty() => json!(s),
            _ => Value::Null,
        };
        row.insert("lane".into(), lane);
        let sid = session.get("id").and_then(|v| v.as_str()).unwrap_or("");
        row.insert(
            "cell".into(),
            claim_cell_by_session.get(sid).cloned().unwrap_or(Value::Null),
        );
        match session.get("last_heartbeat") {
            Some(v) => {
                row.insert("last_heartbeat".into(), v.clone());
            }
            None => {} // undefined -> key dropped by JSON.stringify
        }
        rows.push(row);
    }
    Ok(rows)
}

// ─── reservations over the lease store (reservations.mjs / lease-store.mjs) ─

/// lease-store.mjs listAllLeaseFiles + listLeases (silent per-file skip),
/// re-rooted through reservations.mjs's cycle-safe controlRootFor.
pub(crate) fn list_path_lease_records(ctx: &Ctx) -> Vec<Value> {
    let control = reservations_control_root(ctx);
    let leases_root = control.join(".bee").join("runtime").join("leases");
    let mut records = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = leases_root.join(sub);
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            // readLeaseSafe: silent null on unreadable/corrupt.
            let Some(text) = read_text_opt(&entry.path()) else { continue };
            let Ok(record) = serde_json::from_str::<Value>(&text) else { continue };
            records.push(record);
        }
    }
    // isPathLease filter.
    records
        .into_iter()
        .filter(|r| {
            truthy(r)
                && matches!(vget(r, "resource"), Some(Value::String(s)) if s.starts_with("path:"))
        })
        .collect()
}

/// reservations.mjs isLeaseRecordExpired.
pub(crate) fn is_lease_record_expired(record: &Value, now_ms_v: f64) -> bool {
    let expires = vget(record, "expires_at");
    if nullish(expires) {
        return false;
    }
    let ms = date_parse_val(expires);
    if !ms.is_finite() {
        return false;
    }
    ms <= now_ms_v
}

pub(crate) const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

/// reservations.mjs leaseToReservation — keys inserted only when the JS value
/// is defined (JSON.stringify drops undefined-valued keys).
pub(crate) fn lease_to_reservation(record: &Value) -> JMap {
    let mut out = JMap::new();
    // agent: leaseAgent(record)
    let workspace_id = vget(record, "workspace_id");
    let agent: Option<Value> = match workspace_id {
        Some(Value::String(s)) if s.starts_with("agent:") => Some(json!(&s["agent:".len()..])),
        Some(v) => Some(v.clone()),
        None => None,
    };
    if let Some(a) = agent {
        out.insert("agent".into(), a);
    }
    if let Some(cell) = vget(record, "workflow_id") {
        out.insert("cell".into(), cell.clone());
    }
    if let Some(Value::String(resource)) = vget(record, "resource") {
        out.insert("path".into(), json!(&resource["path:".len()..]));
    }
    // ttl: Math.max(0, Math.round((parse(expires)-parse(acquired))/1000)) or 0.
    let ttl = if nullish(vget(record, "expires_at")) {
        json!(0)
    } else {
        let diff = (date_parse_val(vget(record, "expires_at"))
            - date_parse_val(vget(record, "acquired_at")))
            / 1000.0;
        let rounded = js_round(diff);
        if rounded.is_nan() {
            Value::Null // Math.max(0, NaN) = NaN -> JSON null
        } else {
            json_num(rounded.max(0.0))
        }
    };
    out.insert("ttl_seconds".into(), ttl);
    if let Some(v) = vget(record, "acquired_at") {
        out.insert("reserved_at".into(), v.clone());
    }
    out.insert("released_at".into(), Value::Null);
    if let Some(session) = vget(record, "session_id") {
        if truthy(session) && !str_eq(Some(session), SESSIONLESS_SESSION_ID) {
            out.insert("session".into(), session.clone());
        }
    }
    let kind = vget(record, "kind").filter(|v| truthy(v)).cloned().unwrap_or(json!("lease"));
    out.insert("kind".into(), kind);
    out
}

/// reservations.mjs listReservations.
pub(crate) fn list_reservations(ctx: &Ctx, active_only: bool) -> Vec<JMap> {
    let now = now_ms();
    list_path_lease_records(ctx)
        .into_iter()
        .filter(|r| !active_only || !is_lease_record_expired(r, now))
        .map(|r| lease_to_reservation(&r))
        .collect()
}

/// The retirement backlog: features whose ACTIVE cells are all terminal and
/// which are therefore sitting in the hot scan path for no reason.
///
/// WHY STATUS CARRIES IT. `list_cells` above parses every file in
/// `.bee/cells/` on every `status` and `orient`, so the cost of asking "where
/// am I" grows with the amount of work already finished. `bee close` retires
/// the feature it closes, but nothing retires a feature that was simply
/// abandoned mid-lifecycle or finished before close existed — on this repo
/// that had reached 116 features and 441 files, two thirds of a 300 ms
/// orient. A read-only verb must not fix that itself; it names it, and
/// `bee cells archive --all-but-active` does the work when the reader says so.
///
/// Computed from the cell list the caller ALREADY loaded, so it costs no I/O.
pub(crate) fn archivable_backlog(cells: &[Value], active: Option<&Value>) -> JMap {
    let mut terminal_only: std::collections::BTreeMap<String, (bool, i64)> =
        std::collections::BTreeMap::new();
    for cell in cells {
        let Some(Value::String(feature)) = vget(cell, "feature") else { continue };
        if feature.is_empty() {
            continue;
        }
        let is_terminal = matches!(
            vget(cell, "status").and_then(|v| v.as_str()),
            Some("capped") | Some("dropped")
        );
        let entry = terminal_only.entry(feature.clone()).or_insert((true, 0));
        entry.0 &= is_terminal;
        entry.1 += 1;
    }
    // The active feature is never archivable — its cells are where the work
    // is — so counting it here would make the nudge unclearable.
    let active_name = match active {
        Some(Value::String(f)) if !f.is_empty() => Some(f.as_str()),
        _ => None,
    };
    let mut features: Vec<&String> = terminal_only
        .iter()
        .filter(|(f, (all_terminal, _))| *all_terminal && Some(f.as_str()) != active_name)
        .map(|(f, _)| f)
        .collect();
    features.sort();
    let cell_count: i64 = features
        .iter()
        .map(|f| terminal_only.get(*f).map(|(_, n)| *n).unwrap_or(0))
        .sum();
    let mut out = JMap::new();
    out.insert("features".into(), json!(features.len()));
    out.insert("cells".into(), json!(cell_count));
    out.insert(
        "ids".into(),
        Value::Array(features.iter().take(5).map(|f| json!(f)).collect()),
    );
    out
}
