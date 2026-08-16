// routing, `decisions active` / `search` / `log`
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::textutil::truncate_chars_head;
use crate::verbs::state_group::resolve_mutation_target;
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "decisions" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "active" => run_active_or_search(flags, use_json, t0, false),
        "search" => run_active_or_search(flags, use_json, t0, true),
        "log" => run_log(flags, use_json, t0),
        "tag" => run_tag(flags, use_json, t0),
        "redact" => run_redact(flags, use_json, t0),
        "archive" => run_archive(flags, use_json, t0),
        "render" => run_render(flags, use_json, t0),
        "supersede" => run_supersede(flags, use_json, t0),
        _ => None, // anything else: Node's
    }
}

/// A registry type:"boolean" flag: bare Present, or =true/=false. Any other
/// =value fails Node's validate() — delegate. Returns whether the flag is
/// PRESENT at all (`flags.x !== undefined`, the handlers' actual read).
pub(crate) fn bool_flag_present(flags: &Flags, name: &str) -> Option<bool> {
    match flags.get(name) {
        None => Some(false),
        Some(FlagV::Present) => Some(true),
        Some(FlagV::S(s)) if s == "true" || s == "false" => Some(true),
        Some(FlagV::S(_)) => None,
    }
}

pub(crate) fn str_flag(flags: &Flags, name: &str) -> Option<Option<String>> {
    match flags.get(name) {
        None => Some(None),
        Some(FlagV::S(s)) => Some(Some(s.clone())),
        Some(FlagV::Present) => None, // unreachable for non-boolean names
    }
}

// ─── decisions active / search ─────────────────────────────────────────────

pub(crate) fn run_active_or_search(
    flags: Flags,
    use_json: bool,
    t0: Instant,
    is_search: bool,
) -> Option<ExitCode> {
    let known: &[&str] = if is_search {
        &["text", "tag", "scope", "area", "since", "all", "untagged", "cell", "feature"]
    } else {
        &["recent", "tag", "scope", "area", "since", "all", "untagged", "cell", "feature"]
    };
    if !keys_known(&flags, known) {
        return None;
    }
    let all = bool_flag_present(&flags, "all")?;
    let untagged = bool_flag_present(&flags, "untagged")?;
    let recent_raw = if is_search { None } else { str_flag(&flags, "recent")? };
    let text_raw = if is_search { str_flag(&flags, "text")? } else { None };
    let tag_raw = str_flag(&flags, "tag")?;
    let scope_raw = str_flag(&flags, "scope")?;
    let area_raw = str_flag(&flags, "area")?;
    let since_raw = str_flag(&flags, "since")?;
    let cell_raw = str_flag(&flags, "cell")?;
    let feature_raw = str_flag(&flags, "feature")?;
    // --recent outside the modeled decimal grammar → Node's validate() error.
    if let Some(raw) = &recent_raw {
        if js_number_flag(raw).is_err() {
            return None;
        }
    }

    let cmd: &'static str = if is_search { "decisions search" } else { "decisions active" };
    let ctx = match decisions_prelude(cmd, use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        // Handler-ordered flag resolution (throws surface in this order).
        let recent: Option<f64> = match &recent_raw {
            None => None,
            Some(raw) => match js_number_flag(raw)? {
                Some(v) if v.is_finite() && v > 0.0 => Some(v),
                _ => {
                    return Ok(Out::Thrown("--recent must be a positive integer.".into()));
                }
            },
        };
        let tag = tag_raw.clone();
        let scope = match (&scope_raw, &area_raw) {
            (Some(s), _) => Some(s.clone()),
            (None, Some(a)) => Some(a.clone()),
            (None, None) => None,
        };
        let since_ms: Option<f64> = match &since_raw {
            None => None,
            Some(s) => match js_date_parse(s)? {
                None => {
                    return Ok(Out::Thrown(format!(
                        "--since must be a valid ISO date, got {}.",
                        js_quote(s)
                    )));
                }
                Some(ms) => Some(ms),
            },
        };
        let cell = cell_raw.clone();
        let feature = feature_raw.clone();
        let text = text_raw.clone();

        if is_search {
            let none_set = text.as_deref().map(|s| s.is_empty()).unwrap_or(true)
                && tag.as_deref().map(|s| s.is_empty()).unwrap_or(true)
                && scope.as_deref().map(|s| s.is_empty()).unwrap_or(true)
                && since_ms.is_none()
                && !untagged
                && cell.as_deref().map(|s| s.is_empty()).unwrap_or(true)
                && feature.as_deref().map(|s| s.is_empty()).unwrap_or(true);
            if none_set {
                return Ok(Out::Thrown(
                    "decisions search requires --text, or at least one structured filter (--tag/--scope/--area/--since/--untagged/--cell/--feature).".into(),
                ));
            }
        }

        // `if (tag)` etc — empty strings are falsy, so drop them here.
        let nonempty = |o: Option<String>| o.filter(|s| !s.is_empty());
        let filters = DecisionFilters {
            text: nonempty(text),
            tag: nonempty(tag),
            scope: nonempty(scope),
            since_ms,
            untagged,
            cell: nonempty(cell),
            feature: nonempty(feature),
        };
        let mut decisions = filter_decision_events(active_decisions(&ctx.root, all)?, &filters)?;
        if let Some(n) = recent {
            let take = if n >= decisions.len() as f64 { decisions.len() } else { n as usize };
            decisions.truncate(take);
        }
        let text_out = if decisions.is_empty() {
            if is_search {
                "No active decisions matching the given filters.".to_string()
            } else {
                "No active decisions.".to_string()
            }
        } else {
            decisions.iter().map(format_decision).collect::<Vec<_>>().join("\n")
        };
        Ok(Out::Emit(json!({ "decisions": decisions }), text_out, 0))
    })();
    finish(&ctx, out)
}

// ─── decisions log ─────────────────────────────────────────────────────────

pub(crate) fn run_log(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "decision", "rationale", "alternatives", "scope", "source", "confidence", "tags",
            "relation", "trigger",
        ],
    ) {
        return None;
    }
    let decision = flags.req_str("decision")?.to_string();
    let rationale = flags.req_str("rationale")?.to_string();
    let alternatives = flags.truthy_str("alternatives").map(str::to_string);
    let scope = flags
        .truthy_str("scope")
        .map(str::to_string)
        .unwrap_or_else(|| "repo".to_string());
    let source = flags
        .truthy_str("source")
        .map(str::to_string)
        .unwrap_or_else(|| "user".to_string());
    let confidence_raw = str_flag(&flags, "confidence")?;
    if let Some(raw) = &confidence_raw {
        if js_number_flag(raw).is_err() {
            return None; // Node's validate() owns the message
        }
    }
    let tags_flag: Option<Vec<String>> = match flags.get("tags") {
        None => None,
        Some(FlagV::S(s)) => Some(split_list(s)),
        Some(FlagV::Present) => return None,
    };
    // D3: --relation supersedes:<id>[,...] | touches:<id>[,...] | none — the
    // raw flag VALUE, `None` only when the flag is entirely absent (`Some`
    // wraps even a malformed value; do_log owns both refusals through the
    // same one-line teach so the flag stays a single required surface).
    let relation_raw: Option<String> = str_flag(&flags, "relation")?;
    // D2: --trigger <id> — a kdt-2 trigger registry id, required only when
    // the decision text itself reads as a deferral (matches_deferral_prose).
    let trigger_raw: Option<String> = str_flag(&flags, "trigger")?;

    let ctx = match decisions_prelude("decisions log", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = do_log(
        &ctx.root,
        LogParams {
            decision,
            rationale,
            alternatives,
            scope,
            source,
            confidence_raw,
            tags: tags_flag,
            relation: relation_raw,
            trigger: trigger_raw,
        },
        DECISIONS_LOCK_RETRY_ATTEMPTS,
    );
    finish(&ctx, out)
}

pub(crate) struct LogParams {
    pub(crate) decision: String,
    pub(crate) rationale: String,
    pub(crate) alternatives: Option<String>,
    pub(crate) scope: String,
    pub(crate) source: String,
    pub(crate) confidence_raw: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    /// D3 (knowledge-distill-trigger): the raw `--relation` flag value —
    /// `supersedes:<id>[,...]` | `touches:<id>[,...]` | `none`. `None` means
    /// the flag was never passed at all; do_log refuses that exactly like a
    /// malformed value, quoting up to 3 dcc-1 conflict candidates and
    /// teaching the flag in one line. Every internal (non-CLI) caller must
    /// now pass an explicit `Some("none".to_string())` — the same
    /// "declared, never silent" law the flag enforces at the CLI.
    /// `supersedes:` reuses dsh-1's resolve_supersedes_target (read.rs)
    /// against the active decide/supersede set and writes the same
    /// `supersedes` event field dsh-1 always has; `touches:` resolves the
    /// same way onto a new `touches` array that does NOT exclude its
    /// targets from `active_decisions()` (unlike `supersedes:`).
    pub(crate) relation: Option<String>,
    /// D2's write-path law: a kdt-2 trigger registry id. Required — and
    /// validated against the registry — the moment `decision` reads as a
    /// deferral (matches_deferral_prose); optional otherwise. Persisted
    /// onto the event as `trigger` whenever given and valid.
    pub(crate) trigger: Option<String>,
}

/// dsh-1's prose-supersession guard: decision text that reads as an inline
/// supersession — "supersede"/"supersedes"/"superseded", "replaces",
/// "overrides", "no longer applies", "instead of the previous" — hides the
/// earlier decision from active_decisions() forever unless it is EITHER
/// named via --supersedes here or logged through `decisions supersede`
/// (which carries its own `supersedes` field by construction and never
/// calls this). Word-bounded, case-insensitive; reuses scanners.rs's
/// hand-scanned matching primitives (no regex crate in this workspace).
pub(crate) fn matches_supersession_prose(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let whole_word = |kw: &str| -> bool {
        for i in 0..chars.len() {
            if starts_with_ci(&chars, i, kw) && boundary_before(&chars, i) {
                let end = i + kw.chars().count();
                if end == chars.len() || !is_word(chars[end]) {
                    return true;
                }
            }
        }
        false
    };
    // supersede / supersedes / superseded — shared "supersed" stem.
    const STEM: &str = "supersed";
    for i in 0..chars.len() {
        if !starts_with_ci(&chars, i, STEM) || !boundary_before(&chars, i) {
            continue;
        }
        let after = i + STEM.chars().count();
        for suffix in ["ed", "es", "e"] {
            if starts_with_ci(&chars, after, suffix) {
                let end = after + suffix.chars().count();
                if end == chars.len() || !is_word(chars[end]) {
                    return true;
                }
            }
        }
    }
    if whole_word("replaces") || whole_word("overrides") {
        return true;
    }
    for phrase in ["no longer applies", "instead of the previous"] {
        for i in 0..chars.len() {
            if starts_with_ci(&chars, i, phrase) {
                return true;
            }
        }
    }
    false
}

pub(crate) const SUPERSESSION_PROSE_GUARD_MESSAGE: &str = "logDecision: decision text reads as a supersession (\"supersede\"/\"supersedes\"/\"superseded\"/\"replaces\"/\"overrides\"/\"no longer applies\"/\"instead of the previous\") but names no earlier decision — pass --relation supersedes:<id> to retire it here, or log it through `decisions supersede` instead.";

/// D2's write-path law (mirror of dsh-1's prose guard above): decision text
/// that reads as a deferral — "defer"/"defers"/"deferred"/"deferring",
/// "for now", "revisit when"/"revisit if", or the whole word "later" — but
/// names no `--trigger`. "No deferred condition may exist outside the
/// [kdt-2] registry" (CONTEXT.md D2), so this is the only door a deferred
/// condition can enter through. Word-bounded, case-insensitive; same
/// hand-scanned primitives matches_supersession_prose uses (no regex crate
/// in this workspace).
pub(crate) fn matches_deferral_prose(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let whole_word = |kw: &str| -> bool {
        for i in 0..chars.len() {
            if starts_with_ci(&chars, i, kw) && boundary_before(&chars, i) {
                let end = i + kw.chars().count();
                if end == chars.len() || !is_word(chars[end]) {
                    return true;
                }
            }
        }
        false
    };
    // defer / defers / deferred / deferring — shared "defer" stem.
    const STEM: &str = "defer";
    for i in 0..chars.len() {
        if !starts_with_ci(&chars, i, STEM) || !boundary_before(&chars, i) {
            continue;
        }
        let after = i + STEM.chars().count();
        for suffix in ["", "s", "red", "ring"] {
            if starts_with_ci(&chars, after, suffix) {
                let end = after + suffix.chars().count();
                if end == chars.len() || !is_word(chars[end]) {
                    return true;
                }
            }
        }
    }
    if whole_word("later") {
        return true;
    }
    // "for now" — bounded "for", a whitespace run, then "now".
    for i in 0..chars.len() {
        if starts_with_ci(&chars, i, "for") && boundary_before(&chars, i) {
            let after = i + 3;
            let w = ws_run(&chars, after);
            if w > 0 && starts_with_ci(&chars, after + w, "now") {
                let end = after + w + 3;
                if end == chars.len() || !is_word(chars[end]) {
                    return true;
                }
            }
        }
    }
    // "revisit when" / "revisit if" — bounded "revisit", whitespace, then
    // either tail word, itself word-bounded on its own trailing edge.
    for i in 0..chars.len() {
        if starts_with_ci(&chars, i, "revisit") && boundary_before(&chars, i) {
            let after = i + 7;
            let w = ws_run(&chars, after);
            if w == 0 {
                continue;
            }
            for kw in ["when", "if"] {
                if starts_with_ci(&chars, after + w, kw) {
                    let end = after + w + kw.chars().count();
                    if end == chars.len() || !is_word(chars[end]) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub(crate) const DEFERRAL_WITHOUT_TRIGGER_MESSAGE: &str = "logDecision: decision text reads as a deferral (\"defer\"/\"for now\"/\"revisit when\"/\"revisit if\"/\"later\") but names no --trigger — register the condition first with `bee triggers add --decision <id> --condition \"...\"`, then retry with --trigger <that trigger id>.";

/// doc-impact-synthesis D1a: the log-time touches-sweep's own exclusion
/// list, applied to each `sweep_decision_citations` hit (a root-relative
/// path built by `path_relative`, native separators). Excludes the
/// generated decisions index — regenerated by `decisions render`, never
/// hand-fixed — and, when the logging context has a bound feature, that
/// feature's own live history dir (self-citation of live work is not
/// staleness). Supersede's own sweep carries no such exclusion.
pub(crate) fn touches_sweep_excluded(root_relative_file: &str, bound_feature: Option<&str>) -> bool {
    let native = |p: &str| p.replace('/', std::path::MAIN_SEPARATOR_STR);
    if root_relative_file == native("docs/decisions/index.md") {
        return true;
    }
    if let Some(feat) = bound_feature {
        if !feat.is_empty() && root_relative_file.starts_with(&native(&format!("docs/history/{feat}/"))) {
            return true;
        }
    }
    false
}

/// D3: the required-`--relation` refusal, same shape as the prose guard's
/// own refusal — up to 3 dcc-1 conflict candidates, then the one-line teach.
fn relation_required_message(root: &Path, p: &LogParams) -> R2<String> {
    let raw_tags: Vec<String> = p
        .tags
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|t| js_trim(t).to_string())
        .collect();
    let active = active_decisions(root, false)?;
    let candidates = conflict_candidates(&active, js_trim(&p.decision), &raw_tags, None);
    let mut msg = RELATION_REQUIRED_MESSAGE.to_string();
    msg.push_str(&conflict_candidate_lines(&candidates));
    Ok(msg)
}

pub(crate) const RELATION_REQUIRED_MESSAGE: &str = "logDecision: --relation is required — pass --relation supersedes:<id>[,...] to retire earlier decisions here, --relation touches:<id>[,...] to note a related-but-not-retired decision, or --relation none if this decision relates to nothing active.";

/// D3's declared relation, parsed from the raw `--relation` flag value.
/// `None` from `parse_relation` means a malformed value — do_log folds that
/// into the exact same required-flag refusal a missing flag gets, since
/// both leave the relation undeclared.
pub(crate) enum Relation {
    Supersedes(Vec<String>),
    Touches(Vec<String>),
    None,
}

fn parse_relation(raw: &str) -> Option<Relation> {
    let raw = js_trim(raw);
    if raw == "none" {
        return Some(Relation::None);
    }
    if let Some(rest) = raw.strip_prefix("supersedes:") {
        let ids = split_list(rest);
        return if ids.is_empty() { None } else { Some(Relation::Supersedes(ids)) };
    }
    if let Some(rest) = raw.strip_prefix("touches:") {
        let ids = split_list(rest);
        return if ids.is_empty() { None } else { Some(Relation::Touches(ids)) };
    }
    None
}

pub(crate) fn do_log(root: &Path, p: LogParams, lock_retries: u32) -> R2<Out> {
    // handleDecisionsLog's confidence gate runs before logDecision.
    let confidence: Option<f64> = match &p.confidence_raw {
        None => None,
        Some(raw) => match js_number_flag(raw)? {
            Some(v) if v.is_finite() => Some(v),
            _ => return Ok(Out::Thrown("--confidence must be an integer.".into())),
        },
    };
    // logDecision (lib/decisions.mjs).
    if js_trim(&p.decision).is_empty() {
        return Ok(Out::Thrown("logDecision: decision text is required.".into()));
    }
    if js_trim(&p.rationale).is_empty() {
        return Ok(Out::Thrown("logDecision: rationale is required.".into()));
    }
    for (field, value) in [
        ("decision", Some(p.decision.as_str())),
        ("rationale", Some(p.rationale.as_str())),
        ("alternatives", p.alternatives.as_deref()),
        ("scope", Some(p.scope.as_str())),
        ("source", Some(p.source.as_str())),
    ] {
        if let Err(msg) = assert_safe_content(field, value) {
            return Ok(Out::Thrown(msg));
        }
    }
    // D3: --relation is required — missing OR malformed both refuse the
    // same way (relation stays undeclared either way).
    let relation: Relation = match p.relation.as_deref().and_then(parse_relation) {
        Some(r) => r,
        None => return Ok(Out::Thrown(relation_required_message(root, &p)?)),
    };
    // supersedes: reuses dsh-1's resolve_supersedes_target against the
    // currently active decide/supersede set, same as --supersedes always
    // did — only the source of the raw ids moved (now --relation's value).
    let supersedes: Option<Vec<String>> = match &relation {
        Relation::Supersedes(raw_ids) => {
            let candidates = active_decide_or_supersede_candidates(root)?;
            let mut resolved: Vec<String> = Vec::new();
            for raw in raw_ids {
                let id = match resolve_supersedes_target(&candidates, raw) {
                    Ok(id) => id,
                    Err(msg) => return Ok(Out::Thrown(msg)),
                };
                if !resolved.contains(&id) {
                    resolved.push(id);
                }
            }
            Some(resolved)
        }
        _ => None,
    };
    // touches: resolved the same way (full id or unique short8 against the
    // active decide/supersede set) but persisted onto its OWN `touches`
    // array — unlike `supersedes`, a touched id stays in active_decisions().
    let touches: Option<Vec<String>> = match &relation {
        Relation::Touches(raw_ids) => {
            let candidates = active_decide_or_supersede_candidates(root)?;
            let mut resolved: Vec<String> = Vec::new();
            for raw in raw_ids {
                match resolve_supersedes_target(&candidates, raw) {
                    Ok(id) => {
                        if !resolved.contains(&id) {
                            resolved.push(id);
                        }
                    }
                    Err(_) => {
                        return Ok(Out::Thrown(format!(
                            "decisions log: --relation touches:{} does not resolve to any active decide/supersede event.",
                            js_quote(raw)
                        )));
                    }
                }
            }
            Some(resolved)
        }
        _ => None,
    };
    // dsh-1's prose-supersession guard: refuses inline supersession prose
    // that names no earlier decision — still wins over --relation none (or
    // touches), only an actual resolved --relation supersedes:<id> silences
    // it.
    if supersedes.is_none() && matches_supersession_prose(js_trim(&p.decision)) {
        // dcc-1: the refusal names its own conflict candidates too, so the
        // fix command (--relation supersedes:<id> / `decisions supersede`)
        // is ready-made. Uses the raw (trimmed, not yet slug-validated)
        // tags — matching purposes only, this refusal never writes the
        // event either way.
        let raw_tags: Vec<String> = p
            .tags
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|t| js_trim(t).to_string())
            .collect();
        let active = active_decisions(root, false)?;
        let candidates =
            conflict_candidates(&active, js_trim(&p.decision), &raw_tags, None);
        let mut msg = SUPERSESSION_PROSE_GUARD_MESSAGE.to_string();
        msg.push_str(&conflict_candidate_lines(&candidates));
        return Ok(Out::Thrown(msg));
    }
    // D2's write-path law: a deferral-shaped decision names its --trigger,
    // and that trigger must already be registered (kdt-2). A given-but-bad
    // id refuses regardless of prose shape; a well-formed one is validated
    // and persisted either way.
    let trigger_id: Option<String> = match p.trigger.as_deref().map(js_trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(id) => {
            if !crate::verbs::triggers::trigger_registered(root, id) {
                return Ok(Out::Thrown(format!(
                    "decisions log: --trigger {} does not name a registered trigger — run `bee triggers add --decision <id> --condition \"...\"` first, then retry with --trigger <that id>.",
                    js_quote(id)
                )));
            }
            Some(id.to_string())
        }
    };
    if trigger_id.is_none() && matches_deferral_prose(js_trim(&p.decision)) {
        return Ok(Out::Thrown(DEFERRAL_WITHOUT_TRIGGER_MESSAGE.to_string()));
    }
    let normalized = match normalize_tags(p.tags.clone()) {
        Ok(n) => n,
        Err(msg) => return Ok(Out::Thrown(msg)),
    };
    // classifyDecisionTags(root, normalizedTags || []) — taxonomy-present
    // refusal / unknown-tag candidates append (dp-6, D7b).
    classify_decision_tags(root, &normalized.clone().unwrap_or_default(), lock_retries)?;

    // doc-impact-synthesis D1a: the calling context's active feature —
    // resolved exactly as `state route` resolves it (session-bound lane,
    // else the default `.bee/state.json` record) — stamps onto the new
    // event below and bounds the touches-sweep's own-history exclusion.
    // No resolution (no bound lane, no `feature` on the default record) →
    // `None`, and the event carries no `feature` field at all.
    let bound_feature: Option<String> = {
        let target = resolve_mutation_target(root, None, "decisions log", false)?;
        match target.record().get("feature") {
            Some(v) if truthy(v) => Some(js_disp(v)),
            _ => None,
        }
    };

    let mut event = Map::new();
    event.insert("id".into(), Value::String(pseudo_uuid_v4()));
    event.insert("type".into(), Value::String("decide".into()));
    event.insert("date".into(), Value::String(now_iso()));
    event.insert("decision".into(), Value::String(js_trim(&p.decision).to_string()));
    event.insert("rationale".into(), Value::String(js_trim(&p.rationale).to_string()));
    event.insert(
        "alternatives".into(),
        p.alternatives.clone().map(Value::String).unwrap_or(Value::Null),
    );
    event.insert("scope".into(), Value::String(p.scope.clone()));
    event.insert("source".into(), Value::String(p.source.clone()));
    event.insert(
        "confidence".into(),
        confidence
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
    );
    if let Some(tags) = &normalized {
        event.insert(
            "tags".into(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(ids) = &supersedes {
        event.insert(
            "supersedes".into(),
            Value::Array(ids.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(ids) = &touches {
        event.insert(
            "touches".into(),
            Value::Array(ids.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(id) = &trigger_id {
        event.insert("trigger".into(), Value::String(id.clone()));
    }
    // D3: every new decide event carries its declared relation explicitly —
    // legacy (pre-D3) lines simply lack the field, and readers tolerate
    // that absence.
    let relation_str = match &relation {
        Relation::Supersedes(_) => "supersedes",
        Relation::Touches(_) => "touches",
        Relation::None => "none",
    };
    event.insert("relation".into(), Value::String(relation_str.into()));
    // doc-impact-synthesis D1a: `feature` rides every new decide event the
    // same way `relation` does — present only when the calling context
    // resolved one. Legacy (pre-D1a) lines simply lack the field, and every
    // reader tolerates that absence.
    if let Some(feat) = &bound_feature {
        event.insert("feature".into(), Value::String(feat.clone()));
    }
    let mut event = Value::Object(event);

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    append_jsonl(&decisions_path(root), &event).map_err(|_| Err2::Ex)?;
    drop(guard);

    // dcc-1: computed AFTER the append (so it reads the same active-set
    // shape every other active_decisions() caller sees) and excluded by id
    // so the just-written event is never its own candidate. Never persisted
    // — `append_jsonl` already wrote the event above without this field.
    let new_id = js_disp_opt(jget(&event, "id"));

    // doc-impact-synthesis D1a: log-time touches-sweep. Each resolved
    // `touches:` id walks its own declared citations under docs/**
    // (`sweep_decision_citations`, the exact scan `decisions supersede`
    // already runs) and every surviving hit — the generated decisions
    // index and the logging feature's own live history excluded — enqueues
    // a must-fix capture stub. Supersede's own sweep (`do_supersede`
    // above) is untouched by this addition.
    if let Some(ids) = &touches {
        for touched_id in ids {
            let short8 = truncate_chars_head(touched_id, 8);
            let sweep = sweep_decision_citations(root, touched_id, &short8);
            let hits: Vec<Value> = match jget(&sweep, "files") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            for hit in &hits {
                let file = js_disp_opt(jget(hit, "file"));
                if touches_sweep_excluded(&file, bound_feature.as_deref()) {
                    continue;
                }
                let line = js_disp_opt(jget(hit, "line"));
                let outcome = format!(
                    "{file}:{line} cites decision {touched_id}, now touched by {new_id} — reconcile the citing doc against the touching decision."
                );
                add_capture_stub(
                    root,
                    &outcome,
                    &[touched_id.clone(), new_id.clone()],
                    &[file],
                    "touches-sweep",
                )?;
            }
        }
    }

    let active = active_decisions(root, false)?;
    let candidate_tags = normalized.clone().unwrap_or_default();
    let candidates =
        conflict_candidates(&active, js_trim(&p.decision), &candidate_tags, Some(&new_id));

    // dp-6 warn-only path (handleDecisionsLog).
    let warning = if !taxonomy_file_exists(root) && normalized.is_none() {
        "\nWarning: no taxonomy.json found — this decision was logged without tags. Create docs/decisions/taxonomy.json to require classification going forward."
    } else {
        ""
    };
    let mut text = format!("Logged decision {new_id}.{warning}");
    text.push_str(&conflict_candidate_lines(&candidates));

    if let Value::Object(m) = &mut event {
        m.insert("conflict_candidates".into(), Value::Array(candidates));
    }
    Ok(Out::Emit(event, text, 0))
}
