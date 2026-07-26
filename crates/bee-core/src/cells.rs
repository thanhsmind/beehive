//! cells — read-only listing over `.bee/cells/*.json`, ported from
//! `.bee/bin/lib/cells.mjs`'s `cellsDir`/listing semantics (rust-port-8,
//! CONTEXT.md D3). Read-only, zero subprocess (D5): never claims, caps, or
//! writes a cell file.
//!
//! `.bee/bin/lib/cells.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This module lists the id/status/worker/files shape the
//! guard-support panel named — every other field (title, action,
//! must_haves, verify, trace, tier, ...) survives round-trip via `extra`
//! (D3), so a consumer that needs a field this reader doesn't name
//! explicitly can still reach it without a second reader.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::jsdate::parse_iso_ms;

/// `cellsDir(root)` — mirrors cells.mjs exactly.
pub fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

/// `ARCHIVE_DIR_NAME` — the one reserved child of `cellsDir` a default
/// listing must skip (cells.mjs's own comment: a directory entry already
/// fails the `.json` filter, but the guard stays explicit).
pub const ARCHIVE_DIR_NAME: &str = "archive";

/// One `.bee/cells/<id>.json` record. `id`, `status`, and `files` are
/// named explicitly (the guard-support panel's "id/status/worker/files");
/// `worker` lives nested under `trace.worker` in the real mjs shape, so it
/// is read via [`Cell::worker`] against `extra` rather than a top-level
/// field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Cell {
    /// `trace.worker` — the agent nickname currently holding this cell's
    /// claim, or `None` if unclaimed/absent.
    pub fn worker(&self) -> Option<&str> {
        self.extra.get("trace")?.get("worker")?.as_str()
    }

    pub fn lane(&self) -> Option<&str> {
        self.extra.get("lane")?.as_str()
    }

    pub fn feature(&self) -> Option<&str> {
        self.extra.get("feature")?.as_str()
    }

    /// `cell.tier` (rust-port-13, cells.mjs `MODEL_TIERS`/`tierMix`).
    pub fn tier(&self) -> Option<&str> {
        self.extra.get("tier")?.as_str()
    }

    /// `cell.deps` — dependency cell ids, matching cells.mjs's use of
    /// `cell.deps || []` in `depsAllCapped`.
    pub fn deps(&self) -> Vec<String> {
        self.extra
            .get("deps")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    }

    /// `trace.behavior_change` — matches `(cell.trace || {}).behavior_change === true`.
    pub fn behavior_change(&self) -> bool {
        self.extra
            .get("trace")
            .and_then(|t| t.get("behavior_change"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// `trace.capped_at` (ISO timestamp string), for scribing-debt age checks.
    pub fn capped_at(&self) -> Option<&str> {
        self.extra.get("trace")?.get("capped_at")?.as_str()
    }
}

/// Lists every readable `.bee/cells/*.json` record (skipping the `archive`
/// subdirectory and any non-`.json` entry), matching cells.mjs's default
/// listing scope. Fail-open per entry: a corrupt/unreadable cell file is
/// skipped rather than failing the whole listing — same posture as this
/// crate's `fsutil::read_jsonl` per-line tolerance, applied per-file here
/// since each cell is its own file rather than a jsonl line.
pub fn list_cells(root: &Path) -> Vec<Cell> {
    let dir = cells_dir(root);
    let attempt = fs::read_dir(&dir);
    // rust-port-22: lowest-shared-primitive counter (see
    // `crate::read_accounting`'s module doc comment) — this is the SINGLE
    // funnel every "cells directory scanned" call site in today's code
    // goes through (`ready_cells`, `tier_mix`, `ceiling_scarcity_warning`,
    // `scribing_debt`, `global_scribing_debt` via `list_cells_where`, and
    // `recovery::detect_crash_candidates`'s own direct call). Counted
    // whether the scan succeeds or not — a failed `read_dir` is still a
    // real syscall.
    crate::read_accounting::record_cells_dir_scan();
    let entries = match attempt {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut cells = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue; // skips `archive/` (and any other subdirectory)
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) == Some(ARCHIVE_DIR_NAME) {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if let Ok(cell) = serde_json::from_str::<Cell>(&text) {
            cells.push(cell);
        }
    }
    cells
}


// ─── status readers A (rust-port-13, CONTEXT.md D3/D5) ─────────────────────
// archivedTotals, readyCells, scribingDebt, globalScribingDebt, tierMix,
// ceilingScarcityWarning — ported from cells.mjs's own copies. Every reader
// here is read-only, zero subprocess: no claim/cap/archive write path is
// touched by this cell.

const ID_PATTERN_FIRST_OK: fn(char) -> bool = |c| c.is_ascii_alphanumeric();
const ID_PATTERN_REST_OK: fn(char) -> bool = |c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-';

/// cells.mjs `ID_PATTERN` (`/^[A-Za-z0-9][A-Za-z0-9._-]*$/`).
fn is_valid_cell_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if ID_PATTERN_FIRST_OK(c) => {}
        _ => return false,
    }
    chars.all(ID_PATTERN_REST_OK)
}

fn cell_file(root: &Path, id: &str) -> PathBuf {
    cells_dir(root).join(format!("{id}.json"))
}

/// `readCell(root, id)`: resolves the active cell first (byte-identical
/// fast path); only when the active file is genuinely absent/unreadable/
/// malformed does it fall back to searching `.bee/cells/archive/*/<id>.json`
/// — so dep resolution ([`ready_cells`]'s `depsAllCapped`) transparently
/// sees archived capped cells. An invalid id (fails [`is_valid_cell_id`])
/// short-circuits to `None`, matching the mjs guard.
pub fn read_cell(root: &Path, id: &str) -> Option<Cell> {
    if !is_valid_cell_id(id) {
        return None;
    }
    // rust-port-22: lowest-shared-primitive counter, SEPARATE from
    // `cells_dir_scans` (validation W6) — see `crate::read_accounting`'s
    // module doc comment for why the archive-fallback scan below is
    // folded into this same bucket rather than broken out further.
    crate::read_accounting::record_cell_dep_read();
    if let Ok(text) = fs::read_to_string(cell_file(root, id)) {
        if let Ok(cell) = serde_json::from_str::<Cell>(&text) {
            return Some(cell);
        }
        // Present-but-malformed active file: mjs's readJson falls open to
        // `null` here (warning to stderr), which readCell then treats
        // exactly like a genuinely missing active file — fall through to
        // the archive search below rather than returning `None` directly.
    }
    let archive_root = cells_dir(root).join(ARCHIVE_DIR_NAME);
    let entries = fs::read_dir(&archive_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let candidate = path.join(format!("{id}.json"));
        if let Ok(text) = fs::read_to_string(&candidate) {
            if let Ok(cell) = serde_json::from_str::<Cell>(&text) {
                return Some(cell);
            }
        }
    }
    None
}

/// `listCells(root, {feature, status})` (`includeArchived` always `false` —
/// every real caller of this cell's readers omits it): filters + id-sorts
/// [`list_cells`]'s default active-only scan. Sort is a plain byte-order
/// `String` compare, a deliberate narrowing of mjs's
/// `localeCompare(id, 'en', {numeric:true})` — exact for the ASCII,
/// consistent-case, consistent-digit-width ids every fixture in this cell's
/// oracle suite uses; a future consumer needing true natural/locale
/// ordering should extend this rather than assume it already matches.
pub fn list_cells_where(root: &Path, feature: Option<&str>, status: Option<&str>) -> Vec<Cell> {
    let mut cells = list_cells(root);
    if let Some(f) = feature {
        cells.retain(|c| c.feature() == Some(f));
    }
    if let Some(s) = status {
        cells.retain(|c| c.status.as_deref() == Some(s));
    }
    cells.sort_by(|a, b| a.id.cmp(&b.id));
    cells
}

fn deps_all_capped(root: &Path, cell: &Cell) -> Vec<String> {
    cell.deps()
        .into_iter()
        .filter(|dep| !matches!(read_cell(root, dep), Some(dep_cell) if dep_cell.status.as_deref() == Some("capped")))
        .collect()
}

/// `readyCells(root, feature)`: open cells (in `feature`, when given) whose
/// every dep is capped.
pub fn ready_cells(root: &Path, feature: Option<&str>) -> Vec<Cell> {
    list_cells_where(root, feature, Some("open"))
        .into_iter()
        .filter(|cell| deps_all_capped(root, cell).is_empty())
        .collect()
}

fn archive_summary_file(root: &Path) -> PathBuf {
    cells_dir(root).join(ARCHIVE_DIR_NAME).join("summary.json")
}

/// `archivedSummary(root)`: the on-disk archive ledger
/// `{feature: {capped, dropped, archived_at}}`. Absent/malformed file reads
/// as `{}`.
pub fn archived_summary(root: &Path) -> Value {
    let raw: Value = crate::fsutil::read_json(&archive_summary_file(root), json!({}));
    if raw.is_object() {
        raw
    } else {
        json!({})
    }
}

/// `archivedTotals(root)`: `{capped, dropped, total}` summed across every
/// archived feature's summary entry. Tolerant of a partially-shaped entry
/// (missing/non-numeric `capped`/`dropped` reads as 0).
pub fn archived_totals(root: &Path) -> Value {
    let summary = archived_summary(root);
    let mut capped: i64 = 0;
    let mut dropped: i64 = 0;
    if let Some(obj) = summary.as_object() {
        for entry in obj.values() {
            if !entry.is_object() {
                continue;
            }
            capped += entry.get("capped").and_then(Value::as_i64).unwrap_or(0);
            dropped += entry.get("dropped").and_then(Value::as_i64).unwrap_or(0);
        }
    }
    json!({ "capped": capped, "dropped": dropped, "total": capped + dropped })
}

fn scribing_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("scribing-runs.jsonl")
}

/// `readScribingLedger(root)`.
pub fn read_scribing_ledger(root: &Path) -> Vec<Value> {
    crate::fsutil::read_jsonl(&scribing_ledger_path(root))
}

// `scribingRunStampMs(run)`: `Date.parse(run.at || run.date)`, `None` when
// unparseable/absent (matches `Number.isFinite` false).
fn scribing_run_stamp_ms(run: &Value) -> Option<i64> {
    if run.is_null() {
        return None;
    }
    let at = run.get("at").and_then(Value::as_str).filter(|s| !s.is_empty());
    let date = run.get("date").and_then(Value::as_str).filter(|s| !s.is_empty());
    parse_iso_ms(at.or(date)?)
}

fn max_stamp(best: Option<i64>, candidate: Option<i64>) -> Option<i64> {
    match (best, candidate) {
        (Some(b), Some(c)) => Some(b.max(c)),
        (Some(b), None) => Some(b),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    }
}

/// `bestScribingStampMs(root, feature, ctx)`: the ledger max, then the
/// feature's own lane stamp, then the default record's stamp (only when it
/// names this same feature) — `ledger`/`state` are caller-supplied so
/// [`global_scribing_debt`]'s per-feature sweep can reuse one read of each
/// across every feature instead of re-reading per call.
pub fn best_scribing_stamp_ms(root: &Path, feature: &str, ledger: &[Value], state: &crate::state::State) -> Option<i64> {
    let mut best: Option<i64> = None;
    for entry in ledger {
        if entry.get("feature").and_then(Value::as_str) != Some(feature) {
            continue;
        }
        best = max_stamp(best, entry.get("ts").and_then(Value::as_str).and_then(parse_iso_ms));
    }
    if let Some(lane) = crate::state::read_lane(root, feature) {
        if let Some(run) = lane.extra.get("last_scribing_run") {
            best = max_stamp(best, scribing_run_stamp_ms(run));
        }
    }
    if let Some(run) = state.extra.get("last_scribing_run") {
        if run.get("feature").and_then(Value::as_str) == Some(feature) {
            best = max_stamp(best, scribing_run_stamp_ms(run));
        }
    }
    best
}

/// `scribingDebt(root, {feature, sinceTs})`: capped `behavior_change` cells
/// in `feature` (defaults to the active `state.feature`) capped strictly
/// after the resolved threshold (`sinceTs` override, else the best known
/// scribing-sync stamp for that feature). `{count: 0, cells: []}` when no
/// feature is active/given.
pub fn scribing_debt(root: &Path, feature: Option<&str>, since_ts: Option<i64>) -> Value {
    let state = crate::state::read_state(root);
    let feature = feature.map(str::to_string).or_else(|| state.feature.clone());
    let Some(feature) = feature else {
        return json!({ "count": 0, "cells": Vec::<String>::new() });
    };
    let threshold = match since_ts {
        Some(ts) => ts,
        None => {
            let ledger = read_scribing_ledger(root);
            best_scribing_stamp_ms(root, &feature, &ledger, &state).unwrap_or(0)
        }
    };
    let ids: Vec<String> = list_cells_where(root, Some(&feature), Some("capped"))
        .into_iter()
        .filter(|cell| {
            cell.behavior_change() && cell.capped_at().and_then(parse_iso_ms).is_some_and(|ms| ms > threshold)
        })
        .map(|cell| cell.id)
        .collect();
    json!({ "count": ids.len(), "cells": ids })
}

/// `globalScribingDebt(root)`: every LIVE capped `behavior_change` cell,
/// across every feature at once, whose feature has no sync evidence at or
/// after its `capped_at` — `{count, features: [{feature, cells}]}`,
/// features sorted by name.
pub fn global_scribing_debt(root: &Path) -> Value {
    let cells: Vec<Cell> = list_cells_where(root, None, Some("capped"))
        .into_iter()
        .filter(Cell::behavior_change)
        .collect();
    if cells.is_empty() {
        return json!({ "count": 0, "features": Vec::<Value>::new() });
    }

    let state = crate::state::read_state(root);
    let ledger = read_scribing_ledger(root);
    let mut stamp_cache: HashMap<String, Option<i64>> = HashMap::new();

    // BTreeMap gives feature-name (byte-order) sorting for free, and
    // preserves each feature's own cell-id order as pushed — which is
    // already id-sorted, since `cells` came from `list_cells_where`.
    let mut by_feature: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for cell in &cells {
        let Some(feature) = cell.feature().map(str::to_string) else { continue };
        let capped_at_ms = cell.capped_at().and_then(parse_iso_ms);
        let stamp = *stamp_cache
            .entry(feature.clone())
            .or_insert_with(|| best_scribing_stamp_ms(root, &feature, &ledger, &state));
        let orphaned = match stamp {
            None => true,
            Some(s) => capped_at_ms.is_some_and(|c| c > s),
        };
        if !orphaned {
            continue;
        }
        by_feature.entry(feature).or_default().push(cell.id.clone());
    }

    let features: Vec<Value> = by_feature
        .into_iter()
        .map(|(feature, ids)| json!({ "feature": feature, "cells": ids }))
        .collect();
    let count: usize = features.iter().map(|f| f["cells"].as_array().map_or(0, Vec::len)).sum();
    json!({ "count": count, "features": features })
}

/// cells.mjs `MODEL_TIERS`.
pub const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];
/// cells.mjs `CEILING_MAX_SHARE` (decision 0012 / P7).
pub const CEILING_MAX_SHARE: f64 = 0.4;
const SCARCITY_MIN_TIERED: i64 = 3;

// JS has one numeric type — `0`, `0.5`, and `1` are all just "number", and
// `JSON.stringify` renders a whole-valued number WITHOUT a decimal point
// (`0`, not `0.0`). Rust's `f64` has no such unification: `serde_json`
// always renders an `f64` with a decimal point, even `0.0`/`1.0`. The
// oracle diff caught this directly on `tierMix`'s `ceilingShare` (`0` vs
// `0.0` on an empty/untiered store) — this helper renders a whole-valued
// `f64` as a JSON integer, matching `JSON.stringify`'s actual output byte
// for byte, and only a genuinely fractional value as a JSON float.
fn js_number(v: f64) -> Value {
    if !v.is_finite() {
        return Value::Null; // never produced by ceilingShare's bounded division; never a panic either way
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        json!(v as i64)
    } else {
        Value::Number(serde_json::Number::from_f64(v).expect("finite f64 checked above"))
    }
}

/// `tierMix(root, {feature})`: tier assignment across a feature's cells
/// (all statuses), or every cell when `feature` is `None`.
pub fn tier_mix(root: &Path, feature: Option<&str>) -> Value {
    let cells = list_cells_where(root, feature, None);
    let mut extraction = 0i64;
    let mut generation = 0i64;
    let mut ceiling = 0i64;
    let mut untiered = 0i64;
    for cell in &cells {
        match cell.tier() {
            Some("extraction") => extraction += 1,
            Some("generation") => generation += 1,
            Some("ceiling") => ceiling += 1,
            _ => untiered += 1,
        }
    }
    let tiered = extraction + generation + ceiling;
    let ceiling_share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    json!({
        "counts": { "extraction": extraction, "generation": generation, "ceiling": ceiling, "untiered": untiered },
        "tiered": tiered,
        "ceilingShare": js_number(ceiling_share),
    })
}

/// `ceilingScarcityWarning(root)`: `Some({pct, ceiling, tiered})` when the
/// active feature leans too much on the ceiling model, else `None`.
/// Advisory only — never a blocker.
pub fn ceiling_scarcity_warning(root: &Path) -> Option<Value> {
    let state = crate::state::read_state(root);
    let mix = tier_mix(root, state.feature.as_deref());
    let tiered = mix.get("tiered").and_then(Value::as_i64).unwrap_or(0);
    if tiered < SCARCITY_MIN_TIERED {
        return None;
    }
    let ceiling_share = mix.get("ceilingShare").and_then(Value::as_f64).unwrap_or(0.0);
    if ceiling_share <= CEILING_MAX_SHARE {
        return None;
    }
    let ceiling = mix.get("counts").and_then(|c| c.get("ceiling")).and_then(Value::as_i64).unwrap_or(0);
    Some(json!({
        "pct": (ceiling_share * 100.0).round() as i64,
        "ceiling": ceiling,
        "tiered": tiered,
    }))
}

// Tests live in crates/bee-core/tests/status_readers_a.rs (this cell's
// single integration target — cargo test -p bee-core --test
// status_readers_a) rather than here, so every reader's oracle-diff proof
// sits in one place per must-have.
//
// (rust-port-8's own guard_support.rs tests for `list_cells`/`Cell` above
// stay in that file, unchanged by this cell's additions.)
