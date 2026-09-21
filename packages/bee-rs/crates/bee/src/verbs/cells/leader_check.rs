// bee cells leader-check — record a verified leader completeness check on a
// capped cell (leader-check-door, decisions a38dc4bd and f5e3c084).
//
// WHY A VERB AND TWO DOORS:
// The leader completeness check originally shipped as instruction only (f5e3c084)
// and had no code enforcement. A leader that never compares a capped cell's
// approved requirements against actual artifacts was refused by nothing.
//
// leader-check-door gives the check teeth:
// 1. D1: The mark lives on trace.leader_check as an append-only array.
// 2. D2: Recorded through `bee cells leader-check`.
// 3. D3: The payload answers every derived requirement with an artifact,
//    and artifacts are verified: path-shaped artifacts must exist on disk,
//    and at least one answer must name a path in trace.files_changed.
// 4. D3a: Requirements are derived in order: must_haves.truths when non-empty;
//    else one per trace.files_changed entry; else exactly one free requirement.
// 5. D4: Applies to every lane and every capped cell.
// 6. D5: Grandfathered by LEADER_CHECK_DOOR_INTRODUCED_AT.
// 7. D6: Reached by two doors: bee close and bee worktree merge.
// 8. D7: Named escape: a logged decision tagged leader-check-deferral.
// 9. D8: A gap verdict records and never mutates cell status.

#![allow(unused_imports)]
#![allow(dead_code)]

use super::*;
use crate::jsjson;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{FlagV, Out};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

/// The grandfather cutoff stamp (D5). A cell whose trace.capped_at is missing,
/// unparseable, or earlier than this stamp predates the door and is grandfathered.
pub(crate) const LEADER_CHECK_DOOR_INTRODUCED_AT: &str = "2026-09-21T00:00:00.000Z";

/// The cell-trace key the record appends to. An append-only array.
pub(crate) const LEADER_CHECK_TRACE_KEY: &str = "leader_check";

/// Verb name leading every refusal thrown in this module.
const VERB: &str = "recordLeaderCheck";

#[derive(Debug, Clone)]
pub(crate) struct Answer {
    pub(crate) requirement: String,
    pub(crate) artifact: String,
}

pub(crate) fn run_leader_check(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(
        &flags,
        &["id", "verdict", "file", "session-id", "force-ownership"],
    ) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let verdict = flags.req_str("verdict")?.to_string();
    let file = flags.req_str("file")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells leader-check", use_json, t0, move |ctx| {
        let raw = read_file_text(&file, "leader check payload")?;
        let payload = match parse_json_js(&raw, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => {
                return Err(Fail::Thrown(format!(
                    "{VERB}: payload file \"{file}\" is not valid JSON."
                )));
            }
        };
        let cell = record_leader_check(
            &ctx.root,
            &id,
            &verdict,
            &payload,
            session_flag.as_deref(),
            force,
        )?;
        let text = format!("Recorded leader check on {id}: {verdict}.");
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn record_leader_check(
    root: &Path,
    id: &str,
    verdict: &str,
    payload: &Value,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Value> {
    let verdict = js_trim(verdict);
    if verdict != "ok" && verdict != "gap" {
        return Err(Fail::Thrown(format!(
            "{VERB}: unknown verdict \"{verdict}\" — verdict must be \"ok\" or \"gap\"."
        )));
    }

    let Value::Object(payload_map) = payload else {
        return Err(Fail::Thrown(format!(
            "{VERB}: payload must be a JSON object matching schema \"leader-check/1\"."
        )));
    };
    if payload_map.get("schema").and_then(Value::as_str) != Some("leader-check/1") {
        return Err(Fail::Thrown(format!(
            "{VERB}: cell \"{id}\" payload rejected against schema \"leader-check/1\" — schema field must be \"leader-check/1\"."
        )));
    }
    let Some(answers_arr) = payload_map.get("answers").and_then(Value::as_array) else {
        return Err(Fail::Thrown(format!(
            "{VERB}: cell \"{id}\" payload rejected against schema \"leader-check/1\" — answers field must be an array."
        )));
    };

    let mut answers = Vec::with_capacity(answers_arr.len());
    for ans_val in answers_arr {
        let Value::Object(ans_map) = ans_val else {
            return Err(Fail::Thrown(format!(
                "{VERB}: cell \"{id}\" payload rejected against schema \"leader-check/1\" — answer must be an object."
            )));
        };
        let req_str = ans_map
            .get("requirement")
            .and_then(Value::as_str)
            .map(js_trim)
            .unwrap_or("");
        let art_str = ans_map
            .get("artifact")
            .and_then(Value::as_str)
            .map(js_trim)
            .unwrap_or("");
        if req_str.is_empty() {
            return Err(Fail::Thrown(format!(
                "{VERB}: cell \"{id}\" payload rejected against schema \"leader-check/1\" — answer requirement must be a non-empty string."
            )));
        }
        if art_str.is_empty() {
            return Err(Fail::Thrown(format!(
                "{VERB}: cell \"{id}\" payload rejected against schema \"leader-check/1\" — answer artifact must be a non-empty string."
            )));
        }
        answers.push(Answer {
            requirement: req_str.to_string(),
            artifact: art_str.to_string(),
        });
    }

    prescan_claim(root, id)?;
    delegate_only(load_taxonomy(root))?;
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        assert_not_archived(root, VERB, id)?;
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("{VERB}: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };

        let status = cell_map.get("status").and_then(Value::as_str).unwrap_or("");
        if status != "capped" {
            return Err(Fail::Thrown(format!(
                "{VERB}: cell \"{id}\" status is \"{status}\" — leader-check can only be recorded on a capped cell."
            )));
        }

        // D3a derivation:
        // (1) must_haves.truths when non-empty;
        // (2) else one requirement per trace.files_changed entry;
        // (3) else exactly one free requirement.
        let truths: Vec<String> = cell_map
            .get("must_haves")
            .and_then(|m| m.get("truths"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| js_trim(s).to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let trace_obj = cell_map.get("trace");
        let files_changed: Vec<String> = trace_obj
            .and_then(|t| t.get("files_changed"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| js_trim(s).to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if !truths.is_empty() {
            let answered: HashSet<&str> =
                answers.iter().map(|a| a.requirement.as_str()).collect();
            let misses: Vec<&str> = truths
                .iter()
                .map(|s| s.as_str())
                .filter(|t| !answered.contains(t))
                .collect();
            if !misses.is_empty() {
                return Err(Fail::Thrown(format!(
                    "{VERB}: cell \"{id}\" answers do not cover derived requirement(s): {}",
                    misses.join(", ")
                )));
            }
        } else if !files_changed.is_empty() {
            let answered: HashSet<&str> =
                answers.iter().map(|a| a.requirement.as_str()).collect();
            let misses: Vec<&str> = files_changed
                .iter()
                .map(|s| s.as_str())
                .filter(|f| !answered.contains(f))
                .collect();
            if !misses.is_empty() {
                return Err(Fail::Thrown(format!(
                    "{VERB}: cell \"{id}\" answers do not cover derived requirement(s): {}",
                    misses.join(", ")
                )));
            }
        } else {
            // exactly one free requirement that must be answered non-blank
            if answers.len() != 1 {
                return Err(Fail::Thrown(format!(
                    "{VERB}: cell \"{id}\" has no truths or changed files — requires exactly one non-blank answer, got {}.",
                    answers.len()
                )));
            }
        }

        // D3 verification:
        // (a) an artifact string that looks like a repo path (contains '/' and no whitespace)
        // MUST exist on disk relative to the repo root
        for answer in &answers {
            if answer.artifact.contains('/') && !answer.artifact.chars().any(char::is_whitespace) {
                let p = root.join(&answer.artifact);
                if !p.exists() {
                    return Err(Fail::Thrown(format!(
                        "{VERB}: artifact \"{}\" does not exist on disk relative to repo root.",
                        answer.artifact
                    )));
                }
            }
        }

        // (b) and at least one answer per cell MUST name a path present in that cell's trace.files_changed.
        // Refuse, naming the offending artifact, otherwise.
        if !files_changed.is_empty() {
            let has_overlap = answers
                .iter()
                .any(|a| files_changed.iter().any(|fc| fc == &a.artifact));
            if !has_overlap {
                let offending = answers
                    .iter()
                    .map(|a| format!("\"{}\"", a.artifact))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Fail::Thrown(format!(
                    "{VERB}: cell \"{id}\" answers name no path present in trace.files_changed — offending artifact(s): {offending}."
                )));
            }
        }

        // (c) files_changed is checked against the cell's real commit diff
        // (decision 5e3f0baf): every claimed path must appear in a commit
        // carrying the `cell: <id>` trailer. Extra diff paths are not refused.
        let commit_diff = if files_changed.is_empty() {
            None
        } else {
            let feature = cell_map.get("feature").and_then(Value::as_str);
            let history_root = commit_trailer_history_root(root, feature);
            Some(match cell_commit_files(&history_root, id) {
                None => "skipped: git unavailable",
                Some(paths) if paths.is_empty() => {
                    let pending = trace_obj
                        .and_then(|t| t.get("commit_pending"))
                        .and_then(Value::as_str)
                        .is_some_and(|s| !js_trim(s).is_empty());
                    if !pending {
                        return Err(Fail::Thrown(format!(
                            "{VERB}: cell \"{id}\" has no commit carrying the trailer \"{}\" in the last {COMMIT_TRAILER_WINDOW} commit(s) of {} — files_changed cannot be verified against a real diff.",
                            cell_commit_trailer(id),
                            history_root.display()
                        )));
                    }
                    "skipped: commit_pending"
                }
                Some(paths) => {
                    let untouched: Vec<String> = files_changed
                        .iter()
                        .map(|f| normalize_cell_path(f))
                        .filter(|f| !paths.contains(f))
                        .collect();
                    if !untouched.is_empty() {
                        return Err(Fail::Thrown(format!(
                            "{VERB}: cell \"{id}\" files_changed names path(s) the cell's commit(s) never touched: {}.",
                            untouched.join(", ")
                        )));
                    }
                    "checked"
                }
            })
        };

        let mut trace = merge_trace(cell_map.get("trace"))?;
        trace = guard_claim_ownership(
            root,
            id,
            trace,
            VERB,
            session_flag,
            force,
        )?;

        let mut entry = Map::new();
        entry.insert("verdict".into(), Value::String(verdict.to_string()));
        let answers_json: Vec<Value> = answers
            .iter()
            .map(|a| {
                let mut m = Map::new();
                m.insert("requirement".into(), Value::String(a.requirement.clone()));
                m.insert("artifact".into(), Value::String(a.artifact.clone()));
                Value::Object(m)
            })
            .collect();
        entry.insert("answers".into(), Value::Array(answers_json));
        entry.insert("recorded_at".into(), Value::String(utc_now()));
        let recorded_by = resolve_session_flag_env(session_flag)
            .map(Value::String)
            .unwrap_or(Value::Null);
        entry.insert("recorded_by".into(), recorded_by);
        if let Some(commit_diff) = commit_diff {
            entry.insert("commit_diff".into(), Value::String(commit_diff.into()));
        }

        let mut existing: Vec<Value> = match trace.get(LEADER_CHECK_TRACE_KEY) {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        existing.push(Value::Object(entry));
        trace.insert(LEADER_CHECK_TRACE_KEY.into(), Value::Array(existing));

        // D8: NEVER touch cell status, not even on gap!
        cell_map.insert("trace".into(), Value::Object(trace));
        let value = Value::Object(cell_map);
        write_cell(root, &value)?;
        Ok(value)
    })();
    guard.release();
    saved
}

/// The union of paths changed by every commit in the last
/// `COMMIT_TRAILER_WINDOW` commits of `cwd`'s HEAD whose body carries the
/// exact `cell: <id>` trailer line, each normalized. None when git cannot
/// answer (no repo, no commits, spawn failure); Some(empty) when no commit
/// carries the trailer.
pub(crate) fn cell_commit_files(cwd: &Path, id: &str) -> Option<Vec<String>> {
    let window = COMMIT_TRAILER_WINDOW.to_string();
    let out = crate::verbs::worktree::run_git(cwd, &["log", "-n", &window, "--format=%H%n%B%x00"]);
    if out.status != Some(0) {
        return None;
    }
    let trailer = cell_commit_trailer(id);
    let mut files: Vec<String> = Vec::new();
    for record in out.stdout.unwrap_or_default().split('\u{0}') {
        let mut lines = record.trim_start_matches('\n').lines();
        let Some(hash) = lines.next().map(js_trim).filter(|h| !h.is_empty()) else { continue };
        if !lines.any(|line| js_trim(line) == trailer) {
            continue;
        }
        let show = crate::verbs::worktree::run_git(cwd, &["show", "--name-only", "--format=", hash]);
        if show.status != Some(0) {
            return None;
        }
        for path in show.stdout.unwrap_or_default().lines().map(normalize_cell_path) {
            if !path.is_empty() && !files.contains(&path) {
                files.push(path);
            }
        }
    }
    Some(files)
}

/// D6: The single debt scan for the leader-check obligation, shared by both
/// `bee close` and `bee worktree merge`.
pub(crate) fn feature_leader_check_debt(
    root: &Path,
    feature: &str,
) -> Result<crate::verbs::drivers::DebtSummary, crate::verbs::drivers::Delegate> {
    let cutoff = crate::verbs::drivers::date_parse(Some(&Value::String(
        LEADER_CHECK_DOOR_INTRODUCED_AT.to_string(),
    )));
    let mut ids = Vec::new();
    for cell in crate::verbs::drivers::list_cells_including_archive(root, feature, Some("capped"))? {
        let trace = cell.get("trace").and_then(Value::as_object);
        let capped_at = crate::verbs::drivers::date_parse(
            trace.and_then(|t| t.get("capped_at")),
        );
        if !(capped_at.is_finite() && capped_at >= cutoff) {
            continue; // pre-door or no capped_at: grandfathered
        }
        let is_ok = trace
            .and_then(|t| t.get(LEADER_CHECK_TRACE_KEY))
            .and_then(Value::as_array)
            .and_then(|a| a.last())
            .and_then(|e| e.get("verdict"))
            .and_then(Value::as_str)
            == Some("ok");
        if !is_ok {
            if let Some(id) = cell.get("id") {
                ids.push(id.clone());
            }
        }
    }
    Ok(crate::verbs::drivers::DebtSummary { count: ids.len(), ids })
}

/// D7: The named escape — a logged decision tagged `leader-check-deferral`
/// naming the feature lifts the refusal at BOTH doors.
pub(crate) fn has_leader_check_deferral_decision(
    root: &Path,
    feature: &str,
) -> Result<bool, crate::verbs::drivers::Delegate> {
    let active = crate::verbs::decisions::active_decisions(root, false)
        .map_err(|_| crate::verbs::drivers::Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("leader-check-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| crate::verbs::drivers::Delegate)?;
    Ok(!filtered.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsutil::{read_json, ReadJson};
    use serde_json::json;

    fn write_cell_fixture(root: &Path, id: &str, body: &Value) {
        let dir = cells_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(body)).unwrap();
    }

    fn read_cell_fixture(root: &Path, id: &str) -> Value {
        match read_json(&cells_dir(root).join(format!("{id}.json"))) {
            ReadJson::Parsed(v) => v,
            ReadJson::Missing => panic!("cell {id} fixture missing"),
            ReadJson::Corrupt => panic!("cell {id} fixture corrupt"),
        }
    }

    fn thrown<T>(r: MR<T>) -> String {
        match r {
            Err(Fail::Thrown(m)) => m,
            Err(Fail::Delegate) => panic!("expected a thrown refusal, got Delegate"),
            Ok(_) => panic!("expected a refusal, got Ok"),
        }
    }

    fn make_capped_cell(
        id: &str,
        truths: &[&str],
        files_changed: &[&str],
        capped_at: Option<&str>,
    ) -> Value {
        let mut trace = json!({
            "worker": "w-1",
            "files_changed": files_changed,
        });
        if let Some(ts) = capped_at {
            trace.as_object_mut().unwrap().insert("capped_at".into(), json!(ts));
        }
        json!({
            "id": id,
            "title": format!("title {id}"),
            "status": "capped",
            "lane": "standard",
            "feature": "feat-1",
            "must_haves": {
                "truths": truths,
            },
            "trace": trace,
        })
    }

    #[test]
    fn bad_schema_or_unknown_verdict_refuses() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "c-1", &make_capped_cell("c-1", &[], &[], None));

        let bad_verdict = record_leader_check(
            root,
            "c-1",
            "maybe",
            &json!({"schema": "leader-check/1", "answers": []}),
            None,
            false,
        );
        assert!(thrown(bad_verdict).contains("unknown verdict \"maybe\""));

        let bad_schema = record_leader_check(
            root,
            "c-1",
            "ok",
            &json!({"schema": "wrong/1", "answers": []}),
            None,
            false,
        );
        assert!(thrown(bad_schema).contains("schema field must be \"leader-check/1\""));
    }

    #[test]
    fn uncapped_cell_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let open_cell = json!({
            "id": "c-open",
            "title": "title",
            "status": "open",
            "lane": "standard",
            "feature": "feat-1",
            "trace": { "worker": "w-1" },
        });
        write_cell_fixture(root, "c-open", &open_cell);

        let err = record_leader_check(
            root,
            "c-open",
            "ok",
            &json!({
                "schema": "leader-check/1",
                "answers": [{"requirement": "req", "artifact": "art"}]
            }),
            None,
            false,
        );
        assert!(thrown(err).contains("leader-check can only be recorded on a capped cell"));
    }

    #[test]
    fn archived_cell_is_refused_naming_bee_cells_unarchive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Archive directory fixture
        let arch_dir = root.join(".bee").join("cells").join("archive").join("feat-1");
        std::fs::create_dir_all(&arch_dir).unwrap();
        let cell = make_capped_cell("c-arch", &[], &[], None);
        std::fs::write(arch_dir.join("c-arch.json"), jsjson::stringify_pretty(&cell)).unwrap();

        let err = record_leader_check(
            root,
            "c-arch",
            "ok",
            &json!({
                "schema": "leader-check/1",
                "answers": [{"requirement": "req", "artifact": "art"}]
            }),
            None,
            false,
        );
        let msg = thrown(err);
        assert!(msg.contains("is archived"));
        assert!(msg.contains("bee cells unarchive"));
    }

    #[test]
    fn answer_naming_path_not_on_disk_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(
            root,
            "c-1",
            &make_capped_cell("c-1", &["Truth A"], &["src/foo.rs"], None),
        );

        let payload = json!({
            "schema": "leader-check/1",
            "answers": [
                {
                    "requirement": "Truth A",
                    "artifact": "src/nonexistent.rs"
                }
            ]
        });

        let err = record_leader_check(root, "c-1", "ok", &payload, None, false);
        let msg = thrown(err);
        assert!(msg.contains("does not exist on disk relative to repo root"));
        assert!(msg.contains("src/nonexistent.rs"));
    }

    #[test]
    fn payload_whose_answers_name_no_path_from_files_changed_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Create an actual file on disk that is NOT in files_changed
        let other_path = root.join("other/file.rs");
        std::fs::create_dir_all(other_path.parent().unwrap()).unwrap();
        std::fs::write(&other_path, "// other").unwrap();

        write_cell_fixture(
            root,
            "c-1",
            &make_capped_cell("c-1", &["Truth A"], &["src/actual.rs"], None),
        );

        let payload = json!({
            "schema": "leader-check/1",
            "answers": [
                {
                    "requirement": "Truth A",
                    "artifact": "other/file.rs"
                }
            ]
        });

        let err = record_leader_check(root, "c-1", "ok", &payload, None, false);
        let msg = thrown(err);
        assert!(msg.contains("answers name no path present in trace.files_changed"));
        assert!(msg.contains("other/file.rs"));
    }

    #[test]
    fn cell_with_empty_truths_still_owes_one_answer_per_files_changed_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let p1 = root.join("src/a.rs");
        let p2 = root.join("src/b.rs");
        std::fs::create_dir_all(p1.parent().unwrap()).unwrap();
        std::fs::write(&p1, "// a").unwrap();
        std::fs::write(&p2, "// b").unwrap();

        write_cell_fixture(
            root,
            "c-1",
            &make_capped_cell("c-1", &[], &["src/a.rs", "src/b.rs"], None),
        );

        // Answers only covering src/a.rs should be refused for missing src/b.rs
        let payload_incomplete = json!({
            "schema": "leader-check/1",
            "answers": [
                {
                    "requirement": "src/a.rs",
                    "artifact": "src/a.rs"
                }
            ]
        });
        let err = record_leader_check(root, "c-1", "ok", &payload_incomplete, None, false);
        let msg = thrown(err);
        assert!(msg.contains("answers do not cover derived requirement(s): src/b.rs"));

        // Complete answers pass
        let payload_complete = json!({
            "schema": "leader-check/1",
            "answers": [
                {
                    "requirement": "src/a.rs",
                    "artifact": "src/a.rs"
                },
                {
                    "requirement": "src/b.rs",
                    "artifact": "src/b.rs"
                }
            ]
        });
        let res = record_leader_check(root, "c-1", "ok", &payload_complete, None, false);
        assert!(res.is_ok());
    }

    #[test]
    fn cell_with_empty_truths_and_empty_files_changed_requires_exactly_one_free_answer() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "c-1", &make_capped_cell("c-1", &[], &[], None));

        // 0 answers refused
        let payload_zero = json!({
            "schema": "leader-check/1",
            "answers": []
        });
        let err_zero = record_leader_check(root, "c-1", "ok", &payload_zero, None, false);
        assert!(thrown(err_zero).contains("requires exactly one non-blank answer"));

        // 2 answers refused
        let payload_two = json!({
            "schema": "leader-check/1",
            "answers": [
                {"requirement": "req1", "artifact": "art1"},
                {"requirement": "req2", "artifact": "art2"}
            ]
        });
        let err_two = record_leader_check(root, "c-1", "ok", &payload_two, None, false);
        assert!(thrown(err_two).contains("requires exactly one non-blank answer, got 2"));

        // 1 answer passes (artifact not path-shaped does not need to exist on disk)
        let payload_one = json!({
            "schema": "leader-check/1",
            "answers": [
                {"requirement": "verified visually", "artifact": "terminal inspection"}
            ]
        });
        let res = record_leader_check(root, "c-1", "ok", &payload_one, None, false);
        assert!(res.is_ok());
    }

    #[test]
    fn gap_verdict_is_appended_and_cell_status_stays_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let p = root.join("src/lib.rs");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "// lib").unwrap();

        write_cell_fixture(
            root,
            "c-gap",
            &make_capped_cell("c-gap", &["Truth 1"], &["src/lib.rs"], None),
        );

        let payload = json!({
            "schema": "leader-check/1",
            "answers": [
                {
                    "requirement": "Truth 1",
                    "artifact": "src/lib.rs"
                }
            ]
        });

        let updated = record_leader_check(root, "c-gap", "gap", &payload, None, false).unwrap();
        assert_eq!(updated["status"], json!("capped"));

        let trace = &updated["trace"];
        let checks = trace["leader_check"].as_array().unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0]["verdict"], json!("gap"));
        assert_eq!(checks[0]["answers"][0]["requirement"], json!("Truth 1"));
        assert_eq!(checks[0]["answers"][0]["artifact"], json!("src/lib.rs"));

        // Verify stored file has status "capped"
        let stored = read_cell_fixture(root, "c-gap");
        assert_eq!(stored["status"], json!("capped"));
    }

    #[test]
    fn ok_verdict_appends_and_clears_debt() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let p = root.join("src/lib.rs");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "// lib").unwrap();

        let stamp_now = "2026-09-21T12:00:00.000Z";
        write_cell_fixture(
            root,
            "c-debt",
            &make_capped_cell("c-debt", &["Truth 1"], &["src/lib.rs"], Some(stamp_now)),
        );

        // Before check, counts as debt
        let debt_before = feature_leader_check_debt(root, "feat-1").unwrap();
        assert_eq!(debt_before.count, 1);
        assert_eq!(debt_before.ids, vec![json!("c-debt")]);

        // Record gap: stays debt
        let payload = json!({
            "schema": "leader-check/1",
            "answers": [
                {"requirement": "Truth 1", "artifact": "src/lib.rs"}
            ]
        });
        record_leader_check(root, "c-debt", "gap", &payload, None, false).unwrap();
        let debt_gap = feature_leader_check_debt(root, "feat-1").unwrap();
        assert_eq!(debt_gap.count, 1);

        // Record ok: debt clears
        record_leader_check(root, "c-debt", "ok", &payload, None, false).unwrap();
        let debt_ok = feature_leader_check_debt(root, "feat-1").unwrap();
        assert_eq!(debt_ok.count, 0);

        let stored = read_cell_fixture(root, "c-debt");
        let checks = stored["trace"]["leader_check"].as_array().unwrap();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0]["verdict"], json!("gap"));
        assert_eq!(checks[1]["verdict"], json!("ok"));
    }

    #[test]
    fn grandfathering_by_stamp_and_missing_capped_at() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // 1. Capped before stamp: grandfathered
        write_cell_fixture(
            root,
            "c-old",
            &make_capped_cell("c-old", &[], &[], Some("2026-09-20T23:59:59.000Z")),
        );

        // 2. Missing capped_at: grandfathered (fail-open)
        write_cell_fixture(
            root,
            "c-nocap",
            &make_capped_cell("c-nocap", &[], &[], None),
        );

        // 3. Unparseable capped_at: grandfathered (fail-open)
        let mut unparseable = make_capped_cell("c-corrupt", &[], &[], None);
        unparseable["trace"]["capped_at"] = json!("not-a-date");
        write_cell_fixture(root, "c-corrupt", &unparseable);

        // 4. Capped at or after stamp: debt
        write_cell_fixture(
            root,
            "c-new",
            &make_capped_cell("c-new", &[], &[], Some("2026-09-21T01:00:00.000Z")),
        );

        let debt = feature_leader_check_debt(root, "feat-1").unwrap();
        assert_eq!(debt.count, 1);
        assert_eq!(debt.ids, vec![json!("c-new")]);
    }

    // ─── commit-diff check (decision 5e3f0baf) ──────────────────────────────

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the commit-diff fixtures");
        assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    /// A git repo with an init commit, plus src/a.rs and src/b.rs on disk.
    fn diff_repo(root: &Path) {
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("f.txt"), "x").unwrap();
        std::fs::write(root.join("src/a.rs"), "// a").unwrap();
        std::fs::write(root.join("src/b.rs"), "// b").unwrap();
        git_ok(root, &["init", "-q", "-b", "main", "."]);
        git_ok(root, &["config", "user.email", "a@b.c"]);
        git_ok(root, &["config", "user.name", "t"]);
        git_ok(root, &["add", "f.txt"]);
        git_ok(root, &["commit", "-qm", "init"]);
    }

    fn commit_paths(root: &Path, paths: &[&str], message: &str) {
        let mut args = vec!["add"];
        args.extend_from_slice(paths);
        git_ok(root, &args);
        git_ok(root, &["commit", "-qm", message]);
    }

    fn a_payload() -> Value {
        json!({
            "schema": "leader-check/1",
            "answers": [{"requirement": "Truth A", "artifact": "src/a.rs"}]
        })
    }

    #[test]
    fn claimed_file_in_trailer_commit_passes_with_commit_diff_checked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        diff_repo(root);
        commit_paths(root, &["src/a.rs"], "Add a\n\ncell: c-1");
        write_cell_fixture(root, "c-1", &make_capped_cell("c-1", &["Truth A"], &["src/a.rs"], None));
        let updated = record_leader_check(root, "c-1", "ok", &a_payload(), None, false).unwrap();
        assert_eq!(updated["trace"]["leader_check"][0]["commit_diff"], json!("checked"));
        assert_eq!(updated["status"], json!("capped"));
    }

    #[test]
    fn claimed_file_absent_from_trailer_commit_is_refused_naming_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        diff_repo(root);
        commit_paths(root, &["src/a.rs"], "Add a\n\ncell: c-1");
        commit_paths(root, &["src/b.rs"], "Add b under another cell\n\ncell: c-2");
        write_cell_fixture(
            root,
            "c-1",
            &make_capped_cell("c-1", &["Truth A"], &["src/a.rs", "src/b.rs"], None),
        );

        let msg = thrown(record_leader_check(root, "c-1", "ok", &a_payload(), None, false));
        assert!(msg.contains("files_changed names path(s) the cell's commit(s) never touched: src/b.rs."), "{msg}");
        assert_eq!(read_cell_fixture(root, "c-1")["status"], json!("capped"));
    }

    #[test]
    fn no_trailer_commit_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        diff_repo(root);
        commit_paths(root, &["src/a.rs"], "Add a, mentions c-1 only in prose");
        write_cell_fixture(root, "c-1", &make_capped_cell("c-1", &["Truth A"], &["src/a.rs"], None));

        let msg = thrown(record_leader_check(root, "c-1", "ok", &a_payload(), None, false));
        assert!(msg.contains("has no commit carrying the trailer \"cell: c-1\" in the last 50 commit(s)"), "{msg}");
        assert!(msg.contains("files_changed cannot be verified against a real diff."), "{msg}");
    }

    #[test]
    fn no_trailer_commit_with_commit_pending_passes_with_skip_value() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        diff_repo(root);
        let mut cell = make_capped_cell("c-1", &["Truth A"], &["src/a.rs"], None);
        cell["trace"]["commit_pending"] = json!("committed by the leader later");
        write_cell_fixture(root, "c-1", &cell);

        let updated = record_leader_check(root, "c-1", "ok", &a_payload(), None, false).unwrap();
        assert_eq!(updated["trace"]["leader_check"][0]["commit_diff"], json!("skipped: commit_pending"));
    }

    #[test]
    fn non_git_root_passes_with_git_unavailable_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/a.rs"), "// a").unwrap();
        write_cell_fixture(root, "c-1", &make_capped_cell("c-1", &["Truth A"], &["src/a.rs"], None));

        let updated = record_leader_check(root, "c-1", "ok", &a_payload(), None, false).unwrap();
        assert_eq!(updated["trace"]["leader_check"][0]["commit_diff"], json!("skipped: git unavailable"));
    }
}
