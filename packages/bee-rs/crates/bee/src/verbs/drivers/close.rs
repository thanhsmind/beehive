// close, the report-only scribing/capture doors, and routing
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, write_text_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::{capture_queue_threshold, read_config_raw};
use crate::textutil::truncate_chars_tail;
use crate::verbs::reservations::{
    finish, js_is_ws, now_iso, now_ms, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use crate::verbs::knowledge::{bee_of, collect_concepts, str_array, str_field, touches_subject};
use serde_json::{json, Map, Number, Value};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ close ═════════════════════════════════════════════════════════════════

/// provenance: test-runner.mjs TEST_RESULTS_RELATIVE (verbs/test_runner.rs:60).
pub(crate) const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";

/// provenance: test-runner.mjs FAILURE_EXCERPT_MAX_CHARS (verbs/test_runner.rs:63).
pub(crate) const FAILURE_EXCERPT_MAX: usize = 500;

/// provenance: bee.mjs CLOSE_TESTS_UNDECLARED_DETAIL.
pub(crate) const CLOSE_TESTS_UNDECLARED_DETAIL: &str = "no commands.test declared — close has no test door here; declare commands.test in .bee/config.json (string or array) to give it one";

/// Pinned prefix of the D1 capture-debt refusal headline (message-contract
/// test: `close_refuses_uncaptured_behavior_change_cells`). Cite: CONTEXT.md
/// D1 (c2a7bd4f item 1).
pub(crate) const CLOSE_CAPTURE_DEBT_PREFIX: &str = "Capture debt for";

/// provenance: test-runner.mjs declaredTestCommands + state.mjs
/// normalizeCommands (verbs/test_runner.rs:184 declared_test_commands).
/// `None` == JS `null` (undeclared).
pub(crate) fn declared_test_commands(root: &Path) -> D<Option<Vec<String>>> {
    let config = read_config_raw(root);
    if let Some(Value::Array(items)) = config.get("dogfood_repos") {
        if !items.is_empty() {
            return Err(Delegate); // normalizeDogfoodRepos may warn to stderr
        }
    }
    let raw_test = config
        .get("commands")
        .and_then(Value::as_object)
        .and_then(|c| c.get("test"));
    let normalized: Vec<String> = match raw_test {
        Some(Value::String(s)) => {
            let t = js_trim(s);
            if t.is_empty() { Vec::new() } else { vec![t.to_string()] }
        }
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(js_trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    let cleaned: Vec<String> = normalized.into_iter().filter(|c| c != "none").collect();
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

pub(crate) struct CommandResult {
    pub(crate) command: String,
    pub(crate) exit: Option<i64>,
    pub(crate) duration_ms: u64,
    pub(crate) failure_excerpt: Option<String>,
}

pub(crate) struct TestRun {
    pub(crate) ran_at: String,
    pub(crate) green: bool,
    pub(crate) undeclared: bool,
    pub(crate) commands: Vec<CommandResult>,
    pub(crate) write_error: Option<String>,
}

/// provenance: test-runner.mjs spawnDeclaredCommand + posixShell. Resolution
/// lives in crate::shell — `bee close` runs the SAME proof command
/// `bee test` does, so it must resolve the same shell (and never WSL).
pub(crate) fn shell_command(shell: &str) -> Command {
    crate::shell::command().unwrap_or_else(|| Command::new(shell))
}

/// provenance: test-runner.mjs posixShell — shared resolver, probed once.
pub(crate) fn posix_shell() -> Option<&'static str> {
    crate::shell::posix_shell()
}

/// provenance: test-runner.mjs runDeclaredTests (verbs/test_runner.rs:263).
pub(crate) fn run_declared_tests(root: &Path, commands: &[String], shell: &str) -> TestRun {
    let ran_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut results: Vec<CommandResult> = Vec::new();
    let mut green = true;
    for command in commands {
        let started = Instant::now();
        let spawned = shell_command(shell)
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::null())
            .output();
        let duration_ms = started.elapsed().as_millis() as u64;
        let (mut output, exit, spawn_err) = match &spawned {
            Ok(out) => (
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                out.status.code().map(i64::from),
                None,
            ),
            Err(e) => (String::new(), None, Some(e.to_string())),
        };
        if let Some(msg) = spawn_err {
            output.push_str(&format!("\n[bee test] spawn error: {msg}"));
        }
        let passed = spawned.is_ok() && exit == Some(0);
        if !passed {
            green = false;
        }
        let failure_excerpt = if passed {
            None
        } else {
            let trimmed = js_trim(&output).to_string();
            let tail = truncate_chars_tail(&trimmed, FAILURE_EXCERPT_MAX);
            Some(if tail.is_empty() {
                format!(
                    "(no output; exit {})",
                    exit.map(|e| e.to_string()).unwrap_or_else(|| "null".to_string())
                )
            } else {
                tail
            })
        };
        results.push(CommandResult { command: command.clone(), exit, duration_ms, failure_excerpt });
    }
    let mut record = Map::new();
    record.insert("ran_at".into(), Value::String(ran_at.clone()));
    record.insert("green".into(), Value::Bool(green));
    record.insert(
        "commands".into(),
        Value::Array(results.iter().map(command_result_value).collect()),
    );
    let write_error = write_json_atomic(
        &root.join(".bee").join("logs").join("test-results.json"),
        &Value::Object(record),
    )
    .err()
    .map(|e| e.to_string());
    TestRun { ran_at, green, undeclared: false, commands: results, write_error }
}

/// The `tests` result field shared by every close outcome that has already
/// run the suite (green, and the D1 capture-debt refusal that follows it) —
/// one shape, so the two surfaces can never render the same run differently.
pub(crate) fn tests_result_value(run: &TestRun) -> Value {
    if run.undeclared {
        return Value::Null;
    }
    let mut tests = Map::new();
    tests.insert("ran_at".into(), Value::String(run.ran_at.clone()));
    tests.insert("green".into(), Value::Bool(true));
    tests.insert(
        "commands".into(),
        Value::Array(run.commands.iter().map(command_result_value).collect()),
    );
    tests.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
    Value::Object(tests)
}

/// {command, exit, duration_ms, failure_excerpt} — frozen key order.
pub(crate) fn command_result_value(c: &CommandResult) -> Value {
    let mut m = Map::new();
    m.insert("command".into(), Value::String(c.command.clone()));
    m.insert(
        "exit".into(),
        match c.exit {
            Some(code) => Value::Number(Number::from(code)),
            None => Value::Null,
        },
    );
    m.insert("duration_ms".into(), Value::Number(Number::from(c.duration_ms)));
    m.insert(
        "failure_excerpt".into(),
        match &c.failure_excerpt {
            Some(s) => Value::String(s.clone()),
            None => Value::Null,
        },
    );
    Value::Object(m)
}

/// provenance: bee.mjs renderTestCommandLines (~7601) — shared by `bee test`
/// and close, so the two surfaces can never render the same run differently.
pub(crate) fn render_test_command_lines(run: &TestRun) -> Vec<String> {
    run.commands
        .iter()
        .map(|c| {
            let secs = format!("{:.1}s", c.duration_ms as f64 / 1000.0);
            match &c.failure_excerpt {
                None => format!("✓ {} ({})", c.command, secs),
                Some(_) => format!(
                    "✗ {} ({}, exit {})",
                    c.command,
                    secs,
                    c.exit.map(|e| e.to_string()).unwrap_or_else(|| "spawn-failed".to_string())
                ),
            }
        })
        .collect()
}

/// provenance: test-runner.mjs firstFailureLine (verbs/test_runner.rs:381).
pub(crate) fn first_failure_line(run: &TestRun) -> Option<String> {
    let failing = run
        .commands
        .iter()
        .find(|c| c.failure_excerpt.as_deref().is_some_and(|s| !s.is_empty()))?;
    failing
        .failure_excerpt
        .as_deref()?
        .split('\n')
        .map(js_trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

// ── scribing debt + capture queue (the report-only doors) ──────────────────

/// provenance: cells.mjs scribingRunStampMs (verbs/status_full.rs:1700).
pub(crate) fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let at = vget(run, "at").filter(|v| truthy(v));
    let chosen = at.or_else(|| vget(run, "date"));
    let parsed = date_parse(chosen);
    if parsed.is_finite() { Some(parsed) } else { None }
}

/// provenance: reservations.rs js_date_parse, wrapped: an exotic date shape
/// (which V8 may parse and this port may not) yields NaN here, which is the
/// same control-flow branch Node takes for an unparseable date.
pub(crate) fn date_parse(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => match crate::verbs::reservations::js_date_parse(s) {
            Ok(Some(ms)) => ms,
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

/// provenance: cells.mjs bestScribingStampMs (also ported at
/// verbs/status_full/cells.rs:303 for `status`), scoped to `feature`: the
/// jsonl ledger's max, then the feature's own lane record
/// (`.bee/lanes/<feature>.json`, via `read_lane_display` — the same
/// fail-open display read `status`'s port already uses), then
/// `state.json`'s `last_scribing_run` — the freshest of the three wins, in
/// that order, matching the status-verb port exactly.
pub(crate) fn best_scribing_stamp_ms(root: &Path, feature: &str, state: &Map<String, Value>) -> D<Option<f64>> {
    let feature_value = Value::String(feature.to_string());
    let mut best: Option<f64> = None;
    for entry in read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl")) {
        if !truthy(&entry) || !strict_eq(vget(&entry, "feature"), Some(&feature_value)) {
            continue;
        }
        let parsed = date_parse(vget(&entry, "ts"));
        if parsed.is_finite() && best.map(|b| parsed > b).unwrap_or(true) {
            best = Some(parsed);
        }
    }
    // read_lane_display is the fail-open DISPLAY read: absent reads as None,
    // and a corrupt/mismatched record warns (naming the path) and reads as
    // None too — it never throws, so a bad lane record never stops close.
    let lane = crate::verbs::workflow_store::read_lane_display(root, feature).map_err(|_| Delegate)?;
    if let Some(lane) = lane {
        if let Some(stamp) = scribing_run_stamp_ms(lane.get("last_scribing_run")) {
            if best.map(|b| stamp > b).unwrap_or(true) {
                best = Some(stamp);
            }
        }
    }
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(&feature_value)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    Ok(best)
}

/// provenance: fsutil.mjs readJsonl (verbs/status_full.rs:526) — unparseable
/// lines are silently skipped.
pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

/// provenance: state.mjs readState — the ONE field scribingDebt reads through
/// it here is `last_scribing_run` (the feature comes from --feature, never
/// from the record), and defaultState() carries no such key, so the raw file
/// object IS the merged value for it.
pub(crate) fn read_state(root: &Path) -> D<Map<String, Value>> {
    match rj(&root.join(".bee").join("state.json"))? {
        Some(Value::Object(m)) => Ok(m),
        _ => Ok(Map::new()),
    }
}

pub(crate) struct DebtSummary {
    pub(crate) count: usize,
    pub(crate) ids: Vec<Value>,
}

/// provenance: cells.mjs scribingDebt(root, {feature}) — the feature-scoped
/// overrides arm (scribing-integrity si-1), which is the one close uses.
///
/// debt-door-archive dda-1: reads `list_cells_including_archive` (guard.rs),
/// not the plain active-only `list_cells`, so a behavior_change cell that
/// `bee close`'s own auto-archive already moved to
/// `.bee/cells/archive/<feature>/` still counts against the threshold below
/// — a clear door can no longer be a side effect of archiving the debt away.
pub(crate) fn scribing_debt(root: &Path, feature: &str) -> D<DebtSummary> {
    let state = read_state(root)?;
    let threshold = best_scribing_stamp_ms(root, feature, &state)?.unwrap_or(0.0);
    let mut ids = Vec::new();
    for cell in list_cells_including_archive(root, feature, "capped")? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse(vget(&trace, "capped_at"));
        if capped_at.is_finite() && capped_at > threshold {
            ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        }
    }
    Ok(DebtSummary { count: ids.len(), ids })
}

/// provenance: capture.mjs pendingCaptureStubs + captureQueue
/// (verbs/status_full.rs:2382) — only the COUNT used to reach close's door
/// text (localeCompare sort therefore never mattered); U3 (docs/history/
/// knowledge-usable/CONTEXT.md) also needs the oldest pending stub's age,
/// so both ride the one read below.
pub(crate) fn capture_queue_pending(root: &Path) -> (usize, f64) {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        let id = vget(event, "id");
        if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "flush")
            && id.map(truthy).unwrap_or(false)
        {
            flushed.push(id.unwrap().clone());
        } else if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "stub")
            && id.map(truthy).unwrap_or(false)
        {
            stubs.push(event);
        }
    }
    let pending: Vec<&&Value> = stubs
        .iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .collect();
    let oldest_ms = pending
        .iter()
        .map(|s| date_parse(vget(s, "at")))
        .filter(|ms| ms.is_finite())
        .fold(f64::NAN, |acc, ms| if acc.is_nan() || ms < acc { ms } else { acc });
    (pending.len(), oldest_ms)
}

/// U4 (docs/history/knowledge-usable/CONTEXT.md): the proposal's dominant
/// area — the `area_updates` entry with the most attributed bullets, ties
/// keeping the proposal's own order — names the stub's `area` field. `None`
/// when the proposal named no area at all (D19: a work item with no
/// `bee.areas` and no scribing stamp).
pub(crate) fn dominant_promote_area(proposal: &Value) -> Option<String> {
    proposal["area_updates"]
        .as_array()?
        .iter()
        .max_by_key(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
        .and_then(|u| u["area"].as_str())
        .map(str::to_string)
}

/// U4: once close writes `promote-proposals.md`, it ALSO appends one
/// capture-queue stub pointing at it — the queue is the living channel a
/// proposal reaches flush through (the 22 dead files under
/// docs/history/*/promote-proposals.md proved the standalone file is
/// write-only); the file itself keeps being written unchanged (audit trail,
/// D38). Same stub shape `capture add` writes (verbs/capture.rs run_add) so
/// `bee capture list`/flush treat it identically to a hand-added one.
/// Best-effort: an append failure here never fails close — the proposal file
/// itself is still the durable record.
pub(crate) fn enqueue_promote_stub(root: &Path, feature: &str, proposal: &Value, proposals_rel: &str) {
    let mut stub = Map::new();
    stub.insert("kind".into(), Value::String("stub".into()));
    stub.insert("id".into(), Value::String(pseudo_uuid_v4()));
    stub.insert("at".into(), Value::String(now_iso()));
    stub.insert(
        "outcome".into(),
        Value::String(format!("Promote proposal for \"{feature}\" — {proposals_rel}")),
    );
    stub.insert("dids".into(), Value::Array(Vec::new()));
    stub.insert(
        "area".into(),
        dominant_promote_area(proposal).map(Value::String).unwrap_or(Value::Null),
    );
    stub.insert("files".into(), Value::Array(vec![Value::String(proposals_rel.to_string())]));
    stub.insert("lane".into(), Value::Null);
    stub.insert("source".into(), Value::String("promote".into()));
    let _ = append_jsonl(&root.join(".bee").join("capture-queue.jsonl"), &Value::Object(stub));
}

/// D1 escape hatch: a logged decision tagged `capture-deferral` whose
/// decision/rationale/alternatives text names the feature lifts the
/// scribing-debt refusal. Reuses the decisions verb's own read model
/// (crate::verbs::decisions::active_decisions + filter_decision_events)
/// rather than hand-parsing decisions.jsonl a second way — same tag-exact,
/// whole-token feature match `decisions active --tag --feature` already
/// uses. Cite: CONTEXT.md D1 (precedent: decision c8e25271).
pub(crate) fn has_capture_deferral_decision(root: &Path, feature: &str) -> D<bool> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("capture-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(!filtered.is_empty())
}

pub(crate) struct Door {
    pub(crate) door: &'static str,
    pub(crate) blocking: bool,
    pub(crate) detail: String,
    pub(crate) command: Option<&'static str>,
}

impl Door {
    pub(crate) fn value(&self) -> Value {
        let mut m = Map::new();
        m.insert("door".into(), Value::String(self.door.into()));
        m.insert("blocking".into(), Value::Bool(self.blocking));
        m.insert("detail".into(), Value::String(self.detail.clone()));
        m.insert(
            "command".into(),
            match self.command {
                Some(c) => Value::String(c.into()),
                None => Value::Null,
            },
        );
        Value::Object(m)
    }
}

/// U3 (docs/history/knowledge-usable/CONTEXT.md): past the configured
/// `capture_queue_threshold` — the pending count exceeds it, OR the oldest
/// pending stub is older than the configured day count — the capture-queue
/// door's detail escalates to name the breach. The door stays report-only
/// (`blocking: false`, decision c8e25271's deferral, untouched by U3) either
/// way; only the wording changes.
pub(crate) fn capture_queue_door_detail(root: &Path, queue: usize, oldest_ms: f64) -> String {
    if queue == 0 {
        return "clear".to_string();
    }
    let config = read_config_raw(root);
    let threshold = capture_queue_threshold(&config);
    let oldest_age_days = if oldest_ms.is_nan() { None } else { Some((now_ms() - oldest_ms) / 86_400_000.0) };
    let over_count = queue as u64 > threshold.count;
    let over_age = oldest_age_days.map(|d| d > threshold.days).unwrap_or(false);
    if over_count || over_age {
        let oldest_days = oldest_age_days.unwrap_or(0.0).max(0.0).floor() as u64;
        return format!(
            "OVERDUE — {queue} stub(s) pending, oldest {oldest_days} days — flush before new work; settle via bee-capturing"
        );
    }
    format!("pending — {queue} capture stub(s) awaiting flush; settle later via bee-capturing")
}

/// provenance: bee.mjs buildCloseReportDoors, extended by D1 — the
/// capture-queue door stays report-only (decision c8e25271's blanket
/// deferral, untouched here), but the scribing-debt door now BLOCKS close
/// when the feature has behavior_change cells with no capture recorded and
/// no logged `capture-deferral` decision names the feature (CONTEXT.md D1).
pub(crate) fn build_close_report_doors(root: &Path, feature: &str) -> D<Vec<Door>> {
    let scribing = scribing_debt(root, feature)?;
    let deferred = if scribing.count > 0 {
        has_capture_deferral_decision(root, feature)?
    } else {
        false
    };
    let scribing_blocking = scribing.count > 0 && !deferred;
    let mut doors = Vec::new();
    doors.push(Door {
        door: "scribing-debt",
        blocking: scribing_blocking,
        detail: if scribing.count == 0 {
            "clear".to_string()
        } else if scribing_blocking {
            format!(
                "pending — {} behavior_change cell(s) uncaptured ({}); run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"{feature}\" to defer it",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        } else {
            format!(
                "deferred — {} behavior_change cell(s) uncaptured ({}); a logged capture-deferral decision names \"{feature}\"",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        },
        command: if scribing_blocking { Some("bee-capturing") } else { None },
    });
    let (queue, oldest_ms) = capture_queue_pending(root);
    doors.push(Door {
        door: "capture-queue",
        blocking: false,
        detail: capture_queue_door_detail(root, queue, oldest_ms),
        command: None,
    });
    Ok(doors)
}

// ── U7: close-time pattern-check door ───────────────────────────────────────
//
// docs/history/knowledge-usable/CONTEXT.md U7 (PBI p-21583c96): a report-only
// door that maps the feature's capped cells' touched files to the bundle
// areas they reach — the SAME per-file `touches_subject` match promote.rs's
// own area-update section already applies (decision b032be35: a concept's
// own bundle path plus its recorded `bee.sources`), just unscoped to any one
// work item — then lists the `bee.critical: true` patterns (ku-6's re-graded
// pool, docs/knowledge/areas/okf-profile/critical-bar.md) tagged with any of
// those areas. Smallest transport `bee close` already supports: one new
// value flag, `--pattern-verdicts=<pattern-id>:<verdict>[,<pattern-id>:
// <verdict>...]` (verdict one of violated/respected/not-applicable) — never
// a new answers-file format. A pattern with no matching verdict reports
// `pending` in the detail line and never blocks; a recorded `violated`
// blocks close exactly like a red test, naming the pattern.
pub(crate) const CLOSE_PATTERN_VIOLATED_PREFIX: &str = "Pattern violated for";

pub(crate) const PATTERN_VERDICT_WORDS: [&str; 3] = ["violated", "respected", "not-applicable"];

/// Parses `--pattern-verdicts`' whole value in one pass. A pair with no `:`,
/// an empty id, or a word outside `PATTERN_VERDICT_WORDS` is silently
/// dropped — that pattern reports `pending`, same as one never mentioned at
/// all; malformed input never fails close (the door is report-only until a
/// `violated` verdict is actually recorded).
pub(crate) fn parse_pattern_verdicts(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in raw.split(',') {
        let pair = js_trim(pair);
        let Some(idx) = pair.find(':') else { continue };
        let id = js_trim(&pair[..idx]);
        let verdict = js_trim(&pair[idx + 1..]).to_lowercase();
        if id.is_empty() || !PATTERN_VERDICT_WORDS.contains(&verdict.as_str()) {
            continue;
        }
        out.insert(id.to_string(), verdict);
    }
    out
}

/// The feature's touched files: `files_changed` off every capped cell (live
/// store + archive — the same read `scribing_debt` above uses), deduped,
/// insertion order.
pub(crate) fn feature_touched_files(root: &Path, feature: &str) -> D<Vec<String>> {
    let mut files: Vec<String> = Vec::new();
    for cell in list_cells_including_archive(root, feature, "capped")? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if let Some(Value::Array(items)) = vget(&trace, "files_changed") {
            for item in items {
                if let Value::String(s) = item {
                    if !files.contains(s) {
                        files.push(s.clone());
                    }
                }
            }
        }
    }
    Ok(files)
}

/// Bundle areas the touched files reach. A concept with no `bee.sources`
/// naming a code path (most area concepts, per this very corpus's own
/// pattern `20260805-a-derived-field-empty-for-a-whole-class-of-inputs...`)
/// simply never matches on files alone — the door degrades to `clear`
/// rather than to a wrong answer.
pub(crate) fn touched_bundle_areas(dir: &Path, touched_files: &[String]) -> Vec<String> {
    let Some(concepts) = collect_concepts(dir) else { return Vec::new() };
    let mut areas: Vec<String> = Vec::new();
    for concept in &concepts {
        let bee = bee_of(&concept.data);
        let concept_areas = str_array(&bee, "areas");
        if concept_areas.is_empty() {
            continue;
        }
        let mut subjects: Vec<String> = vec![format!("docs/knowledge/{}", concept.path)];
        subjects.extend(str_array(&bee, "sources").into_iter().filter(|s| !s.is_empty()));
        let touched = touched_files.iter().any(|f| subjects.iter().any(|s| touches_subject(f, s)));
        if !touched {
            continue;
        }
        for a in concept_areas {
            if !areas.contains(&a) {
                areas.push(a);
            }
        }
    }
    areas
}

pub(crate) struct CriticalPattern {
    pub(crate) id: String,
    pub(crate) title: String,
}

/// Every `bee.critical: true` concept tagged with at least one of `areas` —
/// the same predicate `bee knowledge index`'s "Critical patterns" section
/// applies (verbs/knowledge/index.rs), scoped down to the touched areas.
pub(crate) fn critical_patterns_for_areas(dir: &Path, areas: &[String]) -> Vec<CriticalPattern> {
    let Some(concepts) = collect_concepts(dir) else { return Vec::new() };
    let mut out: Vec<CriticalPattern> = concepts
        .iter()
        .filter(|c| {
            let bee = bee_of(&c.data);
            matches!(bee.get("critical"), Some(Value::Bool(true)))
                && str_array(&bee, "areas").iter().any(|a| areas.contains(a))
        })
        .map(|c| {
            let bee = bee_of(&c.data);
            let id = bee.get("id").and_then(Value::as_str).unwrap_or(&c.path).to_string();
            let title = str_field(&c.data, "title").unwrap_or(&c.path).to_string();
            CriticalPattern { id, title }
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// U7's door itself: `blocking` is true the moment ANY matched pattern's
/// verdict is `violated` — the one condition that stops close "exactly like
/// a red test" (CONTEXT.md U7). `respected`/`not-applicable` pass silently;
/// an unanswered pattern reports `pending` in the detail line and never
/// blocks. No bundle, no touched files, no touched area, or no critical
/// pattern in a touched area all collapse to the same `clear` report — the
/// door has nothing to ask for.
pub(crate) fn build_pattern_check_door(root: &Path, feature: &str, verdicts: &HashMap<String, String>) -> D<Door> {
    if !crate::hooks::session_preamble::bundle_mode(root) {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    }
    let Some(dir) = crate::verbs::knowledge::bundle_dir(root) else {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    };
    let touched_files = feature_touched_files(root, feature)?;
    let touched_areas = touched_bundle_areas(&dir, &touched_files);
    let patterns = critical_patterns_for_areas(&dir, &touched_areas);
    if patterns.is_empty() {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    }
    let mut rows: Vec<String> = Vec::new();
    let mut violated: Vec<String> = Vec::new();
    let mut pending = false;
    for p in &patterns {
        let verdict = verdicts.get(&p.id).cloned().unwrap_or_else(|| "pending".to_string());
        if verdict == "violated" {
            violated.push(format!("{} ({})", p.id, p.title));
        }
        if verdict == "pending" {
            pending = true;
        }
        rows.push(format!("{}={verdict}", p.id));
    }
    let mut detail = format!(
        "{} critical pattern(s) in touched area(s) [{}]: {}",
        patterns.len(),
        touched_areas.join(", "),
        rows.join(", ")
    );
    if pending {
        detail.push_str(
            " — unanswered pattern(s) report pending; supply a verdict via \
             --pattern-verdicts=<pattern-id>:<violated|respected|not-applicable>[,<pattern-id>:<verdict>...]",
        );
    }
    Ok(Door { door: "pattern-check", blocking: !violated.is_empty(), detail, command: None })
}

/// JS Array.prototype.join (null/undefined render empty).
pub(crate) fn js_join(items: &[Value], sep: &str) -> String {
    items
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// provenance: bee.mjs renderCloseDoorLines.
pub(crate) fn render_close_door_lines(doors: &[Door]) -> Vec<String> {
    doors
        .iter()
        .map(|d| {
            if !d.blocking && d.detail == "clear" {
                return format!("door {}: clear", d.door);
            }
            format!(
                "door {}: {} — {}{}",
                d.door,
                if d.blocking { "BLOCKING" } else { "open" },
                d.detail,
                match d.command {
                    Some(c) => format!(" | settle: {c}"),
                    None => String::new(),
                }
            )
        })
        .collect()
}

/// provenance: bee.mjs handleClose (~7643). `worktree` is provably null here
/// (see the file header), so the merge-back line never renders natively.
pub(crate) fn close_handler(
    root: &Path,
    feature: &str,
    dry_run: bool,
    declared: Option<Vec<String>>,
    shell: Option<&'static str>,
    pattern_verdicts: &HashMap<String, String>,
) -> D<Out> {
    if dry_run {
        let mut doors = vec![Door {
            door: "tests",
            blocking: false,
            detail: match &declared {
                Some(cmds) => format!(
                    "commands.test declared ({} command(s)) — close runs the full declared suite fresh; a stale test-results record is never trusted",
                    cmds.len()
                ),
                None => CLOSE_TESTS_UNDECLARED_DETAIL.to_string(),
            },
            command: if declared.is_some() { Some("bee test") } else { None },
        }];
        doors.extend(build_close_report_doors(root, feature)?);
        doors.push(build_pattern_check_door(root, feature, pattern_verdicts)?);
        let next_line = match &declared {
            Some(_) => format!("next: bee close --feature {feature} — runs the declared tests and reports"),
            None => format!(
                "next: feature \"{feature}\" has no test door — close proceeds; capture stays pending for bee-capturing"
            ),
        };
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        let mut lines = render_close_door_lines(&doors);
        lines.push(next_line);
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0));
    }

    // The real run: the tests door is the full declared run, fresh.
    let run = match (&declared, shell) {
        (Some(commands), Some(shell)) => run_declared_tests(root, commands, shell),
        _ => TestRun {
            ran_at: String::new(),
            green: false,
            undeclared: true,
            commands: Vec::new(),
            write_error: None,
        },
    };
    if let Some(msg) = &run.write_error {
        // Node: writeJsonAtomic throws -> main's catch -> emitError.
        return Ok(Out::Thrown(msg.clone()));
    }
    let report_doors = build_close_report_doors(root, feature)?;
    let pattern_door = build_pattern_check_door(root, feature, pattern_verdicts)?;

    if !run.undeclared && !run.green {
        let failing: Vec<&CommandResult> =
            run.commands.iter().filter(|c| c.failure_excerpt.is_some()).collect();
        let first_line = first_failure_line(&run);
        let mut doors = vec![Door {
            door: "tests",
            blocking: true,
            detail: format!(
                "the declared test run is RED ({} of {} command(s) failed; record: {TEST_RESULTS_RELATIVE})",
                failing.len(),
                run.commands.len()
            ),
            command: Some("bee test"),
        }];
        doors.extend(report_doors);
        doors.push(pattern_door);
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(true));
        let mut tests = Map::new();
        tests.insert("ran_at".into(), Value::String(run.ran_at.clone()));
        tests.insert("green".into(), Value::Bool(false));
        tests.insert(
            "commands".into(),
            Value::Array(run.commands.iter().map(command_result_value).collect()),
        );
        tests.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
        result.insert("tests".into(), Value::Object(tests));

        let mut lines = vec![format!(
            "Tests RED for \"{feature}\" — close stops at the tests door (record: {TEST_RESULTS_RELATIVE}):"
        )];
        lines.extend(render_test_command_lines(&run));
        for c in &failing {
            lines.push(format!(
                "--- {} (exit {}) ---\n{}",
                c.command,
                c.exit.map(|e| e.to_string()).unwrap_or_else(|| "spawn-failed".to_string()),
                c.failure_excerpt.clone().unwrap_or_default()
            ));
        }
        lines.push(format!(
            "next: the red is the work — fix it ({}), then re-run bee close --feature {feature}",
            first_line.unwrap_or_else(|| "see the excerpt above".to_string())
        ));
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // Green (or no declared test path): what remains is the capture checklist.
    let tests_door = if run.undeclared {
        Door {
            door: "tests",
            blocking: false,
            detail: CLOSE_TESTS_UNDECLARED_DETAIL.to_string(),
            command: None,
        }
    } else {
        Door {
            door: "tests",
            blocking: false,
            detail: format!(
                "GREEN — {} command(s) passed (record: {TEST_RESULTS_RELATIVE})",
                run.commands.len()
            ),
            command: None,
        }
    };
    let scribing_detail = report_doors
        .iter()
        .find(|d| d.door == "scribing-debt")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let queue_detail = report_doors
        .iter()
        .find(|d| d.door == "capture-queue")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let mut doors = vec![tests_door];
    doors.extend(report_doors);
    doors.push(pattern_door);

    // ── D1: refuse on uncaptured behavior_change cells ──────────────────────
    //
    // Tests are GREEN (or undeclared) — the one remaining door that can still
    // stop close is scribing-debt, and only when it is BLOCKING: the feature
    // has behavior_change cells with no capture recorded and no logged
    // `capture-deferral` decision names it (build_close_report_doors is the
    // one place that decides `blocking` — this reads its verdict rather than
    // recomputing it, so the counter and the refusal can never disagree).
    if doors.iter().any(|d| d.door == "scribing-debt" && d.blocking) {
        let debt = scribing_debt(root, feature)?;
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(!run.undeclared));
        result.insert("tests".into(), tests_result_value(&run));
        let lines = vec![
            format!(
                "{CLOSE_CAPTURE_DEBT_PREFIX} \"{feature}\" — close stops at the scribing-debt door: {} behavior_change cell(s) uncaptured ({}).",
                debt.count,
                js_join(&debt.ids, ", ")
            ),
            format!("remedy: run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"{feature}\" to defer it."),
            format!("next: settle the capture debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── U7: refuse on a recorded `violated` pattern verdict ─────────────────
    //
    // Runs only here — tests GREEN (or undeclared) and past the D1 refusal
    // above — same "stops close exactly like a red test" placement the
    // scribing-debt door already established for its own blocking arm.
    if doors.iter().any(|d| d.door == "pattern-check" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(!run.undeclared));
        result.insert("tests".into(), tests_result_value(&run));
        let pattern_detail = doors
            .iter()
            .find(|d| d.door == "pattern-check")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!(
                "{CLOSE_PATTERN_VIOLATED_PREFIX} \"{feature}\" — close stops at the pattern-check door: {pattern_detail}"
            ),
            "remedy: fix the violated pattern's finding, or re-run with a corrected --pattern-verdicts if it is a false positive.".to_string(),
            format!("next: settle the violated pattern(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    let headline = if run.undeclared {
        format!(
            "No commands.test declared for \"{feature}\" — nothing gated close; declare commands.test in .bee/config.json to give close a test door."
        )
    } else {
        format!(
            "Tests GREEN for \"{feature}\" — {} command(s) passed (record: {TEST_RESULTS_RELATIVE}).",
            run.commands.len()
        )
    };
    let mut result = Map::new();
    result.insert("feature".into(), Value::String(feature.to_string()));
    result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
    result.insert("ran_tests".into(), Value::Bool(!run.undeclared));
    result.insert("tests".into(), tests_result_value(&run));

    // ── D2: soft promote door — computed BEFORE retirement ─────────────────
    //
    // Runs only here — past the tests door (GREEN or undeclared) and past
    // the scribing-debt refusal above, so a red close or one stopped on
    // capture debt never reaches it. It must run before the cells are
    // retired below: `build_promotion` scans `.bee/cells/*.json`, and once
    // retirement moves the feature's just-capped cells into
    // `.bee/cells/archive/` that scan would come back empty. `build_promotion`
    // is read-only, so computing it here has no effect beyond what it can
    // see. `build_promotion`'s `None` arm means "delegate to Node", and
    // there is no Node left to delegate to (`rg -l buildContextManifest
    // --glob '*.mjs'` finds nothing); a `Thrown` is promote's own typed
    // refusal (most commonly `unknown_work` for a feature with neither a
    // work-item concept nor a history anchor). Both degrade through the
    // SAME one-line warning pushed further below (line position unchanged)
    // and close proceeds unchanged either way — SOFT means proposing the
    // knowledge a feature earned never blocks finishing it, and D38
    // (promote proposes, it never writes into docs/knowledge/) stays
    // untouched by this door.
    let promote_outcome: Result<Value, String> = match crate::verbs::knowledge::bundle_dir(root) {
        None => Err("no docs/knowledge/ bundle to mine here".to_string()),
        Some(dir) => match crate::verbs::knowledge::build_promotion(root, &dir, feature) {
            None => Err("no docs/knowledge/ bundle to mine here".to_string()),
            Some(crate::verbs::knowledge::Promo::Thrown(msg)) => Err(msg),
            Some(crate::verbs::knowledge::Promo::Ok(proposal)) => Ok(proposal),
        },
    };

    // ── retire the feature's cells ────────────────────────────────────────
    //
    // Close is the lifecycle event that MEANS "this feature is done", and
    // `.bee/cells/` is on the hot read path — every `status` and `orient`
    // parses each file in it. Left to a human remembering `bee cells
    // archive`, cells accumulate for the life of the repo: bee's own store
    // reached 455 files across 118 features, 441 of them belonging to
    // features that were completely finished, and paid for all of them on
    // every orientation.
    //
    // Three conditions, all necessary: the close is GREEN and past the
    // scribing-debt door (a red close, or one refused on capture debt, never
    // reaches here), every one of the feature's cells is terminal (an open
    // cell is reported, not archived), and the repo has not opted out.
    // Reversible either way: `bee cells archive --feature <f>` has an
    // `unarchive` twin and the files stay in git.
    let retired = auto_archive_on_close(root, feature);

    let mut lines = vec![headline];
    if !run.undeclared {
        lines.extend(render_test_command_lines(&run));
    }
    lines.push(format!(
        "Capture (deferred, decision c8e25271): scribing {scribing_detail}; capture queue {queue_detail}."
    ));
    match &retired {
        // `moved == 0` is the feature that had no cells in the first place:
        // real, common (a docs-only close), and not worth a line.
        Retirement::Archived { moved } if *moved > 0 => lines.push(format!(
            "Retired \"{feature}\": {moved} cell(s) moved out of the active scan (bee cells unarchive --feature {feature} to reverse)."
        )),
        Retirement::Archived { .. } => {}
        Retirement::Held { reason } => lines.push(format!(
            "Cells kept in the active scan: {reason}."
        )),
        Retirement::Off => {}
    }
    result.insert("retired".into(), retired.value());

    // `promote_outcome` was computed above, before retirement moved the
    // feature's cells out of `.bee/cells/`, so `build_promotion` still saw
    // them. Rendering the warning line here keeps its position in the
    // output unchanged.
    let promote_line = match promote_outcome {
        Ok(proposal) => {
            let proposals_rel = format!("docs/history/{feature}/promote-proposals.md");
            let text = crate::verbs::knowledge::promote_text(&proposal);
            match write_text_atomic(&root.join(&proposals_rel), &text) {
                Ok(()) => {
                    enqueue_promote_stub(root, feature, &proposal, &proposals_rel);
                    let cells_mined = proposal["cells"].as_array().map(Vec::len).unwrap_or(0);
                    let area_bullets: usize = proposal["area_updates"]
                        .as_array()
                        .map(|updates| {
                            updates
                                .iter()
                                .map(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
                                .sum()
                        })
                        .unwrap_or(0);
                    let pattern_candidates =
                        proposal["pattern_candidates"].as_array().map(Vec::len).unwrap_or(0);
                    format!(
                        "Promote proposed for \"{feature}\": {cells_mined} capped cell(s) mined, {area_bullets} area bullet(s), {pattern_candidates} pattern candidate(s) — see {proposals_rel}."
                    )
                }
                Err(e) => format!(
                    "Promote proposed for \"{feature}\" but {proposals_rel} could not be written: {e}."
                ),
            }
        }
        Err(reason) => format!("Promote skipped for \"{feature}\": {reason}"),
    };
    lines.push(promote_line);

    lines.push(
        "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
            .to_string(),
    );
    Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
}

/// What close did with the feature's cells, and why.
pub(crate) enum Retirement {
    Archived { moved: usize },
    Held { reason: String },
    /// `cells_archive_on_close: false` — the repo asked close to leave the
    /// store alone. Silent, because a switch the owner set is not news.
    Off,
}

impl Retirement {
    fn value(&self) -> Value {
        match self {
            Retirement::Archived { moved } => json!({"archived": true, "moved": moved}),
            Retirement::Held { reason } => json!({"archived": false, "reason": reason}),
            Retirement::Off => json!({"archived": false, "reason": "cells_archive_on_close is off"}),
        }
    }
}

/// Default TRUE: the whole point is that it happens without anyone
/// remembering. `.bee/config.json` `cells_archive_on_close: false` opts out —
/// for a repo whose own tooling reads `.bee/cells/*.json` by path.
fn archive_on_close_enabled(root: &Path) -> bool {
    let config = read_config_raw(root);
    !matches!(config.get("cells_archive_on_close"), Some(Value::Bool(false)))
}

fn auto_archive_on_close(root: &Path, feature: &str) -> Retirement {
    if !archive_on_close_enabled(root) {
        return Retirement::Off;
    }
    // Best-effort throughout: close has already succeeded, and a store that
    // could not be tidied is not a failed close. Every arm says what it did.
    match crate::verbs::cells::archive_feature_for_close(root, feature) {
        Ok(moved) => Retirement::Archived { moved },
        Err(reason) => Retirement::Held { reason },
    }
}

pub(crate) fn run_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "dry-run", "pattern-verdicts"]) {
        return None;
    }
    // validate(): a boolean-typed flag given as =value must be true/false.
    match flags.get("dry-run") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): --feature required; requireFlag also rejects ''/true.
    let feature = flags.req_str("feature")?.to_string();
    // `flags['dry-run'] === true`: only the flag-alone form is JS `true`.
    let dry_run = matches!(flags.get("dry-run"), Some(FlagV::Present));
    // U7: `--pattern-verdicts=<id>:<verdict>[,...]` — absent or a bare flag
    // (no value) both read as "no verdicts supplied," same as an empty one.
    let pattern_verdicts: HashMap<String, String> =
        flags.truthy_str("pattern-verdicts").map(parse_pattern_verdicts).unwrap_or_default();

    // ── everything that can still delegate happens BEFORE prelude, whose
    //    drift-cache write would swallow the Node re-run's drift line. ──────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "close", use_json, t0, &why))
        }
        Roots::None => return Some(emit_no_root_error(&cwd, "close", use_json, t0)),
    };
    let declared = declared_test_commands(&root).ok()?;
    let shell = if !dry_run && declared.is_some() {
        let s = posix_shell()?; // no POSIX sh — Node's cmd.exe fallback owns it
        ensure_dir(&root.join(".bee").join("logs")).ok()?;
        Some(s)
    } else {
        None
    };
    // Delegation pre-flight for the report doors: they are pure reads, so
    // computing them here (and again, for real, after the suite runs) can
    // only cost two cheap directory scans — but it means a corrupt store can
    // still hand the whole command to Node BEFORE a test suite is spent.
    build_close_report_doors(&root, &feature).ok()?;
    build_pattern_check_door(&root, &feature, &pattern_verdicts).ok()?;

    let ctx = match prelude("close", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out: R2<Out> = close_handler(&ctx.root, &feature, dry_run, declared, shell, &pattern_verdicts)
        .map_err(crate::verbs::reservations::Err2::from);
    finish(&ctx, out)
}

// ═══ routing ═══════════════════════════════════════════════════════════════

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    match args.first()?.to_str()? {
        "close" => {
            let toks: Vec<&str> =
                args[1..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None; // Node renders command-scoped help
            }
            let (flags, use_json) = parse_flags(&toks)?;
            run_close(flags, use_json, t0)
        }
        "dispatch" => {
            if args.get(1)?.to_str()? != "prepare" {
                return None;
            }
            let toks: Vec<&str> =
                args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None;
            }
            let (flags, use_json) = parse_flags(&toks)?;
            run_dispatch_prepare(flags, use_json, t0)
        }
        _ => None,
    }
}

// ─── tests: U3 capture-queue pressure escalation (close door) ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    fn stub_line(id: &str, at: &str) -> String {
        format!(r#"{{"kind":"stub","id":"{id}","at":"{at}","outcome":"x"}}"#)
    }

    /// Below the default threshold (5 stubs, 7 days): the door's wording
    /// stays byte-identical to before U3, same as the nudge's contract.
    #[test]
    fn under_threshold_detail_is_byte_identical_to_before() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = crate::verbs::reservations::now_iso();
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &now)));
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(
            capture_queue_door_detail(root, queue, oldest_ms),
            "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing"
        );
    }

    #[test]
    fn zero_pending_reads_clear() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(capture_queue_door_detail(root, 0, f64::NAN), "clear");
    }

    #[test]
    fn over_count_threshold_escalates_the_door_to_overdue_wording() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = crate::verbs::reservations::now_iso();
        let lines: String =
            (0..6).map(|i| format!("{}\n", stub_line(&format!("s{i}"), &now))).collect();
        w(root, ".bee/capture-queue.jsonl", &lines);
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(queue, 6);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(
            detail.starts_with("OVERDUE — 6 stub(s) pending, oldest 0 days — flush before new work"),
            "{detail}"
        );
        assert!(detail.ends_with("settle via bee-capturing"));
    }

    #[test]
    fn over_age_threshold_escalates_even_under_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let old = crate::verbs::reservations::iso_from_ms(now_ms() - 10.0 * 86_400_000.0).ok().unwrap();
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &old)));
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(queue, 1);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(
            detail.starts_with("OVERDUE — 1 stub(s) pending, oldest 10 days — flush before new work"),
            "{detail}"
        );
    }

    #[test]
    fn configured_threshold_overrides_the_default_for_the_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"capture_queue_threshold":{"count":1,"days":30}}"#);
        let now = crate::verbs::reservations::now_iso();
        let lines = format!("{}\n{}\n", stub_line("s1", &now), stub_line("s2", &now));
        w(root, ".bee/capture-queue.jsonl", &lines);
        let (queue, oldest_ms) = capture_queue_pending(root);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(detail.starts_with("OVERDUE — 2 stub(s) pending"), "{detail}");
    }

    /// A malformed threshold falls back to the default (5, 7) — the door
    /// never blocks either way (`build_close_report_doors`' capture-queue
    /// row always carries `blocking: false`).
    #[test]
    fn malformed_threshold_falls_back_and_the_door_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"capture_queue_threshold":{"count":-1,"days":7}}"#);
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &crate::verbs::reservations::now_iso())));
        let doors = build_close_report_doors(root, "demo").unwrap();
        let capture_door = doors.iter().find(|d| d.door == "capture-queue").unwrap();
        assert!(!capture_door.blocking);
        assert_eq!(capture_door.detail, "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing");
    }

    // ─── tests: U7 close-time pattern-check door ───────────────────────────

    /// Writes a minimal bundle: one area concept whose `bee.sources` names
    /// `src/a.rs` (so the touched file matches it) tagged `areas: [demo]`,
    /// and one `bee.critical: true` pattern also tagged `areas: [demo]`.
    fn write_pattern_bundle(root: &Path) {
        w(
            root,
            "docs/knowledge/areas/demo/overview.md",
            "---\ntype: bee.area\ntitle: Demo area\ndescription: d\nbee:\n  id: demo-area\n  lifecycle: active\n  areas: [demo]\n  sources: [src/a.rs]\n---\nbody\n",
        );
        w(
            root,
            "docs/knowledge/patterns/p1.md",
            "---\ntype: bee.pattern\ntitle: Demo critical pattern\ndescription: d\nbee:\n  id: pattern-p1\n  lifecycle: active\n  areas: [demo]\n  critical: true\n---\nbody\n",
        );
    }

    fn write_capped_cell_touching(root: &Path, feature: &str, file: &str) {
        w(
            root,
            &format!(".bee/cells/{feature}-1.json"),
            &format!(
                r#"{{"id":"{feature}-1","feature":"{feature}","status":"capped","trace":{{"behavior_change":true,"outcome":"did the thing","files_changed":["{file}"],"capped_at":"2026-08-10T00:00:00.000Z"}}}}"#
            ),
        );
    }

    #[test]
    fn pattern_check_door_is_clear_with_no_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn pattern_check_door_reports_pending_when_no_verdicts_supplied() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.contains("pattern-p1=pending"), "{}", door.detail);
        assert!(door.detail.contains("--pattern-verdicts="), "{}", door.detail);
    }

    #[test]
    fn pattern_check_door_blocks_on_a_violated_verdict_naming_the_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let mut verdicts = HashMap::new();
        verdicts.insert("pattern-p1".to_string(), "violated".to_string());
        let door = build_pattern_check_door(root, "demo", &verdicts).unwrap();
        assert!(door.blocking);
        assert!(door.detail.contains("pattern-p1=violated"), "{}", door.detail);
    }

    #[test]
    fn pattern_check_door_passes_on_respected_or_not_applicable() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        for verdict in ["respected", "not-applicable"] {
            let mut verdicts = HashMap::new();
            verdicts.insert("pattern-p1".to_string(), verdict.to_string());
            let door = build_pattern_check_door(root, "demo", &verdicts).unwrap();
            assert!(!door.blocking, "{verdict}: {}", door.detail);
            assert!(door.detail.contains(&format!("pattern-p1={verdict}")), "{}", door.detail);
        }
    }

    #[test]
    fn pattern_check_door_clear_when_touched_files_miss_every_area() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/unrelated.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn parse_pattern_verdicts_accepts_known_words_and_drops_the_rest() {
        let parsed = parse_pattern_verdicts("pattern-a:violated, pattern-b:Respected ,malformed,pattern-c:bogus");
        assert_eq!(parsed.get("pattern-a").map(String::as_str), Some("violated"));
        assert_eq!(parsed.get("pattern-b").map(String::as_str), Some("respected"));
        assert_eq!(parsed.get("malformed"), None);
        assert_eq!(parsed.get("pattern-c"), None);
        assert_eq!(parsed.len(), 2);
    }
}
