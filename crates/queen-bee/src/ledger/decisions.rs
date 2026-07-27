//! The `decisions` group — the WRITE half of `.bee/decisions.jsonl`'s CLI
//! surface (rpl-4).
//!
//! Port of `packages/bee/bee.mjs:1932-2001` (`handleDecisionsLog`,
//! `handleDecisionsSupersede`, `handleDecisionsRedact`) and `:6638`
//! (`decisionsUsageFallback`). Presentation ONLY: every rule — the D9 lock
//! retry, the tag taxonomy gate, the `docs/**` citation sweep, the content
//! refusals and the fixed key order — lives in [`bee_core::decisions`], and
//! the queued stubs go through [`bee_core::capture`], exactly as the mjs
//! handlers delegate to `lib/decisions.mjs` and `lib/capture.mjs`.
//!
//! # Seven verbs now, deliberately not eight
//!
//! rpl-5 adds the QUERY half — `active`, `search`, `archive`, `tag`
//! (`bee.mjs:2113-2229`) plus the `dp-1` filter stack they share
//! (`bee.mjs:2049-2110`). `render` alone is still unported: it was split out
//! into rpl-10 on review, so it keeps resolving in the registry, finding no
//! handler here, and taking `dispatch.rs`'s honest "known bee command but is
//! not ported into this binary yet" refusal rather than a guess. The group's
//! `usage_fallback` names ALL EIGHT verbs regardless, because that is what
//! mjs prints — the fallback describes the bee CLI, not this binary's
//! progress.
//!
//! # Where each rule lives
//!
//! `active`/`search` are pure presentation over
//! [`bee_core::decisions::active_decisions`] plus the filter stack below;
//! `archive`/`tag` are read-modify-write and delegate every rule — the D9
//! lock, the all-or-nothing batch validation, the crash ordering — to
//! [`bee_core::decisions`], exactly as the mjs handlers delegate to
//! `lib/decisions.mjs`.
//!
//! # Two ranking comparators, reproduced at their own call sites
//!
//! `activeDecisions` sorts by date-descending with an index tiebreak
//! (`decisions.mjs:873`), and `filterDecisionEvents`'s `--text` branch sorts
//! by term-HIT-COUNT descending over that already-ordered list
//! (`bee.mjs:2093`). They are NOT one comparator: the second is a STABLE
//! re-sort that inherits the first's order as its secondary key, which is
//! precisely why "hit count desc, then date desc" needs no date comparison
//! of its own. Unifying them would have to invent a date comparison the mjs
//! source does not perform, so each stays at its own call site.
//!
//! # Wall clock, stamped at this edge
//!
//! `supersede` reads the clock TWICE in mjs — once for the sweep's
//! `scanned_at` (`decisions.mjs:404`) and once for the event's own `date`
//! (`:471`), in that order — so both are threaded in separately rather than
//! collapsed into one stamp. Every stamp uses
//! [`bee_core::lock::iso8601_millis`]'s exact `toISOString()` shape, which
//! is also the shape `bee_parity::normalize`'s `IsoMillisZ` gate demands: a
//! wrong-precision stamp fails the parity diff instead of masking into
//! silence.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bee_core::capture::{add_capture_stub_lists, random_uuid};
use bee_core::datamark::{datamark, js_trim};
use bee_core::decisions::{
    active_decisions, archive_decisions, canonicalize, log_decision, redact_decision,
    supersede_decision, tag_decision, tag_decisions_batch, taxonomy_file_exists, LogFields,
    SupersedeFields,
};
use bee_core::jsdate::date_parse_ms;
use serde_json::{json, Value};

use crate::dispatch::{emit, emit_error, Call, GroupDef, VerbDef};
use crate::ledger::require_flag;

/// `bee.mjs:6638` `decisionsUsageFallback`. Byte-exact, including the
/// `(missing)` placeholder for a bare `bee decisions`.
fn usage_fallback(leading: &[String]) -> String {
    let verb = leading.get(1).map(String::as_str).unwrap_or("(missing)");
    let verb = if verb.is_empty() { "(missing)" } else { verb };
    format!("Unknown command \"{verb}\". Use: log, supersede, redact, active, search, archive, tag, render.")
}

/// The registration this group contributes. ONE line in
/// [`crate::groups::register_all`] pulls it in.
pub fn group() -> GroupDef {
    GroupDef {
        group: "decisions",
        usage_fallback: Some(usage_fallback),
        verbs: vec![
            VerbDef { verb: "log", handler: handle_log },
            VerbDef { verb: "supersede", handler: handle_supersede },
            VerbDef { verb: "redact", handler: handle_redact },
            VerbDef { verb: "active", handler: handle_active },
            VerbDef { verb: "search", handler: handle_search },
            VerbDef { verb: "archive", handler: handle_archive },
            VerbDef { verb: "tag", handler: handle_tag },
        ],
    }
}

fn now_iso() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    bee_core::lock::iso8601_millis(ms)
}

// ─── the JS coercions the handlers perform implicitly ──────────────────────

/// `String(x)` over the shapes `parseFlags` can produce.
fn js_string_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// `flags.<name> !== undefined ? String(flags.<name>) : undefined` — PRESENCE,
/// not truthiness. Distinct from [`crate::ledger::optional_flag`], which
/// folds the empty string to `None`: `--tags ""` is present-and-empty, and
/// mjs takes the `splitList("")` branch for it.
fn present_flag(call: &Call, name: &str) -> Option<String> {
    call.flag(name).map(js_string_value)
}

/// `flags.<name> ? String(flags.<name>) : <default>` — JS TRUTHINESS, which
/// is what `log`'s `scope`/`source`/`alternatives` use (unlike `supersede`'s
/// `scope`, which is a presence check).
fn truthy_flag(call: &Call, name: &str) -> Option<String> {
    match call.flag(name) {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Bool(true)) => Some("true".to_string()),
        Some(Value::Number(n)) => {
            // JS: 0 is falsy, every other number truthy.
            if n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true) {
                Some(n.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `bee.mjs:2240` `splitList`: split on `,`, trim, drop the falsy.
fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `Number.parseInt(str, 10)`, returning `None` for the `NaN` that
/// `Number.isFinite` then rejects.
///
/// This is deliberately NOT `Number(str)`: `parseInt` consumes the LONGEST
/// leading integer and stops, so `"12.7"` is `12` and `"1e3"` is `1` — both
/// values the registry's `type: "number"` check happily lets through, and
/// both places where a `Number()`-based port would silently diverge.
fn js_parse_int_10(raw: &str) -> Option<i64> {
    let s = js_trim(raw);
    let mut chars = s.chars().peekable();
    let mut negative = false;
    match chars.peek() {
        Some('+') => {
            chars.next();
        }
        Some('-') => {
            negative = true;
            chars.next();
        }
        _ => {}
    }
    let digits: String = chars.take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    // A run longer than i64 becomes a float in JS and stays finite; saturating
    // keeps this total rather than panicking on a pathological argv.
    let magnitude: i64 = digits.parse().unwrap_or(i64::MAX);
    Some(if negative { -magnitude } else { magnitude })
}

// ─── the handlers ──────────────────────────────────────────────────────────

/// `bee.mjs:1932` `handleDecisionsLog`.
///
/// The `--confidence` parse runs FIRST — it is a statement above the object
/// literal, so a bad `--confidence` is reported even when `--decision` is
/// also blank.
fn handle_log(call: &Call) -> ExitCode {
    let confidence = match present_flag(call, "confidence") {
        None => None,
        Some(raw) => match js_parse_int_10(&raw) {
            Some(n) => Some(n),
            None => return emit_error("--confidence must be an integer.", call.json),
        },
    };
    let decision = match require_flag(call, "decision") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let rationale = match require_flag(call, "rationale") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let alternatives = truthy_flag(call, "alternatives");
    let scope = truthy_flag(call, "scope").unwrap_or_else(|| "repo".to_string());
    let source = truthy_flag(call, "source").unwrap_or_else(|| "user".to_string());
    let tags = present_flag(call, "tags").map(|raw| split_list(&raw));

    let fields = LogFields {
        decision: &decision,
        rationale: &rationale,
        alternatives: alternatives.as_deref(),
        scope: &scope,
        source: &source,
        confidence,
        tags: tags.as_deref(),
    };
    let id = match random_uuid() {
        Ok(id) => id,
        Err(e) => return emit_error(&e, call.json),
    };
    match log_decision(&call.root, &fields, &id, &now_iso()) {
        Ok(event) => {
            // dp-6 bootstrap-safe warn-only path. The warning is TEXT only —
            // `--json` stays data-only, per emit()'s result-vs-text split.
            let untagged = !event
                .get("tags")
                .and_then(Value::as_array)
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            let warning = if !taxonomy_file_exists(&call.root) && untagged {
                "\nWarning: no taxonomy.json found — this decision was logged without tags. Create docs/decisions/taxonomy.json to require classification going forward."
            } else {
                ""
            };
            let id = event.get("id").map(js_string_value).unwrap_or_default();
            emit(&event, &format!("Logged decision {id}.{warning}"), 0, call.json)
        }
        Err(message) => emit_error(&message, call.json),
    }
}

/// `bee.mjs:1966` `handleDecisionsSupersede`.
///
/// The capture-stub queueing lives HERE, not in `lib/decisions.mjs`: the mjs
/// comment at `:1958` records why (importing `addCaptureStub` into
/// `decisions.mjs` would close a module cycle, since `capture.mjs` already
/// imports the pattern tables FROM `decisions.mjs`). The lock doctrine is
/// unaffected — the sweep is computed and written inside the event, once;
/// this loop is a downstream side effect over the result the event already
/// carries.
fn handle_supersede(call: &Call) -> ExitCode {
    let supersedes = match require_flag(call, "id") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let decision = match require_flag(call, "decision") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let rationale = match require_flag(call, "rationale") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let tags = present_flag(call, "tags").map(|raw| split_list(&raw));
    let scope = present_flag(call, "scope");

    let fields = SupersedeFields {
        supersedes: &supersedes,
        decision: &decision,
        rationale: &rationale,
        tags: tags.as_deref(),
        scope: scope.as_deref(),
    };
    let id = match random_uuid() {
        Ok(id) => id,
        Err(e) => return emit_error(&e, call.json),
    };
    // Two clock reads, sweep first — see the module doc.
    let scanned_at = now_iso();
    let date = now_iso();
    let event = match supersede_decision(&call.root, &fields, &id, &scanned_at, &date) {
        Ok(event) => event,
        Err(message) => return emit_error(&message, call.json),
    };

    // `event.sweep?.files || []`.
    let hits: Vec<Value> = event
        .get("sweep")
        .and_then(|s| s.get("files"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let event_id = event.get("id").map(js_string_value).unwrap_or_default();
    let superseded = event.get("supersedes").map(js_string_value).unwrap_or_default();

    for hit in &hits {
        let file = hit.get("file").map(js_string_value).unwrap_or_default();
        let line = hit.get("line").map(js_string_value).unwrap_or_default();
        let outcome = format!(
            "{file}:{line} still cites superseded decision {superseded} — reconcile against replacement {event_id}."
        );
        let stub_id = match random_uuid() {
            Ok(id) => id,
            Err(e) => return emit_error(&e, call.json),
        };
        if let Err(message) = add_capture_stub_lists(
            &call.root,
            &outcome,
            &[superseded.clone(), event_id.clone()],
            &[file],
            Some("supersede-sweep"),
            &stub_id,
            &now_iso(),
        ) {
            return emit_error(&message, call.json);
        }
    }

    let header = format!("Superseded {superseded} with {event_id}.");
    let mut lines = vec![header];
    if hits.is_empty() {
        lines.push("Propagation sweep: no citations found under docs/**.".to_string());
    } else {
        lines.push(format!(
            "Propagation sweep: {} citation(s) found under docs/** — a capture stub was queued for each.",
            hits.len()
        ));
        for hit in &hits {
            lines.push(format!(
                "  {}:{}  {}",
                hit.get("file").map(js_string_value).unwrap_or_default(),
                hit.get("line").map(js_string_value).unwrap_or_default(),
                hit.get("excerpt").map(js_string_value).unwrap_or_default(),
            ));
        }
    }
    emit(&event, &lines.join("\n"), 0, call.json)
}

/// `bee.mjs:1995` `handleDecisionsRedact`.
fn handle_redact(call: &Call) -> ExitCode {
    let redacts = match require_flag(call, "id") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let reason = match require_flag(call, "reason") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let id = match random_uuid() {
        Ok(id) => id,
        Err(e) => return emit_error(&e, call.json),
    };
    match redact_decision(&call.root, &redacts, &reason, &id, &now_iso()) {
        Ok(event) => {
            let redacted = event.get("redacts").map(js_string_value).unwrap_or_default();
            emit(&event, &format!("Redacted {redacted}."), 0, call.json)
        }
        Err(message) => emit_error(&message, call.json),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// THE QUERY SIDE (rpl-5)
// ═══════════════════════════════════════════════════════════════════════════

/// JS truthiness over the shapes a stored event field can hold — the test
/// `if (event.alternatives)`, `if (scope)` and friends all perform.
fn js_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Some(Value::Array(_)) | Some(Value::Object(_)) => true,
    }
}

/// `String(x)` for a value read out of the STORE (as opposed to out of
/// argv). Distinct from [`js_string_value`] in exactly one place that
/// matters: an object renders `[object Object]` in JS, never its JSON.
fn js_display_value(v: &Value) -> String {
    match v {
        Value::Object(_) => "[object Object]".to_string(),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(), // `String([null])` is `""`
                other => js_display_value(other),
            })
            .collect::<Vec<_>>()
            .join(","),
        other => js_string_value(other),
    }
}

/// A `${…}` template slot: an ABSENT key renders `undefined`, a JSON null
/// renders `null`. (`datamark`'s own `?? ''` fold is separate — see
/// [`datamark_value`].)
fn js_template_value(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => js_display_value(v),
    }
}

/// `datamark(text)` including its `String(text ?? '')` fold, so a missing or
/// null field neutralizes to the empty `«»` rather than to `«undefined»`.
fn datamark_value(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => datamark(""),
        Some(v) => datamark(&js_display_value(v)),
    }
}

/// `bee.mjs:385` `formatDecision` — the human render both `active` and
/// `search` print. Every resurfaced field goes through `datamark`, so a
/// stored decision can never act as instructions when it is read back.
fn format_decision(event: &Value) -> String {
    let head = format!(
        "[{}] {} (id {}, {})",
        js_template_value(event.get("date")),
        datamark_value(event.get("decision")),
        js_template_value(event.get("id")),
        js_template_value(event.get("type")),
    );
    let mut lines = vec![head, format!("  why: {}", datamark_value(event.get("rationale")))];
    if js_truthy(event.get("alternatives")) {
        lines.push(format!("  alternatives: {}", datamark_value(event.get("alternatives"))));
    }
    lines.join("\n")
}

// ─── the dp-1 filter stack (`bee.mjs:2049-2110`) ───────────────────────────

/// JS `\w` plus `-`: the character class `matchesWholeToken`'s two
/// lookarounds exclude on BOTH edges.
fn is_token_boundary_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// `bee.mjs:2046` `matchesWholeToken`:
/// `new RegExp('(?<![\\w-])' + escapeRegExp(token) + '(?![\\w-])', 'i')`.
///
/// A plain `\b` is NOT enough for a hyphenated id: `-` is itself a non-word
/// character, so `\b` would treat a following hyphen as a boundary and let
/// `si-1` match inside `si-1-extra`. The lookarounds exclude `-` alongside
/// `\w` on both edges, which is what makes `si-1` exclude BOTH `si-10` and
/// `si-1-extra`. Case folding reuses [`bee_core::decisions::canonicalize`]
/// rather than a second copy of the ECMA rule.
fn matches_whole_token(haystacks: &[String], token: &str) -> bool {
    let needle: Vec<char> = token.chars().map(canonicalize).collect();
    if needle.is_empty() {
        // An empty literal matches at any position whose neighbours are both
        // outside `[\w-]` — including position 0 of an empty string.
        return haystacks
            .iter()
            .any(|h| h.chars().next().map(|c| !is_token_boundary_char(c)).unwrap_or(true));
    }
    for haystack in haystacks {
        let chars: Vec<char> = haystack.chars().collect();
        if chars.len() < needle.len() {
            continue;
        }
        for i in 0..=(chars.len() - needle.len()) {
            if !(0..needle.len()).all(|k| canonicalize(chars[i + k]) == needle[k]) {
                continue;
            }
            if i > 0 && is_token_boundary_char(chars[i - 1]) {
                continue;
            }
            if chars.get(i + needle.len()).copied().is_some_and(is_token_boundary_char) {
                continue;
            }
            return true;
        }
    }
    false
}

/// The three prose fields `--cell`/`--feature` search:
/// `[decision, rationale, alternatives].filter(v => v !== null && v !== undefined && v !== '').map(String)`.
fn prose_haystacks(event: &Value) -> Vec<String> {
    ["decision", "rationale", "alternatives"]
        .iter()
        .filter_map(|k| event.get(*k))
        .filter(|v| !matches!(v, Value::Null) && !matches!(v, Value::String(s) if s.is_empty()))
        .map(js_display_value)
        .collect()
}

/// Every filter value here is ALREADY truthiness-folded (an empty string is
/// `None`), because every `if (…)` guard in `filterDecisionEvents` and
/// `handleDecisionsSearch`'s "at least one filter" check is a truthiness
/// test, not a presence test.
#[derive(Default)]
struct DecisionFilters {
    text: Option<String>,
    tag: Option<String>,
    scope: Option<String>,
    since: Option<String>,
    untagged: bool,
    cell: Option<String>,
    feature: Option<String>,
}

impl DecisionFilters {
    /// `!text && !tag && !scope && !since && !untagged && !cell && !feature`.
    fn is_empty(&self) -> bool {
        self.text.is_none()
            && self.tag.is_none()
            && self.scope.is_none()
            && self.since.is_none()
            && !self.untagged
            && self.cell.is_none()
            && self.feature.is_none()
    }
}

fn has_tags(event: &Value) -> bool {
    event.get("tags").and_then(Value::as_array).is_some_and(|a| !a.is_empty())
}

/// `bee.mjs:2058` `filterDecisionEvents`. Filter ORDER is the mjs source's
/// (untagged, cell, feature, tag, scope, since, text) and is observable: the
/// `--text` branch RE-ORDERS what survives, so it has to run last over an
/// already-narrowed list.
fn filter_decision_events(decisions: Vec<Value>, f: &DecisionFilters) -> Vec<Value> {
    let mut result = decisions;
    if f.untagged {
        result.retain(|e| !has_tags(e));
    }
    if let Some(token) = &f.cell {
        result.retain(|e| matches_whole_token(&prose_haystacks(e), token));
    }
    if let Some(token) = &f.feature {
        result.retain(|e| matches_whole_token(&prose_haystacks(e), token));
    }
    if let Some(tag) = &f.tag {
        let needle = tag.to_lowercase();
        result.retain(|e| {
            e.get("tags")
                .and_then(Value::as_array)
                .is_some_and(|tags| tags.iter().any(|t| js_display_value(t).to_lowercase() == needle))
        });
    }
    if let Some(scope) = &f.scope {
        let needle = scope.to_lowercase();
        result.retain(|e| e.get("scope").and_then(Value::as_str).is_some_and(|s| s.to_lowercase() == needle));
    }
    if let Some(since) = &f.since {
        // `Date.parse(since)` was already validated by `resolveSinceFilter`;
        // an event whose own date is unparseable is dropped, matching
        // `Number.isFinite(eventMs) && eventMs >= sinceMs`.
        let since_ms = date_parse_ms(since);
        result.retain(|e| match (e.get("date").and_then(Value::as_str).and_then(date_parse_ms), since_ms) {
            (Some(event_ms), Some(since_ms)) => event_ms >= since_ms,
            _ => false,
        });
    }
    if let Some(text) = &f.text {
        // dp-6 D8b: whitespace-split, case-insensitive, OR across terms,
        // matched over decision/rationale/alternatives AND tags.
        let terms: Vec<String> = text
            .to_lowercase()
            .split(bee_core::datamark::is_js_whitespace)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .collect();
        let mut scored: Vec<(Value, usize)> = result
            .into_iter()
            .map(|event| {
                let mut haystacks = prose_haystacks(&event);
                if let Some(tags) = event.get("tags").and_then(Value::as_array) {
                    haystacks.extend(
                        tags.iter()
                            .filter(|v| !matches!(v, Value::Null) && !matches!(v, Value::String(s) if s.is_empty()))
                            .map(js_display_value),
                    );
                }
                let haystacks: Vec<String> = haystacks.iter().map(|h| h.to_lowercase()).collect();
                let hit_count = terms.iter().filter(|term| haystacks.iter().any(|h| h.contains(*term))).count();
                (event, hit_count)
            })
            .filter(|(_, hit_count)| *hit_count > 0)
            .collect();
        // THE SECOND COMPARATOR (see the module doc): hit count descending
        // only. `Array.prototype.sort` is spec-stable since ES2019, so the
        // incoming newest-first order survives as the secondary key — which
        // is exactly why no date comparison appears here.
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        result = scored.into_iter().map(|(event, _)| event).collect();
    }
    result
}

/// `bee.mjs:2113` `resolveScopeFilter`: `--scope` and `--area` are ONE
/// filter, not two dimensions, and an explicit `--scope` wins.
fn resolve_scope_filter(call: &Call) -> Option<String> {
    present_flag(call, "scope").or_else(|| present_flag(call, "area"))
}

/// `bee.mjs:2119` `resolveSinceFilter`. Validation runs BEFORE the
/// truthiness fold, so `--since ""` refuses rather than reading as "no
/// filter".
fn resolve_since_filter(call: &Call) -> Result<Option<String>, String> {
    let Some(since) = present_flag(call, "since") else { return Ok(None) };
    if date_parse_ms(&since).is_none() {
        return Err(format!(
            "--since must be a valid ISO date, got {}.",
            serde_json::to_string(&since).unwrap_or_else(|_| format!("\"{since}\""))
        ));
    }
    Ok(Some(since))
}

/// The five flags `active` and `search` resolve identically. `--all` and
/// `--untagged` are PRESENCE checks (`flags.x !== undefined`), which is why
/// even `--all=false` turns the branch on — mjs's own shape.
fn shared_filters(call: &Call) -> Result<(DecisionFilters, bool), String> {
    let filters = DecisionFilters {
        text: None,
        tag: present_flag(call, "tag").filter(|s| !s.is_empty()),
        scope: resolve_scope_filter(call).filter(|s| !s.is_empty()),
        since: resolve_since_filter(call)?.filter(|s| !s.is_empty()),
        untagged: call.flag("untagged").is_some(),
        cell: present_flag(call, "cell").filter(|s| !s.is_empty()),
        feature: present_flag(call, "feature").filter(|s| !s.is_empty()),
    };
    Ok((filters, call.flag("all").is_some()))
}

fn render_decisions(decisions: &[Value], empty_text: &str) -> String {
    if decisions.is_empty() {
        empty_text.to_string()
    } else {
        decisions.iter().map(format_decision).collect::<Vec<_>>().join("\n")
    }
}

/// `bee.mjs:2128` `handleDecisionsActive`.
///
/// `--recent` is applied AFTER filtering (never handed down to
/// `activeDecisions`, which would slice before the filters ran).
fn handle_active(call: &Call) -> ExitCode {
    let recent = match present_flag(call, "recent") {
        None => None,
        Some(raw) => match js_parse_int_10(&raw) {
            Some(n) if n > 0 => Some(n as usize),
            _ => return emit_error("--recent must be a positive integer.", call.json),
        },
    };
    let (filters, all) = match shared_filters(call) {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let mut decisions = filter_decision_events(active_decisions(&call.root, None, all), &filters);
    if let Some(n) = recent {
        decisions.truncate(n);
    }
    let text = render_decisions(&decisions, "No active decisions.");
    emit(&json!({ "decisions": decisions }), &text, 0, call.json)
}

/// `bee.mjs:2146` `handleDecisionsSearch`.
fn handle_search(call: &Call) -> ExitCode {
    let (mut filters, all) = match shared_filters(call) {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    filters.text = present_flag(call, "text").filter(|s| !s.is_empty());
    if filters.is_empty() {
        return emit_error(
            "decisions search requires --text, or at least one structured filter (--tag/--scope/--area/--since/--untagged/--cell/--feature).",
            call.json,
        );
    }
    let decisions = filter_decision_events(active_decisions(&call.root, None, all), &filters);
    let text = render_decisions(&decisions, "No active decisions matching the given filters.");
    emit(&json!({ "decisions": decisions }), &text, 0, call.json)
}

/// `bee.mjs:2169` `handleDecisionsArchive`. Presentation only: every rule —
/// the required cutoff, the two archiving rules, the typed
/// nothing-qualifies refusal, the crash ordering and the D9 lock — lives in
/// [`bee_core::decisions::archive_decisions`].
fn handle_archive(call: &Call) -> ExitCode {
    let before = present_flag(call, "before");
    match archive_decisions(&call.root, before.as_deref()) {
        Ok(result) => {
            let archived = result.get("archived").and_then(Value::as_array).map(Vec::len).unwrap_or(0);
            let kept = js_template_value(result.get("kept"));
            let before = js_template_value(result.get("before"));
            let text = format!(
                "Archived {archived} decision(s) to .bee/decisions-archive.jsonl (kept {kept} active, cutoff {before})."
            );
            emit(&result, &text, 0, call.json)
        }
        Err(message) => emit_error(&message, call.json),
    }
}

/// `bee.mjs:2189` `tagEventSummary`.
fn tag_event_summary(event: &Value) -> String {
    let scope_suffix = if js_truthy(event.get("scope")) {
        format!(" scope={}", js_template_value(event.get("scope")))
    } else {
        String::new()
    };
    let tags = event
        .get("tags")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(js_display_value).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    format!("Tagged {} with [{tags}]{scope_suffix}.", js_template_value(event.get("target")))
}

/// `bee.mjs:2194` `handleDecisionsTag`. `--stdin` takes a JSON ARRAY and is
/// all-or-nothing; the single-flag form is one entry through the same path.
fn handle_tag(call: &Call) -> ExitCode {
    let date = now_iso();
    if matches!(call.flag("stdin"), Some(Value::Bool(true))) {
        let mut raw = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut raw) {
            return emit_error(&format!("decisions tag --stdin: cannot read stdin: {e}"), call.json);
        }
        let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
            return emit_error("decisions tag --stdin: input is not valid JSON.", call.json);
        };
        let Some(entries) = parsed.as_array() else {
            return emit_error(
                "decisions tag --stdin: input must be a JSON array of {target, tags, scope?}.",
                call.json,
            );
        };
        let mut ids = Vec::with_capacity(entries.len());
        for _ in entries {
            match random_uuid() {
                Ok(id) => ids.push(id),
                Err(e) => return emit_error(&e, call.json),
            }
        }
        return match tag_decisions_batch(&call.root, entries, &ids, &date) {
            Ok(events) => {
                let text = events.iter().map(tag_event_summary).collect::<Vec<_>>().join("\n");
                emit(&Value::Array(events), &text, 0, call.json)
            }
            Err(message) => emit_error(&message, call.json),
        };
    }

    let target = match require_flag(call, "target") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let tags = match require_flag(call, "tags") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let mut entry = serde_json::Map::new();
    entry.insert("target".to_string(), Value::String(target));
    entry.insert(
        "tags".to_string(),
        Value::Array(split_list(&tags).into_iter().map(Value::String).collect()),
    );
    // `scope: flags.scope !== undefined ? String(flags.scope) : undefined` —
    // an ABSENT key, not a null, so the store layer's own
    // `entry.scope !== undefined` check sees what mjs sees.
    if let Some(scope) = present_flag(call, "scope") {
        entry.insert("scope".to_string(), Value::String(scope));
    }
    let id = match random_uuid() {
        Ok(id) => id,
        Err(e) => return emit_error(&e, call.json),
    };
    match tag_decision(&call.root, &Value::Object(entry), &id, &date) {
        Ok(event) => {
            let text = tag_event_summary(&event);
            emit(&event, &text, 0, call.json)
        }
        Err(message) => emit_error(&message, call.json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    /// The fallback names all EIGHT verbs even though this binary registers
    /// seven — it is mjs's line, not a status report on the port.
    #[test]
    fn usage_fallback_is_byte_exact() {
        assert_eq!(
            usage_fallback(&toks(&["decisions", "frobnicate"])),
            "Unknown command \"frobnicate\". Use: log, supersede, redact, active, search, archive, tag, render."
        );
        assert_eq!(
            usage_fallback(&toks(&["decisions"])),
            "Unknown command \"(missing)\". Use: log, supersede, redact, active, search, archive, tag, render."
        );
    }

    /// SEVEN of eight after rpl-5. `render` is deliberately absent — it was
    /// split out into rpl-10, and this assertion is what stops a later
    /// worker from believing the group is finished.
    #[test]
    fn the_group_registers_every_verb_except_render() {
        let g = group();
        assert_eq!(g.group, "decisions");
        assert_eq!(
            g.verbs.iter().map(|v| v.verb).collect::<Vec<_>>(),
            ["log", "supersede", "redact", "active", "search", "archive", "tag"]
        );
        assert!(
            !g.verbs.iter().any(|v| v.verb == "render"),
            "`render` belongs to rpl-10; registering it here would claim a port that does not exist"
        );
        assert!(g.usage_fallback.is_some(), "the group owns a legacy usage fallback in mjs");
    }

    /// `matchesWholeToken`'s two `[\w-]` lookarounds, spelled out on the
    /// exact trio a plain `\b` gets wrong: `-` is itself a non-word
    /// character, so `\b` would accept `si-1-extra`.
    #[test]
    fn whole_token_match_excludes_longer_and_hyphen_suffixed() {
        let hay = |s: &str| vec![s.to_string()];
        assert!(matches_whole_token(&hay("rpl5 token si-1 exact"), "si-1"));
        assert!(matches_whole_token(&hay("si-1"), "si-1"));
        assert!(matches_whole_token(&hay("(si-1)"), "si-1"));
        assert!(!matches_whole_token(&hay("rpl5 token si-10 longer"), "si-1"));
        assert!(!matches_whole_token(&hay("rpl5 token si-1-extra suffixed"), "si-1"));
        assert!(!matches_whole_token(&hay("xsi-1"), "si-1"));
        assert!(!matches_whole_token(&hay("si-1x"), "si-1"));
        assert!(!matches_whole_token(&hay("si_1"), "si-1"));
        // Case-insensitive via the shared ECMA `Canonicalize`, including
        // its Kelvin-sign carve-out.
        assert!(matches_whole_token(&hay("see SI-1 here"), "si-1"));
        assert!(!matches_whole_token(&hay("\u{212A}"), "k"));
    }

    /// `formatDecision`: `alternatives` renders only when TRUTHY, and every
    /// resurfaced field is datamarked.
    #[test]
    fn format_decision_matches_the_mjs_render() {
        let with_alt = serde_json::json!({
            "id": "d-1", "type": "decide", "date": "2026-07-26T00:00:00.000Z",
            "decision": "pick a", "rationale": "because", "alternatives": "pick b",
        });
        assert_eq!(
            format_decision(&with_alt),
            "[2026-07-26T00:00:00.000Z] «pick a» (id d-1, decide)\n  why: «because»\n  alternatives: «pick b»"
        );
        // A null (and an empty-string) `alternatives` is FALSY: the line is
        // dropped entirely, never rendered as `«»`.
        let mut no_alt = with_alt.clone();
        no_alt["alternatives"] = Value::Null;
        assert_eq!(
            format_decision(&no_alt),
            "[2026-07-26T00:00:00.000Z] «pick a» (id d-1, decide)\n  why: «because»"
        );
        no_alt["alternatives"] = Value::String(String::new());
        assert_eq!(
            format_decision(&no_alt),
            "[2026-07-26T00:00:00.000Z] «pick a» (id d-1, decide)\n  why: «because»"
        );
        // `datamark`'s `?? ''` fold: a missing field neutralizes to the
        // empty mark, while a `${…}` template slot renders `undefined`.
        let bare = serde_json::json!({ "type": "decide" });
        assert_eq!(format_decision(&bare), "[undefined] «» (id undefined, decide)\n  why: «»");
    }

    #[test]
    fn tag_summary_appends_scope_only_when_present() {
        let with_scope = serde_json::json!({ "target": "d-1", "tags": ["a", "b"], "scope": "billing" });
        assert_eq!(tag_event_summary(&with_scope), "Tagged d-1 with [a, b] scope=billing.");
        let without = serde_json::json!({ "target": "d-1", "tags": ["a"] });
        assert_eq!(tag_event_summary(&without), "Tagged d-1 with [a].");
    }

    /// `parseInt` semantics, spelled out: the two cases where a `Number()`
    /// based port would diverge while still passing schema validation.
    #[test]
    fn confidence_parses_like_parse_int_not_like_number() {
        assert_eq!(js_parse_int_10("80"), Some(80));
        assert_eq!(js_parse_int_10("  80  "), Some(80));
        assert_eq!(js_parse_int_10("12.7"), Some(12)); // Number() -> 12.7
        assert_eq!(js_parse_int_10("1e3"), Some(1)); // Number() -> 1000
        assert_eq!(js_parse_int_10("-5"), Some(-5));
        assert_eq!(js_parse_int_10("0x10"), Some(0)); // radix 10 stops at 'x'
        assert_eq!(js_parse_int_10("abc"), None);
        assert_eq!(js_parse_int_10(""), None);
    }

    #[test]
    fn split_list_drops_blanks_like_mjs() {
        assert_eq!(split_list("billing, recall ,,x"), vec!["billing", "recall", "x"]);
        assert!(split_list("").is_empty());
        assert!(split_list(" , ").is_empty());
    }
}
