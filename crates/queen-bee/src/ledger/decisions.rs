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
//! # Only three verbs, deliberately
//!
//! `active`, `search`, `archive`, `tag` and `render` are rpl-5's. They
//! resolve in the registry, find no handler here, and take
//! `dispatch.rs`'s honest "known bee command but is not ported into this
//! binary yet" refusal rather than a guess. The group's `usage_fallback`
//! still names ALL EIGHT verbs, because that is what mjs prints — the
//! fallback describes the bee CLI, not this binary's progress.
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
use bee_core::datamark::js_trim;
use bee_core::decisions::{
    log_decision, redact_decision, supersede_decision, taxonomy_file_exists, LogFields,
    SupersedeFields,
};
use serde_json::Value;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    /// The fallback names all EIGHT verbs even though this binary registers
    /// three — it is mjs's line, not a status report on the port.
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

    #[test]
    fn the_group_registers_exactly_the_three_write_verbs() {
        let g = group();
        assert_eq!(g.group, "decisions");
        assert_eq!(
            g.verbs.iter().map(|v| v.verb).collect::<Vec<_>>(),
            ["log", "supersede", "redact"]
        );
        assert!(g.usage_fallback.is_some(), "the group owns a legacy usage fallback in mjs");
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
