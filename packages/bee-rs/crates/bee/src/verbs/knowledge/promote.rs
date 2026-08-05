// knowledge promote
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::textutil::{char_len, code_unit_cmp, js_default_sort, truncate_chars_head};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════════════════════
// knowledge promote  (bee.mjs handleKnowledgePromote + lib/knowledge.mjs
// buildPromotion / readCappedCellTraces / compareCellIds / oneLine /
// deviationText / verifySummary / isoDate / touchesSubject)
// ═══════════════════════════════════════════════════════════════════════════
//
// No collation anywhere on this path: compareCellIds is a hand-written
// natural-order comparator over `id.split(/(\d+)/)` using `<`/`>` (UTF-16
// code units), and the two `.sort()` calls (capped dates, area subjects)
// are JS default sorts, i.e. UTF-16 code units — never localeCompare. That
// is why this verb ports without the confidence-guard machinery
// verbs/feedback.rs needs.
//
// promote NEVER writes (D2): `writes` is always [], and nothing here opens a
// file for writing. Both typed refusals (missing_work / unknown_work) are
// deterministic text with no V8 message and no lock attempt, so they are
// reproduced natively.

/// `text.split(/\s+/).join(' ').trim()`, optionally capped at `limit` CHARS
/// with a trailing ellipsis (decision D3: char-based, not the historical
/// UTF-16-unit count).
pub(crate) fn one_line(text: &str, limit: usize) -> String {
    let mut flat = String::new();
    let mut in_ws = false;
    for c in text.chars() {
        if js_is_space(c) {
            in_ws = true;
        } else {
            if in_ws {
                flat.push(' ');
            }
            in_ws = false;
            flat.push(c);
        }
    }
    if in_ws {
        flat.push(' ');
    }
    let flat = flat.trim_matches(js_is_space).to_string();
    if limit == 0 || char_len(&flat) <= limit {
        return flat;
    }
    format!("{}\u{2026}", truncate_chars_head(&flat, limit - 1))
}

/// deviationText: a plain string, `type: description`, or JSON.stringify.
pub(crate) fn deviation_text(entry: &Value) -> String {
    match entry {
        Value::String(s) => s.clone(),
        Value::Object(m) => {
            let desc = m.get("description").and_then(Value::as_str).filter(|s| !s.is_empty());
            match desc {
                Some(d) => match m.get("type").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    Some(t) => format!("{t}: {d}"),
                    None => d.to_string(),
                },
                None => jsjson::stringify(entry),
            }
        }
        Value::Array(_) => jsjson::stringify(entry), // typeof [] === 'object'
        other => jsjson::js_to_string(other),
    }
}

/// verifySummary(trace): the first of verify_tail/verify_output/evidence/
/// summary in the parsed evidence JSON, else the raw text.
pub(crate) fn verify_summary(trace: &Value) -> Option<String> {
    let raw = match trace.get("verification_evidence") {
        Some(Value::String(s)) => s.as_str(),
        _ => "",
    };
    if raw.trim_matches(js_is_space).is_empty() {
        return Some(String::new());
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(parsed @ (Value::Object(_) | Value::Array(_))) => {
            for key in ["verify_tail", "verify_output", "evidence", "summary"] {
                if let Some(Value::String(s)) = parsed.get(key) {
                    if !s.trim_matches(js_is_space).is_empty() {
                        return Some(one_line(s, 200));
                    }
                }
            }
            Some(one_line(raw, 200))
        }
        Ok(_) => Some(one_line(raw, 200)), // parsed, but not an object
        // CUTOVER: JSON-looking text serde refuses used to delegate, because
        // only V8 could say whether its own parse threw. Nothing else parses
        // it here now, so the catch branch IS the answer: keep the raw text.
        Err(_) => Some(one_line(raw, 200)),
    }
}

/// compareCellIds — natural order over `id.split(/(\d+)/)`. Pure `<`/`>`
/// string compare (UTF-16 code units) plus numeric compare on digit runs.
pub(crate) fn compare_cell_ids(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn split(id: &str) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();
        let chars: Vec<char> = id.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let digit = chars[i].is_ascii_digit();
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() == digit {
                i += 1;
            }
            parts.push(chars[start..i].iter().collect());
        }
        parts
    }
    let left = split(a);
    let right = split(b);
    for i in 0..left.len().max(right.len()) {
        let (l, r) = (left.get(i), right.get(i));
        match (l, r) {
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(l), Some(r)) => {
                let both_numeric = !l.is_empty()
                    && !r.is_empty()
                    && l.chars().all(|c| c.is_ascii_digit())
                    && r.chars().all(|c| c.is_ascii_digit());
                if both_numeric {
                    // Number(l) — a run long enough to lose precision compares
                    // as the f64 both runtimes produce.
                    let (nl, nr) = (js_digits_to_f64(l), js_digits_to_f64(r));
                    if nl != nr {
                        return if nl < nr { Ordering::Less } else { Ordering::Greater };
                    }
                } else if l != r {
                    return code_unit_cmp(l, r);
                }
            }
        }
    }
    Ordering::Equal
}

pub(crate) fn js_digits_to_f64(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// isoDate: the `YYYY-MM-DD` prefix of an ISO-ish string, else None.
pub(crate) fn iso_date(v: Option<&Value>) -> Option<String> {
    let s = match v {
        Some(Value::String(s)) => s.as_str(),
        _ => return None,
    };
    let b = s.as_bytes();
    let d = |i: usize| i < b.len() && b[i].is_ascii_digit();
    if b.len() >= 10
        && d(0) && d(1) && d(2) && d(3)
        && b[4] == b'-'
        && d(5) && d(6)
        && b[7] == b'-'
        && d(8) && d(9)
    {
        Some(s[..10].to_string())
    } else {
        None
    }
}

/// touchesSubject: exact match, or either path containing the other as a dir.
pub(crate) fn touches_subject(file: &str, subject: &str) -> bool {
    file == subject
        || file.starts_with(&format!("{subject}/"))
        || subject.starts_with(&format!("{file}/"))
}

pub(crate) struct CappedCell {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) lane: Option<String>,
    pub(crate) behavior_change: bool,
    pub(crate) outcome: String,
    pub(crate) files_changed: Vec<String>,
    pub(crate) deviations: Vec<String>,
    pub(crate) failure_signatures: Vec<String>,
    pub(crate) verify: String,
    pub(crate) verify_summary: String,
    pub(crate) capped_at: Option<String>,
    pub(crate) trace_path: String,
}

pub(crate) fn cell_value(c: &CappedCell) -> Value {
    let arr = |v: &Vec<String>| Value::Array(v.iter().cloned().map(Value::String).collect());
    let mut m = Map::new();
    m.insert("id".into(), Value::String(c.id.clone()));
    m.insert("title".into(), Value::String(c.title.clone()));
    m.insert("lane".into(), c.lane.clone().map(Value::String).unwrap_or(Value::Null));
    m.insert("behavior_change".into(), Value::Bool(c.behavior_change));
    m.insert("outcome".into(), Value::String(c.outcome.clone()));
    m.insert("files_changed".into(), arr(&c.files_changed));
    m.insert("deviations".into(), arr(&c.deviations));
    m.insert("failure_signatures".into(), arr(&c.failure_signatures));
    m.insert("verify".into(), Value::String(c.verify.clone()));
    m.insert("verify_summary".into(), Value::String(c.verify_summary.clone()));
    m.insert("capped_at".into(), c.capped_at.clone().map(Value::String).unwrap_or(Value::Null));
    m.insert("trace_path".into(), Value::String(c.trace_path.clone()));
    Value::Object(m)
}

/// readCappedCellTraces(root, feature). None => delegate (an unreadable
/// entry or a non-UTF-8 name; an unparseable cell is skipped, like Node).
pub(crate) fn read_capped_cell_traces(root: &Path, feature: &str) -> Option<Vec<CappedCell>> {
    let dir = root.join(".bee").join("cells");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Some(Vec::new()); // Node's catch: an absent store yields []
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.ok()?;
        let ft = entry.file_type().ok()?;
        if !ft.is_file() {
            continue; // dirs (incl. archive/) and symlinks are skipped
        }
        let name = entry.file_name().to_str()?.to_string();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    // readdirSync order only decides which cells are seen, never their order
    // (the result is sorted by compareCellIds below) — but keep it stable.
    names.sort();

    let mut cells = Vec::new();
    for name in names {
        let bytes = std::fs::read(dir.join(&name)).ok()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let cell: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            // Node silently skips an unparseable cell. CUTOVER: the
            // "JSON-looking text serde refuses" sub-case used to delegate
            // rather than guess which V8 branch ran; there is no other branch
            // now, so every unparseable cell is skipped, as Node skipped it.
            Err(_) => continue,
        };
        let Value::Object(cell_map) = &cell else { continue };
        if cell_map.get("feature").and_then(Value::as_str) != Some(feature)
            || cell_map.get("status").and_then(Value::as_str) != Some("capped")
        {
            continue;
        }
        let empty = Value::Object(Map::new());
        let trace = match cell_map.get("trace") {
            Some(t @ Value::Object(_)) => t,
            Some(t @ Value::Array(_)) => t, // typeof [] === 'object'
            _ => &empty,
        };
        let deviations: Vec<String> = match trace.get("deviations") {
            Some(Value::Array(a)) => a
                .iter()
                .map(deviation_text)
                .filter(|t| !t.trim_matches(js_is_space).is_empty())
                .collect(),
            _ => Vec::new(),
        };
        let mut failure_signatures: Vec<String> = Vec::new();
        for key in ["attempts", "semantic_judge"] {
            if let Some(Value::Array(a)) = trace.get(key) {
                for item in a {
                    if let Some(Value::String(s)) = item.get("failure_signature") {
                        if !s.trim_matches(js_is_space).is_empty() {
                            failure_signatures.push(s.clone());
                        }
                    }
                }
            }
        }
        let id = match cell_map.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => name.trim_end_matches(".json").to_string(),
        };
        let cell_title = match cell_map.get("title") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let behavior_change = matches!(trace.get("behavior_change"), Some(Value::Bool(true)))
            || (trace.get("behavior_change").is_none()
                && matches!(cell_map.get("behavior_change"), Some(Value::Bool(true))));
        let outcome = match trace.get("outcome") {
            Some(Value::String(s)) if !s.trim_matches(js_is_space).is_empty() => s.clone(),
            _ => cell_title.clone(),
        };
        cells.push(CappedCell {
            id: id.clone(),
            title: cell_title,
            lane: match cell_map.get("lane") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            },
            behavior_change,
            outcome,
            files_changed: match trace.get("files_changed") {
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect(),
                _ => Vec::new(),
            },
            deviations,
            failure_signatures,
            verify: match cell_map.get("verify") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            },
            verify_summary: verify_summary(trace)?,
            capped_at: match trace.get("capped_at") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            },
            trace_path: format!(".bee/cells/{id}.json"),
        });
    }
    // Stable sort == JS Array.prototype.sort (spec-guaranteed since ES2019).
    cells.sort_by(|a, b| compare_cell_ids(&a.id, &b.id));
    Some(cells)
}

pub(crate) fn str_array(map: &Map<String, Value>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) enum Promo {
    Ok(Value),
    /// A deterministic typed refusal (bee.mjs emitError bytes).
    Thrown(String),
}

/// buildPromotion(root, {work}). None => delegate.
pub(crate) fn build_promotion(root: &Path, dir: &Path, work: &str) -> Option<Promo> {
    let work_id = work.trim_matches(js_is_space);
    if work_id.is_empty() {
        return Some(Promo::Thrown(
            "knowledge promote: missing_work — --work <id> is required (D38).".into(),
        ));
    }
    let concepts = collect_concepts(dir)?;
    // D5 then D1/D6: a bee.work-item concept whose bee.id matches always
    // wins; otherwise docs/history/<work_id>/CONTEXT.md and/or plan.md,
    // whichever exist; otherwise no anchor at all — unknown_work, unchanged
    // byte for byte (D38), same shared resolver context.rs and kctx.rs use.
    let Some(anchor) = resolve_anchor(&concepts, root, work_id) else {
        return Some(Promo::Thrown(format!(
            "knowledge promote: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D38)."
        )));
    };
    let work_concept: Option<&Concept> = match &anchor {
        Anchor::WorkItem(c) => Some(*c),
        Anchor::History { .. } => None,
    };

    let work_bee = work_concept.map(|c| bee_of(&c.data)).unwrap_or_default();
    let work_areas: Vec<String> = str_array(&work_bee, "areas")
        .into_iter()
        .filter(|a| !a.is_empty())
        .collect();
    let work_decisions = str_array(&work_bee, "decisions");
    let work_tags = work_concept.map(|c| str_array(&c.data, "tags")).unwrap_or_default();
    let cells = read_capped_cell_traces(root, work_id)?;

    // ── (a) delivery draft ────────────────────────────────────────────────
    // A history anchor is not a bundle concept and has no directory of its
    // own to save beside — the proposed save path is the canonical
    // docs/knowledge/work/<slug>/delivery.md (a PROPOSAL; nothing is
    // written, so D5 holds).
    let work_dir = match work_concept {
        Some(wc) => dir_of(&wc.path).to_string(),
        None => format!("work/{work_id}"),
    };
    let delivery_path = if work_dir.is_empty() {
        "delivery.md".to_string()
    } else {
        format!("{work_dir}/delivery.md")
    };
    let mut capped_dates: Vec<String> = cells
        .iter()
        .filter_map(|c| iso_date(c.capped_at.as_ref().map(|s| Value::String(s.clone())).as_ref()))
        .collect();
    js_default_sort(&mut capped_dates);
    let timestamp = match capped_dates.last() {
        Some(d) => Some(d.clone()),
        None => work_concept.and_then(|wc| iso_date(wc.data.get("timestamp"))),
    };
    let deviation_count: usize = cells.iter().map(|c| c.deviations.len()).sum();
    let work_title = match work_concept.and_then(|wc| wc.data.get("title")) {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => work_id.to_string(),
    };

    let mut delivery_data = Map::new();
    delivery_data.insert("type".into(), Value::String("bee.delivery".into()));
    delivery_data.insert("title".into(), Value::String(format!("{work_title} — delivery")));
    delivery_data.insert(
        "description".into(),
        Value::String(format!(
            "Delivery record proposed by bee knowledge promote for work item {work_id}: {} capped cell(s), {deviation_count} recorded deviation(s).",
            cells.len()
        )),
    );
    if !work_tags.is_empty() {
        delivery_data.insert(
            "tags".into(),
            Value::Array(work_tags.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(ts) = &timestamp {
        delivery_data.insert("timestamp".into(), Value::String(ts.clone()));
    }
    let mut delivery_bee = Map::new();
    delivery_bee.insert("id".into(), Value::String(format!("{work_id}-delivery")));
    delivery_bee.insert("lifecycle".into(), Value::String("active".into()));
    if !work_areas.is_empty() {
        delivery_bee.insert(
            "areas".into(),
            Value::Array(work_areas.iter().cloned().map(Value::String).collect()),
        );
    }
    delivery_bee.insert(
        "required_context".into(),
        Value::Array(match work_concept {
            Some(wc) => vec![Value::String(wc.path.clone())],
            None => anchor.paths().into_iter().map(Value::String).collect(),
        }),
    );
    if !work_decisions.is_empty() {
        delivery_bee.insert(
            "decisions".into(),
            Value::Array(work_decisions.iter().cloned().map(Value::String).collect()),
        );
    }
    let mut sources: Vec<Value> = match work_concept {
        Some(wc) => vec![Value::String(format!("docs/knowledge/{}", wc.path))],
        None => anchor.paths().into_iter().map(Value::String).collect(),
    };
    sources.extend(cells.iter().map(|c| Value::String(c.trace_path.clone())));
    delivery_bee.insert("sources".into(), Value::Array(sources));
    if let Some(lane) = work_bee.get("lane").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        delivery_bee.insert("lane".into(), Value::String(lane.to_string()));
    }
    delivery_data.insert("bee".into(), Value::Object(delivery_bee));

    let shipped: Vec<String> = if !cells.is_empty() {
        cells
            .iter()
            .map(|c| {
                format!(
                    "- **{}** — {} ({} file(s) changed)",
                    c.id,
                    one_line(&c.outcome, 0),
                    c.files_changed.len()
                )
            })
            .collect()
    } else {
        vec![format!(
            "No capped cell trace for work item {work_id} exists in .bee/cells/ at proposal time."
        )]
    };
    let verified: Vec<String> = if !cells.is_empty() {
        cells
            .iter()
            .map(|c| {
                let suffix = if c.verify_summary.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", c.verify_summary)
                };
                format!("- **{}** — `{}`{suffix}", c.id, c.verify)
            })
            .collect()
    } else {
        vec!["Nothing to verify: no capped cell trace was found.".to_string()]
    };
    let mut deviation_lines: Vec<String> = Vec::new();
    for c in &cells {
        for d in &c.deviations {
            deviation_lines.push(format!("- **{}** — {}", c.id, one_line(d, 0)));
        }
    }
    if deviation_lines.is_empty() {
        deviation_lines.push("None recorded in the capped cell traces.".to_string());
    }

    let mut body: Vec<String> = vec![
        format!("# {work_title} — Delivery"),
        String::new(),
        "## What shipped".into(),
        String::new(),
    ];
    body.extend(shipped);
    body.extend([String::new(), "## Verify".into(), String::new(),
        "Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.".into(),
        String::new()]);
    body.extend(verified);
    body.extend([String::new(), "## Deviations".into(), String::new()]);
    body.extend(deviation_lines);
    body.extend([String::new(), "## Provenance".into(), String::new()]);
    let provenance_subject = match work_concept {
        Some(wc) => format!("the work item `docs/knowledge/{}`", wc.path),
        None => format!("the anchor `{}`", anchor.paths().join("`, `")),
    };
    body.push(format!(
        "Proposed by `bee knowledge promote --work {work_id}` from {} capped cell trace(s) in `.bee/cells/` and {provenance_subject}. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.",
        cells.len()
    ));
    body.push(String::new());
    let delivery_content = format!("{}\n{}", emit_frontmatter(&delivery_data).ok()?, body.join("\n"));

    let mut delivery = Map::new();
    delivery.insert("path".into(), Value::String(delivery_path.clone()));
    delivery.insert(
        "repo_path".into(),
        Value::String(format!("docs/knowledge/{delivery_path}")),
    );
    delivery.insert("content".into(), Value::String(delivery_content));

    // ── (b) area updates ──────────────────────────────────────────────────
    let mut area_updates: Vec<Value> = Vec::new();
    for area in &work_areas {
        let mut subjects: Vec<String> = Vec::new(); // insertion-ordered Set
        for concept in &concepts {
            let bee = bee_of(&concept.data);
            if !str_array(&bee, "areas").iter().any(|a| a == area) {
                continue;
            }
            let own = format!("docs/knowledge/{}", concept.path);
            if !subjects.contains(&own) {
                subjects.push(own);
            }
            for source in str_array(&bee, "sources") {
                if !source.is_empty() && !subjects.contains(&source) {
                    subjects.push(source);
                }
            }
        }
        let mut bullets: Vec<Value> = Vec::new();
        for c in &cells {
            if !c.behavior_change {
                continue;
            }
            let touched: Vec<String> = c
                .files_changed
                .iter()
                .filter(|file| subjects.iter().any(|s| touches_subject(file, s)))
                .cloned()
                .collect();
            if touched.is_empty() {
                continue;
            }
            let mut b = Map::new();
            b.insert("cell".into(), Value::String(c.id.clone()));
            b.insert("text".into(), Value::String(one_line(&c.outcome, 0)));
            b.insert(
                "files".into(),
                Value::Array(touched.into_iter().map(Value::String).collect()),
            );
            b.insert("trace".into(), Value::String(c.trace_path.clone()));
            bullets.push(Value::Object(b));
        }
        let mut sorted = subjects.clone();
        js_default_sort(&mut sorted);
        let mut u = Map::new();
        u.insert("area".into(), Value::String(area.clone()));
        u.insert(
            "subjects".into(),
            Value::Array(sorted.into_iter().map(Value::String).collect()),
        );
        u.insert("bullets".into(), Value::Array(bullets));
        area_updates.push(Value::Object(u));
    }

    // ── (c) pattern candidates ────────────────────────────────────────────
    let mut pattern_candidates: Vec<Value> = Vec::new();
    for c in &cells {
        if c.deviations.is_empty() && c.failure_signatures.is_empty() {
            continue;
        }
        let mut evidence: Vec<(&'static str, String)> = Vec::new();
        for d in &c.deviations {
            evidence.push(("deviation", d.clone()));
        }
        for f in &c.failure_signatures {
            evidence.push(("failure_signature", f.clone()));
        }
        let mut data = Map::new();
        data.insert("type".into(), Value::String("bee.pattern".into()));
        data.insert(
            "title".into(),
            Value::String(format!("{work_id} cell {} — pitfall candidate", c.id)),
        );
        data.insert(
            "description".into(),
            Value::String(format!(
                "Pitfall candidate mined from cell {}'s capped trace: {}",
                c.id,
                one_line(&evidence[0].1, 160)
            )),
        );
        if let Some(ts) = iso_date(c.capped_at.as_ref().map(|s| Value::String(s.clone())).as_ref())
        {
            data.insert("timestamp".into(), Value::String(ts));
        }
        let mut b = Map::new();
        b.insert("id".into(), Value::String(format!("{work_id}-{}-pitfall", c.id)));
        b.insert("lifecycle".into(), Value::String("draft".into()));
        if !work_areas.is_empty() {
            b.insert(
                "areas".into(),
                Value::Array(work_areas.iter().cloned().map(Value::String).collect()),
            );
        }
        b.insert(
            "sources".into(),
            Value::Array(vec![Value::String(c.trace_path.clone())]),
        );
        b.insert("polarity".into(), Value::String("pitfall".into()));
        data.insert("bee".into(), Value::Object(b));

        let mut lines: Vec<String> = vec![
            format!("# {work_id} cell {} — pitfall candidate", c.id),
            String::new(),
            "## What the cell did".into(),
            String::new(),
            one_line(&c.outcome, 0),
            String::new(),
            format!("## Recorded evidence (verbatim from {})", c.trace_path),
            String::new(),
        ];
        for (kind, text) in &evidence {
            lines.push(format!("- **{kind}** — {}", one_line(text, 0)));
        }
        lines.extend([
            String::new(),
            "## Status".into(),
            String::new(),
            "Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.".into(),
            String::new(),
        ]);

        let rel = format!("patterns/{work_id}-{}-pitfall.md", c.id);
        let mut cand = Map::new();
        cand.insert("cell".into(), Value::String(c.id.clone()));
        cand.insert("path".into(), Value::String(rel.clone()));
        cand.insert("repo_path".into(), Value::String(format!("docs/knowledge/{rel}")));
        cand.insert(
            "evidence".into(),
            Value::Array(
                evidence
                    .iter()
                    .map(|(kind, text)| {
                        let mut e = Map::new();
                        e.insert("kind".into(), Value::String((*kind).into()));
                        e.insert("text".into(), Value::String(text.clone()));
                        Value::Object(e)
                    })
                    .collect(),
            ),
        );
        cand.insert(
            "content".into(),
            Value::String(format!("{}\n{}", emit_frontmatter(&data).ok()?, lines.join("\n"))),
        );
        pattern_candidates.push(Value::Object(cand));
    }

    // work_item carries the concept path under a work-item anchor, unchanged;
    // under a history anchor there is no concept, so it carries the anchor's
    // own repo-relative path(s) instead (D1).
    let work_item_value = match work_concept {
        Some(wc) => wc.path.clone(),
        None => anchor.paths().join(" + "),
    };
    let mut out = Map::new();
    out.insert("work".into(), Value::String(work_id.to_string()));
    out.insert("work_item".into(), Value::String(work_item_value));
    out.insert("anchor".into(), {
        let mut m = Map::new();
        m.insert("kind".into(), Value::String(anchor.kind().to_string()));
        m.insert(
            "paths".into(),
            Value::Array(anchor.paths().into_iter().map(Value::String).collect()),
        );
        Value::Object(m)
    });
    out.insert(
        "cells".into(),
        Value::Array(cells.iter().map(cell_value).collect()),
    );
    out.insert("delivery".into(), Value::Object(delivery));
    out.insert("area_updates".into(), Value::Array(area_updates));
    out.insert("pattern_candidates".into(), Value::Array(pattern_candidates));
    out.insert("writes".into(), Value::Array(Vec::new()));
    Some(Promo::Ok(Value::Object(out)))
}

/// handleKnowledgePromote's human rendering.
pub(crate) fn promote_text(p: &Value) -> String {
    let cells = p["cells"].as_array().cloned().unwrap_or_default();
    let ids: Vec<String> = cells
        .iter()
        .map(|c| c["id"].as_str().unwrap_or("").to_string())
        .collect();
    let head = format!(
        "promote proposal for work item \"{}\" ({}) — {} capped cell(s){}",
        p["work"].as_str().unwrap_or(""),
        p["work_item"].as_str().unwrap_or(""),
        cells.len(),
        if cells.is_empty() { String::new() } else { format!(": {}", ids.join(", ")) }
    );
    let anchor_paths: Vec<String> = p["anchor"]["paths"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    let mut lines = vec![
        head,
        format!(
            "anchor: {} — {}",
            p["anchor"]["kind"].as_str().unwrap_or(""),
            anchor_paths.join(", ")
        ),
        "PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.".to_string(),
        String::new(),
        format!("(a) DELIVERY DRAFT — save as {}", p["delivery"]["repo_path"].as_str().unwrap_or("")),
        String::new(),
        strip_one_trailing_newline(p["delivery"]["content"].as_str().unwrap_or("")),
        String::new(),
        "(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell".to_string(),
        String::new(),
    ];
    let area_updates = p["area_updates"].as_array().cloned().unwrap_or_default();
    if area_updates.is_empty() {
        lines.push(
            "None: the work item declares no bee.areas, so there is no area to sync (D19)."
                .to_string(),
        );
        lines.push(String::new());
    }
    for update in &area_updates {
        lines.push(format!("area {}:", update["area"].as_str().unwrap_or("")));
        let bullets = update["bullets"].as_array().cloned().unwrap_or_default();
        if bullets.is_empty() {
            lines.push("  (no capped behavior_change cell touched this area's subjects)".into());
        }
        for b in &bullets {
            let files: Vec<String> = b["files"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                .unwrap_or_default();
            lines.push(format!(
                "  - [{}] {} — touched {} (trace {})",
                b["cell"].as_str().unwrap_or(""),
                b["text"].as_str().unwrap_or(""),
                files.join(", "),
                b["trace"].as_str().unwrap_or("")
            ));
        }
        lines.push(String::new());
    }
    lines.push("(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall".into());
    lines.push(String::new());
    let candidates = p["pattern_candidates"].as_array().cloned().unwrap_or_default();
    if candidates.is_empty() {
        lines.push("None: no capped cell trace carries a deviation or a failure signature.".into());
        lines.push(String::new());
    }
    for c in &candidates {
        lines.push(format!(
            "from cell {} — save as {}",
            c["cell"].as_str().unwrap_or(""),
            c["repo_path"].as_str().unwrap_or("")
        ));
        lines.push(String::new());
        lines.push(strip_one_trailing_newline(c["content"].as_str().unwrap_or("")));
        lines.push(String::new());
    }
    let bullet_total: usize = area_updates
        .iter()
        .map(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
        .sum();
    lines.push(format!(
        "knowledge promote: {} capped cell(s) mined, 1 delivery draft, {bullet_total} area bullet(s), {} pattern candidate(s), 0 file(s) written.",
        cells.len(),
        candidates.len()
    ));
    lines.join("\n")
}

/// `.replace(/\n$/, '')`
pub(crate) fn strip_one_trailing_newline(s: &str) -> String {
    s.strip_suffix('\n').unwrap_or(s).to_string()
}

pub(crate) fn run_promote(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["work"]) {
        return None;
    }
    // validate() owns the missing/empty required flag; a bare `--work` is
    // impossible (not a FLAG_ALONE_BOOLEAN).
    let work = match flags.get("work") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let ctx = match g_prelude("knowledge promote", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    match build_promotion(&ctx.root, &dir, &work)? {
        Promo::Thrown(msg) => Some(ctx.fail(&msg)),
        Promo::Ok(proposal) => {
            let text = promote_text(&proposal);
            Some(ctx.emit(&proposal, &text, 0))
        }
    }
}
