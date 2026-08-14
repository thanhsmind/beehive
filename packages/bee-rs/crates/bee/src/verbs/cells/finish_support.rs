// the test runner, the reservations-release half of finish, and the delegation pre-scans
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{self, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots, StoreRoots};
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

// ─── test runner (lib/test-runner.mjs) ─────────────────────────────────────

pub(crate) const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";

pub(crate) fn test_results_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("test-results.json")
}

pub(crate) struct CmdRun {
    pub(crate) command: String,
    pub(crate) exit: Option<f64>,
    pub(crate) duration_ms: f64,
    pub(crate) failure_excerpt: Option<String>,
}

pub(crate) struct TestsRun {
    pub(crate) green: bool,
    pub(crate) ran_at: String,
    pub(crate) commands: Vec<CmdRun>,
}

/// posixShell(): on win32 the shared resolver's Git Bash (never the WSL
/// launcher — see crate::shell); elsewhere `shell: true` already IS a POSIX
/// sh, which this verb's own cmd.exe/`/bin/sh` fork below handles.
pub(crate) fn posix_shell() -> Option<&'static str> {
    if !cfg!(windows) {
        return None;
    }
    crate::shell::posix_shell()
}

/// spawnDeclaredCommand — POSIX sh (Git Bash) on win32 when available, the
/// platform shell otherwise. Output captured lossily (Node utf8 decode).
///
/// `bee cells finish` runs the proof command through here, so it takes the
/// SAME resolution `bee test` and `bee close` take: a cap must not depend on
/// which of three copies of this function the caller happened to reach.
pub(crate) fn spawn_declared(command: &str, cwd: &Path) -> Result<(Option<i32>, String, String), String> {
    let mut cmd = if posix_shell().is_some() {
        let mut c = crate::shell::command().expect("resolver just answered Some");
        c.args(["-c", command]);
        c
    } else if cfg!(windows) {
        // Node shell:true on win32: cmd.exe /d /s /c "<command>".
        let mut c = std::process::Command::new("cmd.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.raw_arg(format!("/d /s /c \"{command}\""));
        }
        c
    } else {
        let mut c = std::process::Command::new("/bin/sh");
        c.args(["-c", command]);
        c
    };
    match cmd.current_dir(cwd).output() {
        Ok(out) => Ok((
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )),
        Err(e) => Err(format!("{e}")), // spawn error — Node embeds its own message (residual)
    }
}

/// runDeclaredTests over an already-normalized declared command list.
pub(crate) fn run_declared_tests(root: &Path, commands: &[String]) -> MR<TestsRun> {
    let ran_at = utc_now();
    let mut results: Vec<CmdRun> = Vec::new();
    let mut green = true;
    for command in commands {
        let started = std::time::Instant::now();
        let spawn = spawn_declared(command, root);
        let duration_ms = started.elapsed().as_millis() as f64;
        let (exit, output, passed) = match spawn {
            Ok((code, stdout, stderr)) => {
                let out = format!("{stdout}{stderr}");
                (code.map(|c| c as f64), out, code == Some(0))
            }
            Err(message) => {
                let out = format!("\n[bee test] spawn error: {message}");
                (None, out, false)
            }
        };
        if !passed {
            green = false;
        }
        let excerpt = if passed {
            None
        } else {
            let trimmed = js_trim(&output).to_string();
            Some(fsutil::failure_excerpt(&trimmed, exit.map(|e| e as i64)))
        };
        results.push(CmdRun { command: command.clone(), exit, duration_ms, failure_excerpt: excerpt });
    }
    let record = tests_record_value(&ran_at, green, &results);
    write_json_atomic(&test_results_path(root), &record).map_err(|e| Fail::Thrown(format!("{e}")))?;
    Ok(TestsRun { green, ran_at, commands: results })
}

pub(crate) fn tests_record_value(ran_at: &str, green: bool, commands: &[CmdRun]) -> Value {
    let rows: Vec<Value> = commands
        .iter()
        .map(|c| {
            let mut row = Map::new();
            row.insert("command".into(), Value::String(c.command.clone()));
            row.insert(
                "exit".into(),
                c.exit.and_then(Number::from_f64).map(Value::Number).unwrap_or(Value::Null),
            );
            row.insert(
                "duration_ms".into(),
                Number::from_f64(c.duration_ms).map(Value::Number).unwrap_or(Value::Null),
            );
            row.insert(
                "failure_excerpt".into(),
                c.failure_excerpt.clone().map(Value::String).unwrap_or(Value::Null),
            );
            Value::Object(row)
        })
        .collect();
    let mut record = Map::new();
    record.insert("ran_at".into(), Value::String(ran_at.to_string()));
    record.insert("green".into(), Value::Bool(green));
    record.insert("commands".into(), Value::Array(rows));
    Value::Object(record)
}

/// firstFailureLine — first non-empty line of the first failing excerpt.
pub(crate) fn first_failure_line(run: &TestsRun) -> Option<String> {
    let failing = run
        .commands
        .iter()
        .find(|c| c.failure_excerpt.as_deref().map(|e| !e.is_empty()).unwrap_or(false))?;
    failing
        .failure_excerpt
        .as_deref()
        .unwrap_or("")
        .split('\n')
        .map(|l| js_trim(l.trim_end_matches('\r')))
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

// ─── wfl-1 (docs/history/workflow-lessons/plan.md) — the structured worker
// Result form ─────────────────────────────────────────────────────────────
// `bee cells finish --report <json-string>` — the machine-readable
// counterpart to worker-cell.md's Result-form block, so tending reads the
// form instead of parsing prose. Exactly REPORT_KEYS, each required; an
// unknown key or a missing one is refused by name (frd-1's own "name the
// flag" posture). Absent `--report` never touches `trace.report` at all —
// old finish behavior stays byte-identical.

/// The Result form's five keys, in the order worker-cell.md documents them.
pub(crate) const REPORT_KEYS: [&str; 5] = ["outcome", "commit", "files", "tests", "deviations"];

/// parseReportFlag — `--report`'s raw JSON string validated against the
/// worker Result-form shape. `outcome`/`commit` are non-empty strings;
/// `files`/`deviations` are arrays (their own element shape is the
/// worker's business, not this gate's); `tests` is the literal string
/// `"green"` or `"red"`. Every refusal names the offending key so a cold
/// reader fixes it without re-deriving the shape from this function.
pub(crate) fn parse_report_flag(raw: &str) -> MR<Value> {
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| Fail::Thrown(format!("cells finish: --report is not valid JSON: {e}")))?;
    let Value::Object(map) = parsed else {
        return Err(Fail::Thrown(format!(
            "cells finish: --report must be a JSON object with keys {}.",
            REPORT_KEYS.join(", ")
        )));
    };
    for key in map.keys() {
        if !REPORT_KEYS.contains(&key.as_str()) {
            return Err(Fail::Thrown(format!(
                "cells finish: --report has unknown key \"{key}\" — only {} are allowed.",
                REPORT_KEYS.join(", ")
            )));
        }
    }
    for key in REPORT_KEYS {
        if !map.contains_key(key) {
            return Err(Fail::Thrown(format!(
                "cells finish: --report is missing required key \"{key}\"."
            )));
        }
    }
    match map.get("outcome") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"outcome\" must be a non-empty string.".to_string(),
            ))
        }
    }
    match map.get("commit") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"commit\" must be a non-empty string.".to_string(),
            ))
        }
    }
    match map.get("files") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"files\" must be an array.".to_string(),
            ))
        }
    }
    match map.get("deviations") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"deviations\" must be an array.".to_string(),
            ))
        }
    }
    match map.get("tests") {
        Some(Value::String(s)) if s == "green" || s == "red" => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"tests\" must be the string \"green\" or \"red\"."
                    .to_string(),
            ))
        }
    }
    Ok(Value::Object(map))
}

// ─── D6 (docs/history/hook-teeth/CONTEXT.md) — the cell commit trailer ────
//
// `cells finish` refuses to cap a cell whose files_changed is non-empty
// unless a commit carrying a `cell: <id>` trailer line exists in the RECENT
// history of the feature's own branch — bee's one-commit-per-cell rule
// (AGENTS.md "Care for the session"), made mechanical. `--commit-pending
// <reason>` escapes it, D2's `--fix-first` convention: the reason lands on
// the cap's own trace, never silently.
//
// Branch resolution matters here in a way it would not for an arbitrary
// feature: THIS very feature's `cells finish` calls typically run from the
// MAIN checkout (bee-swarming's worker convention dispatches a worker whose
// `bee cells finish` call reaches the shared store), while the worker's own
// commits land on the FEATURE's WORKTREE branch — never on main. Scanning
// `root`'s own HEAD history in that case would see the wrong branch
// entirely and refuse every legitimate cap. `find_granted_worktree_for_feature`
// (verbs/status_full/topology.rs, landed for ct-1's re-lane worktree block —
// see state_group/workflows.rs's `route_worktree_block`) is exactly the
// bidirectional gitdir walk that answers "does this feature have a granted
// worktree, and where is it" without a second implementation of that
// resolution, so it is reused here rather than re-derived. When no grant
// exists for the feature (an ordinary same-checkout feature, or one with no
// worktree split at all), the CURRENT checkout's own HEAD history is the
// right — and only — thing to scan.

/// The bounded window D6 scans, nearest-to-HEAD first (`git log`'s own
/// order): `cells finish` cap-checks a cell that was JUST claimed and
/// worked, so the qualifying commit is always near HEAD. Walking the WHOLE
/// branch history would cost real time on a long-lived repo for zero added
/// correctness — the same "bounded window" tradeoff D2's red-base read
/// (`.bee/logs/test-results.json`, a single record rather than a log) makes
/// in its own idiom.
pub(crate) const COMMIT_TRAILER_WINDOW: u32 = 50;

/// `cell: <id>` — the ONE trailer shape D6 recognizes, matched as an exact
/// (trimmed) whole line inside a commit's FULL message body, never a
/// substring match: a commit that merely mentions the cell id in prose
/// ("touch up bh-6 handling") must not satisfy this.
pub(crate) fn cell_commit_trailer(id: &str) -> String {
    format!("cell: {id}")
}

/// The history root D6 scans for `feature`: the granted worktree's HEAD
/// history when one exists for it, else `fallback_root`'s own HEAD history.
/// `fallback_root` is the store root `cap_cell_from_flags` already has in
/// hand — the ordinary-checkout case needs no extra resolution at all.
pub(crate) fn commit_trailer_history_root(fallback_root: &Path, feature: Option<&str>) -> PathBuf {
    if let Some(feature) = feature {
        if let Some((_, worktree_root)) =
            crate::verbs::status_full::find_granted_worktree_for_feature(fallback_root, feature)
        {
            return PathBuf::from(worktree_root);
        }
    }
    fallback_root.to_path_buf()
}

/// `git log -n <window> --format=%B%x00` over `cwd`'s HEAD. `%B` is the
/// full, UNWRAPPED message body (never `%s`/`%b`'s subject/body split, which
/// could fold a trailer line into the subject and hide it from a
/// line-by-line scan); the `%x00` separator keeps one commit's body from
/// bleeding into the next when splitting the combined output back apart. A
/// git that cannot be spawned, a `cwd` with no commits yet, or any other
/// non-zero exit answers `false` — fail CLOSED, matching this check's own
/// "unless --commit-pending" contract: an unprovable history is never
/// silently treated as satisfying the rule, it is proven or escaped.
///
/// Shells out through `crate::verbs::worktree::run_git` — the crate's own
/// `spawnSync('git', argv, {cwd})` pattern (worktree-store.mjs runGit, no
/// shell involved) — rather than a second ad-hoc git invocation style.
pub(crate) fn commit_trailer_present(cwd: &Path, id: &str) -> bool {
    let window = COMMIT_TRAILER_WINDOW.to_string();
    let out = crate::verbs::worktree::run_git(cwd, &["log", "-n", &window, "--format=%B%x00"]);
    if out.status != Some(0) {
        return false;
    }
    let trailer = cell_commit_trailer(id);
    out.stdout
        .unwrap_or_default()
        .split('\u{0}')
        .any(|body| body.lines().any(|line| js_trim(line) == trailer))
}

// ─── FULL-door topology (wf-1) ──────────────────────────────────────────────
// `cells finish`'s own split of `resolve_store_root_worktree`'s
// `StoreRoots`, per the logged decision: the cell record and its claim
// resolve at MAIN (one ledger, and the claim being validated already lives
// there); the declared test command's cwd is the calling worktree when
// granted (the changed code is the evidence); the hold-release topology is
// `StoreRoots::hold_topology()` unchanged (roots.rs:541-551). A pure
// function of `StoreRoots` — no cwd read — so it is exercisable directly
// against a `resolve_store_root_worktree` fixture, the same shape
// reservations/tests.rs's `hold_topology_matches_node_for_every_checkout_kind`
// already uses.
pub(crate) fn finish_topology(roots: &StoreRoots) -> (PathBuf, PathBuf, Option<(PathBuf, String)>) {
    let cells_root = roots.main_root();
    let test_root = match &roots.linked {
        Some(l) if l.granted() => l.worktree_root.clone(),
        _ => cells_root.clone(),
    };
    let topo = roots.hold_topology();
    (cells_root, test_root, topo)
}

// ─── reservations-release subset (finish's release half) ───────────────────
// Provenance: lib/reservations.mjs release/listReservations + bee.mjs
// releaseReservationsForAgent, mirrored from verbs/reservations.rs's own
// release_exec (those fns are module-private there; this copy keeps cells.rs
// self-contained per the one-file rule).

pub(crate) const CROSS_WORKTREE_HOLDS_LOCK: &str = "cross-worktree-holds";

pub(crate) fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn holds_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// worktree-holds.mjs readStore — `readJson(path, null)` then a shape check
/// that turns anything without an array `holds` into `{holds: []}`. A corrupt
/// ledger warns and takes that same `{holds: []}` fallback (Node's `null`
/// fallback reached it through `!store`). Null hold ENTRIES still delegate:
/// that is a JS-exotic shape, not a parse failure.
pub(crate) fn read_holds_store(root: &Path) -> MR<Value> {
    let ledger = holds_ledger_path(root);
    let store = match read_json(&ledger) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json_once(&ledger);
            None
        }
        ReadJson::Parsed(v) => Some(rsv::js_numberify(&v).map_err(|_| Fail::Delegate)?),
    };
    let ok_shape = store
        .as_ref()
        .map(|s| matches!(s.get("holds"), Some(Value::Array(_))))
        .unwrap_or(false);
    if !ok_shape {
        return Ok(json!({ "holds": [] }));
    }
    let store = store.unwrap();
    if let Some(Value::Array(holds)) = store.get("holds") {
        if holds.iter().any(|h| h.is_null()) {
            return Err(Fail::Delegate);
        }
    }
    Ok(store)
}

pub(crate) fn list_path_lease_records(root: &Path) -> MR<Vec<Map<String, Value>>> {
    let control = control_root(root)?;
    let leases_root = control.join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(dir.join(&name)) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    // readLeaseSafe: corrupt silently skipped (no warn in Node).
                    if let Value::Object(m) = rsv::js_numberify(&parsed).map_err(|_| Fail::Delegate)? {
                        let is_path =
                            matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                        if is_path {
                            out.push(m);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

pub(crate) fn path_lease_file(control_root: &Path, raw_path_id: &str) -> PathBuf {
    let canonical = rsv::res_normalize_path(raw_path_id);
    let resource_key = format!("path:{canonical}");
    control_root
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)))
}

pub(crate) fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> MR<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match rsv::date_parse_val(Some(v)).map_err(|_| Fail::Delegate)? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

pub(crate) struct ResvLite {
    pub(crate) agent: Option<Value>,
    pub(crate) cell: Option<Value>,
    pub(crate) path: String,
    pub(crate) session: Option<Value>,
}

pub(crate) fn lease_to_resv_lite(rec: &Map<String, Value>) -> MR<ResvLite> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(Fail::Delegate),
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => Value::String(s["agent:".len()..].to_string()),
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if js_truthy(v) && !matches!(v, Value::String(s) if s == rsv::SESSIONLESS_SESSION_ID) => {
            Some(v.clone())
        }
        _ => None,
    };
    Ok(ResvLite {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        session,
    })
}

pub(crate) struct ReleaseOutcome {
    pub(crate) paths: Vec<String>,
    /// Mirrors Node's holdsReleased (reservations-release parity); the finish
    /// text/result never surfaces it — kept for the ledger write's own count.
    #[allow(dead_code)]
    pub(crate) holds_released: u64,
}

/// bee.mjs releaseReservationsForAgent(root, agent, cell) — matched-rows
/// derivation, local lease release, {cell, session}-scoped ledger release.
///
/// `topo` is `StoreRoots::hold_topology()` (wf-1) — `(main_root, holder)`,
/// `None` for an ungranted linked worktree — replacing the ordinary-only
/// assumption (ledger at `root`, holder hardcoded `"main"`) this used to
/// carry unconditionally, matching `verbs/reservations/release.rs`'s own
/// `release_exec` topology gate: no topology, no lock, no ledger read, the
/// ledger step skipped entirely rather than silently guarding an empty one.
/// The local lease release (`control_root(root)`) is unaffected — leases
/// always live off `root`, independent of the ledger's own home.
pub(crate) fn release_reservations_for_agent(
    topo: Option<(&Path, &str)>,
    root: &Path,
    agent: &str,
    cell_id: &str,
) -> MR<ReleaseOutcome> {
    let now = rsv::now_ms();
    let records = list_path_lease_records(root)?;
    let mut matched: Vec<ResvLite> = Vec::new();
    for rec in &records {
        if lease_record_expired(rec, now)? {
            continue; // activeOnly
        }
        let resv = lease_to_resv_lite(rec)?;
        let agent_match = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        let cell_match =
            matches!(&resv.cell, Some(v) if v == &Value::String(cell_id.to_string()));
        if agent_match && cell_match {
            matched.push(resv);
        }
    }
    let mut pairs: Vec<(Value, Option<Value>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in &matched {
        let Some(cell_v) = r.cell.as_ref().filter(|c| js_truthy(c)) else { continue };
        let session_v = r.session.as_ref().filter(|s| js_truthy(s)).cloned();
        let key = format!(
            "{}::{}",
            jsjson::js_to_string(cell_v),
            session_v.as_ref().map(|s| jsjson::js_to_string(s)).unwrap_or_default()
        );
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v.clone(), session_v));
        }
    }

    // reservations.mjs release(root, {agent, cell}).
    let control = control_root(root)?;
    let trimmed_agent = js_trim(agent);
    for rec in &records {
        let lease_agent = match rec.get("workspace_id") {
            Some(Value::String(s)) if s.starts_with("agent:") => {
                Value::String(s["agent:".len()..].to_string())
            }
            Some(other) => other.clone(),
            None => continue,
        };
        if !matches!(&lease_agent, Value::String(s) if s == trimmed_agent) {
            continue;
        }
        let matches_cell = matches!(
            rec.get("workflow_id"),
            Some(v) if v == &Value::String(cell_id.to_string())
        );
        if !matches_cell {
            continue;
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Fail::Thrown(format!("{e}"))),
        }
    }

    // xwh-2/gfb-1: ledger release per {cell, session} pair, gated on a
    // hold-worthy topology (wf-1). `None` — an ungranted linked worktree —
    // skips the whole ledger step, same as reservations::release_exec.
    let mut holds_released: u64 = 0;
    if let Some((main_root, holder)) = topo {
        for (cell_v, session_v) in &pairs {
            let mut guard = acquire_named_lock(main_root, CROSS_WORKTREE_HOLDS_LOCK)?;
            let outcome = (|| -> MR<u64> {
                let mut store = read_holds_store(main_root)?;
                let released_at = utc_now();
                let mut count: u64 = 0;
                if let Some(Value::Array(holds)) = store.get_mut("holds") {
                    for hold in holds.iter_mut() {
                        let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
                        if !unreleased {
                            continue;
                        }
                        if !matches!(hold.get("holder"), Some(Value::String(s)) if s == holder) {
                            continue;
                        }
                        if let Some(s) = session_v {
                            if !matches!(hold.get("session"), Some(v) if v == s) {
                                continue;
                            }
                        }
                        if !matches!(hold.get("cell"), Some(v) if v == cell_v) {
                            continue;
                        }
                        if let Value::Object(m) = hold {
                            m.insert("released_at".into(), Value::String(released_at.clone()));
                        }
                        count += 1;
                    }
                }
                if count > 0 {
                    write_json_atomic(&holds_ledger_path(main_root), &store)
                        .map_err(|e| Fail::Thrown(format!("{e}")))?;
                }
                Ok(count)
            })();
            guard.release();
            holds_released += outcome?;
        }
    }

    let mut paths: Vec<String> = Vec::new();
    for r in &matched {
        if !paths.contains(&r.path) {
            paths.push(r.path.clone());
        }
    }
    Ok(ReleaseOutcome { paths, holds_released })
}

// RETIRED at the R6 cutover: the cap-time impact-registry cross-check (E1).
// It queried `scripts/impact-registry.json` — a suite-impact graph derived
// by parsing `scripts/run_verify.mjs` and the `.mjs` import closure — to warn
// when a cell's verify command missed a direct-edge Node suite. Both the
// registry and the graph it was derived from are gone with the Node tree, and
// the cargo suite that replaced them runs whole in ~20s, so there is no
// filtering left to advise about. `trace.warnings` keeps its slot (existing
// capped cells carry it) but this producer no longer exists.

// ─── fa-1: diff-vs-test advisory ────────────────────────────────────────────
// A NEW producer for the `trace.warnings` slot the E1 retirement above left
// empty: a large commit that touches no test-shaped path is worth a nudge.
// Runs at the GREEN-cap path of `cells finish` alone (the caller gates this
// on `finish`, mirroring D6's own "finish only" scoping) — every earlier
// door (test-green, commit-trailer, lane/worker checks) already let the cap
// through by the time this producer runs, so it can only ever ADD a line to
// `trace.warnings` and print that SAME line once to stderr; it never touches
// the exit code, the cap outcome, or any other part of the result shape.
// Every git failure — no git on PATH, HEAD's body doesn't carry THIS cell's
// own `cell: <id>` trailer, numstat unreadable — is a silent skip: this is
// best-effort telemetry, never a second gate alongside D6's real one.

/// The default for config key `finish.advisory_untested_lines` —
/// `{finish: {advisory_untested_lines: N}}` in `.bee/config.json`, the same
/// nested-object shape `guards.write_policy`
/// (write_guard/checks.rs `resolve_write_policy_mode`) already establishes
/// for a dotted doc-facing key name. `0` is the documented disable, never a
/// "run always" typo — the producer checks for it explicitly before doing
/// any git work.
pub(crate) const DEFAULT_ADVISORY_UNTESTED_LINES: u64 = 150;

/// advisoryUntestedLinesThreshold — absent/null/non-numeric/negative all
/// fall back to the default silently (`resolve_write_policy_mode`'s own
/// posture for a single-scalar nested key, not `capture_queue_threshold`'s
/// warn-and-fallback: a malformed advisory threshold is worth exactly one
/// nudge line, never a stderr warning of its own).
pub(crate) fn advisory_untested_lines_threshold(config: &Map<String, Value>) -> u64 {
    match config.get("finish").and_then(|f| f.get("advisory_untested_lines")) {
        Some(Value::Number(n)) => match n.as_u64() {
            Some(v) => v,
            None => DEFAULT_ADVISORY_UNTESTED_LINES,
        },
        _ => DEFAULT_ADVISORY_UNTESTED_LINES,
    }
}

/// headCommitCarriesTrailer — unlike [`commit_trailer_present`]'s
/// `COMMIT_TRAILER_WINDOW`-commit scan (D6 only needs to prove a qualifying
/// commit exists SOMEWHERE near HEAD), the advisory describes the diff of
/// the ONE commit the one-commit-per-cell convention names as THIS cell's
/// own: HEAD, and HEAD alone. A HEAD whose body does not carry the exact
/// trailer line (an unrelated commit landed on top, `--commit-pending`
/// escaped D6's own check entirely, this checkout has no commits yet, git
/// itself is missing) answers `false` — the producer only ever describes a
/// commit it is confident belongs to `id`.
pub(crate) fn head_commit_carries_trailer(cwd: &Path, id: &str) -> bool {
    let out = crate::verbs::worktree::run_git(cwd, &["log", "-1", "--format=%B"]);
    if out.status != Some(0) {
        return false;
    }
    let trailer = cell_commit_trailer(id);
    out.stdout.unwrap_or_default().lines().any(|line| js_trim(line) == trailer)
}

/// One `git show --numstat` row: `<added>\t<deleted>\t<path>`.
pub(crate) struct NumstatRow {
    pub(crate) added: u64,
    pub(crate) deleted: u64,
    pub(crate) path: String,
}

/// headCommitNumstat — `git show -1 --numstat --format=` HEAD: one row per
/// changed path, no commit-message preamble (`--format=` empty — the same
/// "spawn once, parse exactly what was asked for" posture
/// [`commit_trailer_present`]'s `%B`-only spawn already takes, rather than a
/// second ad-hoc parse of git's default log format). A binary file's
/// `-\t-\tpath` row parses both fields as `0`: line counts are not
/// observable for it, and treating the dash as "large" would be a false
/// positive this producer has no way to justify. `None` on any non-zero git
/// exit (no git, no HEAD, a detached/empty repo) — the caller treats that
/// exactly like every other silent-skip path.
pub(crate) fn head_commit_numstat(cwd: &Path) -> Option<Vec<NumstatRow>> {
    let out = crate::verbs::worktree::run_git(cwd, &["show", "-1", "--numstat", "--format="]);
    if out.status != Some(0) {
        return None;
    }
    let mut rows = Vec::new();
    for line in out.stdout.unwrap_or_default().lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let (Some(added), Some(deleted), Some(path)) = (parts.next(), parts.next(), parts.next()) else {
            continue; // an unparsable row is skipped, not fatal to the whole read
        };
        rows.push(NumstatRow {
            added: added.parse().unwrap_or(0),
            deleted: deleted.parse().unwrap_or(0),
            path: path.to_string(),
        });
    }
    Some(rows)
}

/// pathLooksLikeTest — four shapes, any one of which qualifies a changed
/// path as test-shaped: a path SEGMENT (a `/`-split component, so
/// `contest/a.rs` and `latest.rs` never false-positive on a `test`
/// substring) named exactly `test` or `tests`; the bare filename
/// `tests.rs`; or a filename carrying `_test.` / `.test.` anywhere before
/// its extension (`foo_test.rs`, `foo.test.ts`).
pub(crate) fn path_looks_like_test(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized.split('/').collect();
    if segments.iter().any(|s| *s == "test" || *s == "tests") {
        return true;
    }
    let filename = segments.last().copied().unwrap_or("");
    filename == "tests.rs" || filename.contains("_test.") || filename.contains(".test.")
}

/// advisoryUntestedLinesLine — the ONE stderr line this producer ever
/// prints, and byte-identical to the ONE line appended to `trace.warnings`
/// (a single representation, never a stderr copy plus a differently-worded
/// trace copy).
pub(crate) fn advisory_untested_lines_line(total_lines: u64, threshold: u64) -> String {
    format!(
        "advisory: this cap's commit changes {total_lines} line(s) (over the {threshold}-line finish.advisory_untested_lines threshold) but touches no test-shaped path — consider adding test coverage."
    )
}

/// diffVsTestAdvisory — the producer itself. `None` on every silent-skip
/// path: the threshold is `0` (disabled), HEAD doesn't carry `id`'s own
/// trailer, numstat is unreadable, at least one changed path already looks
/// test-shaped, or the total changed-line count does not exceed the
/// threshold. `Some(line)` is the one line the caller both prints to
/// stderr and appends to `trace.warnings` — this function has no other
/// side effect and never returns an `Err`.
pub(crate) fn diff_vs_test_advisory(cwd: &Path, id: &str, threshold: u64) -> Option<String> {
    if threshold == 0 {
        return None;
    }
    if !head_commit_carries_trailer(cwd, id) {
        return None;
    }
    let rows = head_commit_numstat(cwd)?;
    if rows.iter().any(|r| path_looks_like_test(&r.path)) {
        return None;
    }
    let total: u64 = rows.iter().map(|r| r.added + r.deleted).sum();
    if total <= threshold {
        return None;
    }
    Some(advisory_untested_lines_line(total, threshold))
}

// ─── delegation pre-scans ──────────────────────────────────────────────────
// The mutators must never return None after an output or a write, so the
// JS-exotic store shapes that still delegate (an array where a record is
// expected, a string/array `trace`) are probed up front; Thrown-class
// outcomes are ignored here (the real flow reproduces them at Node's own
// point in the order).
//
// CUTOVER: `prescan_cells_store` used to walk EVERY active and archived cell
// file so a corrupt one could delegate before any output. Corrupt JSON is
// native now, and that walk would have warned about cell files the command
// never reads — so it is deleted rather than kept as a second, louder read.
// The probes that survive read exactly the file the flow reads, and
// `warn_corrupt_json_once` keeps that from printing twice.

pub(crate) fn delegate_only<T>(result: MR<T>) -> MR<()> {
    match result {
        Err(Fail::Delegate) => Err(Fail::Delegate),
        _ => Ok(()),
    }
}

pub(crate) fn prescan_claim(root: &Path, id: &str) -> MR<()> {
    let control = control_root(root)?;
    if id_pattern_ok(id) {
        delegate_only(read_claim(&control, id))?;
    }
    Ok(())
}
