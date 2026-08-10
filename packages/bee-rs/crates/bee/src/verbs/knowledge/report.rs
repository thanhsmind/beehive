// bee knowledge report — read-only recurrence measurement over `bee.critical`
// patterns that carry a `bee.signature`.
//
// Feature: docs/history/knowledge-usable/CONTEXT.md (U8).
//
// Signature (U8, Agent's Discretion): a critical `bee.pattern` concept may
// carry a `bee.signature` string in its frontmatter — a deterministic,
// literally grep-able token an author chooses when the pattern is written
// (e.g. `stale-installed-binary-not-reinstalled-after-merge`). It is a plain
// scalar under the pattern's `bee:` block, unknown to `BEE_KEY_ORDER`, so
// `emit_frontmatter` places it after the known keys — round-trip-canonical
// as long as it is physically the last `bee:` line in the source file
// (`frontmatter.rs`'s `emit_entries`: known keys in `BEE_KEY_ORDER`, then
// unknown keys alphabetically; one unknown key needs no internal sort).
//
// Corpus: every `bee.pattern` concept of the active bundle with
// `bee.critical: true` (`is_critical_pattern` below) — the same pool
// `context.rs`'s critical ranking and `index.rs`'s "Critical patterns"
// section already read. A critical pattern with no signature is NEVER
// guessed at; it renders in `unmeasured` and nowhere else.
//
// Matching (no fuzzy matching, per U8's prohibitions): a signature counts a
// decision-log entry (`.bee/decisions.jsonl`, fields `decision`/`rationale`/
// `alternatives`) or a capture-queue stub (`.bee/capture-queue.jsonl`, kind
// `stub`, fields `outcome`/`area`/`files`) only when the signature string
// appears as a LITERAL substring of that entry's joined text — never
// lowercased, never term-split, never OR-matched the way `knowledge search`
// or `decisions search` score a query. `flush` rows in the capture queue are
// never stubs and are skipped outright.
//
// Date filter: an entry counts only when its own date is strictly AFTER the
// pattern's `timestamp` — compared as calendar days (`date_key`: the first
// 10 bytes of an ISO date/datetime, which is already `YYYY-MM-DD` and sorts
// lexically). A same-day entry does not count as a recurrence; the pattern's
// own authoring day is excluded on purpose so citing the incident that
// PRODUCED the pattern is never counted as its own recurrence.
//
// Read-only (U8's prohibition): no `std::fs::write`/`create_dir_all`/state
// call anywhere in this file. `read_jsonl` (verbs::feedback) is fail-open —
// a missing or corrupt `.bee/*.jsonl` file reads as zero rows, never a
// delegate — so every critical pattern with a signature always gets a
// number, never a gap in the report itself.

#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::feedback::read_jsonl;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── corpus (bee.pattern + bee.critical: true only) ────────────────────────

fn is_critical_pattern(concept: &Concept) -> bool {
    concept.data.get("type").and_then(Value::as_str) == Some("bee.pattern")
        && matches!(bee_of(&concept.data).get("critical"), Some(Value::Bool(true)))
}

// ─── matching (literal substring, no fuzzy matching) ───────────────────────

/// The first 10 bytes of an ISO `YYYY-MM-DD` or `YYYY-MM-DDTHH:mm:ss.sssZ`
/// string — the calendar-day key every date comparison below uses. Shorter
/// or malformed input passes through unchanged (an empty/garbage pattern
/// date excludes every entry from ever being "after" it, which is the safe
/// direction: never a guessed recurrence).
fn date_key(raw: &str) -> &str {
    if raw.len() >= 10 {
        &raw[..10]
    } else {
        raw
    }
}

/// A decision-log event's matchable text (D3-style join, literal — no
/// lowercasing).
fn decision_text(entry: &Value) -> String {
    let mut parts = Vec::new();
    for key in ["decision", "rationale", "alternatives"] {
        if let Some(Value::String(s)) = entry.get(key) {
            parts.push(s.clone());
        }
    }
    parts.join("\n")
}

/// A capture-queue stub's matchable text.
fn stub_text(stub: &Value) -> String {
    let mut parts = Vec::new();
    if let Some(Value::String(s)) = stub.get("outcome") {
        parts.push(s.clone());
    }
    if let Some(Value::String(s)) = stub.get("area") {
        parts.push(s.clone());
    }
    if let Some(Value::Array(files)) = stub.get("files") {
        for f in files {
            if let Value::String(s) = f {
                parts.push(s.clone());
            }
        }
    }
    parts.join("\n")
}

/// Recurrence count + last-seen calendar day for one pattern's signature,
/// over BOTH sources. Never delegates: `read_jsonl` already fails open on a
/// missing/corrupt file (empty rows), which is exactly "zero recurrences
/// measured" for a report, not a gap in the report itself.
pub(crate) fn measure_recurrence(root: &Path, signature: &str, pattern_date: &str) -> (usize, Option<String>) {
    let pattern_key = date_key(pattern_date);
    let mut count = 0usize;
    let mut last_seen: Option<String> = None;
    let mut note = |date: &str| {
        let key = date_key(date).to_string();
        if last_seen.as_deref().map(|s| key.as_str() > s).unwrap_or(true) {
            last_seen = Some(key);
        }
    };

    let decisions = read_jsonl(&root.join(".bee").join("decisions.jsonl")).rows;
    for entry in &decisions {
        if !entry.is_object() {
            continue;
        }
        let Some(Value::String(date)) = entry.get("date") else { continue };
        if date_key(date) <= pattern_key {
            continue;
        }
        if !decision_text(entry).contains(signature) {
            continue;
        }
        count += 1;
        note(date);
    }

    let queue = read_jsonl(&root.join(".bee").join("capture-queue.jsonl")).rows;
    for entry in &queue {
        if entry.get("kind").and_then(Value::as_str) != Some("stub") {
            continue;
        }
        let Some(Value::String(date)) = entry.get("at") else { continue };
        if date_key(date) <= pattern_key {
            continue;
        }
        if !stub_text(entry).contains(signature) {
            continue;
        }
        count += 1;
        note(date);
    }

    (count, last_seen)
}

// ─── report assembly (pure — root + concepts in, rows + lines out) ────────

/// Builds the measured/unmeasured rows and the text lines over every
/// critical pattern in CONCEPTS, reading recurrence sources from ROOT. Pulled
/// out of `run_report` so it is directly testable without a resolved store
/// root / cwd dance — the same shape `search_bundle` gives `run_search`.
pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Vec<String>) {
    let mut criticals: Vec<&Concept> = concepts.iter().filter(|c| is_critical_pattern(c)).collect();
    // Deterministic order (no fuzzy anything here either): path ascending.
    criticals.sort_by(|a, b| a.path.cmp(&b.path));

    let mut measured: Vec<Value> = Vec::new();
    let mut unmeasured: Vec<Value> = Vec::new();
    let mut lines: Vec<String> = Vec::new();

    for concept in &criticals {
        let bee = bee_of(&concept.data);
        let title = str_field(&concept.data, "title").unwrap_or(&concept.path).to_string();
        let path = format!("docs/knowledge/{}", concept.path);

        let signature = match bee.get("signature") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => {
                lines.push(format!("UNMEASURED {path} — {title} (no bee.signature)"));
                let mut row = Map::new();
                row.insert("path".into(), Value::String(path));
                row.insert("title".into(), Value::String(title));
                unmeasured.push(Value::Object(row));
                continue;
            }
        };
        let pattern_date = match concept.data.get("timestamp") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let (count, last_seen) = measure_recurrence(root, &signature, &pattern_date);
        lines.push(format!(
            "{path} — {title} — signature {signature:?} — {count} recurrence(s){}",
            match &last_seen {
                Some(d) => format!(", last seen {d}"),
                None => String::new(),
            }
        ));
        let mut row = Map::new();
        row.insert("path".into(), Value::String(path));
        row.insert("title".into(), Value::String(title));
        row.insert("signature".into(), Value::String(signature));
        row.insert("count".into(), Value::from(count));
        row.insert("last_seen".into(), last_seen.map(Value::String).unwrap_or(Value::Null));
        measured.push(Value::Object(row));
    }

    lines.push(format!(
        "knowledge report: {} measured critical pattern(s), {} unmeasured (no bee.signature).",
        measured.len(),
        unmeasured.len()
    ));
    (measured, unmeasured, lines)
}

// ─── routing ────────────────────────────────────────────────────────────

pub(crate) fn run_report(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match g_prelude("knowledge report", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let concepts = collect_concepts(&dir)?;

    let (measured, unmeasured, lines) = build_report(&ctx.root, &concepts);

    let mut result = Map::new();
    let measured_count = measured.len();
    let unmeasured_count = unmeasured.len();
    result.insert("measured".into(), Value::Array(measured));
    result.insert("unmeasured".into(), Value::Array(unmeasured));
    result.insert("measured_count".into(), Value::from(measured_count));
    result.insert("unmeasured_count".into(), Value::from(unmeasured_count));
    Some(ctx.emit(&Value::Object(result), &lines.join("\n"), 0))
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        (tmp, dir)
    }

    fn write(dir: &Path, rel: &str, text: &str) {
        let abs = join_rel(dir, rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(abs, text).unwrap();
    }

    fn write_root(root: &Path, rel: &str, lines: &str) {
        let abs = root.join(".bee").join(rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(abs, lines).unwrap();
    }

    fn critical_pattern(title: &str, timestamp: &str, signature: Option<&str>) -> String {
        let sig_line = signature.map(|s| format!("\n  signature: {s}")).unwrap_or_default();
        format!(
            "---\ntype: bee.pattern\ntitle: {title}\ndescription: {title}\ntimestamp: {timestamp}\nbee:\n  id: {}\n  lifecycle: active\n  polarity: pitfall\n  critical: true{sig_line}\n---\nbody\n",
            title.to_lowercase().replace(' ', "-"),
        )
    }

    fn uncritical_pattern(title: &str, timestamp: &str) -> String {
        format!(
            "---\ntype: bee.pattern\ntitle: {title}\ndescription: {title}\ntimestamp: {timestamp}\nbee:\n  id: {}\n  lifecycle: active\n  polarity: practice\n  critical: false\n---\nbody\n",
            title.to_lowercase().replace(' ', "-"),
        )
    }

    fn decision_line(date: &str, decision: &str) -> String {
        format!(
            "{{\"id\":\"{date}\",\"type\":\"decide\",\"date\":\"{date}\",\"decision\":\"{decision}\",\"rationale\":\"r\",\"alternatives\":null,\"scope\":\"repo\",\"source\":\"user\",\"confidence\":null}}\n"
        )
    }

    fn stub_line(id: &str, at: &str, outcome: &str) -> String {
        format!(
            "{{\"kind\":\"stub\",\"id\":\"{id}\",\"at\":\"{at}\",\"outcome\":\"{outcome}\",\"dids\":[],\"area\":null,\"files\":[],\"lane\":null}}\n"
        )
    }

    #[test]
    fn signature_bearing_critical_counts_and_dates_after_the_pattern() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write(
            &dir,
            "patterns/sig.md",
            &critical_pattern("Signature thing", "2026-08-05", Some("stale-installed-binary")),
        );
        write_root(
            &root,
            "decisions.jsonl",
            &(decision_line("2026-08-06T00:00:00.000Z", "stale-installed-binary recurred again")
                + &decision_line("2026-08-04T00:00:00.000Z", "stale-installed-binary before the pattern")
                + &decision_line("2026-08-07T00:00:00.000Z", "unrelated decision text")),
        );
        write_root(
            &root,
            "capture-queue.jsonl",
            &stub_line("s1", "2026-08-09T00:00:00.000Z", "stale-installed-binary hit again"),
        );

        let concepts = collect_concepts(&dir).unwrap();
        let concept = concepts.iter().find(|c| c.path == "patterns/sig.md").unwrap();
        let bee = bee_of(&concept.data);
        let signature = bee.get("signature").and_then(Value::as_str).unwrap();
        let pattern_date = concept.data.get("timestamp").and_then(Value::as_str).unwrap();
        let (count, last_seen) = measure_recurrence(&root, signature, pattern_date);
        // The 2026-08-04 decision predates the pattern (excluded); the
        // 2026-08-07 decision postdates it but does not contain the
        // signature (excluded); the 2026-08-06 decision and the 2026-08-09
        // stub both postdate it AND contain the signature (counted).
        assert_eq!(count, 2, "only the after-date, signature-matching rows count");
        assert_eq!(last_seen.as_deref(), Some("2026-08-09"));
    }

    #[test]
    fn same_day_entry_is_not_a_recurrence() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write_root(
            &root,
            "decisions.jsonl",
            &decision_line("2026-08-05T23:00:00.000Z", "same-day-signature hit"),
        );
        let (count, last_seen) = measure_recurrence(&root, "same-day-signature", "2026-08-05");
        assert_eq!(count, 0, "the pattern's own authoring day never counts as its own recurrence");
        assert_eq!(last_seen, None);
    }

    #[test]
    fn matching_is_literal_no_fuzzy_term_splitting() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write_root(
            &root,
            "decisions.jsonl",
            &decision_line("2026-08-06T00:00:00.000Z", "the fan and the out were separate words here"),
        );
        // Both terms present separately, but not as the literal joined
        // signature — a fuzzy/OR matcher would count this; a literal
        // substring matcher must not.
        let (count, _) = measure_recurrence(&root, "fan-out", "2026-08-05");
        assert_eq!(count, 0, "no fuzzy matching — the signature must appear as one literal substring");
    }

    #[test]
    fn flush_events_are_never_counted_as_stubs() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write_root(
            &root,
            "capture-queue.jsonl",
            "{\"kind\":\"flush\",\"id\":\"x\",\"at\":\"2026-08-06T00:00:00.000Z\",\"into\":\"sig-marker\"}\n",
        );
        let (count, _) = measure_recurrence(&root, "sig-marker", "2026-08-05");
        assert_eq!(count, 0, "a flush row is never a stub, whatever its `into` text says");
    }

    #[test]
    fn signatureless_critical_is_unmeasured_never_guessed() {
        let (_tmp, dir) = bundle();
        write(&dir, "patterns/no-sig.md", &critical_pattern("No signature yet", "2026-08-05", None));
        let concepts = collect_concepts(&dir).unwrap();
        let concept = concepts.iter().find(|c| c.path == "patterns/no-sig.md").unwrap();
        assert!(is_critical_pattern(concept));
        assert!(bee_of(&concept.data).get("signature").is_none());
    }

    #[test]
    fn uncritical_pattern_is_never_a_candidate() {
        let (_tmp, dir) = bundle();
        write(&dir, "patterns/mundane.md", &uncritical_pattern("Mundane", "2026-08-05"));
        let concepts = collect_concepts(&dir).unwrap();
        let concept = concepts.iter().find(|c| c.path == "patterns/mundane.md").unwrap();
        assert!(!is_critical_pattern(concept), "critical: false must never enter the report's corpus");
    }

    #[test]
    fn build_report_renders_measured_and_unmeasured_rows_and_skips_uncritical() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write(
            &dir,
            "patterns/sig.md",
            &critical_pattern("Signature thing", "2026-08-05", Some("recur-me")),
        );
        write(&dir, "patterns/no-sig.md", &critical_pattern("Bare thing", "2026-08-06", None));
        write(&dir, "patterns/mundane.md", &uncritical_pattern("Mundane", "2026-08-05"));
        write_root(&root, "decisions.jsonl", &decision_line("2026-08-06T00:00:00.000Z", "recur-me landed again"));

        let concepts = collect_concepts(&dir).unwrap();
        let (measured, unmeasured, lines) = build_report(&root, &concepts);

        assert_eq!(measured.len(), 1, "only the signature-bearing critical is measured");
        let row = &measured[0];
        assert_eq!(row["path"], "docs/knowledge/patterns/sig.md");
        assert_eq!(row["signature"], "recur-me");
        assert_eq!(row["count"], 1);
        assert_eq!(row["last_seen"], "2026-08-06");

        assert_eq!(unmeasured.len(), 1, "the signatureless critical is unmeasured, never guessed");
        assert_eq!(unmeasured[0]["path"], "docs/knowledge/patterns/no-sig.md");

        assert!(lines.iter().any(|l| l.contains("recur-me") && l.contains("1 recurrence(s)")));
        assert!(lines.iter().any(|l| l.starts_with("UNMEASURED docs/knowledge/patterns/no-sig.md")));
        assert!(
            lines.iter().all(|l| !l.contains("Mundane")),
            "an uncritical pattern must never appear in either bucket"
        );
        assert!(lines.last().unwrap().contains("1 measured critical pattern(s), 1 unmeasured"));
    }

    #[test]
    fn report_never_writes_the_bundle_or_the_logs_it_reads() {
        let (_tmp, dir) = bundle();
        let root = dir.parent().unwrap().parent().unwrap().to_path_buf();
        write(
            &dir,
            "patterns/sig.md",
            &critical_pattern("Signature thing", "2026-08-05", Some("recur-me")),
        );
        write_root(&root, "decisions.jsonl", &decision_line("2026-08-06T00:00:00.000Z", "recur-me landed again"));
        let before_bundle = std::fs::read_to_string(join_rel(&dir, "patterns/sig.md")).unwrap();
        let before_decisions = std::fs::read_to_string(root.join(".bee").join("decisions.jsonl")).unwrap();
        let concepts = collect_concepts(&dir).unwrap();
        let concept = concepts.iter().find(|c| c.path == "patterns/sig.md").unwrap();
        let bee = bee_of(&concept.data);
        let signature = bee.get("signature").and_then(Value::as_str).unwrap();
        let pattern_date = concept.data.get("timestamp").and_then(Value::as_str).unwrap();
        let _ = measure_recurrence(&root, signature, pattern_date);
        assert_eq!(
            std::fs::read_to_string(join_rel(&dir, "patterns/sig.md")).unwrap(),
            before_bundle,
            "knowledge report must never touch the pattern it reads (U8 read-only)"
        );
        assert_eq!(
            std::fs::read_to_string(root.join(".bee").join("decisions.jsonl")).unwrap(),
            before_decisions,
            "knowledge report must never touch the decision log it reads (U8 read-only)"
        );
    }

    #[test]
    fn missing_logs_read_as_zero_recurrences_never_a_delegate() {
        let tmp = tempfile::tempdir().unwrap();
        let (count, last_seen) = measure_recurrence(tmp.path(), "anything", "2026-08-05");
        assert_eq!(count, 0);
        assert_eq!(last_seen, None);
    }
}
