//! The `intent` group — the FIRST group through the rpl-1 dispatch seam.
//!
//! Port of `packages/bee/bee.mjs:4059-4134` (the four handlers) plus
//! `:6503-6506` (`intentUsageFallback`, the group's own unknown-VERB line).
//! Presentation ONLY: every rule — verbatim request, immutability of
//! request/acceptance, key resolution, fail-open reads, the no-lock write —
//! lives in [`bee_core::intent`], exactly as the mjs handlers delegate every
//! rule to `lib/intent.mjs`. If a behavior needs to move, it moves there.
//!
//! Wall-clock is stamped HERE and threaded into the store functions rather
//! than read inside them, so the store layer stays deterministic and
//! unit-testable. The format is [`bee_core::lock::iso8601_millis`], which is
//! the exact 24-character `YYYY-MM-DDTHH:MM:SS.mmmZ` shape
//! `Date.prototype.toISOString()` emits — the same shape
//! `bee_parity::normalize::Shape::IsoMillisZ` shape-gates on, so a
//! wrong-precision stamp fails the diff instead of being masked into silence.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bee_core::intent::{
    self, precompact_block, resume_block, Anchor, IntentFields, Lookup,
};
use serde_json::{json, Value};

use crate::dispatch::{emit, emit_error, Call, GroupDef, VerbDef};
use crate::ledger::{optional_flag, require_flag};

/// `bee.mjs:6503` `intentUsageFallback`. Byte-exact, including the
/// `(missing)` placeholder for a bare `bee intent`.
fn usage_fallback(leading: &[String]) -> String {
    let verb = leading.get(1).map(String::as_str).unwrap_or("(missing)");
    let verb = if verb.is_empty() { "(missing)" } else { verb };
    format!("Unknown command \"{verb}\". Use: set, show, advance, clear.")
}

/// The registration this group contributes. ONE line in
/// [`crate::groups::register_all`] pulls it in.
pub fn group() -> GroupDef {
    GroupDef {
        group: "intent",
        usage_fallback: Some(usage_fallback),
        verbs: vec![
            VerbDef { verb: "set", handler: handle_set },
            VerbDef { verb: "show", handler: handle_show },
            VerbDef { verb: "advance", handler: handle_advance },
            VerbDef { verb: "clear", handler: handle_clear },
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

/// `bee.mjs:4063` `intentLookupOptions`. Note what it does NOT carry: `key`.
/// The CLI never sets it, so `intentKeyCandidates`'s explicit-key branch is
/// CLI-unreachable in both runtimes — ported anyway, never exercised here.
fn lookup(call: &Call) -> Lookup {
    Lookup {
        feature: optional_flag(call, "feature"),
        session_id: optional_flag(call, "session"),
        key: None,
    }
}

/// `bee.mjs:4070` `formatAnchor`.
///
/// `written_at` is interpolated with `${}`, so a record whose `written_at` is
/// absent renders the literal `null` — not an empty string, and not a
/// silently omitted parenthetical.
fn format_anchor(anchor: &Anchor) -> String {
    let written = anchor.written_at.as_deref().unwrap_or("null");
    let mut lines = vec![
        format!("Intent anchor \"{}\" (written {written})", anchor.key),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        anchor.request.clone(),
        format!("DONE MEANS: {}", anchor.acceptance),
    ];
    if let Some(next) = &anchor.next_action {
        lines.push(format!("NEXT ACTION: {next}"));
    }
    if !anchor.do_not_reverse.is_empty() {
        lines.push(format!("DO NOT REVERSE: {}", anchor.do_not_reverse.join(" | ")));
    }
    if !anchor.stop_conditions.is_empty() {
        lines.push(format!("STOP IF: {}", anchor.stop_conditions.join(" | ")));
    }
    lines.join("\n")
}

fn anchor_value(anchor: Option<&Anchor>) -> Value {
    anchor.map(|a| serde_json::to_value(a).unwrap_or(Value::Null)).unwrap_or(Value::Null)
}

/// `bee.mjs:4083` `handleIntentSet`.
fn handle_set(call: &Call) -> ExitCode {
    // requireFlag runs BEFORE writeIntent, so its message is the one a
    // caller sees for an empty --request even though writeIntent has a
    // refusal of its own for the same condition.
    let request = match require_flag(call, "request") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let acceptance = match require_flag(call, "acceptance") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let fields = IntentFields {
        request: Some(request),
        acceptance: Some(acceptance),
        next_action: optional_flag(call, "next-action"),
        feature: optional_flag(call, "feature"),
        lane: optional_flag(call, "lane"),
        cell: optional_flag(call, "cell"),
        do_not_reverse: optional_flag(call, "do-not-reverse"),
        stop_conditions: optional_flag(call, "stop-conditions"),
    };
    // `String(flags.force) === 'true'` — an absent flag stringifies to
    // "undefined", so anything but the literal string "true" is false.
    let force = call.flag_str("force") == Some("true");

    match intent::write_intent(&call.root, &fields, &lookup(call), force, &now_iso()) {
        Ok(anchor) => emit(
            &anchor_value(Some(&anchor)),
            &format!(
                "Intent anchored at \"{}\". It is re-asserted at PreCompact and read first on a compact/resume start.",
                anchor.key
            ),
            0,
            call.json,
        ),
        Err(message) => emit_error(&message, call.json),
    }
}

/// `bee.mjs:4108` `handleIntentShow`.
fn handle_show(call: &Call) -> ExitCode {
    let anchor = intent::read_intent(&call.root, &lookup(call));
    let render = optional_flag(call, "render");
    if let Some(render) = render.as_deref() {
        if render == "precompact" || render == "resume" {
            let block = if render == "precompact" {
                precompact_block(anchor.as_ref())
            } else {
                resume_block(anchor.as_ref())
            };
            // Key order: anchor, render, block — the mjs object literal's.
            let result = json!({
                "anchor": anchor_value(anchor.as_ref()),
                "render": render,
                "block": block,
            });
            let text = if block.is_empty() { "(no intent anchor)" } else { block.as_str() };
            return emit(&result, text, 0, call.json);
        }
    }
    let text = anchor.as_ref().map(format_anchor).unwrap_or_else(|| "(no intent anchor)".to_string());
    emit(&anchor_value(anchor.as_ref()), &text, 0, call.json)
}

/// `bee.mjs:4120` `handleIntentAdvance`.
fn handle_advance(call: &Call) -> ExitCode {
    let next_action = match require_flag(call, "next-action") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    match intent::advance_intent(&call.root, Some(&next_action), &lookup(call), &now_iso()) {
        None => emit_error(
            "intent advance: no intent anchor exists to advance — run `bee intent set` first.",
            call.json,
        ),
        Some(anchor) => {
            // `${anchor.next_action}` — a whitespace-only --next-action
            // normalizes to null and renders as the literal "null".
            let next = anchor.next_action.as_deref().unwrap_or("null");
            emit(
                &anchor_value(Some(&anchor)),
                &format!(
                    "Advanced intent anchor \"{}\" — next action: {next}. Request and acceptance are unchanged.",
                    anchor.key
                ),
                0,
                call.json,
            )
        }
    }
}

/// `bee.mjs:4131` `handleIntentClear`.
fn handle_clear(call: &Call) -> ExitCode {
    let record = intent::clear_intent(&call.root, &lookup(call));
    let text = if record.cleared {
        format!("Cleared intent anchor \"{}\".", record.key)
    } else {
        format!("No intent anchor at \"{}\" to clear.", record.key)
    };
    emit(&serde_json::to_value(&record).unwrap_or(Value::Null), &text, 0, call.json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn usage_fallback_is_byte_exact() {
        assert_eq!(
            usage_fallback(&toks(&["intent", "bogusverb"])),
            "Unknown command \"bogusverb\". Use: set, show, advance, clear."
        );
        assert_eq!(
            usage_fallback(&toks(&["intent"])),
            "Unknown command \"(missing)\". Use: set, show, advance, clear."
        );
    }

    #[test]
    fn the_group_registers_exactly_the_four_verbs() {
        let g = group();
        assert_eq!(g.group, "intent");
        assert_eq!(g.verbs.iter().map(|v| v.verb).collect::<Vec<_>>(), ["set", "show", "advance", "clear"]);
        assert!(g.usage_fallback.is_some(), "the group owns a legacy usage fallback in mjs");
    }

    #[test]
    fn format_anchor_renders_an_absent_written_at_as_the_literal_null() {
        let anchor = bee_core::intent::normalize_anchor(
            &serde_json::json!({ "request": "r", "acceptance": "a" }),
            "k",
        )
        .unwrap();
        let text = format_anchor(&anchor);
        assert!(text.starts_with("Intent anchor \"k\" (written null)"), "{text}");
        assert!(!text.contains("NEXT ACTION"));
        assert!(!text.contains("DO NOT REVERSE"));
    }

    #[test]
    fn now_iso_has_the_to_iso_string_shape() {
        let s = now_iso();
        assert_eq!(s.len(), 24, "{s}");
        assert!(s.ends_with('Z') && s.as_bytes()[19] == b'.', "{s}");
    }
}
