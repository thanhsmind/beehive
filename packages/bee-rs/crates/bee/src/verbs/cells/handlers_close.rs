// verb handlers: cap, finish, block, drop, unclaim, reopen, tier
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt};
use crate::state as bstate;
// dol-1: the ONE rendering of a deviation entry, borrowed from the reader
// that mines them (`verbs/knowledge/promote.rs`) rather than copied — two
// copies of those match arms would drift, and the cap's dedup must agree
// with the miner's own idea of what counts as the same deviation.
use crate::verbs::knowledge::deviation_text;
// hm-2 (docs/history/human-mailbox/CONTEXT.md D4): the entry layer a
// clean stop appends to the moment it happens. See the three-stops map
// above `record_cap_in_mailbox`.
use crate::verbs::mailbox;
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

// ── cells cap / cells finish ───────────────────────────────────────────────

pub(crate) const CAP_FLAGS: [&str; 13] = [
    "id",
    "outcome",
    "files",
    "deviations-file",
    "deviation",
    "friction",
    "override-judge",
    "session-id",
    "force-ownership",
    "commit-pending",
    "inline-reason",
    "report",
    "sync-ack",
];

/// resolveDeclaredBehaviorChange (E6).
pub(crate) fn resolve_declared_behavior_change(cell: &Map<String, Value>) -> bool {
    match cell.get("behavior_change") {
        Some(Value::Bool(true)) => true,
        Some(Value::Bool(false)) => false,
        _ => {
            let trace = cell.get("trace");
            matches!(trace, Some(t) if js_truthy(t))
                && matches!(
                    trace.and_then(|t| t.get("behavior_change")),
                    Some(Value::Bool(true))
                )
        }
    }
}

/// parseDeviationsFile.
pub(crate) fn parse_deviations_file(file: &str) -> MR<Vec<Value>> {
    let raw = read_file_text(file, "deviations")?;
    match parse_json_js(&raw, false) {
        JsParse::Value(Value::Array(a)) => Ok(a),
        JsParse::Value(other) => Ok(vec![Value::String(jsjson::js_to_string(&other))]),
        // Node's `catch`: not JSON -> one deviation per non-blank line. A
        // lone-surrogate escape now takes that same branch.
        JsParse::NotJson => Ok(raw
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .filter(|l| !js_trim(l).is_empty())
            .map(|l| Value::String(l.to_string()))
            .collect()),
    }
}

pub(crate) struct CapFlags {
    pub(crate) id: String,
    pub(crate) outcome: Option<String>,       // flags.outcome ? String : undefined
    pub(crate) friction: Option<String>,      // flags.friction ? String : null
    pub(crate) files_changed: Vec<Value>,
    pub(crate) deviations: Vec<Value>,
    /// frd-1: `--deviation "<one line>"`, RAW (untrimmed) — `None` = not
    /// passed. Validated (refused if it trims to empty) and trimmed at cap
    /// time in `cap_cell_from_flags`, then appended to `deviations` above.
    /// dol-1: this flag is for a deviation the ORCHESTRATOR observed and
    /// the worker did NOT report — a worker's own structured deviations
    /// (`--report`'s `deviations` array) are merged into `trace.deviations`
    /// by `cap_cell_from_flags` itself, so nobody re-types a report line
    /// here to get it into the pattern-candidate mining
    /// `verbs/knowledge/promote.rs` reads. The flag parser (`rsv::Flags`,
    /// mirroring `--did` on `capture add`) is single-value — a repeated
    /// `--deviation` keeps only the LAST occurrence — so this carries one
    /// line per cap/finish call, same discipline as every other value flag
    /// here.
    pub(crate) deviation: Option<String>,
    pub(crate) override_reason: String,       // trimmed; '' = none
    pub(crate) session_flag: Option<String>,
    pub(crate) force_ownership: bool,
    /// D6 (hook-teeth CONTEXT.md): `--commit-pending <reason>`, trimmed;
    /// `None` = not passed. Escapes the commit-trailer check below and is
    /// recorded on the capped cell's own `trace.commit_pending`.
    pub(crate) commit_pending: Option<String>,
    /// wp-1 (AGENTS.md "never zero execution workers"): `--inline-reason
    /// <reason>`, trimmed; `None` = not passed. Escapes the registered-
    /// worker check below for a small+ cell and is recorded on the capped
    /// cell's own `trace.inline_reason`.
    pub(crate) inline_reason: Option<String>,
    /// wfl-1/D8: `--report <json-string>`, RAW — validated against the
    /// worker Result-form shape (finish_support.rs's `parse_report_flag`)
    /// before any write, then stored verbatim as `trace.report`. `None`
    /// means the flag was not passed on the command line; `cap_cell_from_flags`
    /// now REFUSES that case (D8: --report is required on every cap path)
    /// rather than silently leaving `trace.report` untouched.
    pub(crate) report: Option<String>,
    /// D3/D4: `--sync-ack "<reason>"`, RAW — escapes the sync door (ownership,
    /// applied_at, prediction) when non-empty. Stored on trace.sync_ack and
    /// appended to trace.deviations.
    pub(crate) sync_ack: Option<String>,
}

/// wp-1: is `worker` (the cap's own `trace.worker`) a REGISTERED worker for
/// cell `id` — an entry in state.json's `workers[]` (the shape `bee state
/// worker add --nickname N --cell ID --tier T --status S` writes, see
/// verbs/state_group/workers.rs's `run_worker_add`) whose own `nickname`
/// matches `worker` AND whose own `cell` matches `id`? A missing/empty
/// `worker`, a missing/corrupt/non-array `workers` key, or a missing
/// state.json altogether all answer `false` — fail CLOSED, matching this
/// check's own "unless --inline-reason" contract (an unprovable registry is
/// never silently treated as satisfying the rule).
pub(crate) fn registered_worker_for_cell(root: &Path, id: &str, worker: Option<&str>) -> MR<bool> {
    let Some(worker) = worker else { return Ok(false) };
    let state = read_store_json(&bstate::state_path(root))?;
    let workers = match state.as_ref().and_then(|s| s.get("workers")) {
        Some(Value::Array(a)) => a,
        _ => return Ok(false),
    };
    Ok(workers.iter().any(|w| {
        js_truthy(w)
            && matches!(w.get("nickname"), Some(Value::String(n)) if n == worker)
            && matches!(w.get("cell"), Some(Value::String(c)) if c == id)
    }))
}

/// capCellFromFlags — the ONE cap door cap and finish share.
/// decision 13ce1858 (test-cadence-boundary D1): no test command runs here
/// anymore — the declared-test cwd split (`test_root`) this signature used
/// to carry is gone with it. Every read/write here is keyed at `root`, the
/// cell/claim store — always MAIN's.
pub(crate) fn cap_cell_from_flags(root: &Path, f: &CapFlags, finish: bool) -> MR<Value> {
    let id = &f.id;
    // Pre-scan (see the pre-scan section header).
    prescan_claim(root, id)?;
    let commands = read_commands_slice(root)?;
    if !f.override_reason.is_empty() {
        delegate_only(load_taxonomy(root))?;
    }
    if finish {
        delegate_only(list_path_lease_records(root))?;
        delegate_only(read_holds_store(root))?;
    }
    // Cheap pre-checks BEFORE the (possibly long) test run.
    let existing = read_cell_norm(root, id)?;
    let Some(existing) = existing else {
        return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
    };
    let Value::Object(existing_map) = &existing else { return Err(Fail::Delegate) };
    match existing_map.get("status") {
        Some(Value::String(s)) if s == "capped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
        }
        Some(Value::String(s)) if s == "dropped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
        }
        _ => {}
    }
    delegate_only(merge_trace(existing_map.get("trace")))?;

    // frd-1: `--deviation` shape check, BEFORE the (possibly long) test run
    // and before any write — an empty or whitespace-only value is refused by
    // name; nothing is written on refusal, matching `--files a.js,b.js`-style
    // required-value checks elsewhere in this function.
    if let Some(raw) = &f.deviation {
        if js_trim(raw).is_empty() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" refused — --deviation requires a non-empty value (a blank or whitespace-only line records nothing to trace.deviations)."
            )));
        }
    }

    // D3/D4: `--sync-ack` shape check
    if let Some(raw) = &f.sync_ack {
        if js_trim(raw).is_empty() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" refused — --sync-ack requires a non-empty value (a blank or whitespace-only reason cannot escape the sync door)."
            )));
        }
    }

    // D8 (docs/history/test-doctrine/CONTEXT.md): `--report` is now
    // REQUIRED on every cap path — `cells cap` and `cells finish` share
    // this one door, so a cap left without a proof string would
    // grandfather itself as a permanent legacy record, the exact hole the
    // doors' proof-check exists to close. Shape check runs BEFORE the
    // (possibly long) test run and before any write — same "refuse before
    // doing real work" posture as `--deviation` above.
    let report_value: Value = match &f.report {
        Some(raw) => parse_report_flag(raw)?,
        None => {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" refused — --report is required: a JSON object with keys {} (tests as a D8 proof string \"<command> — <result> — <scope reason>\", e.g. \"cargo test -p bee — green — touched close.rs\").",
                REPORT_KEYS.join(", ")
            )))
        }
    };

    // decision 13ce1858 (test-cadence-boundary D1): the one test door used
    // to run the declared command HERE and refuse a red cap — that run and
    // its red-refusal path are gone, for both `cells finish` and
    // `cells cap` (no `!finish` guard ever separated them). D7 (docs/
    // history/test-doctrine/CONTEXT.md) retired the run at the boundary too:
    // `bee close` and `bee worktree merge` no longer spawn `commands.test`
    // either — they read the D8 proof string recorded on `trace.report`
    // instead (verbs/cells/proof.rs `feature_proof_check`). A cap is
    // commit-only proof — `declared` below only decides which legacy
    // sentinel (`boundary` vs `undeclared`) `trace.tests` itself still
    // records, at `:capped_at` time near the end of this function; nothing
    // here spawns a process, and the doors no longer read that field.
    let declared = commands
        .test
        .as_ref()
        .map(|list| list.iter().filter(|c| *c != NO_TEST_SENTINEL).cloned().collect::<Vec<_>>())
        .filter(|l| !l.is_empty());

    // D6 (hook-teeth CONTEXT.md) — the one-commit-per-cell trailer check.
    // Runs AFTER the test-green door above (a red run refuses first, same
    // cell — D7 sequencing) but BEFORE the per-cell lock, reading only
    // already-fetched state (`existing_map`, `f`), same posture as the
    // pre-checks above. D6 scopes this to `cells finish` alone — `cells cap`
    // (finish == false) never runs it, even for a non-empty files_changed.
    if finish && !f.files_changed.is_empty() {
        let feature = match existing_map.get("feature") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.as_str()),
            _ => None,
        };
        // The history root to scan: THIS feature's granted worktree HEAD
        // history when one exists — necessary because `cells finish` for a
        // feature commonly runs from the MAIN checkout (bee-swarming's
        // worker convention) while the worker's own commits land on that
        // feature's WORKTREE branch, never on main; falling back to `root`
        // (an ordinary same-checkout feature, or no worktree split at all)
        // otherwise.
        let history_root = commit_trailer_history_root(root, feature);
        if !commit_trailer_present(&history_root, id) && f.commit_pending.is_none() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" refused — one commit per cell: no commit in the last {COMMIT_TRAILER_WINDOW} commit(s) of {} carries the trailer \"{}\". Commit the work with that trailer, then retry \"bee cells finish --id {id}\" — or pass --commit-pending \"<reason>\" to finish anyway (the reason is stored on the cell's own trace.commit_pending).",
                history_root.display(),
                cell_commit_trailer(id),
            )));
        }
    }

    // capCell (lib/cells.mjs) under the per-cell lock.
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        assert_not_archived(root, "capCell", id)?;
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        let bc = resolve_declared_behavior_change(&cell_map);
        match cell_map.get("status") {
            Some(Value::String(s)) if s == "capped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
            }
            Some(Value::String(s)) if s == "dropped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
            }
            _ => {}
        }
        let mut trace = merge_trace(cell_map.get("trace"))?;
        trace = guard_claim_ownership(
            root,
            id,
            trace,
            "capCell",
            f.session_flag.as_deref(),
            f.force_ownership,
        )?;
        let judge_entries: Vec<Value> = match trace.get("semantic_judge") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let latest_judge = judge_entries.last().cloned();
        let latest_needs_revision = matches!(
            latest_judge.as_ref().filter(|l| js_truthy(l)).and_then(|l| l.get("verdict")),
            Some(Value::String(s)) if s == "NEEDS_REVISION"
        );
        if latest_needs_revision && f.override_reason.is_empty() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" has a NEEDS_REVISION semantic-judge verdict — rework the cell and record a PASS verdict (bee cells judge-record), or cap with an audited override (bee cells cap --id {id} --override-judge \"<reason>\")."
            )));
        }
        if !f.override_reason.is_empty() {
            let overrides: Vec<Value> = match trace.get("judge_overrides") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut entry = Map::new();
            entry.insert("overridden_at".into(), Value::String(utc_now()));
            entry.insert("reason".into(), Value::String(f.override_reason.clone()));
            match &latest_judge {
                None => {
                    entry.insert("last_verdict".into(), Value::Null);
                }
                Some(l) => match l.get("verdict") {
                    Some(v) => {
                        entry.insert("last_verdict".into(), v.clone());
                    }
                    None => {} // {last_verdict: undefined} — JSON.stringify drops the key
                },
            }
            let worker_disp = match trace.get("worker") {
                Some(w) if js_truthy(w) => jsjson::js_to_string(w),
                _ => "unknown".to_string(),
            };
            log_decision(
                root,
                &format!(
                    "«cells cap: cell \"{id}\" judge override by {worker_disp} — {}»",
                    f.override_reason
                ),
                "Audited cap over a NEEDS_REVISION (or absent) semantic-judge verdict (D-GHF-C, GH #27.5) — the verdict itself is never rewritten, only a judge_overrides marker appended.",
                &["cells", "judge"],
            )?;
            let mut next = overrides;
            next.push(Value::Object(entry));
            trace.insert("judge_overrides".into(), Value::Array(next));
        }
        let lane = match cell_map.get("lane") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        if lane == "small" || lane == "standard" || lane == "high-risk" {
            if f.files_changed.is_empty() {
                return Err(Fail::Thrown(format!(
                    "capCell: lane \"{lane}\" cell \"{id}\" requires non-empty files_changed (--files a.js,b.js) — record what the worker actually touched. A cell that changed nothing is a drop or a NOOP, not a cap."
                )));
            }
            // wp-1 (AGENTS.md "never zero execution workers"): from `small`
            // up, the cell's claiming worker (trace.worker) must be a
            // REGISTERED worker for THIS cell — an entry in state.json's
            // workers[] whose own `cell` names this id (the shape `bee
            // state worker add` writes). `tiny` never reaches this branch,
            // matching AGENTS.md's own carve-out ("a tiny cell may run
            // inline"). `--inline-reason` is the named-deviation escape,
            // recorded on the cap's own trace below.
            if f.inline_reason.is_none() {
                let worker = match trace.get("worker") {
                    Some(Value::String(w)) if !w.is_empty() => Some(w.as_str()),
                    _ => None,
                };
                if !registered_worker_for_cell(root, id, worker)? {
                    let worker_disp = worker.unwrap_or("unknown");
                    return Err(Fail::Thrown(format!(
                        "capCell: lane \"{lane}\" cell \"{id}\" refused — no registered execution worker: trace.worker \"{worker_disp}\" does not appear in state.json workers[] with cell \"{id}\" (AGENTS.md: cells from small up run through dispatched workers, never zero execution workers). FIX: dispatch this cell to a registered worker (bee state worker add --nickname <nickname> --cell {id} --tier <tier> --status running), then retry — or re-run with --inline-reason \"<why>\" to record the named deviation on this cap's own trace (trace.inline_reason)."
                    )));
                }
            }
        }
        if lane == "high-risk" && f.outcome.as_deref().map(|o| js_trim(o).is_empty()).unwrap_or(true) {
            return Err(Fail::Thrown(format!(
                "capCell: high-risk cell \"{id}\" requires an outcome summary."
            )));
        }
        cell_map.insert("status".into(), Value::String("capped".into()));
        trace.insert("files_changed".into(), Value::Array(f.files_changed.clone()));
        // frd-1: `--deviations-file` first (unchanged order), then the
        // single `--deviation` line, already validated non-empty above —
        // both may be passed together, each contributes its own lines.
        let mut deviations = f.deviations.clone();
        if let Some(raw) = &f.deviation {
            let trimmed = js_trim(raw);
            if !trimmed.is_empty() {
                deviations.push(Value::String(trimmed.to_string()));
            }
        }
        // dol-1: last, the worker's OWN structured deviations — the
        // `deviations` array of the validated `--report` object below.
        // `parse_report_flag` already proved it is an array, so this reads
        // it, never re-validates or re-parses the raw flag. A deviation the
        // worker recorded structurally used to reach only `trace.report`,
        // which `bee knowledge promote` never reads (it mines
        // `trace.deviations` alone), so the lesson stayed invisible until
        // someone hand-copied it into `--deviation`. This is a UNION, never
        // a move: `trace.report` keeps its verbatim copy (D8, below).
        //
        // The two sources stay SYMMETRIC. A string entry is trimmed and an
        // empty one dropped, exactly as `--deviation` above; anything else
        // passes through VERBATIM, exactly as a `--deviations-file` entry
        // already does. Mining reads a non-string entry fine — knowledge's
        // own `deviation_text` has a first-class `{type, description}` arm,
        // and a live cell already carries that shape from a deviations-file
        // — so skipping one here would leave this cell's own defect alive
        // in one branch: dropped silently on the report side, mined on the
        // file side. Tolerated rather than refused, because a refusal would
        // block a cap mid-flight over a shape that has never occurred.
        // Dedup is by that same `deviation_text` rendering rather than raw
        // equality, so an object and the string it renders to count as ONE
        // deviation; the earlier source keeps its place and its form.
        if let Some(Value::Array(reported)) = report_value.get("deviations") {
            for entry in reported {
                let candidate = match entry {
                    Value::String(raw) => {
                        let trimmed = js_trim(raw);
                        if trimmed.is_empty() {
                            continue;
                        }
                        Value::String(trimmed.to_string())
                    }
                    other => other.clone(),
                };
                let text = deviation_text(&candidate);
                if !deviations.iter().any(|d| deviation_text(d) == text) {
                    deviations.push(candidate);
                }
            }
        }
        // D3/D4: --sync-ack reason recorded on trace.sync_ack and appended to trace.deviations
        if let Some(raw) = &f.sync_ack {
            let trimmed = js_trim(raw);
            if !trimmed.is_empty() {
                trace.insert("sync_ack".into(), Value::String(trimmed.to_string()));
                let ack_line = format!("sync-ack: {trimmed}");
                if !deviations.iter().any(|d| matches!(d, Value::String(s) if s == &ack_line)) {
                    deviations.push(Value::String(ack_line));
                }
            }
        }
        trace.insert("deviations".into(), Value::Array(deviations));
        trace.insert(
            "friction".into(),
            f.friction.clone().map(Value::String).unwrap_or(Value::Null),
        );
        trace.insert("behavior_change".into(), Value::Bool(bc));
        // D6: the --commit-pending reason, when passed — recorded so a cold
        // reader can see WHY this cap escaped the commit-trailer check
        // without re-deriving it from git history.
        if let Some(reason) = &f.commit_pending {
            trace.insert("commit_pending".into(), Value::String(reason.clone()));
        }
        // wp-1: the --inline-reason reason, when passed — recorded so a cold
        // reader can see WHY this small+ cap escaped the registered-worker
        // check without re-deriving it from state.json's workers[] history.
        if let Some(reason) = &f.inline_reason {
            trace.insert("inline_reason".into(), Value::String(reason.clone()));
        }
        // D8: the validated --report object — stored verbatim
        // (`parse_report_flag` already proved the exact five keys). Always
        // present now that --report is required on every cap path.
        trace.insert("report".into(), report_value.clone());
        let outcome_value = match &f.outcome {
            Some(o) if !js_trim(o).is_empty() => Value::String(o.clone()),
            _ => trace.get("outcome").cloned().unwrap_or(Value::Null),
        };
        trace.insert("outcome".into(), outcome_value);
        trace.insert("capped_at".into(), Value::String(utc_now()));
        // fa-1: diff-vs-test advisory — the ONLY producer for this slot
        // since the E1 impact-registry check retired. Scoped to `cells
        // finish` alone (D6's own "finish only" posture): `cells cap`
        // (finish == false) never shells out to git here. Every earlier
        // door above (test-green, D6's commit-trailer check, the lane/
        // worker checks) already let this cap through, so the block below
        // can only ever ADD a line — it never refuses, and a git failure of
        // any kind is a silent skip (finish_support.rs's
        // diff_vs_test_advisory).
        let mut warnings: Vec<Value> = Vec::new();
        if finish {
            let feature = match cell_map.get("feature") {
                Some(Value::String(s)) if !s.is_empty() => Some(s.as_str()),
                _ => None,
            };
            let advisory_root = commit_trailer_history_root(root, feature);
            let threshold = advisory_untested_lines_threshold(&bstate::read_config_raw(root));
            if let Some(line) = diff_vs_test_advisory(&advisory_root, id, threshold) {
                eprintln!("{line}");
                warnings.push(Value::String(line));
            }
        }
        trace.insert("warnings".into(), Value::Array(warnings));
        // D1a (cap record honesty): a cap never claims a test run it did not
        // perform. A declared-test repo records `boundary` — tests prove at
        // `bee close`/`bee worktree merge`, not here; the `undeclared`
        // sentinel is unchanged for a repo with no declared test command.
        // `trace.results`/`trace.ran_at` are no longer written at cap —
        // there is no run to point at.
        trace.insert(
            "tests".into(),
            Value::String(if declared.is_some() { "boundary" } else { "undeclared" }.into()),
        );

        // D3/D4: the sync door check (ownership, applied_at, prediction).
        // Skipped when a non-blank --sync-ack was passed.
        if f.sync_ack.as_deref().map(js_trim).unwrap_or("").is_empty() {
            let feature = match cell_map.get("feature") {
                Some(Value::String(s)) if !s.is_empty() => Some(s.as_str()),
                _ => None,
            };
            let history_root = commit_trailer_history_root(root, feature);
            let mut touched_set: Vec<String> = Vec::new();
            if let Some(rows) = head_commit_numstat(&history_root) {
                for row in rows {
                    let norm = normalize_cell_path(&row.path);
                    if !norm.is_empty() && !touched_set.contains(&norm) {
                        touched_set.push(norm);
                    }
                }
            }
            for f_val in &f.files_changed {
                if let Some(s) = f_val.as_str() {
                    let norm = normalize_cell_path(s);
                    if !norm.is_empty() && !touched_set.contains(&norm) {
                        touched_set.push(norm);
                    }
                }
            }
            if let Some(refusal) = sync_refusal(root, &cell_map, &touched_set) {
                return Err(Fail::Thrown(refusal));
            }
            // D3/D4: legacy cell predates affects_skills — check (c) is
            // skipped by `sync_refusal` for it. Only worth a deviation line
            // when it actually mattered — the touched set carries a
            // skills/** path, so a non-legacy cell would have had its
            // prediction checked here. A cell that never touches skills/
            // stays byte-identical to a pre-koh-5 cap (a legacy cell that
            // predicted nothing, checked against nothing touched, has
            // nothing to make visible).
            if cell_map.get("affects_skills").is_none()
                && touched_set.iter().any(|p| path_under_root(p, "skills"))
            {
                if let Some(Value::Array(devs)) = trace.get_mut("deviations") {
                    let legacy_line = "sync: no prediction on legacy cell";
                    if !devs.iter().any(|d| matches!(d, Value::String(s) if s == legacy_line)) {
                        devs.push(Value::String(legacy_line.to_string()));
                    }
                }
            }
        }

        cell_map.insert("trace".into(), Value::Object(trace));
        let cell_value = Value::Object(cell_map);
        write_cell(root, &cell_value)?;
        Ok(cell_value)
    })();
    guard.release();
    let mut saved = saved?;
    release_claim_file_best_effort(root, id); // D1 Δ2: cap clears the claim
    // D4 (human-mailbox): the cap is on disk and the claim is released —
    // this stop has HAPPENED, so its entry is appended now, not gathered
    // at the end of the run. Never refuses (D10), never conditional on
    // arming (D9). See `record_cap_in_mailbox`.
    record_cap_in_mailbox(root, f, &saved, &report_value);
    // merge-ready-fact D1: the cap that leaves NOTHING outstanding for this
    // feature is the one moment bee can know the feature is finished in its
    // worktree, so this is where the stored fact is written. Everything about
    // it is fail-open (`set_after_cap` never returns an error): the cap above
    // is already committed to disk, and no failure to record a board-facing
    // convenience may turn a landed cap into a refusal.
    //
    // The answer rides the cap RESULT under `merge_ready` — `null` on every
    // arm that wrote nothing (an open sibling, no worktree grant, no record
    // for the feature), so the key's presence is stable and only its value
    // moves. The cell FILE is untouched: `write_cell` already ran above, and
    // this fact belongs to the feature, not the cell.
    let merge_ready = match saved.get("feature") {
        Some(Value::String(feature)) => {
            let feature = feature.clone();
            crate::verbs::workflow_store::merge_ready::set_after_cap(root, &feature, &utc_now())
        }
        _ => None,
    };
    if let Value::Object(map) = &mut saved {
        map.insert("merge_ready".into(), merge_ready.unwrap_or(Value::Null));
    }
    Ok(saved)
}

// ── the human mailbox: D4's three clean stops ──────────────────────────────
//
// plan.md deferred one question to this cell: "cap, feature close and blocker
// are three code paths; missing one silently truncates a letter." Traced
// outward from this file, and written down here for the cells that wire the
// two kinds hm-2 does not:
//
//   1. cap — `bee cells cap` and `bee cells finish` are ONE door.
//      `run_cap(finish)` (this file) parses the flags; with `finish == false`
//      it calls `cap_cell_from_flags(root, f, false)` directly, and with
//      `finish == true` it detours through `run_finish` ->
//      `finish_cap_and_release` (which re-roots the cell ledger onto the main
//      checkout and releases reservations afterwards) into
//      `cap_cell_from_flags(root, f, true)`. So ONE hook inside that function
//      covers both verbs, and that hook is `record_cap_in_mailbox` below —
//      the only stop wired today.
//   2. feature close — `bee close --feature <f>`: `verbs/drivers/close.rs`
//      `run_close` -> `close_handler(root, feature, dry_run, …)`. One door
//      too, but with a fork the cap has no equivalent of: `--dry-run` lists
//      the doors, writes nothing and stops nothing, so it must append
//      NOTHING. The entry belongs on the non-dry-run tail, once the doors
//      have passed. (D14's feature-close LETTER is phase 4; the feature-close
//      ENTRY is what `mailbox::KIND_FEATURE_CLOSE` is for.)
//   3. blocker — `bee cells block --id <id> --reason <why>`: `run_block`
//      (this file) -> `mutate_cell(root, id, "blockCell", …)`, which writes
//      `status: "blocked"` and `trace.blocked_reason` under the per-cell
//      lock. The stop is complete when `mutate_cell` returns the cell, and
//      the reason it just stored is the sentence's raw material.
//
// Deliberately NOT stops: `cells drop` (work abandoned, not done), `cells
// unclaim` and `cells reopen` (a claim moved; nothing finished), and
// `bee close --dry-run` above. A letter reports what a run DID.

/// D4/D8/D9 (docs/history/human-mailbox/CONTEXT.md): record this cap as one
/// human-mailbox entry, THE MOMENT the cap lands.
///
/// Called after the cell file is written and the claim released — the stop
/// has happened, and nothing below may undo it. That ordering is the whole
/// point of D4's entry layer: a run that dies at 3am must still leave
/// everything up to the moment it died, so an entry is appended per stop
/// rather than gathered at the end of a run.
///
/// UNCONDITIONAL, by D9: every session appends its entries, attended or not.
/// Arming (`mailbox::armed`) decides only whether a letter is composed at the
/// end of the run — a session that starts attended and becomes an overnight
/// run must carry a complete record of its whole span, which it cannot do if
/// the appends waited for the arming answer.
///
/// FAIL-OPEN, by D10: `mailbox::record_stop` warns and returns. A cap in a
/// run that files no letter keeps the byte-identical behaviour it had before
/// this feature — nothing here refuses a cap, and no cap flag changes shape.
fn record_cap_in_mailbox(root: &Path, f: &CapFlags, capped: &Value, report: &Value) {
    // The run is the SESSION's span (D9, D12 — see `mailbox::run_id`): the
    // caller's own session, resolved through the ordinary chain, and the
    // claim's recorded session when this process has no session of its own
    // (a cap typed in a plain shell for a cell a session claimed).
    let session = resolve_session_flag_env(f.session_flag.as_deref()).or_else(|| {
        match capped.get("trace").and_then(|t| t.get("claim_session")) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
            _ => None,
        }
    });
    let run = mailbox::run_id(session.as_deref());

    // Everything below is already in hand — the cap invents nothing.
    let report_line = |key: &str| match report.get(key) {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    };
    let entry = mailbox::Entry {
        at: utc_now(),
        kind: mailbox::KIND_CAP.to_string(),
        // D8: the plain-language sentence is written HERE, at the moment of
        // the event, never at composition.
        what: mailbox::cap_sentence(
            f.outcome.as_deref(),
            capped.get("title").and_then(Value::as_str),
        ),
        files: f.files_changed.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
        // "none" is the Result form's own word for "no commit", not a sha.
        commit: report_line("commit").filter(|c| c != "none"),
        // D8's proof line, exactly as the worker recorded it.
        proof: report_line("tests"),
        // Only a report deviation that already carries D5's three parts can
        // be read as a departure. A free-form line has no `why` and no
        // `kind`, and inventing either would be authoring (D8) — phase 2
        // makes the three parts the required shape.
        departure: report
            .get("deviations")
            .and_then(Value::as_array)
            .and_then(|entries| entries.iter().find_map(mailbox::Departure::from_value)),
        // D13's Needs-your-call items have no source at a cap: a cap records
        // finished work, and a question that blocks something is a blocker
        // (stop kind 2 above) or a gate. Empty rather than guessed.
        needs_you: Vec::new(),
    };
    mailbox::record_stop(root, &run, &entry);
}

pub(crate) fn cap_flags_from(flags: &rsv::Flags) -> Option<CapFlags> {
    let id = flags.req_str("id")?.to_string();
    let outcome = flags.truthy_str("outcome").map(str::to_string);
    let friction = flags.truthy_str("friction").map(str::to_string);
    let files_changed: Vec<Value> = flags
        .truthy_str("files")
        .map(|s| {
            s.split(',')
                .map(js_trim)
                .filter(|p| !p.is_empty())
                .map(|p| Value::String(p.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let override_reason = match opt_string_flag(flags, "override-judge")? {
        Some(s) => js_trim(&s).to_string(),
        None => String::new(),
    };
    let session_flag = opt_string_flag(flags, "session-id")?;
    // frd-1: raw (untrimmed) on purpose — `cap_cell_from_flags` refuses a
    // whitespace-only value by name instead of this probe silently treating
    // it as absent.
    let deviation = opt_string_flag(flags, "deviation")?;
    let force_ownership = bool_flag(flags, "force-ownership")?;
    // D2's --fix-first convention: trimmed; empty/absent = None.
    let commit_pending = match opt_string_flag(flags, "commit-pending")? {
        Some(s) if !js_trim(&s).is_empty() => Some(js_trim(&s).to_string()),
        _ => None,
    };
    // wp-1: same --fix-first convention as --commit-pending above.
    let inline_reason = match opt_string_flag(flags, "inline-reason")? {
        Some(s) if !js_trim(&s).is_empty() => Some(js_trim(&s).to_string()),
        _ => None,
    };
    // wfl-1: raw (untrimmed, unparsed) on purpose — `cap_cell_from_flags`
    // owns the actual validation via `parse_report_flag`, same split
    // `--deviation`'s own probe takes above.
    let report = opt_string_flag(flags, "report")?;
    let sync_ack = opt_string_flag(flags, "sync-ack")?;
    Some(CapFlags {
        id,
        outcome,
        friction,
        files_changed,
        deviations: Vec::new(), // filled inside dispatch (file read may throw)
        deviation,
        override_reason,
        session_flag,
        force_ownership,
        commit_pending,
        inline_reason,
        report,
        sync_ack,
    })
}

pub(crate) fn cap_text(cell: &Value) -> String {
    let trace = cell.get("trace");
    let capped_at = trace.and_then(|t| t.get("capped_at"));
    let tests = trace.and_then(|t| t.get("tests"));
    format!(
        "Capped {} at {} (tests: {}).",
        js_string_or_undefined(cell.get("id")),
        js_string_or_undefined(capped_at),
        match tests {
            None | Some(Value::Null) => "not run".to_string(), // ?? 'not run'
            Some(v) => jsjson::js_to_string(v),
        }
    )
}

/// `cells cap` stays on the narrow door (`dispatch`, `resolve_store_root`):
/// it refuses from a granted worktree exactly as it did before this cell.
/// `cells finish` is the one mutating verb ported onto the FULL door —
/// `run_finish` below — per wf-1's logged decision.
pub(crate) fn run_cap(finish: bool, flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &CAP_FLAGS) {
        return None;
    }
    let cap_flags = cap_flags_from(&flags)?;
    let deviations_file = opt_string_flag(&flags, "deviations-file")?;
    let cap_flags_owned = cap_flags;

    if finish {
        return run_finish("cells finish", use_json, t0, cap_flags_owned, deviations_file);
    }

    dispatch("cells cap", use_json, t0, move |ctx| {
        let mut cap_flags = cap_flags_owned;
        let root = ctx.root.clone();
        if let Some(file) = &deviations_file {
            if !file.is_empty() {
                // `flags['deviations-file'] ? parse : []` — truthy only.
                cap_flags.deviations = parse_deviations_file(file)?;
            }
        }
        let cell = cap_cell_from_flags(&root, &cap_flags, false)?;
        let text = cap_text(&cell);
        Ok(Out::Emit(cell, text, 0))
    })
}

/// `cells finish` (wf-1) — the FULL worktree door
/// (`crate::roots::resolve_store_root_worktree`), not the narrow one every
/// other mutating cells verb still uses: a granted worktree used to be
/// refused here by `emit_unsupported_root` (`Unsupported::GrantedWorktree`),
/// which inverted bee-swarming's cap-before-merge contract (a dispatched
/// worker could never cap the cell it just did; the orchestrator capped
/// after merge instead). Root split, per the logged decision:
///   * the cell record and its claim resolve at `StoreRoots::main_root()` —
///     one ledger, and the claim `finish` validates already lives there;
///   * reservation/hold release threads `StoreRoots::hold_topology()`
///     (main_root, holder) instead of the ordinary-only assumption
///     `release_reservations_for_agent` used to hardcode (holder `"main"`,
///     ledger at whatever `root` happened to be) — the un-ported piece
///     roots.rs:91-100 names as the reason cells stayed narrow.
/// From the MAIN checkout `roots.linked` is `None`, so `finish_topology`
/// answers `(root, Some((root, "main")))` — byte-identical to what the
/// narrow door produced before this cell. decision 13ce1858
/// (test-cadence-boundary D1): `finish_topology` used to also answer the
/// calling worktree's own root, the declared test command's cwd — dropped
/// with the per-cap test run itself; the cap no longer spawns that command
/// from either root.
fn run_finish(
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
    cap_flags_owned: CapFlags,
    deviations_file: Option<String>,
) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, use_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, use_json, t0)),
    };
    let (cells_root, topo) = finish_topology(&roots);
    let drift = check_manifest_drift(&cells_root);
    let ctx = rsv::Ctx {
        root: cells_root,
        cmd,
        use_json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    };
    let topo_ref = topo.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
    let out = finish_cap_and_release(&ctx.root, topo_ref, cap_flags_owned, deviations_file.as_deref());
    rsv::finish(&ctx, to_r2(out))
}

/// `cells finish`'s cap + reservation-release core, split out of
/// `run_finish` so it is directly testable against explicit roots and
/// topology — no process cwd mutation, unsafe under parallel `cargo test`
/// and avoided everywhere else in this crate (reservations/tests.rs's
/// `reserve_exec`/`release_exec` are the same shape, tested the same way).
pub(crate) fn finish_cap_and_release(
    root: &Path,
    topo: Option<(&Path, &str)>,
    cap_flags_owned: CapFlags,
    deviations_file: Option<&str>,
) -> MR<Out> {
    let mut cap_flags = cap_flags_owned;
    if let Some(file) = deviations_file {
        if !file.is_empty() {
            // `flags['deviations-file'] ? parse : []` — truthy only.
            cap_flags.deviations = parse_deviations_file(file)?;
        }
    }
    let cell = cap_cell_from_flags(root, &cap_flags, true)?;

    // cells.finish: release every reservation the claiming agent holds.
    let agent = match cell.get("trace").and_then(|t| t.get("worker")) {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    };
    let cell_id = js_string_or_undefined(cell.get("id"));
    let mut released: Vec<String> = Vec::new();
    let mut release_failure: Option<(String, String)> = None;
    if let Some(agent) = &agent {
        match release_reservations_for_agent(topo, root, agent, &cell_id) {
            Ok(outcome) => released = outcome.paths,
            Err(Fail::Thrown(message)) => {
                release_failure = Some((
                    message,
                    format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                ));
            }
            Err(Fail::Delegate) => {
                // Pre-scanned; a mid-command race lands here (header
                // residual): report it as a release failure, never a
                // rollback of the already-committed cap.
                release_failure = Some((
                    "reservation store shape changed mid-command (unrepresentable natively)".to_string(),
                    format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                ));
            }
        }
    }
    let Value::Object(cell_map) = &cell else { return Err(Fail::Delegate) };
    let mut result = cell_map.clone();
    result.insert(
        "released".into(),
        Value::Array(released.iter().map(|p| Value::String(p.clone())).collect()),
    );
    if let Some((error, fix)) = &release_failure {
        let mut rf = Map::new();
        rf.insert("error".into(), Value::String(error.clone()));
        rf.insert("fix".into(), Value::String(fix.clone()));
        result.insert("release_failed".into(), Value::Object(rf));
    }
    let mut lines = vec![cap_text(&cell)];
    lines.push(match (&release_failure, released.len()) {
        (Some((error, fix)), _) => {
            format!("Cap stands, but releasing reservations FAILED ({error}) — run: {fix}")
        }
        (None, 0) => "No active reservations to release.".to_string(),
        (None, n) => format!("Released {n} reservation(s): {}.", released.join(", ")),
    });
    lines.push("next: reply [DONE] with the one-line outcome, files touched, and the commit hash.".to_string());
    Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
}

// ── block / drop / unclaim / reopen / tier ─────────────────────────────────

/// Shared frame: pre-scan, per-cell lock, read (Delegate on non-object),
/// mutate, write, optional claim clear.
pub(crate) fn mutate_cell(
    root: &Path,
    id: &str,
    verb_not_found: &str,
    archived_verb: Option<&str>,
    clear_claim_after: bool,
    mutate: impl FnOnce(&mut Map<String, Value>) -> MR<()>,
) -> MR<Value> {
    prescan_claim(root, id)?;
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        if let Some(verb) = archived_verb {
            assert_not_archived(root, verb, id)?;
        }
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("{verb_not_found}: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        mutate(&mut cell_map)?;
        let value = Value::Object(cell_map);
        write_cell(root, &value)?;
        Ok(value)
    })();
    guard.release();
    let saved = saved?;
    if clear_claim_after {
        release_claim_file_best_effort(root, id);
    }
    Ok(saved)
}

pub(crate) fn ownership_args(flags: &rsv::Flags) -> Option<(Option<String>, bool)> {
    Some((opt_string_flag(flags, "session-id")?, bool_flag(flags, "force-ownership")?))
}

pub(crate) fn run_block(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells block", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("blockCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "blockCell", Some("blockCell"), true, move |cell_map| {
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "blockCell",
                session_flag.as_deref(),
                force,
            )?;
            let mut trace = append_attempt(
                &root2,
                &id2,
                trace,
                "blocked",
                Some(normalize_failure_signature(&reason2)),
                Some(&reason2),
            )?;
            cell_map.insert("status".into(), Value::String("blocked".into()));
            trace.insert("blocked_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Blocked {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn run_drop(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    dispatch("cells drop", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("dropCell: a reason is required.".into()));
        }
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "dropCell", Some("dropCell"), true, move |cell_map| {
            let mut trace = merge_trace(cell_map.get("trace"))?;
            cell_map.insert("status".into(), Value::String("dropped".into()));
            trace.insert("dropped_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Dropped {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn run_unclaim(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells unclaim", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = unclaim_cell(&root, &id, session_flag.as_deref(), force)?;
        let text = format!("Unclaimed {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

/// cells.mjs `unclaimCell(root, id, {sessionId, forceOwnership})`.
///
/// pub(crate) since the `dispatch prepare --claim` port: bee.mjs's
/// claimAndReserveForDispatch calls this as the SECOND rung of its unwind
/// ladder when a reservation conflicts, so the conflict refusal can promise
/// "the claim was unwound and state restored as found".
pub(crate) fn unclaim_cell(
    root: &Path,
    id: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Value> {
    let root = root.to_path_buf();
    let id = id.to_string();
    {
        let root2 = root.clone();
        let id2 = id.clone();
        let session_flag = session_flag.map(str::to_string);
        // unclaimCell has NO assertNotArchived (an archived cell reads as
        // capped/dropped and takes the not-claimed refusal instead).
        let cell = mutate_cell(&root, &id, "unclaimCell", None, true, move |cell_map| {
            let claimed = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "claimed");
            if !claimed {
                return Err(Fail::Thrown(format!(
                    "unclaimCell: cell \"{id2}\" is \"{}\", not \"claimed\" — only a claimed cell can be unclaimed (returned to open). For a capped/blocked/dropped cell use bee cells reopen.",
                    js_string_or_undefined(cell_map.get("status"))
                )));
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "unclaimCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            cell_map.insert("trace".into(), Value::Object(release_trace(trace)));
            Ok(())
        })?;
        // merge-ready-fact D2: the feature just grew an open cell again, so
        // it is no longer finished. Fail-open, and after the status flip —
        // the next last-cap re-sets the fact with a fresh `since`.
        clear_merge_ready_for(&root, &cell);
        Ok(cell)
    }
}

/// merge-ready-fact D2's one shared line for the reopen paths: drop the
/// feature's stored merge-ready fact once a cell of it is open again. Never
/// returns anything — a reopen's own result and exit code are untouched.
pub(crate) fn clear_merge_ready_for(root: &Path, cell: &Value) {
    if let Some(Value::String(feature)) = cell.get("feature") {
        crate::verbs::workflow_store::merge_ready::clear(root, feature);
    }
}

pub(crate) fn run_reopen(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells reopen", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("reopenCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "reopenCell", Some("reopenCell"), true, move |cell_map| {
            match cell_map.get("status") {
                Some(Value::String(s)) if s == "open" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is already \"open\"."
                    )))
                }
                Some(Value::String(s)) if s == "claimed" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is \"claimed\" — use bee cells unclaim to release the claim back to open."
                    )))
                }
                _ => {}
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "reopenCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            let mut trace = release_trace(trace);
            trace.insert("blocked_reason".into(), Value::Null);
            trace.insert("dropped_reason".into(), Value::Null);
            trace.insert("reopened_at".into(), Value::String(utc_now()));
            trace.insert("reopened_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        // merge-ready-fact D2 — same reason as `unclaim_cell` above.
        clear_merge_ready_for(&root, &cell);
        let text = format!("Reopened {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── tier's ceiling-share budget (D3, decision 0012) ────────────────────────
//
// "Keep ceiling scarce" (decision 0012) already backs `bee status`'s
// ceiling-scarcity line — status_full/cells.rs's `tier_mix` computes the
// SAME `ceiling / tiered` share this refusal checks. `tier_mix` takes a
// `status_full::Ctx` whose fields are private to that module (and that
// module is out of this cell's file reservation), so the formula is
// re-derived here rather than imported. This is a LIFT, not a second
// implementation — one membership test (MODEL_TIERS' own partition:
// extraction/generation/else-is-ceiling), one formula (ceiling/tiered) —
// following the precedent hooks/session_preamble/store.rs's own
// `ceiling_scarcity_warning` set for the identical "may not edit that file"
// boundary.

/// D3 (counter-teeth) / decision 0012 — no more than 40% of a feature's
/// tiered cells may sit on "ceiling" (the scarce, session-cost model)
/// without a named `--reason` override. Exactly 40% is allowed; strictly
/// over refuses.
pub(crate) const CEILING_SHARE_REFUSAL_MAX: f64 = 0.4;

struct CeilingShare {
    ceiling: i64,
    tiered: i64,
    share: f64,
}

/// The ceiling share of `feature`'s tiered cells (the whole store when
/// `feature` is absent — matching `tier_mix`'s own null-feature behavior)
/// AFTER a hypothetical `new_tier` assignment to `exclude_id`. `exclude_id`
/// is dropped from the scan first so the assigning cell is counted exactly
/// once, under its NEW tier rather than its old one.
fn ceiling_share_after(
    root: &Path,
    feature: Option<&str>,
    exclude_id: &str,
    new_tier: &str,
) -> MR<CeilingShare> {
    let cells = list_cells(root, feature, None).map_err(|_| Fail::Delegate)?;
    let (mut extraction, mut generation, mut ceiling) = (0i64, 0i64, 0i64);
    for cell in &cells {
        if matches!(cell.get("id"), Some(Value::String(cid)) if cid == exclude_id) {
            continue;
        }
        match cell.get("tier").and_then(|t| t.as_str()) {
            Some(t) if MODEL_TIERS.contains(&t) => match t {
                "extraction" => extraction += 1,
                "generation" => generation += 1,
                _ => ceiling += 1,
            },
            _ => {}
        }
    }
    match new_tier {
        "extraction" => extraction += 1,
        "generation" => generation += 1,
        _ => ceiling += 1,
    }
    let tiered = extraction + generation + ceiling;
    let share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    Ok(CeilingShare { ceiling, tiered, share })
}

/// setTier's mutator (D3): assigning tier "ceiling" when the post-assignment
/// ceiling share would exceed `CEILING_SHARE_REFUSAL_MAX` refuses, naming
/// the share and the threshold, unless `reason` is a non-blank override — in
/// which case the reason is recorded on the cell's trace (`tier_reason`).
/// Any other tier is never budget-checked. `--reason` given for an
/// already-under-budget "ceiling" assignment is still persisted (harmless
/// metadata; never required there).
pub(crate) fn set_tier(root: &Path, id: &str, tier: &str, reason: Option<&str>) -> MR<Value> {
    let id2 = id.to_string();
    let tier2 = tier.to_string();
    let reason2 = reason.map(str::to_string);
    mutate_cell(root, id, "setTier", Some("setTier"), false, move |cell_map| {
        if tier2 == "ceiling" {
            let feature = match cell_map.get("feature") {
                Some(Value::String(f)) if !f.is_empty() => Some(f.as_str()),
                _ => None,
            };
            let share = ceiling_share_after(root, feature, &id2, &tier2)?;
            let override_reason = reason2.as_deref().map(js_trim).filter(|r| !r.is_empty());
            if share.share > CEILING_SHARE_REFUSAL_MAX && override_reason.is_none() {
                return Err(Fail::Thrown(format!(
                    "setTier: cell \"{id2}\" refused — assigning tier \"ceiling\" would put {}/{} tiered cells on ceiling ({}%), over the {}% ceiling budget (D3, decision 0012). Pass --reason <text> to override; the reason is recorded on the cell's tier record.",
                    share.ceiling,
                    share.tiered,
                    jsjson::js_f64_to_string(rsv::js_round(share.share * 100.0)),
                    jsjson::js_f64_to_string(rsv::js_round(CEILING_SHARE_REFUSAL_MAX * 100.0)),
                )));
            }
            if let Some(r) = override_reason {
                let mut trace = merge_trace(cell_map.get("trace"))?;
                trace.insert("tier_reason".into(), Value::String(r.to_string()));
                cell_map.insert("trace".into(), Value::Object(trace));
            }
        }
        cell_map.insert("tier".into(), Value::String(tier2.clone()));
        Ok(())
    })
}

pub(crate) fn run_tier(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "tier", "reason"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let tier = flags.req_str("tier")?.to_string();
    if !MODEL_TIERS.contains(&tier.as_str()) {
        return None; // validate()'s required-field enum refusal — Node's bytes
    }
    // --reason <text> (D3): overrides the ceiling-share budget refusal
    // below. A flag-alone `--reason` (no value) is unprovable here — same
    // as every other optional value-flag this verb group parses through
    // `opt_string_flag` — so it delegates to Node's validate().
    let reason = opt_string_flag(&flags, "reason")?;
    dispatch("cells tier", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = set_tier(&root, &id, &tier, reason.as_deref())?;
        let text = format!(
            "Cell {} tier set to {}.",
            js_string_or_undefined(cell.get("id")),
            js_string_or_undefined(cell.get("tier"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── the mailbox hook at a cap (hm-2) ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const PROOF: &str = "cargo test -p bee — green — the cap hook only";

    fn mailbox_cell(root: &Path, id: &str) {
        let dir = root.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        let body = json!({
            "id": id, "feature": "human-mailbox", "title": "the store that holds a letter",
            "action": "a", "verify": "cargo test", "lane": "tiny", "status": "claimed",
            "deps": [], "files": [], "trace": {},
        });
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(&body)).unwrap();
    }

    fn mailbox_cap_flags(id: &str, outcome: &str) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: Some(outcome.to_string()),
            friction: None,
            files_changed: vec![json!("packages/bee-rs/crates/bee/src/verbs/mailbox.rs")],
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            // Pinned so the run this test reads back can never depend on a
            // BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID the test process inherited.
            session_flag: Some("mb-run".to_string()),
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(
                json!({
                    "outcome": "o",
                    "commit": "abc1234",
                    "files": [],
                    "tests": PROOF,
                    "deviations": [
                        "a free-form line with no three parts",
                        {
                            "what": "Recorded the sentence at the stop",
                            "why": "The composing pass may not author",
                            "kind": "found a better route",
                        },
                    ],
                })
                .to_string(),
            ),
            sync_ack: None,
        }
    }

    #[test]
    fn a_cap_appends_its_mailbox_entry_the_moment_it_lands() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), "{}").unwrap();

        // D9: this checkout is not armed — no herding block, no owner
        // switch — and it still records everything.
        assert!(!crate::verbs::mailbox::armed(root));

        mailbox_cell(root, "mb-1");
        let capped =
            cap_cell_from_flags(root, &mailbox_cap_flags("mb-1", "Taught bee to write a note"), false)
                .unwrap();
        assert_eq!(capped["status"], json!("capped"));

        let entries = crate::verbs::mailbox::read_entries(root, "mb-run");
        assert_eq!(entries.len(), 1, "the cap appended exactly one entry");
        let entry = &entries[0];
        assert_eq!(entry.kind, "cap");
        // D8: the sentence, written at the moment, is the human line the cap
        // already carried.
        assert_eq!(entry.what, "Taught bee to write a note");
        assert_eq!(entry.files, vec!["packages/bee-rs/crates/bee/src/verbs/mailbox.rs".to_string()]);
        assert_eq!(entry.commit.as_deref(), Some("abc1234"));
        assert_eq!(entry.proof.as_deref(), Some(PROOF));
        let departure = entry.departure.as_ref().expect("the three-part report deviation is read");
        assert_eq!(departure.kind, "found a better route");
        assert!(!entry.at.is_empty(), "the moment is recorded");

        // A second stop in the same run appends beside the first, in order —
        // one file per run, never one letter per stop (D11).
        mailbox_cell(root, "mb-2");
        cap_cell_from_flags(root, &mailbox_cap_flags("mb-2", "Wired the second stop"), false)
            .unwrap();
        let entries = crate::verbs::mailbox::read_entries(root, "mb-run");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].what, "Wired the second stop");
    }

    #[test]
    fn a_cap_whose_mailbox_cannot_be_written_still_caps() {
        // D10: a cap in a run that files no letter behaves exactly as it did
        // before this feature. `entries` occupied by a plain file makes every
        // append fail; the cap must land anyway.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(crate::verbs::mailbox::mailbox_dir(root)).unwrap();
        std::fs::write(crate::verbs::mailbox::entries_dir(root), "not a directory").unwrap();

        mailbox_cell(root, "mb-3");
        let capped =
            cap_cell_from_flags(root, &mailbox_cap_flags("mb-3", "Landed anyway"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert!(crate::verbs::mailbox::read_entries(root, "mb-run").is_empty());
    }

    #[test]
    fn a_cap_with_no_outcome_falls_back_to_the_title_in_the_mailbox() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        mailbox_cell(root, "mb-4");
        let mut flags = mailbox_cap_flags("mb-4", "");
        flags.outcome = None;
        cap_cell_from_flags(root, &flags, false).unwrap();
        let entries = crate::verbs::mailbox::read_entries(root, "mb-run");
        assert_eq!(entries[0].what, "Finished the store that holds a letter.");
    }
}
