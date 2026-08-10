// bee knowledge search — read-only symptom pull over the bundle's patterns
// and area concepts.
//
// Feature: docs/history/knowledge-search/CONTEXT.md (D1-D5, D7).
//
// Grammar mirrors `decisions search` (D1): `--text` is required,
// whitespace-split into terms, lowercased, OR-matched by substring
// containment (the same rule `decisions search --text` and its
// `count_term_hits` scorer use); `--limit` is optional (default 5) and only
// ever widens the returned rows; `json` is the ordinary flag every verb
// carries. A missing/empty `--text` returns None here so the router's own
// BAD ARGS refusal (crate::catalog::missing_required, driven by the
// registry's `required: ["text"]`) names the gap — the same shape `context`
// uses for its required `--work`.
//
// Corpus (D2): `bee.pattern` and `bee.area` concepts of the active bundle
// only, via the shared `collect_concepts` walk — which already never leaves
// docs/knowledge/ (so `docs/specs/` and the `.bee/` decisions store are
// never reachable) and already skips the generated `index.md`/`log.md`
// files (`is_reserved_basename`). `bee.decision` concepts live in the same
// walk but are filtered out here by type: `decisions search` owns that
// corpus (D2).
//
// Match fields (D3): concept title, body (frontmatter stripped), and the
// frontmatter `bee.sources`/`bee.decisions` entries — where incident
// signatures and error text often live. Ranking is deterministic: term-hit
// count descending, then the bundle's own `timestamp` frontmatter field
// descending (ISO `YYYY-MM-DD` sorts lexically), then path ascending as the
// final tiebreak so two runs over the same bundle are byte-identical. A
// concept with no `timestamp` sorts as the oldest (empty string is the
// lexical minimum), never crashes the sort.
//
// Output (D4): top-N rows of `path — title — why-matched`, where the
// why-matched clause names each hit term and the field(s) it matched in.
// Zero hits is an empty `results` array plus a one-line note, exit 0 —
// never a typed refusal (D5): a search miss is data mid-debug, not an
// error.
//
// Read-only (D7): no `std::fs::write`/`create_dir_all`/state call anywhere
// in this file — every function here only reads the bundle already walked
// by `collect_concepts`/`concept_body`. Callable at any phase, from any
// checkout, exactly like every other verb in this read-only group.

#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_number_flag, js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── grammar ────────────────────────────────────────────────────────────

pub(crate) const SEARCH_DEFAULT_LIMIT: usize = 5;

/// `text.toLowerCase().split(/\s+/)`-equivalent, deduped in first-seen
/// order (an OR set, not a multiset — repeating a term in the query cannot
/// inflate its own hit count).
pub(crate) fn split_search_terms(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    let mut terms: Vec<String> = Vec::new();
    for raw in lowered.split(js_is_space) {
        if raw.is_empty() {
            continue;
        }
        if !terms.iter().any(|t| t == raw) {
            terms.push(raw.to_string());
        }
    }
    terms
}

// ─── matching (D3) ──────────────────────────────────────────────────────

/// A concept's matchable text, lowercased once per field so every term test
/// below is a plain substring check.
struct SearchFields {
    title: String,
    body: String,
    sources: String,
    decisions: String,
}

fn joined_lower(items: Option<&Value>) -> String {
    match items {
        Some(Value::Array(a)) => a
            .iter()
            .map(jsjson::js_to_string)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
        _ => String::new(),
    }
}

fn search_fields(dir: &Path, concept: &Concept) -> SearchFields {
    let bee = bee_of(&concept.data);
    SearchFields {
        title: str_field(&concept.data, "title").unwrap_or("").to_lowercase(),
        body: concept_body(dir, &concept.path).unwrap_or_default().to_lowercase(),
        sources: joined_lower(bee.get("sources")),
        decisions: joined_lower(bee.get("decisions")),
    }
}

/// Which fields (title/body/sources/decisions, D3's field order) a
/// lowercased TERM hits — the raw material for the why-matched line (D4).
fn term_fields(term: &str, f: &SearchFields) -> Vec<&'static str> {
    let mut out = Vec::new();
    if f.title.contains(term) {
        out.push("title");
    }
    if f.body.contains(term) {
        out.push("body");
    }
    if f.sources.contains(term) {
        out.push("sources");
    }
    if f.decisions.contains(term) {
        out.push("decisions");
    }
    out
}

// ─── ranking (D3) + rows (D4) ───────────────────────────────────────────

pub(crate) struct SearchHit {
    pub(crate) path: String, // "docs/knowledge/<rel>" — directly readable
    pub(crate) title: String,
    pub(crate) hits: usize,
    pub(crate) timestamp: String,
    pub(crate) why: String,
}

/// bee.pattern + bee.area concepts only (D2); every other type (including
/// bee.decision) is skipped here without ever being read.
fn is_searched_type(concept: &Concept) -> bool {
    matches!(
        concept.data.get("type").and_then(Value::as_str),
        Some("bee.pattern") | Some("bee.area")
    )
}

/// None => delegate (the same walk/name-shape gap `collect_concepts`
/// signals for every other knowledge verb); an absent bundle directory is
/// NOT this arm — `collect_concepts` already answers `Some(vec![])` for it,
/// which reaches the empty-result path below like a real zero-hit search.
pub(crate) fn search_bundle(dir: &Path, text: &str, limit: usize) -> Option<Vec<SearchHit>> {
    let terms = split_search_terms(text);
    if terms.is_empty() {
        return Some(Vec::new());
    }
    let concepts = collect_concepts(dir)?;

    let mut hits: Vec<SearchHit> = Vec::new();
    for concept in concepts.iter().filter(|c| is_searched_type(c)) {
        let fields = search_fields(dir, concept);
        let mut hit_count = 0usize;
        let mut parts: Vec<String> = Vec::new();
        for term in &terms {
            let matched = term_fields(term, &fields);
            if matched.is_empty() {
                continue;
            }
            hit_count += 1;
            parts.push(format!("{term:?} ({})", matched.join(", ")));
        }
        if hit_count == 0 {
            continue;
        }
        let title = str_field(&concept.data, "title").unwrap_or(&concept.path).to_string();
        let timestamp = match concept.data.get("timestamp") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        hits.push(SearchHit {
            path: format!("docs/knowledge/{}", concept.path),
            title,
            hits: hit_count,
            timestamp,
            why: parts.join("; "),
        });
    }

    // Deterministic order (D3): hit count desc, then timestamp desc (a
    // missing timestamp is the lexical minimum, so it sorts last on its
    // own), then path asc as the final, always-present tiebreak.
    hits.sort_by(|a, b| {
        b.hits
            .cmp(&a.hits)
            .then_with(|| b.timestamp.cmp(&a.timestamp))
            .then_with(|| a.path.cmp(&b.path))
    });
    hits.truncate(limit);
    Some(hits)
}

pub(crate) fn zero_hit_note(text: &str) -> String {
    format!("No knowledge results matching {}.", js_quote_str(text))
}

// ─── routing ────────────────────────────────────────────────────────────

pub(crate) fn run_search(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["text", "limit"]) {
        return None;
    }
    // --text: required, non-empty after trim (D1). Missing/empty falls
    // through to the router's registry-driven BAD ARGS refusal, same as
    // `context`'s required --work.
    let text = match flags.get("text") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => s.clone(),
        _ => return None,
    };
    // --limit: optional positive integer, default 5, widens only.
    let limit: usize = match flags.get("limit") {
        None => SEARCH_DEFAULT_LIMIT,
        Some(FlagV::S(s)) => match js_number_flag(s) {
            Ok(Some(n)) if n.is_finite() && n >= 1.0 => n as usize,
            _ => return None,
        },
        Some(FlagV::Present) => return None,
    };

    let ctx = match g_prelude("knowledge search", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let hits = search_bundle(&dir, &text, limit)?;

    let mut lines: Vec<String> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    for hit in &hits {
        lines.push(format!("{} — {} — {}", hit.path, hit.title, hit.why));
        let mut row = Map::new();
        row.insert("path".into(), Value::String(hit.path.clone()));
        row.insert("title".into(), Value::String(hit.title.clone()));
        row.insert("hits".into(), Value::from(hit.hits));
        row.insert("why".into(), Value::String(hit.why.clone()));
        rows.push(Value::Object(row));
    }
    if rows.is_empty() {
        lines.push(zero_hit_note(&text));
    }

    let count = rows.len();
    let mut result = Map::new();
    result.insert("text".into(), Value::String(text));
    result.insert("results".into(), Value::Array(rows));
    result.insert("count".into(), Value::from(count));
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

    fn pattern(title: &str, timestamp: &str, sources: &str, decisions: &str, body: &str) -> String {
        format!(
            "---\ntype: bee.pattern\ntitle: {title}\ndescription: {title}\ntimestamp: {timestamp}\nbee:\n  id: {}\n  lifecycle: active\n  sources: [{sources}]\n  decisions: [{decisions}]\n---\n{body}\n",
            title.to_lowercase().replace(' ', "-"),
        )
    }

    #[test]
    fn search_ranks_by_hit_count_then_recency_then_path() {
        let (_tmp, dir) = bundle();
        // both query terms hit ("cache" in title, "warm" in body) — 2 hits,
        // ranks first regardless of the (deliberately oldest) timestamp.
        write(
            &dir,
            "patterns/both-terms.md",
            &pattern("Cache thing", "2026-01-01", "", "", "a warm cache is a happy cache"),
        );
        // only "cache" hits — 1 hit, newer timestamp than the other 1-hit row.
        write(
            &dir,
            "patterns/newer-one-hit.md",
            &pattern("Cache only", "2026-07-20", "", "", "no overlap here"),
        );
        // only "warm" hits — 1 hit, older timestamp — loses the recency tiebreak.
        write(
            &dir,
            "patterns/older-one-hit.md",
            &pattern("Warm blanket", "2026-07-01", "", "", "no overlap here either"),
        );
        // equal hits (1, "cache" only) AND equal timestamp as newer-one-hit —
        // path ascending is the final tiebreak ("patterns/newer-one-hit.md" <
        // "patterns/tie-same-timestamp.md" lexically).
        write(
            &dir,
            "patterns/tie-same-timestamp.md",
            &pattern("Cache tie", "2026-07-20", "", "", "no overlap here"),
        );
        // no hit at all — never appears.
        write(&dir, "patterns/none.md", &pattern("Unrelated", "2026-08-05", "", "", "nothing here"));
        // wrong type — never appears even with a perfect hit.
        write(
            &dir,
            "not-a-pattern.md",
            "---\ntype: bee.decision\ntitle: Cache warm decision\ndescription: cache warm\nbee:\n  id: dec-1\n  lifecycle: active\n---\ncache warm\n",
        );

        let hits = search_bundle(&dir, "cache warm", SEARCH_DEFAULT_LIMIT).expect("bundle walk");
        let paths: Vec<&str> = hits.iter().map(|h| h.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "docs/knowledge/patterns/both-terms.md",
                "docs/knowledge/patterns/newer-one-hit.md",
                "docs/knowledge/patterns/tie-same-timestamp.md",
                "docs/knowledge/patterns/older-one-hit.md",
            ]
        );
        assert_eq!(hits[0].hits, 2);
        assert_eq!(hits[1].hits, 1);
        assert_eq!(hits[2].hits, 1);
        assert_eq!(hits[3].hits, 1);
    }

    #[test]
    fn search_limit_widens_and_defaults_to_five() {
        let (_tmp, dir) = bundle();
        for i in 0..7 {
            write(
                &dir,
                &format!("patterns/p{i:02}.md"),
                &pattern(&format!("Widget {i}"), "2026-07-01", "", "", "widget lesson"),
            );
        }
        let default = search_bundle(&dir, "widget", SEARCH_DEFAULT_LIMIT).unwrap();
        assert_eq!(default.len(), 5);
        let widened = search_bundle(&dir, "widget", 7).unwrap();
        assert_eq!(widened.len(), 7);
    }

    #[test]
    fn search_why_matched_names_term_and_field() {
        let (_tmp, dir) = bundle();
        write(
            &dir,
            "patterns/hit.md",
            &pattern(
                "Retry storm",
                "2026-07-10",
                "\"error log: connection refused\"",
                "",
                "A retry storm follows a connection refused burst.",
            ),
        );
        let hits = search_bundle(&dir, "storm refused missing", SEARCH_DEFAULT_LIMIT).unwrap();
        assert_eq!(hits.len(), 1);
        let why = &hits[0].why;
        assert!(why.contains("\"storm\" (title, body)"), "{why}");
        assert!(why.contains("\"refused\" (body, sources)"), "{why}");
        assert!(!why.contains("missing"), "an unmatched term must not appear: {why}");
        assert_eq!(hits[0].hits, 2); // "storm" and "refused" hit; "missing" does not
    }

    #[test]
    fn search_zero_hit_is_an_empty_result_never_a_delegate() {
        let (_tmp, dir) = bundle();
        write(&dir, "patterns/only.md", &pattern("Something else", "2026-07-01", "", "", "no overlap"));
        let hits = search_bundle(&dir, "nonexistent-term-xyz", SEARCH_DEFAULT_LIMIT).unwrap();
        assert!(hits.is_empty());
        assert_eq!(zero_hit_note("nonexistent-term-xyz"), "No knowledge results matching \"nonexistent-term-xyz\".");
    }

    #[test]
    fn search_bundle_absent_is_a_zero_hit_result_not_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge"); // never created
        let hits = search_bundle(&dir, "anything", SEARCH_DEFAULT_LIMIT).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn search_never_writes_the_bundle_it_reads() {
        let (_tmp, dir) = bundle();
        write(&dir, "patterns/hit.md", &pattern("Cache thing", "2026-07-01", "", "", "cache body"));
        let before: Vec<(PathBuf, std::time::SystemTime)> = std::fs::read_dir(join_rel(&dir, "patterns"))
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.path(), e.metadata().unwrap().modified().unwrap())
            })
            .collect();
        let _ = search_bundle(&dir, "cache", SEARCH_DEFAULT_LIMIT).unwrap();
        let after: Vec<(PathBuf, std::time::SystemTime)> = std::fs::read_dir(join_rel(&dir, "patterns"))
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.path(), e.metadata().unwrap().modified().unwrap())
            })
            .collect();
        assert_eq!(before, after, "knowledge search must never touch the bundle it reads (D7)");
    }
}
