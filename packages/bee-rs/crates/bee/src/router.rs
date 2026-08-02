// Routing: which argv shapes are served natively.
//
// Native verbs register a `try_native(args, t0) -> Option<ExitCode>` probe:
// returning None (for any reason — an unrecognized flag shape, an argv the
// port never proved) means NO output has been produced and no verb claimed the
// command. `bee rs-info` is a diagnostic outside the porcelain namespace, so
// it can never collide with a porcelain command.
//
// CUTOVER: None used to mean "hand it to `node bee.mjs`". The Node tree is
// gone, so None now ends at `emit_unsupported_shape` below — a named refusal
// on stderr with a non-zero exit. Silence is the one outcome forbidden here:
// a dispatcher that neither serves nor complains is indistinguishable from a
// command that succeeded and did nothing.
//
// "Corrupt JSON input" used to be on the list of decline reasons. It no longer
// is — a corrupt store file warns natively (fsutil::warn_corrupt_json) and
// takes the fallback Node's readJson would have returned, so no probe declines
// a command over one.

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
    "dev release-manifest --write|--check|--selftest",
    // R6a — the bee-herding cockpit's executable helpers, ported off Node
    "herding classify-lane <PBI-ID>",
    "herding interlock [--main-root PATH]",
    "herding command-template <key> [--main-root PATH]",
    "herding herdr-result <dotted.path> [--context NAME]",
    "herding herdr-pane-id --label <label>",
    // R3 wave 4 — the last per-group debts
    "state set|gate|scribing-run|compounding-run|plan-rev bump|handoff (ALL repo shapes)",
    "state workflows list|close",
    "decisions supersede|render",
    "backlog rank|badges|render",
    "feedback digest|collect|rank",
    "knowledge promote",
    // R6 — the last debts blocking Node deletion
    "state rebuild-projections",
    "cells claim-next",
    "state route --show",
    "state route --set (repos with no granted worktree)",
    // Linked-worktree coverage. These serve from INSIDE a linked worktree —
    // granted or ungranted — not just the main checkout; every other verb
    // still delegates there. A BROKEN link delegates for everyone. The
    // authoritative flipped/delegated split lives in roots.rs's header.
    "status|status --lanes-full|orient (inside linked worktrees)",
    "reservations list|reserve|release|sweep (inside linked worktrees)",
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
    // render-prompt, statusline, release-manifest). Like
    // onboard it is not a bee.mjs porcelain verb, so it probes before the
    // verb tree; a `dev` shape it does not serve returns None with no output
    // and the delegate reports unknown-command exactly as Node does.
    if let Some(code) = crate::devtools::try_native(args) {
        return Some(code);
    }
    // `bee herding …` is the R6a home of the bee-herding cockpit's two
    // executable helpers (classify-lane, interlock), ported off their .mjs.
    // No Node command ever spelled `herding`, so like onboard and dev it
    // probes ahead of the verb tree and cannot shadow a delegated verb.
    if let Some(code) = crate::herding::try_native(args) {
        return Some(code);
    }
    verbs::try_native(args, t0)
}

/// The end of the line. Reached only when no probe claimed the argv and no
/// probe emitted anything — the shape is either a command this binary has no
/// spelling for, or a flag combination the owning verb never proved.
///
/// Deliberately generic. bee.mjs's dispatcher carried a nearest-match
/// suggester and per-group `Use: …` usage lines; campaign rule 1 kept that
/// machinery out of the port on purpose (a verb served only proven shapes and
/// Node owned every error path). Reproducing it now would be inventing an
/// error surface, not porting one — so this says exactly what is true and
/// points at the two help surfaces that ARE native and complete.
///
/// `--json` is honoured: a script that asked for JSON gets `{"error": …}` on
/// stdout rather than an unparseable empty document.
pub fn emit_unsupported_shape(args: &[OsString]) -> ExitCode {
    let shown: Vec<String> =
        args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let attempted = if shown.is_empty() { "(no command)".to_string() } else { shown.join(" ") };
    let msg = format!(
        "bee: unsupported command shape: `bee {attempted}`.{} FIX: `bee --help` for the flow surface, `bee --help --all` for every command.",
        group_hint(shown.first().map(String::as_str))
    );
    let json = shown.iter().any(|a| a == "--json" || a.starts_with("--json="));
    if json {
        println!("{}", crate::jsjson::stringify(&serde_json::json!({ "error": msg })));
    } else {
        eprintln!("{msg}");
    }
    ExitCode::FAILURE
}

/// The one thing worth reconstructing from bee.mjs's error machinery: when the
/// first token IS a known group, say what that group actually takes. This is
/// what turned `bee state show` and `bee backlog list` — the two shapes the
/// cutover found in live use — from a bare "unsupported" into a usable answer,
/// and it costs nothing: the sub-verb list is read straight out of
/// `REGISTRY_PAYLOAD`, the same bytes `--help` renders from.
fn group_hint(first: Option<&str>) -> String {
    let Some(group) = first.filter(|g| !g.starts_with('-')) else { return String::new() };
    let Ok(payload) =
        serde_json::from_str::<serde_json::Value>(crate::registry::REGISTRY_PAYLOAD)
    else {
        return String::new();
    };
    let Some(commands) = payload.get("commands").and_then(|c| c.as_array()) else {
        return String::new();
    };
    let prefix = format!("{group}.");
    let mut subs: Vec<String> = commands
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .filter_map(|n| n.strip_prefix(&prefix))
        .map(|rest| rest.replace('.', " "))
        .collect();
    if subs.is_empty() {
        return String::new();
    }
    subs.dedup();
    format!(" `bee {group}` takes: {}.", subs.join(", "))
}

fn rs_info() -> ExitCode {
    let info = serde_json::json!({
        "runtime": "rust",
        "version": env!("CARGO_PKG_VERSION"),
        "ported": PORTED,
        "fallback": null,
    });
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
    ExitCode::SUCCESS
}
