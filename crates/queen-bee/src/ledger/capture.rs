//! The `capture` group — the capture queue's CLI surface.
//!
//! Port of `packages/bee/bee.mjs:4021-4065` (`formatCaptureStub` plus the four
//! handlers) and `:6550-6553` (`captureUsageFallback`, the group's own
//! unknown-VERB line). Presentation ONLY: every rule — the required outcome,
//! the high-risk refusal, `normalizeList`'s split/trim/filter, the
//! stubs-minus-flushed fold, the no-lock append, and the content-safety
//! refusal texts — lives in [`bee_core::capture`] and [`bee_core::datamark`],
//! exactly as the mjs handlers delegate everything to `lib/capture.mjs`.
//!
//! Wall-clock and entropy are stamped HERE and threaded into the store
//! functions, the same discipline [`crate::ledger::intent`] uses: the store
//! layer stays deterministic and unit-testable, and `.bee/capture-queue.jsonl`
//! gets its `at` in [`bee_core::lock::iso8601_millis`]'s exact
//! `Date.prototype.toISOString()` shape — which is also the shape
//! `bee_parity::normalize::Shape::IsoMillisZ` gates the `at` mask on, so a
//! wrong-precision stamp fails the parity diff instead of being masked into
//! silence.
//!
//! # Why `add` is the group's only unmasked-text hazard
//!
//! `capture add`'s human text interpolates the FRESH uuid
//! (`Queued capture stub ${stub.id}.`) as bare prose, not as a JSON key, and
//! `bee_parity::normalize` masks by key name only — deliberately, it has no
//! pattern scrubber. So the parity scenarios drive `add` through `--json`,
//! where `id`/`at` are real keys and mask on both legs. `list`, `count` and
//! `flush` have no such hazard: every id and timestamp they print comes from
//! the fixture, identical on both legs.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bee_core::capture::{
    add_capture_stub, capture_queue, flush_capture_stub, pending_capture_stubs, random_uuid,
    StubFields,
};
use serde_json::{json, Value};

use crate::dispatch::{emit, emit_error, Call, GroupDef, VerbDef};
use crate::ledger::{optional_flag, require_flag};

/// `bee.mjs:6550` `captureUsageFallback`. Byte-exact, including the
/// `(missing)` placeholder for a bare `bee capture`.
fn usage_fallback(leading: &[String]) -> String {
    let verb = leading.get(1).map(String::as_str).unwrap_or("(missing)");
    let verb = if verb.is_empty() { "(missing)" } else { verb };
    format!("Unknown command \"{verb}\". Use: add, list, flush, count.")
}

/// The registration this group contributes. ONE line in
/// [`crate::groups::register_all`] pulls it in.
pub fn group() -> GroupDef {
    GroupDef {
        group: "capture",
        usage_fallback: Some(usage_fallback),
        verbs: vec![
            VerbDef { verb: "add", handler: handle_add },
            VerbDef { verb: "list", handler: handle_list },
            VerbDef { verb: "flush", handler: handle_flush },
            VerbDef { verb: "count", handler: handle_count },
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

// ─── the JS coercions `formatCaptureStub` performs implicitly ──────────────
//
// Every field it prints goes through `${…}` (i.e. `String(x)`) or a
// truthiness test, on values that came off DISK. `addCaptureStub` only ever
// writes strings, arrays and nulls, but the reader is fail-open by design and
// a queue row is not schema-checked, so the coercions are ported rather than
// assumed away.

/// `String(x)` for the shapes a JSONL row can hold. Arrays stringify by
/// joining with `,` and rendering null/undefined members as EMPTY — which is
/// why [`join_js`] exists separately from this and does not call it per member
/// through the `String(null) === "null"` path.
fn js_string_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                other => js_string_value(other),
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// `${obj.key}` where the key may be ABSENT — which stringifies to
/// `"undefined"`, not to `"null"` and not to the empty string.
fn js_string(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(value) => js_string_value(value),
    }
}

/// JS truthiness for `if (stub.area)` / `if (stub.source)`.
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Some(Value::Array(_)) | Some(Value::Object(_)) => true,
    }
}

/// `arr && arr.length` — the guard both list fields use. Only an ARRAY is
/// handled: `addCaptureStub` writes `normalizeList`'s array unconditionally,
/// so any other shape means a hand-mangled row, and on such a row mjs's
/// `.join` would THROW rather than print. Rather than invent a Rust rendering
/// mjs would never produce, that shape is left to the shared reader contract
/// (corrupt rows are skipped at parse time by `readJsonl`) and is deliberately
/// not exercised by a parity scenario.
fn non_empty_array(v: Option<&Value>) -> Option<&Vec<Value>> {
    match v {
        Some(Value::Array(items)) if !items.is_empty() => Some(items),
        _ => None,
    }
}

/// `Array.prototype.join(', ')`: null and undefined members render EMPTY.
fn join_js(items: &[Value]) -> String {
    items
        .iter()
        .map(|item| match item {
            Value::Null => String::new(),
            other => js_string_value(other),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `bee.mjs:4021` `formatCaptureStub`.
///
/// The `[mined]` marker is a STRICT equality against `'mined'`
/// (`bee.mjs:4022`), so a stub carrying any other provenance gets the
/// `source:` line without the marker — which is exactly the pair of shapes the
/// fixture seeds.
fn format_capture_stub(stub: &Value) -> String {
    let marker = if stub.get("source") == Some(&Value::String("mined".to_string())) {
        " [mined]"
    } else {
        ""
    };
    let at = js_string(stub.get("at"));
    let outcome = js_string(stub.get("outcome"));
    let id = js_string(stub.get("id"));
    let mut parts = vec![format!("[{at}] {outcome}{marker} (id {id})")];
    if let Some(items) = non_empty_array(stub.get("dids")) {
        parts.push(format!("  decisions: {}", join_js(items)));
    }
    if truthy(stub.get("area")) {
        parts.push(format!("  area: {}", js_string(stub.get("area"))));
    }
    if let Some(items) = non_empty_array(stub.get("files")) {
        parts.push(format!("  files: {}", join_js(items)));
    }
    if truthy(stub.get("source")) {
        parts.push(format!("  source: {}", js_string(stub.get("source"))));
    }
    parts.join("\n")
}

/// `bee.mjs:4031` `handleCaptureAdd`.
///
/// `requireFlag(flags, 'outcome')` is evaluated as the FIRST property of the
/// object literal handed to `addCaptureStub`, so a missing `--outcome` yields
/// the generic missing-flag refusal, never `addCaptureStub`'s own
/// "outcome text is required." — a caller can reach the latter only by
/// passing whitespace.
fn handle_add(call: &Call) -> ExitCode {
    let outcome = match require_flag(call, "outcome") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let dids = optional_flag(call, "did");
    let area = optional_flag(call, "area");
    let files = optional_flag(call, "files");
    let lane = optional_flag(call, "lane");
    let source = optional_flag(call, "source");
    let fields = StubFields {
        outcome: Some(outcome.as_str()),
        dids: dids.as_deref(),
        area: area.as_deref(),
        files: files.as_deref(),
        lane: lane.as_deref(),
        source: source.as_deref(),
    };
    let id = match random_uuid() {
        Ok(id) => id,
        Err(e) => return emit_error(&e, call.json),
    };
    match add_capture_stub(&call.root, &fields, &id, &now_iso()) {
        Ok(stub) => {
            let id = js_string(stub.get("id"));
            emit(
                &stub,
                &format!(
                    "Queued capture stub {id}. Flush via bee-scribing at wrap-up, before compact/clear, or next session (decision 0017)."
                ),
                0,
                call.json,
            )
        }
        Err(message) => emit_error(&message, call.json),
    }
}

/// `bee.mjs:4046` `handleCaptureList`.
fn handle_list(call: &Call) -> ExitCode {
    let stubs = pending_capture_stubs(&call.root);
    let text = if stubs.is_empty() {
        "Capture queue is empty.".to_string()
    } else {
        stubs.iter().map(format_capture_stub).collect::<Vec<_>>().join("\n")
    };
    // Key order is the mjs object literal's: count, then stubs.
    let result = json!({ "count": stubs.len(), "stubs": stubs });
    emit(&result, &text, 0, call.json)
}

/// `bee.mjs:4052` `handleCaptureFlush`.
fn handle_flush(call: &Call) -> ExitCode {
    let id = match require_flag(call, "id") {
        Ok(v) => v,
        Err(msg) => return emit_error(&msg, call.json),
    };
    let into = optional_flag(call, "into");
    match flush_capture_stub(&call.root, Some(id.as_str()), into.as_deref(), &now_iso()) {
        Ok(record) => {
            // `${record.into ? ` into ${record.into}` : ''}` — the tail is
            // omitted entirely for a null `into`, never rendered as
            // " into null".
            let tail = if truthy(record.get("into")) {
                format!(" into {}", js_string(record.get("into")))
            } else {
                String::new()
            };
            let id = js_string(record.get("id"));
            emit(&record, &format!("Flushed stub {id}{tail}."), 0, call.json)
        }
        Err(message) => emit_error(&message, call.json),
    }
}

/// `bee.mjs:4062` `handleCaptureCount`.
///
/// The result carries ONLY `count` — `captureQueue`'s `stubs` array is read
/// and then dropped, so `--json` here is not a smaller spelling of `list`.
fn handle_count(call: &Call) -> ExitCode {
    let queue = capture_queue(&call.root);
    let count = queue.get("count").cloned().unwrap_or(Value::Null);
    let text = format!("{} pending capture stub(s).", js_string(Some(&count)));
    emit(&json!({ "count": count }), &text, 0, call.json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn toks(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn usage_fallback_is_byte_exact() {
        assert_eq!(
            usage_fallback(&toks(&["capture", "frobnicate"])),
            "Unknown command \"frobnicate\". Use: add, list, flush, count."
        );
        assert_eq!(
            usage_fallback(&toks(&["capture"])),
            "Unknown command \"(missing)\". Use: add, list, flush, count."
        );
    }

    #[test]
    fn the_group_registers_exactly_the_four_verbs() {
        let g = group();
        assert_eq!(g.group, "capture");
        assert_eq!(
            g.verbs.iter().map(|v| v.verb).collect::<Vec<_>>(),
            ["add", "list", "flush", "count"]
        );
        assert!(g.usage_fallback.is_some(), "the group owns a legacy usage fallback in mjs");
    }

    #[test]
    fn format_stub_renders_every_optional_branch() {
        let full = json!({
            "kind": "stub",
            "id": "11111111-1111-4111-8111-111111111111",
            "at": "2026-07-26T00:00:01.000Z",
            "outcome": "chốt một",
            "dids": ["0017", "0023"],
            "area": "docs/specs/capture.md",
            "files": ["a.mjs", "b.rs"],
            "lane": "tiny",
        });
        assert_eq!(
            format_capture_stub(&full),
            "[2026-07-26T00:00:01.000Z] chốt một (id 11111111-1111-4111-8111-111111111111)\n  decisions: 0017, 0023\n  area: docs/specs/capture.md\n  files: a.mjs, b.rs"
        );

        let bare = json!({
            "kind": "stub",
            "id": "22222222-2222-4222-8222-222222222222",
            "at": "2026-07-26T00:00:02.000Z",
            "outcome": "chốt hai",
            "dids": [],
            "area": Value::Null,
            "files": [],
            "lane": Value::Null,
        });
        assert_eq!(
            format_capture_stub(&bare),
            "[2026-07-26T00:00:02.000Z] chốt hai (id 22222222-2222-4222-8222-222222222222)"
        );
    }

    /// The marker is `=== 'mined'`, and it is INDEPENDENT of the `source:`
    /// line — a mined stub prints both, any other provenance prints only the
    /// line.
    #[test]
    fn the_mined_marker_is_a_strict_equality_and_not_the_source_line() {
        let mined = json!({ "id": "i", "at": "t", "outcome": "o", "source": "mined" });
        assert_eq!(format_capture_stub(&mined), "[t] o [mined] (id i)\n  source: mined");

        let other = json!({ "id": "i", "at": "t", "outcome": "o", "source": "transcript-recovery" });
        assert_eq!(format_capture_stub(&other), "[t] o (id i)\n  source: transcript-recovery");
    }

    /// An ABSENT key stringifies to `"undefined"`, a null one to `"null"` —
    /// two different strings, and neither is the empty string a naive port
    /// would emit.
    #[test]
    fn absent_and_null_stringify_differently() {
        assert_eq!(js_string(None), "undefined");
        assert_eq!(js_string(Some(&Value::Null)), "null");
        // But INSIDE join, a null member is empty. `String([null])` is "".
        assert_eq!(join_js(&[Value::Null, Value::String("b".into())]), ", b");
    }
}
