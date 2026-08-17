// Routing: which argv shapes are served natively.
//
// Native verbs register a `try_native(args, t0) -> Option<ExitCode>` probe:
// returning None (for any reason — an unrecognized flag shape, an argv the
// port never proved) means NO output has been produced and no verb claimed the
// command. `bee rs-info` is a diagnostic outside the porcelain namespace, so
// it can never collide with a porcelain command.
//
// None ends at `emit_unsupported_shape` below — a named refusal on stderr
// with a non-zero exit. Silence is the one outcome forbidden here: a
// dispatcher that neither serves nor complains is indistinguishable from a
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
    "state session list|bind|unbind|release",
    "state set|gate|scribing-run|compounding-run|plan-rev bump|handoff (no-lane/no-workflow repos)",
    "intent set|show|advance|clear",
    "reviews create|list|show|record|candidate add|candidates|status (--file shapes)",
    "knowledge check|index|list|context",
    "tmp sweep",
    // R3 wave 3
    "close (incl. --dry-run; lane and non-lane features alike)",
    "dispatch prepare (all kinds; --claim still delegated)",
    "dispatch wave --runtime <claude|codex> [--feature F] [--limit N] [--session-id S] \
     (the current schedule wave, claimed + reserved + prepared in one call, scoped to \
     one resolved feature; --limit bounds how many cells are claimed)",
    "worktree list|register|unregister",
    // R4 dev surface
    "onboard [--repo-root R] [--apply] [--json] [--repo-hooks] [--plugin-source] \
     [--runtime R] [--claude-md|--no-claude-md] [--global-skills] [--force-downgrade]",
    "dev render-skill-trees",
    "dev render-prompt <name> [--var K=V]",
    "dev statusline",
    "dev release-manifest --write|--check|--selftest",
    "dev regen (render-skill-trees -> onboard --repo-root . --apply -> \
     release-manifest --write, stops on the first red)",
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
    // wayfinding-flow — the discovery map registry (D2, D4, D6.3)
    "discovery list [--json]",
    "discovery stub --effort <slug> --from <text> [--json]",
];

// ── the front door's two namespaces ────────────────────────────────────────
//
// Before this, `bee --help` filtered 124 top-level verbs down to 17 and called
// the result a porcelain surface. Nothing had actually moved: the flow the
// lifecycle talks about — route the work, shape it, approve the gate, finish
// the cell — was spelled in terms of the STORE it happens to live in
// (`state route`, `intent set`, `state gate`, `cells finish`), so the skills
// had to carry the translation. Prose was sequencing the machine.
//
// Two changes, both here so the whole split is readable in one place:
//
//   * FLOW_VERBS gives each lifecycle step its own one-token spelling. They
//     are ALIASES, not new behaviour: argv is rewritten and the proven verb
//     runs, so there is exactly one implementation and one set of tests per
//     operation. The target keeps working and keeps its own registry entry;
//     it just leaves the flow surface, since two porcelain spellings of one
//     command would grow the list the flow surface exists to keep short.
//
//   * `bee internal …` is the plumbing namespace. It dispatches identically
//     after the prefix is stripped, and REFUSES a flow verb — so the boundary
//     is real in both directions rather than being a help filter. The bare
//     top-level spellings still work; retiring them is a breaking change with
//     its own migration, and a namespace nobody can call yet would be worse
//     than the filter it replaces.
pub const FLOW_VERBS: &[(&str, &[&str])] = &[
    ("route", &["state", "route"]),
    ("shape", &["intent", "set"]),
    ("gate", &["state", "gate"]),
    ("finish", &["cells", "finish"]),
];

pub const INTERNAL: &str = "internal";

fn flow_expansion(head: &str) -> Option<&'static [&'static str]> {
    FLOW_VERBS.iter().find(|(name, _)| *name == head).map(|(_, to)| *to)
}

/// argv as the verb tree should see it: a flow verb expanded to its target,
/// or one `internal` prefix stripped. None when neither applies.
fn rewrite_front_door(args: &[OsString]) -> Option<Vec<OsString>> {
    let head = args.first()?.to_str()?;
    if head == INTERNAL {
        let rest = &args[1..];
        // A second `internal` is a caller who thinks the namespace nests.
        if rest.first().and_then(|a| a.to_str()) == Some(INTERNAL) {
            return None;
        }
        return Some(rest.to_vec());
    }
    let expansion = flow_expansion(head)?;
    let mut out: Vec<OsString> = expansion.iter().map(OsString::from).collect();
    out.extend_from_slice(&args[1..]);
    Some(out)
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first().and_then(|a| a.to_str()) == Some(INTERNAL) {
        if let Some(code) = internal_namespace(args, t0) {
            return Some(code);
        }
    }
    // Help probes the argv AS TYPED, before any rewrite. Help renders from
    // the registry, and a flow verb has its own registry entry — so
    // `bee route --help` must describe `bee route`. Rewriting first would
    // answer with `bee state route`, i.e. document a spelling the caller did
    // not use and send them back to the group structure the flow verb exists
    // to hide. (The probe is a no-op for every non-help argv.)
    if let Some(code) = crate::verbs::help::try_native(args, t0) {
        return Some(code);
    }
    match rewrite_front_door(args) {
        Some(rewritten) => dispatch(&rewritten, t0),
        None => dispatch(args, t0),
    }
}

/// The `internal` prefix's own two answers, before anything is dispatched:
/// its help surface, and the refusal that keeps a flow verb out of it.
fn internal_namespace(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    let rest: Vec<&str> = args[1..].iter().filter_map(|a| a.to_str()).collect();
    let wants_help = rest.is_empty() || rest.iter().any(|t| *t == "--help");
    if wants_help && rest.iter().all(|t| *t == "--help" || *t == "--json") {
        return crate::verbs::help::internal_surface(rest.contains(&"--json"), t0);
    }
    let head = rest.first()?;
    if flow_expansion(head).is_some() {
        eprintln!(
            "bee: `bee {head}` is a flow command, not plumbing — call it as `bee {head}`, without \
             `internal`. `bee internal --help` lists what the namespace holds."
        );
        return Some(ExitCode::FAILURE);
    }
    None
}

fn dispatch(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first().and_then(|a| a.to_str()) == Some("rs-info") {
        return Some(rs_info());
    }
    if let Some(code) = crate::hooks::try_native(args) {
        return Some(code);
    }
    // onboard is a maintenance surface, not a porcelain verb, so it probes
    // BEFORE the verb tree: nothing in `verbs` can claim the word.
    if let Some(code) = crate::onboard::try_native(args) {
        return Some(code);
    }
    // `bee dev …` is the R4 dev-surface namespace (render-skill-trees,
    // render-prompt, statusline, release-manifest). Like onboard it is not a
    // porcelain verb, so it probes before the verb tree; a `dev` shape it
    // does not serve returns None with no output and the unknown-command
    // refusal below reports it.
    if let Some(code) = crate::devtools::try_native(args) {
        return Some(code);
    }
    // `bee herding …` is the R6a home of the bee-herding cockpit's two
    // executable helpers (classify-lane, interlock). No porcelain verb ever
    // spelled `herding`, so like onboard and dev it probes ahead of the verb
    // tree and cannot shadow a served verb.
    // `bee doctor` is the runtime health verdict. Like onboard and dev it is
    // not a porcelain verb, so it probes ahead of the verb tree.
    if let Some(code) = crate::doctor::try_native(args) {
        return Some(code);
    }
    if let Some(code) = crate::herding::try_native(args) {
        return Some(code);
    }
    verbs::try_native(args, t0)
}

/// The end of the line: no probe claimed the argv and no probe emitted
/// anything.
///
/// It used to answer every such argv with one sentence — "unsupported command
/// shape", followed by the group's sub-verb list. For `bee knowledge context`
/// that reads as *this command does not exist*, and then lists `context` among
/// the valid ones. The real answer was "add --work and --budget". An agent
/// that believes a verb is gone goes looking for a detour; an agent told which
/// flag is missing adds the flag. So the refusal now separates the three
/// situations it was collapsing (see crate::catalog):
///
///   * UNKNOWN      — nothing in the registry spells this. Suggest the
///                    nearest spellings, or list the group's verbs when the
///                    first token IS a group.
///   * UNAVAILABLE  — the registry declares it and this build has no
///                    implementation. Name the gap instead of implying a typo.
///   * BAD ARGS     — a real, built command declined this argv. Name the
///                    missing required flags, and quote the command's own
///                    example.
///
/// Every branch still ends in a non-zero exit with output on exactly one
/// stream, and `--json` still yields `{"error": …}` on stdout so a script that
/// asked for JSON never has to parse an empty document. The JSON form also
/// carries `{kind, command}` — a caller can now BRANCH on the refusal instead
/// of regex-matching prose.
pub fn emit_unsupported_shape(args: &[OsString]) -> ExitCode {
    let shown: Vec<String> = args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let (kind, command, msg) = classify(&shown);

    let json = shown.iter().any(|a| a == "--json" || a.starts_with("--json="));
    if json {
        println!(
            "{}",
            crate::jsjson::stringify(&serde_json::json!({
                "error": msg,
                "kind": kind,
                "command": command,
            }))
        );
    } else {
        eprintln!("{msg}");
    }
    ExitCode::FAILURE
}

const HELP_FIX: &str =
    "FIX: `bee --help` for the flow surface, `bee --help --all` for every command.";

/// The five leading phrases every dispatcher refusal starts with, one per
/// branch of `classify`. They are a CONTRACT, not prose: `tests/concurrency.rs`
/// keys its unserved-shape tripwire on them (a scenario it cannot run must be
/// skipped out loud, never silently passed) and `tests/registry_dispatch.rs`
/// keys the registry↔dispatcher walk on them. `refusal_headlines_are_stable`
/// below fails if a reworded message stops matching, so the two black-box
/// suites can never go quietly vacuous.
#[cfg_attr(not(test), allow(dead_code))]
pub const REFUSAL_HEADLINES: [&str; 5] = [
    "bee: unknown command",
    "bee: not built into this binary",
    "bee: unexpected positional argument",
    "bee: missing required argument",
    "bee: unsupported argument shape",
];

/// (kind, resolved command name or null, message).
fn classify(shown: &[String]) -> (&'static str, serde_json::Value, String) {
    if shown.is_empty() {
        return (
            "unknown_command",
            serde_json::Value::Null,
            format!("bee: unknown command (no command given). {HELP_FIX}"),
        );
    }
    let mut leading: Vec<&str> = shown
        .iter()
        .take_while(|t| !t.starts_with("--"))
        .map(String::as_str)
        .collect();
    let attempted = shown.join(" ");
    // Resolve against the argv the verb tree saw, not the one that was typed:
    // `bee internal state shwo` must be answered about `state shwo`, and
    // `bee gate --name x` about `state gate`. The message still quotes what
    // was typed.
    if leading.first() == Some(&INTERNAL) {
        leading.remove(0);
        if leading.is_empty() {
            return (
                "unknown_command",
                serde_json::Value::Null,
                format!("bee: unknown command `bee {attempted}` — `bee internal` is the plumbing namespace, not a command. FIX: `bee internal --help` for what it holds."),
            );
        }
    }
    // A flow verb has its own registry entry, so it resolves as typed — and
    // must, or `bee shape` would report its missing flags against `bee intent
    // set`, naming a command the caller never typed. The expansion is only a
    // fallback for a spelling the registry has not caught up with.
    let expanded: Vec<&str>;
    if crate::catalog::resolve(&leading).is_none() {
        if let Some(to) = leading.first().and_then(|h| flow_expansion(h)) {
            expanded = to.iter().copied().chain(leading[1..].iter().copied()).collect();
            leading = expanded;
        }
    }

    let Some((entry, extra)) = crate::catalog::resolve(&leading) else {
        return ("unknown_command", serde_json::Value::Null, unknown_message(&leading, &attempted));
    };

    if let Some(gap) = &entry.unavailable {
        // The honest answer to `bee doctor` after the Node deletion. Saying
        // "unsupported shape" here would send the caller hunting for the right
        // flags for a command that has no code at all.
        return (
            "command_unavailable",
            serde_json::Value::String(entry.name.clone()),
            format!(
                "bee: not built into this binary: `{}` is declared in the command registry, {}. \
                 Nothing ran and nothing changed. FIX: {}",
                entry.invoke, gap.reason, gap.fix
            ),
        );
    }

    if !extra.is_empty() {
        // A real command with a stray positional: `bee status foo`.
        return (
            "unexpected_argument",
            serde_json::Value::String(entry.name.clone()),
            format!(
                "bee: unexpected positional argument `{}` after `{}`. FIX: `{} --help` for the flags it accepts.",
                extra.join(" "),
                entry.invoke,
                entry.invoke
            ),
        );
    }

    let missing = crate::catalog::missing_required(entry, shown);
    if !missing.is_empty() {
        let flags = missing
            .iter()
            .map(|m| format!("--{m} <{}>", entry.type_of(m)))
            .collect::<Vec<_>>()
            .join(" ");
        let named =
            missing.iter().map(|m| format!("--{m}")).collect::<Vec<_>>().join(", ");
        let example = entry
            .examples
            .first()
            .map(|e| format!(" EXAMPLE: {e}"))
            .unwrap_or_default();
        return (
            "missing_required_argument",
            serde_json::Value::String(entry.name.clone()),
            format!(
                "bee: missing required argument(s) for `{}`: {named}. USAGE: {} {flags}.{example}",
                entry.invoke, entry.invoke
            ),
        );
    }

    // Named, built, required flags all present. Two situations look the same
    // from here: a `--flag` the schema does not declare at all (the caller
    // can fix it by spelling one of the accepted flags), or a declared flag
    // refused for its VALUE — an out-of-enum string, a target that does not
    // exist. Only the first has a flag name to offer, so name every unknown
    // flag when there is one; the out-of-enum case keeps today's wording,
    // which is accurate exactly there. Honors cli-ergonomics D1 (8ef2bae6):
    // every problem named in one message.
    let unknown = unknown_flags(entry, shown);
    if !unknown.is_empty() {
        let named = unknown
            .iter()
            .map(|f| match nearest_flag(entry, f) {
                Some(sug) => format!("--{f} (did you mean --{sug}?)"),
                None => format!("--{f}"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let word = if unknown.len() == 1 { "flag" } else { "flags" };
        return (
            "unsupported_argument_shape",
            serde_json::Value::String(entry.name.clone()),
            format!(
                "bee: unsupported argument shape for `{}`: `bee {attempted}` names unknown {word} \
                 {named}. FIX: `{} --help` for every accepted flag and its type.",
                entry.invoke, entry.invoke
            ),
        );
    }

    // No unknown flag name — the verb declined the argv for a reason of its
    // own (an out-of-enum value, a target that does not exist).
    (
        "unsupported_argument_shape",
        serde_json::Value::String(entry.name.clone()),
        format!(
            "bee: unsupported argument shape for `{}`: `bee {attempted}`. Its required arguments \
             are all present, so what it refused is an optional flag, a flag value, or a target \
             that does not exist. FIX: `{} --help` for every accepted flag and its type.",
            entry.invoke, entry.invoke
        ),
    )
}

/// The `--flag` names actually typed, in order, values stripped: `--lane
/// standard` and `--lane=standard` both yield `"lane"`. A bare `--` or an
/// empty flag name (never produced by a real shell, but cheap to guard) is
/// dropped rather than reported as an unknown flag with no name to show.
fn argv_flags(argv: &[String]) -> Vec<&str> {
    argv.iter()
        .filter(|a| a.starts_with("--"))
        .map(|a| a.trim_start_matches('-').split('=').next().unwrap_or(""))
        .filter(|f| !f.is_empty())
        .collect()
}

/// Typed flags the entry's own schema does not declare — the `unsupported_
/// argument_shape` branch's one actionable fact. `--json` is never special-
/// cased: it is a real declared property on almost every entry, so an entry
/// that lacks it is answered honestly rather than silently excused.
fn unknown_flags<'a>(entry: &crate::catalog::Entry, argv: &'a [String]) -> Vec<&'a str> {
    argv_flags(argv).into_iter().filter(|f| !entry.properties.contains_key(*f)).collect()
}

/// The entry's own nearest declared flag spelling, when one is close enough
/// to be worth printing. Same cutoff shape as `catalog::nearest` — allow
/// roughly half the typed length in edits — but computed against this one
/// entry's flags rather than the whole registry.
fn nearest_flag(entry: &crate::catalog::Entry, flag: &str) -> Option<String> {
    let cutoff = 1 + flag.chars().count() / 2;
    entry
        .properties
        .keys()
        .map(|k| (crate::catalog::distance(flag, k), k))
        .filter(|(d, _)| *d <= cutoff)
        .min_by_key(|(d, k)| (*d, (*k).clone()))
        .map(|(_, k)| k.clone())
}

/// Nothing in the registry spells the leading tokens. Two useful answers: when
/// the first token IS a group, list what that group takes (this is what turned
/// `bee state show` and `bee backlog list` — the two shapes the cutover found
/// in live use — into usable output); otherwise suggest nearest spellings.
fn unknown_message(leading: &[&str], attempted: &str) -> String {
    if let Some(group) = leading.first().filter(|g| !g.starts_with('-')) {
        let subs = crate::catalog::group_subverbs(group);
        if !subs.is_empty() {
            let typed = leading[1..].join(" ");
            let got = if typed.is_empty() {
                format!("`bee {group}` is a command group, not a command")
            } else {
                format!("`bee {group}` has no `{typed}` verb")
            };
            return format!(
                "bee: unknown command `bee {attempted}` — {got}. `bee {group}` takes: {}. \
                 FIX: `bee {group} --help` for each verb's flags.",
                subs.join(", ")
            );
        }
    }
    let near = crate::catalog::nearest(leading, 3);
    let hint = if near.is_empty() {
        String::new()
    } else {
        format!(" Closest spellings: {}.", near.join(", "))
    };
    format!("bee: unknown command `bee {attempted}`.{hint} {HELP_FIX}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify_argv(argv: &[&str]) -> (&'static str, String) {
        let shown: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
        let (kind, _, msg) = classify(&shown);
        (kind, msg)
    }

    /// The regression this taxonomy exists for: the old message said the
    /// command was unsupported and then listed it as valid.
    #[test]
    fn a_missing_required_flag_is_not_reported_as_a_missing_command() {
        let (kind, msg) = classify_argv(&["knowledge", "context"]);
        assert_eq!(kind, "missing_required_argument");
        assert!(msg.contains("--work"), "{msg}");
        assert!(msg.contains("--budget"), "{msg}");
        assert!(!msg.contains("unknown command"), "{msg}");
        assert!(!msg.contains("unsupported command shape"), "{msg}");
    }

    #[test]
    fn an_unbuilt_but_declared_command_says_so_instead_of_suggesting_flags() {
        // `doctor` stood here until it was ported. The class needs a LIVE
        // example or the branch stops being exercised at all.
        let (kind, msg) = classify_argv(&["config", "get", "--key", "gate_bypass"]);
        assert_eq!(kind, "command_unavailable");
        assert!(msg.contains("not built into this binary"), "{msg}");
        assert!(msg.contains("Nothing ran and nothing changed"), "{msg}");
    }

    #[test]
    fn a_group_with_a_wrong_verb_lists_the_verbs_it_has() {
        let (kind, msg) = classify_argv(&["state", "show"]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("has no `show` verb"), "{msg}");
        assert!(msg.contains("`bee state` takes:"), "{msg}");
    }

    #[test]
    fn a_typo_suggests_the_spelling_it_meant() {
        let (kind, msg) = classify_argv(&["stauts"]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("bee status"), "{msg}");
    }

    #[test]
    fn nonsense_still_refuses_and_points_at_help() {
        let (kind, msg) = classify_argv(&["definitely-not-a-bee-verb"]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("unknown command"), "{msg}");
        assert!(msg.contains("bee --help"), "{msg}");
    }

    #[test]
    fn a_stray_positional_on_a_real_command_names_the_positional() {
        let (kind, msg) = classify_argv(&["status", "wat"]);
        assert_eq!(kind, "unexpected_argument");
        assert!(msg.contains("`wat`"), "{msg}");
    }

    #[test]
    fn an_empty_argv_still_names_itself() {
        let (kind, msg) = classify_argv(&[]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("(no command given)"), "{msg}");
    }

    fn strs(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    #[test]
    fn a_flow_verb_expands_to_its_proven_target() {
        for (name, target) in FLOW_VERBS {
            let rewritten = rewrite_front_door(&strs(&[name, "--json"])).expect("{name} rewrites");
            let got: Vec<String> =
                rewritten.iter().map(|a| a.to_string_lossy().into_owned()).collect();
            let mut want: Vec<String> = target.iter().map(|s| s.to_string()).collect();
            want.push("--json".to_string());
            assert_eq!(got, want, "{name}");
            // …and the target it expands to is a real command, not a wish.
            assert!(
                crate::catalog::resolve(target).is_some(),
                "{name} expands to {target:?}, which the registry does not declare"
            );
        }
    }

    /// Both spellings are the flow surface's, so a caller who typed the flow
    /// verb must be answered about the flow verb — not about the group the
    /// command happens to be stored under.
    #[test]
    fn a_flow_verb_is_named_in_its_own_refusal() {
        let (kind, msg) = classify_argv(&["shape"]);
        assert_eq!(kind, "missing_required_argument");
        assert!(msg.contains("`bee shape`"), "{msg}");
        assert!(!msg.contains("intent set"), "the caller never typed that: {msg}");
    }

    #[test]
    fn the_internal_prefix_strips_exactly_one_level() {
        let rewritten = rewrite_front_door(&strs(&["internal", "state", "lanes"])).unwrap();
        let got: Vec<String> = rewritten.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(got, vec!["state", "lanes"]);
        // A second `internal` is a caller who thinks the namespace nests.
        assert!(rewrite_front_door(&strs(&["internal", "internal", "status"])).is_none());
    }

    #[test]
    fn a_bare_internal_is_not_a_command() {
        let (kind, msg) = classify_argv(&["internal"]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("plumbing namespace"), "{msg}");
    }

    /// The refusal resolves against what the verb tree SAW, so an unknown verb
    /// under the namespace still gets its group's list rather than a shrug.
    #[test]
    fn an_unknown_verb_under_internal_is_answered_about_the_group() {
        let (kind, msg) = classify_argv(&["internal", "state", "shwo"]);
        assert_eq!(kind, "unknown_command");
        assert!(msg.contains("`bee state` takes:"), "{msg}");
    }

    /// Every flow verb is a registry entry in its own right — otherwise it
    /// would not appear in `--help`, and the CLI-shape write guard would have
    /// no schema to validate it against.
    #[test]
    fn every_flow_verb_is_declared_porcelain_in_the_registry() {
        for (name, _) in FLOW_VERBS {
            let (entry, rest) = crate::catalog::resolve(&[name]).unwrap_or_else(|| {
                panic!("{name} is not in the command registry")
            });
            assert!(rest.is_empty());
            assert_eq!(entry.invoke, format!("bee {name}"));
            assert!(entry.unavailable.is_none(), "{name} must be callable");
        }
    }

    /// The regression this cell exists for: `state route --lane-value` used
    /// to be answered with "an optional flag, a flag value, or a target that
    /// does not exist" — true, but not which one. It now names the flag and,
    /// since `--lane` is one edit away, offers the spelling that was meant.
    #[test]
    fn an_unknown_flag_is_named_with_its_nearest_spelling() {
        let (kind, msg) = classify_argv(&["state", "route", "--lane-value", "standard"]);
        assert_eq!(kind, "unsupported_argument_shape");
        assert!(msg.contains("--lane-value"), "{msg}");
        assert!(msg.contains("(did you mean --lane?)"), "{msg}");
    }

    /// D1 (8ef2bae6): every problem named in one message, not just the first.
    #[test]
    fn two_unknown_flags_are_both_named() {
        let (kind, msg) =
            classify_argv(&["state", "route", "--lane-value", "standard", "--klass", "feature"]);
        assert_eq!(kind, "unsupported_argument_shape");
        assert!(msg.contains("--lane-value"), "{msg}");
        assert!(msg.contains("--klass"), "{msg}");
    }

    /// When every typed flag IS declared, the refusal is about a value or a
    /// target, not a name — today's wording already says that accurately, so
    /// it must not change.
    #[test]
    fn a_call_with_only_declared_flags_keeps_todays_wording() {
        let (kind, msg) =
            classify_argv(&["capture", "flush", "--id", "nope", "--into", "docs/specs/x.md"]);
        assert_eq!(kind, "unsupported_argument_shape");
        assert!(
            msg.contains(
                "Its required arguments are all present, so what it refused is an optional \
                 flag, a flag value, or a target that does not exist."
            ),
            "{msg}"
        );
    }

    /// The black-box suites detect "the front door refused" by these five
    /// leading phrases. If a message is reworded without updating
    /// REFUSAL_HEADLINES, `tests/concurrency.rs` stops recognising an unserved
    /// shape and starts silently passing scenarios it never ran — so the
    /// coupling is pinned here, at the source.
    #[test]
    fn refusal_headlines_are_stable() {
        let cases: [&[&str]; 6] = [
            &[],
            &["definitely-not-a-bee-verb"],
            &["state", "show"],
            &["doctor"],
            &["status", "wat"],
            &["knowledge", "context"],
        ];
        for argv in cases {
            let (_, msg) = classify_argv(argv);
            assert!(
                REFUSAL_HEADLINES.iter().any(|h| msg.starts_with(h)),
                "no headline matches the refusal for {argv:?}: {msg}"
            );
        }
        // And the shape branch, which needs a command whose required flags are
        // all present but whose argv the verb still declines.
        let (kind, msg) = classify_argv(&["capture", "flush", "--id", "nope", "--wat"]);
        assert_eq!(kind, "unsupported_argument_shape");
        assert!(msg.starts_with(REFUSAL_HEADLINES[4]), "{msg}");
    }
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
