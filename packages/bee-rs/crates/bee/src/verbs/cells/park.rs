use super::*;
use crate::lock;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::Out;
use serde_json::{Map, Value};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const PARK_KEY: &str = "park";

pub(crate) fn answer_remedy(id: &str) -> String {
    format!("bee cells answer --id {id} --answer \"<text>\"")
}

pub(crate) fn has_open_question(trace: Option<&Value>) -> bool {
    trace
        .and_then(|t| t.get(PARK_KEY))
        .and_then(Value::as_array)
        .and_then(|a| a.last())
        .is_some_and(|e| entry_text(e, "answer").is_none())
}

pub(crate) fn refuse_parked(verb: &str, id: &str, status: Option<&Value>, trace: Option<&Value>) -> MR<()> {
    let parked = matches!(status, Some(Value::String(s)) if s == "parked");
    if !parked && !has_open_question(trace) {
        return Ok(());
    }
    Err(Fail::Thrown(format!(
        "{verb}: cell \"{id}\" is \"{}\" with its parked question unanswered — only an answer moves it on. Run: {}",
        js_string_or_undefined(status),
        answer_remedy(id)
    )))
}

fn entry_text(entry: &Value, key: &str) -> Option<String> {
    match entry.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn park_entries(cell: &Value) -> Vec<Value> {
    match cell.get("trace").and_then(|t| t.get(PARK_KEY)) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    }
}

fn open_question(cell: &Value) -> Option<String> {
    park_entries(cell)
        .last()
        .filter(|e| entry_text(e, "answer").is_none())
        .and_then(|e| entry_text(e, "question"))
}

pub(crate) fn park_answers(cell: &Value) -> Vec<(String, String)> {
    park_entries(cell)
        .iter()
        .filter_map(|e| Some((entry_text(e, "question")?, entry_text(e, "answer")?)))
        .collect()
}

pub(crate) fn feature_parked_cells(
    root: &Path,
    feature: &str,
) -> Result<Vec<(String, String)>, crate::verbs::drivers::Delegate> {
    let cells = crate::verbs::drivers::list_cells_including_archive(root, feature, Some("parked"))?;
    Ok(cells
        .iter()
        .filter_map(|c| Some((entry_text(c, "id")?, open_question(c).unwrap_or_default())))
        .collect())
}

pub(crate) fn parked_cells(root: &Path) -> Vec<(String, String)> {
    let live = list_cells(root, None, Some("parked")).unwrap_or_default();
    let archived = std::fs::read_dir(cells_dir(root).join(ARCHIVE_DIR_NAME))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().to_str().map(str::to_string));
    let mut features: Vec<String> = live.iter().filter_map(|c| entry_text(c, "feature")).chain(archived).collect();
    features.sort();
    features.dedup();
    features
        .iter()
        .flat_map(|f| feature_parked_cells(root, f).unwrap_or_default())
        .collect()
}

pub(crate) fn run_park(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "question", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let question = flags.req_str("question")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells park", use_json, t0, move |ctx| {
        let cell = park_cell(&ctx.root, &id, &question, session_flag.as_deref(), force)?;
        let text = format!("Parked {id} on a question. Answer it with: {}", answer_remedy(&id));
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn park_cell(
    root: &Path,
    id: &str,
    question: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Value> {
    let question = js_trim(question).to_string();
    if question.is_empty() {
        return Err(Fail::Thrown("parkCell: a question is required.".into()));
    }
    assert_safe_decision_fields(&[("question", Some(&question))])?;
    let root2 = root.to_path_buf();
    let id2 = id.to_string();
    let session_owned = session_flag.map(str::to_string);
    let cell = mutate_cell(root, id, "parkCell", Some("parkCell"), true, move |cell_map| {
        let status = js_string_or_undefined(cell_map.get("status"));
        if status == "parked" {
            return Err(Fail::Thrown(format!(
                "parkCell: cell \"{id2}\" is already \"parked\" on a question. Run: {}",
                answer_remedy(&id2)
            )));
        }
        if status != "open" && status != "claimed" {
            return Err(Fail::Thrown(format!(
                "parkCell: cell \"{id2}\" is \"{status}\" — only an open or claimed cell can be parked."
            )));
        }
        let trace = merge_trace(cell_map.get("trace"))?;
        let mut trace = release_trace(guard_claim_ownership(
            &root2,
            &id2,
            trace,
            "parkCell",
            session_owned.as_deref(),
            force,
        )?);
        let mut entries: Vec<Value> = match trace.get(PARK_KEY) {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let mut entry = Map::new();
        entry.insert("question".into(), Value::String(question.clone()));
        entry.insert("parked_at".into(), Value::String(utc_now()));
        entry.insert("answer".into(), Value::Null);
        entry.insert("answer_decision".into(), Value::Null);
        entry.insert("answered_at".into(), Value::Null);
        entries.push(Value::Object(entry));
        trace.insert(PARK_KEY.into(), Value::Array(entries));
        cell_map.insert("status".into(), Value::String("parked".into()));
        cell_map.insert("trace".into(), Value::Object(trace));
        Ok(())
    })?;
    release_reservations_for_unclaimed_cell(root, id);
    Ok(cell)
}

pub(crate) fn run_answer(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "answer"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let answer = flags.req_str("answer")?.to_string();
    dispatch("cells answer", use_json, t0, move |ctx| {
        let cell = answer_cell(&ctx.root, &id, &answer)?;
        let text = format!(
            "Answered the parked question on {id} — the cell is {}.",
            js_string_or_undefined(cell.get("status"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn answer_cell(root: &Path, id: &str, answer: &str) -> MR<Value> {
    let answer = js_trim(answer).to_string();
    if answer.is_empty() {
        return Err(Fail::Thrown("answerCell: an answer is required.".into()));
    }
    assert_safe_decision_fields(&[("answer", Some(&answer))])?;
    let root2 = root.to_path_buf();
    let id2 = id.to_string();
    let cell = mutate_cell(root, id, "answerCell", Some("answerCell"), false, move |cell_map| {
        let status = js_string_or_undefined(cell_map.get("status"));
        if status != "parked" && !has_open_question(cell_map.get("trace")) {
            return Err(Fail::Thrown(format!(
                "answerCell: cell \"{id2}\" is \"{status}\", not \"parked\" — only a parked cell takes an answer."
            )));
        }
        let mut trace = merge_trace(cell_map.get("trace"))?;
        let mut entries: Vec<Value> = match trace.get(PARK_KEY) {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let Some(Value::Object(mut entry)) = entries.pop().filter(|e| entry_text(e, "answer").is_none()) else {
            return Err(Fail::Thrown(format!(
                "answerCell: cell \"{id2}\" is \"parked\" but carries no open question — nothing to answer."
            )));
        };
        let question = entry.get("question").and_then(Value::as_str).unwrap_or_default().to_string();
        if let lock::AcquireOnce::Busy { holder } = lock::acquire_store_lock_once(&root2, "cells-archive") {
            return Err(Fail::Thrown(format!(
                "answerCell: cell \"{id2}\" not answered — the \"cells-archive\" lock is held by {} (a live archive/unarchive transaction). Nothing was logged; retry once it completes.",
                holder_who(&holder)
            )));
        }
        let decision_id = log_decision_id(
            &root2,
            &format!("cells answer: the parked question on cell \"{id2}\" is answered. Question: \"{question}\" Answer: \"{answer}\""),
            "D2 (8ed6f4a2): an answer to a parked cell is logged as a decision the cell cites, and the cell returns to open.",
            &["cells", "slp"],
        )?;
        entry.insert("answer".into(), Value::String(answer.clone()));
        entry.insert("answer_decision".into(), Value::String(decision_id.clone()));
        entry.insert("answered_at".into(), Value::String(utc_now()));
        entries.push(Value::Object(entry));
        trace.insert(PARK_KEY.into(), Value::Array(entries));
        let mut decisions = match cell_map.get("decisions") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        decisions.push(Value::String(decision_id));
        cell_map.insert("decisions".into(), Value::Array(decisions));
        if status == "parked" {
            cell_map.insert("status".into(), Value::String("open".into()));
        }
        cell_map.insert("trace".into(), Value::Object(trace));
        Ok(())
    })?;
    if cell.get("status").and_then(Value::as_str) == Some("open") {
        clear_merge_ready_for(root, &cell);
    }
    Ok(cell)
}
