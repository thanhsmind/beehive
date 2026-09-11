// `gate preview` / `state gate --preview` — parse, validate, hash, and render
// exact cell execution packets from the feature worktree plan.md (D3).
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::verbs::cells::{id_pattern_ok, LANES};
use crate::verbs::reservations::{
    finish, js_disp, keys_known, now_iso, parse_flags, truthy,
    Err2, Flags, Out, R2,
};
use crate::verbs::workflow_store::list_workflows;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// `## Some title` -> `Some title`, for 1-6 hashes followed by whitespace.
fn heading_title(line: &str) -> Option<&str> {
    let t = line.trim();
    let hashes = t.len() - t.trim_start_matches('#').len();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &t[hashes..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(rest.trim())
}

/// Find the `## Cells...` section in plan.md text.
fn find_cells_section(text: &str) -> Option<&str> {
    let mut start = None;
    let mut end = text.len();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let here = offset;
        offset += line.len();
        if let Some(title) = heading_title(line) {
            let lower = title.to_ascii_lowercase();
            if start.is_none() {
                if lower.starts_with("cells") {
                    start = Some(offset);
                }
            } else {
                end = here;
                break;
            }
        }
    }
    let start = start?;
    Some(&text[start..end.max(start)])
}

/// Extract the first fenced JSON block (` ```json ... ``` ` or ` ``` ... ``` `) from text.
fn extract_fenced_json(section: &str) -> Option<String> {
    let mut in_fence = false;
    let mut block = String::new();
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if !in_fence {
                in_fence = true;
                continue;
            } else {
                return Some(block);
            }
        }
        if in_fence {
            block.push_str(line);
            block.push('\n');
        }
    }
    None
}

/// Parse and validate the exact cell packet from plan.md.
pub(crate) fn parse_plan_packets(text: &str) -> Result<Vec<Value>, String> {
    let Some(section) = find_cells_section(text) else {
        return Err("plan.md has no \"## Cells...\" section".to_string());
    };
    let Some(fenced) = extract_fenced_json(section) else {
        return Err("the cells preview section carries no fenced JSON code block".to_string());
    };
    let parsed: Value = serde_json::from_str(&fenced)
        .map_err(|e| format!("could not parse cell packet JSON from plan.md: {e}"))?;
    let cells = match parsed {
        Value::Array(a) => a,
        _ => return Err("cell packet must be a JSON array of cell objects".to_string()),
    };
    if cells.is_empty() {
        return Err("the cell packet has zero cells \u{2014} current slice must declare at least one cell".to_string());
    }

    let mut problems = Vec::new();
    for (idx, cell) in cells.iter().enumerate() {
        let n = idx + 1;
        let Value::Object(map) = cell else {
            problems.push(format!("cell {n} is not a JSON object"));
            continue;
        };
        let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
        if id.is_empty() {
            problems.push(format!("cell {n} is missing required field \"id\""));
        } else if !id_pattern_ok(id) {
            problems.push(format!("cell {n} has invalid id \"{id}\""));
        }
        let id_disp = if id.is_empty() {
            format!("cell {n}")
        } else {
            format!("cell \"{id}\"")
        };

        for field in ["title", "action", "verify"] {
            match map.get(field).and_then(|v| v.as_str()).map(str::trim) {
                Some(s) if !s.is_empty() => {}
                _ => problems.push(format!(
                    "{id_disp} is missing required field \"{field}\" (non-empty string)"
                )),
            }
        }

        match map.get("files") {
            Some(Value::Array(a))
                if !a.is_empty()
                    && a.iter().all(|item| matches!(item, Value::String(s) if !s.trim().is_empty())) => {}
            _ => problems.push(format!(
                "{id_disp} is missing required field \"files\" (non-empty array of file path strings)"
            )),
        }

        match map.get("read_first") {
            Some(Value::Array(a)) if a.iter().all(|item| matches!(item, Value::String(_))) => {}
            _ => problems.push(format!(
                "{id_disp} is missing required field \"read_first\" (array of strings)"
            )),
        }

        let lane = map.get("lane").and_then(|v| v.as_str()).unwrap_or("").trim();
        if lane.is_empty() || !LANES.contains(&lane) {
            problems.push(format!(
                "{id_disp} has invalid lane \"{lane}\" \u{2014} must be one of {}",
                LANES.join(", ")
            ));
        }

        let role = map.get("role").and_then(|v| v.as_str()).unwrap_or("").trim();
        if role.is_empty() {
            problems.push(format!(
                "{id_disp} is missing required field \"role\" (non-empty string)"
            ));
        }

        match map.get("must_haves") {
            Some(Value::Object(mh)) => {
                if lane == "standard" || lane == "high-risk" {
                    match mh.get("truths") {
                        Some(Value::Array(t))
                            if !t.is_empty()
                                && t.iter().all(|item| {
                                    matches!(item, Value::String(s) if !s.trim().is_empty())
                                }) => {}
                        _ => problems.push(format!(
                            "{id_disp} requires non-empty must_haves.truths for lane \"{lane}\""
                        )),
                    }
                }
            }
            _ => problems.push(format!(
                "{id_disp} is missing required object field \"must_haves\""
            )),
        }
    }

    if !problems.is_empty() {
        return Err(format!(
            "cell packet validation failed: {}",
            problems.join("; ")
        ));
    }
    Ok(cells)
}

/// SHA-256 of plan.md bytes.
pub(crate) fn compute_plan_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Render preview text showing action, files, read_first, must_haves, and exact verify.
pub(crate) fn render_preview_text(feature: &str, cells: &[Value], plan_sha256: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Gate preview for feature \"{feature}\" ({} cell(s), plan sha256: {plan_sha256}):",
        cells.len()
    ));
    for cell in cells {
        let id = cell.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        let title = cell.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let lane = cell.get("lane").and_then(|v| v.as_str()).unwrap_or("");
        let role = cell.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let action = cell.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let verify = cell.get("verify").and_then(|v| v.as_str()).unwrap_or("");
        let files: Vec<&str> = cell
            .get("files")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_default();
        let read_first: Vec<&str> = cell
            .get("read_first")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_default();
        let must_haves = cell.get("must_haves").cloned().unwrap_or(Value::Null);

        lines.push(format!("\nCell: {id} \u{2014} {title}"));
        lines.push(format!("  lane: {lane}, role: {role}"));
        lines.push(format!("  action: {action}"));
        lines.push(format!("  files: {}", files.join(", ")));
        lines.push(format!("  read_first: {}", read_first.join(", ")));
        lines.push(format!("  verify: {verify}"));
        lines.push(format!(
            "  must_haves: {}",
            serde_json::to_string(&must_haves).unwrap_or_default()
        ));
    }
    lines.join("\n")
}

/// The CLI runner for `gate preview` / `state gate --preview`.
pub(crate) fn run_gate_preview(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "no-lane", "preview"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("gate preview", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = run_gate_preview_body(&ctx.root, &flags, use_json);
    finish(&ctx, out)
}

/// Root-parameterized body for tests and CLI.
pub(crate) fn run_gate_preview_body(root: &Path, flags: &Flags, _use_json: bool) -> R2<Out> {
    let (lane_feature, no_lane) = match mutation_lane_selector(flags, "gate preview") {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let scope = resolve_mutation_lock_scope(root, lane_feature.as_deref(), no_lane)?;
    let workflows = list_workflows(root)?;
    let locks = acquire_mutation_locks(root, &scope, &workflows)?;
    let mut target =
        resolve_mutation_target(root, lane_feature.as_deref(), "gate preview", no_lane)?;
    let lane_note = target.lane_note();

    let feature = target
        .record()
        .get("feature")
        .filter(|v| truthy(v))
        .map(js_disp);
    let Some(feature) = feature else {
        return Ok(Out::Thrown(
            "gate preview: no active feature to preview plan for. FIX: start a feature before previewing gate packet."
                .to_string(),
        ));
    };
    let plan_path = advisor_plan_path(root, &feature);
    let bytes = match std::fs::read(&plan_path) {
        Ok(b) => b,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok(Out::Thrown(format!(
                "gate preview: plan.md does not exist for feature \"{feature}\" at {}. FIX: create plan.md with current slice cells before previewing.",
                plan_path.display()
            )));
        }
        Err(e) => {
            return Ok(Out::Thrown(format!(
                "gate preview: could not read plan.md ({}).",
                io_read_reason(&plan_path, &e)
            )));
        }
    };
    let text = String::from_utf8_lossy(&bytes);
    let cells = match parse_plan_packets(&text) {
        Ok(c) => c,
        Err(e) => return Ok(Out::Thrown(format!("gate preview: {e}"))),
    };
    let plan_sha256 = compute_plan_sha256(&bytes);

    let mut preview_map = Map::new();
    preview_map.insert("feature".into(), json!(feature));
    preview_map.insert("plan_sha256".into(), json!(plan_sha256));
    preview_map.insert("previewed_at".into(), json!(now_iso()));
    preview_map.insert("cells".into(), Value::Array(cells.clone()));
    let preview_val = Value::Object(preview_map.clone());

    target
        .record_mut()
        .insert("gate_preview".into(), preview_val.clone());
    let record = target.record().clone();
    write_through_projection(root, &target, &record, &[])?;
    drop(locks);

    let text_out = format!("{}{lane_note}", render_preview_text(&feature, &cells, &plan_sha256));
    Ok(Out::Emit(preview_val, text_out, 0))
}

/// Refusal text when gate shape approval is attempted without a fresh preview.
pub(crate) fn gate_preview_refusal(
    root: &Path,
    record: &Map<String, Value>,
    lane: Option<&str>,
) -> Option<String> {
    let feature = match lane {
        Some(l) => l.to_string(),
        None => record.get("feature").filter(|v| truthy(v)).map(js_disp)?,
    };
    let mode = record
        .get("mode")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            record
                .get("route")
                .and_then(|r| r.get("lane"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| crate::verbs::drivers::feature_route(root, &feature).ok().flatten());
    let is_exempt = matches!(mode.as_deref(), Some("tiny") | Some("small") | Some("docs"));
    if is_exempt {
        return None;
    }

    let plan_path = advisor_plan_path(root, &feature);
    let bytes = match std::fs::read(&plan_path) {
        Ok(b) => b,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Some(format!(
                "gate: approval refused \u{2014} plan.md does not exist for feature \"{feature}\" at {}. FIX: create plan.md with current slice cells and run `bee gate --preview` before approving the shape gate.",
                plan_path.display()
            ));
        }
        Err(e) => {
            return Some(format!(
                "gate: approval refused \u{2014} plan.md is present but could not be read ({})",
                io_read_reason(&plan_path, &e)
            ));
        }
    };
    let current_sha256 = compute_plan_sha256(&bytes);
    let preview = match record.get("gate_preview") {
        Some(Value::Object(m)) => m,
        _ => {
            return Some(format!(
                "gate: approval refused \u{2014} missing gate preview for feature \"{feature}\". FIX: run `bee gate --preview` to review the execution packet before approving the shape gate."
            ));
        }
    };
    if preview.get("feature").map(js_disp).as_deref() != Some(&feature) {
        return Some(format!(
            "gate: approval refused \u{2014} gate preview feature mismatch for \"{feature}\". FIX: run `bee gate --preview` for this feature."
        ));
    }
    let preview_sha256 = preview
        .get("plan_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if preview_sha256 != current_sha256 {
        return Some(format!(
            "gate: approval refused \u{2014} stale gate preview for feature \"{feature}\" (plan.md changed since preview was generated). FIX: run `bee gate --preview` to preview the updated cell packet before approving the shape gate."
        ));
    }
    None
}

/// Helper to get the approved cell packet for a feature from its lane or state record.
fn get_approved_preview_packet(root: &Path, feature: &str) -> Option<Map<String, Value>> {
    let lane_file = root.join(".bee").join("lanes").join(format!("{feature}.json"));
    if let Ok(text) = std::fs::read_to_string(&lane_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str(&text) {
            if let Some(Value::Object(p)) = m.get("approved_cell_packet").or_else(|| m.get("gate_preview")) {
                return Some(p.clone());
            }
        }
    }
    let state_file = root.join(".bee").join("state.json");
    if let Ok(text) = std::fs::read_to_string(&state_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str(&text) {
            if m.get("feature").map(js_disp).as_deref() == Some(feature) {
                if let Some(Value::Object(p)) = m.get("approved_cell_packet").or_else(|| m.get("gate_preview")) {
                    return Some(p.clone());
                }
            }
        }
    }
    None
}

fn get_approved_preview_cells(root: &Path, feature: &str) -> Option<Vec<Value>> {
    let packet = get_approved_preview_packet(root, feature)?;
    match packet.get("cells") {
        Some(Value::Array(cells)) => Some(cells.clone()),
        _ => None,
    }
}

/// Verify that an incoming cell being added matches the approved preview packet.
pub(crate) fn check_cell_matches_approved_preview(
    root: &Path,
    feature: &str,
    cell: &Value,
) -> Result<(), String> {
    let Some(approved_packet) = get_approved_preview_packet(root, feature) else {
        return Ok(()); // no approved preview (e.g. tiny/docs lane)
    };
    let Some(Value::Array(approved_cells)) = approved_packet.get("cells") else {
        return Ok(());
    };

    let plan_path = advisor_plan_path(root, feature);
    if plan_path.exists() {
        let bytes = match std::fs::read(&plan_path) {
            Ok(b) => b,
            Err(e) => {
                return Err(format!(
                    "addCells: could not read plan.md ({})",
                    io_read_reason(&plan_path, &e)
                ));
            }
        };
        let current_sha256 = compute_plan_sha256(&bytes);
        if let Some(approved_sha256) = approved_packet.get("plan_sha256").and_then(Value::as_str) {
            if !approved_sha256.is_empty() && approved_sha256 != current_sha256 {
                return Err(format!(
                    "addCells: approved gate preview is stale for feature \"{feature}\" (plan.md changed since preview was approved). FIX: run `bee gate --preview` and re-approve the shape gate before adding cells."
                ));
            }
        }
    }

    let Some(id) = cell.get("id").and_then(|v| v.as_str()) else {
        return Ok(());
    };
    let Some(approved) = approved_cells
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(id))
    else {
        return Err(format!(
            "addCells: cell \"{id}\" was not declared in the approved gate preview packet."
        ));
    };

    let inc_action = cell.get("action").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    let app_action = approved.get("action").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if inc_action != app_action {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (action mismatch)."
        ));
    }

    let inc_verify = cell.get("verify").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    let app_verify = approved.get("verify").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if inc_verify != app_verify {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (verify mismatch)."
        ));
    }

    let inc_files = cell.get("files");
    let app_files = approved.get("files");
    if inc_files != app_files {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (files mismatch)."
        ));
    }

    let inc_rf = cell.get("read_first");
    let app_rf = approved.get("read_first");
    if inc_rf != app_rf {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (read_first mismatch)."
        ));
    }

    let inc_mh = cell.get("must_haves");
    let app_mh = approved.get("must_haves");
    if inc_mh != app_mh {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (must_haves mismatch)."
        ));
    }

    let inc_title = cell.get("title").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    let app_title = approved.get("title").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if inc_title != app_title {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (title mismatch)."
        ));
    }

    let inc_lane = cell.get("lane").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    let app_lane = approved.get("lane").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if inc_lane != app_lane {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (lane mismatch)."
        ));
    }

    let inc_role = cell.get("role").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    let app_role = approved.get("role").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if inc_role != app_role {
        return Err(format!(
            "addCells: cell \"{id}\" differs from approved preview packet (role mismatch)."
        ));
    }

    Ok(())
}

/// Verify that an incoming batch array matches the approved preview packet.
pub(crate) fn check_batch_matches_approved_preview(
    root: &Path,
    feature: &str,
    cells: &[Value],
) -> Result<(), String> {
    let Some(approved_cells) = get_approved_preview_cells(root, feature) else {
        return Ok(());
    };
    if cells.len() != approved_cells.len() {
        return Err(format!(
            "addCells: batch contains {} cell(s), but approved gate preview packet declared {} cell(s).",
            cells.len(),
            approved_cells.len()
        ));
    }
    for cell in cells {
        check_cell_matches_approved_preview(root, feature, cell)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn write_plan(root: &Path, feature: &str, content: &str) -> PathBuf {
        let dir = root.join("docs").join("history").join(feature);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("plan.md");
        std::fs::write(&path, content).unwrap();
        path
    }

    fn sample_plan_with_cells(feature: &str, cells_json: &str) -> String {
        format!(
            "# Plan: {feature}\n\n## Load-bearing claims\n\n| # | Claim | Label | Anchor | Verbatim evidence |\n|---|---|---|---|---|\n| 1 | c | read | src/lib.rs:1 | fn test |\n\n## Cells, current slice preview\n\n```json\n{cells_json}\n```\n\n## Open Questions\n\n(none)\n"
        )
    }

    #[test]
    fn pihp_gate_packet_preview_renders_action_files_read_first_must_haves_and_exact_verify() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("lib.rs"), "fn test() {}\n").unwrap();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"planning","feature":"demo-feat","mode":"standard","approved_gates":{"shape":false,"execution":false}}"#,
        );
        let cells = json!([
            {
                "id": "c1",
                "feature": "demo-feat",
                "title": "Do thing 1",
                "lane": "standard",
                "role": "code",
                "action": "Implement the core parser",
                "files": ["src/parser.rs"],
                "read_first": ["docs/spec.md"],
                "must_haves": {
                    "truths": ["Parser handles all tokens"]
                },
                "verify": "cargo test -p bee parse_test"
            }
        ]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&cells).unwrap()));

        let flags = parse_flags(&["--no-lane"]).unwrap().0;
        let out = run_gate_preview_body(root, &flags, false).unwrap();
        let Out::Emit(val, text, code) = out else {
            panic!("expected Out::Emit");
        };
        assert_eq!(code, 0);
        assert!(text.contains("Implement the core parser"), "shows action: {text}");
        assert!(text.contains("src/parser.rs"), "shows files: {text}");
        assert!(text.contains("docs/spec.md"), "shows read_first: {text}");
        assert!(text.contains("Parser handles all tokens"), "shows must_haves: {text}");
        assert!(text.contains("cargo test -p bee parse_test"), "shows exact verify: {text}");

        assert_eq!(val["feature"], json!("demo-feat"));
        assert!(val["plan_sha256"].is_string());
        assert_eq!(val["cells"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn pihp_gate_packet_refuses_zero_cells_and_preserves_one_and_seven_cells() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("lib.rs"), "fn test() {}\n").unwrap();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"planning","feature":"demo-feat","mode":"standard"}"#,
        );

        // Zero cells: refuses
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", "[]"));
        let flags = parse_flags(&["--no-lane"]).unwrap().0;
        let out = run_gate_preview_body(root, &flags, false).unwrap();
        let Out::Thrown(err) = out else {
            panic!("zero cells must refuse");
        };
        assert!(err.contains("zero cells"), "{err}");

        // One cell: succeeds
        let one_cell = json!([
            {
                "id": "c1",
                "feature": "demo-feat",
                "title": "Title 1",
                "lane": "standard",
                "role": "code",
                "action": "Action 1",
                "files": ["src/f1.rs"],
                "read_first": [],
                "must_haves": { "truths": ["truth 1"] },
                "verify": "cargo test -p bee t1"
            }
        ]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&one_cell).unwrap()));
        let out = run_gate_preview_body(root, &flags, false).unwrap();
        let Out::Emit(val, _, _) = out else {
            panic!("one cell must succeed");
        };
        assert_eq!(val["cells"].as_array().unwrap().len(), 1);

        // Seven cells: succeeds and preserves every packet
        let mut seven_cells = Vec::new();
        for i in 1..=7 {
            seven_cells.push(json!({
                "id": format!("c{i}"),
                "feature": "demo-feat",
                "title": format!("Title {i}"),
                "lane": "standard",
                "role": "code",
                "action": format!("Action {i}"),
                "files": [format!("src/f{i}.rs")],
                "read_first": [],
                "must_haves": { "truths": [format!("truth {i}")] },
                "verify": format!("cargo test -p bee t{i}")
            }));
        }
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&seven_cells).unwrap()));
        let out = run_gate_preview_body(root, &flags, false).unwrap();
        let Out::Emit(val, _, _) = out else {
            panic!("seven cells must succeed");
        };
        assert_eq!(val["cells"].as_array().unwrap().len(), 7);
        for i in 0..7 {
            assert_eq!(val["cells"][i]["id"], format!("c{}", i + 1));
        }
    }

    #[test]
    fn pihp_gate_packet_refuses_missing_execution_field() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("lib.rs"), "fn test() {}\n").unwrap();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"planning","feature":"demo-feat","mode":"standard"}"#,
        );
        let flags = parse_flags(&["--no-lane"]).unwrap().0;

        // Missing verify
        let no_verify = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&no_verify).unwrap()));
        let Out::Thrown(err) = run_gate_preview_body(root, &flags, false).unwrap() else { panic!() };
        assert!(err.contains("missing required field \"verify\""), "{err}");

        // Missing files
        let no_files = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": [], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&no_files).unwrap()));
        let Out::Thrown(err) = run_gate_preview_body(root, &flags, false).unwrap() else { panic!() };
        assert!(err.contains("missing required field \"files\""), "{err}");

        // Missing action
        let no_action = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t", "lane": "standard", "role": "code",
            "action": "", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&no_action).unwrap()));
        let Out::Thrown(err) = run_gate_preview_body(root, &flags, false).unwrap() else { panic!() };
        assert!(err.contains("missing required field \"action\""), "{err}");

        // Missing must_haves.truths on standard lane
        let no_truths = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": ["src/lib.rs"], "read_first": [], "must_haves": {}, "verify": "cargo test -p bee"
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&no_truths).unwrap()));
        let Out::Thrown(err) = run_gate_preview_body(root, &flags, false).unwrap() else { panic!() };
        assert!(err.contains("requires non-empty must_haves.truths"), "{err}");
    }

    #[test]
    fn pihp_gate_packet_shape_approval_refuses_missing_and_stale_preview() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("lib.rs"), "fn test() {}\n").unwrap();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"planning","feature":"demo-feat","mode":"standard","approved_gates":{"shape":false,"execution":false}}"#,
        );
        let cell_pkt = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&cell_pkt).unwrap()));

        // 1. Approval without preview -> refuses missing preview
        let gate_flags = parse_flags(&["--no-lane", "--name", "shape", "--approved", "true"]).unwrap().0;
        let out = run_gate_body(root, &gate_flags).unwrap();
        let Out::Thrown(err) = out else { panic!("missing preview must refuse shape approval") };
        assert!(err.contains("missing gate preview"), "{err}");

        // 2. Run preview -> succeeds
        let preview_flags = parse_flags(&["--no-lane"]).unwrap().0;
        let out = run_gate_preview_body(root, &preview_flags, false).unwrap();
        let Out::Emit(..) = out else { panic!("preview must succeed") };

        // 3. Approval now succeeds
        let out = run_gate_body(root, &gate_flags).unwrap();
        let Out::Emit(..) = out else { panic!("approval must succeed after preview") };

        // 4. Modify plan -> preview becomes stale
        let cell_pkt2 = json!([{
            "id": "c1", "feature": "demo-feat", "title": "t modified", "lane": "standard", "role": "code",
            "action": "a modified", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        write_plan(root, "demo-feat", &sample_plan_with_cells("demo-feat", &serde_json::to_string(&cell_pkt2).unwrap()));

        // Approval refuses stale preview
        let out = run_gate_body(root, &gate_flags).unwrap();
        let Out::Thrown(err) = out else { panic!("stale preview must refuse shape approval") };
        assert!(err.contains("stale gate preview"), "{err}");

        // 5. Re-run preview -> approval succeeds again
        let out = run_gate_preview_body(root, &preview_flags, false).unwrap();
        let Out::Emit(..) = out else { panic!("re-preview must succeed") };
        let out = run_gate_body(root, &gate_flags).unwrap();
        let Out::Emit(..) = out else { panic!("approval must succeed after re-preview") };
    }

    #[test]
    fn pihp_gate_packet_gate_and_advisor_read_feature_worktree_plan() {
        let tmp = tmp_root();
        let main_root = tmp.path().join("main");
        let wt_root = tmp.path().join("wt-feat");
        std::fs::create_dir_all(&main_root).unwrap();
        std::fs::create_dir_all(&wt_root).unwrap();

        // Main store has worktree grants and git setup
        std::fs::create_dir_all(main_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main_root.join(".bee").join("runtime").join("worktree-grants.json"),
            r#"{"wt-feat": true}"#,
        ).unwrap();

        // git worktrees dir in main
        let git_wt = main_root.join(".git").join("worktrees").join("wt-feat");
        std::fs::create_dir_all(&git_wt).unwrap();
        let wt_git_file = wt_root.join(".git");
        std::fs::write(&wt_git_file, format!("gitdir: {}\n", git_wt.display())).unwrap();
        std::fs::write(git_wt.join("gitdir"), format!("{}\n", wt_git_file.display())).unwrap();

        // Worktree identity in wt_root
        std::fs::create_dir_all(wt_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wt_root.join(".bee").join("runtime").join("worktree-identity.json"),
            r#"{"feature":"feat-x"}"#,
        ).unwrap();

        // Main checkout has NO plan.md for feat-x
        // Worktree HAS plan.md for feat-x
        let cell_pkt = json!([{
            "id": "c1", "feature": "feat-x", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        let wt_plan_path = write_plan(&wt_root, "feat-x", &sample_plan_with_cells("feat-x", &serde_json::to_string(&cell_pkt).unwrap()));

        // advisor_plan_path resolves the worktree plan
        let resolved = advisor_plan_path(&main_root, "feat-x");
        assert_eq!(resolved, wt_plan_path, "advisor_plan_path must resolve to worktree plan");

        // advisor_ref_anchors hashes the worktree plan
        let anchors = advisor_ref_anchors(&main_root, &json!("feat-x"));
        assert_ne!(anchors["plan_sha256"], json!(ADVISOR_PLAN_ABSENT_SENTINEL));
    }

    #[test]
    fn pihp_gate_packet_gate_and_advisor_resolve_worktree_plan_when_both_main_and_worktree_plans_exist() {
        let tmp = tmp_root();
        let main_root = tmp.path().join("main");
        let wt_root = tmp.path().join("wt-feat");
        std::fs::create_dir_all(&main_root).unwrap();
        std::fs::create_dir_all(&wt_root).unwrap();

        std::fs::create_dir_all(main_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main_root.join(".bee").join("runtime").join("worktree-grants.json"),
            r#"{"wt-feat": true}"#,
        ).unwrap();

        let git_wt = main_root.join(".git").join("worktrees").join("wt-feat");
        std::fs::create_dir_all(&git_wt).unwrap();
        let wt_git_file = wt_root.join(".git");
        std::fs::write(&wt_git_file, format!("gitdir: {}\n", git_wt.display())).unwrap();
        std::fs::write(git_wt.join("gitdir"), format!("{}\n", wt_git_file.display())).unwrap();

        std::fs::create_dir_all(wt_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wt_root.join(".bee").join("runtime").join("worktree-identity.json"),
            r#"{"feature":"feat-x"}"#,
        ).unwrap();

        // Both main checkout and worktree have plan.md for feat-x with different contents
        let main_plan_path = write_plan(&main_root, "feat-x", "# Main Plan (Stale)\n");
        let cell_pkt = json!([{
            "id": "c1", "feature": "feat-x", "title": "t", "lane": "standard", "role": "code",
            "action": "a", "files": ["src/lib.rs"], "read_first": [], "must_haves": { "truths": ["t"] }, "verify": "cargo test -p bee"
        }]);
        let wt_plan_path = write_plan(&wt_root, "feat-x", &sample_plan_with_cells("feat-x", &serde_json::to_string(&cell_pkt).unwrap()));

        assert!(main_plan_path.exists());
        assert!(wt_plan_path.exists());

        // advisor_plan_path MUST resolve to the worktree plan, NEVER main checkout plan
        let resolved = advisor_plan_path(&main_root, "feat-x");
        assert_eq!(resolved, wt_plan_path, "advisor_plan_path must resolve to worktree plan even when main plan also exists");
        assert_ne!(resolved, main_plan_path);

        // advisor_ref_anchors hashes the worktree plan, not the main plan
        let wt_bytes = std::fs::read(&wt_plan_path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&wt_bytes);
        let expected_wt_sha = format!("{:x}", hasher.finalize());

        let anchors = advisor_ref_anchors(&main_root, &json!("feat-x"));
        assert_eq!(anchors["plan_sha256"], json!(expected_wt_sha));
    }

    #[test]
    fn pihp_gate_packet_high_risk_approval_refuses_missing_plan() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"planning","feature":"feat-high","mode":"high-risk","approved_gates":{"shape":false,"execution":false}}"#,
        );
        // NO plan.md is created
        let gate_flags = parse_flags(&["--no-lane", "--name", "shape", "--approved", "true"]).unwrap().0;
        let out = run_gate_body(root, &gate_flags).unwrap();
        let Out::Thrown(err) = out else { panic!("missing plan.md in high-risk lane must refuse shape approval") };
        assert!(err.contains("plan.md does not exist"), "{err}");
        assert!(err.contains("feat-high"), "{err}");
    }
}
