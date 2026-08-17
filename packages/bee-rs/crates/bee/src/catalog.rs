// catalog — the registry read as a DISPATCH question rather than a help
// question: does this argv name a command, is that command built into this
// binary, and does the caller's flag set satisfy what it declares?
//
// WHY IT EXISTS. `router::emit_unsupported_shape` used to answer three very
// different situations with one sentence:
//
//   bee: unsupported command shape: `bee knowledge context`.
//        `bee knowledge` takes: check, index, list, context, promote.
//
// — which says the command does not exist and then lists it as valid. The
// truth was "you left off --work and --budget". An agent reading that
// concludes the verb is gone and goes looking for a detour, which is the most
// expensive wrong turn this CLI can cause. The three situations are:
//
//   Unknown       nothing in the registry spells this          → suggest
//   Unavailable   the registry declares it, this build has     → say so, name
//                 no implementation (the R6 Node deletion         the gap
//                 left 23 such commands advertised in --help)
//   BadArgs       a real, built command declined this argv     → name the
//                                                                 missing
//                                                                 required
//                                                                 flags
//
// The `unavailable` marker lives on the registry entry itself (alongside
// `deprecated`, same optional-object shape) so exactly one file states what
// this binary can do. tests/registry_dispatch.rs runs every entry's own
// example through the built binary and fails if the marker and the dispatcher
// disagree in EITHER direction — that is the test whose absence let the Node
// deletion ship 23 commands that --help advertises and no code serves.

use serde_json::{Map, Value};
use std::sync::OnceLock;

pub struct Entry {
    /// Dotted registry name, e.g. `state.worker.add`.
    pub name: String,
    /// Spelling as typed, e.g. `bee state worker add`.
    pub invoke: String,
    pub required: Vec<String>,
    pub properties: Map<String, Value>,
    pub examples: Vec<String>,
    /// Present when the registry declares the command but this build has no
    /// implementation. `{reason, fix}`.
    pub unavailable: Option<Unavailable>,
}

pub struct Unavailable {
    pub reason: String,
    pub fix: String,
}

impl Entry {
    /// Command tokens: `state.worker.add` → ["state", "worker", "add"].
    pub fn tokens(&self) -> Vec<&str> {
        self.name.split('.').collect()
    }

    pub fn type_of(&self, param: &str) -> &str {
        self.properties
            .get(param)
            .and_then(|p| p.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("value")
    }
}

fn parse() -> Vec<Entry> {
    let Ok(payload) = serde_json::from_str::<Value>(crate::registry::REGISTRY_PAYLOAD) else {
        return Vec::new();
    };
    let Some(commands) = payload.get("commands").and_then(Value::as_array) else {
        return Vec::new();
    };
    commands
        .iter()
        .filter_map(|c| {
            let name = c.get("name")?.as_str()?.to_string();
            let invoke = c.get("invoke").and_then(Value::as_str).unwrap_or("").to_string();
            let params = c.get("parameters");
            let required = params
                .and_then(|p| p.get("required"))
                .and_then(Value::as_array)
                .map(|r| r.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            let properties = params
                .and_then(|p| p.get("properties"))
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let examples = c
                .get("examples")
                .and_then(Value::as_array)
                .map(|e| e.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            let unavailable = c.get("unavailable").and_then(Value::as_object).map(|u| Unavailable {
                reason: u.get("reason").and_then(Value::as_str).unwrap_or("").to_string(),
                fix: u.get("fix").and_then(Value::as_str).unwrap_or("").to_string(),
            });
            Some(Entry { name, invoke, required, properties, examples, unavailable })
        })
        .collect()
}

pub fn entries() -> &'static [Entry] {
    static CELL: OnceLock<Vec<Entry>> = OnceLock::new();
    CELL.get_or_init(parse)
}

/// Longest-prefix resolution over the LEADING (non-flag) argv tokens, the same
/// rule `verbs/help.rs::resolve_command` uses so help and refusals can never
/// name different commands for the same argv. Returns the entry plus the
/// tokens left over after it.
pub fn resolve<'a>(leading: &[&'a str]) -> Option<(&'static Entry, Vec<&'a str>)> {
    for n in (1..=leading.len()).rev() {
        let candidate = leading[..n].join(".");
        if let Some(e) = entries().iter().find(|e| e.name == candidate) {
            return Some((e, leading[n..].to_vec()));
        }
    }
    None
}

/// Sub-verbs declared under a group token, in registry order (`bee state` →
/// "set", "gate", "worker add", …).
///
/// Unavailable verbs are LEFT OUT. Listing them here would reproduce, one
/// level down, exactly the defect this module exists to remove: a refusal that
/// offers the caller a command the binary cannot run. Spelling one out still
/// gets the honest `not built into this binary` answer, because `resolve`
/// finds it in the registry — the gap is reported, never advertised.
pub fn group_subverbs(group: &str) -> Vec<String> {
    let prefix = format!("{group}.");
    let mut out: Vec<String> = entries()
        .iter()
        .filter(|e| e.unavailable.is_none())
        .filter_map(|e| e.name.strip_prefix(&prefix))
        .map(|rest| rest.replace('.', " "))
        .collect();
    out.dedup();
    out
}

/// Which of the entry's required parameters have no `--flag` in this argv.
/// Only the flag NAME is checked: whether a value is acceptable is the verb's
/// business, and guessing at it here would invent refusals.
pub fn missing_required(entry: &Entry, argv: &[String]) -> Vec<String> {
    entry
        .required
        .iter()
        .filter(|r| {
            let flag = format!("--{r}");
            let eq = format!("--{r}=");
            !argv.iter().any(|a| *a == flag || a.starts_with(&eq))
        })
        .cloned()
        .collect()
}

/// Up to `limit` registry spellings closest to what was typed. Cheap
/// Levenshtein over the whole dotted name AND over the first token, so both
/// `bee stat` and `bee state shwo` land somewhere useful.
pub fn nearest(leading: &[&str], limit: usize) -> Vec<String> {
    if leading.is_empty() {
        return Vec::new();
    }
    let typed = leading.join(".");
    let mut scored: Vec<(usize, &Entry)> = entries()
        .iter()
        .filter(|e| e.unavailable.is_none())
        .map(|e| {
            let whole = distance(&typed, &e.name);
            // A wrong first token should not be rescued by a long tail match.
            let head = distance(leading[0], e.tokens()[0]);
            (whole.min(head + e.name.len().saturating_sub(e.tokens()[0].len())), e)
        })
        .collect();
    scored.sort_by_key(|(d, e)| (*d, e.name.clone()));
    let cutoff = 1 + typed.chars().count() / 2;
    scored
        .into_iter()
        .take_while(|(d, _)| *d <= cutoff)
        .take(limit)
        .map(|(_, e)| e.invoke.clone())
        .collect()
}

/// Plain Levenshtein, two-row. Small inputs; no need for anything cleverer.
/// `pub(crate)` so `router::nearest_flag` can score one entry's flags with
/// the same routine `nearest` uses over the whole registry — one
/// implementation, not two copies of the same algorithm.
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_registry_parses_into_entries() {
        let e = entries();
        assert!(e.len() > 100, "only {} entries parsed", e.len());
        assert!(e.iter().any(|e| e.name == "status"));
        assert!(e.iter().any(|e| e.name == "state.worker.add"));
    }

    #[test]
    fn resolve_takes_the_longest_prefix_and_returns_the_remainder() {
        let (e, rest) = resolve(&["state", "worker", "add"]).unwrap();
        assert_eq!(e.name, "state.worker.add");
        assert!(rest.is_empty());
        let (e, rest) = resolve(&["status", "junk"]).unwrap();
        assert_eq!(e.name, "status");
        assert_eq!(rest, vec!["junk"]);
        assert!(resolve(&["definitely-not-a-verb"]).is_none());
    }

    #[test]
    fn missing_required_names_only_absent_flags() {
        let (ctx, _) = resolve(&["knowledge", "context"]).unwrap();
        assert_eq!(
            missing_required(ctx, &["--json".to_string()]),
            vec!["work".to_string(), "budget".to_string()]
        );
        assert!(missing_required(
            ctx,
            &["--work".into(), "w".into(), "--budget=10".into()]
        )
        .is_empty());
    }

    #[test]
    fn nearest_finds_a_typo_and_stays_quiet_on_nonsense() {
        let hits = nearest(&["stauts"], 3);
        assert!(hits.contains(&"bee status".to_string()), "{hits:?}");
        // A string with nothing in common must not drag in a random verb.
        assert!(nearest(&["zzzzzzzzzzzzzzzzzzzz"], 3).is_empty());
    }

    #[test]
    fn unavailable_entries_carry_a_reason_and_a_fix() {
        for e in entries().iter().filter(|e| e.unavailable.is_some()) {
            let u = e.unavailable.as_ref().unwrap();
            assert!(!u.reason.trim().is_empty(), "{}: unavailable needs a reason", e.name);
            assert!(!u.fix.trim().is_empty(), "{}: unavailable needs a fix line", e.name);
        }
    }

    #[test]
    fn group_subverbs_reads_out_of_the_registry() {
        let subs = group_subverbs("knowledge");
        assert!(subs.contains(&"context".to_string()), "{subs:?}");
    }

    /// A vocabulary ratchet, not a rename. A caller who has learned that
    /// `--worker` names the acting agent on one command should not have to
    /// relearn `--agent` for the same concept on the next — a new spelling
    /// for an existing idea makes that caller's knowledge useless one
    /// command later. This test pins the number of DISTINCT flag names
    /// declared across the whole registry so growth is a decision, not a
    /// side effect: adding a command that reuses an existing flag name
    /// costs nothing here; adding one that invents a new name for an old
    /// idea must bump `PINNED_FLAG_COUNT` on purpose, with a reason in the
    /// commit. It renames nothing today — renaming the 143 already-shipped
    /// names would break every existing caller, which this cell does not
    /// take on — it only stops the count from climbing unnoticed.
    #[test]
    fn distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision() {
        // 143 -> 144 (dispatch-label-chokepoint dlc-1): `dispatch.prepare`
        // gained `--purpose`, a one-line statement of what a non-cell
        // dispatch is FOR, folded into the dispatch label. The existing
        // `reason`/`note` flags (cells.block/drop/reopen/reset-budget,
        // decisions.redact, perf.stop/section) all name why something
        // ALREADY happened; `--purpose` names what a dispatch about to
        // happen is for — a prospective, not retrospective, concept, so
        // reusing one of those names would have been the wrong ratchet.
        // 144 -> 145 (decision-supersede-hygiene dsh-1): `decisions.log`
        // gained `--supersedes`, an array of ids the new decision retires.
        // Checked first: `decisions.supersede --id`/`decisions.redact --id`
        // both name a SINGLE target id already carried by a dedicated event
        // shape, and `--tags` classifies, it does not name a prior decision
        // — none of those is this concept, so reusing one would have hidden
        // an array-of-targets flag behind a singular-id name (or vice
        // versa). `--supersedes` is new.
        // 145 -> 146 (finish-records-deviations frd-1): `cells.cap`/
        // `cells.finish` gained `--deviation`, ONE prose line appended to
        // `trace.deviations` at cap time. Checked first: `--deviations-file`
        // already exists on both commands but names a PATH to a list (JSON
        // array or newline text) read from disk — the concept this cell adds
        // is a single inline line with no file, the shape a worker's own cap
        // invocation can carry directly; `--friction` names a distinct
        // concept (a trigger-fired note, not a named deviation). None of
        // those is "one deviation line, inline" — `--deviation` is new.
        // 146 -> 147 (workflow-lessons wfl-5): `cells.finish` already parsed
        // `--report` (wfl-1, handlers_close.rs CAP_FLAGS) but the registry
        // entry never named it, so it was undiscoverable through
        // `bee --help --json`/catalog lookups even though the flag worked.
        // Checked first: no existing flag names "the worker Result form as
        // JSON" — `--outcome` is a plain one-line string, `--files` a
        // comma-separated list, neither the structured five-key object
        // `--report` carries. This bump documents the flag, it invents
        // nothing new at the CLI surface.
        // 147 -> 148 (awaiting-human ah-4): `state.waiting-on.set` gained
        // `--subject`, the gate name or question text a waiting-on mark
        // names. Checked first: `--reason`/`--note` (cells.block/drop/
        // reopen/reset-budget, decisions.redact, perf.stop/section) all name
        // WHY something already happened, retrospective; `--next-action`
        // (state.handoff.write/state.set) names a saved next STEP, not the
        // subject of a live wait; `--name` (state.gate) is scoped to the
        // closed gate-name enum alone and cannot also carry free-form
        // question text. None of those is "what a wait is about" — `kind`,
        // `session-id`, `lane`, `no-lane` and `json` all reuse existing
        // names for the same concepts unchanged; `--subject` is the one new
        // spelling this cell adds.
        // 148 -> 150 (gate-help-drift ghd-1): `state.gate`/`gate` accepted
        // `--actor`, `--bypass-level` and `--reason` since D2 (set_gate.rs)
        // but the payload never declared them, so `bee --help --json`
        // understated the command. Checked first: `--reason` already names
        // "why" on cells.block/drop/reopen/reset-budget, decisions.redact,
        // perf.stop/section, and is reused here unchanged (no count change
        // from it); no existing flag names "who is approving" or "the
        // bypass level an auto-approval recorded" — `--actor` and
        // `--bypass-level` are the two new spellings. This bump documents
        // flags the handler already accepted; it invents nothing new at the
        // CLI surface.
        // 150 -> 153 (knowledge-distill-trigger C2, kdt-2): `triggers add`/
        // `list`/`resolve` land the deferred-decision trigger registry.
        // Checked first: `--decision` and `--outcome` already exist
        // (decisions.log/supersede's "decision" text, cells.finish's
        // "outcome" result string) and are reused here for a related but
        // distinct concept each (a decision's id rather than its text; a
        // trigger's resolution outcome rather than a worker's) — the plan
        // (docs/history/knowledge-distill-trigger/plan.md C2) names both
        // flags explicitly, so neither is renamed to force a fresh
        // reuse-check; no count cost either way since both names already
        // existed. Three names are genuinely new: `--condition` (the
        // prose firing condition — prospective, unlike the retrospective
        // `--reason`/`--note` pair, and not the wait-subject `--subject`
        // carries either); `--predicate` (a machine-checkable
        // path-exists/path-missing spec — no existing flag names a
        // checkable condition, only free text); `--due` (`triggers list`'s
        // filter to triggers needing a human's attention now — closer to,
        // but not the same concept as, `reservations.list`'s
        // `--active-only`, which means "not yet released/expired" rather
        // than "needs a look").
        // kdt-3 (knowledge-distill-trigger, D3 + D2's write-path law):
        // `decisions.log` drops `--supersedes` (-1) for a required
        // `--relation supersedes:<id>|touches:<id>|none` (+1, reused for the
        // supersedes:<id> case rather than adding a second flag) plus a new
        // `--trigger <id>` (+1) naming a registered kdt-2 trigger — no
        // existing flag names "a decision's own relation to prior ones" or
        // "the trigger this decision's deferral registers against", so both
        // are genuinely new concepts. Net +1: 153 -> 154.
        // uat-gate-before-merge D1: `worktree.merge` gets a new
        // `--skip-uat` (+1) — the one-merge opt-out of the new uat-gate
        // precondition. `--no-cleanup` is the closest existing "one-merge
        // opt-out" shape but names a DIFFERENT concept (skip teardown, not
        // skip a gate check); no existing flag means "skip this one merge's
        // gate precondition", so it is genuinely new. Net +1: 154 -> 155.
        // staging-lane sl-2: `staging.rebuild` gets a new `--without` (+1) —
        // a comma-separated list of features to keep MEMBERS of the staged
        // set but skip merging this one rebuild run. Checked first:
        // `--skip-uat` is a bare boolean gate bypass, a different concept
        // entirely; `--no-cleanup` again names skipping teardown, not
        // skipping a merge; no existing flag means "exclude these features
        // from this run without dropping them from staging" — genuinely
        // new. Net +1: 155 -> 156.
        // wayfinding-flow wayf-6: `discovery list`/`discovery stub` (built
        // by wayf-2, catalog.rs left untouched) gain `--effort` (+1) and
        // `--from` (+1) — this cell only documents flags the handler
        // already accepted (verbs/discovery.rs), same as wfl-5's `--report`
        // bump; it invents nothing new at the CLI surface. Checked first:
        // `--name` (gate/state.gate) is a closed enum of gate names, not a
        // free slug naming a new directory; `--note` (perf.stop/section) is
        // an optional RETROSPECTIVE summary, not a required seed for new
        // content; `--text` (decisions/backlog/knowledge search) is a
        // search-term filter, not prose to store; `--context`
        // (herding.herdr-result) names an error-line label, unrelated. None
        // of those is "a new artifact's directory-naming slug" or "prose
        // seeded into a fresh document's body" — `--effort` and `--from`
        // are the two new spellings. `discovery list`'s `--json` reuses the
        // existing name unchanged, no count cost. Net +2: 156 -> 158.
        const PINNED_FLAG_COUNT: usize = 158;

        let names: std::collections::BTreeSet<&str> =
            entries().iter().flat_map(|e| e.properties.keys()).map(String::as_str).collect();

        assert_eq!(
            names.len(),
            PINNED_FLAG_COUNT,
            "distinct flag-name count changed from {PINNED_FLAG_COUNT} to {}. A new \
             spelling for an existing concept makes a caller's knowledge of one verb \
             useless on the next command, so growing this vocabulary is a decision to \
             take on purpose: first check whether an existing flag name already means \
             what the new one means and reuse it; only if none does, bump \
             PINNED_FLAG_COUNT above and say why in the commit.",
            names.len()
        );

        // The worst-known divergence, named so this test documents it rather
        // than leaving it to be rediscovered: the acting agent is spelled
        // two different ways for the same concept. Neither is renamed here.
        let (claim, _) = resolve(&["cells", "claim"]).expect("cells.claim is in the registry");
        assert!(
            claim.properties.contains_key("worker"),
            "cells claim no longer declares --worker: {:?}",
            claim.properties.keys().collect::<Vec<_>>()
        );
        let (reserve, _) =
            resolve(&["reservations", "reserve"]).expect("reservations.reserve is in the registry");
        assert!(
            reserve.properties.contains_key("agent"),
            "reservations reserve no longer declares --agent: {:?}",
            reserve.properties.keys().collect::<Vec<_>>()
        );
    }

    /// A refusal must not hand the caller another dead end.
    #[test]
    fn group_subverbs_never_offers_an_unavailable_verb() {
        let subs = group_subverbs("state");
        assert!(subs.contains(&"gate".to_string()), "{subs:?}");
        for dead in ["compact-log", "compact-check"] {
            assert!(!subs.contains(&dead.to_string()), "{dead} is still advertised: {subs:?}");
        }
        // `bee state compact-log` still RESOLVES — the gap is reported, not
        // hidden.
        assert!(resolve(&["state", "compact-log"]).is_some());
        // advisor-ref record/show are ported (agp-1/agp-2, no longer a gap) —
        // both now belong on the advertised list, not the dead one.
        for live in ["advisor-ref record", "advisor-ref show"] {
            assert!(subs.contains(&live.to_string()), "{live} is missing: {subs:?}");
        }
    }
}
