use super::*;
use crate::fsutil::{read_json, ReadJson};
use serde_json::{Map, Value};
use std::path::Path;

const DESIGN_GATE: &str = "bee design gate";
const DESIGN_NOT_RUN: &str = "design gate not run";
const DESIGN_BASELINE: &str = ".bee/design/gate-baseline.json";
const DESIGN_LAST_RUN: &str = ".bee/design/last-run.json";
const DEFAULT_UI_GLOBS: [&str; 7] = ["*.html", "*.css", "*.scss", "*.jsx", "*.tsx", "*.vue", "*.svelte"];

fn cell_files(cell: &Map<String, Value>) -> Vec<String> {
    match cell.get("files") {
        Some(Value::Array(files)) => files.iter().map(frozen_normalize_path).collect(),
        _ => Vec::new(),
    }
}

fn bound_ui_cell(root: &Path, cell: &Map<String, Value>) -> bool {
    let config = crate::state::read_config_raw(root);
    let Some(design) = config.get("design") else { return false };
    if !matches!(design.get("packs"), Some(Value::Array(packs)) if !packs.is_empty()) {
        return false;
    }
    let globs: Vec<&str> = match design.get("ui_globs") {
        Some(Value::Array(globs)) => globs.iter().filter_map(Value::as_str).collect(),
        _ => DEFAULT_UI_GLOBS.to_vec(),
    };
    cell_files(cell).iter().any(|path| {
        let name = path.rsplit('/').next().unwrap_or(path);
        globs.iter().any(|g| glob_covers(g, name) || glob_covers(g, path))
    })
}

fn runs_gate(cell: &Map<String, Value>) -> bool {
    matches!(cell.get("verify"), Some(Value::String(v)) if v.contains(DESIGN_GATE))
}

fn gate_not_run(dir: &Path) -> bool {
    matches!(
        read_json(&dir.join(DESIGN_LAST_RUN)),
        ReadJson::Parsed(record) if record.get("status").and_then(Value::as_str) == Some("not-run")
    )
}

pub(crate) fn design_cap_refusal(root: &Path, id: &str, cell: &Map<String, Value>, proof_reason: &str) -> Option<String> {
    if cell_files(cell).iter().any(|p| p == DESIGN_BASELINE) {
        return Some(format!(
            "capCell: cell \"{id}\" refused — its files list {DESIGN_BASELINE}, the baseline the design gate checks it against. FIX: drop that path from the cell's files; the baseline is recorded by a person with `bee design gate --write` when no cell is claimed."
        ));
    }
    if !bound_ui_cell(root, cell) {
        return None;
    }
    if !runs_gate(cell) {
        return Some(format!(
            "capCell: cell \"{id}\" refused — it changes UI files in a project that picked design packs, and its verify does not run `{DESIGN_GATE}`. FIX: add `{DESIGN_GATE}` to the cell's verify (`bee cells update --id {id} --stdin` while it is open) and run it."
        ));
    }
    let worktree = commit_trailer_history_root(root, cell.get("feature").and_then(Value::as_str));
    if !proof_reason.contains(DESIGN_NOT_RUN) && (gate_not_run(root) || gate_not_run(&worktree)) {
        return Some(format!(
            "capCell: cell \"{id}\" refused — {DESIGN_LAST_RUN} records that the design gate did not run. FIX: say in the proof reason that the design gate did not run because Node 20 or newer is missing (\"{DESIGN_NOT_RUN} — Node 20 or newer is missing\")."
        ));
    }
    None
}

pub(crate) fn emit_design_gate_warnings(root: &Path, cells: &[Value]) {
    for cell in cells {
        let Value::Object(map) = cell else { continue };
        if bound_ui_cell(root, map) && !runs_gate(map) {
            let id = map.get("id").and_then(Value::as_str).unwrap_or("(unknown id)");
            eprintln!(
                "WARNING: cell \"{id}\" changes UI files in a project that picked design packs, and its verify does not run `{DESIGN_GATE}` — its cap will be refused. FIX: add `{DESIGN_GATE}` to the cell's verify (`bee cells update --id {id} --stdin`)."
            );
        }
    }
}
