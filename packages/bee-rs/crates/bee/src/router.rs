// Routing: which argv shapes are served natively.
//
// Native verbs register a `try_native(args, t0) -> Option<ExitCode>` probe:
// returning None (for any reason — unrecognized flag shape, linked-worktree
// root, corrupt JSON input) falls through to the Node delegate BEFORE any
// output is produced. `bee rs-info` is a diagnostic outside the porcelain
// namespace, so it can never collide with a Node-surface command.

use crate::verbs;
use std::ffi::OsString;
use std::process::ExitCode;
use std::time::Instant;

/// Human-readable list for `bee rs-info` — keep in sync with the probes below.
pub const PORTED: &[&str] = &[
    "status --brief",
    "hook tools-logger",
    "hook codex-subagent-audit",
    "hook chain-nudge",
    "hook state-sync",
    "hook prompt-context",
    "hook session-init",
    "hook model-guard",
    "hook session-close",
    "hook write-guard",
    "cells list",
    "cells ready",
    "cells show",
    "status",
    "orient",
    "reservations list",
    "reservations reserve",
    "reservations release",
    "reservations sweep",
    "decisions active",
    "decisions search",
    "decisions log",
    "decisions tag",
    "decisions redact",
    "decisions archive",
    "capture count",
    "capture list",
    "capture add",
    "capture flush",
    "backlog counts",
    "backlog findings",
    "backlog add",
    "backlog propose",
    "backlog pbi add",
    "backlog pbi status",
    "backlog pbi amend",
    "backlog pbi list",
    "feedback count",
    // R3 wave 2
    "--help (all forms, incl. group-scoped)",
    "test",
    "cells add|update|claim|unclaim|cap|finish|block|drop|reopen|tier",
    "cells judge|judge-record|reset-budget|schedule|archive|unarchive",
    "state worker add|update|remove|clear|prune",
    "state lanes",
    "state session list|bind|unbind",
    "state set|gate|scribing-run|compounding-run|plan-rev bump|handoff (no-lane/no-workflow repos)",
    "intent set|show|advance|clear",
    "reviews create|list|show|record|candidate add|candidates|status (--file shapes)",
    "knowledge check|index|list|context",
    "tmp sweep",
];

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first().and_then(|a| a.to_str()) == Some("rs-info") {
        return Some(rs_info());
    }
    if let Some(code) = crate::hooks::try_native(args) {
        return Some(code);
    }
    verbs::try_native(args, t0)
}

fn rs_info() -> ExitCode {
    let info = serde_json::json!({
        "runtime": "rust",
        "version": env!("CARGO_PKG_VERSION"),
        "ported": PORTED,
        "fallback": "node bee.mjs",
    });
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
    ExitCode::SUCCESS
}
