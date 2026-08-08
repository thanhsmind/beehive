// routing, `decisions active` / `search` / `log`
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
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
    let ctx = match crate::verbs::reservations::prelude(cmd, use_json, t0)? {
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
            "supersedes",
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
    // dsh-1: --supersedes, same comma-separated-array shape as --tags.
    let supersedes_flag: Option<Vec<String>> = match flags.get("supersedes") {
        None => None,
        Some(FlagV::S(s)) => Some(split_list(s)),
        Some(FlagV::Present) => return None,
    };

    let ctx = match crate::verbs::reservations::prelude("decisions log", use_json, t0)? {
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
            supersedes: supersedes_flag,
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
    /// dsh-1 (decision-supersede-hygiene): full ids or unique short8s of
    /// decide/supersede events this decision retires. Resolved against the
    /// currently ACTIVE set only (see read.rs's active_decide_or_supersede_
    /// candidates/resolve_supersedes_target) and written onto the decide
    /// event as a `supersedes` array — active_decisions() then excludes
    /// every named target, the same way a type=="supersede" event's single
    /// `supersedes` string always has.
    pub(crate) supersedes: Option<Vec<String>>,
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

pub(crate) const SUPERSESSION_PROSE_GUARD_MESSAGE: &str = "logDecision: decision text reads as a supersession (\"supersede\"/\"supersedes\"/\"superseded\"/\"replaces\"/\"overrides\"/\"no longer applies\"/\"instead of the previous\") but names no earlier decision — pass --supersedes <id> to retire it here, or log it through `decisions supersede` instead.";

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
    // dsh-1: resolve --supersedes against the currently active decide/
    // supersede set. Runs BEFORE the prose guard, so the flag's presence —
    // not merely its shape — is what silences the guard below.
    let supersedes: Option<Vec<String>> = match &p.supersedes {
        None => None,
        Some(raw_ids) if raw_ids.is_empty() => None,
        Some(raw_ids) => {
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
    };
    // dsh-1's prose-supersession guard: refuses inline supersession prose
    // that names no earlier decision via --supersedes.
    if supersedes.is_none() && matches_supersession_prose(js_trim(&p.decision)) {
        return Ok(Out::Thrown(SUPERSESSION_PROSE_GUARD_MESSAGE.into()));
    }
    let normalized = match normalize_tags(p.tags.clone()) {
        Ok(n) => n,
        Err(msg) => return Ok(Out::Thrown(msg)),
    };
    // classifyDecisionTags(root, normalizedTags || []) — taxonomy-present
    // refusal / unknown-tag candidates append (dp-6, D7b).
    classify_decision_tags(root, &normalized.clone().unwrap_or_default(), lock_retries)?;

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
    let event = Value::Object(event);

    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    append_jsonl(&decisions_path(root), &event).map_err(|_| Err2::Ex)?;
    drop(guard);

    // dp-6 warn-only path (handleDecisionsLog).
    let warning = if !taxonomy_file_exists(root) && normalized.is_none() {
        "\nWarning: no taxonomy.json found — this decision was logged without tags. Create docs/decisions/taxonomy.json to require classification going forward."
    } else {
        ""
    };
    let text = format!("Logged decision {}.{warning}", js_disp_opt(jget(&event, "id")));
    Ok(Out::Emit(event, text, 0))
}
