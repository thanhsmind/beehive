// bee hook activity — agent-activity-hook D1/D2/D3/D5.
//
// The WRITER side of "what is this agent session doing right now". A dashboard
// (waggledance) reads `.bee/sessions/<id>.json` and wants `working` /
// `waiting_input` / `blocked` / `idle` / `exited` without screen-scraping a
// pane. This hook is the only thing that writes that field.
//
// POSTURE (D1). Passive measurement, zero enforcement, exactly like
// `tools_logger`: it can never deny, never writes a verdict, never prints on
// stdout, and ALWAYS exits 0. Every failure — an unusable payload, a busy
// lock, a refused `waiting_on` target — is swallowed into
// `.bee/logs/hooks.jsonl` (log_crash) plus one line in the hook's own capped
// `.bee/logs/hook-activity.log`. A non-zero exit here would block a PreToolUse
// call, i.e. freeze a whole session over a bookkeeping bug.
//
// STATE MACHINE (D3). The mapping is in `map_event`; the two STICKY rules are
// in `sticky_suppresses`. `blocked` and `waiting_input` are the states that
// mean "a human is needed", so ordinary tool traffic must NOT wash them away:
//   * `blocked` survives PreToolUse/PostToolUse/PostToolUseFailure unless the
//     event names the SAME `tool_use_id` that blocked (or, when no id was
//     recorded, the same `tool_name`) — a suppressed event is not a
//     transition and writes nothing at all;
//   * `waiting_input` survives everything except a turn boundary
//     (UserPromptSubmit / Stop) or a stronger human-needed signal
//     (`blocked` / `exited`).
//
// STORE (D2). `activity{state,event,tool_name?,tool_use_id?,at,pane?,cwd}` on
// the session record, plus the last 50 TRANSITIONS in
// `<id>.activity.jsonl` (every session enumerator filters on `.json`, so the
// sidecar is invisible to them). A repeat of the same state refreshes
// `at`/`event` on the record but is NOT a transition, or the sidecar would be
// a tool log instead of a state history.
//
// waiting_on (D5). A transition INTO `waiting_input`/`blocked` also sets the
// record's waiting mark (`question`/`gate`) through the SAME store functions
// `bee state waiting-on set` uses — no second writer. `activity
// .waiting_on_set_by_hook` records that the hook owns that mark: the hook
// clears it at the next turn boundary, and never overwrites a live
// gate/question mark it did not set (the agent's own question always wins).

use crate::fsutil::{append_jsonl, read_json, ReadJson};
use crate::hooks::adapter::{bee_installed, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::lock::{acquire_store_lock_once, AcquireOnce};
use crate::state::hook_enabled;
use crate::verbs::reservations::Err2;
use crate::verbs::state_group::store::set_default_state_waiting_on;
use crate::verbs::state_group::waiting_on::resolve_waiting_on_target;
use crate::verbs::workflow_store::{
    clear_default_state_waiting_on, clear_workflow_waiting_on, read_workflow_record,
    rebuild_lane_projection, rebuild_state_projection, set_workflow_waiting_on, waiting_on_is_live,
};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "activity";
/// D1's "capped log": past this the file is cut down to its last half.
const LOG_CAP_BYTES: u64 = 256 * 1024;
/// D2's transition window.
const MAX_TRANSITIONS: usize = 50;
/// The verb name the `waiting_on` target resolver puts in its refusals.
const WAITING_ON_VERB: &str = "hook activity";

// ── the five states (D3) ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Working,
    Blocked,
    WaitingInput,
    Idle,
    Exited,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            State::Working => "working",
            State::Blocked => "blocked",
            State::WaitingInput => "waiting_input",
            State::Idle => "idle",
            State::Exited => "exited",
        }
    }

    fn parse(s: &str) -> Option<State> {
        match s {
            "working" => Some(State::Working),
            "blocked" => Some(State::Blocked),
            "waiting_input" => Some(State::WaitingInput),
            "idle" => Some(State::Idle),
            "exited" => Some(State::Exited),
            _ => None,
        }
    }
}

/// D3's event → state table. `None` means "this event says nothing about the
/// agent's state" — SubagentStop (never handled, by decision), a
/// `SessionEnd` that is really a `/clear` or a `--resume`, an unrecognised
/// `Notification`, and every event bee was not told about.
fn map_event(event: &str, notification_type: Option<&str>, reason: Option<&str>) -> Option<State> {
    match event {
        "UserPromptSubmit" | "PreToolUse" | "PostToolUse" | "PostToolUseFailure" => {
            Some(State::Working)
        }
        "PermissionRequest" => Some(State::Blocked),
        "Notification" => match notification_type.unwrap_or("") {
            "permission_prompt" => Some(State::Blocked),
            "agent_needs_input" => Some(State::WaitingInput),
            "idle_prompt" | "agent_completed" => Some(State::Idle),
            _ => None,
        },
        "Stop" => Some(State::Idle),
        // `clear` and `resume` end a TRANSCRIPT, not a session: the same
        // session id keeps working right after, so marking it exited would
        // make a live agent read as gone.
        "SessionEnd" => match reason.unwrap_or("") {
            "clear" | "resume" => None,
            _ => Some(State::Exited),
        },
        _ => None,
    }
}

/// A turn boundary — the human just spoke, or the agent just finished
/// speaking. Both end any wait the hook itself recorded (D5).
fn is_turn_boundary(event: &str) -> bool {
    event == "UserPromptSubmit" || event == "Stop"
}

// ── entry point ─────────────────────────────────────────────────────────────

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    Outcome::Done(run_inner(argv, stdin))
}

fn run_inner(argv: &[String], stdin: &str) -> ExitCode {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return ExitCode::SUCCESS; // not in a bee repo — decide nothing
    };
    if !bee_installed(&root) {
        return ExitCode::SUCCESS;
    }
    if !hook_enabled(&root, HOOK_NAME) {
        return ExitCode::SUCCESS;
    }
    if let Err(message) = record_activity(&ctx, &root) {
        log_crash(Some(&root), HOOK_NAME, &message, ctx.source);
        append_activity_log(&root, &message);
    }
    ExitCode::SUCCESS
}

// ── payload helpers ─────────────────────────────────────────────────────────

/// A trimmed, non-empty string field; every other JSON shape reads as absent.
fn field(payload: &Map<String, Value>, key: &str) -> Option<String> {
    match payload.get(key) {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

/// Session ids address files, so the same shape check every other session
/// writer makes (state_sync's `well_formed_id`).
fn well_formed_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

// ── the run, once the gates are open ────────────────────────────────────────

struct Prior {
    state: Option<State>,
    tool_name: Option<String>,
    tool_use_id: Option<String>,
    marker: bool,
    /// The record's own `lane` — the session's bound feature, when it has one.
    /// Read here so the work stamp costs no second read of the same file.
    lane: Option<String>,
}

fn record_activity(ctx: &HookContext, root: &Path) -> Result<(), String> {
    let event = field(&ctx.payload, "hook_event_name").unwrap_or_default();
    // No session id, no record to write: this is the empty/garbage/huge-stdin
    // case the contract matrix feeds us, and it earns its own log line.
    let Some(session_id) = field(&ctx.payload, "session_id") else {
        return Err(format!(
            "unusable payload \u{2014} no session_id (hook_event_name \"{event}\"); nothing recorded"
        ));
    };
    if !well_formed_id(&session_id) {
        return Err(format!(
            "session_id {} is not a plain id (no path separators); nothing recorded",
            crate::jsjson::stringify(&Value::String(session_id))
        ));
    }

    let notification_type = field(&ctx.payload, "notification_type");
    let reason = field(&ctx.payload, "reason");
    let Some(target) = map_event(&event, notification_type.as_deref(), reason.as_deref()) else {
        return Ok(()); // no transition this event describes — silent, exit 0
    };

    let ctrl = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
    let tool_name = field(&ctx.payload, "tool_name");
    let tool_use_id = field(&ctx.payload, "tool_use_id");
    let prior = read_prior(&ctrl, &session_id);

    if sticky_suppresses(&prior, &event, target, tool_name.as_deref(), tool_use_id.as_deref()) {
        return Ok(()); // D3: the human-needed state stands; not a transition
    }
    let is_transition = match prior.state {
        Some(s) if s == target => {
            // A repeat is not history — EXCEPT a second, different block: two
            // permission prompts in a row are two things to approve.
            target == State::Blocked
                && tool_use_id.is_some()
                && tool_use_id != prior.tool_use_id
        }
        _ => true,
    };

    // D5 runs OUTSIDE the sessions lock: its setters take the `state` /
    // workflow locks of their own, and nesting store locks is how a hook
    // deadlocks a session. A refusal here is logged and the activity write
    // still happens — the mark is a courtesy, the state is the contract.
    let mut marker = prior.marker;
    let waiting_on = sync_waiting_on(
        root,
        &session_id,
        &event,
        target,
        is_transition,
        marker,
        tool_name.as_deref(),
        field(&ctx.payload, "message").as_deref(),
    );
    match waiting_on {
        Ok(Some(next)) => marker = next,
        Ok(None) => {}
        Err(message) => append_activity_log(root, &format!("waiting_on: {message}")),
    }

    let mut activity = Map::new();
    activity.insert("state".into(), Value::String(target.as_str().to_string()));
    activity.insert("event".into(), Value::String(event.clone()));
    if let Some(name) = &tool_name {
        activity.insert("tool_name".into(), Value::String(name.clone()));
    }
    if let Some(id) = &tool_use_id {
        activity.insert("tool_use_id".into(), Value::String(id.clone()));
    }
    activity.insert("at".into(), Value::String(now_iso()));
    if let Some((transport, pane)) = pane_from_env(|key| std::env::var(key).ok()) {
        activity.insert("pane".into(), Value::String(pane));
        activity.insert("pane_transport".into(), Value::String(transport));
    }
    activity.insert("cwd".into(), Value::String(ctx.cwd.to_string_lossy().into_owned()));
    // WHAT this session is working on (D2, additive). Both fields are
    // resolved fresh on every write and are fail-open: unknown means the key
    // is simply absent — never an error, never a non-zero exit. Neither takes
    // part in the transition test, so a session that changes lane or picks up
    // a new cell without changing state refreshes the record and appends
    // nothing to the sidecar; the sidecar stays a state history.
    if let Some(feature) = resolve_feature(&ctrl, prior.lane.as_deref()) {
        activity.insert("feature".into(), Value::String(feature));
    }
    if let Some(cell) = resolve_cell(&ctrl, &session_id) {
        activity.insert("cell".into(), Value::String(cell));
    }
    if marker {
        activity.insert("waiting_on_set_by_hook".into(), Value::Bool(true));
    }

    write_activity(ctx, &ctrl, &session_id, &activity, is_transition)
}

/// The pane this session runs in, and the name of the multiplexer that named
/// it: `("herdr", id)` from `HERDR_PANE_ID`, else `("tmux", id)` from
/// `TMUX_PANE`, else nothing. This is identity recording — it reports the pane
/// the session is ALREADY inside — so it reads no config and selects no
/// transport; D1 governs transport choice and is untouched here.
fn pane_from_env(lookup: impl Fn(&str) -> Option<String>) -> Option<(String, String)> {
    for (transport, key) in [("herdr", "HERDR_PANE_ID"), ("tmux", "TMUX_PANE")] {
        if let Some(id) = lookup(key).filter(|p| !p.trim().is_empty()) {
            return Some((transport.to_string(), id.trim().to_string()));
        }
    }
    None
}

/// D3's two sticky rules. `true` means "leave the record exactly as it is".
fn sticky_suppresses(
    prior: &Prior,
    event: &str,
    target: State,
    tool_name: Option<&str>,
    tool_use_id: Option<&str>,
) -> bool {
    match prior.state {
        Some(State::Blocked)
            if matches!(event, "PreToolUse" | "PostToolUse" | "PostToolUseFailure") =>
        {
            // Only the tool call that BLOCKED can unblock: any other tool
            // traffic in the same turn is a different call, and letting it
            // through would report "working" while a prompt still waits.
            let clears = match &prior.tool_use_id {
                Some(blocked_id) => tool_use_id == Some(blocked_id.as_str()),
                None => match &prior.tool_name {
                    Some(blocked_name) => tool_name == Some(blocked_name.as_str()),
                    None => false,
                },
            };
            !clears
        }
        Some(State::WaitingInput) => {
            // A question stands until the human answers (UserPromptSubmit),
            // the turn ends (Stop), or something stronger happens.
            !is_turn_boundary(event) && target != State::Exited && target != State::Blocked
        }
        _ => false,
    }
}

// ── session-record reads / writes (D2) ──────────────────────────────────────

/// Fail-open display read, `state_sync::read_session_failopen`'s shape: a
/// missing, corrupt, or id-mismatched record reads as "no session".
fn read_session_failopen(ctrl: &Path, session_id: &str) -> Option<Map<String, Value>> {
    if !well_formed_id(session_id) {
        return None;
    }
    let file = session_file(ctrl, session_id);
    let session = match read_json(&file) {
        ReadJson::Parsed(Value::Object(m)) => m,
        _ => return None,
    };
    if session.get("id") != Some(&Value::String(session_id.to_string())) {
        return None;
    }
    Some(session)
}

fn sessions_dir(ctrl: &Path) -> PathBuf {
    ctrl.join(".bee").join("sessions")
}

fn session_file(ctrl: &Path, session_id: &str) -> PathBuf {
    sessions_dir(ctrl).join(format!("{session_id}.json"))
}

fn read_prior(ctrl: &Path, session_id: &str) -> Prior {
    let record = read_session_failopen(ctrl, session_id);
    let activity = record
        .as_ref()
        .and_then(|r| r.get("activity"))
        .and_then(Value::as_object);
    let str_of = |key: &str| {
        activity
            .and_then(|a| a.get(key))
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };
    let lane = record
        .as_ref()
        .and_then(|r| r.get("lane"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Prior {
        state: str_of("state").as_deref().and_then(State::parse),
        tool_name: str_of("tool_name"),
        tool_use_id: str_of("tool_use_id"),
        marker: activity.and_then(|a| a.get("waiting_on_set_by_hook")) == Some(&Value::Bool(true)),
        lane,
    }
}

/// The minimal record D2 promises when the hook meets a session no
/// `SessionStart` ever registered (a `--resume`d agent, or hooks installed
/// mid-session). Deliberately NOT a heartbeat: this hook measures, it does
/// not vouch for the session being alive.
fn minimal_record(session_id: &str, payload: &Map<String, Value>) -> Map<String, Value> {
    let mut record = Map::new();
    record.insert("id".into(), Value::String(session_id.to_string()));
    record.insert("started_at".into(), Value::String(now_iso()));
    if let Some(path) = field(payload, "transcript_path") {
        record.insert("transcript_path".into(), Value::String(path));
    }
    // The hook context carries no workspace classification and re-deriving one
    // costs a git walk per event; "main" is the default every unclassified
    // record already reads as.
    record.insert("workspace_id".into(), Value::String("main".into()));
    record.insert("source".into(), Value::String("activity".into()));
    record
}

/// Read-modify-write under the sessions store lock, tried ONCE: a busy lock
/// means another writer is mid-flight, and the next event (never more than a
/// tool call away) records the state anyway.
fn write_activity(
    ctx: &HookContext,
    ctrl: &Path,
    session_id: &str,
    activity: &Map<String, Value>,
    is_transition: bool,
) -> Result<(), String> {
    let dir = sessions_dir(ctrl);
    std::fs::create_dir_all(&dir).map_err(|e| format!("ensureDir({}): {e}", dir.display()))?;
    match acquire_store_lock_once(ctrl, "sessions") {
        AcquireOnce::Busy { .. } => Ok(()), // typed refusal, never a throw
        AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> Result<(), String> {
                // Re-read inside the lock so a concurrent heartbeat's fields
                // survive: only `activity` is ours to set.
                let mut record = read_session_failopen(ctrl, session_id)
                    .unwrap_or_else(|| minimal_record(session_id, &ctx.payload));
                record.insert("activity".into(), Value::Object(activity.clone()));
                write_json_atomic_retry(&session_file(ctrl, session_id), &Value::Object(record))?;
                if is_transition {
                    append_transition(ctrl, session_id, activity)?;
                }
                Ok(())
            })();
            guard.release();
            result
        }
    }
}

fn transitions_file(ctrl: &Path, session_id: &str) -> PathBuf {
    sessions_dir(ctrl).join(format!("{session_id}.activity.jsonl"))
}

fn append_transition(
    ctrl: &Path,
    session_id: &str,
    activity: &Map<String, Value>,
) -> Result<(), String> {
    let mut row = activity.clone();
    row.insert("session_id".into(), Value::String(session_id.to_string()));
    let file = transitions_file(ctrl, session_id);
    append_jsonl(&file, &Value::Object(row))
        .map_err(|e| format!("append {}: {e}", file.display()))?;
    trim_transitions(&file)
}

/// D2's window: the file never grows past the last 50 transitions, and the
/// rewrite is atomic (write a sibling temp, rename over) so a reader never
/// sees a half-file.
fn trim_transitions(file: &Path) -> Result<(), String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Ok(()); // unreadable: nothing to trim, and never an error here
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() <= MAX_TRANSITIONS {
        return Ok(());
    }
    let kept = &lines[lines.len() - MAX_TRANSITIONS..];
    let mut body = kept.join("\n");
    body.push('\n');
    let tmp = file.with_file_name(format!(
        "{}.{}.tmp",
        file.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    with_transient_fs_retry(|| std::fs::write(&tmp, &body).and_then(|()| std::fs::rename(&tmp, file)))
        .map_err(|e| format!("trim {}: {e}", file.display()))
}

// ── transient-FS retry (state_sync's posture, same window) ──────────────────

#[cfg(windows)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(4 | 5 | 32 | 33 | 145))
}

#[cfg(unix)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(code) if {
        code == libc::EBUSY || code == libc::EPERM || code == libc::ENOTEMPTY
            || code == libc::EMFILE || code == libc::ENFILE
    })
}

fn with_transient_fs_retry<T>(mut f: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut attempt = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if !is_transient_fs_error(&e) || attempt >= 15 {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
}

fn write_json_atomic_retry(file: &Path, value: &Value) -> Result<(), String> {
    with_transient_fs_retry(|| crate::fsutil::write_json_atomic(file, value))
        .map_err(|e| format!("write {}: {e}", file.display()))
}

// ── what this session is working on (D2, additive) ──────────────────────────

/// The feature this session is on: its OWN bound lane first — a lane-bound
/// session is working on that lane whatever the shared default says — and the
/// control root's default `state.json` feature only when it has none. A
/// missing or corrupt `state.json` reads as "unknown", the same fail-open
/// collapse every other display read here makes.
fn resolve_feature(ctrl: &Path, lane: Option<&str>) -> Option<String> {
    if let Some(lane) = lane.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(lane.to_string());
    }
    let ReadJson::Parsed(Value::Object(state)) = read_json(&ctrl.join(".bee").join("state.json"))
    else {
        return None;
    };
    state
        .get("feature")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// The cell this session is executing: the first ACTIVE claim in
/// `<control_root>/.bee/claims/` whose `session` is ours. Same shape and same
/// activity rule the worker rows of `bee status` use (`activeWorkers` /
/// `isClaimActive` in `verbs/status_full/cells.rs`), re-stated here because
/// those readers take a `Ctx` no hook has.
///
/// Cost is one readdir plus one small read per claim file — a claim record
/// already carries the cell id, so no cell file is ever opened.
fn resolve_cell(ctrl: &Path, session_id: &str) -> Option<String> {
    let Ok(entries) = std::fs::read_dir(ctrl.join(".bee").join("claims")) else {
        return None; // no claims directory at all: nothing is claimed, not an error
    };
    let now = now_ms();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if !well_formed_id(stem) {
            continue; // a filename readClaim's requireId would refuse
        }
        let ReadJson::Parsed(Value::Object(claim)) = read_json(&entry.path()) else {
            continue; // missing or corrupt: reads as no claim, never a failure
        };
        if claim.get("session").and_then(Value::as_str).map(str::trim) != Some(session_id) {
            continue;
        }
        if !claim_is_active(&claim, now) {
            continue; // expired: the session no longer holds this cell
        }
        return claim
            .get("cell")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
    }
    None
}

/// `isClaimActive`: a claim without a usable TTL, or without a usable
/// `claimed_at`, is active — only a claim whose own numbers say it expired is
/// dropped.
fn claim_is_active(claim: &Map<String, Value>, now_ms: f64) -> bool {
    let Some(ttl) = claim.get("ttl_seconds").and_then(Value::as_f64) else {
        return true;
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return true;
    }
    let Some(claimed) = claim.get("claimed_at").and_then(Value::as_str).and_then(iso_to_ms) else {
        return true;
    };
    claimed + ttl * 1000.0 > now_ms
}

fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(f64::NAN)
}

/// Date.parse for the ISO shape bee's own writers emit.
fn iso_to_ms(s: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.timestamp_millis() as f64)
}

// ── the waiting mark (D5) ───────────────────────────────────────────────────

fn err2_text(e: Err2) -> String {
    match e {
        Err2::Msg(m) => m,
        Err2::Ex => "an exotic record shape this hook does not model".to_string(),
    }
}

/// Returns the marker's new value when it changed, `None` when nothing about
/// the mark moved.
#[allow(clippy::too_many_arguments)]
fn sync_waiting_on(
    root: &Path,
    session_id: &str,
    event: &str,
    target: State,
    is_transition: bool,
    marker: bool,
    tool_name: Option<&str>,
    message: Option<&str>,
) -> Result<Option<bool>, String> {
    if is_turn_boundary(event) {
        // The hook clears ONLY what the hook set — an agent's own pending
        // question is not this hook's to end.
        if !marker {
            return Ok(None);
        }
        clear_mark(root)?;
        return Ok(Some(false));
    }
    if !is_transition {
        return Ok(None);
    }
    let (kind, subject) = match target {
        State::WaitingInput => (
            "question",
            message.unwrap_or("agent needs input").to_string(),
        ),
        State::Blocked => (
            "gate",
            tool_name.or(message).unwrap_or("permission prompt").to_string(),
        ),
        _ => return Ok(None),
    };
    if set_mark(root, session_id, kind, &subject, marker)? {
        return Ok(Some(true));
    }
    Ok(None)
}

/// `.bee/state.json`'s own `waiting_on`, read fail-open (a missing or corrupt
/// file reads as "no mark", exactly what `read_state_peek` collapses to).
fn default_state_waiting_on(root: &Path) -> Option<Value> {
    match read_json(&root.join(".bee").join("state.json")) {
        ReadJson::Parsed(Value::Object(m)) => m.get("waiting_on").cloned(),
        _ => None,
    }
}

/// `false` when an agent-set mark is standing and the hook stepped back.
fn set_mark(
    root: &Path,
    session_id: &str,
    kind: &str,
    subject: &str,
    marker: bool,
) -> Result<bool, String> {
    let target = resolve_waiting_on_target(root, WAITING_ON_VERB, None, false, Some(session_id))
        .map_err(err2_text)?;
    let live = match &target {
        Some((id, _)) => read_workflow_record(root, id)
            .ok()
            .and_then(|record| record.get("waiting_on").cloned()),
        None => default_state_waiting_on(root),
    };
    // D5's one hard rule: a live gate/question the hook did not set is the
    // agent's own pending ask, and the hook never speaks over it.
    if !marker && waiting_on_is_live(live.as_ref()) {
        let existing_kind = live
            .as_ref()
            .and_then(|v| v.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if existing_kind == "gate" || existing_kind == "question" {
            return Ok(false);
        }
    }
    match target {
        Some((id, feature)) => {
            set_workflow_waiting_on(root, &id, kind, subject, session_id).map_err(err2_text)?;
            rebuild_lane_projection(root, &feature).map_err(err2_text)?;
            rebuild_state_projection(root).map_err(err2_text)?;
        }
        None => {
            set_default_state_waiting_on(root, kind, subject, session_id).map_err(err2_text)?;
        }
    }
    Ok(true)
}

fn clear_mark(root: &Path) -> Result<(), String> {
    let target = resolve_waiting_on_target(root, WAITING_ON_VERB, None, false, None)
        .map_err(err2_text)?;
    match target {
        Some((id, feature)) => {
            clear_workflow_waiting_on(root, &id).map_err(err2_text)?;
            rebuild_lane_projection(root, &feature).map_err(err2_text)?;
            rebuild_state_projection(root).map_err(err2_text)?;
        }
        None => {
            clear_default_state_waiting_on(root).map_err(err2_text)?;
        }
    }
    Ok(())
}

// ── the hook's own capped log (D1) ──────────────────────────────────────────

/// One line per failure, in `.bee/logs/hook-activity.log`. Capped by cutting
/// the file back to its last half at a line boundary — the oldest lines are
/// the least useful, and an unbounded log in a passive hook is a disk leak.
/// Every failure here is swallowed: logging never changes a hook's exit code.
fn append_activity_log(root: &Path, message: &str) {
    let logs_dir = root.join(".bee").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return;
    }
    let file = logs_dir.join("hook-activity.log");
    if std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0) > LOG_CAP_BYTES {
        cap_log(&file);
    }
    use std::io::Write;
    if let Ok(mut handle) = std::fs::OpenOptions::new().create(true).append(true).open(&file) {
        let _ = handle.write_all(format!("{} {}\n", now_iso(), message.replace('\n', " ")).as_bytes());
    }
}

fn cap_log(file: &Path) {
    let Ok(text) = std::fs::read_to_string(file) else {
        let _ = std::fs::remove_file(file); // unreadable bytes: start clean
        return;
    };
    let cut = text.len() / 2;
    let keep = match text[cut..].find('\n') {
        Some(offset) => &text[cut + offset + 1..],
        None => "",
    };
    let _ = std::fs::write(file, keep);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Repo {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    fn repo() -> Repo {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        Repo { _dir: dir, root }
    }

    /// Drive the hook exactly as the dispatcher does: argv tail + raw stdin.
    fn fire(repo: &Repo, payload: Value) -> ExitCode {
        let mut payload = payload;
        payload["cwd"] = Value::String(repo.root.to_string_lossy().into_owned());
        run_inner(&[], &payload.to_string())
    }

    fn event(name: &str, session: &str) -> Value {
        serde_json::json!({ "hook_event_name": name, "session_id": session })
    }

    fn record(repo: &Repo, session: &str) -> Map<String, Value> {
        match read_json(&session_file(&repo.root, session)) {
            ReadJson::Parsed(Value::Object(m)) => m,
            _ => panic!("no readable session record for {session}"),
        }
    }

    fn activity(repo: &Repo, session: &str) -> Map<String, Value> {
        record(repo, session)
            .get("activity")
            .and_then(Value::as_object)
            .cloned()
            .expect("the record carries an activity block")
    }

    fn state_of(repo: &Repo, session: &str) -> String {
        activity(repo, session).get("state").and_then(Value::as_str).unwrap().to_string()
    }

    fn transitions(repo: &Repo, session: &str) -> Vec<Value> {
        let file = transitions_file(&repo.root, session);
        let Ok(text) = std::fs::read_to_string(&file) else {
            return Vec::new();
        };
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn ok(code: ExitCode) {
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
    }

    // ── mapping table (D3), as a pure unit ─────────────────────────────────

    #[test]
    fn activity_pane_prefers_herdr_when_both_transports_name_a_pane() {
        let got = pane_from_env(|key| match key {
            "HERDR_PANE_ID" => Some("herdr-3".to_string()),
            "TMUX_PANE" => Some("%5".to_string()),
            _ => None,
        });
        assert_eq!(got, Some(("herdr".to_string(), "herdr-3".to_string())));
    }

    #[test]
    fn activity_pane_reads_tmux_when_only_tmux_names_a_pane() {
        let got = pane_from_env(|key| match key {
            "TMUX_PANE" => Some("%5".to_string()),
            _ => None,
        });
        assert_eq!(got, Some(("tmux".to_string(), "%5".to_string())));
    }

    #[test]
    fn activity_pane_is_absent_when_neither_transport_names_a_pane() {
        assert_eq!(pane_from_env(|_| None), None);
    }

    #[test]
    fn activity_pane_counts_an_empty_or_blank_value_as_unset() {
        // A herdr variable exported empty is not a pane: tmux still answers.
        let got = pane_from_env(|key| match key {
            "HERDR_PANE_ID" => Some("   ".to_string()),
            "TMUX_PANE" => Some(" %9 ".to_string()),
            _ => None,
        });
        assert_eq!(got, Some(("tmux".to_string(), "%9".to_string())));
        assert_eq!(pane_from_env(|_| Some(String::new())), None);
    }

    #[test]
    fn the_event_table_maps_every_decided_event_and_nothing_else() {
        assert_eq!(map_event("UserPromptSubmit", None, None), Some(State::Working));
        assert_eq!(map_event("PreToolUse", None, None), Some(State::Working));
        assert_eq!(map_event("PostToolUse", None, None), Some(State::Working));
        assert_eq!(map_event("PostToolUseFailure", None, None), Some(State::Working));
        assert_eq!(map_event("PermissionRequest", None, None), Some(State::Blocked));
        assert_eq!(map_event("Stop", None, None), Some(State::Idle));
        assert_eq!(
            map_event("Notification", Some("permission_prompt"), None),
            Some(State::Blocked)
        );
        assert_eq!(
            map_event("Notification", Some("agent_needs_input"), None),
            Some(State::WaitingInput)
        );
        assert_eq!(map_event("Notification", Some("idle_prompt"), None), Some(State::Idle));
        assert_eq!(map_event("Notification", Some("agent_completed"), None), Some(State::Idle));
        assert_eq!(map_event("Notification", Some("something-else"), None), None);
        assert_eq!(map_event("SessionEnd", None, Some("other")), Some(State::Exited));
        assert_eq!(map_event("SessionEnd", None, Some("clear")), None);
        assert_eq!(map_event("SessionEnd", None, Some("resume")), None);
        // Never SubagentStop — a subagent finishing says nothing about the
        // session's own agent (CONTEXT.md, out of scope).
        assert_eq!(map_event("SubagentStop", None, None), None);
        assert_eq!(map_event("PreCompact", None, None), None);
    }

    // ── happy path ─────────────────────────────────────────────────────────

    #[test]
    fn a_user_prompt_records_working_and_one_transition() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(state_of(&repo, "s1"), "working");
        let rows = transitions(&repo, "s1");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["session_id"], Value::String("s1".into()));
        assert_eq!(rows[0]["event"], Value::String("UserPromptSubmit".into()));
        assert_eq!(
            activity(&repo, "s1").get("cwd").and_then(Value::as_str).unwrap(),
            repo.root.to_string_lossy()
        );
    }

    #[test]
    fn a_repeat_of_the_same_state_refreshes_the_record_but_is_not_a_transition() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PreToolUse", "session_id": "s1", "tool_name": "Read"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "working");
        assert_eq!(
            activity(&repo, "s1").get("event").and_then(Value::as_str),
            Some("PreToolUse"),
            "the record still follows the newest event"
        );
        assert_eq!(transitions(&repo, "s1").len(), 1, "working -> working is not history");
    }

    #[test]
    fn a_permission_request_blocks_and_records_the_tool_it_blocked_on() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        let a = activity(&repo, "s1");
        assert_eq!(a["state"], Value::String("blocked".into()));
        assert_eq!(a["tool_name"], Value::String("Bash".into()));
        assert_eq!(a["tool_use_id"], Value::String("call-1".into()));
        assert_eq!(transitions(&repo, "s1").len(), 1);
    }

    #[test]
    fn the_matching_post_tool_use_clears_the_block() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PostToolUse", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "working");
        assert_eq!(transitions(&repo, "s1").len(), 2);
    }

    // ── the sticky rules (D3) ──────────────────────────────────────────────

    #[test]
    fn a_post_tool_use_with_a_different_tool_use_id_leaves_the_block_standing() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PostToolUse", "session_id": "s1",
                "tool_name": "Read", "tool_use_id": "call-2"
            }),
        ));
        let a = activity(&repo, "s1");
        assert_eq!(a["state"], Value::String("blocked".into()));
        assert_eq!(a["tool_use_id"], Value::String("call-1".into()), "the block is untouched");
        assert_eq!(transitions(&repo, "s1").len(), 1, "a suppressed event appends nothing");
    }

    #[test]
    fn without_a_recorded_tool_use_id_the_tool_name_clears_the_block() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1", "tool_name": "Bash"
            }),
        ));
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PostToolUse", "session_id": "s1", "tool_name": "Read"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "blocked", "a different tool does not clear it");
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PostToolUse", "session_id": "s1", "tool_name": "Bash"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "working");
    }

    #[test]
    fn a_turn_boundary_always_clears_a_block() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(state_of(&repo, "s1"), "working");
    }

    #[test]
    fn waiting_input_survives_tool_traffic_and_ends_only_at_a_turn_boundary() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "Notification", "session_id": "s1",
                "notification_type": "agent_needs_input", "message": "Which branch?"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "waiting_input");
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PreToolUse", "session_id": "s1", "tool_name": "Read"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "waiting_input", "tool traffic does not answer a question");
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "Notification", "session_id": "s1",
                "notification_type": "idle_prompt"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "waiting_input", "nor does an idle notification");
        ok(fire(&repo, event("Stop", "s1")));
        assert_eq!(state_of(&repo, "s1"), "idle");
    }

    #[test]
    fn a_permission_prompt_overrides_waiting_input() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "Notification", "session_id": "s1",
                "notification_type": "agent_needs_input"
            }),
        ));
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1", "tool_name": "Bash"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "blocked");
    }

    // ── SessionEnd ─────────────────────────────────────────────────────────

    #[test]
    fn session_end_marks_exited_but_clear_and_resume_do_not() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        for reason in ["clear", "resume"] {
            ok(fire(
                &repo,
                serde_json::json!({
                    "hook_event_name": "SessionEnd", "session_id": "s1", "reason": reason
                }),
            ));
            assert_eq!(state_of(&repo, "s1"), "working", "reason {reason} is not an exit");
        }
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "SessionEnd", "session_id": "s1", "reason": "exit"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "exited");
    }

    #[test]
    fn subagent_stop_is_never_handled() {
        let repo = repo();
        ok(fire(&repo, event("SubagentStop", "s1")));
        assert!(
            !session_file(&repo.root, "s1").exists(),
            "SubagentStop must not even create a record"
        );
    }

    // ── D2 store shape ─────────────────────────────────────────────────────

    #[test]
    fn a_missing_session_record_is_created_minimally() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "UserPromptSubmit", "session_id": "s1",
                "transcript_path": "/tmp/t.jsonl"
            }),
        ));
        let r = record(&repo, "s1");
        assert_eq!(r["id"], Value::String("s1".into()));
        assert!(r.contains_key("started_at"));
        assert_eq!(r["transcript_path"], Value::String("/tmp/t.jsonl".into()));
        assert_eq!(r["workspace_id"], Value::String("main".into()));
        assert_eq!(r["source"], Value::String("activity".into()));
    }

    #[test]
    fn an_existing_record_keeps_its_own_fields() {
        let repo = repo();
        let dir = sessions_dir(&repo.root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("s1.json"),
            serde_json::json!({ "id": "s1", "last_heartbeat": "2026-08-22T00:00:00.000Z" })
                .to_string(),
        )
        .unwrap();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        let r = record(&repo, "s1");
        assert_eq!(r["last_heartbeat"], Value::String("2026-08-22T00:00:00.000Z".into()));
        assert!(r.contains_key("activity"));
    }

    #[test]
    fn the_fifty_first_transition_trims_the_file_to_the_last_fifty() {
        let repo = repo();
        // Alternate blocked/working with matching ids so every event is a real
        // transition: 60 events -> 60 transitions -> trimmed to 50.
        for n in 0..30 {
            ok(fire(
                &repo,
                serde_json::json!({
                    "hook_event_name": "PermissionRequest", "session_id": "s1",
                    "tool_name": "Bash", "tool_use_id": format!("call-{n}")
                }),
            ));
            ok(fire(
                &repo,
                serde_json::json!({
                    "hook_event_name": "PostToolUse", "session_id": "s1",
                    "tool_name": "Bash", "tool_use_id": format!("call-{n}")
                }),
            ));
        }
        let rows = transitions(&repo, "s1");
        assert_eq!(rows.len(), MAX_TRANSITIONS);
        assert_eq!(
            rows.last().unwrap()["tool_use_id"],
            Value::String("call-29".into()),
            "the newest transition survives the trim"
        );
        assert_eq!(
            rows[0]["tool_use_id"],
            Value::String("call-5".into()),
            "the oldest kept row is exactly 50 back"
        );
    }

    // ── adversarial stdin (D1) ─────────────────────────────────────────────

    #[test]
    fn empty_garbage_and_huge_stdin_exit_zero_and_leave_a_log_line() {
        let repo = repo();
        let cwd = repo.root.to_string_lossy().into_owned();
        let huge = serde_json::json!({ "cwd": cwd, "blob": "a".repeat(64 * 1024) }).to_string();
        for raw in ["", "not json at all {{{", "null", "[]", &huge] {
            ok(run_inner(&[], raw));
        }
        // The rows without a cwd resolve no root, so only the huge one logs;
        // fire the empty case from inside the repo explicitly.
        ok(fire(&repo, serde_json::json!({ "hook_event_name": "PreToolUse" })));
        let log = std::fs::read_to_string(repo.root.join(".bee").join("logs").join("hook-activity.log"))
            .expect("the hook wrote its own log");
        assert!(log.contains("no session_id"), "the log names why nothing was recorded: {log}");
        assert!(
            !sessions_dir(&repo.root).join("s1.json").exists(),
            "an unusable payload writes no session record"
        );
    }

    #[test]
    fn a_session_id_with_path_separators_is_refused_and_logged() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "../escape")));
        let log = std::fs::read_to_string(repo.root.join(".bee").join("logs").join("hook-activity.log"))
            .unwrap();
        assert!(log.contains("plain id"), "got: {log}");
    }

    #[test]
    fn the_log_is_capped_to_its_last_half() {
        let repo = repo();
        let logs = repo.root.join(".bee").join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        let file = logs.join("hook-activity.log");
        let line = format!("{}\n", "x".repeat(255));
        std::fs::write(&file, line.repeat(2000)).unwrap(); // ~512 KiB
        append_activity_log(&repo.root, "after the cap");
        let size = std::fs::metadata(&file).unwrap().len();
        assert!(size < LOG_CAP_BYTES, "the log was cut back, got {size} bytes");
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.ends_with("after the cap\n"));
        assert!(
            text.lines().all(|l| !l.is_empty()),
            "the cut lands on a line boundary, never mid-line"
        );
    }

    // ── what this session is working on (D2, additive) ─────────────────────

    /// Put a session record on disk before the hook ever sees the session.
    fn seed_session(repo: &Repo, session: &str, record: Value) {
        let dir = sessions_dir(&repo.root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{session}.json")), record.to_string()).unwrap();
    }

    fn seed_claim(repo: &Repo, cell: &str, claim: Value) {
        let dir = repo.root.join(".bee").join("claims");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{cell}.json")), claim.to_string()).unwrap();
    }

    fn iso_ago(seconds: i64) -> String {
        let then = chrono::Utc::now() - chrono::Duration::seconds(seconds);
        then.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    #[test]
    fn a_lane_bound_session_stamps_its_lane_as_the_feature() {
        let repo = repo();
        std::fs::write(
            repo.root.join(".bee").join("state.json"),
            serde_json::json!({ "feature": "the-default-feature" }).to_string(),
        )
        .unwrap();
        seed_session(&repo, "s1", serde_json::json!({ "id": "s1", "lane": "my-lane" }));
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(
            activity(&repo, "s1").get("feature"),
            Some(&Value::String("my-lane".into())),
            "the session's own lane outranks the shared default"
        );
        assert_eq!(
            transitions(&repo, "s1")[0]["feature"],
            Value::String("my-lane".into()),
            "the sidecar row carries the same stamp"
        );
    }

    #[test]
    fn an_unbound_session_falls_back_to_the_default_states_feature() {
        let repo = repo();
        std::fs::write(
            repo.root.join(".bee").join("state.json"),
            serde_json::json!({ "feature": "the-default-feature" }).to_string(),
        )
        .unwrap();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(
            activity(&repo, "s1").get("feature"),
            Some(&Value::String("the-default-feature".into()))
        );
    }

    #[test]
    fn no_lane_and_no_state_file_stamps_no_feature_at_all() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert!(
            !activity(&repo, "s1").contains_key("feature"),
            "unknown is an absent key, never a guess"
        );
    }

    #[test]
    fn an_active_claim_held_by_this_session_stamps_its_cell() {
        let repo = repo();
        seed_claim(
            &repo,
            "aah-4",
            serde_json::json!({
                "cell": "aah-4", "session": "s1",
                "claimed_at": iso_ago(10), "ttl_seconds": 3600
            }),
        );
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(activity(&repo, "s1").get("cell"), Some(&Value::String("aah-4".into())));
        assert_eq!(transitions(&repo, "s1")[0]["cell"], Value::String("aah-4".into()));
    }

    #[test]
    fn an_expired_or_foreign_claim_stamps_no_cell() {
        let repo = repo();
        seed_claim(
            &repo,
            "aah-9",
            serde_json::json!({
                "cell": "aah-9", "session": "s1",
                "claimed_at": iso_ago(7200), "ttl_seconds": 60
            }),
        );
        seed_claim(
            &repo,
            "aah-8",
            serde_json::json!({
                "cell": "aah-8", "session": "someone-else",
                "claimed_at": iso_ago(10), "ttl_seconds": 3600
            }),
        );
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert!(
            !activity(&repo, "s1").contains_key("cell"),
            "an expired claim is not work in flight, and another session's claim is not ours"
        );
    }

    #[test]
    fn a_claim_without_a_ttl_still_counts_as_held() {
        let repo = repo();
        seed_claim(
            &repo,
            "aah-4",
            serde_json::json!({ "cell": "aah-4", "session": "s1" }),
        );
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(activity(&repo, "s1").get("cell"), Some(&Value::String("aah-4".into())));
    }

    #[test]
    fn a_missing_claims_dir_or_an_unreadable_claim_never_blocks_the_state_write() {
        let repo = repo();
        // No claims directory at all.
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert_eq!(state_of(&repo, "s1"), "working");
        assert!(!activity(&repo, "s1").contains_key("cell"));

        // A corrupt claim, and a filename no plain-id check would accept.
        let dir = repo.root.join(".bee").join("claims");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("broken.json"), "{ not json at all").unwrap();
        std::fs::write(dir.join("..odd.json"), "{}").unwrap();
        ok(fire(&repo, event("Stop", "s2")));
        assert_eq!(state_of(&repo, "s2"), "idle", "the activity write still lands");
        assert!(!activity(&repo, "s2").contains_key("cell"));
    }

    #[test]
    fn picking_up_a_cell_without_changing_state_refreshes_but_is_not_a_transition() {
        let repo = repo();
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        seed_claim(
            &repo,
            "aah-4",
            serde_json::json!({
                "cell": "aah-4", "session": "s1",
                "claimed_at": iso_ago(1), "ttl_seconds": 3600
            }),
        );
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PreToolUse", "session_id": "s1", "tool_name": "Read"
            }),
        ));
        assert_eq!(activity(&repo, "s1").get("cell"), Some(&Value::String("aah-4".into())));
        assert_eq!(transitions(&repo, "s1").len(), 1, "working -> working is still not history");
    }

    // ── the waiting mark (D5) ──────────────────────────────────────────────

    fn state_waiting_on(repo: &Repo) -> Option<Value> {
        default_state_waiting_on(&repo.root)
    }

    #[test]
    fn waiting_input_sets_a_question_mark_and_a_turn_boundary_clears_it() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "Notification", "session_id": "s1",
                "notification_type": "agent_needs_input", "message": "Which branch?"
            }),
        ));
        let mark = state_waiting_on(&repo).expect("the hook set a mark");
        assert_eq!(mark["kind"], Value::String("question".into()));
        assert_eq!(mark["subject"], Value::String("Which branch?".into()));
        assert_eq!(mark["session"], Value::String("s1".into()));
        assert_eq!(
            activity(&repo, "s1").get("waiting_on_set_by_hook"),
            Some(&Value::Bool(true))
        );

        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        assert!(!waiting_on_is_live(state_waiting_on(&repo).as_ref()), "the hook cleared its mark");
        assert!(
            !activity(&repo, "s1").contains_key("waiting_on_set_by_hook"),
            "and dropped its marker"
        );
    }

    #[test]
    fn a_block_sets_a_gate_mark_naming_the_tool() {
        let repo = repo();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1",
                "tool_name": "Bash", "tool_use_id": "call-1"
            }),
        ));
        let mark = state_waiting_on(&repo).expect("the hook set a mark");
        assert_eq!(mark["kind"], Value::String("gate".into()));
        assert_eq!(mark["subject"], Value::String("Bash".into()));
    }

    #[test]
    fn an_agent_set_mark_is_never_overwritten_or_cleared_by_the_hook() {
        let repo = repo();
        set_default_state_waiting_on(&repo.root, "question", "the agent's own ask", "s1").unwrap();
        ok(fire(
            &repo,
            serde_json::json!({
                "hook_event_name": "PermissionRequest", "session_id": "s1", "tool_name": "Bash"
            }),
        ));
        assert_eq!(state_of(&repo, "s1"), "blocked", "the activity state is still recorded");
        let mark = state_waiting_on(&repo).expect("the agent's mark stands");
        assert_eq!(mark["subject"], Value::String("the agent's own ask".into()));
        assert!(
            !activity(&repo, "s1").contains_key("waiting_on_set_by_hook"),
            "the hook never claims a mark it did not set"
        );

        // …and a turn boundary leaves it alone too.
        ok(fire(&repo, event("UserPromptSubmit", "s1")));
        let mark = state_waiting_on(&repo).expect("still the agent's mark");
        assert_eq!(mark["subject"], Value::String("the agent's own ask".into()));
    }
}
