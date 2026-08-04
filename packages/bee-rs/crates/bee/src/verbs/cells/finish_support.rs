// the test runner, the reservations-release half of finish, and the delegation pre-scans
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
use crate::textutil::truncate_chars_tail;
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

pub(crate) const FAILURE_EXCERPT_MAX_CHARS: usize = 500;

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
        let (exit, mut output, passed) = match spawn {
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
            output = js_trim(&output).to_string();
            let tail = truncate_chars_tail(&output, FAILURE_EXCERPT_MAX_CHARS);
            Some(if tail.is_empty() {
                format!(
                    "(no output; exit {})",
                    exit.map(jsjson::js_f64_to_string).unwrap_or_else(|| "null".to_string())
                )
            } else {
                tail
            })
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
pub(crate) fn release_reservations_for_agent(root: &Path, agent: &str, cell_id: &str) -> MR<ReleaseOutcome> {
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

    // xwh-2/gfb-1: ledger release per {cell, session} pair (holder 'main').
    let mut holds_released: u64 = 0;
    for (cell_v, session_v) in &pairs {
        let mut guard = acquire_named_lock(root, CROSS_WORKTREE_HOLDS_LOCK)?;
        let outcome = (|| -> MR<u64> {
            let mut store = read_holds_store(root)?;
            let released_at = utc_now();
            let mut count: u64 = 0;
            if let Some(Value::Array(holds)) = store.get_mut("holds") {
                for hold in holds.iter_mut() {
                    let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
                    if !unreleased {
                        continue;
                    }
                    if !matches!(hold.get("holder"), Some(Value::String(s)) if s == "main") {
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
                write_json_atomic(&holds_ledger_path(root), &store)
                    .map_err(|e| Fail::Thrown(format!("{e}")))?;
            }
            Ok(count)
        })();
        guard.release();
        holds_released += outcome?;
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
