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
    // R3 wave 3
    "close (incl. --dry-run; non-lane features)",
    "dispatch prepare (all kinds; --claim still delegated)",
    "worktree list|register|unregister",
    // R4 dev surface
    "onboard [--repo-root R] [--apply] [--json] [--repo-hooks] [--plugin-source] \
     [--runtime R] [--claude-md|--no-claude-md] [--global-skills] [--force-downgrade]",
    "dev render-skill-trees",
    "dev render-prompt <name> [--var K=V]",
    "dev statusline",
    "dev impact-registry --write|--check|--query <file...> [--level 1]",
    "dev release-manifest --write|--check|--selftest",
    // R3 wave 4 — the last per-group debts
    "state set|gate|scribing-run|compounding-run|plan-rev bump|handoff (ALL repo shapes)",
    "state workflows list|close",
    "decisions supersede|render",
    "backlog rank|badges|render",
    "feedback digest|collect|rank",
    "knowledge promote",
];

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first().and_then(|a| a.to_str()) == Some("rs-info") {
        return Some(rs_info());
    }
    if let Some(code) = crate::hooks::try_native(args) {
        return Some(code);
    }
    // onboard is a maintenance surface, not a bee.mjs porcelain verb, so it
    // probes BEFORE the verb tree: nothing in `verbs` can claim the word.
    if let Some(code) = crate::onboard::try_native(args) {
        return Some(code);
    }
    // `bee dev …` is the R4 dev-surface namespace (render-skill-trees,
    // render-prompt, statusline, impact-registry, release-manifest). Like
    // onboard it is not a bee.mjs porcelain verb, so it probes before the
    // verb tree; a `dev` shape it does not serve returns None with no output
    // and the delegate reports unknown-command exactly as Node does.
    if let Some(code) = crate::devtools::try_native(args) {
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
