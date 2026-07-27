//! chain_nudge — Rust port of `.bee/bin/hooks/bee-chain-nudge.mjs`
//! (rust-port-17, CONTEXT.md D2/D7): SubagentStop. Advances the bee chain
//! mechanically: a registered bee worker (or the swarming phase) gets a
//! nudge to collect its [STATUS] token; the reviewing phase gets a
//! reviewer-synthesis nudge; otherwise silent. Read-only: this hook never
//! writes anything — it only emits an advisory JSON systemMessage on
//! stdout (D2: SubagentStop is advisory-only, never `decision:"block"`).
//!
//! `.bee/bin/hooks/bee-chain-nudge.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is conformance-checked against a
//! sha256-verified copy of it (`crates/queen-bee/tests/
//! heavyhooks_conformance.rs`), never edited to "improve" on it.
//!
//! Every call here uses the hook's OWN `root` (the physical checkout, never
//! `ctx.control_root`) — matching the mjs source's literal usage exactly:
//! bee-chain-nudge.mjs never references `ctx.controlRoot` at all.

use std::io::Write as _;
use std::path::Path;

use serde_json::Value;

use bee_core::state;

use crate::adapter::{self, HookContext};
use crate::hookconfig;
use crate::hooks::write_guard::{crash_seam_panic_if_armed, run_fail_open};

const HOOK_NAME: &str = "chain-nudge";

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx: HookContext = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return 0;
    }
    let source = ctx.source.clone();
    run_fail_open(Some(&root), HOOK_NAME, source.as_deref(), || {
        crash_seam_panic_if_armed(HOOK_NAME);
        decide(&ctx, &root)
    })
}

/// `getAgentName(payload)` — first non-blank candidate among
/// `agent_name`/`agentName`/`agent_nickname`/`subagent_type`/`agent_type`.
fn get_agent_name(payload: &Value) -> String {
    for key in ["agent_name", "agentName", "agent_nickname", "subagent_type", "agent_type"] {
        if let Some(s) = payload.get(key).and_then(Value::as_str) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

/// `workerName(entry)` — a string entry is itself the name; an object entry
/// is `nickname || name || agent || worker`; anything else is `""`.
fn worker_name(entry: &Value) -> String {
    match entry {
        Value::String(s) => s.clone(),
        Value::Object(_) => {
            for key in ["nickname", "name", "agent", "worker"] {
                if let Some(s) = entry.get(key).and_then(Value::as_str) {
                    if !s.is_empty() {
                        return s.to_string();
                    }
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}

/// `getSessionId(payload)` — a non-blank `payload.session_id` string, else
/// `None` (fail-open: an unresolvable lane binding never blocks the nudge).
fn get_session_id(payload: &Value) -> Option<String> {
    payload.get("session_id").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

const REVIEW_MSG: &str = concat!(
    "bee chain-nudge: a review agent finished. Collect its findings report, ",
    "score severities independently (P1/P2/P3), and when all reviewers are done ",
    "synthesize findings (corroboration promotes one level; disagreements go ",
    "conservative), then present Gate 4."
);

fn worker_nudge_message(who: &str) -> String {
    format!("bee chain-nudge: {who} returned - collect its [STATUS] token ")
        + concat!(
            "([DONE]/[BLOCKED]/[HANDOFF]/[NOOP]), update the cell ",
            "(node .bee/bin/bee.mjs cells), and check/release its reservations ",
            "(node .bee/bin/bee.mjs reservations list --active-only). ",
            "When the wave is clean, move to the next wave or the next chain step."
        )
}

fn scribing_debt_suffix(count: i64, cells: &[String]) -> String {
    format!(
        "\n⚠ Scribing debt: {count} behavior_change cell(s) capped since the last capture ({}) — run bee-scribing capture now; don't wait for review (decision 0011).",
        cells.join(", ")
    )
}

fn decide(ctx: &HookContext, root: &Path) -> i32 {
    if !hookconfig::hook_enabled(root, HOOK_NAME) {
        return 0;
    }

    let bee_state = state::read_state(root);
    let session_id = get_session_id(&ctx.payload);
    let pipeline = state::resolve_pipeline(root, session_id.as_deref());
    let phase = match &pipeline {
        Ok(res) => res.record.phase.clone(),
        Err(_) => bee_state.phase.clone(),
    };
    let phase = if phase.is_empty() { "idle".to_string() } else { phase };

    let agent_name = get_agent_name(&ctx.payload);
    let is_registered_worker = !agent_name.is_empty() && bee_state.workers.iter().any(|w| worker_name(w) == agent_name);

    let msg: Option<String> = if phase == "reviewing" {
        Some(REVIEW_MSG.to_string())
    } else if is_registered_worker || phase == "swarming" {
        let who = if agent_name.is_empty() { "A bee worker".to_string() } else { format!("Worker \"{agent_name}\"") };
        let mut m = worker_nudge_message(&who);
        // Decision 0011: capture-mode spine — nudge scribing capture
        // in-flight, not only at feature close.
        //
        // rust-port-23: moved to the shared-read signature. The memo is
        // LAZY, so this hook's read profile is unchanged — with no active
        // feature `scribing_debt_from` returns before touching
        // `shared.cells()` and this hook still scans no cells at all,
        // which matters because it runs on EVERY SubagentStop event.
        let shared = bee_core::shared_reads::SharedReads::new(root);
        let debt = bee_core::cells::scribing_debt_from(&shared, None, None);
        let count = debt.get("count").and_then(Value::as_i64).unwrap_or(0);
        if count > 0 {
            let cells: Vec<String> = debt
                .get("cells")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            m.push_str(&scribing_debt_suffix(count, &cells));
        }
        Some(m)
    } else {
        None
    };

    if let Some(msg) = msg {
        // This wrapper is only wired to SubagentStop, so a payload missing
        // hook_event_name still encodes as an advisory.
        let event = if ctx.event.is_empty() { "SubagentStop" } else { ctx.event.as_str() };
        if let Some(out) = adapter::emit_hook_output(event, &msg) {
            let _ = std::io::stdout().write_all(out.as_bytes());
        }
    }
    0
}
