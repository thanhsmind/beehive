// close, the report-only scribing/capture doors, and routing
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::textutil::truncate_chars_tail;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
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

/// provenance: cells.mjs bestScribingStampMs (verbs/status_full.rs:1718),
/// scoped to `feature`. LANE records are excluded by routing (a repo carrying
/// `.bee/lanes/*.json` delegates before this runs), so the lane arm is a
/// provable no-op here.
pub(crate) fn best_scribing_stamp_ms(root: &Path, feature: &str, state: &Map<String, Value>) -> Option<f64> {
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
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(&feature_value)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    best
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
pub(crate) fn scribing_debt(root: &Path, feature: &str) -> D<DebtSummary> {
    let state = read_state(root)?;
    let threshold = best_scribing_stamp_ms(root, feature, &state).unwrap_or(0.0);
    let mut ids = Vec::new();
    for cell in list_cells(root, feature, "capped")? {
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
/// (verbs/status_full.rs:2382) — only the COUNT reaches close's door text, so
/// pendingCaptureStubs' localeCompare sort cannot affect an emitted byte.
pub(crate) fn capture_queue_count(root: &Path) -> usize {
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
    stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .count()
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

/// provenance: bee.mjs buildCloseReportDoors — capture is DEFERRED (decision
/// c8e25271): both doors are report-only reminders, never a due-now step.
pub(crate) fn build_close_report_doors(root: &Path, feature: &str) -> D<Vec<Door>> {
    let scribing = scribing_debt(root, feature)?;
    let mut doors = Vec::new();
    doors.push(Door {
        door: "scribing-debt",
        blocking: false,
        detail: if scribing.count > 0 {
            format!(
                "pending — {} behavior_change cell(s) uncaptured ({}); settle later via bee-capturing",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        } else {
            "clear".to_string()
        },
        command: None,
    });
    let queue = capture_queue_count(root);
    doors.push(Door {
        door: "capture-queue",
        blocking: false,
        detail: if queue > 0 {
            format!("pending — {queue} capture stub(s) awaiting flush; settle later via bee-capturing")
        } else {
            "clear".to_string()
        },
        command: None,
    });
    Ok(doors)
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
    result.insert(
        "tests".into(),
        if run.undeclared {
            Value::Null
        } else {
            let mut tests = Map::new();
            tests.insert("ran_at".into(), Value::String(run.ran_at.clone()));
            tests.insert("green".into(), Value::Bool(true));
            tests.insert(
                "commands".into(),
                Value::Array(run.commands.iter().map(command_result_value).collect()),
            );
            tests.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
            Value::Object(tests)
        },
    );

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
    // Three conditions, all necessary: the close is GREEN (a red close never
    // reaches here), every one of the feature's cells is terminal (close's
    // only blocking door is tests, so a feature CAN close green holding an
    // open cell — that one is reported, not archived), and the repo has not
    // opted out. Reversible either way: `bee cells archive --feature <f>`
    // has an `unarchive` twin and the files stay in git.
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

/// provenance: state.mjs readLane / lanePath / requireLaneFeature — the
/// DELEGATION slice. The only lane read on close's path is
/// bestScribingStampMs' `readLane(root, feature)`, which touches exactly ONE
/// file: `.bee/lanes/<feature>.json`. Absent (or a malformed feature name,
/// which lanePath throws on and readLane catches as "no lane") it is a
/// provable no-op, so a lane-using repo still runs close natively for every
/// feature that has no lane record of its own. When the file IS there, the
/// lane record's own `last_scribing_run` joins the threshold AND a corrupt
/// record prints a console.warn with a path.relative-derived string — both
/// unported (the blueprint's lane/workflow coverage debt), so that ONE
/// feature delegates.
///
/// Workflows are deliberately NOT part of this guard: nothing close reads
/// (readState / listCells / captureQueue / readConfig) consults
/// `.bee/runtime/workflows/`, so their presence changes no byte here.
pub(crate) fn feature_has_lane_record(root: &Path, feature: &str) -> bool {
    let trimmed = js_trim(feature);
    if trimmed.is_empty()
        || trimmed.contains('\\')
        || trimmed.contains('/')
        || trimmed.contains("..")
    {
        return false; // lanePath throws -> readLane returns null (fail-open)
    }
    root.join(".bee").join("lanes").join(format!("{trimmed}.json")).exists()
}

pub(crate) fn run_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "dry-run"]) {
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
    if feature_has_lane_record(&root, &feature) {
        return None;
    }
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

    let ctx = match prelude("close", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out: R2<Out> = close_handler(&ctx.root, &feature, dry_run, declared, shell)
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
