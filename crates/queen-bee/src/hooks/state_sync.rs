//! state_sync — Rust port of `.bee/bin/hooks/bee-state-sync.mjs`
//! (rust-port-17, CONTEXT.md D2/D3/D7): PostToolUse
//! (update_plan|TaskCreate|TaskUpdate|TodoWrite) + SubagentStop + Stop.
//! Always silent (no stdout — the Codex advisory-output requirement is
//! satisfied by silence, same as the mjs source) and always exit 0 (via
//! [`run_fail_open`]).
//!
//! Four side effects, each independently fail-open (a miss in one never
//! blocks the others):
//!   1. throttled session-heartbeat write (`claims::heartbeat_touch`),
//!      gating...
//!   2. ...this session's reservation-lease renewal
//!      (`reservations::renew_holds_by_session`) and...
//!   3. ...this session's cross-worktree hold renewal
//!      (`holds::renew_holds`) — only when (1) actually touched;
//!   4. the locked `rebuildStateProjection` call (rust-port-16), refreshing
//!      `cells`/`last_activity` every run regardless of (1)-(3), with
//!      `LOCK_BUSY` skipping silently (msn-7 F8: this hook never writes a
//!      workflow record — it only ever calls `rebuild_state_projection`,
//!      never `create_workflow`/`update_workflow`).
//!
//! `.bee/bin/hooks/bee-state-sync.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is conformance-checked against a
//! sha256-verified copy of it (`crates/queen-bee/tests/
//! heavyhooks_conformance.rs`), never edited to "improve" on it.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use bee_core::lock::{self, LockOptions, WithLockError};
use bee_core::state_projection::StateOverrides;
use bee_core::{cells, claims, holds, reservations};

use crate::adapter::{self, HookContext};
use crate::hookconfig;
use crate::hooks::write_guard::{crash_seam_panic_if_armed, run_fail_open};

const HOOK_NAME: &str = "state-sync";

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx: HookContext = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return 0;
    }
    let source = ctx.source.clone();
    run_fail_open(Some(&root), source.as_deref(), || {
        crash_seam_panic_if_armed(HOOK_NAME);
        decide(&ctx, &root);
        0
    })
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn session_id(ctx: &HookContext) -> Option<String> {
    ctx.payload.get("session_id").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn cell_counts(root: &Path) -> Value {
    let mut open = 0i64;
    let mut claimed = 0i64;
    let mut capped = 0i64;
    let mut blocked = 0i64;
    for cell in cells::list_cells(root) {
        match cell.status.as_deref() {
            Some("open") => open += 1,
            Some("claimed") => claimed += 1,
            Some("capped") => capped += 1,
            Some("blocked") => blocked += 1,
            _ => {}
        }
    }
    json!({ "open": open, "claimed": claimed, "capped": capped, "blocked": blocked })
}

fn decide(ctx: &HookContext, root: &Path) {
    if !hookconfig::hook_enabled(root, HOOK_NAME) {
        return;
    }

    // D5 — throttled heartbeat + claim/hold lease renewal, each in its own
    // try/catch-equivalent (a miss here never blocks the state-counts
    // refresh below). `heartbeatTouch` and `worktree-holds.renewHolds` both
    // take the CONTROL root (sessions/claims/cross-worktree holds are
    // control-plane, multisession-native-18c); `renewHoldsBySession` takes
    // the bare hook root and re-roots internally in the mjs source — this
    // port's established pattern instead has the CALLER pass the
    // already-resolved control root directly (rust-port-16), which is the
    // exact same value for every ordinary/solo checkout and for a linked
    // worktree alike (`ctx.control_root` is computed by the SAME
    // `resolve_roots` walk mjs's own `controlRootFor`/`resolveRoots` calls
    // would produce from this same cwd).
    if let Some(sid) = session_id(ctx) {
        let control_root = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
        let ts = now_ms();
        let touch = claims::heartbeat_touch(&control_root, &sid, ts);
        if touch.touched {
            let _ = reservations::renew_holds_by_session(&control_root, &sid, ts);
            // hardening-1-7-10 (D3) — the cross-worktree ledger renewal,
            // same try-once/never-block posture: a missed renewal is
            // skipped, never waited on, never escalated past this hook's
            // own fail-open boundary.
            let _ = holds::renew_holds(&control_root, &sid, LockOptions::try_once());
        }
    }

    let counts = cell_counts(root);
    let overrides = StateOverrides {
        cell_counts: Some(counts),
        last_activity: Some(Value::String(adapter::timestamp_now())),
    };

    // multisession-native-7 (F8, binding): this write is a FULL idempotent
    // rebuild via rebuild_state_projection (rust-port-16) — never a partial
    // patch of just cells/last_activity. LOCK_BUSY skips this sync
    // silently (fail-open preserved), never waited on, never escalated to
    // a crash log. This hook never calls create_workflow/update_workflow,
    // so it never writes a workflow record.
    let control_root = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
    match lock::with_store_lock(root, "state", LockOptions::try_once(), || {
        bee_core::state_projection::rebuild_state_projection(root, &control_root, &overrides)
    }) {
        Ok(_io_result) => {
            // An io error surfaces here as Ok(Err(..)) from the closure's
            // own return value — same fail-open posture as every other
            // best-effort write in this hook: never propagated further.
        }
        Err(WithLockError::Busy { .. }) => {
            // LOCK_BUSY — mirrors the mjs source's own
            // `if (!(error instanceof lockLib.LockBusyError)) throw error;`
            // catch, which swallows exactly this case.
        }
        Err(WithLockError::Io(_)) => {
            // A lock-acquisition I/O fault: fail-open, same as above.
        }
    }
}
