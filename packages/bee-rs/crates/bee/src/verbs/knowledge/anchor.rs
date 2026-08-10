// The shared "--work <id> resolves to no bee.work-item concept" fallback
// (D1/D5/D6, extended by D34ccf18d, widened by U5): a bee.work-item concept
// whose bee.id matches `work` always wins; otherwise docs/history/<work>/
// CONTEXT.md and/or plan.md, whichever exist, both when both do; otherwise
// the ledger arm, which now fires on ANY of: `work`'s most recent
// .bee/logs/scribing-runs.jsonl entry (a small/tiny-lane feature logs its
// scoping synthesis as a decision instead of a docs/history/ artifact, so
// the ledger is the one thing still on disk for it), a bare
// .bee/lanes/<work>.json record with no scribing-run entry required (a
// bound feature carries a lane record from the moment it is bound, well
// before its first scribing run), or any docs/history/<work>/ file other
// than CONTEXT.md/plan.md; otherwise no anchor at all, which every caller
// renders as today's unknown_work refusal, byte for byte, unchanged.
//
// Consumed identically by knowledge::context's build_context_manifest and
// its byte-parity port at drivers/kctx.rs (D8) — both copies call the same
// resolve_anchor here so they cannot drift apart on the fallback's shape.
// The two ports carry independent `Concept` structs (kctx.rs is a hand-kept
// duplicate, not a re-export — see its own header comment), so this module
// is generic over anything with a bundle-relative path and parsed
// frontmatter data (`ConceptLike`) rather than depending on either port's
// concrete type; each port supplies a two-line impl for its own `Concept`.

use super::walk::Concept;
use crate::verbs::state_group::read_scribing_ledger;
use crate::verbs::workflow_store::{lanes_dir, read_lane_display};
use serde_json::{Map, Value};
use std::path::Path;

/// The shape knowledge::walk::Concept and drivers::kctx::Concept both carry
/// (independently ported, kept identical by construction).
pub(crate) trait ConceptLike {
    fn concept_path(&self) -> &str;
    fn concept_data(&self) -> &Map<String, Value>;
}

pub(crate) enum Anchor<'a, C: ConceptLike> {
    WorkItem(&'a C),
    History {
        paths: Vec<String>,
        meta: String,
        body: String,
        bytes: u64,
    },
    /// D34ccf18d, widened by U5: neither a work-item concept nor a
    /// docs/history/<work>/CONTEXT.md/plan.md exists, but at least one of
    /// `work`'s most recent .bee/logs/scribing-runs.jsonl entry, a bare
    /// .bee/lanes/<work>.json record, or some OTHER docs/history/<work>/
    /// file does. `meta`/`body`/`bytes` are built the same way the History
    /// arm builds them — from what was actually read off disk — so every
    /// caller below can treat the two arms identically.
    Ledger {
        paths: Vec<String>,
        meta: String,
        body: String,
        bytes: u64,
    },
}

impl<'a, C: ConceptLike> Anchor<'a, C> {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Anchor::WorkItem(_) => "work-item",
            Anchor::History { .. } => "history",
            Anchor::Ledger { .. } => "ledger",
        }
    }

    /// Repo-relative paths the anchor was built from — a single-element list
    /// for a work item's own bundle file, one or two docs/history entries
    /// for the history fallback; for the ledger fallback, whichever of
    /// .bee/logs/scribing-runs.jsonl, .bee/lanes/<work>.json, and other
    /// docs/history/<work>/ file names actually fired (U5).
    pub(crate) fn paths(&self) -> Vec<String> {
        match self {
            Anchor::WorkItem(c) => vec![format!("docs/knowledge/{}", c.concept_path())],
            Anchor::History { paths, .. } => paths.clone(),
            Anchor::Ledger { paths, .. } => paths.clone(),
        }
    }
}

fn matches_work_item(data: &Map<String, Value>, work: &str) -> bool {
    let bee = match data.get("bee") {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    matches!(data.get("type"), Some(Value::String(t)) if t == "bee.work-item")
        && matches!(bee.get("id"), Some(Value::String(id)) if id == work)
}

/// The first Markdown heading line (`# ...`) in `text`, trimmed of its
/// leading `#`s and surrounding whitespace — read straight off the file,
/// never composed prose (D10, concept-model-and-authoring.md:55).
fn first_heading(text: &str) -> Option<String> {
    for line in text.split('\n') {
        let line = line.trim_end_matches('\r');
        let Some(rest) = line.trim_start().strip_prefix('#') else { continue };
        let heading = rest.trim_start_matches('#').trim();
        if !heading.is_empty() {
            return Some(heading.to_string());
        }
    }
    None
}

/// A best-effort leading `---\n...\n---` frontmatter fence strip. Anchor
/// resolution only needs the body text and the first heading below it, not
/// the parsed fields, so it does not need either port's full Fm parser —
/// and CONTEXT.md/plan.md are hand-authored bee artifacts, not bundle
/// concepts, so no OKF frontmatter is expected on them in the first place.
fn strip_frontmatter_fence(raw: &str) -> String {
    let mut lines = raw.split('\n');
    let Some(first) = lines.next() else { return String::new() };
    if first.trim_end_matches('\r') != "---" {
        return raw.to_string();
    }
    let mut closed = false;
    let mut body_lines: Vec<&str> = Vec::new();
    for line in lines {
        if !closed && line.trim_end_matches('\r') == "---" {
            closed = true;
            continue;
        }
        if closed {
            body_lines.push(line);
        }
    }
    if closed {
        body_lines.join("\n")
    } else {
        raw.to_string()
    }
}

/// Read one docs/history/<work>/<name> file: its body (frontmatter fence
/// stripped when present), first heading, and real byte size. None when the
/// file does not exist or is unreadable.
fn read_history_file(root: &Path, work: &str, name: &str) -> Option<(String, String, u64)> {
    let mut abs = root.to_path_buf();
    for seg in ["docs", "history", work, name] {
        abs.push(seg);
    }
    let bytes = std::fs::metadata(&abs).ok()?.len();
    let raw = std::fs::read(&abs).ok()?;
    let raw = String::from_utf8_lossy(&raw).into_owned();
    let body = strip_frontmatter_fence(&raw);
    let heading = first_heading(&body).unwrap_or_default();
    Some((body, heading, bytes))
}

/// `work`'s most recent .bee/logs/scribing-runs.jsonl entry — most recent by
/// `ts` (a plain string compare; the ledger's ISO-8601 stamps sort
/// chronologically as UTF-16 code units, same as every other ISO-string sort
/// on this path). Reads through `read_scribing_ledger`
/// (verbs/state_group/ledger.rs) rather than re-parsing the file. Unlike
/// promote's own `latest_scribing_areas` reach, an entry naming `work` with
/// no (or an empty) `areas` array still counts — the ledger arm only needs
/// the entry to exist, not to carry an area list.
fn latest_ledger_areas(root: &Path, work: &str) -> Option<Vec<String>> {
    let entries = read_scribing_ledger(root).ok()?;
    let mut best: Option<(String, Vec<String>)> = None;
    for entry in &entries {
        if !matches!(entry.get("feature"), Some(Value::String(f)) if f == work) {
            continue;
        }
        let Some(ts) = entry.get("ts").and_then(Value::as_str) else { continue };
        let areas: Vec<String> = match entry.get("areas") {
            Some(Value::Array(items)) => {
                items.iter().filter_map(Value::as_str).map(str::to_string).collect()
            }
            _ => Vec::new(),
        };
        if best.as_ref().map(|(b, _)| ts > b.as_str()).unwrap_or(true) {
            best = Some((ts.to_string(), areas));
        }
    }
    best.map(|(_ts, areas)| areas)
}

/// `.bee/lanes/<work>.json`'s `last_scribing_run.next_action`, read through
/// the same fail-open display reader the ledger's own stamp lookup
/// (`best_scribing_stamp_ms`, verbs/state_group/ledger.rs) uses — a corrupt
/// or absent lane record simply carries no next_action; this is a read-only
/// anchor lookup, never a mutation, so it never throws.
fn ledger_next_action(root: &Path, work: &str) -> Option<String> {
    let lane = read_lane_display(root, work).ok()??;
    let next = lane.get("last_scribing_run")?.get("next_action")?.as_str()?;
    if next.is_empty() { None } else { Some(next.to_string()) }
}

/// U5: does `.bee/lanes/<work>.json` exist at all, regardless of whether it
/// carries a `last_scribing_run`? A bound feature gets this file the moment
/// it is bound (`bee state bind` / a lane claim), well before its first
/// scribing run — so its bare existence is its own ledger-arm signal, not
/// gated on `ledger_next_action` finding a next_action inside it.
fn lane_record_exists(root: &Path, work: &str) -> bool {
    lanes_dir(root).join(format!("{work}.json")).is_file()
}

/// U5: file names under docs/history/<work>/ other than CONTEXT.md/plan.md —
/// those two already won the History arm above by the time this runs, so a
/// hit here means the feature dropped something else there (a differently
/// named note, a scoped excerpt) that the History arm's fixed two-name list
/// never looked for. Sorted for byte-stable paths across directory-order
/// differences. `[]` (never an error) when the directory does not exist.
fn other_history_file_names(root: &Path, work: &str) -> Vec<String> {
    let mut dir = root.to_path_buf();
    for seg in ["docs", "history", work] {
        dir.push(seg);
    }
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "CONTEXT.md" && name != "plan.md")
        .collect();
    names.sort();
    names
}

/// D34ccf18d, widened by U5: the third and last arm, now firing on ANY of
/// three signals rather than requiring a scribing-ledger entry alone —
/// `work`'s most recent .bee/logs/scribing-runs.jsonl entry (unchanged), a
/// bare .bee/lanes/<work>.json record (`lane_record_exists`), or any
/// docs/history/<work>/ file other than CONTEXT.md/plan.md
/// (`other_history_file_names`). This session's own repro: a feature bound
/// with only a lane record (no scribing run yet, no docs/history/ file)
/// used to fall through to the caller's recency fallback — now it resolves
/// here instead. `meta`/`body` are built from whatever actually fired —
/// stamped area names, the lane record's next_action, other file names —
/// concatenated exactly as read, never composed prose, the same discipline
/// `meta_text_of`/`first_heading` already hold; `bytes` sizes the anchor off
/// that same built text, since (unlike the history arm) there is no single
/// backing file to stat.
fn read_ledger_anchor(root: &Path, work: &str) -> Option<(Vec<String>, String, String, u64)> {
    let areas = latest_ledger_areas(root, work);
    let next_action = ledger_next_action(root, work);
    let has_lane_record = lane_record_exists(root, work);
    let other_files = other_history_file_names(root, work);

    if areas.is_none() && !has_lane_record && other_files.is_empty() {
        return None;
    }

    let mut paths = Vec::new();
    if areas.is_some() {
        paths.push(".bee/logs/scribing-runs.jsonl".to_string());
    }
    if has_lane_record {
        paths.push(format!(".bee/lanes/{work}.json"));
    }
    for name in &other_files {
        paths.push(format!("docs/history/{work}/{name}"));
    }

    let mut meta_parts = vec![work.to_string()];
    if let Some(areas) = &areas {
        meta_parts.extend(areas.iter().cloned());
    }
    meta_parts.extend(other_files.iter().cloned());

    let mut body_parts: Vec<String> = Vec::new();
    if let Some(areas) = &areas {
        if !areas.is_empty() {
            body_parts.push(areas.join(" "));
        }
    }
    if let Some(next) = &next_action {
        body_parts.push(next.clone());
    }
    let body = body_parts.join("\n\n");
    let bytes = body.len() as u64;

    Some((paths, meta_parts.join(" "), body, bytes))
}

/// D5 then D1/D6, then D34ccf18d, widened by U5: a bee.work-item concept
/// whose bee.id matches `work` always wins; otherwise docs/history/<work>/
/// CONTEXT.md and plan.md, whichever exist, both when both do; otherwise the
/// ledger arm (`read_ledger_anchor`, U5-widened); otherwise None — the
/// caller's unknown_work refusal (D27).
pub(crate) fn resolve_anchor<'a, C: ConceptLike>(concepts: &'a [C], root: &Path, work: &str) -> Option<Anchor<'a, C>> {
    if let Some(c) = concepts.iter().find(|c| matches_work_item(c.concept_data(), work)) {
        return Some(Anchor::WorkItem(c));
    }

    let mut paths = Vec::new();
    let mut headings = Vec::new();
    let mut bodies = Vec::new();
    let mut bytes = 0u64;
    for name in ["CONTEXT.md", "plan.md"] {
        let Some((body, heading, file_bytes)) = read_history_file(root, work, name) else { continue };
        paths.push(format!("docs/history/{work}/{name}"));
        if !heading.is_empty() {
            headings.push(heading);
        }
        bodies.push(body);
        bytes += file_bytes;
    }
    if !paths.is_empty() {
        let mut meta_parts = vec![work.to_string()];
        meta_parts.extend(headings);
        return Some(Anchor::History {
            paths,
            meta: meta_parts.join(" "),
            body: bodies.join("\n\n"),
            bytes,
        });
    }

    let (paths, meta, body, bytes) = read_ledger_anchor(root, work)?;
    Some(Anchor::Ledger { paths, meta, body, bytes })
}

impl ConceptLike for Concept {
    fn concept_path(&self) -> &str {
        &self.path
    }
    fn concept_data(&self) -> &Map<String, Value> {
        &self.data
    }
}
