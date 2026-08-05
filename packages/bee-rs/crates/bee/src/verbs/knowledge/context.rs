// buildContextManifest and relevance ranking
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── context (buildContextManifest + relevance ranking) ────────────────────

pub(crate) const CONTEXT_ESTIMATOR: &str = "bytes/4";

pub(crate) const KEEP: usize = 20;

pub(crate) const FLOOR: usize = 3;

pub(crate) const META_WEIGHT: f64 = 0.25;

pub(crate) const BODY_WEIGHT: f64 = 1.0;

pub(crate) const TAG_WEIGHT: f64 = 0.05;

pub(crate) const AREA_WEIGHT: f64 = 0.05;

pub(crate) const ZERO_SIGNAL_MIN_POPULATION: usize = 10;

pub(crate) const ZERO_SIGNAL_MAX_RATIO: f64 = 0.5;

pub(crate) const RELEVANCE_STOPWORDS: &str = "a an the and or but if then else for of to in on at by is are was were be been being it its this that these those with without from as not no never always every each any all some one two three you your we our they their he she i me my do does did done can could should would may might must will shall have has had so than which who whom what when where why how more most less least very just only also into out up down over under again further once here there both few other own same too s t don now";

pub(crate) fn stopwords() -> HashSet<&'static str> {
    RELEVANCE_STOPWORDS.split(' ').collect()
}

/// relevanceTokens(text) — lowercase, [a-z0-9]+ runs, >2 chars, stopped,
/// crude singularization.
pub(crate) fn relevance_tokens(text: &str, stops: &HashSet<&'static str>) -> Vec<String> {
    let lower: String = text.to_lowercase();
    let mut out = Vec::new();
    for raw in lower.split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit()) {
        if raw.len() <= 2 || stops.contains(raw) {
            continue;
        }
        let token = if raw.len() > 4 && raw.ends_with('s') && !raw.ends_with("ss") {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        out.push(token.to_string());
    }
    out
}

/// Insertion-ordered unique token list (JS Set semantics for f64-sum order).
pub(crate) fn uniq(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tokens {
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

pub(crate) fn concept_body(dir: &Path, rel: &str) -> Option<String> {
    let raw = match read_file_lossy(&join_rel(dir, rel)) {
        Ok(t) => t,
        Err(_) => return Some(String::new()),
    };
    match parse_frontmatter(&raw) {
        Fm::Parsed { body, .. } => Some(body),
        _ => Some(raw),
    }
}

pub(crate) fn meta_text_of(concept: &Concept) -> String {
    let t = match concept.data.get("title") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let d = match concept.data.get("description") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let tags = match concept.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| match v {
                Value::Null => String::new(),
                other => jsjson::js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    };
    format!("{t} {d} {tags}")
}

/// Number(score.toFixed(6)) — display-precision rounding (divergence note in
/// the header covers the tie-rounding difference).
pub(crate) fn to_fixed6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

pub(crate) fn score_critical_relevance(
    dir: &Path,
    criticals: &[&Concept],
    query_meta: &str,
    query_body: &str,
    query_tags: &HashSet<String>,
    query_areas: &HashSet<&str>,
) -> Option<Vec<(String, f64)>> {
    let stops = stopwords();
    let mut fields: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut df: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for concept in criticals {
        let meta = uniq(relevance_tokens(&meta_text_of(concept), &stops));
        let body = uniq(relevance_tokens(&concept_body(dir, &concept.path)?, &stops));
        let mut union_seen: HashSet<&String> = HashSet::new();
        for token in meta.iter().chain(body.iter()) {
            if union_seen.insert(token) {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }
        fields.push((meta, body));
    }
    let population = criticals.len();
    let idf = |token: &str| ((population as f64 + 1.0) / (*df.get(token).unwrap_or(&0) as f64 + 1.0)).ln() + 1.0;

    let query: HashSet<String> = relevance_tokens(&format!("{query_meta} {query_body}"), &stops)
        .into_iter()
        .collect();

    let coverage = |set: &[String]| -> f64 {
        let mut hit = 0.0f64;
        let mut total = 0.0f64;
        for token in set {
            let weight = idf(token);
            total += weight;
            if query.contains(token) {
                hit += weight;
            }
        }
        if total == 0.0 {
            0.0
        } else {
            hit / total
        }
    };

    let mut scores = Vec::new();
    for (i, concept) in criticals.iter().enumerate() {
        let bee = bee_of(&concept.data);
        let tags = match concept.data.get("tags") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if query_tags.contains(&s.to_lowercase())))
                .count(),
            _ => 0,
        };
        let areas = match bee.get("areas") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if query_areas.contains(s.as_str())))
                .count(),
            _ => 0,
        };
        let (meta, body) = &fields[i];
        let score = TAG_WEIGHT * tags as f64
            + AREA_WEIGHT * areas as f64
            + META_WEIGHT * coverage(meta)
            + BODY_WEIGHT * coverage(body);
        scores.push((concept.path.clone(), to_fixed6(score)));
    }
    Some(scores)
}

pub(crate) fn num(v: f64) -> Value {
    Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
}

pub(crate) enum ManifestOut {
    Built(Value),
    Thrown(String),
    NeedsNode,
}

pub(crate) fn build_context_manifest(dir: &Path, work: &str, budget: f64, budget_raw: &Value) -> ManifestOut {
    let work_id = js_trim(work);
    if work_id.is_empty() {
        return ManifestOut::Thrown("knowledge context: missing_work — --work <id> is required (D27).".to_string());
    }
    if !budget.is_finite() || budget < 0.0 {
        // Node's message JSON.stringify's the RAW flags.budget — the CLI's
        // string (quoted) or the lane-preset number, never the conversion.
        return ManifestOut::Thrown(format!(
            "knowledge context: bad_budget — --budget must be a non-negative token count, got {} (D27).",
            jsjson::stringify(budget_raw)
        ));
    }

    let Some(concepts) = collect_concepts(dir) else { return ManifestOut::NeedsNode };

    // D5 then D1/D6: a bee.work-item concept whose bee.id matches always
    // wins; otherwise docs/history/<work_id>/CONTEXT.md and/or plan.md,
    // whichever exist; otherwise no anchor at all — unknown_work, unchanged.
    let root = dir.parent().and_then(Path::parent).unwrap_or(dir);
    let Some(anchor) = resolve_anchor(&concepts, root, work_id) else {
        return ManifestOut::Thrown(format!(
            "knowledge context: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D27)."
        ));
    };
    let work_concept: Option<&Concept> = match &anchor {
        Anchor::WorkItem(c) => Some(*c),
        Anchor::History { .. } => None,
    };

    let mut ranked: Vec<(String, String)> = Vec::new(); // (rel, reason)
    let mut selected: HashSet<String> = HashSet::new();
    let by_path: std::collections::HashMap<&str, &Concept> =
        concepts.iter().map(|c| (c.path.as_str(), c)).collect();
    let select = |rel: &str, reason: String, ranked: &mut Vec<(String, String)>, selected: &mut HashSet<String>| {
        if selected.contains(rel) || !by_path.contains_key(rel) {
            return false;
        }
        selected.insert(rel.to_string());
        ranked.push((rel.to_string(), reason));
        true
    };

    // (1) the work item — a history anchor is not a bundle concept; it is
    // sized and ranked as entry rank 1 directly, below (D6/D8).
    if let Some(wc) = work_concept {
        select(&wc.path, "work item".to_string(), &mut ranked, &mut selected);
    }

    // (2) the plan sibling in the same work/<id>/ directory
    if let Some(wc) = work_concept {
        let work_dir = dir_of(&wc.path).to_string();
        let plan = concepts.iter().find(|c| {
            matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.plan") && dir_of(&c.path) == work_dir
        });
        if let Some(plan) = plan {
            select(&plan.path, format!("plan sibling in {work_dir}/"), &mut ranked, &mut selected);
        }
    }

    // (3) required_context, transitive, BFS depth order, cycles deduped silently
    let mut queue: std::collections::VecDeque<(String, usize)> =
        ranked.iter().map(|(rel, _)| (rel.clone(), 0usize)).collect();
    while let Some((node_rel, depth)) = queue.pop_front() {
        let targets = match by_path.get(node_rel.as_str()).map(|c| bee_of(&c.data)) {
            Some(bee) => match bee.get("required_context") {
                Some(Value::Array(items)) => items.clone(),
                _ => continue,
            },
            None => continue,
        };
        for target in &targets {
            let Value::String(target) = target else { continue };
            let rel = match normalize_bundle_target(dir, target) {
                Ok(Some(rel)) => rel,
                Ok(None) => continue,
                Err(()) => return ManifestOut::NeedsNode,
            };
            if !by_path.contains_key(rel.as_str()) || selected.contains(&rel) {
                continue;
            }
            select(&rel, format!("required_context depth {} via {node_rel}", depth + 1), &mut ranked, &mut selected);
            queue.push_back((rel, depth + 1));
        }
    }

    // (4) the critical concepts, ranked by relevance and cut (G5/G11)
    let criticals: Vec<&Concept> = concepts
        .iter()
        .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
        .collect();
    let (query_meta, query_body): (String, String) = match &anchor {
        Anchor::WorkItem(c) => (meta_text_of(c), concept_body(dir, &c.path).unwrap_or_default()),
        Anchor::History { meta, body, .. } => (meta.clone(), body.clone()),
    };
    let query_tags: HashSet<String> = match work_concept {
        Some(c) => match c.data.get("tags") {
            Some(Value::Array(items)) => {
                items.iter().filter_map(|v| v.as_str()).map(|s| s.to_lowercase()).collect()
            }
            _ => HashSet::new(),
        },
        None => HashSet::new(),
    };
    let query_bee = work_concept.map(|c| bee_of(&c.data)).unwrap_or_default();
    let query_areas: HashSet<&str> = match query_bee.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => HashSet::new(),
    };
    let Some(relevance) = score_critical_relevance(dir, &criticals, &query_meta, &query_body, &query_tags, &query_areas)
    else {
        return ManifestOut::NeedsNode;
    };
    let score_of = |path: &str| -> f64 {
        relevance.iter().find(|(p, _)| p == path).map(|(_, s)| *s).unwrap_or(0.0)
    };
    let zero_signal_count = criticals.iter().filter(|c| score_of(&c.path) == 0.0).count();
    // D7: a history anchor carries no tags/areas of its own, so more
    // criticals land at exactly 0.0 as an artifact of the anchor kind, not a
    // signal about the bundle — report it in the manifest (zero_signal_count,
    // below), never refuse, under that arm. The work-item arm keeps refusing.
    if work_concept.is_some()
        && criticals.len() >= ZERO_SIGNAL_MIN_POPULATION
        && (zero_signal_count as f64) > criticals.len() as f64 * ZERO_SIGNAL_MAX_RATIO
    {
        return ManifestOut::Thrown(format!(
            "knowledge context: zero_signal — {zero_signal_count} of {} bee.critical concepts score 0 against work item \"{work_id}\", above the pinned {} ratio. A ranking where most items tie at zero is a path sort wearing a relevance label — widen the work item's description/body, or fix the ranking, but do not ship this order (G11).",
            criticals.len(),
            jsjson::js_f64_to_string(ZERO_SIGNAL_MAX_RATIO)
        ));
    }
    let mut ranked_criticals: Vec<&&Concept> = criticals.iter().collect();
    ranked_criticals.sort_by(|a, b| {
        score_of(&b.path)
            .partial_cmp(&score_of(&a.path))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    let mut excluded: Vec<Value> = Vec::new();
    let mut floor_paths: Vec<String> = Vec::new();
    let mut kept = 0usize;
    for (index, concept) in ranked_criticals.iter().enumerate() {
        let rank = index + 1;
        let score = score_of(&concept.path);
        if selected.contains(&concept.path) {
            continue; // already in via required_context — never re-cut
        }
        if kept >= KEEP {
            let mut m = Map::new();
            m.insert("path".into(), Value::String(format!("docs/knowledge/{}", concept.path)));
            m.insert("score".into(), num(score));
            m.insert(
                "reason".into(),
                Value::String(format!(
                    "below the relevance cut — rank {rank} of {}, keep {KEEP} (G5)",
                    ranked_criticals.len()
                )),
            );
            excluded.push(Value::Object(m));
            continue;
        }
        let is_floor = kept < FLOOR;
        if is_floor {
            floor_paths.push(format!("docs/knowledge/{}", concept.path));
        }
        select(
            &concept.path,
            format!(
                "critical pattern (relevance {}, rank {rank} of {}{})",
                jsjson::js_f64_to_string(score),
                ranked_criticals.len(),
                if is_floor { ", floor" } else { "" }
            ),
            &mut ranked,
            &mut selected,
        );
        kept += 1;
    }

    // (5) decisions whose areas overlap the work item's areas — a history
    // anchor has no bee.areas of its own, so this stays empty under it.
    let decision_bee = work_concept.map(|wc| bee_of(&wc.data)).unwrap_or_default();
    let work_areas: Vec<&str> = match decision_bee.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    };
    for concept in &concepts {
        if !matches!(concept.data.get("type"), Some(Value::String(t)) if t == "bee.decision") {
            continue;
        }
        let areas = match bee_of(&concept.data).get("areas") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let overlap: Vec<String> = areas
            .iter()
            .filter(|a| matches!(a, Value::String(s) if work_areas.contains(&s.as_str())))
            .map(jsjson::js_to_string)
            .collect();
        if overlap.is_empty() {
            continue;
        }
        select(&concept.path, format!("decision for area {}", overlap.join(", ")), &mut ranked, &mut selected);
    }

    struct Sized {
        repo_rel: String,
        reason: String,
        bytes: u64,
        est: f64,
        floor: bool,
    }
    // A history anchor is not a bundle concept: `select()` never carries it
    // into `ranked` (by_path only holds docs/knowledge/ concepts), and
    // sizing off `join_rel(dir, ...)` would measure a docs/knowledge/ path
    // that does not exist. It is sized here directly, off its own real
    // paths/bytes, and prepended so it lands as entry rank 1 — the same slot
    // the work item's own bundle file occupies under that arm — so
    // `rank_one_cost` below reserves budget against the anchor, not the top
    // critical (D8 discovery).
    let mut sized: Vec<Sized> = Vec::new();
    if let Anchor::History { paths, bytes, .. } = &anchor {
        let repo_rel = paths.join(" + ");
        let est = (*bytes as f64 / 4.0).ceil();
        sized.push(Sized { repo_rel, reason: "history anchor".to_string(), bytes: *bytes, est, floor: false });
    }
    sized.extend(ranked.iter().map(|(rel, reason)| {
        let repo_rel = format!("docs/knowledge/{rel}");
        let bytes = std::fs::metadata(join_rel(dir, rel)).map(|m| m.len()).unwrap_or(0);
        let est = (bytes as f64 / 4.0).ceil();
        let floor = floor_paths.contains(&repo_rel);
        Sized { repo_rel, reason: reason.clone(), bytes, est, floor }
    }));

    let floor_cost: f64 = sized.iter().filter(|s| s.floor).map(|s| s.est).sum();
    let rank_one_cost = sized.first().map(|s| s.est).unwrap_or(0.0);
    let mut reserve = (budget - rank_one_cost).min(floor_cost).max(0.0);
    let mut available = budget - reserve;

    let mut entries: Vec<Value> = Vec::new();
    let mut truncated: Vec<String> = Vec::new();
    let mut total_est = 0.0f64;
    let mut cutting = false;
    for item in &sized {
        if item.floor {
            if item.est > reserve {
                truncated.push(item.repo_rel.clone());
                continue;
            }
            reserve -= item.est;
            total_est += item.est;
        } else {
            if cutting || item.est > available {
                cutting = true;
                truncated.push(item.repo_rel.clone());
                continue;
            }
            available -= item.est;
            total_est += item.est;
        }
        let mut m = Map::new();
        m.insert("path".into(), Value::String(item.repo_rel.clone()));
        m.insert("bytes".into(), Value::Number(Number::from(item.bytes)));
        m.insert("est_tokens".into(), num(item.est));
        m.insert("reason".into(), Value::String(item.reason.clone()));
        entries.push(Value::Object(m));
    }

    let decisions: Vec<Value> = match work_concept.map(|wc| bee_of(&wc.data)) {
        Some(bee) => match bee.get("decisions") {
            Some(Value::Array(items)) => items.iter().filter(|v| v.is_string()).cloned().collect(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    };

    // CONSERVATION (G11)
    let accounted: HashSet<String> = entries
        .iter()
        .filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string))
        .chain(truncated.iter().cloned())
        .chain(excluded.iter().filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string)))
        .collect();
    let lost: Vec<String> = criticals
        .iter()
        .map(|c| format!("docs/knowledge/{}", c.path))
        .filter(|repo_rel| !accounted.contains(repo_rel))
        .collect();
    if !lost.is_empty() {
        return ManifestOut::Thrown(format!(
            "knowledge context: conservation — {} bee.critical concept(s) were neither included, truncated nor excluded: {} (G11). This is a bug in the ranking, not a condition of the bundle.",
            lost.len(),
            lost.join(", ")
        ));
    }

    let mut manifest = Map::new();
    manifest.insert("work".into(), Value::String(work_id.to_string()));
    manifest.insert("anchor".into(), {
        let mut m = Map::new();
        m.insert("kind".into(), Value::String(anchor.kind().to_string()));
        m.insert("paths".into(), Value::Array(anchor.paths().into_iter().map(Value::String).collect()));
        Value::Object(m)
    });
    manifest.insert("decisions".into(), Value::Array(decisions));
    manifest.insert("budget".into(), num(budget));
    manifest.insert("estimator".into(), Value::String(CONTEXT_ESTIMATOR.to_string()));
    manifest.insert("total_est".into(), num(total_est));
    manifest.insert("entries".into(), Value::Array(entries));
    manifest.insert("truncated".into(), Value::Array(truncated.into_iter().map(Value::String).collect()));
    manifest.insert("excluded".into(), Value::Array(excluded));
    manifest.insert("floor".into(), Value::Array(floor_paths.into_iter().map(Value::String).collect()));
    manifest.insert("critical_total".into(), Value::Number(Number::from(criticals.len())));
    manifest.insert("zero_signal_count".into(), Value::Number(Number::from(zero_signal_count)));
    ManifestOut::Built(Value::Object(manifest))
}
