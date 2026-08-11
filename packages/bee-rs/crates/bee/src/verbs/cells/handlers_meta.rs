// verb handlers: judge, reset-budget, judge-record, schedule, archive, unarchive
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

// ── cells judge (read-only frozen-judge check) ─────────────────────────────

pub(crate) fn run_judge(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    dispatch("cells judge", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = read_cell_norm(&root, &id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("judgeCell: cell \"{id}\" not found.")));
        };
        let Value::Object(cell_map) = &cell else { return Err(Fail::Delegate) };
        let changed = cell_map
            .get("trace")
            .filter(|t| js_truthy(t))
            .and_then(|t| t.get("files_changed"))
            .cloned()
            .unwrap_or(Value::Null);
        let declared = cell_map.get("files").cloned().unwrap_or(Value::Null);
        let hits = frozen_judge_hits(&changed, &declared);
        let id_disp = js_string_or_undefined(cell_map.get("id"));
        let text = if hits.is_empty() {
            format!("Judge intact for {id_disp}: no undeclared test/CI/lockfile changes.")
        } else {
            format!(
                "FROZEN-JUDGE HITS for {id_disp}: {} — do not count this cell toward a clean wave; flag it for review (decision 0018).",
                hits.iter()
                    .map(|(file, rule)| format!("{file} ({rule})"))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        let mut result = Map::new();
        result.insert("id".into(), cell_map.get("id").cloned().unwrap_or(Value::Null));
        // NOTE: judgeCell returns {id: cell.id, hits}; an absent id would be
        // dropped by JSON.stringify — cells with a string id (the only shape
        // this native path serves) always carry one.
        result.insert(
            "hits".into(),
            Value::Array(
                hits.iter()
                    .map(|(file, rule)| {
                        let mut h = Map::new();
                        h.insert("file".into(), Value::String(file.clone()));
                        h.insert("rule".into(), Value::String(rule.to_string()));
                        Value::Object(h)
                    })
                    .collect(),
            ),
        );
        if !matches!(cell_map.get("id"), Some(Value::String(_))) {
            return Err(Fail::Delegate); // undefined-id JSON shape — Node's
        }
        Ok(Out::Emit(Value::Object(result), text, 0))
    })
}

// ── cells reset-budget ─────────────────────────────────────────────────────

pub(crate) fn run_reset_budget(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "operator", "session-id"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let operator = opt_string_flag(&flags, "operator")?;
    let session_flag = opt_string_flag(&flags, "session-id")?;
    dispatch("cells reset-budget", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("resetCellBudget: a reason is required.".into()));
        }
        let reason_text = js_trim(&reason).to_string();
        let by_session = resolve_session_flag_env(session_flag.as_deref());
        let actor = operator
            .as_deref()
            .filter(|o| !js_trim(o).is_empty())
            .map(|o| js_trim(o).to_string())
            .or_else(|| env_nonempty("BEE_AGENT_NAME"));
        delegate_only(load_taxonomy(&root))?;
        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let saved = (|| -> MR<Value> {
            assert_not_archived(&root, "resetCellBudget", &id)?;
            let cell = read_cell_norm(&root, &id)?;
            let Some(cell) = cell else {
                return Err(Fail::Thrown(format!("resetCellBudget: cell \"{id}\" not found.")));
            };
            let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
            let Some(actor) = actor.clone() else {
                return Err(Fail::Thrown(format!(
                    "resetCellBudget: an actor is required — pass --operator \"<name>\" or set BEE_AGENT_NAME in the environment before resetting cell \"{id}\"'s budget."
                )));
            };
            match check_cell_budgets(&cell_map)? {
                BudgetCheck::Refused { .. } => {}
                BudgetCheck::Ok => {
                    return Err(Fail::Thrown(format!(
                        "resetCellBudget: cell \"{id}\" is not budget-blocked (checkCellBudgets reports ok) — a reset is only needed once the claim door is actually closed by CELL_BUDGET_EXHAUSTED or REPEATED_FAILURE."
                    )));
                }
            }
            let mut trace = merge_trace(cell_map.get("trace"))?;
            let resets: Vec<Value> = match trace.get("budget_resets") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut entry = Map::new();
            entry.insert("reset_at".into(), Value::String(utc_now()));
            entry.insert("reason".into(), Value::String(reason_text.clone()));
            entry.insert(
                "by_session".into(),
                by_session.clone().map(Value::String).unwrap_or(Value::Null),
            );
            entry.insert("by_actor".into(), Value::String(actor.clone()));
            // Audit BEFORE write (D-GHF-C).
            log_decision(
                &root,
                &format!(
                    "«cells reset-budget: cell \"{id}\" claim-lifetime budget reset by {actor} — {reason_text}»"
                ),
                "Audited reopening of a D2 loop-safety door (self-correcting-loop); the attempt ledger itself is never rewritten, only a budget_resets marker appended.",
                &["cells"],
            )?;
            let mut next = resets;
            next.push(Value::Object(entry));
            trace.insert("budget_resets".into(), Value::Array(next));
            cell_map.insert("trace".into(), Value::Object(trace));
            let value = Value::Object(cell_map);
            write_cell(&root, &value)?;
            Ok(value)
        })();
        guard.release();
        let cell = saved?;
        let text = format!(
            "Reset the claim-lifetime budget door for {}.",
            js_string_or_undefined(cell.get("id"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── cells judge-record ─────────────────────────────────────────────────────

pub(crate) fn run_judge_record(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(
        &flags,
        &["id", "file", "builder-model", "judge-model", "session-id", "force-ownership"],
    ) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let file = flags.req_str("file")?.to_string();
    let builder_model = opt_string_flag(&flags, "builder-model")?;
    let judge_model = opt_string_flag(&flags, "judge-model")?;
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells judge-record", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let raw = read_file_text(&file, "judge verdict")?;
        let verdict = match parse_json_js(&raw, false) {
            JsParse::Value(v) => v,
            // free prose — validator rejects it; a lone-surrogate escape is
            // "not JSON this CLI can parse" and takes the same branch.
            JsParse::NotJson => Value::String(raw.clone()),
        };
        let (ok, errors) = validate_judge_verdict(&verdict);
        if !ok {
            return Err(Fail::Thrown(format!(
                "recordJudgeVerdict: cell \"{id}\" verdict rejected against schema \"judge-verdict/1\" — {} FIX: the judge dispatch must return the schema verbatim (never free prose); re-dispatch once, then record model_independence \"unverified\" if it fails again (D5).",
                errors.join(" ")
            )));
        }
        let verdict_map = match &verdict {
            Value::Object(m) => m.clone(),
            _ => unreachable!("validated object"),
        };
        let independence = derive_model_independence(
            builder_model.as_deref(),
            builder_model.as_deref().map(|_| PINNED_MODEL_STATUS),
            judge_model.as_deref(),
            judge_model.as_deref().map(|_| PINNED_MODEL_STATUS),
        );
        prescan_claim(&root, &id)?;
        delegate_only(load_taxonomy(&root))?;
        let mut reopened = false;
        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let saved = (|| -> MR<Value> {
            assert_not_archived(&root, "recordJudgeVerdict", &id)?;
            let cell = read_cell_norm(&root, &id)?;
            let Some(cell) = cell else {
                return Err(Fail::Thrown(format!("recordJudgeVerdict: cell \"{id}\" not found.")));
            };
            let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
            let mut entry = Map::new();
            entry.insert("schema".into(), verdict_map.get("schema").cloned().unwrap_or(Value::Null));
            entry.insert("verdict".into(), verdict_map.get("verdict").cloned().unwrap_or(Value::Null));
            entry.insert("checks".into(), verdict_map.get("checks").cloned().unwrap_or(Value::Null));
            entry.insert(
                "failure_signature".into(),
                match verdict_map.get("failure_signature") {
                    None | Some(Value::Null) => Value::Null, // ?? null
                    Some(v) => v.clone(),
                },
            );
            entry.insert(
                "fixability".into(),
                verdict_map.get("fixability").cloned().unwrap_or(Value::Null),
            );
            entry.insert(
                "confidence".into(),
                verdict_map.get("confidence").cloned().unwrap_or(Value::Null),
            );
            let model_or_null = |m: &Option<String>| match m {
                Some(s) if !js_trim(s).is_empty() => Value::String(s.clone()),
                _ => Value::Null,
            };
            entry.insert("builder_model".into(), model_or_null(&builder_model));
            entry.insert("judge_model".into(), model_or_null(&judge_model));
            entry.insert("model_independence".into(), Value::String(independence.to_string()));
            entry.insert("recorded_at".into(), Value::String(utc_now()));
            let mut trace = merge_trace(cell_map.get("trace"))?;
            trace = guard_claim_ownership(
                &root,
                &id,
                trace,
                "recordJudgeVerdict",
                session_flag.as_deref(),
                force,
            )?;
            let existing: Vec<Value> = match trace.get("semantic_judge") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut next = existing;
            next.push(Value::Object(entry));
            trace.insert("semantic_judge".into(), Value::Array(next));
            let needs_revision =
                matches!(verdict_map.get("verdict"), Some(Value::String(s)) if s == "NEEDS_REVISION");
            let capped = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "capped");
            if needs_revision && capped {
                cell_map.insert("status".into(), Value::String("open".into()));
                let mut rework = Map::new();
                rework.insert("at".into(), Value::String(utc_now()));
                rework.insert(
                    "reason".into(),
                    Value::String("NEEDS_REVISION semantic-judge verdict recorded after cap".into()),
                );
                trace.insert("reopened_for_rework".into(), Value::Object(rework));
                trace = release_trace(trace);
                reopened = true;
                log_decision(
                    &root,
                    &format!(
                        "«cells judge-record: cell \"{id}\" reopened capped->open by a NEEDS_REVISION semantic-judge verdict»"
                    ),
                    "A NEEDS_REVISION verdict recorded after cap must have teeth: the cell is reopened to open (clean slate) for rework, with claim + verify evidence cleared, instead of being silently logged into an inert trace entry (hardening-3) or left falsely \"claimed\" with stale verify_passed that a later PASS verdict could re-cap on with zero fresh verify (hardening-1-7-10 D7).",
                    &["cells", "judge"],
                )?;
            }
            cell_map.insert("trace".into(), Value::Object(trace));
            let value = Value::Object(cell_map);
            write_cell(&root, &value)?;
            Ok(value)
        })();
        guard.release();
        let cell = saved?;
        if reopened {
            release_claim_file_best_effort(&root, &id);
        }
        let entries = cell.get("trace").and_then(|t| t.get("semantic_judge"));
        let latest = match entries {
            Some(Value::Array(a)) => a.last().cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        };
        let text = format!(
            "Recorded judge verdict on {}: {} (model_independence={}).",
            js_string_or_undefined(cell.get("id")),
            js_string_or_undefined(latest.get("verdict")),
            js_string_or_undefined(latest.get("model_independence"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── cells schedule (read-only computed schedule) ───────────────────────────

pub(crate) fn run_schedule(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = flags.truthy_str("feature").map(str::to_string);
    dispatch("cells schedule", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cells = list_cells(&root, feature.as_deref(), None).map_err(|_| Fail::Delegate)?;
        // A schedulable cell with a non-string id takes JS-exotic Map-key
        // paths (undefined keys, undefined-in-sort) — Node's.
        for cell in &cells {
            let schedulable = matches!(cell.get("status"), Some(Value::String(s)) if s == "open" || s == "claimed");
            if schedulable && !matches!(cell.get("id"), Some(Value::String(_))) {
                return Err(Fail::Delegate);
            }
        }
        let schedule = compute_schedule(&cells);
        let mut lines: Vec<String> = Vec::new();
        if schedule.waves.is_empty() {
            lines.push("No schedulable cells.".to_string());
        } else {
            for (index, wave) in schedule.waves.iter().enumerate() {
                lines.push(format!("Wave {}: {}", index + 1, wave.join(", ")));
            }
        }
        if !schedule.cycles.is_empty() {
            lines.push("Cycles:".to_string());
            for cycle in &schedule.cycles {
                lines.push(format!("- {}", cycle.join(" -> ")));
            }
        }
        if !schedule.unsatisfiable.is_empty() {
            lines.push("Unsatisfiable deps:".to_string());
            for (cell, dep, reason) in &schedule.unsatisfiable {
                lines.push(format!("- {cell} -> {dep} ({reason})"));
            }
        }
        if !schedule.empty_files.is_empty() {
            lines.push(format!("Empty files: {}", schedule.empty_files.join(", ")));
        }
        for (deferred, blocking, root) in &schedule.obligation_conflicts {
            lines.push(format!("{deferred} waits for {blocking} — shared regen root {root}"));
        }
        let result = json!({
            "waves": schedule.waves,
            "obligation_conflicts": schedule
                .obligation_conflicts
                .iter()
                .map(|(deferred, blocking, root)| json!({
                    "deferred": deferred,
                    "blocking": blocking,
                    "root": root,
                }))
                .collect::<Vec<_>>(),
            "diagnostics": {
                "cycles": schedule.cycles,
                "unsatisfiable_deps": schedule
                    .unsatisfiable
                    .iter()
                    .map(|(cell, dep, reason)| json!({"cell": cell, "dep": dep, "reason": reason}))
                    .collect::<Vec<_>>(),
                "empty_files": schedule.empty_files,
            }
        });
        Ok(Out::Emit(result, lines.join("\n"), 0))
    })
}

// ── cells archive / unarchive ──────────────────────────────────────────────

pub(crate) const ARCHIVE_JOURNAL_FILE: &str = ".journal.json";

pub(crate) fn cells_archive_dir(root: &Path, feature: &str) -> PathBuf {
    cells_dir(root).join(ARCHIVE_DIR_NAME).join(feature)
}

pub(crate) fn archive_journal_path(root: &Path, feature: &str) -> PathBuf {
    cells_archive_dir(root, feature).join(ARCHIVE_JOURNAL_FILE)
}

pub(crate) fn archive_summary_file(root: &Path) -> PathBuf {
    cells_dir(root).join(ARCHIVE_DIR_NAME).join("summary.json")
}

/// assertValidFeatureSlug (hardening-1).
pub(crate) fn assert_valid_feature_slug(verb: &str, feature: &str) -> MR<String> {
    if js_trim(feature).is_empty() {
        return Err(Fail::Thrown(format!("{verb}: feature is required.")));
    }
    let pattern_ok = !feature.is_empty()
        && feature
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    let all_dots = !feature.is_empty() && feature.chars().all(|c| c == '.');
    if !pattern_ok || all_dots {
        return Err(Fail::Thrown(format!(
            "{verb}: invalid feature \"{feature}\" — use letters, digits, dot, dash, underscore only (no path separators, and never \".\" or \"..\"). Refusing before any file is touched."
        )));
    }
    Ok(feature.to_string())
}

/// recoverArchiveJournal (hardening-1-7-10 D4) — direction-agnostic repair.
pub(crate) fn recover_archive_journal(root: &Path, feature: &str) -> MR<()> {
    let journal_path = archive_journal_path(root, feature);
    let journal = match read_json(&journal_path) {
        ReadJson::Missing => return Ok(()), // removeFileIfExists on nothing
        // readJson(journalPath, null) fails open to null, and `!journal`
        // then DELETES the journal and returns — the file is present here,
        // so the removal is what makes this arm equal to Node's.
        ReadJson::Corrupt => {
            warn_corrupt_json_once(&journal_path);
            crate::fsutil::remove_file_if_exists(&journal_path);
            return Ok(());
        }
        ReadJson::Parsed(v) => v,
    };
    let planned = journal.get("planned");
    let Some(Value::Array(planned)) = planned else {
        crate::fsutil::remove_file_if_exists(&journal_path);
        return Ok(());
    };
    for m in planned {
        let (Some(Value::String(from)), Some(Value::String(to))) = (m.get("from"), m.get("to")) else {
            continue;
        };
        let from_p = Path::new(from);
        let to_p = Path::new(to);
        if to_p.exists() && !from_p.exists() {
            let _ = std::fs::rename(to_p, from_p); // best-effort
        }
    }
    crate::fsutil::remove_file_if_exists(&journal_path);
    Ok(())
}

/// archivedSummary — {} on absent/shape-less, and on corrupt too (warn +
/// readJson's `{}` fallback).
pub(crate) fn archived_summary(root: &Path) -> MR<Map<String, Value>> {
    match read_store_json(&archive_summary_file(root))? {
        Some(Value::Object(m)) => Ok(m),
        _ => Ok(Map::new()),
    }
}

pub(crate) fn assert_archive_dir_contained(verb: &str, root: &Path, archive_dir: &Path) -> MR<()> {
    let base = cells_dir(root).join(ARCHIVE_DIR_NAME);
    let base_s = base.to_string_lossy().into_owned();
    let resolved_s = archive_dir.to_string_lossy().into_owned();
    let sep = std::path::MAIN_SEPARATOR;
    if resolved_s == base_s || resolved_s.starts_with(&format!("{base_s}{sep}")) {
        return Ok(());
    }
    Err(Fail::Thrown(format!(
        "{verb}: resolved archive path \"{resolved_s}\" escapes the archive root \"{base_s}\" — refusing before any file is touched."
    )))
}

/// Archive ONE feature's cells. The caller owns the `cells-archive` lock, the
/// active-feature guard and the slug check — so the batch path below can hold
/// the lock once across a whole sweep instead of contending with itself.
fn archive_one_feature(root: &Path, feature: &str) -> MR<(Vec<Value>, f64, f64)> {
    let feature = feature.to_string();
    let feature = &feature;
    {
        {
            recover_archive_journal(root, feature)?;
            let cells =
                list_cells(root, Some(feature.as_str()), None).map_err(|_| Fail::Delegate)?;
            if cells.is_empty() {
                return Err(Fail::Thrown(format!(
                    "archiveFeature: no cells found for feature \"{feature}\" — nothing to archive."
                )));
            }
            let terminal = |c: &Value| {
                matches!(c.get("status"), Some(Value::String(s)) if s == "capped" || s == "dropped")
            };
            let non_terminal: Vec<&Value> = cells.iter().filter(|c| !terminal(c)).collect();
            if !non_terminal.is_empty() {
                let named = non_terminal
                    .iter()
                    .map(|c| {
                        format!(
                            "{} ({})",
                            js_string_or_undefined(c.get("id")),
                            js_string_or_undefined(c.get("status"))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Fail::Thrown(format!(
                    "archiveFeature: feature \"{feature}\" has non-terminal cell(s) — {named} — only a feature whose cells are ALL capped/dropped can be archived."
                )));
            }
            let archive_dir = cells_archive_dir(&root, &feature);
            assert_archive_dir_contained("archiveFeature", &root, &archive_dir)?;
            std::fs::create_dir_all(&archive_dir).map_err(|e| Fail::Thrown(format!("{e}")))?;
            // Every cell.id must be a plain string for path.join — anything
            // else takes a V8 TypeError in Node.
            let mut planned: Vec<(Value, String, PathBuf, PathBuf)> = Vec::new();
            for cell in &cells {
                let Some(Value::String(cid)) = cell.get("id") else { return Err(Fail::Delegate) };
                let status = js_string_or_undefined(cell.get("status"));
                planned.push((
                    cell.get("id").cloned().unwrap_or(Value::Null),
                    status,
                    cell_file(&root, cid),
                    archive_dir.join(format!("{cid}.json")),
                ));
            }
            let collisions: Vec<String> = planned
                .iter()
                .filter(|(_, _, _, to)| to.exists())
                .map(|(id, _, _, _)| jsjson::js_to_string(id))
                .collect();
            if !collisions.is_empty() {
                return Err(Fail::Thrown(format!(
                    "archiveFeature: feature \"{feature}\" refused — a archived file already exists for {}. Refusing before any file is touched (never overwrite existing data).",
                    collisions.join(", ")
                )));
            }
            let planned_value = Value::Array(
                planned
                    .iter()
                    .map(|(id, _, from, to)| {
                        json!({
                            "id": id,
                            "from": from.to_string_lossy(),
                            "to": to.to_string_lossy(),
                        })
                    })
                    .collect(),
            );
            write_json_atomic(
                &archive_journal_path(&root, &feature),
                &json!({"op": "archive", "feature": feature, "planned": planned_value, "started_at": utc_now()}),
            )
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
            let mut moved: Vec<(PathBuf, PathBuf, Value)> = Vec::new();
            let mut capped = 0f64;
            let mut dropped = 0f64;
            for (id, status, from, to) in &planned {
                if let Err(e) = std::fs::rename(from, to) {
                    for (from_r, to_r, _) in moved.iter().rev() {
                        let _ = std::fs::rename(to_r, from_r); // best-effort rollback
                    }
                    crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
                    return Err(Fail::Thrown(format!("{e}"))); // residual: libuv message in Node
                }
                moved.push((from.clone(), to.clone(), id.clone()));
                if status == "capped" {
                    capped += 1.0;
                } else if status == "dropped" {
                    dropped += 1.0;
                }
            }
            let mut summary = archived_summary(&root)?;
            let mut entry = Map::new();
            entry.insert("capped".into(), Value::Number(Number::from_f64(capped).unwrap()));
            entry.insert("dropped".into(), Value::Number(Number::from_f64(dropped).unwrap()));
            entry.insert("archived_at".into(), Value::String(utc_now()));
            summary.insert(feature.clone(), Value::Object(entry));
            write_json_atomic(&archive_summary_file(&root), &Value::Object(summary))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            crate::fsutil::remove_file_if_exists(&archive_journal_path(root, feature));
            Ok((moved.into_iter().map(|(_, _, id)| id).collect(), capped, dropped))
        }
    }
}

/// Features whose ACTIVE cells are all terminal, excluding the active feature.
/// Returns (eligible, skipped-with-reason) — nothing is dropped silently: a
/// sweep that quietly passed over a feature would look identical to one that
/// had nothing to do.
fn terminal_features(root: &Path, active: Option<&str>) -> MR<(Vec<String>, Vec<(String, String)>)> {
    let cells = list_cells(root, None, None).map_err(|_| Fail::Delegate)?;
    let mut order: Vec<String> = Vec::new();
    let mut by_feature: std::collections::HashMap<String, Vec<&Value>> =
        std::collections::HashMap::new();
    for cell in &cells {
        let Some(Value::String(f)) = cell.get("feature") else { continue };
        if f.is_empty() {
            continue;
        }
        if !by_feature.contains_key(f) {
            order.push(f.clone());
        }
        by_feature.entry(f.clone()).or_default().push(cell);
    }
    order.sort();
    let (mut eligible, mut skipped) = (Vec::new(), Vec::new());
    for feature in order {
        let group = &by_feature[&feature];
        if active == Some(feature.as_str()) {
            skipped.push((feature, "active feature (state.feature)".to_string()));
            continue;
        }
        let live: Vec<String> = group
            .iter()
            .filter(|c| {
                !matches!(c.get("status"), Some(Value::String(s)) if s == "capped" || s == "dropped")
            })
            .map(|c| {
                format!(
                    "{} ({})",
                    js_string_or_undefined(c.get("id")),
                    js_string_or_undefined(c.get("status"))
                )
            })
            .collect();
        if live.is_empty() {
            eligible.push(feature);
        } else {
            skipped.push((feature, format!("non-terminal cell(s): {}", live.join(", "))));
        }
    }
    Ok((eligible, skipped))
}

pub(crate) fn run_archive(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature", "all-but-active"]) {
        return None;
    }
    let sweep = matches!(flags.get("all-but-active"), Some(FlagV::Present));
    let feature_flag = flags.truthy_str("feature").map(str::to_string);
    dispatch("cells archive", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let state = bstate::read_state_brief(&root);
        let active: Option<String> = match &state.feature {
            Value::String(f) if js_truthy(&state.feature) => Some(f.clone()),
            _ => None,
        };

        // Two modes, and the registry cannot say "exactly one of" in a flat
        // JSON-Schema `required` — so it declares neither and the handler
        // owns the choice, exactly as `state workflows close` does.
        if sweep == feature_flag.is_some() {
            return Err(Fail::Thrown(
                "cells archive: requires exactly one of --feature <feature> (retire that one feature) or --all-but-active (retire every finished feature in one pass). Example: bee cells archive --all-but-active --json"
                    .to_string(),
            ));
        }

        if sweep {
            // The same guard `state workflows close --all-but-active` carries,
            // for the same reason: with no resolvable active feature, "all but
            // active" silently becomes "all", and the one feature that must
            // never be archived is the in-flight one.
            let Some(active_feature) = active.as_deref() else {
                return Err(Fail::Thrown(
                    "cells archive --all-but-active: refused — state.feature is empty, so \"all but active\" cannot be evaluated and would degrade into \"all\", archiving the in-flight feature's cells. Nothing was archived. Set the active feature, or archive one feature at a time with --feature."
                        .to_string(),
                ));
            };
            let (eligible, skipped) = terminal_features(&root, Some(active_feature))?;
            let mut guard = acquire_named_lock(&root, "cells-archive")?;
            let outcome = (|| -> MR<(Vec<Value>, f64, f64, Vec<(String, String)>)> {
                let (mut rows, mut capped_total, mut dropped_total) = (Vec::new(), 0f64, 0f64);
                let mut failed: Vec<(String, String)> = Vec::new();
                for feature in &eligible {
                    let slug = match assert_valid_feature_slug("archiveFeature", feature) {
                        Ok(s) => s,
                        // One malformed slug must not abort a sweep that is
                        // otherwise fine — it is reported, not fatal.
                        Err(Fail::Thrown(why)) => {
                            failed.push((feature.clone(), why));
                            continue;
                        }
                        Err(other) => return Err(other),
                    };
                    match archive_one_feature(&root, &slug) {
                        Ok((moved, capped, dropped)) => {
                            capped_total += capped;
                            dropped_total += dropped;
                            rows.push(json!({
                                "feature": slug,
                                "moved": moved.len(),
                                "counts": {"capped": capped, "dropped": dropped},
                            }));
                        }
                        Err(Fail::Thrown(why)) => failed.push((slug, why)),
                        Err(other) => return Err(other),
                    }
                }
                Ok((rows, capped_total, dropped_total, failed))
            })();
            guard.release();
            let (rows, capped, dropped, failed) = outcome?;
            let moved_total: u64 = rows
                .iter()
                .filter_map(|r| r.get("moved").and_then(Value::as_u64))
                .sum();
            let result = json!({
                "mode": "all-but-active",
                "active_feature": active_feature,
                "archived": rows,
                "counts": {"capped": capped, "dropped": dropped},
                "skipped": skipped
                    .iter()
                    .map(|(f, why)| json!({"feature": f, "reason": why}))
                    .collect::<Vec<_>>(),
                "failed": failed
                    .iter()
                    .map(|(f, why)| json!({"feature": f, "reason": why}))
                    .collect::<Vec<_>>(),
            });
            let mut text = format!(
                "Archived {} feature(s): {moved_total} cell(s) moved (capped={} dropped={}). Kept {} in the active scan.",
                rows.len(),
                jsjson::js_f64_to_string(capped),
                jsjson::js_f64_to_string(dropped),
                skipped.len()
            );
            for (f, why) in &failed {
                text.push_str(&format!("\n  ! {f}: {why}"));
            }
            return Ok(Out::Emit(result, text, if failed.is_empty() { 0 } else { 1 }));
        }

        let feature = feature_flag.clone().unwrap_or_default();
        // handleCellsArchive: active-feature guard from state.json.
        if active.as_deref() == Some(feature.as_str()) {
            return Err(Fail::Thrown(format!(
                "cells archive: feature \"{feature}\" is the active feature (state.feature) — only a closed/inactive feature can be archived. Switch or clear state.feature first, or archive a different feature."
            )));
        }
        let feature = assert_valid_feature_slug("archiveFeature", &feature)?;
        let mut guard = acquire_named_lock(&root, "cells-archive")?;
        let outcome = archive_one_feature(&root, &feature);
        guard.release();
        let (moved, capped, dropped) = outcome?;
        let result = json!({
            "feature": feature,
            "moved": moved,
            "counts": {"capped": capped, "dropped": dropped},
        });
        let text = format!(
            "Archived feature \"{feature}\": {} cell(s) moved (capped={} dropped={}).",
            moved.len(),
            jsjson::js_f64_to_string(capped),
            jsjson::js_f64_to_string(dropped)
        );
        Ok(Out::Emit(result, text, 0))
    })
}

pub(crate) fn run_unarchive(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = flags.req_str("feature")?.to_string();
    dispatch("cells unarchive", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let feature = assert_valid_feature_slug("unarchiveFeature", &feature)?;
        let mut guard = acquire_named_lock(&root, "cells-archive")?;
        let outcome = (|| -> MR<Vec<String>> {
            recover_archive_journal(&root, &feature)?;
            let archive_dir = cells_archive_dir(&root, &feature);
            assert_archive_dir_contained("unarchiveFeature", &root, &archive_dir)?;
            let entries = match std::fs::read_dir(&archive_dir) {
                Ok(e) => e,
                Err(_) => {
                    return Err(Fail::Thrown(format!(
                        "unarchiveFeature: no archived cells found for feature \"{feature}\"."
                    )))
                }
            };
            let names: Vec<String> = entries
                .flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_string))
                .collect();
            let json_files: Vec<String> = names
                .into_iter()
                .filter(|f| f.ends_with(".json") && f != ARCHIVE_JOURNAL_FILE)
                .collect();
            if json_files.is_empty() {
                return Err(Fail::Thrown(format!(
                    "unarchiveFeature: no archived cells found for feature \"{feature}\"."
                )));
            }
            let planned: Vec<(String, PathBuf, PathBuf)> = json_files
                .iter()
                .map(|f| {
                    (
                        f[..f.len() - ".json".len()].to_string(),
                        archive_dir.join(f),
                        cells_dir(&root).join(f),
                    )
                })
                .collect();
            let collisions: Vec<String> = planned
                .iter()
                .filter(|(_, _, to)| to.exists())
                .map(|(id, _, _)| id.clone())
                .collect();
            if !collisions.is_empty() {
                return Err(Fail::Thrown(format!(
                    "unarchiveFeature: feature \"{feature}\" refused — a active file already exists for {}. Refusing before any file is touched (never overwrite existing data).",
                    collisions.join(", ")
                )));
            }
            let planned_value = Value::Array(
                planned
                    .iter()
                    .map(|(id, from, to)| {
                        json!({"id": id, "from": from.to_string_lossy(), "to": to.to_string_lossy()})
                    })
                    .collect(),
            );
            write_json_atomic(
                &archive_journal_path(&root, &feature),
                &json!({"op": "unarchive", "feature": feature, "planned": planned_value, "started_at": utc_now()}),
            )
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
            let mut moved: Vec<(PathBuf, PathBuf, String)> = Vec::new();
            for (id, from, to) in &planned {
                if let Err(e) = std::fs::rename(from, to) {
                    for (from_r, to_r, _) in moved.iter().rev() {
                        let _ = std::fs::rename(to_r, from_r);
                    }
                    crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
                    return Err(Fail::Thrown(format!("{e}")));
                }
                moved.push((from.clone(), to.clone(), id.clone()));
            }
            crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
            let _ = std::fs::remove_dir(&archive_dir); // best-effort
            let mut summary = archived_summary(&root)?;
            summary.shift_remove(&feature);
            write_json_atomic(&archive_summary_file(&root), &Value::Object(summary))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            Ok(moved.into_iter().map(|(_, _, id)| id).collect())
        })();
        guard.release();
        let moved = outcome?;
        let result = json!({"feature": feature, "moved": moved});
        let text = format!(
            "Unarchived feature \"{feature}\": {} cell(s) restored to .bee/cells/.",
            moved.len()
        );
        Ok(Out::Emit(result, text, 0))
    })
}

/// `bee close`'s retirement entry point: archive `feature`'s cells now that
/// close has proven the feature done.
///
/// It deliberately does NOT carry `run_archive`'s active-feature guard. That
/// guard exists to stop a caller archiving work that is still in flight; close
/// has just run the declared suite green over this exact feature, which is the
/// evidence the guard is a proxy for. The condition it DOES keep is the real
/// one: every cell terminal. Close's only blocking door is tests, so a feature
/// can close green while still holding an open cell, and that cell belongs in
/// the active scan.
///
/// `Err(reason)` is a sentence for the caller to print, never a failed close.
pub(crate) fn archive_feature_for_close(root: &Path, feature: &str) -> Result<usize, String> {
    let slug = match assert_valid_feature_slug("archiveFeature", feature) {
        Ok(s) => s,
        Err(Fail::Thrown(why)) => return Err(why),
        Err(_) => return Err(format!("feature \"{feature}\" could not be resolved")),
    };
    let cells = match list_cells(root, Some(slug.as_str()), None) {
        Ok(c) => c,
        Err(_) => return Err(format!("the cell store for \"{slug}\" could not be read")),
    };
    if cells.is_empty() {
        // Nothing to retire is not a held store — a docs-only feature, or one
        // whose cells were archived already, has nothing to say about them.
        return Ok(0);
    }
    let live: Vec<String> = cells
        .iter()
        .filter(|c| {
            !matches!(c.get("status"), Some(Value::String(s)) if s == "capped" || s == "dropped")
        })
        .map(|c| {
            format!(
                "{} ({})",
                js_string_or_undefined(c.get("id")),
                js_string_or_undefined(c.get("status"))
            )
        })
        .collect();
    if !live.is_empty() {
        return Err(format!(
            "feature \"{slug}\" still holds {}",
            live.join(", ")
        ));
    }
    let mut guard = match acquire_named_lock(root, "cells-archive") {
        Ok(g) => g,
        // Another session is archiving. Not this close's problem to solve.
        Err(_) => return Err(format!("the cells-archive lock is held — \"{slug}\" stays for now")),
    };
    let outcome = archive_one_feature(root, &slug);
    guard.release();
    match outcome {
        Ok((moved, _, _)) => Ok(moved.len()),
        Err(Fail::Thrown(why)) => Err(why),
        Err(_) => Err(format!("feature \"{slug}\" could not be archived")),
    }
}
