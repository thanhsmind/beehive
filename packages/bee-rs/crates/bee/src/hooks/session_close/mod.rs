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


use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, read_hook_context, HookContext};

use crate::hooks::Outcome;

use crate::jsjson::js_to_string;


use serde_json::{Map, Value};


use std::path::Path;

use std::process::ExitCode;

const HOOK_NAME: &str = "session-close";

const INJECT_INTERVAL_MS: f64 = 30.0 * 60.0 * 1000.0;

const DECISION_RECENT_MS: f64 = 6.0 * 3600.0 * 1000.0;

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
