use super::{control_root, list_claimed_cells, read_json_failopen, read_jsonl};
use crate::fsutil::ReadJson;
use crate::hooks::adapter::{now_iso, HookContext};
use crate::verbs::state_group::{advisor_ref_anchors, advisor_ref_stale};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const DEFAULT_SCOPE_WORDS: &[&str] = &[
    "change the",
    "instead of",
    "thay vì",
    "đổi sang",
    "bỏ đi",
    "scope",
    "also add",
    "stop doing",
    "do not",
    "don't",
    "remove",
    "rename",
    "switch to",
    "thêm vào",
];

pub(crate) fn record_scope_input(root: &Path, ctx: &HookContext, text: &str) {
    let off = crate::hooks::herding_worker_marker_set()
        || crate::state::read_config_raw(root).get("pi_harness_workflow") == Some(&Value::Bool(false));
    if off {
        return;
    }
    let config = crate::state::read_config_raw(root);
    let words_vec: Vec<String>;
    let words: Vec<&str> = match config.get("pi_scope_words") {
        Some(Value::Array(arr)) => {
            words_vec = arr.iter().filter_map(Value::as_str).map(String::from).collect();
            words_vec.iter().map(String::as_str).collect()
        }
        _ => DEFAULT_SCOPE_WORDS.to_vec(),
    };
    if words.is_empty() || !matches_scope_words(text, &words) {
        return;
    }
    let ctl = control_root(root, ctx);
    let feature = resolve_feature(root, &ctl);
    let entry = json!({
        "text": text,
        "time": now_iso(),
        "feature": feature,
    });
    let _ = prune_and_write_scope_inputs(root, &ctl, Some(entry));
}

fn matches_scope_words(text: &str, words: &[&str]) -> bool {
    let text_lower = text.to_lowercase();
    for word in words {
        let w_lower = word.to_lowercase();
        if w_lower.is_empty() {
            continue;
        }
        for (start, matched) in text_lower.match_indices(&w_lower) {
            let end = start + matched.len();
            let left_ok = start == 0 || text_lower[..start].chars().next_back().map_or(true, |c| !c.is_alphanumeric());
            let right_ok = end == text_lower.len() || text_lower[end..].chars().next().map_or(true, |c| !c.is_alphanumeric());
            if left_ok && right_ok {
                return true;
            }
        }
    }
    false
}

fn text_sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn resolve_feature(root: &Path, ctl: &Path) -> String {
    for base in [root, ctl] {
        let state_file = base.join(".bee").join("state.json");
        if let ReadJson::Parsed(Value::Object(m)) = read_json_failopen(&state_file) {
            if let Some(f) = m.get("feature").and_then(Value::as_str) {
                if !f.is_empty() {
                    return f.to_string();
                }
            }
        }
    }
    for base in [root, ctl] {
        for cell in list_claimed_cells(base).unwrap_or_default() {
            if let Some(f) = cell.get("feature").and_then(Value::as_str) {
                if !f.is_empty() {
                    return f.to_string();
                }
            }
        }
    }
    String::new()
}

fn has_decision_after(root: &Path, ctl: &Path, input_time: &str) -> bool {
    let file = if ctl.join(".bee").join("decisions.jsonl").exists() {
        ctl.join(".bee").join("decisions.jsonl")
    } else {
        root.join(".bee").join("decisions.jsonl")
    };
    let events = read_jsonl(&file);
    let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
    for event in &events {
        if event.get("type").and_then(Value::as_str) == Some("supersede") {
            if let Some(target) = event.get("supersedes") {
                if let Some(k) = super::primitive_key(target) {
                    superseded.insert(k);
                }
            }
        }
        if event.get("type").and_then(Value::as_str) == Some("redact") {
            if let Some(target) = event.get("redacts") {
                if let Some(k) = super::primitive_key(target) {
                    redacted.insert(k);
                }
            }
        }
    }
    for event in &events {
        let ty = event.get("type").and_then(Value::as_str);
        if !matches!(ty, Some("decide") | Some("supersede")) {
            continue;
        }
        let id_key = event.get("id").and_then(super::primitive_key);
        if let Some(k) = &id_key {
            if superseded.contains(k) || redacted.contains(k) {
                continue;
            }
        }
        if let Some(date) = event.get("date").and_then(Value::as_str) {
            if date > input_time {
                return true;
            }
        }
    }
    false
}

fn prune_and_write_scope_inputs(
    root: &Path,
    ctl: &Path,
    new_input: Option<Value>,
) -> std::io::Result<()> {
    let file = if root.join(".bee").join("runtime").join("scope-inputs.jsonl").exists() {
        root.join(".bee").join("runtime").join("scope-inputs.jsonl")
    } else {
        ctl.join(".bee").join("runtime").join("scope-inputs.jsonl")
    };
    let served_file = ctl.join(".bee").join("runtime").join("settle-obligations.json");
    let served = read_served(&served_file);
    let current_feature = resolve_feature(root, ctl);

    let mut kept = Vec::new();
    if file.exists() {
        for line in read_jsonl(&file) {
            let Some(text) = line.get("text").and_then(Value::as_str) else { continue };
            let time = line.get("time").or_else(|| line.get("recorded_at")).and_then(Value::as_str).unwrap_or("");
            let feat = line.get("feature").and_then(Value::as_str).filter(|f| !f.is_empty()).unwrap_or(&current_feature);
            let sha = text_sha256(text);
            let key = format!("{feat}:scope:{sha}");
            if served.contains_key(&key) {
                continue;
            }
            if !time.is_empty() && has_decision_after(root, ctl, time) {
                continue;
            }
            kept.push(line);
        }
    }
    if let Some(entry) = new_input {
        kept.push(entry);
    }
    let runtime_dir = file.parent().unwrap_or(ctl);
    std::fs::create_dir_all(runtime_dir)?;
    let mut content = String::new();
    for entry in kept {
        content.push_str(&serde_json::to_string(&entry).unwrap_or_default());
        content.push('\n');
    }
    crate::fsutil::write_text_atomic(&file, &content)
}

pub(crate) fn answer(root: &Path, ctx: &HookContext) -> Option<String> {
    let skip_key = ctx.payload.get("skip_key").and_then(Value::as_str);
    let obligations_only = ctx.payload.get("obligations_only") == Some(&Value::Bool(true));
    if skip_key.is_none() && !obligations_only {
        return None;
    }
    let off = crate::hooks::herding_worker_marker_set()
        || crate::state::read_config_raw(root).get("pi_harness_workflow") == Some(&Value::Bool(false));
    let ctl = control_root(root, ctx);
    let file = ctl.join(".bee").join("runtime").join("settle-obligations.json");
    if let Some(key) = skip_key {
        if !off {
            let _ = mark_served(&file, &[key.to_string()]);
            let _ = prune_and_write_scope_inputs(root, &ctl, None);
        }
        return Some(String::new());
    }
    let owed = if off { Vec::new() } else { unserved(root, &ctl, &file) };
    let keys: Vec<String> = owed.iter().filter_map(|o| o["key"].as_str().map(String::from)).collect();
    let owed = if keys.is_empty() || mark_served(&file, &keys).is_ok() { owed } else { Vec::new() };
    if !keys.is_empty() {
        let _ = prune_and_write_scope_inputs(root, &ctl, None);
    }
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

fn unserved(root: &Path, ctl: &Path, file: &Path) -> Vec<Value> {
    let served = read_served(file);
    let mut owed: Vec<Value> = Vec::new();
    for cell in list_claimed_cells(root).unwrap_or_default() {
        let Some(id) = cell.get("id").and_then(Value::as_str) else { continue };
        if is_cell_worker_running(root, id) {
            continue;
        }
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
Run the plan-step hat wave now: 5 runs, one bee_advisor call each for the five hat seats \
(hat-facts-gaps, hat-risks, hat-value, hat-alternatives, hat-user-impact) via bee dispatch prepare --kind advisor, \
synthesize it, then record it: bee state advisor-ref record --advisor \"<identity>\" --digest-file <path>. \
Do not approve the gate; the user does.",
                staleness.reasons.join("; ")
            ),
            "user_notice": format!(
                "bee: the plan for {feature} is ready for its gate but has no fresh advisor review. \
One extra turn starts now to run the advisor (5 runs, one bee_advisor call each: \
hat-facts-gaps, hat-risks, hat-value, hat-alternatives, hat-user-impact). \
(Esc stops it. This message will not repeat for this plan.)"
            ),
        }));
    }
    let scope_file = if root.join(".bee").join("runtime").join("scope-inputs.jsonl").exists() {
        root.join(".bee").join("runtime").join("scope-inputs.jsonl")
    } else {
        ctl.join(".bee").join("runtime").join("scope-inputs.jsonl")
    };
    let current_feature = resolve_feature(root, ctl);
    let mut unserved_scope: Vec<(String, String)> = Vec::new();
    if scope_file.exists() {
        for line in read_jsonl(&scope_file) {
            let Some(text) = line.get("text").and_then(Value::as_str) else { continue };
            let time = line.get("time").or_else(|| line.get("recorded_at")).and_then(Value::as_str).unwrap_or("");
            let feat = line.get("feature").and_then(Value::as_str).filter(|f| !f.is_empty()).unwrap_or(&current_feature);
            let sha = text_sha256(text);
            let key = format!("{feat}:scope:{sha}");
            if served.contains_key(&key) {
                continue;
            }
            if !time.is_empty() && has_decision_after(root, ctl, time) {
                continue;
            }
            unserved_scope.push((key, text.to_string()));
        }
    }
    if let Some((key, text)) = unserved_scope.last() {
        owed.push(json!({
            "key": key,
            "kind": "scope",
            "text": text,
            "message": format!(
                "bee: mid-run input reads as a scope change ({text:?}). \
Log it with bee decisions log or ask the user: \
a change to an approved plan reopens the gate and only the user answers it. \
For a false match, skip it: /bee-obligation-skip {key}"
            ),
            "user_notice": format!(
                "bee: typed input reads as a scope change to an approved plan. \
One extra turn starts now to log it or ask the user. \
(Esc stops it. This message will not repeat for this input.)"
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

fn is_cell_worker_running(root: &Path, cell_id: &str) -> bool {
    has_running_registered_worker(root, cell_id)
}

fn has_running_registered_worker(root: &Path, cell_id: &str) -> bool {
    let state_file = root.join(".bee").join("state.json");
    let ReadJson::Parsed(Value::Object(state)) = read_json_failopen(&state_file) else {
        return false;
    };
    let Some(Value::Array(workers)) = state.get("workers") else {
        return false;
    };
    let mailbox_dir = root.join(".bee").join("mailbox");
    for w in workers {
        if matches!(w.get("status"), Some(Value::String(s)) if s == "capped") {
            continue;
        }
        if w.get("cell").and_then(Value::as_str) != Some(cell_id) {
            continue;
        }
        let nickname = w.get("nickname").and_then(Value::as_str);
        if let Some(nick) = nickname {
            let job_dir = mailbox_dir.join(nick);
            if job_dir.is_dir() && !has_result_file(&job_dir) {
                return true;
            }
        }
        if mailbox_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&mailbox_dir) {
                for entry in entries.flatten() {
                    let Ok(ft) = entry.file_type() else { continue };
                    if !ft.is_dir() {
                        continue;
                    }
                    let job_path = entry.path();
                    if job_dir_matches_worker_or_cell(&job_path, nickname, cell_id) && !has_result_file(&job_path) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn has_result_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(digits) = name.strip_prefix("result-").and_then(|s| s.strip_suffix(".json")) {
            if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
}

fn job_dir_matches_worker_or_cell(dir: &Path, nickname: Option<&str>, cell_id: &str) -> bool {
    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if let Some(nick) = nickname {
        if dir_name == nick {
            return true;
        }
    }
    let job_file = dir.join("job.json");
    if let ReadJson::Parsed(Value::Object(job)) = read_json_failopen(&job_file) {
        if job.get("cell_id").and_then(Value::as_str) == Some(cell_id) {
            return true;
        }
        if let Some(nick) = nickname {
            if job.get("job_id").and_then(Value::as_str) == Some(nick) {
                return true;
            }
        }
        if let Some(task) = job.get("task").and_then(Value::as_str) {
            if task.contains(&format!("cell: {cell_id}"))
                || task.contains(&format!("\"cell_id\": \"{cell_id}\""))
                || task.contains(&format!("Assigned cell id: {cell_id}"))
            {
                return true;
            }
        }
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("ack-") && name.ends_with(".json") {
                if let ReadJson::Parsed(Value::Object(ack)) = read_json_failopen(&e.path()) {
                    if ack.get("cell_id").and_then(Value::as_str) == Some(cell_id) {
                        return true;
                    }
                    if let Some(nick) = nickname {
                        if ack.get("nickname").and_then(Value::as_str) == Some(nick) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
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
