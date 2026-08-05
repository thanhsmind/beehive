// The shared "--work <id> resolves to no bee.work-item concept" fallback
// (D1/D5/D6): a bee.work-item concept whose bee.id matches `work` always
// wins; otherwise docs/history/<work>/CONTEXT.md and/or plan.md, whichever
// exist, both when both do; otherwise no anchor at all, which every caller
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
}

impl<'a, C: ConceptLike> Anchor<'a, C> {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Anchor::WorkItem(_) => "work-item",
            Anchor::History { .. } => "history",
        }
    }

    /// Repo-relative paths the anchor was built from — a single-element list
    /// for a work item's own bundle file, one or two docs/history entries
    /// for the fallback.
    pub(crate) fn paths(&self) -> Vec<String> {
        match self {
            Anchor::WorkItem(c) => vec![format!("docs/knowledge/{}", c.concept_path())],
            Anchor::History { paths, .. } => paths.clone(),
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

/// D5 then D1/D6: a bee.work-item concept whose bee.id matches `work` always
/// wins; otherwise docs/history/<work>/CONTEXT.md and plan.md, whichever
/// exist, both when both do; otherwise None — the caller's unknown_work
/// refusal (D27).
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
    if paths.is_empty() {
        return None;
    }
    let mut meta_parts = vec![work.to_string()];
    meta_parts.extend(headings);
    Some(Anchor::History {
        paths,
        meta: meta_parts.join(" "),
        body: bodies.join("\n\n"),
        bytes,
    })
}

impl ConceptLike for Concept {
    fn concept_path(&self) -> &str {
        &self.path
    }
    fn concept_data(&self) -> &Map<String, Value> {
        &self.data
    }
}
