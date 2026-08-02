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
fn distance(a: &str, b: &str) -> usize {
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

    /// A refusal must not hand the caller another dead end.
    #[test]
    fn group_subverbs_never_offers_an_unavailable_verb() {
        let subs = group_subverbs("state");
        assert!(subs.contains(&"gate".to_string()), "{subs:?}");
        for dead in ["compact-log", "compact-check", "advisor-ref show"] {
            assert!(!subs.contains(&dead.to_string()), "{dead} is still advertised: {subs:?}");
        }
        // `bee state compact-log` still RESOLVES — the gap is reported, not
        // hidden.
        assert!(resolve(&["state", "compact-log"]).is_some());
    }
}
