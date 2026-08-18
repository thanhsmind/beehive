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
// Every corruption is handled at its real read, warning once in bee's own
// words and taking the fallback an absent file would get: state.json reads
// as default_state(), HANDOFF.json reads as absent (so the mid-phase "hive
// door open" warning still fires), a corrupt inject cache falls through to
// the legacy location and then to `{}` (the nudge re-injects), a corrupt
// session record reads as no session, a corrupt lane still takes its second
// warn and the LANE_CORRUPT refusal, a corrupt cell is skipped from the
// claimed list, and a corrupt perf meta is skipped. The hook still exits 0.
//
// Warnings are QUEUED, not printed (see queue_corrupt_json_warning): the hook
// buffers all output so that a run which later delegates has emitted nothing.
//
// Delegate bails (Outcome::Delegate), all decided BEFORE any write/output:
//   - event == "PreCompact": the compaction path (intent anchor re-assert,
//     compaction record, forced nudges) delegates wholesale.
//   - an inject cache whose parsed content is a non-object, and a truthy
//     non-object approved_gates (both JS spread/assignment exotica).
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


use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};

use crate::hooks::Outcome;

use crate::lock::AcquireOnce;

use crate::jsjson::js_to_string;

use crate::textutil::truncate_chars_head;

use serde_json::{Map, Value};


use std::path::Path;

use std::process::ExitCode;

const HOOK_NAME: &str = "session-close";

const INJECT_INTERVAL_MS: f64 = 30.0 * 60.0 * 1000.0;

const DECISION_RECENT_MS: f64 = 6.0 * 3600.0 * 1000.0;

/// auto-wait-mark D6: the truncation cap on a `turn-end` mark's `subject` —
/// chosen so the line still renders on ONE line on both the session preamble
/// and the compact capsule (D6's agent-discretion constraint). Char-counted
/// (`truncate_chars_head`, the crate's one front-of-string primitive), not
/// byte-counted, so astral-plane input never lands mid-character.
const TURN_END_SUBJECT_MAX_CHARS: usize = 140;

/// auto-wait-mark D6: `build_waiting_on` refuses an empty `subject`, so a
/// turn whose final assistant text block is missing, empty, or
/// whitespace-only still needs a non-empty fallback — this is it.
const TURN_END_FALLBACK_SUBJECT: &str = "(turn ended)";

/// Internal control flow: Delegate = re-run the Node wrapper; Crash(msg) =
/// the Node path would THROW here (main's catch → logCrash → advisory emit).
#[derive(Debug)]
pub(crate) enum Flow {
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
    // SessionEnd: native-only (the Node wrapper never wired this event —
    // Stop/PreCompact/SubagentStop were its whole surface). Best-effort marks
    // the session record closed under the sessions store lock so a cleanly
    // exited session stops counting as live before its heartbeat goes stale;
    // fails open on a busy lock, a missing/corrupt record, or a mismatched
    // id. Never prints, never delegates.
    if ctx.event == "SessionEnd" {
        close_session_record(&root, &ctx);
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
    // auto-wait-mark rework: `perf_refresh` hands back the transcript events
    // it already resolved and read (`Some`, possibly empty, when a
    // transcript resolved at all) so the turn-end mark below reuses them
    // instead of a second `read_jsonl` of the same file.
    let perf_events = match perf_refresh(&root, session_id.as_deref()) {
        Ok(events) => events,
        Err(msg) => {
            log_crash(Some(&root), HOOK_NAME, &msg, Some("perf-refresh"));
            None
        }
    };

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
        Ok(AdvisoryOutcome::Done) => {
            // auto-wait-mark D1: `advisory()` only ever returns `Done` after
            // the GitHub #18 bypass net (`maybe_bypass_block`) decided NOT to
            // block — i.e. control genuinely returns to the human this turn,
            // never a forced continuation. `Disabled` and `Block` both
            // `return` their own branch above and never reach here.
            if ctx.event == "Stop" {
                set_turn_end_mark_best_effort(&root, &ctx, session_id.as_deref(), perf_events);
            }
        }
        Err(Flow::Delegate) => return Err(()),
        Err(Flow::Crash(msg)) => {
            log_crash(Some(&root), HOOK_NAME, &msg, ctx.source);
            // Best-effort, same principle as `prompt_context.rs`'s clear
            // path: the Stop event genuinely fired regardless of an
            // unrelated pipeline crash (e.g. a corrupt lane record) inside
            // advisory()'s own nudges, so the mark still gets its chance.
            if ctx.event == "Stop" {
                set_turn_end_mark_best_effort(&root, &ctx, session_id.as_deref(), perf_events);
            }
        }
    }

    flush(&stdout, &stderr);
    if !parts.is_empty() {
        emit_hook_output(&ctx, &parts.join("\n"), "Stop");
    }
    Ok(())
}

/// Marks `.bee/sessions/<id>.json` closed: `status: "closed"` plus
/// `closed_at: <now ISO>`. Mirrors state_sync.rs's `heartbeat_session` for
/// the lock/read/write shape (acquire the "sessions" store lock once, read
/// fail-open, write atomically), but never crash-logs and never touches the
/// exit code or output — a SessionEnd hook has nothing left to report to.
fn close_session_record(root: &Path, ctx: &HookContext) {
    let Some(session_id) = get_session_id(&ctx.payload) else { return };
    if !is_plain_id(&session_id) {
        return;
    }
    let ctl = control_root(root, ctx);
    let mut guard = match crate::lock::acquire_store_lock_once(&ctl, "sessions") {
        AcquireOnce::Busy { .. } => return,
        AcquireOnce::Acquired(guard) => guard,
    };
    let file = ctl.join(".bee").join("sessions").join(format!("{session_id}.json"));
    let session = match crate::fsutil::read_json(&file) {
        crate::fsutil::ReadJson::Parsed(Value::Object(m))
            if m.get("id").and_then(Value::as_str) == Some(session_id.as_str()) =>
        {
            m
        }
        _ => {
            guard.release();
            return;
        }
    };
    let mut session = session;
    session.insert("status".to_string(), Value::String("closed".to_string()));
    session.insert("closed_at".to_string(), Value::String(now_iso()));
    let _ = crate::fsutil::write_json_atomic(&file, &Value::Object(session));
    guard.release();
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

// ─── auto-wait-mark D1: the Stop-only turn-end mark ────────────────────────
//
// D1: a Stop event where control has provably returned to the human (the
// bypass net above did not force a continuation) gets a `waiting_on` mark of
// kind `turn-end` — no text heuristic, Stop itself is the whole signal. D2:
// the same store setters an agent-declared mark uses, so the record carries
// no provenance. D5: never overwrites a live mark — checked at the exact
// target `resolve_waiting_on_target` (D3) resolves, via the store's own
// `waiting_on_is_live`, never a second predicate. Best-effort throughout,
// mirroring `prompt_context.rs`'s `clear_and_reap_waiting_on_best_effort`
// shape: a failed write is logged and never fails the hook or its output.
fn set_turn_end_mark_best_effort(
    root: &Path,
    ctx: &HookContext,
    session_id: Option<&str>,
    cached_events: Option<Vec<Value>>,
) {
    // `build_waiting_on` refuses an empty session, and D4's stale-expiry
    // reap has nothing to reclaim against without one — a Stop whose session
    // id cannot be resolved writes nothing rather than failing.
    let Some(session) = session_id else { return };
    if let Err(msg) = try_set_turn_end_mark(root, session, cached_events) {
        log_crash(Some(root), HOOK_NAME, &msg, ctx.source);
    }
}

fn try_set_turn_end_mark(
    root: &Path,
    session: &str,
    cached_events: Option<Vec<Value>>,
) -> Result<(), String> {
    // A missing, unreadable, or malformed transcript (no session-scoped
    // .jsonl resolves, or it carries zero parseable events) writes no mark
    // at all — distinct from a transcript that resolves fine but whose
    // FINAL assistant text happens to be blank, which still writes (with
    // the fallback subject, below `turn_end_subject`'s `Some` branch).
    let Some(subject) = turn_end_subject(root, Some(session), cached_events) else {
        return Ok(());
    };
    // D3's own target resolution (session-scoped first, feature-scoped only
    // when a feature is live) — the SAME resolver `state waiting-on set`
    // calls, never a second one (R128).
    let target = crate::verbs::state_group::resolve_waiting_on_target(
        root,
        "hooks stop",
        None,
        false,
        Some(session),
    )
    .map_err(waiting_on_err_message)?;
    match target {
        Some((id, feature)) => {
            let current =
                crate::verbs::workflow_store::read_workflow_record(root, &id).map_err(|e| e.0)?;
            if crate::verbs::workflow_store::waiting_on_is_live(current.get("waiting_on")) {
                return Ok(()); // D5: an agent-declared wait stands untouched
            }
            crate::verbs::workflow_store::set_workflow_waiting_on(
                root,
                &id,
                crate::verbs::workflow_store::WAITING_ON_KIND_TURN_END,
                &subject,
                session,
            )
            .map_err(waiting_on_err_message)?;
            crate::verbs::workflow_store::rebuild_lane_projection(root, &feature)
                .map_err(waiting_on_err_message)?;
            crate::verbs::workflow_store::rebuild_state_projection(root)
                .map_err(waiting_on_err_message)?;
        }
        None => {
            let current = crate::verbs::state_group::read_state_peek(root).map_err(|_| {
                "waiting_on turn-end: could not peek the default state record (exotic value)"
                    .to_string()
            })?;
            if crate::verbs::workflow_store::waiting_on_is_live(current.get("waiting_on")) {
                return Ok(()); // D5
            }
            crate::verbs::state_group::set_default_state_waiting_on(
                root,
                crate::verbs::workflow_store::WAITING_ON_KIND_TURN_END,
                &subject,
                session,
            )
            .map_err(waiting_on_err_message)?;
        }
    }
    Ok(())
}

fn waiting_on_err_message(e: crate::verbs::reservations::Err2) -> String {
    match e {
        crate::verbs::reservations::Err2::Msg(m) => m,
        crate::verbs::reservations::Err2::Ex => {
            "waiting_on turn-end: an exotic (unmodelable) record value".to_string()
        }
    }
}

/// D6: the `subject` a `turn-end` mark carries — the last non-empty line of
/// the turn's final assistant text block, trimmed and truncated. Reuses
/// `perf.rs`'s own `resolve_transcript_for`, already resolved (and its file
/// read) on every Stop for the perf rollup — no second resolution path, no
/// second transcript read: `cached_events` carries the SAME events
/// `perf_refresh` already parsed for this Stop (`Some`, possibly empty, once
/// a transcript resolved at all), and this function reuses them instead of
/// reading the file again. `None` is only passed by a caller outside the
/// Stop dispatch (there is none today; kept so a direct/test call without a
/// prior `perf_refresh` still resolves and reads for itself, exactly as
/// before).
///
/// `None` is the "nothing to report" case — a missing, unreadable, or
/// malformed transcript (`resolve_transcript_for` finds nothing, or the file
/// carries zero parseable JSONL events) — and the caller writes no mark at
/// all for it. `Some` is returned once the transcript itself is usable, even
/// when the specific last line it yields is empty or whitespace-only:
/// `build_waiting_on` refuses an empty subject, so that case still resolves
/// to `TURN_END_FALLBACK_SUBJECT` rather than `None` — the transcript did
/// its job, there was just nothing to quote.
fn turn_end_subject(
    root: &Path,
    session_id: Option<&str>,
    cached_events: Option<Vec<Value>>,
) -> Option<String> {
    let events = match cached_events {
        Some(events) => events,
        None => {
            let path = resolve_transcript_for(root, session_id)?;
            read_jsonl(&path)
        }
    };
    if events.is_empty() {
        return None;
    }
    let line = final_assistant_text_line(&events).unwrap_or_default();
    let trimmed = line.trim();
    Some(if trimmed.is_empty() {
        TURN_END_FALLBACK_SUBJECT.to_string()
    } else {
        truncate_chars_head(trimmed, TURN_END_SUBJECT_MAX_CHARS)
    })
}

/// Scans backward for the last assistant transcript entry that actually
/// carries a text block — a turn's final assistant entry is often a bare
/// `tool_use` with no text at all — then returns THAT block's own last
/// non-empty line. An entry with content but no text block is skipped
/// entirely (never falls back to an earlier block within it); an entry
/// whose final text block is present but blank stops the search there too
/// (empty is a valid finding, not a reason to keep scanning further back).
fn final_assistant_text_line(events: &[Value]) -> Option<String> {
    for event in events.iter().rev() {
        if event.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let Some(content) =
            event.get("message").and_then(|m| m.get("content")).and_then(Value::as_array)
        else {
            continue;
        };
        let Some(text_block) =
            content.iter().rev().find(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        else {
            continue;
        };
        let text = text_block.get("text").and_then(Value::as_str).unwrap_or("");
        return Some(last_non_empty_line(text).unwrap_or_default());
    }
    None
}

fn last_non_empty_line(text: &str) -> Option<String> {
    text.lines().map(str::trim).rev().find(|l| !l.is_empty()).map(String::from)
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
            "Either finish and cap the work, write .bee/HANDOFF.json and release \
reservations so the next session can resume cleanly, or record a capture \
stub for what settled (bee capture add) and close cleanly."
                .to_string(),
        );
        parts.push(lines.join("\n"));
    }
    Ok(AdvisoryOutcome::Done)
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod reads;
mod nudges;
mod store;
mod perf;
mod html;
pub(crate) use self::reads::*;
pub(crate) use self::nudges::*;
pub(crate) use self::store::*;
pub(crate) use self::perf::*;
pub(crate) use self::html::*;
