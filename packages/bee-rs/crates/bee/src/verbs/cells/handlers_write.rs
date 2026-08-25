// verb handlers: add, update, claim
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

// ─── verb handlers ─────────────────────────────────────────────────────────

pub(crate) fn dispatch(
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
    f: impl FnOnce(&rsv::Ctx) -> MR<Out>,
) -> Option<ExitCode> {
    let ctx = match rsv::prelude(cmd, use_json, t0)? {
        rsv::Pre::Go(c) => c,
        rsv::Pre::Emitted(code) => return Some(code),
    };
    let out = f(&ctx);
    rsv::finish(&ctx, to_r2(out))
}

/// requireFlag(flags, name) — Missing/empty/boolean-true refuse with the
/// handler's own deterministic message (validate() never guards these
/// verbs' optional-at-schema flags).
pub(crate) fn require_flag_native(flags: &rsv::Flags, name: &str) -> MR<String> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Fail::Thrown(format!("Missing required flag --{name}."))),
    }
}

pub(crate) fn read_file_text(file: &str, label: &str) -> MR<String> {
    match std::fs::read(file) {
        Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => Err(Fail::Thrown(format!("Cannot read {label} file: {file}"))),
    }
}

pub(crate) fn read_stdin_text() -> MR<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    match std::io::stdin().lock().read_to_end(&mut bytes) {
        Ok(_) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) => Err(Fail::Thrown(format!("{e}"))),
    }
}

pub(crate) const RELEASE_MANIFEST_LINT_PATH: &str = "docs/history/codex-harness-hardening/release-manifest.json";

/// bee.mjs manifestLintWarning + emitManifestLintWarnings (stderr).
pub(crate) fn emit_manifest_lint_warnings(cells: &[Value]) {
    for cell in cells {
        let Value::Object(map) = cell else { continue };
        let verify = match map.get("verify") {
            Some(Value::String(s)) => s,
            _ => continue,
        };
        if !verify.contains("release_manifest") {
            continue;
        }
        let files = match map.get("files") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        if files.iter().any(|f| matches!(f, Value::String(s) if s == RELEASE_MANIFEST_LINT_PATH)) {
            continue;
        }
        let id = match map.get("id") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => "(unknown id)".to_string(),
        };
        eprintln!(
            "WARNING: cell \"{id}\" verify mentions release_manifest but files is missing \"{RELEASE_MANIFEST_LINT_PATH}\" — a cold worker will hit red verify with no sanctioned fix. FIX: add the manifest path to files; regenerate it only via \"bee dev release-manifest --write\"."
        );
    }
}

// ── cells add ──────────────────────────────────────────────────────────────

// ─── D3 (no cells before the gate, docs/history/hook-teeth CONTEXT.md) ────
//
// `cells add` refuses (whole batch, same as every other addCells problem)
// when the target CELL's feature is gated — phase "exploring" or "planning"
// — and that feature's OWN approved_gates.execution is not true.
//
// Lane-aware, mirroring plan_freeze_shape_approved's precedence (D1,
// hooks/write_guard/checks.rs): `.bee/lanes/<feature>.json` wins when it
// parses as an object naming this same feature; the default `.bee/
// state.json` is consulted ONLY when its own `feature` field names this
// same feature too — never borrowed for a feature it doesn't name (unlike
// claim's `default_gate_approved`, which floats the default pipeline's gate
// over every lane-less feature; D3 draws the line tighter). A mismatched,
// corrupt, or nowhere-found resolution is "no opinion" — an unknown feature
// (a greenfield add, no route or lane taken yet) is always allowed, never
// guessed at. A docs-lane record (`mode` "docs") is exempt regardless of
// phase or gate — docs-lane work never gates on execution.
pub(crate) fn gated_add_refusal(root: &Path, feature: &str) -> MR<Option<String>> {
    let Some(id) = lane_feature_ok(feature) else { return Ok(None) };
    let lane_file = lanes_dir(root).join(format!("{id}.json"));
    let resolved: Option<(Value, Value, bool)> = match read_json(&lane_file) {
        ReadJson::Parsed(Value::Object(m)) if matches!(m.get("feature"), Some(Value::String(f)) if *f == id) => {
            let phase = m.get("phase").cloned().unwrap_or_else(|| Value::String("idle".into()));
            let mode = m.get("mode").cloned().unwrap_or(Value::Null);
            let approved = matches!(
                m.get("approved_gates").and_then(|g| g.as_object()).and_then(|g| g.get("execution")),
                Some(Value::Bool(true))
            );
            Some((phase, mode, approved))
        }
        // Corrupt or present-but-mismatched (names a different feature): no
        // opinion, exactly like plan_freeze_shape_approved's own corrupt arm
        // — mutation doors refuse loudly elsewhere; a display-shaped read
        // like this one never guesses.
        ReadJson::Parsed(_) | ReadJson::Corrupt => None,
        ReadJson::Missing => {
            let state = bstate::read_state_brief(root);
            match &state.feature {
                Value::String(f) if *f == id => {
                    let approved = matches!(state.gates.get("execution"), Some(Value::Bool(true)));
                    Some((state.phase.clone(), state.mode.clone(), approved))
                }
                _ => None,
            }
        }
    };
    let Some((phase, mode, approved)) = resolved else { return Ok(None) };
    if matches!(&mode, Value::String(s) if s == "docs") {
        return Ok(None);
    }
    let gated = matches!(&phase, Value::String(s) if s == "exploring" || s == "planning");
    if !gated || approved {
        return Ok(None);
    }
    Ok(Some(format!(
        "addCells: feature \"{id}\" is gated (phase \"{}\") and its execution gate is not approved — D3: no cells before the gate. FIX: get the merged shape+execution gate approved (`bee state gate --merge --approved true`) for feature \"{id}\", then retry.",
        jsjson::js_to_string(&phase)
    )))
}

/// buildAddCellsReport row.
pub(crate) struct AddReportRow {
    pub(crate) id: String,
    pub(crate) ok: bool,
    pub(crate) problems: Vec<String>,
}

pub(crate) fn build_add_cells_report(root: &Path, cells: &[Value]) -> MR<(bool, Vec<AddReportRow>, Option<Vec<Value>>)> {
    let mut seen: Vec<String> = Vec::new();
    let mut rows: Vec<AddReportRow> = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        let id = match cell.get("id") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => format!("(index {index})"),
        };
        let mut problems: Vec<String> = validate_new_cell_problems(root, cell)?;
        // D3: gated-phase refusal folds into the SAME whole-batch problem
        // list every other addCells check uses — one gated cell fails the
        // whole batch, nothing written, exactly like a duplicate id or a
        // batch-wide cycle.
        if let Some(Value::String(feature)) = cell.get("feature") {
            if let Some(reason) = gated_add_refusal(root, feature)? {
                problems.push(reason);
            }
        }
        if let Some(Value::String(cid)) = cell.get("id") {
            if !cid.is_empty() {
                if seen.contains(cid) {
                    problems.push(format!("addCells: duplicate id \"{cid}\" within the batch."));
                } else {
                    seen.push(cid.clone());
                }
            }
        }
        rows.push(AddReportRow { id, ok: problems.is_empty(), problems });
    }
    let mut normalized: Option<Vec<Value>> = None;
    if rows.iter().all(|r| r.ok) {
        let mut list = Vec::new();
        for cell in cells {
            list.push(normalize_new_cell(cell)?);
        }
        let cycles = compute_incoming_cycles(root, &list)?;
        if !cycles.is_empty() {
            let cycle_ids: Vec<String> = cycles.iter().flatten().cloned().collect();
            let message = format_cycle_refusal("addCells", &cycles);
            for row in rows.iter_mut() {
                if cycle_ids.contains(&row.id) {
                    row.problems.push(message.clone());
                    row.ok = false;
                }
            }
        } else {
            normalized = Some(list);
        }
    }
    let ok = rows.iter().all(|r| r.ok);
    Ok((ok, rows, if ok { normalized } else { None }))
}

pub(crate) fn add_report_rows_value(rows: &[AddReportRow]) -> Value {
    Value::Array(
        rows.iter()
            .map(|r| {
                let mut m = Map::new();
                m.insert("id".into(), Value::String(r.id.clone()));
                m.insert("ok".into(), Value::Bool(r.ok));
                m.insert(
                    "problems".into(),
                    Value::Array(r.problems.iter().map(|p| Value::String(p.clone())).collect()),
                );
                Value::Object(m)
            })
            .collect(),
    )
}

pub(crate) fn run_add(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["file", "stdin", "dry-run"]) {
        return None;
    }
    let stdin = bool_flag(&flags, "stdin")?;
    // `flags['dry-run'] !== undefined` — a "false" string still triggers it.
    let dry_run = match flags.get("dry-run") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" || s == "false" => true,
        Some(FlagV::S(_)) => return None,
    };
    dispatch("cells add", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        delegate_only(read_commands_slice(&root))?;
        let text = if stdin {
            read_stdin_text()?
        } else {
            let file = require_flag_native(&flags, "file")?;
            read_file_text(&file, "cell")?
        };
        // Lone surrogates and |n| >= 1e21 used to fork here; both are gone
        // (see parse_json_js), so every unparseable payload — file or stdin —
        // takes the one refusal Node's `catch` threw.
        let payload = match parse_json_js(&text, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => return Err(Fail::Thrown("add: input is not valid JSON.".into())),
        };
        if dry_run {
            let batch: Vec<Value> = match &payload {
                Value::Array(a) => a.clone(),
                other => vec![other.clone()],
            };
            if batch.is_empty() {
                return Err(Fail::Thrown(
                    "previewAddCells: expected a non-empty JSON array of cells.".into(),
                ));
            }
            let (ok, rows, _) = build_add_cells_report(&root, &batch)?;
            let mut lines: Vec<String> = Vec::new();
            let failing = rows.iter().filter(|r| !r.ok).count();
            lines.push(if ok {
                format!("dry-run: {} cell(s) valid — nothing written.", batch.len())
            } else {
                format!(
                    "dry-run: {failing} of {} cell(s) failed validation — nothing written.",
                    batch.len()
                )
            });
            for r in &rows {
                lines.push(format!(
                    "{} {}{}",
                    if r.ok { "OK" } else { "FAIL" },
                    r.id,
                    if r.problems.is_empty() { String::new() } else { format!(": {}", r.problems.join("; ")) }
                ));
            }
            let mut result = Map::new();
            result.insert("dry_run".into(), Value::Bool(true));
            result.insert("ok".into(), Value::Bool(ok));
            result.insert("cells".into(), add_report_rows_value(&rows));
            return Ok(Out::Emit(Value::Object(result), lines.join("\n"), if ok { 0 } else { 1 }));
        }
        if let Value::Array(batch) = &payload {
            if batch.is_empty() {
                return Err(Fail::Thrown("addCells: expected a non-empty JSON array of cells.".into()));
            }
            let (ok, rows, normalized) = build_add_cells_report(&root, batch)?;
            if !ok {
                let failing: Vec<&AddReportRow> = rows.iter().filter(|r| !r.ok).collect();
                let named = failing
                    .iter()
                    .map(|r| format!("{} ({})", r.id, r.problems.join("; ")))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Fail::Thrown(format!(
                    "addCells: {} of {} cell(s) failed validation — {named}. Nothing written.",
                    failing.len(),
                    batch.len()
                )));
            }
            let normalized = normalized.expect("ok report carries normalized cells");
            for cell in &normalized {
                write_cell(&root, cell)?;
            }
            emit_manifest_lint_warnings(&normalized);
            let text = normalized
                .iter()
                .map(|c| format!("Added {}", summarize_cell(c)))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok(Out::Emit(Value::Array(normalized), text, 0));
        }
        validate_new_cell(&root, &payload)?;
        // D3: the single-cell shape is the same "cells add" door as the
        // batch array — one cell IS a batch of one, so it takes the same
        // gated-phase refusal.
        if let Some(Value::String(feature)) = payload.get("feature") {
            if let Some(reason) = gated_add_refusal(&root, feature)? {
                return Err(Fail::Thrown(reason));
            }
        }
        let normalized = normalize_new_cell(&payload)?;
        assert_no_cycle(&root, "addCell", std::slice::from_ref(&normalized))?;
        write_cell(&root, &normalized)?;
        emit_manifest_lint_warnings(std::slice::from_ref(&normalized));
        let text = format!("Added {}", summarize_cell(&normalized));
        Ok(Out::Emit(normalized, text, 0))
    })
}

// ── cells update ───────────────────────────────────────────────────────────

pub(crate) const UPDATE_FIELDS: [&str; 15] = [
    "title",
    "action",
    "verify",
    "files",
    "read_first",
    "deps",
    "decisions",
    "must_haves",
    "behavior_change",
    "lane",
    "pbi",
    "change_class",
    REGEN_ACK_FIELD,
    "affects_skills",
    "affects_specs",
];

pub(crate) fn update_field_problem(key: &str, value: &Value) -> Option<String> {
    let bad = |msg: &str| Some(msg.to_string());
    match key {
        "title" | "action" | "verify" => {
            if nonblank_string(Some(value)) {
                None
            } else {
                bad("must be a non-empty string")
            }
        }
        "files" | "read_first" | "deps" | "decisions" | "affects_skills" | "affects_specs" => {
            if is_string_array(value) {
                None
            } else {
                bad("must be an array of strings")
            }
        }
        "must_haves" => {
            if matches!(value, Value::Object(_)) {
                None
            } else {
                bad("must be a JSON object")
            }
        }
        "behavior_change" => {
            if matches!(value, Value::Bool(_)) {
                None
            } else {
                bad("must be a boolean")
            }
        }
        "lane" => {
            if matches!(value, Value::String(s) if LANES.contains(&s.as_str())) {
                None
            } else {
                Some(format!("must be one of: {}", LANES.join(", ")))
            }
        }
        "pbi" => {
            if matches!(value, Value::Null | Value::String(_)) {
                None
            } else {
                bad("must be a string or null")
            }
        }
        "change_class" => {
            if matches!(value, Value::Null)
                || matches!(value, Value::String(s) if CHANGE_CLASSES.contains(&s.as_str()))
            {
                None
            } else {
                Some(format!("must be null or one of: {}", CHANGE_CLASSES.join(", ")))
            }
        }
        _ if key == REGEN_ACK_FIELD => {
            if matches!(value, Value::Null) || nonblank_string(Some(value)) {
                None
            } else {
                bad("must be null or a non-empty string (the one-line reason for skipping the derived regen obligation)")
            }
        }
        _ => unreachable!("caller checks membership"),
    }
}

pub(crate) fn update_frozen_hint(key: &str) -> Option<&'static str> {
    match key {
        "id" => Some("a cell id is permanent — add a new cell instead"),
        "feature" => Some("a cell never moves between features — drop and re-add instead"),
        "status" => Some("status moves only through claim/verify/cap/block/drop"),
        "trace" => Some("the trace is the frozen audit record — claim/verify/cap own it"),
        // D4 (store `97ce5225`): `tier` is retired as the model selector, so
        // this hint no longer names a verb to set it — there is none. It
        // stays a FROZEN key rather than an unknown one because stored
        // records still carry the field and a patch naming it deserves the
        // sentence that explains what replaced it.
        "tier" => Some(
            "tier is retired as the model selector — a cell's \"role\" is the job that picks its model, and escalation is the \"escalate\" flag (bee cells escalate --id ID)",
        ),
        _ => None,
    }
}

pub(crate) fn run_update(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "file", "stdin"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string(); // schema-required: missing -> validate() -> Node
    let stdin = bool_flag(&flags, "stdin")?;
    dispatch("cells update", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        delegate_only(read_commands_slice(&root))?;
        let text = if stdin {
            read_stdin_text()?
        } else {
            let file = require_flag_native(&flags, "file")?;
            read_file_text(&file, "patch")?
        };
        let patch = match parse_json_js(&text, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => {
                return Err(Fail::Thrown("update: patch input is not valid JSON.".into()))
            }
        };
        // updateCell — pure validation before the lock.
        if id.is_empty() || !id_pattern_ok(&id) {
            return Err(Fail::Thrown(format!("updateCell: invalid id \"{id}\".")));
        }
        let patch_map = match &patch {
            Value::Object(m) => m.clone(),
            _ => return Err(Fail::Thrown("updateCell: patch must be a JSON object.".into())),
        };
        if patch_map.is_empty() {
            return Err(Fail::Thrown("updateCell: patch is empty — nothing to update.".into()));
        }
        for (key, value) in &patch_map {
            if !UPDATE_FIELDS.contains(&key.as_str()) {
                let message = match update_frozen_hint(key) {
                    Some(hint) => format!(
                        "updateCell: field \"{key}\" is frozen — {hint}. The whole patch is refused; the cell is untouched."
                    ),
                    None => format!(
                        "updateCell: unknown field \"{key}\" — updatable fields: {}. The whole patch is refused; the cell is untouched.",
                        UPDATE_FIELDS.join(", ")
                    ),
                };
                return Err(Fail::Thrown(message));
            }
            if let Some(problem) = update_field_problem(key, value) {
                return Err(Fail::Thrown(format!(
                    "updateCell: field \"{key}\" {problem}. The whole patch is refused; the cell is untouched."
                )));
            }
        }
        // Same format door `cells add` runs (validate.rs): a backfill can no
        // more smuggle in a bare skill name than an add can. Every bad entry
        // is named in this one refusal; the patch is refused whole.
        if let Some(value) = patch_map.get("affects_skills") {
            let problems = affects_skills_path_problems(&root, "updateCell", value);
            if !problems.is_empty() {
                return Err(Fail::Thrown(format!(
                    "{} The whole patch is refused; the cell is untouched.",
                    problems.join(" ")
                )));
            }
        }
        if let Some(verify) = patch_map.get("verify") {
            assert_verify_sentinel_allowed(&root, "updateCell", verify)?;
        }

        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let outcome = (|| -> MR<Value> {
            assert_not_archived(&root, "updateCell", &id)?;
            // readCellStrictForUpdate — raw read, no BOM strip.
            let file = cell_file(&root, &id);
            let raw = match std::fs::read(&file) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Fail::Thrown(format!("updateCell: cell \"{id}\" not found.")))
                }
                // readCellStrictForUpdate's unreadable branch (lib/cells.mjs
                // :1474). Node interpolated err.code; this carries the Rust
                // io error in the same sentence, same refusal.
                Err(e) => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: could not read \"{}\" ({e}) — refusing to touch it. FIX: inspect/restore the file, then retry.",
                        file.display()
                    )))
                }
            };
            let sep = std::path::MAIN_SEPARATOR;
            let rel = format!(".bee{sep}cells{sep}{id}.json");
            let cell_map = match parse_json_js(&raw, false) {
                JsParse::Value(Value::Object(m)) => m,
                JsParse::Value(_) => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: \"{}\" exists but is not a JSON object — refusing to merge a patch over a corrupt cell.",
                        file.display()
                    )))
                }
                // Lone surrogates land here too — a cell file this CLI cannot
                // parse is corrupt, and the refusal is the same either way.
                JsParse::NotJson => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: \"{}\" exists but is not valid JSON — refusing to merge a patch over a corrupt cell. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.",
                        file.display()
                    )))
                }
            };
            let status_ok = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "open" || s == "blocked");
            if !status_ok {
                return Err(Fail::Thrown(format!(
                    "updateCell: cell \"{id}\" has status \"{}\" — only open or blocked cells are updatable (claimed = a live worker owns it; capped/dropped = frozen audit). The cell is untouched.",
                    js_string_or_undefined(cell_map.get("status"))
                )));
            }
            let mut merged = cell_map.clone();
            spread_into(&mut merged, &patch_map);
            // B-P2-8: this SAME call setting change_class to "behavior" arms
            // trace.behavior_change=true, unless the same patch already
            // declares its own (flat) "behavior_change" — an explicit false
            // is a deliberate opt-out and is respected as-is via the shared
            // door (arms_behavior_door, validate.rs) addCell's own
            // normalize_new_cell calls too. A patch that changes
            // change_class AWAY from "behavior" (or never touches it)
            // changes nothing: the door only ever arms, never disarms.
            if arms_behavior_door(patch_map.get("change_class"), patch_map.contains_key("behavior_change")) {
                let mut trace = match merged.get("trace") {
                    Some(Value::Object(t)) => t.clone(),
                    _ => Map::new(),
                };
                trace.insert("behavior_change".into(), Value::Bool(true));
                merged.insert("trace".into(), Value::Object(trace));
            }
            let merged_lane = match merged.get("lane") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if merged_lane == "standard" || merged_lane == "high-risk" {
                let truths = merged
                    .get("must_haves")
                    .filter(|m| js_truthy(m))
                    .and_then(|m| m.get("truths"));
                if !matches!(truths, Some(Value::Array(a)) if !a.is_empty()) {
                    return Err(Fail::Thrown(format!(
                        "updateCell: lane \"{merged_lane}\" requires non-empty must_haves.truths — the patch would leave \"{id}\" without them. The cell is untouched."
                    )));
                }
            }
            let merged_value = Value::Object(merged.clone());
            if patch_map.contains_key("deps") {
                assert_no_cycle(&root, "updateCell", std::slice::from_ref(&merged_value))?;
            }
            assert_regen_obligation(&merged, "updateCell")?;
            write_cell(&root, &merged_value)?;
            Ok(merged_value)
        })();
        guard.release();
        let updated = outcome?;
        emit_manifest_lint_warnings(std::slice::from_ref(&updated));
        let keys: Vec<String> = patch_map.keys().cloned().collect();
        let text = format!(
            "Updated {} ({}).",
            js_string_or_undefined(updated.get("id")),
            keys.join(", ")
        );
        Ok(Out::Emit(updated, text, 0))
    })
}

// ── cells claim ────────────────────────────────────────────────────────────

/// D1 (claim-time-worktree-redirect, plan.md C1) — the claim-time
/// execution-location annotation shared by `cells claim` and `cells
/// claim-next`. When the claimed cell's `feature` holds a granted
/// worktree, appends one line to the success `text` and inserts
/// `worktree_root` into the success JSON `obj`; otherwise leaves both
/// untouched (fail open, D2/D6 — an `Unresolvable` grant entry, exactly
/// like `NotFound`, must never become a confident annotation any more than
/// it becomes a confident refusal). Threads an explicit `main_root` —
/// never reads the acting cwd — so it is testable without
/// `std::env::set_current_dir`.
pub(crate) fn append_worktree_execution_annotation(
    main_root: &Path,
    feature: Option<&str>,
    obj: &mut Map<String, Value>,
    text: &mut String,
) {
    let Some(feature) = feature else { return };
    let Some(main_root_s) = main_root.to_str() else { return };
    let grant = match crate::hooks::write_guard::find_feature_worktree_grant(main_root_s, feature) {
        Ok(g) => g,
        Err(_) => return, // Nd — unproven input never becomes a confident annotation
    };
    let crate::hooks::write_guard::FeatureWorktreeGrant::Found(_id, worktree_root) = grant else {
        return; // NotFound / Unresolvable — fail open
    };
    text.push_str(&format!(
        "\nworktree: {worktree_root} — execution runs from a session rooted there; a subagent \
dispatched from main inherits main's cwd and cannot write here."
    ));
    obj.insert("worktree_root".into(), Value::String(worktree_root));
}

// ── crf-1: the claim-time reservation ACQUIRE half ─────────────────────────
//
// docs/history/claim-reserves-files/CONTEXT.md: `finish_cap_and_release`
// (handlers_close.rs) has always RELEASED by `(trace.worker, cell.id)` — the
// values `cells claim --worker` records — but nothing ever ACQUIRED under that
// key, so the release half ran against reservations that were never taken and
// every dispatched worker wrote unreserved. This is the missing half of an
// existing designed symmetry: `cells claim` and `cells claim-next` now reserve
// the claimed cell's declared `files` under the SAME `(agent, cell)` key the
// cap releases by.
//
// Both doors run the SHARED reserve section — `reservations::reserve_prechecks`
// then `reservations::reserve_exec`, the exact pair `reservations reserve` and
// `reserve_path_atomic` (the `dispatch prepare --claim` door) themselves call.
// Never a second reservation path.
//
// Why not `reserve_path_atomic` itself: it hardcodes `session: None`, because
// its one caller passes no session. A claim MUST thread the session it just
// claimed under — `resolve_session_id` reads flag → BEE_SESSION_ID →
// CLAUDE_CODE_SESSION_ID → single-live-session adoption, so dropping the
// `--session-id` flag would make every claim in concurrent mode refuse with
// reserve's own SESSION_REQUIRED. One layer down is the same door; a second
// implementation would not be.

/// `reserve_claimed_files`' typed verdict.
pub(crate) enum ClaimReserve {
    /// Every declared path is held under `(worker, cell)` — including any path
    /// this same worker already held for this same cell, which was already
    /// ours. Carries no payload on purpose: the claim's emitted bytes are
    /// unchanged by this cell, so the ONE readable answer is the reservation
    /// store itself (`reservations list`), never a second copy travelling here.
    Held,
    /// A genuine conflict. The claim and every reservation THIS call created
    /// are already rolled back; each caller prefixes the code with its own verb
    /// word, exactly like `CrossClaim::Refused`.
    Refused { code: &'static str, reason: String },
}

fn err2_to_fail(e: Err2) -> Fail {
    match e {
        Err2::Ex => Fail::Delegate,
        Err2::Msg(m) => Fail::Thrown(m),
    }
}

/// Is EVERY conflicting lease already ours, for this very cell? `find_conflicts`
/// filters same-agent rows out on the pre-check side, so this can only be the
/// O_EXCL exact-path arm meeting a lease we took under an earlier claim of the
/// same cell whose claim file has since expired or been swept. Re-claiming must
/// not error on that. A same-agent lease for a DIFFERENT cell is NOT ours here
/// — that other cell's cap is what releases it — so it stays a real conflict.
fn conflicts_are_our_own(conflicts: &[Value], worker: &str, cell_id: &str) -> bool {
    !conflicts.is_empty()
        && conflicts.iter().all(|c| {
            matches!(c.get("agent"), Some(Value::String(a)) if a == worker)
                && matches!(c.get("cell"), Some(Value::String(x)) if x == cell_id)
        })
}

/// The hold topology the acquire runs under. `cells claim` resolves its store
/// root through the NARROW door (`resolve_store_root`), which carries no
/// topology; this reads the same `StoreRoots::hold_topology()` that
/// `finish_topology` hands the RELEASE half, so acquire and release mirror one
/// another across checkouts. Anything unresolvable answers `None` — the same
/// "skip the cross-worktree wiring entirely" arm an ungranted linked worktree
/// takes — never a guessed topology.
pub(crate) fn claim_hold_topology() -> Option<(PathBuf, String)> {
    let cwd = std::env::current_dir().ok()?;
    match crate::roots::resolve_store_root_worktree(&cwd) {
        crate::roots::RootsWt::Go(roots) => roots.hold_topology(),
        _ => None,
    }
}

/// Reserve the just-claimed cell's declared `files`, in declaration order,
/// stopping at the first conflict and unwinding what this call created IN
/// REVERSE (reservations first, then the claim) — the same post-creation
/// rollback discipline `worktree new` and `claim_and_reserve_for_dispatch` use,
/// so a refusal can truthfully say the store is back in its pre-call state.
///
/// A cell declaring no files reserves nothing and refuses nothing: zero paths
/// is not an error, and the claim runs exactly as it did before this cell.
///
/// Two recorded residuals, both inherited from the shared doors:
///   - the rollback releases by `(worker, cell)`, the only scoping the shared
///     release door offers. When this call created at least one reservation AND
///     the worker also held an older lease for the SAME cell, that older lease
///     goes too. Nothing is released when this call created nothing.
///   - an unproven store shape (`Err2::Ex`) reaching here travels out as
///     `Fail::Delegate` with the claim already standing — the identical
///     accepted residual `claim_and_reserve_for_dispatch` records, and for the
///     same reason: every store this loop reads was already probed by the claim
///     door's own prescans and by `reserve_prechecks`.
pub(crate) fn reserve_claimed_files(
    root: &Path,
    topo: Option<(&Path, &str)>,
    cell: &Value,
    worker: &str,
    session_flag: Option<&str>,
) -> MR<ClaimReserve> {
    let claimed_id = match cell.get("id") {
        Some(Value::String(s)) => s.clone(),
        other => jsjson::js_to_string(other.unwrap_or(&Value::Null)),
    };
    // `Array.isArray(cell.files) ? cell.files.filter(f => typeof f === 'string' && f) : []`
    let files: Vec<String> = match cell.get("files") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    if files.is_empty() {
        return Ok(ClaimReserve::Held);
    }
    let root_s = root.to_str().ok_or(Fail::Delegate)?;

    let t = topo.map(|(m, h)| rsv::Topo { main_root: m, holder: h });

    let mut created: Vec<String> = Vec::new();
    for file_path in &files {
        let params = rsv::ReserveParams {
            agent: worker.to_string(),
            cell: claimed_id.clone(),
            path: file_path.clone(),
            ttl: None,     // reserve() defaults to DEFAULT_TTL_SECONDS
            session: session_flag.map(str::to_string),
            kind: None,    // 'lease' — hard conflicts, same as every other door
        };
        // Every delegate-trigger front-loaded, before the cross-worktree lock —
        // reserve_path_atomic's own order.
        rsv::reserve_prechecks(t, root_s, &params).map_err(|_| Fail::Delegate)?;
        let out = rsv::reserve_exec(t, root_s, &params, lock::MAX_ATTEMPTS).map_err(err2_to_fail)?;
        let refusal = match &out {
            // reserve()'s own argument refusals — structurally unreachable from
            // here (agent, cell and path are all non-empty by construction),
            // but never guessed at: reported, after the same rollback.
            Out::Thrown(m) => Some(vec![format!("- {m}")]),
            Out::Emit(Value::Object(m), _, _) => {
                if m.get("ok") == Some(&Value::Bool(true)) {
                    // The NORMALIZED path off the lease record, not files[i].
                    let p = match m.get("reservation").and_then(|r| r.get("path")) {
                        Some(Value::String(s)) => s.clone(),
                        other => jsjson::js_to_string(other.unwrap_or(&Value::Null)),
                    };
                    created.push(p);
                    None
                } else if matches!(m.get("code"), Some(Value::String(c)) if c == "FOREIGN_HOLD") {
                    let or_unknown = |k: &str| match m.get(k) {
                        Some(v) if rsv::truthy(v) => jsjson::js_to_string(v),
                        _ => "unknown".to_string(),
                    };
                    Some(vec![format!(
                        "- checkout \"{}\" holds \"{}\" (cross-worktree hold, feature {}, cell {})",
                        m.get("holder").map_or("undefined".into(), jsjson::js_to_string),
                        m.get("path").map_or("undefined".into(), jsjson::js_to_string),
                        or_unknown("feature"),
                        or_unknown("cell"),
                    )])
                } else if matches!(m.get("code"), Some(Value::String(c)) if c == "SESSION_REQUIRED") {
                    // reserve's own identity refusal, carried verbatim rather
                    // than flattened into an empty conflict list.
                    Some(vec![format!(
                        "- {}",
                        m.get("reason").map_or("undefined".into(), jsjson::js_to_string)
                    )])
                } else {
                    let conflicts = match m.get("conflicts") {
                        Some(Value::Array(a)) => a.clone(),
                        _ => Vec::new(),
                    };
                    if conflicts_are_our_own(&conflicts, worker, &claimed_id) {
                        None
                    } else {
                        Some(
                            conflicts
                                .iter()
                                .map(|c| {
                                    format!(
                                        "- {} holds \"{}\" (cell {})",
                                        c.get("agent").map_or("undefined".into(), jsjson::js_to_string),
                                        c.get("path").map_or("undefined".into(), jsjson::js_to_string),
                                        c.get("cell").map_or("undefined".into(), jsjson::js_to_string),
                                    )
                                })
                                .collect(),
                        )
                    }
                }
            }
            Out::Emit(..) => return Err(Fail::Delegate), // unreachable: always an object
        };
        let Some(conflict_lines) = refusal else { continue };

        // Unwind, in reverse: the reservations this call took, then the claim.
        let mut note = "the claim was rolled back and the store restored as found".to_string();
        let unwound = (|| -> MR<Result<(), String>> {
            if !created.is_empty() {
                if let Out::Thrown(m) =
                    rsv::release_reservations_for_agent(topo, root_s, worker, Some(&claimed_id))
                        .map_err(err2_to_fail)?
                {
                    return Ok(Err(m));
                }
            }
            match unclaim_cell(root, &claimed_id, session_flag, false) {
                Ok(_) => Ok(Ok(())),
                Err(Fail::Delegate) => Err(Fail::Delegate),
                Err(Fail::Thrown(m)) => Ok(Err(m)),
            }
        })()?;
        if let Err(message) = unwound {
            note = format!(
                "ROLLBACK FAILED ({message}) — restore by hand: bee reservations release --agent {worker} --cell {claimed_id} --json ; bee cells unclaim --id {claimed_id} --json"
            );
        }
        let mut lines = vec![format!(
            "cell \"{claimed_id}\" declares files that could not be reserved — nothing claimed; {note}:"
        )];
        lines.extend(conflict_lines);
        return Ok(ClaimReserve::Refused {
            code: "RESERVATION_CONFLICT",
            reason: lines.join("\n"),
        });
    }
    Ok(ClaimReserve::Held)
}

/// `cells claim`'s door with crf-1's acquire attached — the claim, then the
/// declared files reserved under the claiming `--worker`. Kept as ONE function
/// so `run_claim` (which resolves its root off `std::env::current_dir()`) and
/// the tests exercise the identical composition.
pub(crate) fn claim_cell_with_reservations(
    root: &Path,
    topo: Option<(&Path, &str)>,
    id: &str,
    worker: &str,
    session_flag: Option<&str>,
    ttl: Option<f64>,
    fix_first: Option<&str>,
) -> MR<ClaimDoor> {
    let door = claim_cell_from_flags_ex(root, id, worker, session_flag, ttl, fix_first)?;
    match reserve_claimed_files(root, topo, &door.cell, worker, door.session_id.as_deref())? {
        ClaimReserve::Held => Ok(door),
        ClaimReserve::Refused { code, reason } => {
            Err(Fail::Thrown(format!("claim: {code} — {reason}")))
        }
    }
}

pub(crate) fn run_claim(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "worker", "session-id", "ttl", "isolate", "fix-first"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let worker = flags.req_str("worker")?.to_string();
    let session_flag = opt_string_flag(&flags, "session-id")?;
    let _isolate = bool_flag(&flags, "isolate")?;
    // D2 (no claim on a red base): trimmed; empty/absent = None, same
    // convention as capCellFromFlags's override_reason.
    let fix_first: Option<String> = match opt_string_flag(&flags, "fix-first")? {
        Some(s) if !js_trim(&s).is_empty() => Some(js_trim(&s).to_string()),
        _ => None,
    };
    let ttl: Option<f64> = match flags.get("ttl") {
        None => None,
        Some(FlagV::Present) => return None,
        Some(FlagV::S(s)) => match rsv::js_number_flag(s) {
            Err(_) => return None, // validate() refuses the shape — Node's message
            Ok(parsed) => Some(parsed.unwrap_or(f64::NAN)),
        },
    };
    dispatch("cells claim", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        // crf-1: claim, then hold the declared files under the same
        // `(--worker, cell)` key the cap already releases by. A success emits
        // exactly the bytes it always did — the acquire adds nothing to the
        // payload; only a genuine conflict is visible, as a typed refusal.
        let topology = claim_hold_topology();
        let topo = topology.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
        let mut claimed = claim_cell_with_reservations(
            &root,
            topo,
            &id,
            &worker,
            session_flag.as_deref(),
            ttl,
            fix_first.as_deref(),
        )?
        .cell;
        let worker_disp = match claimed.get("trace").and_then(|t| t.get("worker")) {
            Some(v) => jsjson::js_to_string(v),
            None => "undefined".to_string(),
        };
        let mut text = format!(
            "Claimed {} for {}.",
            js_string_or_undefined(claimed.get("id")),
            worker_disp
        );
        let feature = match claimed.get("feature") {
            Some(Value::String(f)) => Some(f.clone()),
            _ => None,
        };
        if let Value::Object(map) = &mut claimed {
            append_worktree_execution_annotation(&root, feature.as_deref(), map, &mut text);
        }
        Ok(Out::Emit(claimed, text, 0))
    })
}

// ─── D2 (no claim on a red base, docs/history/hook-teeth CONTEXT.md) ──────
//
// `cells claim` (and every door that shares claim_cell_cross_session_ex)
// refuses when the LAST recorded declared-test run is red, unless the claim
// carries `--fix-first <reason>` — that reason lands on the winning claim's
// own trace (`trace.fix_first`) so a cold reader sees why a red base was
// claimed onto anyway, without re-deriving it from logs.
//
// The record read is `.bee/logs/test-results.json` under the CONTROL root
// (claims-store territory, msn-18b — the same root claims_dir/sessions_dir
// already resolve through), in the EXACT shape
// finish_support::run_declared_tests/tests_record_value writes:
// `{ran_at, green, commands:[{command, exit, duration_ms, failure_excerpt, failure_log}]}`.
// The named failing command is the first row carrying a non-null
// `failure_excerpt` — the same row run_declared_tests marks not-passed.
//
// A missing file (nothing has ever run the declared tests here) or one this
// reader cannot trust (not an object, `green` not a bool, `commands` not an
// array — a shape this schema's own writer would never produce) can prove
// neither red nor green: it warns to stderr and lets the claim proceed,
// exactly like D4's own "cannot know" arms elsewhere in this file. A GREEN
// record is untouched — no warning, no refusal.
pub(crate) enum RedBaseStatus {
    /// No record on file, or one this reader cannot trust as the schema
    /// finish_support writes — cannot know either way.
    Unknown,
    Green,
    Red { failing_command: String },
}

/// Pure classifier — no I/O side effects beyond the one read — so the D7
/// red/green/missing/unparseable classification can be pinned by a test
/// with no captured stderr.
pub(crate) fn classify_red_base(control: &Path) -> RedBaseStatus {
    let value = match read_json(&test_results_path(control)) {
        ReadJson::Missing => return RedBaseStatus::Unknown,
        ReadJson::Corrupt => return RedBaseStatus::Unknown,
        ReadJson::Parsed(v) => v,
    };
    let Value::Object(map) = &value else { return RedBaseStatus::Unknown };
    let green = match map.get("green") {
        Some(Value::Bool(b)) => *b,
        _ => return RedBaseStatus::Unknown,
    };
    if green {
        return RedBaseStatus::Green;
    }
    let failing_command = match map.get("commands") {
        Some(Value::Array(rows)) => rows.iter().find_map(|row| {
            let Value::Object(m) = row else { return None };
            let failed = !matches!(m.get("failure_excerpt"), None | Some(Value::Null));
            if !failed {
                return None;
            }
            match m.get("command") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            }
        }),
        _ => None,
    };
    RedBaseStatus::Red {
        failing_command: failing_command.unwrap_or_else(|| "(unknown command)".to_string()),
    }
}

/// The claim-time gate itself: `None` lets the claim proceed (green,
/// unknown, or escaped via `--fix-first`); `Some(reason)` is the typed
/// refusal text `claim_cell_cross_session_ex` wraps as `RED_BASE`. The
/// "cannot know" warning prints here, exactly once, at the point the door
/// actually needed the answer — never speculatively.
pub(crate) fn red_base_refusal(control: &Path, cell_id: &str, fix_first: Option<&str>) -> Option<String> {
    match classify_red_base(control) {
        RedBaseStatus::Green => None,
        RedBaseStatus::Unknown => {
            eprintln!(
                "WARNING: {TEST_RESULTS_RELATIVE} is missing or not a recognized test-results record — cannot know whether the base is green or red; claim proceeding."
            );
            None
        }
        RedBaseStatus::Red { failing_command } => {
            if fix_first.is_some() {
                return None;
            }
            Some(format!(
                // D7 (docs/history/test-doctrine/CONTEXT.md): `bee test` is
                // now the ONLY writer of `{TEST_RESULTS_RELATIVE}` — `bee
                // close`/`bee worktree merge` stopped running
                // `commands.test` — so the refresh path this refusal names
                // must be `bee test`, never a bare "fix the red".
                "cell \"{cell_id}\" refused — the last recorded test run is red (\"{failing_command}\" failed; record: {TEST_RESULTS_RELATIVE}). D2: never claim onto a red base. FIX: fix the red, run `bee test` to refresh the record, then retry — or pass --fix-first \"<reason>\" to claim anyway (the reason is stored on the claim's own trace.fix_first)."
            ))
        }
    }
}

// ─── D4 (route-record warn-to-deny escalation, docs/history/counter-teeth
// CONTEXT.md) ────────────────────────────────────────────────────────────
//
// D3 gave every claim on a no-route feature the same stderr warning,
// forever. D4 spends that warning once per (feature, session): the FIRST
// `cells claim` (or `dispatch prepare --claim`, the other door onto
// claim_cell_from_flags) a given session makes against a feature with no
// route record still only warns; that SAME session's second and later ones
// refuse outright, naming "bee state route --set" as the remedy (D5: safe
// to flip now that ct-1 ported the granted-worktree arm the remedy needs).
//
// Scoped per SESSION, not bare per feature: bee's own swarming model fans
// a feature's cells out to many concurrently-dispatched worker sessions
// (AGENTS.md "Work in parallel" — "never zero execution workers"), each
// claiming its own assigned cell before anyone has had a chance to route.
// A bare per-feature count would make the SECOND worker's first-ever claim
// refuse, breaking that fan-out on every routeless feature. Per-session
// scoping keeps the nag aimed at the one session doing REPEAT claims
// without ever routing, while every other session's own first claim still
// only warns — the (session, feature) tuple is the count's real subject.
//
// The count has to survive a claim being unclaimed and reclaimed — of the
// SAME cell, not just a different one — by the SAME session. Neither
// existing claim-adjacent field can carry that: release_trace() (trace.rs)
// nulls a cell's own trace.claimed_at back to null on every unclaim, and
// claim_cell_file (claims.rs) always stamps a brand-new claim file with
// fence_epoch=1 on the next O_EXCL claim — both are scoped to "is this ONE
// cell claimed right now", not "how many times has this session claimed
// into this feature while it stayed routeless". So this is its own small
// per-(feature, session) counter, persisted beside claims_dir under the
// control root (msn-18b: control-plane, the same root every claim/session
// file already resolves through) and bumped ONLY after a claim actually
// succeeds — a refused claim never advances it, and neither a different
// feature's nor a different session's counter file is ever touched by this
// one. A sessionless caller (no --session-id, no BEE_SESSION_ID/
// CLAUDE_CODE_SESSION_ID) is its own fixed bucket ("none"), so repeated
// sessionless claims in the same feature still escalate.
pub(crate) fn no_route_claim_counts_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("no_route_claims")
}

/// `<feature>__<session-fingerprint>` — the session half is a truncated
/// sha256 (never the raw id) so an arbitrary CLAUDE_CODE_SESSION_ID value
/// can never smuggle path separators or other filesystem-unsafe bytes into
/// the counter's file name; require_id already guards the feature half.
fn no_route_claim_key(feature: &str, session: Option<&str>) -> MR<String> {
    let feature_id = require_id(feature, "feature id")?;
    let session_part = match session.map(js_trim) {
        Some(s) if !s.is_empty() => {
            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            format!("{:x}", hasher.finalize())[..16].to_string()
        }
        _ => "none".to_string(),
    };
    Ok(format!("{feature_id}__{session_part}"))
}

pub(crate) fn no_route_claim_count_path(
    control: &Path,
    feature: &str,
    session: Option<&str>,
) -> MR<PathBuf> {
    Ok(no_route_claim_counts_dir(control).join(format!("{}.json", no_route_claim_key(feature, session)?)))
}

/// How many cells `session` has already claimed for `feature` while it had
/// no route record. 0 when the marker file is absent, unreadable, or
/// malformed — a corrupt counter fails open to "first claim", it never
/// blocks a claim door on its own bookkeeping.
pub(crate) fn no_route_claim_count(control: &Path, feature: &str, session: Option<&str>) -> MR<u64> {
    let file = no_route_claim_count_path(control, feature, session)?;
    let Ok(text) = std::fs::read_to_string(&file) else { return Ok(0) };
    let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) else { return Ok(0) };
    Ok(match m.get("count") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        _ => 0,
    })
}

/// Records that `session` just claimed another cell of `feature` with no
/// route record on file, advancing THIS (feature, session) pair's persisted
/// count by exactly one.
pub(crate) fn bump_no_route_claim_count(control: &Path, feature: &str, session: Option<&str>) -> MR<u64> {
    let next = no_route_claim_count(control, feature, session)? + 1;
    let file = no_route_claim_count_path(control, feature, session)?;
    let _ = std::fs::create_dir_all(no_route_claim_counts_dir(control));
    transient_fs_retry(|| {
        write_json_atomic(&file, &json!({ "feature": feature, "session": session, "count": next }))
    })
    .map_err(|e| Fail::Thrown(format!("{e}")))?;
    Ok(next)
}

/// claimCellFromFlags's product — bee.mjs returns `{cell, sessionId}`.
///
/// `policy` has no variant here on purpose: `applyWritePolicy` runs with
/// `enforceIsolation: false` on this door, so its only acting arms are
/// `observe` (a no-op) and `shared-disjoint` (a refusal); the `isolated`
/// workspace-attach / auto-isolate machinery that produces `redirect: true`
/// is structurally unreachable. Node's `if (policy.redirect) return {policy}`
/// in claimCellFromFlags — and therefore `handleDispatchPrepare`'s own
/// `if (claimOutcome.policy)` early return — are both provably dead code for
/// every argv this door serves. (Same reasoning already recorded for
/// claim-next below.)
pub(crate) struct ClaimDoor {
    pub(crate) cell: Value,
    pub(crate) session_id: Option<String>,
}

/// bee.mjs's `claimCellFromFlags` — "One claim door for cells.claim and
/// dispatch.prepare --claim": the write-policy resolution, the claims.mjs
/// claim-file-first sequence, the byte-identical claim refusal and the route
/// soft-warning, all in ONE body so the door cannot diverge between the two
/// verbs. `cells claim` adds only its own emit text; `dispatch prepare
/// --claim` adds only the reserve loop that follows it.
///
/// pub(crate) since the `dispatch prepare --claim` port — previously this was
/// inlined in `run_claim`'s closure, which is exactly why that verb delegated.
///
/// Every delegate-trigger is FRONT-LOADED (the two prescans, the store reads,
/// the exotic-shape probes) because nothing after claimCellFile's O_EXCL
/// write may delegate: the claim file would already exist for the Node re-run.
///
/// Thin wrapper over [`claim_cell_from_flags_ex`] for the callers (the
/// `dispatch prepare --claim` door, and the pre-D2 test fixtures) that carry
/// no `--fix-first` escape — same as passing `None`.
pub(crate) fn claim_cell_from_flags(
    root: &Path,
    id: &str,
    worker: &str,
    session_flag: Option<&str>,
    ttl: Option<f64>,
) -> MR<ClaimDoor> {
    claim_cell_from_flags_ex(root, id, worker, session_flag, ttl, None)
}

/// `claim_cell_from_flags` + D2's `--fix-first <reason>` escape (cells
/// claim's own flag; the `dispatch prepare --claim` door does not carry it).
pub(crate) fn claim_cell_from_flags_ex(
    root: &Path,
    id: &str,
    worker: &str,
    session_flag: Option<&str>,
    ttl: Option<f64>,
    fix_first: Option<&str>,
) -> MR<ClaimDoor> {
    let root = root.to_path_buf();
    let id = id.to_string();
    {
        if let Some(t) = ttl {
            if !t.is_finite() || t <= 0.0 {
                return Err(Fail::Thrown("--ttl must be a positive integer (seconds).".into()));
            }
        }
        // Pre-scan: everything after claimCellFile's O_EXCL write must never
        // delegate (the file would already exist for a retry).
        prescan_claim(&root, &id)?;
        let control = control_root(&root)?;
        delegate_only(list_session_records(&control))?;
        bstate::read_state_brief(&root);
        let config = bstate::read_config_raw(&root);
        let cell_for_policy = read_cell_norm(&root, &id)?;
        if let Some(cell) = &cell_for_policy {
            if !matches!(cell, Value::Object(_)) {
                return Err(Fail::Delegate); // truthy non-object cell — JS-exotic downstream
            }
            delegate_only(merge_trace(cell.get("trace")))?;
            delegate_only(lane_record_gates(&root, cell.get("feature")))?;
            // CUTOVER: the read_lane_route probe that stood here had exactly
            // one delegating arm — a corrupt/mismatched lane record — which
            // is native now. Keeping it would print readLane's warning twice.
            match cell.get("deps") {
                None => {}
                Some(deps) if !js_truthy(deps) => {}
                Some(Value::Array(_)) => {}
                Some(_) => return Err(Fail::Delegate), // truthy non-array deps
            }
        }

        let session_id = resolve_session_flag_env(session_flag);

        // applyWritePolicy (state.mjs) with enforceIsolation:false — only the
        // observe/shared-disjoint arms can act; 'isolated' passes through.
        let mode = match config.get("guards").and_then(|g| g.get("write_policy")) {
            Some(Value::String(s)) if js_trim(s) == "observe" => "observe",
            Some(Value::String(s)) if js_trim(s) == "shared-disjoint" => "shared-disjoint",
            _ => "isolated",
        };
        if mode == "shared-disjoint" {
            let declared: Vec<String> = match cell_for_policy.as_ref().and_then(|c| c.get("files")) {
                Some(Value::Array(files)) => files
                    .iter()
                    .filter_map(|f| match f {
                        Value::String(s) if !js_trim(s).is_empty() => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            if !declared.is_empty() {
                let now = rsv::now_ms();
                let records = list_path_lease_records(&root)?;
                let mut active: Vec<ResvLite> = Vec::new();
                for rec in &records {
                    if lease_record_expired(rec, now)? {
                        continue;
                    }
                    active.push(lease_to_resv_lite(rec)?);
                }
                let missing: Vec<String> = declared
                    .iter()
                    .filter(|p| {
                        !active.iter().any(|r| {
                            let session_match = match (&session_id, &r.session) {
                                (Some(sid), Some(Value::String(s))) => s == sid,
                                _ => false,
                            };
                            session_match && !r.path.ends_with('*') && rsv::paths_overlap(&r.path, p)
                        })
                    })
                    .cloned()
                    .collect();
                if !missing.is_empty() {
                    let session_suffix = session_id
                        .as_deref()
                        .map(|s| format!(" --session-id {s}"))
                        .unwrap_or_default();
                    return Err(Fail::Thrown(format!(
                        "bee write-policy (shared-disjoint): no exact-path lease held for: {}. A broad/glob reservation never satisfies shared-disjoint — an exact-path lease is mandatory before write. FIX: bee reservations reserve --agent <worker> --cell <id> --path <path>{session_suffix} for each path, then retry.",
                        missing.join(", ")
                    )));
                }
            }
        }

        // claimCellCrossSession (shared with claim-next — see its own comment).
        // D4's no-route deny and D2's red-base deny both live INSIDE that
        // call now, checked only once this caller has actually won the claim
        // door (its own comment explains why: a racing LOSER must see the
        // typed CLAIMED refusal, never either of these).
        let session = session_id.clone();
        let cell_id = js_trim(&id).to_string();
        let claimed = match claim_cell_cross_session_ex(
            &root,
            &control,
            session.as_deref(),
            worker,
            &id,
            ttl,
            cell_for_policy.as_ref(),
            fix_first,
        )? {
            CrossClaim::Ok { cell, .. } => cell,
            CrossClaim::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim: {code} — {reason}")));
            }
        };
        let _ = &cell_id;
        // explicit-triage D3/D4 soft route warning. THIS session's second
        // no-route claim never reaches here at all — claim_cell_cross_session
        // already refused it as NO_ROUTE_RECORD above — so anything landing
        // here with no route on file is guaranteed to be THIS session's
        // first claim of the feature, and is free to spend its one-time
        // warning.
        if !claimed_feature_has_route(&root, claimed.get("feature"))? {
            if let Some(Value::String(feature_name)) = claimed.get("feature") {
                bump_no_route_claim_count(&control, feature_name, session.as_deref())?;
            }
            eprint!(
                "WARNING: cell \"{}\" claimed for feature \"{}\" with no route record — run \"bee state route --set --class <c> --lane <l> --flags <f> --files <n>\" to record the triage (D3, soft enforcement; this session's next no-route claim for this feature will be refused — D4).\n",
                js_string_or_undefined(claimed.get("id")),
                js_string_or_undefined(claimed.get("feature"))
            );
        }
        Ok(ClaimDoor { cell: claimed, session_id })
    }
}

/// Visibility helper for rdv-1 (friction row 618): does `dep_cell` sit
/// "open" because a semantic-judge NEEDS_REVISION verdict reopened it (the
/// only writer of this shape is `run_judge_record`, handlers_meta.rs, which
/// flips a capped cell back to "open" exactly when its newest verdict is
/// "NEEDS_REVISION" — see its `reopened` branch)? Returns the quoted verdict
/// kind when true, so a claim refusal can name the real cause instead of
/// reading as a generic, permanent deadlock. This is read-only: it changes
/// no law — an uncapped dep is still uncapped either way.
fn revision_reopened_verdict(dep_cell: &Value) -> Option<String> {
    let is_open = matches!(dep_cell.get("status"), Some(Value::String(s)) if s == "open");
    if !is_open {
        return None;
    }
    let entries = dep_cell.get("trace").and_then(|t| t.get("semantic_judge"))?;
    let Value::Array(entries) = entries else { return None };
    let latest = entries.last()?;
    match latest.get("verdict") {
        Some(Value::String(s)) if s == "NEEDS_REVISION" => Some(s.clone()),
        _ => None,
    }
}

/// claimCellCrossSession's typed outcome. Node returns `{ok:false, code,
/// reason}`; each CLI caller prefixes it with its own verb word (`claim: …`
/// / `claim-next: …`), so the refusal stays typed until then.
pub(crate) enum CrossClaim {
    /// `{ok:true, cell, claim}` — `cells claim` reads only the cell,
    /// `cells claim-next` emits the whole envelope.
    Ok { cell: Value, claim: Value },
    Refused { code: String, reason: String },
}

/// lib/cells.mjs claimCellCrossSession — the CLAIM half shared by
/// `cells claim` and `cells claim-next`: claimCellFile's O_EXCL protocol, the
/// budget unwind (releaseClaim before surfacing, so a refused acquisition
/// never orphans a claims-store file), then claimCell under the `cells:<id>`
/// store lock with every throw unwinding into CLAIM_CELL_FAILED.
///
/// `cell_for_budget` is the caller's already-read cell record — Node re-reads
/// it here (`readCell(root, id)`); both callers pre-read it in the same
/// command, and the store cannot change under this process between the two
/// points, so the read is hoisted rather than repeated.
///
/// Thin wrapper over [`claim_cell_cross_session_ex`] for `cells claim-next`
/// (handlers_select.rs), which carries no `--fix-first` escape — same as
/// passing `None`: a red base always refuses that door.
#[allow(clippy::too_many_arguments)]
pub(crate) fn claim_cell_cross_session(
    root: &Path,
    control: &Path,
    session: Option<&str>,
    worker: &str,
    cell_id_in: &str,
    ttl: Option<f64>,
    cell_for_budget: Option<&Value>,
) -> MR<CrossClaim> {
    claim_cell_cross_session_ex(root, control, session, worker, cell_id_in, ttl, cell_for_budget, None)
}

/// `claim_cell_cross_session` + D2's `--fix-first <reason>` escape (docs/
/// history/hook-teeth CONTEXT.md D2: "no claim on a red base").
#[allow(clippy::too_many_arguments)]
pub(crate) fn claim_cell_cross_session_ex(
    root: &Path,
    control: &Path,
    session: Option<&str>,
    worker: &str,
    cell_id_in: &str,
    ttl: Option<f64>,
    cell_for_budget: Option<&Value>,
    fix_first: Option<&str>,
) -> MR<CrossClaim> {
    if js_trim(worker).is_empty() {
        return Err(Fail::Thrown("claimCellCrossSession: worker is required.".into()));
    }
    if js_trim(cell_id_in).is_empty() {
        return Err(Fail::Thrown("claimCellCrossSession: cellId is required.".into()));
    }
    let cell_id = js_trim(cell_id_in).to_string();
    let file_claim = match claim_cell_file(control, session, &cell_id, ttl)? {
        ClaimFileOutcome::Refused { code, reason } => {
            return Ok(CrossClaim::Refused { code: code.to_string(), reason });
        }
        ClaimFileOutcome::Ok { claim } => claim,
    };
    // Budget check inside the O_EXCL window.
    if let Some(Value::Object(cell_map)) = cell_for_budget {
        match check_cell_budgets(cell_map) {
            Ok(BudgetCheck::Ok) => {}
            Ok(BudgetCheck::Refused { code, reason }) => {
                release_claim(control, session, &cell_id)?;
                return Ok(CrossClaim::Refused { code: code.to_string(), reason });
            }
            Err(fail) => {
                // Pre-scanned; a mid-command race lands here — unwind the
                // claim file before surfacing anything.
                release_claim(control, session, &cell_id)?;
                return Err(fail);
            }
        }
    }
    // D4 (route-record warn-to-deny escalation) — checked ONLY here, after
    // this caller has already won the O_EXCL claim-file race and cleared
    // the budget door. Ordering is load-bearing: in a real claim race the
    // loser must see the typed CLAIMED refusal from claim_cell_file above,
    // never this one — a racing loser is not "continuing to claim without a
    // route", it never had the cell to begin with. Checked before the
    // store-lock mutation below so a refusal here never touches cell
    // status at all.
    if let Some(Value::Object(cell_map)) = cell_for_budget {
        let feature_val = cell_map.get("feature").cloned();
        let has_route = match claimed_feature_has_route(root, feature_val.as_ref()) {
            Ok(v) => v,
            Err(fail) => {
                release_claim(control, session, &cell_id)?;
                return Err(fail);
            }
        };
        if !has_route {
            if let Some(Value::String(feature_name)) = &feature_val {
                let seen = match no_route_claim_count(control, feature_name, session) {
                    Ok(v) => v,
                    Err(fail) => {
                        release_claim(control, session, &cell_id)?;
                        return Err(fail);
                    }
                };
                if seen >= 1 {
                    release_claim(control, session, &cell_id)?;
                    return Ok(CrossClaim::Refused {
                        code: "NO_ROUTE_RECORD".to_string(),
                        reason: format!(
                            "cell \"{cell_id}\" refused — feature \"{feature_name}\" still has no route record, and this session already spent its one-time warning on an earlier claim (D4: warn once per session, then refuse). FIX: bee state route --set --class <c> --lane <l> --flags <f> --files <n> to record the triage, then retry."
                        ),
                    });
                }
            }
        }
    }
    // D2 (no claim on a red base, docs/history/hook-teeth CONTEXT.md) —
    // checked AFTER the already-claimed refusal (claim_cell_file above) and
    // the D4 no-route deny, same reasoning as D4's own ordering note: a
    // racing loser or a session still owed its one-time no-route warning
    // must see ITS typed refusal first. Checked before the store-lock
    // mutation below so a refusal here never touches cell status either.
    if let Some(reason) = red_base_refusal(control, &cell_id, fix_first) {
        release_claim(control, session, &cell_id)?;
        return Ok(CrossClaim::Refused { code: "RED_BASE".to_string(), reason });
    }
    // claimCell under the per-cell store lock; every throw unwinds the
    // claim file and surfaces as CLAIM_CELL_FAILED.
    let claim_result = (|| -> MR<Value> {
        let mut guard = acquire_named_lock(root, &format!("cells:{cell_id}"))?;
        let outcome = (|| -> MR<Value> {
            let root = root;
            let worker = worker;
            {
                assert_not_archived(root, "claimCell", &cell_id)?;
                let cell = read_cell_norm(root, &cell_id)?;
                let lane_gates = match &cell {
                    Some(c) if js_truthy(c) => lane_record_gates(&root, c.get("feature"))?,
                    _ => None,
                };
                let approved = match &lane_gates {
                    Some(gates) => matches!(gates.get("execution"), Some(Value::Bool(true))),
                    None => default_gate_approved(&root, "execution")?,
                };
                if !approved {
                    let message = match (&lane_gates, &cell) {
                        (Some(_), Some(c)) => format!(
                            "claimCell: lane \"{}\" gate \"execution\" is not approved — cells of this feature cannot be claimed before ITS lane passes Gate 2 (D2: only the lane's own approvals authorize its cells — the default pipeline's gate never does). Surface Gate 2 to the user for lane \"{}\" and set its approved_gates.execution once approved.",
                            js_string_or_undefined(c.get("feature")),
                            js_string_or_undefined(c.get("feature"))
                        ),
                        _ => "claimCell: gate \"execution\" is not approved — cells cannot be claimed before execution is approved. Surface Gate 2 to the user (\"Work shape is ready. Approve before current-work preparation?\" — the merged shape+execution question) and set approved_gates.execution once approved. The opt-in gate_bypass switch may self-approve: level \"normal\" covers tiny/small/standard non-hard-gate work only; levels \"full\" and \"total\" also self-approve high-risk/hard-gate execution (decision 0010, total-autopilot dcf01d7b).".to_string(),
                    };
                    return Err(Fail::Thrown(message));
                }
                let Some(cell) = cell else {
                    return Err(Fail::Thrown(format!("claimCell: cell \"{cell_id}\" not found.")));
                };
                let status_open = matches!(cell.get("status"), Some(Value::String(s)) if s == "open");
                if !status_open {
                    return Err(Fail::Thrown(format!(
                        "claimCell: cell \"{cell_id}\" is \"{}\", not \"open\" — only open cells can be claimed. Run bee cells ready to list claimable cells.",
                        js_string_or_undefined(cell.get("status"))
                    )));
                }
                // depsAllCapped (cells.mjs flavor — collects misses). Also
                // notes which misses are a NEEDS_REVISION reopen (visibility
                // only, D2/rdv-1: the LAW is unchanged — deps still must be
                // capped — this only names a dep that is "open" because a
                // judge sent it back, not stuck for the ordinary reason.
                let mut uncapped: Vec<Value> = Vec::new();
                let mut revision_reopened: Vec<(String, String)> = Vec::new();
                if let Some(deps) = cell.get("deps") {
                    if js_truthy(deps) {
                        let Value::Array(deps) = deps else { return Err(Fail::Delegate) };
                        for dep in deps {
                            let dep_id = jsjson::js_to_string(dep);
                            let dep_cell = read_cell_norm(&root, &dep_id)?;
                            let capped = match &dep_cell {
                                Some(dc) => matches!(dc.get("status"), Some(Value::String(s)) if s == "capped"),
                                None => false,
                            };
                            if !capped {
                                uncapped.push(dep.clone());
                                if let Some(dc) = &dep_cell {
                                    if let Some(verdict) = revision_reopened_verdict(dc) {
                                        revision_reopened.push((dep_id.clone(), verdict));
                                    }
                                }
                            }
                        }
                    }
                }
                if !uncapped.is_empty() {
                    if revision_reopened.is_empty() {
                        return Err(Fail::Thrown(format!(
                            "claimCell: cell \"{cell_id}\" has uncapped deps: {} — deps must be capped first.",
                            js_join(&uncapped, ", ")
                        )));
                    }
                    let named = revision_reopened
                        .iter()
                        .map(|(id, verdict)| format!("\"{id}\" (reopened by a \"{verdict}\" judge verdict)"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(Fail::Thrown(format!(
                        "claimCell: cell \"{cell_id}\" has uncapped deps: {} — deps must be capped first. {named} is not stuck for the ordinary reason — this is not a permanent deadlock. Two sanctioned roads: (a) claim and re-cap the reopened dependency yourself first; (b) run `bee cells update` on \"{cell_id}\" to change its deps, recording a reason.",
                        js_join(&uncapped, ", ")
                    )));
                }
                let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
                cell_map.insert("status".into(), Value::String("claimed".into()));
                let mut trace = merge_trace(cell_map.get("trace"))?;
                trace.insert("worker".into(), Value::String(js_trim(worker).to_string()));
                trace.insert(
                    "claim_session".into(),
                    session.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null),
                );
                trace.insert("claimed_at".into(), Value::String(utc_now()));
                // D2: the --fix-first reason that escaped a red base, if any
                // — a cold reader must be able to see WHY this claim went
                // through onto a red result without re-deriving it.
                if let Some(reason) = fix_first {
                    trace.insert("fix_first".into(), Value::String(reason.to_string()));
                }
                cell_map.insert("trace".into(), Value::Object(trace));
                let cell_value = Value::Object(cell_map);
                write_cell(root, &cell_value)?;
                Ok(cell_value)
            }
        })();
        guard.release();
        outcome
    })();
    match claim_result {
        Ok(cell) => Ok(CrossClaim::Ok { cell, claim: file_claim }),
        Err(Fail::Thrown(message)) => {
            release_claim(control, session, &cell_id)?;
            Ok(CrossClaim::Refused { code: "CLAIM_CELL_FAILED".into(), reason: message })
        }
        Err(Fail::Delegate) => {
            // Pre-scanned; only a mid-command race lands here. Unwind so the
            // Node re-run isn't refused by our own claim file.
            release_claim(control, session, &cell_id)?;
            Err(Fail::Delegate)
        }
    }
}

// ── cells backfill-roles ───────────────────────────────────────────────────
//
// D9 (store `4eaf1b71`, plan.md S5) — the ONE-TIME backfill that gives every
// cell written before `role` existed the role it would have carried.
//
// WHY A VERB AND NOT LAZY-ON-READ. Three counters scan the whole store and
// divide by what they find — `ceiling_share_after` (handlers_close.rs, the
// 40% refusal), `status_full/cells.rs` and `hooks/session_preamble/store.rs`.
// A store where half the records answer "role" and half answer nothing makes
// every one of those denominators a lie, and the lie is silent. So the
// migration is a single pass with a single answer: after it runs, every
// readable stored cell carries a role.
//
// WHY IT IS IDEMPOTENT AND NOT MERELY RE-RUNNABLE. A cell that already
// carries a non-blank `role` is not re-derived, not re-normalized and not
// rewritten — its file is never opened for writing at all. That is what
// makes a second run a no-op down to the byte, and it is what makes an
// INTERRUPTED first run safe to finish by simply running again: the cells
// already done are indistinguishable from cells authored with a role.
//
// WHAT `ceiling` TAKES NOW. D5 (store `97ce5225`) landed the escalation
// flag, so this pass has a second job: every stored cell recording
// `tier: "ceiling"` — the old spelling of "run on the session model and
// charge the 40% ration" — is marked `escalate: true`. It is the same pass
// on purpose. A separate migration would leave a store where some
// escalations answer the flag and some answer the tier, and the ration
// divides by a whole-store scan.
//
// WHAT THE RETIRED `tier_reason` TAKES NOW. D4 (store `97ce5225`) retired
// the `tier` selector, and the escalation reason went with its name:
// `trace.tier_reason` is `trace.escalation_reason` from here on. mrs-14 left
// the key alone on purpose — `docs/handbook/register.md` publishes it and
// stored records carry it — so the rename lands as one change with its
// surfaces, and this pass is the third of them: wherever a stored trace still
// spells the key the old way, it is renamed in place, VALUE UNTOUCHED. Same
// pass, same reason as `escalate`: a half-renamed store would answer the same
// question two ways.
//
// The `tier` string itself is LEFT IN PLACE. D4 retires `tier` as a
// SELECTOR; it does not order stored history rewritten, and a legacy record
// carrying the field is harmless — `cell_is_escalated` reads exactly one of
// its values and nothing else reads it at all. So the verb writes three
// things, and only where they are missing or misspelled: `role`, `escalate`,
// and the escalation reason's key.
//
// NO COUNT IS HARDCODED. The decision measured 484 / 2 / 20 on 2026-08-24;
// the store has grown since and will grow again. Every number below is
// computed from the store the verb is handed, and nothing asserts a
// remembered total.

/// D9's mapping, in D9's own order, as `(source label, role)`. The label is
/// the REASON a cell takes its role, reported per-source so an operator sees
/// the shape of what is about to change rather than one lump total — and so
/// a source with zero cells is reported AS zero rather than omitted (an
/// absent row and an empty row must not read the same).
pub(crate) const ROLE_BACKFILL_SOURCES: [(&str, &str); 4] = [
    ("tier:generation", "code"),
    ("no-tier", "code"),
    ("tier:ceiling", "code"),
    ("tier:extraction", "read"),
];

/// D9's mapping as a function of the cell's recorded `tier`.
///
/// `None` in, `Some("code")` out: an absent (or blank) tier is the 215-cell
/// majority D4 measured, and D9 gives it the same role as `generation`.
/// `None` OUT is the deliberate hole: a tier value outside the three legal
/// ones is data this mapping has no answer for, and guessing "code" for it
/// would be exactly the silent default D7 exists to end. Those cells are
/// counted and NAMED instead, so a store that is not fully migrated says so.
pub(crate) fn d9_role_for_tier(tier: Option<&str>) -> Option<&'static str> {
    match tier.map(js_trim) {
        None | Some("") => Some("code"),
        Some("generation") => Some("code"),
        Some("ceiling") => Some("code"),
        Some("extraction") => Some("read"),
        Some(_) => None,
    }
}

/// The source label for a cell's recorded tier — the left column of
/// `ROLE_BACKFILL_SOURCES`.
pub(crate) fn d9_source_for_tier(tier: Option<&str>) -> &'static str {
    match tier.map(js_trim) {
        None | Some("") => "no-tier",
        Some("generation") => "tier:generation",
        Some("ceiling") => "tier:ceiling",
        Some("extraction") => "tier:extraction",
        Some(_) => "unmapped",
    }
}

/// What one full pass found. `written` is separate from `assigned` on
/// purpose: under `--dry-run` they differ (assigned N, written 0), and a
/// reader who would otherwise confuse "planned" with "done" is the reader
/// this split protects.
#[derive(Default)]
pub(crate) struct RoleBackfill {
    pub(crate) scanned: u64,
    pub(crate) already_roled: u64,
    pub(crate) assigned: u64,
    pub(crate) written: u64,
    /// Per-source counts, positionally aligned with `ROLE_BACKFILL_SOURCES`.
    pub(crate) by_source: [u64; 4],
    /// D5: cells converted from the legacy `tier: "ceiling"` spelling onto
    /// the `escalate` flag. Counted separately from `assigned` because the
    /// two answer different questions and a cell can need both.
    pub(crate) escalated: u64,
    /// D4: traces whose `tier_reason` key was renamed to `escalation_reason`.
    /// Its own counter for the same reason: a cell can need this and neither
    /// of the other two, and an operator reading "0 escalated" must not read
    /// it as "no reason moved".
    pub(crate) reasons_renamed: u64,
    /// `(id, tier)` for every cell whose tier D9's mapping does not cover.
    pub(crate) unmapped: Vec<(String, String)>,
    /// Store-relative paths of files that are absent, corrupt, or not a JSON
    /// object — skipped and named, never guessed at.
    pub(crate) unreadable: Vec<String>,
    /// Store-relative paths of records that MOVED between the scan and the
    /// lock — another writer landed, or `bee close` archived the feature.
    /// The part of their planned migration the record no longer asks for was
    /// dropped rather than written over it. Named rather than retried in
    /// place, because the pass is idempotent and a re-run finishes them
    /// against a store that has stopped moving.
    pub(crate) changed_during_pass: Vec<String>,
}

impl RoleBackfill {
    /// Role totals folded out of the per-source counts, so the two can never
    /// disagree.
    pub(crate) fn by_role(&self) -> Vec<(&'static str, u64)> {
        let mut out: Vec<(&'static str, u64)> = Vec::new();
        for (i, (_, role)) in ROLE_BACKFILL_SOURCES.iter().enumerate() {
            match out.iter_mut().find(|(r, _)| r == role) {
                Some(slot) => slot.1 += self.by_source[i],
                None => out.push((role, self.by_source[i])),
            }
        }
        out
    }
}

/// Every stored cell file: the hot `.bee/cells/*.json` scan path AND
/// `.bee/cells/archive/<feature>/*.json`.
///
/// The archive is in scope because D9 says "the stored cells", and a capped
/// cell that was archived is stored history exactly as an active one is —
/// `readCell` reaches it, `cells unarchive` brings it back live, and a role
/// count taken after an unarchive would otherwise find a hole. Sorted by
/// path so two runs over one store report their findings in one order.
pub(crate) fn stored_cell_files(root: &Path) -> Vec<PathBuf> {
    fn push_json_files(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(d) else { return };
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name();
            if name.to_str().map(|n| n.ends_with(".json")).unwrap_or(false) {
                out.push(entry.path());
            }
        }
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let dir = cells_dir(root);
    push_json_files(&dir, &mut out);
    let archive_root = dir.join(ARCHIVE_DIR_NAME);
    if let Ok(features) = std::fs::read_dir(&archive_root) {
        let mut feature_dirs: Vec<PathBuf> = features
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .collect();
        feature_dirs.sort();
        for feature_dir in feature_dirs {
            push_json_files(&feature_dir, &mut out);
        }
    }
    out.sort();
    out
}

/// A path rendered relative to the store root when it sits under one, so a
/// report names `.bee/cells/x.json` rather than a machine-specific absolute.
fn store_relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().replace('\\', "/")
}

/// The three keys D9's migration owns, derived from ONE record: `role`
/// (D9), `escalate` (D5), and the escalation reason's key inside `trace`
/// (D4). Naming the change as a value rather than as a rewritten object is
/// what lets the write half re-read a record under the lock and apply only
/// the part that record still asks for — a cell's other fields are never
/// this pass's to write, and a whole-object write of a stale clone is
/// exactly how a concurrent writer gets reversed.
#[derive(Clone, Default, PartialEq)]
pub(crate) struct RoleMigration {
    /// The role to add. `None` when the record already carries one, and
    /// `None` when its tier is outside D9's mapping — that hole is counted
    /// and named, never guessed at.
    role: Option<&'static str>,
    /// Which row of `ROLE_BACKFILL_SOURCES` the role came from, so the
    /// per-source counts fold out of the same derivation that produced the
    /// bytes rather than out of a second reading of them.
    source: Option<usize>,
    /// D5: the legacy `tier: "ceiling"` spelling, not yet on the flag.
    escalate: bool,
    /// D4: the reason value still sitting under the retired key.
    reason: Option<Value>,
}

impl RoleMigration {
    /// Nothing to add — not re-derived, not rewritten, not even opened for
    /// writing. This IS the idempotence guarantee.
    fn is_noop(&self) -> bool {
        self.role.is_none() && !self.escalate && self.reason.is_none()
    }

    /// The part of a planned migration that a fresh reading of the same
    /// record still asks for, key by key. A key both readings agree on is
    /// applied; a key that moved between the scan and the lock belongs to
    /// whoever moved it, so it is dropped here rather than written over.
    /// Dropping costs nothing but a re-run: the pass is idempotent, and the
    /// record it dropped is named in `changed_during_pass`.
    fn still_agreed(&self, fresh: &Self) -> Self {
        let role_agrees = matches!((self.role, fresh.role), (Some(p), Some(f)) if p == f);
        Self {
            role: if role_agrees { self.role } else { None },
            source: if role_agrees { self.source } else { None },
            escalate: self.escalate && fresh.escalate,
            reason: match (&self.reason, &fresh.reason) {
                (Some(planned), Some(now)) if planned == now => Some(now.clone()),
                _ => None,
            },
        }
    }
}

/// What ONE record needs, read off that record and nothing remembered.
fn plan_role_migration(cell: &Map<String, Value>) -> RoleMigration {
    let tier = cell.get("tier").and_then(|t| t.as_str());
    let mut plan = RoleMigration::default();
    if !nonblank_string(cell.get("role")) {
        plan.role = d9_role_for_tier(tier);
        if plan.role.is_some() {
            let source = d9_source_for_tier(tier);
            plan.source = ROLE_BACKFILL_SOURCES.iter().position(|(s, _)| *s == source);
        }
    }
    // D5: the legacy escalation spelling, and whether it has already been
    // converted. A cell that carries the flag key AT ALL is done, whichever
    // pass or operator put it there — escalate-off-disarm D1: an explicit
    // false is a recorded disarm, and re-deriving true from the tier string
    // would silently reverse the operator's act on the next run.
    plan.escalate = matches!(tier.map(js_trim), Some(t) if t == crate::verbs::drivers::ESCALATION_WORD)
        && !matches!(cell.get(ESCALATE_FIELD), Some(Value::Bool(_)));
    // D4: the escalation reason under its retired key. Renamed only when
    // the new key is not already there, so a record migrated once is never
    // rewritten and never has its current reason overwritten by a stale one.
    plan.reason = cell
        .get("trace")
        .and_then(Value::as_object)
        .filter(|t| !t.contains_key(ESCALATION_REASON_KEY))
        .and_then(|t| t.get(LEGACY_ESCALATION_REASON_KEY))
        .cloned();
    plan
}

/// The migration, onto the record it was derived from — three keys and no
/// others. `tier` is not among what this writes: D4 retires it as a
/// selector, and a stored record may keep carrying the string harmlessly.
fn apply_role_migration(cell: &mut Map<String, Value>, plan: &RoleMigration) {
    if let Some(role) = plan.role {
        cell.insert("role".into(), Value::String(role.to_string()));
    }
    if plan.escalate {
        cell.insert(ESCALATE_FIELD.into(), Value::Bool(true));
    }
    if let Some(reason) = plan.reason.clone() {
        if let Some(trace) = cell.get_mut("trace").and_then(Value::as_object_mut) {
            trace.remove(LEGACY_ESCALATION_REASON_KEY);
            trace.insert(ESCALATION_REASON_KEY.into(), reason);
        }
    }
}

/// The counters for one migration. In an applied run this is fed from what
/// was WRITTEN rather than from what was planned, so every number describes
/// bytes that are on disk.
fn record_role_migration(report: &mut RoleBackfill, plan: &RoleMigration) {
    if plan.role.is_some() {
        report.assigned += 1;
        if let Some(i) = plan.source {
            report.by_source[i] += 1;
        }
    }
    if plan.escalate {
        report.escalated += 1;
    }
    if plan.reason.is_some() {
        report.reasons_renamed += 1;
    }
}

/// One full pass. Scans EVERY stored cell and builds the whole plan before
/// it writes a single file. Count provenance is split, and deliberately so
/// (role-edge-hardening D1): the mutation counts — `assigned`, `escalated`,
/// `reasons_renamed`, `written`, `changed_during_pass` — are folded from the
/// under-lock pass, so they describe what was actually written.
/// `already_roled` and `unmapped` come from the unlocked scan, corrected for
/// the one way they can move: a planned cell whose fresh under-lock reading
/// shows a role it lacked at scan time is counted into `already_roled`
/// (`role` has no removal door, so the count can drift in no other
/// direction). Recounting the REST of the store under the lock would hold
/// the archive lock across a full re-scan — the exact refusal-for-a-second
/// this design exists to avoid.
///
/// Concurrency: the scan runs UNLOCKED, deliberately. A 40 000-cell store
/// takes about a second to scan, and holding the archive lock across it
/// would refuse every other writer in the repository for that whole second.
/// What an unlocked scan may produce is therefore a PLAN, never bytes.
///
/// The write half takes the `cells-archive` store lock — the same lock
/// `writeCell` acquires (single-attempt) on every cell write, and the same
/// lock `archiveFeature` holds across a whole feature's move — and then
/// RE-READS every planned file under it. Each record is migrated from that
/// fresh reading, and only for the keys this migration owns (`role`,
/// `escalate`, `trace.escalation_reason`) that the fresh reading still asks
/// for. Three things follow, and they are the contract:
///
/// - A writer that COMPLETES during the scan is not reversed. Its bytes are
///   in the copy this pass migrates, and any owned key it moved is dropped
///   from the plan and named in `changed_during_pass` for the next run.
/// - A file that went away under the scan — `bee close` archiving its
///   feature — is skipped, never recreated. A whole-object write here would
///   resurrect an archived cell as a live duplicate, because `readCell`
///   prefers the live copy.
/// - A writer that ARRIVES during the write half is refused by the lock
///   itself, with the typed CELLS_ARCHIVE_BUSY message telling it to retry.
pub(crate) fn backfill_roles(root: &Path, dry_run: bool) -> MR<RoleBackfill> {
    backfill_roles_interleaved(root, dry_run, || {})
}

/// `backfill_roles` with a seam between the scan and the lock. That window —
/// unlocked, plan built, nothing written yet — is the one the concurrency
/// contract above is entirely about, and handing a test the window itself is
/// the only way to pin the contract without a sleep or a thread race.
/// Production takes the no-op closure.
pub(crate) fn backfill_roles_interleaved(
    root: &Path,
    dry_run: bool,
    between_scan_and_write: impl FnOnce(),
) -> MR<RoleBackfill> {
    let mut report = RoleBackfill::default();
    let mut plan: Vec<(PathBuf, RoleMigration, bool)> = Vec::new();

    for file in stored_cell_files(root) {
        report.scanned += 1;
        let cell = match read_json(&file) {
            ReadJson::Parsed(Value::Object(map)) => map,
            ReadJson::Parsed(_) | ReadJson::Missing | ReadJson::Corrupt => {
                report.unreadable.push(store_relative(root, &file));
                continue;
            }
        };
        let has_role = nonblank_string(cell.get("role"));
        if has_role {
            report.already_roled += 1;
        }
        let planned = plan_role_migration(&cell);
        if !has_role && planned.role.is_none() {
            report.unmapped.push((
                js_string_or_undefined(cell.get("id")),
                cell.get("tier").and_then(|t| t.as_str()).unwrap_or("").to_string(),
            ));
        }
        if planned.is_noop() {
            continue;
        }
        plan.push((file, planned, has_role));
    }

    if dry_run {
        for (_, planned, _) in &plan {
            record_role_migration(&mut report, planned);
        }
        return Ok(report); // `written` stays 0 — nothing was opened for writing
    }

    between_scan_and_write();

    let mut guard = acquire_named_lock(root, "cells-archive")?;
    let mut written = 0u64;
    let mut applied: Vec<RoleMigration> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    let outcome = (|| -> MR<()> {
        for (file, planned, had_role_at_scan) in &plan {
            let fresh = match read_json(file) {
                ReadJson::Parsed(Value::Object(map)) => map,
                // Gone or unreadable since the scan: archived out from under
                // the pass, or replaced by something that is not a record.
                // Writing the plan here is what would recreate an archived
                // cell as a live duplicate.
                ReadJson::Parsed(_) | ReadJson::Missing | ReadJson::Corrupt => {
                    changed.push(store_relative(root, file));
                    continue;
                }
            };
            // A role gained during the scan window is counted from the fresh
            // reading — `role` has no removal door, so this is the one
            // direction `already_roled` can drift (see the fn doc).
            if !had_role_at_scan && nonblank_string(fresh.get("role")) {
                report.already_roled += 1;
            }
            let agreed = planned.still_agreed(&plan_role_migration(&fresh));
            if agreed != *planned {
                changed.push(store_relative(root, file));
            }
            if agreed.is_noop() {
                continue;
            }
            let mut migrated = fresh;
            apply_role_migration(&mut migrated, &agreed);
            write_json_atomic(file, &Value::Object(migrated)).map_err(|e| {
                Fail::Thrown(format!("cells backfill-roles: writing {} — {e}", file.display()))
            })?;
            written += 1;
            applied.push(agreed);
        }
        Ok(())
    })();
    guard.release();
    outcome?;
    for applied_plan in &applied {
        record_role_migration(&mut report, applied_plan);
    }
    report.written = written;
    report.changed_during_pass = changed;
    Ok(report)
}

/// The report as JSON. `dry_run` leads because every other number is read
/// differently depending on it.
pub(crate) fn role_backfill_json(report: &RoleBackfill, dry_run: bool) -> Value {
    let mut by_source = Map::new();
    for (i, (source, _)) in ROLE_BACKFILL_SOURCES.iter().enumerate() {
        by_source.insert((*source).to_string(), json!(report.by_source[i]));
    }
    let mut by_role = Map::new();
    for (role, count) in report.by_role() {
        by_role.insert(role.to_string(), json!(count));
    }
    json!({
        "dry_run": dry_run,
        "scanned": report.scanned,
        "already_roled": report.already_roled,
        "assigned": report.assigned,
        "written": report.written,
        "by_role": Value::Object(by_role),
        "by_source": Value::Object(by_source),
        "escalated": report.escalated,
        "reasons_renamed": report.reasons_renamed,
        "unmapped": report
            .unmapped
            .iter()
            .map(|(id, tier)| json!({"id": id, "tier": tier}))
            .collect::<Vec<_>>(),
        "unreadable": report.unreadable.clone(),
        "changed_during_pass": report.changed_during_pass.clone(),
    })
}

/// The human report. Every line is a count this pass measured; none of it is
/// remembered from the decision that ordered the migration.
pub(crate) fn role_backfill_text(report: &RoleBackfill, dry_run: bool) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "cells backfill-roles{}: {} stored cell(s) scanned, {} already carry a role, {} {} a role.",
        if dry_run { " --dry-run" } else { "" },
        report.scanned,
        report.already_roled,
        report.assigned,
        if dry_run { "would take" } else { "took" }
    ));
    for (i, (source, role)) in ROLE_BACKFILL_SOURCES.iter().enumerate() {
        lines.push(format!(
            "  {:<16} -> role {:<5} {:>5}{}",
            source,
            role,
            report.by_source[i],
            if *source == "tier:ceiling" { "  (plus the escalation flag)" } else { "" }
        ));
    }
    lines.push(format!(
        "  {:<16} -> escalate: true {:>3}{}",
        "tier:ceiling",
        report.escalated,
        if dry_run { "  (would be marked)" } else { "  (marked)" }
    ));
    lines.push(format!(
        "  {:<16} -> trace.{} {:>3}{}",
        format!("trace.{LEGACY_ESCALATION_REASON_KEY}"),
        ESCALATION_REASON_KEY,
        report.reasons_renamed,
        if dry_run { "  (would be renamed)" } else { "  (renamed)" }
    ));
    if !report.unmapped.is_empty() {
        lines.push(format!(
            "  {} cell(s) carry a tier D9 does not map and were left alone: {}",
            report.unmapped.len(),
            report
                .unmapped
                .iter()
                .map(|(id, tier)| format!("{id} (tier \"{tier}\")"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !report.unreadable.is_empty() {
        lines.push(format!(
            "  {} unreadable file(s) skipped: {}",
            report.unreadable.len(),
            report.unreadable.join(", ")
        ));
    }
    if !report.changed_during_pass.is_empty() {
        lines.push(format!(
            "  {} cell(s) changed under the pass and kept their own writer's value: {}",
            report.changed_during_pass.len(),
            report.changed_during_pass.join(", ")
        ));
    }
    lines.push(if dry_run {
        "Nothing was written. Re-run without --dry-run to apply; running it twice changes nothing the second time.".to_string()
    } else {
        format!("{} cell file(s) written. Re-running changes nothing.", report.written)
    });
    lines.join("\n")
}

/// `bee cells backfill-roles [--dry-run] [--json]`.
pub(crate) fn run_backfill_roles(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["dry-run"]) {
        return None;
    }
    let dry_run = bool_flag(&flags, "dry-run")?;
    dispatch("cells backfill-roles", use_json, t0, move |ctx| {
        let report = backfill_roles(&ctx.root, dry_run)?;
        Ok(Out::Emit(role_backfill_json(&report, dry_run), role_backfill_text(&report, dry_run), 0))
    })
}
