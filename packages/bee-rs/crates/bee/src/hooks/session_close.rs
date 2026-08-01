// bee hook session-close — Rust port of hooks/bee-session-close.mjs (Stop +
// PreCompact). The STOP path — the frequent one — is fully native: the perf
// refresh, the GitHub-#18 bypass net (decision:"block"), the capture-queue /
// capture / decision nudges, and the "hive door open" mid-phase warning all
// render byte-identically to the Node wrapper. Always exits 0.
//
// Ported lib functions (provenance: the vendored <root>/.bee/bin/lib copies,
// byte-identical to packages/bee/lib at port time):
//   - state.mjs: readState (via crate::state::read_state_brief), readHandoff
//     (truthiness only), resolvePipeline (+ readLane/laneRecordFrom,
//     requireLaneFeature, controlRootFor's linked-worktree behavior incl. the
//     WorktreeLinkInvalidError throw emulation), resolveProductRoot,
//     bypassLevel (crate::state), hookEnabled.
//   - claims.mjs: sessionPath/readSession (requireId rules).
//   - inject.mjs: shouldInject / markInjected (30-min dedup, legacy cache
//     migration).
//   - decisions.mjs: activeDecisions({recent:1}) reduced to the newest active
//     decision's id/date (supersede/redact filtering; tag overlay never
//     touches id/date).
//   - capture.mjs: pendingCaptureStubs.
//   - knowledge.mjs: bundleMode (parseFrontmatter subset, listBundleMarkdown).
//   - reservations.mjs + lease-store.mjs: listReservations({activeOnly}) over
//     the sharded path leases, with reservations.mjs's own fail-open
//     control-root resolution.
//   - cells.mjs: listCells({status:'claimed'}) (localeCompare-en-numeric sort
//     approximated — see cmp_locale_numeric).
//   - perf.mjs: claudeProjectsRoot, encodeProjectDir, resolveTranscript,
//     rollupTranscript (aggregateUsage / walkSubagents / runningTimeMs /
//     detectParallel), sessionRecord, upsertSessionRecords,
//     readSessionRecords, buildMatrixFromLog, renderMatrixHtml, writeReport,
//     humanizeMs.
//
// CUTOVER (2026-08-01). Present-but-corrupt JSON used to be a strangler bail,
// decided by a PRE-FLIGHT probe over every file the Stop path could read
// (state/HANDOFF/inject caches/session record/bound lane/cells/perf sidecar
// metas), because Node's readJson warning quoted V8's parse sentence. That
// probe is DELETED — it would now warn about each bad file a second time — and
// every corruption is handled at its real read, warning once in bee's own
// words and taking readJson's `null` fallback: state.json reads as
// defaultState(), HANDOFF.json reads as absent (so the mid-phase "hive door
// open" warning still fires), a corrupt inject cache falls through to the
// legacy location and then to `{}` (the nudge re-injects), a corrupt session
// record reads as no session, a corrupt lane still takes readLane's second
// warn and the LANE_CORRUPT refusal, a corrupt cell is skipped from the
// claimed list, and a corrupt perf meta is skipped. The hook still exits 0.
//
// Warnings are QUEUED, not printed (see queue_corrupt_json_warning): the hook
// buffers all output so that a run which later delegates has emitted nothing.
//
// Strangler bails (Outcome::Delegate), all decided BEFORE any write/output:
//   - event == "PreCompact": the compaction path (intent anchor re-assert,
//     compaction record, forced nudges) delegates to Node wholesale.
//   - an inject cache whose parsed content is a non-object, and a truthy
//     non-object approved_gates (both Node spread/assignment exotica).
// (A corrupt .bee/config.json is native, inside crate::state::read_config_raw;
// that reader prints immediately rather than queueing, so a bad config plus a
// non-object inject cache can leak that one line before the delegate.)
// Accepted divergences (documented for the port record):
//   - localeCompare('en', numeric) approximated for cell-id sorting (exact
//     for lowercase alnum/hyphen slug ids).
//   - Date.parse of non-ISO local-time strings is not replicated (bee only
//     writes toISOString values).
//   - toFixed/Math.round exact-decimal tie-rounding in the derived
//     performance.html report (never observable on stdout/stderr).

use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "session-close";
const INJECT_INTERVAL_MS: f64 = 30.0 * 60.0 * 1000.0;
const DECISION_RECENT_MS: f64 = 6.0 * 3600.0 * 1000.0;

/// Internal control flow: Delegate = re-run the Node wrapper; Crash(msg) =
/// the Node path would THROW here (main's catch → logCrash → advisory emit).
#[derive(Debug)]
enum Flow {
    Delegate,
    Crash(String),
}

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    match run_inner(argv, stdin) {
        Ok(()) => Outcome::Done(ExitCode::SUCCESS),
        Err(()) => Outcome::Delegate,
    }
}

fn run_inner(argv: &[String], stdin: &str) -> Result<(), ()> {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Ok(());
    };
    if !crate::hooks::adapter::bee_installed(&root) {
        return Ok(());
    }
    // PreCompact (intent anchor + compaction record + forced nudges): the
    // Node wrapper owns this event; nothing was written or printed yet.
    if ctx.event == "PreCompact" {
        return Err(());
    }

    let session_id = get_session_id(&ctx.payload);
    clear_corrupt_json_warnings(); // one queue per run (tests reuse the thread)

    // ── pre-flight ──────────────────────────────────────────────────────────
    // Only the two remaining delegate classes are decided here (a corrupt
    // CONFIG file, and a non-object inject cache), both BEFORE any side effect.
    // Corrupt data files are no longer probed: each real read below warns for
    // itself, exactly once, and fails open.
    let config = match preflight(&root) {
        Ok(config) => config,
        Err(()) => return Err(()),
    };

    // ── perf refresh (best-effort, before the hookEnabled check — as in the
    // .mjs, and never allowed to touch the exit code or the advisory path) ──
    if let Err(msg) = perf_refresh(&root, session_id.as_deref()) {
        log_crash(Some(&root), HOOK_NAME, &msg, Some("perf-refresh"));
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut parts: Vec<String> = Vec::new();

    // The .mjs's advisory try/catch: a Crash logs and falls through to the
    // emit of whatever parts were already collected.
    match advisory(&root, &ctx, &config, session_id.as_deref(), &mut parts, &mut stderr) {
        Ok(AdvisoryOutcome::Disabled) => {
            flush(&stdout, &stderr);
            return Ok(());
        }
        Ok(AdvisoryOutcome::Block(reason)) => {
            stdout.push_str(&encode_block(&reason));
            flush(&stdout, &stderr);
            return Ok(());
        }
        Ok(AdvisoryOutcome::Done) => {}
        Err(Flow::Delegate) => return Err(()),
        Err(Flow::Crash(msg)) => {
            log_crash(Some(&root), HOOK_NAME, &msg, ctx.source);
        }
    }

    flush(&stdout, &stderr);
    if !parts.is_empty() {
        emit_hook_output(&ctx, &parts.join("\n"), "Stop");
    }
    Ok(())
}

fn flush(stdout: &str, stderr: &str) {
    use std::io::Write;
    // Corrupt-JSON warnings first: that is where Node's readJson emitted them,
    // ahead of anything the advisory itself wrote.
    let corrupt = take_corrupt_json_warnings();
    if !corrupt.is_empty() {
        let _ = std::io::stderr().write_all(corrupt.as_bytes());
    }
    if !stderr.is_empty() {
        let _ = std::io::stderr().write_all(stderr.as_bytes());
    }
    if !stdout.is_empty() {
        let _ = std::io::stdout().write_all(stdout.as_bytes());
    }
}

enum AdvisoryOutcome {
    Disabled,
    Block(String),
    Done,
}

fn advisory(
    root: &Path,
    ctx: &HookContext,
    config: &Map<String, Value>,
    session_id: Option<&str>,
    parts: &mut Vec<String>,
    stderr: &mut String,
) -> Result<AdvisoryOutcome, Flow> {
    // hookEnabled (config.hooks['session-close'] !== false).
    if matches!(config.get("hooks").and_then(|h| h.get(HOOK_NAME)), Some(Value::Bool(false))) {
        return Ok(AdvisoryOutcome::Disabled);
    }

    // GitHub #18 — the mechanical bypass net takes precedence over every
    // advisory; when it fires we emit ONLY the block.
    if let Some(reason) = maybe_bypass_block(root, ctx, config, session_id, stderr)? {
        return Ok(AdvisoryOutcome::Block(reason));
    }

    // (PreCompact-only anchor/record parts never run here — that event
    // delegates to Node before this function.)

    if let Some(msg) = maybe_capture_queue_nudge(root)? {
        parts.push(msg);
    }
    if let Some(msg) = maybe_capture_nudge(root, config, stderr)? {
        parts.push(msg);
    }

    let state = read_state_record(root)?;
    let pipeline = resolve_pipeline(root, ctx, session_id, stderr)?;
    let phase_val = match &pipeline {
        Pipeline::Ok { record, .. } => record.phase.clone(),
        Pipeline::Refused => state.phase.clone(),
    };
    let phase = if js_truthy(&phase_val) { phase_val } else { Value::String("idle".into()) };

    if phase == Value::String("idle".into()) || phase == Value::String("compounding-complete".into()) {
        if let Some(msg) = maybe_decision_nudge(root)? {
            parts.push(msg);
        }
    } else if !read_handoff_truthy(root)? {
        // CUTOVER: cells.mjs / reservations.mjs presence gates stood here.
        let claimed = list_claimed_cells(root)?;
        let active = list_active_reservations(root, ctx);

        let mut lines = vec![format!(
            "bee session-close warning: session is ending mid-phase (phase: {}) \
with no .bee/HANDOFF.json. You are about to leave the hive door open.",
            js_to_string(&phase)
        )];
        if !claimed.is_empty() {
            let rendered = claimed
                .iter()
                .map(|cell| {
                    let id = js_to_string(cell.get("id").unwrap_or(&Value::Null));
                    let worker = cell
                        .get("trace")
                        .filter(|t| js_truthy(t))
                        .and_then(|t| t.get("worker"))
                        .filter(|w| js_truthy(w));
                    match worker {
                        Some(w) => format!("{id} ({})", js_to_string(w)),
                        None => id,
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("Claimed-but-uncapped cells: {rendered}."));
        }
        if !active.is_empty() {
            let rendered = active
                .iter()
                .map(|r| {
                    let base = format!("{} -> {}", js_to_string(&r.agent), r.path);
                    match &r.cell {
                        Some(c) if js_truthy(c) => format!("{base} (cell {})", js_to_string(c)),
                        _ => base,
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            lines.push(format!("Active reservations: {rendered}."));
        }
        lines.push(
            "Either finish and cap the work, or write .bee/HANDOFF.json and release \
reservations so the next session can resume cleanly."
                .to_string(),
        );
        parts.push(lines.join("\n"));
    }
    Ok(AdvisoryOutcome::Done)
}

// ─── deferred corrupt-JSON warnings ────────────────────────────────────────
// This hook buffers every byte it emits so a delegating run stays silent, so
// the readJson warning cannot be printed at read time. It is queued here and
// written by flush(), ahead of the advisory's own stderr.

thread_local! {
    static CORRUPT_JSON_WARNINGS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Queues the sentence crate::fsutil::warn_corrupt_json would have printed.
fn queue_corrupt_json_warning(file: &Path) {
    let reason = {
        // Same derivation as fsutil.rs's private corrupt_json_reason.
        match std::fs::read(file) {
            Err(_) => "the file could not be read".to_string(),
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
                match serde_json::from_str::<Value>(text) {
                    Ok(_) => "invalid JSON".to_string(), // raced: it parses now
                    Err(e) if e.line() > 0 => {
                        format!("invalid JSON at line {} column {}", e.line(), e.column())
                    }
                    Err(_) => "invalid JSON".to_string(),
                }
            }
        }
    };
    CORRUPT_JSON_WARNINGS.with(|q| {
        q.borrow_mut().push(format!(
            "bee: could not parse JSON at {} — {reason}. Using fallback; fix the file.\n",
            file.display()
        ))
    });
}

/// read_json plus the queued warning: Corrupt collapses onto Missing, which is
/// what readJson's `null` fallback meant to every caller in this hook.
fn read_json_failopen(file: &Path) -> ReadJson {
    match read_json(file) {
        ReadJson::Corrupt => {
            queue_corrupt_json_warning(file);
            ReadJson::Missing
        }
        other => other,
    }
}

fn take_corrupt_json_warnings() -> String {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().drain(..).collect::<Vec<_>>().join(""))
}

fn clear_corrupt_json_warnings() {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().clear());
}

// ─── pre-flight (the two remaining delegate classes) ───────────────────────

/// Returns the merged tracked+overlay config (read_config_raw warns and reads
/// a corrupt file as absent, so its Err arm is unreachable) after screening the
/// one non-V8 delegate this hook has left: an inject cache that PARSES to a
/// non-object, whose spread/assignment is JS exotica.
fn preflight(root: &Path) -> Result<Map<String, Value>, ()> {
    let bee = root.join(".bee");
    let config = read_config_raw(root).map_err(|_| ())?;
    for cache in [bee.join("cache").join("inject-cache.json"), bee.join(".inject-cache.json")] {
        // Deliberately plain read_json: a corrupt cache is NOT screened here
        // (read_inject_cache warns for it), so this probe never double-warns.
        if let ReadJson::Parsed(v) = read_json(&cache) {
            if !matches!(v, Value::Object(_) | Value::Null | Value::Bool(false)) {
                return Err(());
            }
        }
    }
    Ok(config)
}

// ─── shared JS helpers ─────────────────────────────────────────────────────

fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn get_session_id(payload: &Map<String, Value>) -> Option<String> {
    payload
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// claims.mjs requireId / state.mjs requireLaneFeature shape rule.
fn is_plain_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Date.parse for the ISO shapes bee writes (toISOString / date-only).
fn js_date_parse(text: &str) -> Option<f64> {
    let t = text.trim();
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(t) {
        return Some(dt.timestamp_millis() as f64);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0)?;
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    None
}

fn js_date_parse_value(v: &Value) -> Option<f64> {
    js_date_parse(&js_to_string(v))
}

/// new Date(ms).toISOString() — millisecond ISO, Z suffix. Err on invalid.
fn ms_to_iso(ms: f64) -> Result<String, ()> {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return Err(());
    }
    let dt = chrono::DateTime::from_timestamp_millis(ms.trunc() as i64).ok_or(())?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

fn js_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                0.0
            } else {
                t.parse::<f64>().unwrap_or(f64::NAN)
            }
        }
        _ => f64::NAN,
    }
}

/// readJsonl — skip corrupt lines, never fail.
fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    text.split(['\n'])
        .map(|l| l.trim_end_matches('\r').trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

/// A JS-Set primitive key (SameValueZero over the truthy primitives bee logs).
fn primitive_key(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(format!("s:{s}")),
        Value::Number(n) => Some(format!("n:{}", jsjson::js_f64_to_string(n.as_f64()?))),
        Value::Bool(b) => Some(format!("b:{b}")),
        _ => None,
    }
}

// ─── state / pipeline reads ────────────────────────────────────────────────

struct Record {
    phase: Value,
    mode: Value,
    gates: Map<String, Value>,
}

/// state.mjs readState, reduced to the phase/mode/gates slice this hook uses.
///
/// Inlined rather than delegated to `crate::state::read_state_brief` because
/// that reader PRINTS its corrupt-JSON warning immediately, and this hook must
/// QUEUE its warnings — a delegating run (PreCompact) has to emit zero bytes
/// first. Both readers agree on the outcome: readJson's `null` fallback →
/// defaultState(). The exotic-gates arm below still delegates here, where
/// state.rs now spreads it natively; this hook keeps the delegate because it
/// still has a live delegate path to take.
fn read_state_record(root: &Path) -> Result<Record, Flow> {
    let defaults = || Record {
        phase: Value::String("idle".into()),
        mode: Value::Null,
        gates: default_gates(),
    };
    let file = root.join(".bee").join("state.json");
    let ReadJson::Parsed(Value::Object(state)) = read_json_failopen(&file) else {
        // Missing, corrupt (warned) or a non-object parse — all defaultState().
        return Ok(defaults());
    };
    // approved_gates: { ...defaults, ...(state.approved_gates || {}) }.
    let gates = match state.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut merged = default_gates();
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
            merged
        }
        // A truthy non-object spreads exotic key sets in JS — delegate.
        Some(_) => return Err(Flow::Delegate),
    };
    let mut phase = state.get("phase").cloned().unwrap_or(Value::String("idle".into()));
    if phase == Value::String("validating".into()) {
        phase = Value::String("planning".into()); // coerceLegacyPhase (D13)
    }
    Ok(Record { phase, mode: state.get("mode").cloned().unwrap_or(Value::Null), gates })
}

fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

/// state.mjs laneRecordFrom — null when not a lane record for this feature.
fn lane_record_from(feature: &str, parsed: &Value) -> Result<Option<Record>, Flow> {
    let Value::Object(obj) = parsed else { return Ok(None) };
    if obj.get("feature").and_then(Value::as_str) != Some(feature) {
        return Ok(None);
    }
    let gates = match obj.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut merged = default_gates();
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
            merged
        }
        Some(_) => return Err(Flow::Delegate), // JS spread exotica
    };
    let mut phase = obj.get("phase").cloned().unwrap_or(Value::String("idle".into()));
    if phase == Value::String("validating".into()) {
        phase = Value::String("planning".into()); // coerceLegacyPhase (D13)
    }
    Ok(Some(Record {
        phase,
        mode: obj.get("mode").cloned().unwrap_or(Value::Null),
        gates,
    }))
}

enum Pipeline {
    Ok { record: Record },
    /// LANE_INVALID / LANE_MISSING / LANE_CORRUPT — callers on the Stop path
    /// only ever branch on ok-ness and fall back to readState.
    Refused,
}

/// state.mjs controlRootFor: mainRoot for a linked worktree, root otherwise;
/// linked-invalid THROWS (WorktreeLinkInvalidError) in the Node lib.
fn control_root(root: &Path, ctx: &HookContext) -> PathBuf {
    if ctx.worktree_resolution == "linked-valid" {
        ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf())
    } else {
        root.to_path_buf()
    }
}

fn resolve_pipeline(
    root: &Path,
    ctx: &HookContext,
    session_id: Option<&str>,
    stderr: &mut String,
) -> Result<Pipeline, Flow> {
    let Some(sid) = session_id else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    if ctx.worktree_resolution == "linked-invalid" {
        // state.mjs resolveRootsCore throws WorktreeLinkInvalidError here.
        return Err(Flow::Crash(format!(
            "WorktreeLinkInvalidError: linked worktree link is invalid ({})",
            root.join(".git").display()
        )));
    }
    let ctl = control_root(root, ctx);
    // readSession (fail-open id validation).
    if !is_plain_id(sid) {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    }
    let session_file = ctl.join(".bee").join("sessions").join(format!("{sid}.json"));
    // Corrupt → warned, then reads as "no session" (readJson's null fallback
    // through readSession's `!session || session.id !== id` guard).
    let session = match read_json_failopen(&session_file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Parsed(Value::Object(m)) => {
            if m.get("id").and_then(Value::as_str) == Some(sid) {
                Some(m)
            } else {
                None
            }
        }
        ReadJson::Parsed(_) => None,
    };
    let Some(session) = session else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    let bound = session
        .get("lane")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let Some(bound) = bound else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    if !is_plain_id(&bound) {
        return Ok(Pipeline::Refused); // LANE_INVALID
    }
    let lane_file = ctl.join(".bee").join("lanes").join(format!("{bound}.json"));
    if !lane_file.exists() {
        return Ok(Pipeline::Refused); // LANE_MISSING
    }
    // A corrupt lane gets BOTH of Node's lines: readJson's (queued above by
    // read_json_failopen) and then readLane's own, via the None arm below —
    // laneRecordFrom(null) is null just like a shape-wrong record.
    match read_json_failopen(&lane_file) {
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Missing => {
            // Vanished mid-read, or corrupt: laneRecordFrom(null) → readLane's
            // warn → LANE_CORRUPT. (A genuinely absent file was already
            // short-circuited by the exists() check above.)
            let rel = format!(".bee{s}lanes{s}{bound}.json", s = std::path::MAIN_SEPARATOR);
            stderr.push_str(&format!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations \
through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \
\"git checkout -- {rel}\").\n"
            ));
            Ok(Pipeline::Refused)
        }
        ReadJson::Parsed(parsed) => match lane_record_from(&bound, &parsed)? {
            Some(record) => Ok(Pipeline::Ok { record }),
            None => {
                // readLane's deterministic console.warn (corrupt lane shape).
                let rel = format!(".bee{s}lanes{s}{bound}.json", s = std::path::MAIN_SEPARATOR);
                stderr.push_str(&format!(
                    "readLane: skipping corrupt lane record \"{rel}\" for display — mutations \
through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \
\"git checkout -- {rel}\").\n"
                ));
                Ok(Pipeline::Refused) // LANE_CORRUPT
            }
        },
    }
}

/// state.mjs readHandoff, truthiness only. A corrupt HANDOFF.json warns and
/// reads as absent — so the "hive door open" mid-phase warning still fires,
/// which is the conservative half of readJson's fail-open here.
fn read_handoff_truthy(root: &Path) -> Result<bool, Flow> {
    match read_json_failopen(&root.join(".bee").join("HANDOFF.json")) {
        ReadJson::Missing => Ok(false),
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Parsed(v) => Ok(js_truthy(&v)),
    }
}

// ─── inject.mjs dedup cache ────────────────────────────────────────────────

fn inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join("cache").join("inject-cache.json")
}

fn legacy_inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join(".inject-cache.json")
}

/// readJson(new, null) || readJson(legacy, {}) || {} — object caches only. A
/// corrupt cache warns and reads as absent, so the chain falls through to the
/// legacy file and then to `{}` (the nudge simply re-injects once). Non-object
/// parses still delegate; they are screened in preflight and re-checked here.
fn read_inject_cache(root: &Path) -> Result<Map<String, Value>, Flow> {
    for (file, missing_falls_through) in
        [(inject_cache_path(root), true), (legacy_inject_cache_path(root), false)]
    {
        match read_json_failopen(&file) {
            ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
            ReadJson::Missing => {
                if !missing_falls_through {
                    return Ok(Map::new());
                }
            }
            ReadJson::Parsed(Value::Object(m)) => return Ok(m),
            // A parsed falsy value (null/false) falls through like "missing";
            // any other non-object shape is JS exotica — delegate.
            ReadJson::Parsed(Value::Null) | ReadJson::Parsed(Value::Bool(false)) => {
                if !missing_falls_through {
                    return Ok(Map::new());
                }
            }
            ReadJson::Parsed(_) => return Err(Flow::Delegate),
        }
    }
    Ok(Map::new())
}

fn should_inject(root: &Path, key: &str, hash: &str) -> Result<bool, Flow> {
    let cache = read_inject_cache(root)?;
    let Some(entry) = cache.get(key) else { return Ok(true) };
    if !js_truthy(entry) {
        return Ok(true);
    }
    if entry.get("hash").and_then(Value::as_str) != Some(hash) {
        return Ok(true);
    }
    let at = entry.get("at").cloned().unwrap_or(Value::Null);
    let Some(last_ms) = js_date_parse_value(&at) else { return Ok(true) };
    Ok(now_ms() - last_ms > INJECT_INTERVAL_MS)
}

fn mark_injected(root: &Path, key: &str, hash: &str) -> Result<(), Flow> {
    let mut cache = read_inject_cache(root)?;
    let mut entry = Map::new();
    entry.insert("hash".into(), Value::String(hash.to_string()));
    entry.insert("at".into(), Value::String(now_iso()));
    cache.insert(key.to_string(), Value::Object(entry));
    let _ = crate::fsutil::write_json_atomic(&inject_cache_path(root), &Value::Object(cache));
    crate::fsutil::remove_file_if_exists(&legacy_inject_cache_path(root));
    Ok(())
}

// ─── decisions.mjs newest active decision ──────────────────────────────────

/// activeDecisions(root, {recent:1})[0] reduced to (id, date). The tag
/// overlay (dp-5) never touches id/date, so it is irrelevant here.
fn newest_active_decision(root: &Path) -> Option<(Value, Value)> {
    let events = read_jsonl(&root.join(".bee").join("decisions.jsonl"));
    let mut superseded: HashSet<String> = HashSet::new();
    let mut redacted: HashSet<String> = HashSet::new();
    for event in &events {
        if event.get("type").and_then(Value::as_str) == Some("supersede") {
            if let Some(target) = event.get("supersedes").filter(|v| js_truthy(v)) {
                if let Some(k) = primitive_key(target) {
                    superseded.insert(k);
                }
            }
        }
        if event.get("type").and_then(Value::as_str) == Some("redact") {
            if let Some(target) = event.get("redacts").filter(|v| js_truthy(v)) {
                if let Some(k) = primitive_key(target) {
                    redacted.insert(k);
                }
            }
        }
    }
    let mut newest: Option<&Value> = None;
    for event in &events {
        let ty = event.get("type").and_then(Value::as_str);
        if !matches!(ty, Some("decide") | Some("supersede")) {
            continue;
        }
        let id_key = event.get("id").and_then(primitive_key);
        if let Some(k) = &id_key {
            if superseded.contains(k) || redacted.contains(k) {
                continue;
            }
        }
        newest = Some(event);
    }
    newest.map(|e| {
        (e.get("id").cloned().unwrap_or(Value::Null), e.get("date").cloned().unwrap_or(Value::Null))
    })
}

// ─── maybeCaptureQueueNudge (decision 0017) ────────────────────────────────

fn pending_capture_stub_ids(root: &Path) -> Vec<String> {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: HashSet<String> = HashSet::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !event.is_object() {
            continue;
        }
        let kind = event.get("kind").and_then(Value::as_str);
        let id_truthy = event.get("id").map(js_truthy).unwrap_or(false);
        if kind == Some("flush") && id_truthy {
            if let Some(k) = event.get("id").and_then(primitive_key) {
                flushed.insert(k);
            }
        } else if kind == Some("stub") && id_truthy {
            stubs.push(event);
        }
    }
    stubs
        .into_iter()
        .filter(|s| {
            s.get("id").and_then(primitive_key).map(|k| !flushed.contains(&k)).unwrap_or(true)
        })
        .map(|s| js_to_string(s.get("id").unwrap_or(&Value::Null)))
        .collect()
}

fn maybe_capture_queue_nudge(
    root: &Path,
) -> Result<Option<String>, Flow> {
    let pending = pending_capture_stub_ids(root);
    if pending.is_empty() {
        return Ok(None);
    }
    let mut ids = pending.clone();
    ids.sort(); // JS default sort over string ids
    let hash = ids.join("|");
    if !should_inject(root, "capture-queue-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "capture-queue-nudge", &hash)?;
    Ok(Some(format!(
        "bee capture queue (decision 0017): {} settlement stub(s) are queued and \
unflushed. Flush them now via bee-capturing (drain oldest-first, merge each into its \
area spec) — or they must survive into the next session's preamble, never be dropped.",
        pending.len()
    )))
}

// ─── maybeCaptureNudge (decision 0003) ─────────────────────────────────────

/// state.mjs resolveProductRoot — warnings replicated byte-for-byte; emitted
/// on EVERY call, exactly as the .mjs re-runs it per call site.
fn resolve_product_root(root: &Path, config: &Map<String, Value>, stderr: &mut String) -> PathBuf {
    let configured = match config.get("product_root") {
        None | Some(Value::Null) => return root.to_path_buf(),
        Some(Value::String(s)) if s.is_empty() => return root.to_path_buf(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            let type_of = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => "object",
            };
            stderr.push_str(&format!(
                "bee: .bee/config.json product_root must be a string path (got {type_of}); \
ignoring it and using the bee root.\n"
            ));
            return root.to_path_buf();
        }
    };
    let candidate = Path::new(&configured);
    let resolved: PathBuf = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        normalize_join(root, candidate)
    };
    let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
    if !is_dir {
        stderr.push_str(&format!(
            "bee: config product_root \"{configured}\" -> \"{}\" is not an existing directory; \
product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix \
.bee/config.json product_root. (GitHub #14)\n",
            resolved.display()
        ));
    }
    resolved
}

/// path.resolve(root, rel) for ordinary relative paths ('.'/'..' collapsed).
fn normalize_join(root: &Path, rel: &Path) -> PathBuf {
    let mut out = root.to_path_buf();
    for comp in rel.components() {
        use std::path::Component;
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The newest .md mtimeMs under dir (flat or recursive). Err = fs error the
/// .mjs would throw (caught by the nudge's own try → null).
fn newest_md(dir: &Path, recursive: bool) -> Result<f64, ()> {
    let mut newest = 0.0f64;
    if !dir.exists() {
        return Ok(newest);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current).map_err(|_| ())?;
        for entry in entries {
            let entry = entry.map_err(|_| ())?;
            let ft = entry.file_type().map_err(|_| ())?;
            if ft.is_dir() {
                if recursive {
                    stack.push(entry.path());
                }
                continue;
            }
            if !ft.is_file() || !entry.file_name().to_string_lossy().ends_with(".md") {
                continue;
            }
            let meta = std::fs::metadata(entry.path()).map_err(|_| ())?;
            let mtime = meta
                .modified()
                .map_err(|_| ())?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0);
            if mtime > newest {
                newest = mtime;
            }
        }
    }
    Ok(newest)
}

fn maybe_capture_nudge(
    root: &Path,
    config: &Map<String, Value>,
    stderr: &mut String,
) -> Result<Option<String>, Flow> {
    // state.mjs is import-gated by the wrapper already; resolveProductRoot may
    // warn (replicated), then the docs trees are checked.
    let product_root = resolve_product_root(root, config, stderr);
    let specs_dir = product_root.join("docs").join("specs");
    let knowledge_dir = product_root.join("docs").join("knowledge");
    if !specs_dir.exists() && !knowledge_dir.exists() {
        return Ok(None);
    }
    let Some((id, date)) = newest_active_decision(root) else { return Ok(None) };
    let decision_ts = if js_truthy(&date) { js_date_parse_value(&date) } else { None };
    let Some(decision_ts) = decision_ts.filter(|t| *t != 0.0) else {
        return Ok(None); // !decisionTs (0 or NaN)
    };
    let Ok(newest_spec) = newest_md(&specs_dir, false).and_then(|a| Ok(a.max(newest_md(&knowledge_dir, true)?)))
    else {
        return Ok(None); // fs throw → catch → null
    };
    if decision_ts <= newest_spec {
        return Ok(None);
    }
    let hash_src = if js_truthy(&id) { id } else { date };
    let hash = js_to_string(&hash_src);
    if !should_inject(root, "capture-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "capture-nudge", &hash)?;
    // knowledge.mjs is imported AFTER the mark in the .mjs — a missing module
    // throws there and the nudge is consumed without being emitted.
    if bundle_mode(root, config, stderr) {
        return Ok(Some(
            "bee capture nudge (decision 0003): the newest decision is more recent than every \
concept in the knowledge bundle (docs/knowledge/) — a settled outcome may exist only \
in the decision log and the chat. Before finishing, invoke bee-capturing capture to \
author it as a concept in the touched area's bundle folder (or confirm no area is affected)."
                .to_string(),
        ));
    }
    Ok(Some(
        "bee capture nudge (decision 0003): the newest decision is more recent than every \
area spec under docs/specs/ — a settled outcome may exist only in the decision log \
and the chat. Before finishing, invoke bee-capturing capture to merge it into the \
touched area's spec (or confirm no spec is affected)."
            .to_string(),
    ))
}

// ─── maybeDecisionNudge (repository-harness lesson) ────────────────────────

fn git_status_porcelain(root: &Path) -> Result<String, ()> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ())?;
    let mut out = child.stdout.take().ok_or(())?;
    let reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = out.read_to_end(&mut buf);
        buf
    });
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed().as_millis() > 3000 {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(()); // execSync timeout → throw → catch → null
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => return Err(()),
        }
    };
    let buf = reader.join().map_err(|_| ())?;
    if !status.success() {
        return Err(()); // execSync non-zero exit → throw
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn nudge_allowed(path: &str) -> bool {
    // /^(\.bee\/|docs\/|plans\/|AGENTS\.md$)/
    path.starts_with(".bee/") || path.starts_with("docs/") || path.starts_with("plans/") || path == "AGENTS.md"
}

fn maybe_decision_nudge(root: &Path) -> Result<Option<String>, Flow> {
    let Ok(out) = git_status_porcelain(root) else { return Ok(None) };
    let mut changed: Vec<String> = out
        .split('\n')
        .map(|line| {
            // line.slice(3).trim().replace(/^"|"$/g, "")
            let sliced: String = line.chars().skip(3).collect();
            let mut s = sliced.trim();
            if let Some(x) = s.strip_prefix('"') {
                s = x;
            }
            if let Some(x) = s.strip_suffix('"') {
                s = x;
            }
            s.to_string()
        })
        .filter(|p| !p.is_empty())
        .filter(|p| !nudge_allowed(p))
        .collect();
    if changed.is_empty() {
        return Ok(None);
    }
    let last_ts = newest_active_decision(root)
        .and_then(|(_, date)| if js_truthy(&date) { js_date_parse_value(&date) } else { Some(0.0) })
        .unwrap_or(0.0);
    if last_ts != 0.0 && !last_ts.is_nan() && now_ms() - last_ts < DECISION_RECENT_MS {
        return Ok(None);
    }
    let count = changed.len();
    changed.sort();
    let hash = changed.join("|");
    if !should_inject(root, "decision-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "decision-nudge", &hash)?;
    Ok(Some(format!(
        "bee decision review: {count} source file(s) changed with no bee flow active \
and no recent decision logged. Before finishing, ask the user: is there a durable \
decision or convention here worth recording? If yes: bee decisions log \
--decision \"...\" --rationale \"...\" (or a dated learning in docs/history/learnings/). \
If not, carry on."
    )))
}

// ─── maybeBypassBlock (GitHub #18) ─────────────────────────────────────────

fn level_covers_gate(level: &str, mode: &Value) -> bool {
    if level == "total" || level == "full" {
        return true;
    }
    if level == "normal" {
        return matches!(mode, Value::String(s) if matches!(s.as_str(), "tiny" | "small" | "standard"));
    }
    false
}

fn maybe_bypass_block(
    root: &Path,
    ctx: &HookContext,
    config: &Map<String, Value>,
    session_id: Option<&str>,
    stderr: &mut String,
) -> Result<Option<String>, Flow> {
    if ctx.event != "Stop" {
        return Ok(None);
    }
    let level = bypass_level(config);
    if level == "off" {
        return Ok(None);
    }
    let pipeline = resolve_pipeline(root, ctx, session_id, stderr)?;
    let record = match pipeline {
        Pipeline::Ok { record } => record,
        Pipeline::Refused => read_state_record(root)?,
    };
    let phase_val = if js_truthy(&record.phase) { record.phase.clone() } else { Value::String("idle".into()) };
    // PHASE_GATE property lookup coerces the key to a string.
    let phase = js_to_string(&phase_val);
    if phase != "planning" {
        return Ok(None);
    }
    let gate = "execution";
    let mode = if js_truthy(&record.mode) { record.mode.clone() } else { Value::Null };
    if !level_covers_gate(level, &mode) {
        return Ok(None);
    }
    if record.gates.get(gate) == Some(&Value::Bool(true)) {
        return Ok(None); // gate already passed — nothing to force
    }
    let key = "bypass-stop-net";
    let hash = format!("{}:{phase}:{gate}:{level}", session_id.unwrap_or("nosession"));
    if !should_inject(root, key, &hash)? {
        return Ok(None);
    }
    mark_injected(root, key, &hash)?;

    let gate_no = "3"; // gate is never "shape" (PHASE_GATE maps planning→execution)
    let consult_sentence = if mode == Value::String("high-risk".into()) {
        "High-risk execution requires a live advisor consult first: resolve the advisor from \
config (models.<runtime>.advisor), run it read-only with the evidence bundle on stdin, then \
record it via bee state advisor-ref record --advisor \"<identity>\" \
--digest-file <path> (the gate throws without a non-stale advisor_ref, per AO3/AO13) — do \
this BEFORE setting the gate. "
    } else {
        ""
    };
    Ok(Some(format!(
        "⚡ GATE BYPASS ({level}): you are stopping mid-{phase} with Gate {gate_no} \
({gate}) still pending, but bypass level \"{level}\" requires auto-approval at \
this lane — do NOT ask the human. {consult_sentence}Set the gate yourself now: \
bee state gate --name {gate} --approved true ; log a one-line \
audit decision (bee decisions log --decision \"auto-approved Gate \
{gate_no} (bypass): <choice>\" --rationale \"<why>\"); post the short \"⚡ auto-approved \
Gate {gate_no} (bypass)\" line; then CONTINUE to the next phase. Do not re-emit the \
gate question. (If you genuinely need information only the human holds — not a \
rubber-stamp — ask that specific question instead; this net blocks once, then steps \
aside.)"
    )))
}

// ─── cells.mjs listCells({status:'claimed'}) ───────────────────────────────

fn list_claimed_cells(root: &Path) -> Result<Vec<Map<String, Value>>, Flow> {
    let dir = root.join(".bee").join("cells");
    let mut cells: Vec<Map<String, Value>> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                continue; // `archive` (or any dir) is never a cell
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            // A corrupt cell warns and is skipped (readJson null → `!cell`).
            match read_json_failopen(&entry.path()) {
                ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
                ReadJson::Missing => continue,
                ReadJson::Parsed(Value::Object(cell)) => {
                    if cell.get("status") == Some(&Value::String("claimed".into())) {
                        cells.push(cell);
                    }
                }
                // Arrays pass the .mjs's typeof-object filter but can never
                // carry status === 'claimed'; everything else is skipped.
                ReadJson::Parsed(_) => continue,
            }
        }
    }
    cells.sort_by(|a, b| {
        cmp_locale_numeric(
            &js_to_string(a.get("id").unwrap_or(&Value::Null)),
            &js_to_string(b.get("id").unwrap_or(&Value::Null)),
        )
    });
    Ok(cells)
}

/// String#localeCompare(x, 'en', {numeric: true}) approximation: digit runs
/// compare numerically, letters case-insensitively (lowercase-first tiebreak),
/// exact for the lowercase slug ids bee generates.
fn cmp_locale_numeric(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ac = a.chars().peekable();
    let mut bc = b.chars().peekable();
    // Case is an ICU tertiary difference: recorded at the first divergence but
    // applied only when everything primary-level compares equal.
    let mut case_tiebreak = Ordering::Equal;
    loop {
        match (ac.peek().copied(), bc.peek().copied()) {
            (None, None) => break,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let mut xs = String::new();
                    let mut ys = String::new();
                    while ac.peek().is_some_and(|c| c.is_ascii_digit()) {
                        xs.push(ac.next().unwrap());
                    }
                    while bc.peek().is_some_and(|c| c.is_ascii_digit()) {
                        ys.push(bc.next().unwrap());
                    }
                    let xt = xs.trim_start_matches('0');
                    let yt = ys.trim_start_matches('0');
                    let ord = xt.len().cmp(&yt.len()).then_with(|| xt.cmp(yt));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                } else {
                    let xl = x.to_lowercase().next().unwrap_or(x);
                    let yl = y.to_lowercase().next().unwrap_or(y);
                    if xl != yl {
                        return xl.cmp(&yl);
                    }
                    if x != y && case_tiebreak == Ordering::Equal {
                        // lowercase before uppercase (en tertiary)
                        case_tiebreak = x.is_uppercase().cmp(&y.is_uppercase());
                    }
                    ac.next();
                    bc.next();
                }
            }
        }
    }
    case_tiebreak
}

// ─── reservations.mjs listReservations({activeOnly:true}) ──────────────────

struct ReservationRow {
    agent: Value,
    path: String,
    cell: Option<Value>,
}

fn list_active_reservations(root: &Path, ctx: &HookContext) -> Vec<ReservationRow> {
    // reservations.mjs's own controlRootFor: fail-open findMainRoot ?? root.
    let ctl = if ctx.worktree_resolution == "linked-valid" {
        ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf())
    } else {
        root.to_path_buf()
    };
    let leases_root = ctl.join(".bee").join("runtime").join("leases");
    let now = now_ms();
    let mut rows = Vec::new();
    for kind_dir in ["cells", "paths"] {
        let dir = leases_root.join(kind_dir);
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_file || !name.ends_with(".json") {
                continue;
            }
            // readLeaseSafe: JSON.parse in a try — silent on corruption.
            let Ok(text) = std::fs::read(entry.path()) else { continue };
            let Ok(record) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&text)) else {
                continue;
            };
            // isPathLease: string resource with the 'path:' prefix.
            let Some(resource) = record.get("resource").and_then(Value::as_str) else { continue };
            let Some(path_part) = resource.strip_prefix("path:") else { continue };
            // activeOnly: isLeaseRecordExpired on the raw expires_at.
            let expired = match record.get("expires_at") {
                None | Some(Value::Null) => false,
                Some(v) => match js_date_parse_value(v) {
                    Some(ms) => ms <= now,
                    None => false,
                },
            };
            if expired {
                continue;
            }
            let workspace_id = record.get("workspace_id").cloned().unwrap_or(Value::Null);
            let agent = match &workspace_id {
                Value::String(s) if s.starts_with("agent:") => {
                    Value::String(s["agent:".len()..].to_string())
                }
                other => other.clone(),
            };
            rows.push(ReservationRow {
                agent,
                path: path_part.to_string(),
                cell: record.get("workflow_id").cloned(),
            });
        }
    }
    rows
}

// ─── knowledge.mjs bundleMode (parseFrontmatter subset) ────────────────────

fn bundle_mode(root: &Path, config: &Map<String, Value>, stderr: &mut String) -> bool {
    // bundleDir(root) = resolveProductRoot(root)/docs/knowledge — the .mjs
    // re-runs resolveProductRoot here, re-emitting any warning.
    let dir = resolve_product_root(root, config, stderr).join("docs").join("knowledge");
    let Ok(meta) = std::fs::metadata(&dir) else { return false };
    if !meta.is_dir() {
        return false;
    }
    for rel in list_bundle_markdown(&dir) {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if base == "index.md" || base == "log.md" {
            continue;
        }
        let file = dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        let Ok(bytes) = std::fs::read(&file) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        if frontmatter_has_type(&text) {
            return true;
        }
    }
    false
}

fn list_bundle_markdown(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(abs) else { return };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue; // never follow (D23)
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&entry.path(), &child_rel, out);
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
    }
    if dir.exists() {
        walk(dir, "", &mut out);
    }
    out.sort();
    out
}

/// parseFrontmatter reduced to bundleMode's one question: does this file
/// parse ok, carry frontmatter, and hold a non-empty string `type`?
fn frontmatter_has_type(text: &str) -> bool {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return false; // present: false
    };
    // Locate the closing --- line.
    let mut cursor = open_len;
    let mut inner_end: Option<usize> = None;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|i| cursor + i);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = Some(cursor);
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(inner_end) = inner_end else { return false }; // unclosed
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop(); // split(...).slice(0, -1)
        v
    };

    let mut root_keys: HashSet<String> = HashSet::new();
    let mut bee_keys: HashSet<String> = HashSet::new();
    let mut in_bee_map = false;
    let mut type_is_concept = false;
    for raw_line in inner_lines {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() || line.contains('\t') {
            return false;
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map || inner.starts_with(' ') {
                return false;
            }
            if !parse_kv_line(inner, &mut bee_keys, &mut None) {
                return false;
            }
            continue;
        }
        if line.starts_with(' ') {
            return false;
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a bee: map header.
        if let Some(key) = line.strip_suffix(':') {
            if !key.is_empty() && !key.contains(':') && !key.chars().any(is_js_ws) {
                if !fm_key_ok(key) || key != "bee" || root_keys.contains("bee") {
                    return false;
                }
                root_keys.insert("bee".into());
                bee_keys.clear();
                in_bee_map = true;
                continue;
            }
        }
        let mut type_slot = Some(&mut type_is_concept);
        if !parse_kv_line(line, &mut root_keys, &mut type_slot) {
            return false;
        }
    }
    type_is_concept
}

fn fm_key_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// JS regex \s (no /u): the class used by the frontmatter header rule.
fn is_js_ws(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r' | ' ' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}'
        | '\u{3000}' | '\u{feff}')
}

/// parseKeyValueLine — returns false on any typed parse failure. When
/// `type_slot` is set (root level) and the key is "type", records whether the
/// value is a non-empty string.
fn parse_kv_line(line: &str, keys: &mut HashSet<String>, type_slot: &mut Option<&mut bool>) -> bool {
    let Some(sep) = line.find(": ") else { return false };
    let key = &line[..sep];
    if !fm_key_ok(key) || keys.contains(key) {
        return false;
    }
    keys.insert(key.to_string());
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return false;
    }
    let value: Option<String> = if raw.starts_with('[') {
        if !parse_flow_list(raw) {
            return false;
        }
        None
    } else {
        match parse_scalar_token(raw) {
            ScalarParse::Fail => return false,
            ScalarParse::Bool => None,
            ScalarParse::Str(s) => Some(s),
        }
    };
    if key == "type" {
        if let Some(flag) = type_slot.as_deref_mut() {
            *flag = value.map(|s| !s.is_empty()).unwrap_or(false);
        }
    }
    true
}

enum ScalarParse {
    Fail,
    Bool,
    Str(String),
}

fn parse_scalar_token(raw: &str) -> ScalarParse {
    if raw == "true" || raw == "false" {
        return ScalarParse::Bool;
    }
    if raw.starts_with('"') {
        return match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => ScalarParse::Str(s),
            _ => ScalarParse::Fail,
        };
    }
    if raw.starts_with('\'') {
        return ScalarParse::Fail;
    }
    if raw.starts_with(['&', '*', '!', '|', '>', '%', '@', '`', '{', '}']) {
        return ScalarParse::Fail;
    }
    ScalarParse::Str(raw.to_string())
}

fn parse_flow_list(raw: &str) -> bool {
    if !raw.ends_with(']') {
        return false;
    }
    let inner = raw[1..raw.len() - 1].trim_matches(is_js_ws);
    if inner.is_empty() {
        return true;
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch == ',' {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if in_quote {
        return false;
    }
    segments.push(current);
    for segment in segments {
        let token = segment.trim_matches(is_js_ws);
        if token.is_empty() || matches!(parse_scalar_token(token), ScalarParse::Fail) {
            return false;
        }
    }
    true
}

// ─── perf.mjs — the maybePerfRefresh pipeline ──────────────────────────────

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn node_homedir() -> String {
    if cfg!(windows) {
        env_nonempty("USERPROFILE").unwrap_or_default()
    } else {
        env_nonempty("HOME").unwrap_or_default()
    }
}

fn claude_projects_root() -> PathBuf {
    let base = env_nonempty("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(node_homedir()).join(".claude"));
    base.join("projects")
}

fn global_perf_dir() -> PathBuf {
    if let Some(dir) = env_nonempty("BEEHIVE_PERF_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = env_nonempty("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("beehive");
    }
    PathBuf::from(node_homedir()).join(".config").join("beehive")
}

fn global_perf_log_path() -> PathBuf {
    global_perf_dir().join("performance.jsonl")
}

/// encodeProjectDir: replace [\\/.:] with '-'.
///
/// Divergence from perf.mjs, taken deliberately AT CUTOVER (plans/rust-port.md,
/// "the two filed win32 defects"): Node's `/[\\/.]/g` kept the Windows drive
/// colon, spelling a transcript directory (`D:-projects-…`) that cannot exist
/// on NTFS. Mapping ':' as well gives `D--projects-…`, the spelling Claude Code
/// itself writes, so the perf rollup can finally find a transcript on win32.
fn encode_project_dir(project_path: &str) -> String {
    project_path
        .chars()
        .map(|c| if matches!(c, '\\' | '/' | '.' | ':') { '-' } else { c })
        .collect()
}

fn strip_jsonl_suffix(file: &Path) -> PathBuf {
    let s = file.to_string_lossy();
    PathBuf::from(s.strip_suffix(".jsonl").map(String::from).unwrap_or_else(|| s.into_owned()))
}

fn resolve_transcript_for(root: &Path, session_id: Option<&str>) -> Option<PathBuf> {
    let projects_root = claude_projects_root();
    let dir = projects_root.join(encode_project_dir(&root.to_string_lossy()));
    if let Some(sid) = session_id {
        let file = dir.join(format!("{sid}.jsonl"));
        return file.exists().then_some(file);
    }
    // Newest-mtime top-level *.jsonl (the live session).
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut best: Option<PathBuf> = None;
    let mut best_mtime = f64::NEG_INFINITY;
    for entry in entries.flatten() {
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_file || !name.ends_with(".jsonl") {
            continue;
        }
        let Ok(meta) = std::fs::metadata(entry.path()) else { continue };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(f64::NEG_INFINITY);
        if mtime > best_mtime {
            best_mtime = mtime;
            best = Some(entry.path());
        }
    }
    best
}

#[derive(Clone, Default)]
struct ModelAcc {
    input: f64,
    output: f64,
    cache_write: f64,
    cache_read: f64,
    new_t: f64,
    cached: f64,
    total: f64,
}

impl ModelAcc {
    fn finalize(&mut self) {
        self.new_t = self.input + self.output + self.cache_write;
        self.cached = self.cache_read;
        self.total = self.new_t + self.cached;
    }
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        let n = |v: f64| serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null);
        m.insert("input".into(), n(self.input));
        m.insert("output".into(), n(self.output));
        m.insert("cache_write".into(), n(self.cache_write));
        m.insert("cache_read".into(), n(self.cache_read));
        m.insert("new".into(), n(self.new_t));
        m.insert("cached".into(), n(self.cached));
        m.insert("total".into(), n(self.total));
        Value::Object(m)
    }
}

/// Insertion-ordered model accumulator map (JS object semantics).
#[derive(Default)]
struct ModelMap(Vec<(String, ModelAcc)>);

impl ModelMap {
    fn entry(&mut self, key: &str) -> &mut ModelAcc {
        if let Some(pos) = self.0.iter().position(|(k, _)| k == key) {
            return &mut self.0[pos].1;
        }
        self.0.push((key.to_string(), ModelAcc::default()));
        let last = self.0.len() - 1;
        &mut self.0[last].1
    }
    fn finalize(&mut self) {
        for (_, acc) in &mut self.0 {
            acc.finalize();
        }
    }
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        for (k, acc) in &self.0 {
            m.insert(k.clone(), acc.to_value());
        }
        Value::Object(m)
    }
}

fn num_field(v: &Value, key: &str) -> f64 {
    match v.get(key) {
        Some(Value::Number(n)) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.is_finite() {
                f
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

struct UsageAgg {
    models: ModelMap,
    /// Kept for shape parity with aggregateUsage; only computeMetrics (not
    /// ported — the hook never calls it) reads the totals.
    #[allow(dead_code)]
    totals: ModelAcc,
}

fn aggregate_usage(events: &[Value]) -> UsageAgg {
    struct Rec {
        model: String,
        input: f64,
        output: f64,
        cache_write: f64,
        cache_read: f64,
    }
    let mut by_req: Vec<(String, Rec)> = Vec::new();
    let mut no_req: Vec<Rec> = Vec::new();
    let mut obj_counter = 0usize;
    for ev in events {
        if ev.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let msg = ev.get("message").filter(|m| js_truthy(m)).cloned().unwrap_or(Value::Object(Map::new()));
        let Some(model_v) = msg.get("model").filter(|m| js_truthy(m)) else { continue };
        let model = js_to_string(model_v);
        if model == "<synthetic>" {
            continue;
        }
        let usage = msg.get("usage").filter(|u| js_truthy(u)).cloned().unwrap_or(Value::Object(Map::new()));
        let rec = Rec {
            model,
            input: num_field(&usage, "input_tokens"),
            output: num_field(&usage, "output_tokens"),
            cache_write: num_field(&usage, "cache_creation_input_tokens"),
            cache_read: num_field(&usage, "cache_read_input_tokens"),
        };
        let rid = ev.get("requestId").filter(|r| js_truthy(r));
        if let Some(rid) = rid {
            // Map key: primitives by value; objects are always distinct keys.
            let key = primitive_key(rid).unwrap_or_else(|| {
                obj_counter += 1;
                format!("o:{obj_counter}")
            });
            if let Some(pos) = by_req.iter().position(|(k, _)| *k == key) {
                if rec.output > by_req[pos].1.output {
                    by_req[pos].1 = rec;
                }
            } else {
                by_req.push((key, rec));
            }
        } else {
            no_req.push(rec);
        }
    }
    let mut models = ModelMap::default();
    let mut totals = ModelAcc::default();
    for r in by_req.into_iter().map(|(_, r)| r).chain(no_req) {
        let m = models.entry(&r.model);
        m.input += r.input;
        m.output += r.output;
        m.cache_write += r.cache_write;
        m.cache_read += r.cache_read;
        totals.input += r.input;
        totals.output += r.output;
        totals.cache_write += r.cache_write;
        totals.cache_read += r.cache_read;
    }
    models.finalize();
    totals.finalize();
    UsageAgg { models, totals }
}

fn event_ms(ev: &Value) -> Option<f64> {
    ev.get("timestamp").and_then(Value::as_str).and_then(js_date_parse)
}

fn running_time_ms(events: &[Value]) -> f64 {
    let turns: Vec<f64> = events
        .iter()
        .filter(|e| {
            e.get("type").and_then(Value::as_str) == Some("system")
                && e.get("subtype").and_then(Value::as_str) == Some("turn_duration")
                && matches!(e.get("durationMs"), Some(Value::Number(n)) if n.as_f64().is_some_and(f64::is_finite))
        })
        .map(|e| e.get("durationMs").and_then(Value::as_f64).unwrap_or(0.0))
        .collect();
    if !turns.is_empty() {
        return turns.iter().sum();
    }
    let mut stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
    stamps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut sum = 0.0;
    for pair in stamps.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > 0.0 && gap < 300_000.0 {
            sum += gap;
        }
    }
    sum
}

struct AgentSpan {
    start_ms: f64,
    end_ms: f64,
}

fn detect_parallel(agents: &[AgentSpan], parent_events: &[Value]) -> bool {
    let mut spans: Vec<(f64, f64)> = agents
        .iter()
        .filter(|a| a.start_ms.is_finite() && a.end_ms.is_finite())
        .map(|a| (a.start_ms, a.end_ms))
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for pair in spans.windows(2) {
        if pair[1].0 <= pair[0].1 {
            return true;
        }
    }
    for ev in parent_events {
        if ev.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let Some(Value::Array(content)) = ev.get("message").and_then(|m| m.get("content")) else {
            continue;
        };
        let agent_calls = content
            .iter()
            .filter(|b| {
                js_truthy(b)
                    && b.get("type").and_then(Value::as_str) == Some("tool_use")
                    && b.get("name").and_then(Value::as_str) == Some("Agent")
            })
            .count();
        if agent_calls >= 2 {
            return true;
        }
    }
    false
}

struct SubWalk {
    models: ModelMap,
    agents: Vec<AgentSpan>,
}

fn walk_subagents(session_dir: &Path) -> SubWalk {
    let mut out = SubWalk { models: ModelMap::default(), agents: Vec::new() };
    let sub_dir = session_dir.join("subagents");
    let Ok(entries) = std::fs::read_dir(&sub_dir) else { return out };
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    // fs.readdirSync returns OS order; sort for cross-run determinism — the
    // aggregation below is order-insensitive for every serialized field.
    names.sort();
    for name in names {
        if !name.ends_with(".jsonl") {
            continue;
        }
        let events = read_jsonl(&sub_dir.join(&name));
        let stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
        if stamps.is_empty() {
            continue;
        }
        // perf.mjs reads `<name>.meta.json` here for agentType, which nothing
        // downstream of this port consumes — but readJson still WARNS on a
        // corrupt sidecar, so the read is kept for that one observable effect.
        // (Was covered by the deleted corrupt-JSON pre-flight.)
        let meta = sub_dir.join(format!("{}.meta.json", &name[..name.len() - ".jsonl".len()]));
        let _ = read_json_failopen(&meta);
        let a_start = stamps.iter().cloned().fold(f64::INFINITY, f64::min);
        let a_end = stamps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if a_end < 0.0 {
            continue; // outside the [0, MAX] window
        }
        let agg = aggregate_usage(&events);
        for (model, m) in &agg.models.0 {
            let acc = out.models.entry(model);
            acc.input += m.input;
            acc.output += m.output;
            acc.cache_write += m.cache_write;
            acc.cache_read += m.cache_read;
        }
        out.agents.push(AgentSpan { start_ms: a_start, end_ms: a_end });
    }
    out.models.finalize();
    out
}

struct Rollup {
    session_id: String,
    cwd: Option<Value>,
    models: Value,
    subagent_models: Value,
    subagent_count: usize,
    parallel: bool,
    running_time_ms: f64,
    event_count: usize,
    started_ms: Option<f64>,
    ended_ms: Option<f64>,
}

fn rollup_transcript(file: &Path) -> Option<Rollup> {
    let events = read_jsonl(file);
    if events.is_empty() {
        return None;
    }
    let usage = aggregate_usage(&events);
    let session_dir = strip_jsonl_suffix(file);
    let sub = walk_subagents(&session_dir);
    let stamps: Vec<f64> = events.iter().filter_map(event_ms).collect();
    let cwd = events
        .iter()
        .find_map(|e| e.get("cwd").filter(|c| matches!(c, Value::String(s) if !s.is_empty())).cloned());
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let session_id = name.strip_suffix(".jsonl").unwrap_or(&name).to_string();
    Some(Rollup {
        session_id,
        cwd,
        models: usage.models.to_value(),
        subagent_models: sub.models.to_value(),
        subagent_count: sub.agents.len(),
        parallel: detect_parallel(&sub.agents, &events),
        running_time_ms: running_time_ms(&events),
        event_count: events.len(),
        started_ms: stamps.iter().cloned().fold(None, |acc: Option<f64>, x| Some(acc.map_or(x, |a| a.min(x)))),
        ended_ms: stamps.iter().cloned().fold(None, |acc: Option<f64>, x| Some(acc.map_or(x, |a| a.max(x)))),
    })
}

fn project_name(p: &Value) -> String {
    if !js_truthy(p) {
        return "(unknown)".to_string();
    }
    let s = js_to_string(p);
    let trimmed = s.trim_end_matches(['\\', '/']);
    let last = trimmed.rsplit(['\\', '/']).next().unwrap_or("");
    if last.is_empty() {
        s
    } else {
        last.to_string()
    }
}

fn session_record(rollup: &Rollup) -> Result<Value, String> {
    let mut m = Map::new();
    let n = |v: f64| serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null);
    let project = rollup.cwd.clone().unwrap_or(Value::Null);
    m.insert("schema".into(), Value::String("bee-perf/v1".into()));
    m.insert("kind".into(), Value::String("session".into()));
    m.insert("session_id".into(), Value::String(rollup.session_id.clone()));
    m.insert("project".into(), project.clone());
    m.insert("project_name".into(), Value::String(project_name(&project)));
    m.insert("branch".into(), Value::Null);
    let iso_or_null = |ms: Option<f64>| -> Result<Value, String> {
        match ms {
            None => Ok(Value::Null),
            Some(ms) => ms_to_iso(ms).map(Value::String).map_err(|_| "RangeError: Invalid time value".to_string()),
        }
    };
    m.insert("started_at".into(), iso_or_null(rollup.started_ms)?);
    m.insert("ended_at".into(), iso_or_null(rollup.ended_ms)?);
    m.insert("running_time_ms".into(), n(rollup.running_time_ms));
    m.insert("parallel".into(), Value::Bool(rollup.parallel));
    m.insert("subagent_count".into(), n(rollup.subagent_count as f64));
    m.insert("models".into(), rollup.models.clone());
    m.insert("subagent_models".into(), rollup.subagent_models.clone());
    m.insert("event_count".into(), n(rollup.event_count as f64));
    m.insert("started_ms".into(), rollup.started_ms.map(n).unwrap_or(Value::Null));
    m.insert("ended_ms".into(), rollup.ended_ms.map(n).unwrap_or(Value::Null));
    m.insert("logged_at".into(), Value::String(now_iso()));
    Ok(Value::Object(m))
}

fn upsert_session_records(records: &[Value]) -> Result<(), String> {
    let file = global_perf_log_path();
    let ids: HashSet<String> =
        records.iter().filter_map(|r| r.get("session_id").and_then(primitive_key)).collect();
    let kept: Vec<Value> = read_jsonl(&file)
        .into_iter()
        .filter(|r| {
            !(js_truthy(r)
                && r.get("kind").and_then(Value::as_str) == Some("session")
                && r.get("session_id").and_then(primitive_key).map(|k| ids.contains(&k)).unwrap_or(false))
        })
        .collect();
    let merged: Vec<&Value> = kept.iter().chain(records.iter()).collect();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("Error: {e}"))?;
    }
    let content = if merged.is_empty() {
        String::new()
    } else {
        format!("{}\n", merged.iter().map(|r| jsjson::stringify(r)).collect::<Vec<_>>().join("\n"))
    };
    std::fs::write(&file, content).map_err(|e| format!("Error: {e}"))
}

fn read_session_records() -> Vec<Value> {
    let mut by_session: Vec<(String, Value)> = Vec::new();
    for r in read_jsonl(&global_perf_log_path()) {
        if !js_truthy(&r)
            || r.get("kind").and_then(Value::as_str) != Some("session")
            || !r.get("session_id").map(js_truthy).unwrap_or(false)
        {
            continue;
        }
        let Some(key) = r.get("session_id").and_then(primitive_key) else { continue };
        let logged = js_to_string(r.get("logged_at").filter(|v| js_truthy(v)).unwrap_or(&Value::String(String::new())));
        if let Some(pos) = by_session.iter().position(|(k, _)| *k == key) {
            let prev_logged = js_to_string(
                by_session[pos].1.get("logged_at").filter(|v| js_truthy(v)).unwrap_or(&Value::String(String::new())),
            );
            if logged >= prev_logged {
                by_session[pos].1 = r;
            }
        } else {
            by_session.push((key, r));
        }
    }
    by_session.into_iter().map(|(_, v)| v).collect()
}

struct ProjectAgg {
    project: String,
    paths: Vec<String>,
    sessions: f64,
    parallel_sessions: f64,
    subagent_count: f64,
    event_count: f64,
    running_time_ms: f64,
    models: ModelMap,
    first_ms: Option<f64>,
    last_ms: Option<f64>,
    total_tokens: f64,
    new_tokens: f64,
    cached_tokens: f64,
}

fn add_raw_models(dst: &mut ModelMap, src: Option<&Value>) {
    let Some(Value::Object(src)) = src else { return };
    for (model, v) in src {
        let acc = dst.entry(model);
        acc.input += num_field(v, "input");
        acc.output += num_field(v, "output");
        acc.cache_write += num_field(v, "cache_write");
        acc.cache_read += num_field(v, "cache_read");
    }
}

fn build_matrix_from_log() -> Vec<ProjectAgg> {
    let mut by_name: Vec<ProjectAgg> = Vec::new();
    for r in read_session_records() {
        let name = match r.get("project_name").filter(|v| js_truthy(v)) {
            Some(v) => js_to_string(v),
            None => project_name(r.get("project").unwrap_or(&Value::Null)),
        };
        let pos = by_name.iter().position(|p| p.project == name).unwrap_or_else(|| {
            by_name.push(ProjectAgg {
                project: name.clone(),
                paths: Vec::new(),
                sessions: 0.0,
                parallel_sessions: 0.0,
                subagent_count: 0.0,
                event_count: 0.0,
                running_time_ms: 0.0,
                models: ModelMap::default(),
                first_ms: None,
                last_ms: None,
                total_tokens: 0.0,
                new_tokens: 0.0,
                cached_tokens: 0.0,
            });
            by_name.len() - 1
        });
        let p = &mut by_name[pos];
        if let Some(project) = r.get("project").filter(|v| js_truthy(v)) {
            let project = js_to_string(project);
            if !p.paths.contains(&project) {
                p.paths.push(project);
            }
        }
        // mergeSessionIntoProject
        p.sessions += 1.0;
        add_raw_models(&mut p.models, r.get("models"));
        p.running_time_ms += r.get("running_time_ms").map(|v| num_finite(v)).unwrap_or(0.0);
        if r.get("parallel").map(js_truthy).unwrap_or(false) {
            p.parallel_sessions += 1.0;
        }
        p.subagent_count += r.get("subagent_count").map(|v| num_finite(v)).unwrap_or(0.0);
        p.event_count += r.get("event_count").map(|v| num_finite(v)).unwrap_or(0.0);
        for (field, is_first) in [("started_ms", true), ("ended_ms", false)] {
            match r.get(field) {
                None | Some(Value::Null) => {}
                Some(v) => {
                    let x = js_number(v);
                    let slot = if is_first { &mut p.first_ms } else { &mut p.last_ms };
                    *slot = Some(match *slot {
                        None => x,
                        Some(prev) => {
                            if prev.is_nan() || x.is_nan() {
                                f64::NAN
                            } else if is_first {
                                prev.min(x)
                            } else {
                                prev.max(x)
                            }
                        }
                    });
                }
            }
        }
    }
    for p in &mut by_name {
        p.models.finalize();
        let mut total = 0.0;
        let mut fresh = 0.0;
        let mut cached = 0.0;
        for (_, m) in &p.models.0 {
            total += m.total;
            fresh += m.new_t;
            cached += m.cached;
        }
        p.total_tokens = total;
        p.new_tokens = fresh;
        p.cached_tokens = cached;
    }
    by_name.sort_by(|a, b| {
        b.total_tokens.partial_cmp(&a.total_tokens).unwrap_or(std::cmp::Ordering::Equal)
    });
    by_name
}

fn num_finite(v: &Value) -> f64 {
    match v {
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.is_finite() {
                f
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

// ── HTML rendering (renderMatrixHtml) ──────────────────────────────────────

fn js_math_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Number#toFixed for the report's 1-2 digit uses (round half away from zero
/// approximated on the scaled double).
fn js_to_fixed(v: f64, digits: u32) -> String {
    let scale = 10f64.powi(digits as i32);
    let y = v * scale;
    let fl = y.floor();
    let n = if y - fl >= 0.5 { fl + 1.0 } else { fl };
    format!("{:.*}", digits as usize, n / scale)
}

fn fmt_tokens(v: f64) -> String {
    if v >= 1e9 {
        format!("{}B", js_to_fixed(v / 1e9, 2))
    } else if v >= 1e6 {
        format!("{}M", js_to_fixed(v / 1e6, 2))
    } else if v >= 1e3 {
        format!("{}k", js_to_fixed(v / 1e3, 1))
    } else {
        jsjson::js_f64_to_string(v)
    }
}

fn cache_pct(total: f64, cached: f64) -> String {
    if total > 0.0 {
        format!("{}%", jsjson::js_f64_to_string(js_math_round(cached / total * 100.0)))
    } else {
        "—".to_string()
    }
}

fn humanize_ms(ms: f64) -> String {
    if !ms.is_finite() || ms <= 0.0 {
        return "0s".to_string();
    }
    let s = js_math_round(ms / 1000.0) as i64;
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    let mut parts = Vec::new();
    if h != 0 {
        parts.push(format!("{h}h"));
    }
    if m != 0 {
        parts.push(format!("{m}m"));
    }
    if sec != 0 || parts.is_empty() {
        parts.push(format!("{sec}s"));
    }
    parts.join("")
}

fn esc(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn short_model(model: &str) -> String {
    let m = model.strip_prefix("claude-").unwrap_or(model);
    // /-\d{6,}$/ — a trailing -<6+ digits> run.
    if let Some(dash) = m.rfind('-') {
        let tail = &m[dash + 1..];
        if tail.len() >= 6 && tail.chars().all(|c| c.is_ascii_digit()) {
            return m[..dash].to_string();
        }
    }
    m.to_string()
}

fn fmt_date(ms: Option<f64>) -> Result<String, String> {
    match ms {
        None => Ok("—".to_string()),
        Some(v) => {
            let iso = ms_to_iso(v).map_err(|_| "RangeError: Invalid time value".to_string())?;
            Ok(iso[..16].replacen('T', " ", 1))
        }
    }
}

fn render_matrix_html(projects: &[ProjectAgg], generated_at: &str) -> Result<String, String> {
    let mut totals_models = ModelMap::default();
    let mut t_sessions = 0.0;
    let mut t_running = 0.0;
    let mut t_total = 0.0;
    let mut t_new = 0.0;
    let mut t_cached = 0.0;
    for p in projects {
        t_sessions += p.sessions;
        t_running += p.running_time_ms;
        t_total += p.total_tokens;
        t_new += p.new_tokens;
        t_cached += p.cached_tokens;
        let models_value = p.models.to_value();
        add_raw_models(&mut totals_models, Some(&models_value));
    }
    totals_models.finalize();

    let mut rows_html: Vec<String> = Vec::new();
    for (i, p) in projects.iter().enumerate() {
        let mut sorted_models: Vec<&(String, ModelAcc)> = p.models.0.iter().collect();
        sorted_models
            .sort_by(|a, b| b.1.total.partial_cmp(&a.1.total).unwrap_or(std::cmp::Ordering::Equal));
        let models_rows = sorted_models
            .iter()
            .map(|(m, v)| {
                format!(
                    "<tr><td class=\"mdl\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                    esc(&short_model(m)),
                    fmt_tokens(v.total),
                    fmt_tokens(v.new_t),
                    fmt_tokens(v.cached)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        let model_names = {
            let names =
                p.models.0.iter().map(|(m, _)| short_model(m)).collect::<Vec<_>>().join(", ");
            if names.is_empty() {
                "—".to_string()
            } else {
                names
            }
        };
        let title = if p.paths.is_empty() { p.project.clone() } else { p.paths.join(", ") };
        rows_html.push(format!(
            "<tbody class=\"proj\">\n  <tr class=\"row\" data-i=\"{i}\">\n    <td class=\"name\" title=\"{}\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num strong\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}/{}</td>\n    <td class=\"models\">{}</td>\n    <td class=\"num\">{}</td>\n  </tr>\n  <tr class=\"detail\"><td colspan=\"10\"><table class=\"mtx\"><thead><tr><th>model</th><th>total</th><th>new</th><th>cached</th></tr></thead><tbody>{}</tbody></table></td></tr>\n</tbody>",
            esc(&title),
            esc(&p.project),
            jsjson::js_f64_to_string(p.sessions),
            esc(&humanize_ms(p.running_time_ms)),
            fmt_tokens(p.total_tokens),
            fmt_tokens(p.new_tokens),
            fmt_tokens(p.cached_tokens),
            cache_pct(p.total_tokens, p.cached_tokens),
            jsjson::js_f64_to_string(p.parallel_sessions),
            jsjson::js_f64_to_string(p.sessions),
            esc(&model_names),
            esc(&fmt_date(p.last_ms)?),
            models_rows
        ));
    }
    let summary = [
        ("projects", jsjson::js_f64_to_string(projects.len() as f64)),
        ("sessions", jsjson::js_f64_to_string(t_sessions)),
        ("active time", humanize_ms(t_running)),
        ("total tokens", fmt_tokens(t_total)),
        ("new", fmt_tokens(t_new)),
        ("cached", fmt_tokens(t_cached)),
        ("cache %", cache_pct(t_total, t_cached)),
    ]
    .iter()
    .map(|(k, v)| format!("<div class=\"card\"><div class=\"k\">{}</div><div class=\"v\">{}</div></div>", esc(k), esc(v)))
    .collect::<Vec<_>>()
    .join("");
    let rows = rows_html.join("\n");
    let rows_or_empty = if rows.is_empty() {
        "<tbody><tr><td class=\"empty\" colspan=\"10\">No sessions found yet. Do some work, then reopen this page.</td></tr></tbody>".to_string()
    } else {
        rows
    };
    let generated = esc(&generated_at.chars().take(19).collect::<String>().replacen('T', " ", 1));
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>bee performance</title>
<style>
:root{{--bg:#f7f8fa;--fg:#1a1d23;--muted:#6b7280;--card:#fff;--line:#e5e7eb;--accent:#b45309;--rowhover:#f0f1f4;}}
@media (prefers-color-scheme: dark){{:root{{--bg:#0f1115;--fg:#e6e8eb;--muted:#9aa1ab;--card:#171a21;--line:#262b34;--accent:#f59e0b;--rowhover:#1c2029;}}}}
*{{box-sizing:border-box}}
body{{margin:0;background:var(--bg);color:var(--fg);font:14px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;padding:24px;}}
h1{{font-size:20px;margin:0 0 4px}}
.sub{{color:var(--muted);font-size:12px;margin-bottom:20px}}
.cards{{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:24px}}
.card{{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 16px;min-width:110px}}
.card .k{{color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.04em}}
.card .v{{font-size:20px;font-weight:600;margin-top:2px}}
.wrap{{overflow-x:auto;border:1px solid var(--line);border-radius:10px;background:var(--card)}}
table.matrix{{border-collapse:collapse;width:100%;min-width:820px}}
table.matrix thead th{{position:sticky;top:0;background:var(--card);text-align:right;padding:10px 12px;font-size:11px;text-transform:uppercase;letter-spacing:.04em;color:var(--muted);border-bottom:1px solid var(--line);cursor:pointer;white-space:nowrap}}
table.matrix thead th:first-child{{text-align:left}}
.row td{{padding:10px 12px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap}}
.row td.name{{text-align:left;font-weight:600;max-width:340px;overflow:hidden;text-overflow:ellipsis}}
.row td.models{{text-align:left;color:var(--muted);font-size:12px;max-width:220px;overflow:hidden;text-overflow:ellipsis}}
.row td.strong{{font-weight:700;color:var(--accent)}}
.num{{font-variant-numeric:tabular-nums}}
.row:hover{{background:var(--rowhover)}}
.row{{cursor:pointer}}
.detail{{display:none}}
.detail.open{{display:table-row}}
.detail td{{padding:0 12px 12px 24px;border-bottom:1px solid var(--line)}}
table.mtx{{border-collapse:collapse;margin:6px 0}}
table.mtx th,table.mtx td{{padding:3px 14px 3px 0;text-align:right;font-size:12px;color:var(--muted)}}
table.mtx th:first-child,table.mtx td.mdl{{text-align:left;color:var(--fg)}}
.empty{{padding:40px;text-align:center;color:var(--muted)}}
</style>
</head>
<body>
<h1>bee performance</h1>
<div class="sub">{count} project(s) · generated {generated} UTC · active time excludes idle</div>
<div class="cards">{summary}</div>
<div class="wrap">
<table class="matrix">
<thead><tr>
<th data-sort="name">Project</th><th data-sort="num">Sessions</th><th data-sort="num">Active</th>
<th data-sort="num">Total</th><th data-sort="num">New</th><th data-sort="num">Cached</th><th data-sort="num">Cache%</th>
<th data-sort="num">Parallel</th><th data-sort="name">Models</th><th data-sort="num">Last active</th>
</tr></thead>
{rows_or_empty}
</table>
</div>
<script>
// expand a project row to show its per-model breakdown
document.querySelectorAll('tr.row').forEach(function(r){{
  r.addEventListener('click',function(){{
    var d=r.parentNode.querySelector('tr.detail');
    if(d) d.classList.toggle('open');
  }});
}});
</script>
</body>
</html>
"#,
        count = jsjson::js_f64_to_string(projects.len() as f64),
    ))
}

/// maybePerfRefresh — best-effort; Err(msg) => logCrash(source 'perf-refresh').
fn perf_refresh(root: &Path, session_id: Option<&str>) -> Result<(), String> {
    if let Some(transcript) = resolve_transcript_for(root, session_id) {
        if let Some(rollup) = rollup_transcript(&transcript) {
            let record = session_record(&rollup)?;
            upsert_session_records(&[record])?;
        }
    }
    if !read_session_records().is_empty() {
        let projects = build_matrix_from_log();
        let html = render_matrix_html(&projects, &now_iso())?;
        let out = global_perf_dir().join("performance.html");
        if let Some(dir) = out.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Error: {e}"))?;
        }
        std::fs::write(&out, html).map_err(|e| format!("Error: {e}"))?;
    }
    Ok(())
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let lib = root.join(".bee").join("bin").join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        for name in ["state.mjs", "inject.mjs", "decisions.mjs", "capture.mjs", "knowledge.mjs", "cells.mjs", "reservations.mjs"] {
            std::fs::write(lib.join(name), "// stub\n").unwrap();
        }
        dir
    }

    fn write_json_file(path: &Path, v: &Value) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(v).unwrap())).unwrap();
    }

    fn run_stop(root: &Path, extra: Value) -> Result<(String, Vec<String>, String), ()> {
        // Runs the advisory pipeline the way run_inner does (skipping the perf
        // refresh so tests never touch the machine-global perf store), and
        // returns (stdout, parts, stderr).
        let mut body = json!({"hook_event_name": "Stop", "cwd": root.to_string_lossy()});
        if let Value::Object(m) = extra {
            for (k, v) in m {
                body[k.as_str()] = v;
            }
        }
        let stdin = serde_json::to_string(&body).unwrap();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let root = ctx.root.clone().expect("fixture root resolves");
        let session_id = get_session_id(&ctx.payload);
        clear_corrupt_json_warnings();
        let config = preflight(&root)?;
        let mut parts = Vec::new();
        let mut stderr = String::new();
        let mut stdout = String::new();
        match advisory(&root, &ctx, &config, session_id.as_deref(), &mut parts, &mut stderr) {
            Ok(AdvisoryOutcome::Block(reason)) => stdout = encode_block(&reason),
            Ok(_) => {}
            Err(Flow::Delegate) => return Err(()),
            Err(Flow::Crash(_)) => {}
        }
        // flush() writes the queued corrupt-JSON warnings ahead of `stderr`;
        // tests read them from the same string.
        Ok((stdout, parts, format!("{}{stderr}", take_corrupt_json_warnings())))
    }

    #[test]
    fn bypass_net_blocks_planning_once_then_steps_aside() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "total"}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "planning", "mode": "standard", "approved_gates": {"execution": false}}),
        );
        let (stdout, parts, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
        assert!(stdout.starts_with("{\"decision\":\"block\",\"reason\":\"⚡ GATE BYPASS (total): "));
        assert!(stdout.contains("mid-planning with Gate 3 (execution) still pending"));
        assert!(stdout.contains("state gate --name execution --approved true"));
        assert!(!stdout.contains("High-risk execution requires"));
        assert!(parts.is_empty());
        // loop-guard: the same (session, phase, gate, level) key degrades to advisory
        let (stdout2, _, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
        assert_eq!(stdout2, "");
    }

    #[test]
    fn bypass_net_high_risk_consult_sentence_and_mode_floor() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "full"}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "validating", "mode": "high-risk", "approved_gates": {}}),
        );
        let (stdout, _, _) = run_stop(root, json!({})).unwrap();
        // legacy 'validating' coerces to planning; full covers high-risk
        assert!(stdout.contains("High-risk execution requires a live advisor consult first"));
        // normal does NOT cover high-risk
        let fx2 = fixture();
        let root2 = fx2.path();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({"gate_bypass": true}));
        write_json_file(
            &root2.join(".bee").join("state.json"),
            &json!({"phase": "planning", "mode": "high-risk", "approved_gates": {}}),
        );
        let (stdout2, _, _) = run_stop(root2, json!({})).unwrap();
        assert_eq!(stdout2, "");
    }

    #[test]
    fn mid_phase_warning_lists_cells_and_reservations() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "swarming"}));
        write_json_file(
            &root.join(".bee").join("cells").join("b.json"),
            &json!({"id": "w-10", "status": "claimed", "trace": {"worker": "worker-b"}}),
        );
        write_json_file(
            &root.join(".bee").join("cells").join("a.json"),
            &json!({"id": "w-2", "status": "claimed"}),
        );
        write_json_file(
            &root.join(".bee").join("cells").join("c.json"),
            &json!({"id": "w-3", "status": "capped"}),
        );
        write_json_file(
            &root.join(".bee").join("runtime").join("leases").join("paths").join("h1.json"),
            &json!({"resource": "path:src/api", "workflow_id": "w-2", "workspace_id": "agent:alpha", "acquired_at": "2026-01-01T00:00:00.000Z", "expires_at": null}),
        );
        let (stdout, parts, _) = run_stop(root, json!({})).unwrap();
        assert_eq!(stdout, "");
        assert_eq!(parts.len(), 1);
        let text = &parts[0];
        assert!(text.starts_with("bee session-close warning: session is ending mid-phase (phase: swarming) "));
        // numeric-aware id sort: w-2 before w-10
        assert!(text.contains("Claimed-but-uncapped cells: w-2, w-10 (worker-b)."));
        assert!(text.contains("Active reservations: alpha -> src/api (cell w-2)."));
        assert!(text.ends_with("resume cleanly."));
    }

    #[test]
    fn handoff_suppresses_warning_and_expired_leases_drop() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "swarming"}));
        write_json_file(&root.join(".bee").join("HANDOFF.json"), &json!({"kind": "pause"}));
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert!(parts.is_empty());
        // expired lease is not "active"
        std::fs::remove_file(root.join(".bee").join("HANDOFF.json")).unwrap();
        write_json_file(
            &root.join(".bee").join("runtime").join("leases").join("paths").join("h1.json"),
            &json!({"resource": "path:src", "workflow_id": "w", "workspace_id": "agent:a", "acquired_at": "2020-01-01T00:00:00.000Z", "expires_at": "2020-01-01T01:00:00.000Z"}),
        );
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(!parts[0].contains("Active reservations"));
    }

    #[test]
    fn capture_queue_nudge_counts_pending_and_dedupes() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let queue = root.join(".bee").join("capture-queue.jsonl");
        std::fs::write(
            &queue,
            concat!(
                "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\",\"outcome\":\"x\"}\n",
                "{\"kind\":\"stub\",\"id\":\"s2\",\"at\":\"2026-01-02T00:00:00.000Z\",\"outcome\":\"y\"}\n",
                "{\"kind\":\"flush\",\"id\":\"s1\",\"at\":\"2026-01-03T00:00:00.000Z\"}\n"
            ),
        )
        .unwrap();
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert!(parts.iter().any(|p| p.starts_with("bee capture queue (decision 0017): 1 settlement stub(s) are queued")));
        // deduped on the second run (same pending set, < 30 min)
        let (_, parts2, _) = run_stop(root, json!({})).unwrap();
        assert!(!parts2.iter().any(|p| p.contains("bee capture queue")));
    }

    #[test]
    fn capture_nudge_fires_when_decision_newer_than_docs() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        std::fs::create_dir_all(root.join("docs").join("specs")).unwrap();
        std::fs::write(root.join("docs").join("specs").join("area.md"), "# spec\n").unwrap();
        let recent = ms_to_iso(now_ms() + 60_000.0).unwrap(); // decision newer than the spec file
        std::fs::write(
            root.join(".bee").join("decisions.jsonl"),
            format!("{{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"{recent}\",\"decision\":\"x\"}}\n"),
        )
        .unwrap();
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        let nudge = parts.iter().find(|p| p.starts_with("bee capture nudge (decision 0003)")).unwrap();
        assert!(nudge.contains("area spec under docs/specs/")); // no-bundle variant
        // bundle variant: a concept with type frontmatter flips the wording
        let fx2 = fixture();
        let root2 = fx2.path();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root2.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let bundle = root2.join("docs").join("knowledge").join("areas").join("x");
        std::fs::create_dir_all(&bundle).unwrap();
        std::fs::write(bundle.join("concept.md"), "---\ntype: concept\n---\nbody\n").unwrap();
        std::fs::write(
            root2.join(".bee").join("decisions.jsonl"),
            format!("{{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"{recent}\",\"decision\":\"x\"}}\n"),
        )
        .unwrap();
        // make the concept file older than the decision
        let old = filetime::FileTime::from_unix_time(1_600_000_000, 0);
        filetime::set_file_mtime(bundle.join("concept.md"), old).unwrap();
        let (_, parts2, _) = run_stop(root2, json!({})).unwrap();
        let nudge2 = parts2.iter().find(|p| p.starts_with("bee capture nudge (decision 0003)")).unwrap();
        assert!(nudge2.contains("knowledge bundle (docs/knowledge/)"));
    }

    #[test]
    fn superseded_and_redacted_decisions_are_skipped() {
        let fx = fixture();
        let root = fx.path();
        std::fs::write(
            root.join(".bee").join("decisions.jsonl"),
            concat!(
                "{\"id\":\"a\",\"type\":\"decide\",\"date\":\"2026-01-01T00:00:00.000Z\"}\n",
                "{\"id\":\"b\",\"type\":\"decide\",\"date\":\"2026-01-02T00:00:00.000Z\"}\n",
                "{\"id\":\"c\",\"type\":\"redact\",\"redacts\":\"b\",\"date\":\"2026-01-03T00:00:00.000Z\"}\n"
            ),
        )
        .unwrap();
        let (id, date) = newest_active_decision(root).unwrap();
        assert_eq!(id, json!("a"));
        assert_eq!(date, json!("2026-01-01T00:00:00.000Z"));
    }

    #[test]
    fn corrupt_state_reads_as_defaults_and_precompact_still_delegates() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        std::fs::write(root.join(".bee").join("state.json"), "{broken").unwrap();
        let (stdout, parts, stderr) = run_stop(root, json!({})).expect("must run natively");
        // defaultState() → phase idle → the decision-nudge branch, never the
        // mid-phase warning; no block; the corruption is reported once.
        assert_eq!(stdout, "");
        assert!(!parts.iter().any(|p| p.contains("hive door open")));
        // TWO lines, matching Node: bee-session-close.mjs reads state.json
        // once itself and once more through resolvePipeline's defaults().
        assert_eq!(stderr.matches("could not parse JSON at").count(), 2);
        assert!(stderr.contains("Using fallback; fix the file."));
        // PreCompact still delegates in run_inner.
        let fx2 = fixture();
        write_json_file(&fx2.path().join(".bee").join("config.json"), &json!({}));
        let body = json!({"hook_event_name": "PreCompact", "cwd": fx2.path().to_string_lossy()});
        assert!(run_inner(&[], &serde_json::to_string(&body).unwrap()).is_err());
    }

    #[test]
    fn corrupt_handoff_still_raises_the_mid_phase_warning() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "swarming", "mode": "standard"}),
        );
        std::fs::write(root.join(".bee").join("HANDOFF.json"), "{broken").unwrap();
        let (stdout, parts, stderr) = run_stop(root, json!({})).expect("must run natively");
        assert_eq!(stdout, "");
        // readHandoff's null fallback = "no handoff" → the door-open warning.
        assert!(parts.iter().any(|p| p.contains("You are about to leave the hive door open")));
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_lane_record_refuses_and_falls_back_to_state() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_json_file(
            &root.join(".bee").join("sessions").join("s-1.json"),
            &json!({"id": "s-1", "lane": "l1"}),
        );
        std::fs::create_dir_all(root.join(".bee").join("lanes")).unwrap();
        std::fs::write(root.join(".bee").join("lanes").join("l1.json"), "{broken").unwrap();
        let (_, _, stderr) = run_stop(root, json!({"session_id": "s-1"})).expect("native");
        // Both of Node's lines, in Node's order: readJson's, then readLane's.
        let readjson_at = stderr.find("could not parse JSON at").unwrap();
        let readlane_at = stderr.find("readLane: skipping corrupt lane record").unwrap();
        assert!(readjson_at < readlane_at);
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_session_record_reads_as_no_session() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        std::fs::create_dir_all(root.join(".bee").join("sessions")).unwrap();
        std::fs::write(root.join(".bee").join("sessions").join("s-1.json"), "{broken").unwrap();
        let (stdout, _, stderr) = run_stop(root, json!({"session_id": "s-1"})).expect("native");
        assert_eq!(stdout, "");
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_cell_is_skipped_from_the_claimed_list() {
        let fx = fixture();
        let root = fx.path();
        let cells = root.join(".bee").join("cells");
        write_json_file(&cells.join("c-1.json"), &json!({"id": "c-1", "status": "claimed"}));
        std::fs::write(cells.join("bad.json"), "{broken").unwrap();
        let listed = list_claimed_cells(root).expect("must not delegate");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].get("id"), Some(&json!("c-1")));
        assert_eq!(take_corrupt_json_warnings().matches("bad.json").count(), 1);
    }

    #[test]
    fn corrupt_inject_cache_falls_through_to_empty() {
        let fx = fixture();
        let root = fx.path();
        clear_corrupt_json_warnings();
        std::fs::create_dir_all(root.join(".bee").join("cache")).unwrap();
        std::fs::write(inject_cache_path(root), "{broken").unwrap();
        // Reads as absent → `{}` → every key is due for injection again.
        let cache = read_inject_cache(root).expect("must not delegate");
        assert!(cache.is_empty());
        assert!(should_inject(root, "any-key", "h1").unwrap());
        // A non-object cache is still a delegate (JS assignment exotica).
        std::fs::write(inject_cache_path(root), "[1,2]").unwrap();
        assert!(read_inject_cache(root).is_err());
        clear_corrupt_json_warnings();
    }

    #[test]
    fn frontmatter_subset_rules() {
        assert!(frontmatter_has_type("---\ntype: concept\n---\nbody\n"));
        assert!(frontmatter_has_type("---\r\ntitle: \"x: y\"\r\ntype: note\r\n---\r\n"));
        assert!(!frontmatter_has_type("no frontmatter"));
        assert!(!frontmatter_has_type("---\ntype: concept\n")); // unclosed
        assert!(!frontmatter_has_type("---\ntype: concept\n\n---\n")); // blank line
        assert!(!frontmatter_has_type("---\ntype: true\n---\n")); // boolean type
        assert!(!frontmatter_has_type("---\ntype: \"\"\n---\n")); // empty string
        assert!(!frontmatter_has_type("---\ntype: concept\ntype: again\n---\n")); // dup
        assert!(!frontmatter_has_type("---\nnested:\n  k: v\n---\n")); // non-bee map
        assert!(frontmatter_has_type("---\ntype: concept\nbee:\n  cell: x\n---\n"));
        assert!(!frontmatter_has_type("---\ntags: [a, \"b\"\ntype: t\n---\n")); // bad list
        assert!(frontmatter_has_type("---\ntags: [a, \"b\"]\ntype: t\n---\n"));
    }

    #[test]
    fn locale_numeric_sort_matches_expected_slug_order() {
        let mut ids = vec!["w-10", "w-2", "w-1", "x-1", "a2", "a10", "A3"];
        ids.sort_by(|a, b| cmp_locale_numeric(a, b));
        assert_eq!(ids, vec!["a2", "A3", "a10", "w-1", "w-2", "w-10", "x-1"]);
    }

    #[test]
    fn perf_helpers_match_node_shapes() {
        // Cutover fix: the drive colon is encoded away too, so the name is
        // legal on NTFS (Node spelled "D:-a-b-c", a component mkdir rejects).
        assert_eq!(encode_project_dir("D:\\a\\b.c"), "D--a-b-c");
        assert_eq!(encode_project_dir("/a/b.c"), "-a-b-c");
        assert_eq!(humanize_ms(3_723_000.0), "1h2m3s");
        assert_eq!(humanize_ms(0.0), "0s");
        assert_eq!(fmt_tokens(1_234.0), "1.2k");
        assert_eq!(fmt_tokens(999.0), "999");
        assert_eq!(fmt_tokens(2_500_000.0), "2.50M");
        assert_eq!(short_model("claude-sonnet-4-20250514"), "sonnet-4");
        assert_eq!(short_model("gpt-5.5"), "gpt-5.5");
        assert_eq!(cache_pct(200.0, 50.0), "25%");
        assert_eq!(cache_pct(0.0, 0.0), "—");
        assert_eq!(project_name(&json!("D:\\x\\proj\\")), "proj");
        assert_eq!(project_name(&Value::Null), "(unknown)");
    }

    #[test]
    fn rollup_and_upsert_roundtrip_in_isolated_perf_dir() {
        // BEEHIVE_PERF_DIR isolates the machine-global store for this test.
        let perf = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        let tdir = tempfile::tempdir().unwrap();
        let transcript = tdir.path().join("sess-1.jsonl");
        std::fs::write(
            &transcript,
            concat!(
                "{\"type\":\"assistant\",\"timestamp\":\"2026-01-01T00:00:00.000Z\",\"requestId\":\"r1\",\"cwd\":\"D:\\\\p\\\\demo\",\"message\":{\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":10,\"output_tokens\":5,\"cache_read_input_tokens\":100}}}\n",
                "{\"type\":\"assistant\",\"timestamp\":\"2026-01-01T00:01:00.000Z\",\"requestId\":\"r1\",\"message\":{\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":10,\"output_tokens\":9,\"cache_read_input_tokens\":100}}}\n",
                "{\"type\":\"system\",\"subtype\":\"turn_duration\",\"timestamp\":\"2026-01-01T00:01:01.000Z\",\"durationMs\":1500}\n"
            ),
        )
        .unwrap();
        let rollup = rollup_transcript(&transcript).unwrap();
        assert_eq!(rollup.session_id, "sess-1");
        assert_eq!(rollup.event_count, 3);
        assert_eq!(rollup.running_time_ms, 1500.0);
        // requestId dedupe keeps the larger-output record
        assert_eq!(
            jsjson::stringify(&rollup.models),
            r#"{"claude-sonnet-4-20250514":{"input":10,"output":9,"cache_write":0,"cache_read":100,"new":19,"cached":100,"total":119}}"#
        );
        let record = session_record(&rollup).unwrap();
        upsert_session_records(&[record.clone()]).unwrap();
        upsert_session_records(&[record]).unwrap(); // dedupe by session_id
        assert_eq!(read_session_records().len(), 1);
        let projects = build_matrix_from_log();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project, "demo");
        assert_eq!(projects[0].total_tokens, 119.0);
        let html = render_matrix_html(&projects, "2026-01-01T00:00:00.000Z").unwrap();
        assert!(html.contains("<title>bee performance</title>"));
        assert!(html.contains("sonnet-4"));
        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
    }
}
