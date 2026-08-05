// inject.mjs proper and the three shared renderers
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;

// ─── inject.mjs proper ─────────────────────────────────────────────────────

/// `state.adoptHandoff`'s typed result, as the SessionStart hook computes it
/// and hands it to the renderer. The renderer NEVER mutates: adoption is the
/// hook's job (inject.mjs's PURITY PIN, fsh-10 panel W2).
#[derive(Debug, Clone, Default)]
pub struct HandoffOutcome {
    pub ok: bool,
    pub code: Option<String>,
    pub reason: Option<String>,
    pub next_cell: Option<String>,
}

/// inject.mjs knowledgeContextBudgetForMode (i54-closeout D3): the ONE preset
/// table `--lane` resolves against; an unset/unrecognized mode falls back to
/// the bare default.
pub(crate) fn knowledge_context_budget_for_mode(mode: Option<&Value>) -> i64 {
    let Some(Value::String(m)) = mode else { return KNOWLEDGE_CONTEXT_DEFAULT_BUDGET };
    KNOWLEDGE_CONTEXT_LANE_BUDGETS
        .iter()
        .find(|(k, _)| k == m)
        .map(|(_, v)| *v)
        .unwrap_or(KNOWLEDGE_CONTEXT_DEFAULT_BUDGET)
}

pub(crate) fn is_no_work_phase(record: &JMap) -> bool {
    let phase = record.get("phase");
    NO_WORK_PHASES.iter().any(|p| str_eq(phase, p))
}

/// inject.mjs visibleGates — the review gate (Gate 3) is user-invoked, so it
/// is pending only inside a live review session, and a terminal record owes no
/// gate at all.
pub(crate) fn visible_gates(record: &JMap) -> Vec<&'static str> {
    if is_no_work_phase(record) {
        return Vec::new();
    }
    if str_eq(record.get("phase"), "reviewing") {
        GATE_NAMES.to_vec()
    } else {
        GATE_NAMES.iter().copied().filter(|g| *g != "review").collect()
    }
}

/// The first gate this record still owes, or `None`. Shared with the compact
/// capsule (compaction-hardening D6 item 8) so a compacted session is told
/// about exactly the gates a live session would be told about.
pub fn first_open_gate(record: &JMap) -> Option<&'static str> {
    let gates = record.get("approved_gates");
    visible_gates(record)
        .into_iter()
        .find(|gate| !matches!(gates.and_then(|v| vget(v, gate)), Some(Value::Bool(true))))
}

pub(crate) fn gates_line(record: &JMap) -> String {
    let shown = visible_gates(record);
    if shown.is_empty() {
        return "none pending (no active work)".to_string();
    }
    let gates = record.get("approved_gates");
    shown
        .iter()
        .map(|gate| {
            let approved = matches!(gates.and_then(|v| vget(v, gate)), Some(Value::Bool(true)));
            format!("{gate}: {}", if approved { "approved" } else { "pending" })
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

// ─── the three shared renderers (compaction-hardening cz-5, D6/D26) ────────
//
// build_session_preamble below and build_compact_capsule (hooks/compaction.rs)
// are two callers of ONE truth for each of these three blocks, never two
// copies of it: D6 items 3, 4 and 5 require the capsule to carry these EXACT
// bytes, and a second hand-written copy is the classic way "verbatim" quietly
// stops being verbatim one edit later.
//
// BLANK-LINE OWNERSHIP IS THE CALLER'S — decided deliberately (cz-5 STEP 1).
// The preamble opens its HANDOFF block with a bare `lines.push("")`. That
// blank is SPACING BETWEEN SECTIONS, not part of the block: the capsule
// composes its own sections with its own joiner, and a helper that carried a
// leading blank would force the capsule to inherit the preamble's spacing
// assumptions (and would make the block's first byte depend on what happens
// to precede it). `handoff_block_lines` therefore returns the block's OWN
// lines only; each caller emits its own separator.

/// inject.mjs's onboarding line, all three arms (missing / drifted / ok).
pub fn onboarding_line(onboarding: Option<&Value>) -> String {
    let Some(onboarding) = onboarding.filter(|v| truthy(v)) else {
        return "- Onboarding: MISSING — run bee-hive onboarding before anything else.".to_string();
    };
    let version = vget(onboarding, "bee_version");
    if opt_truthy(version) && !str_eq(version, BEE_VERSION) {
        return format!(
            "- Onboarding: installed at bee {} but plugin is {BEE_VERSION} — re-run onboarding to refresh vendored helpers.",
            tpl(version)
        );
    }
    let shown = if opt_truthy(version) { tpl(version) } else { BEE_VERSION.to_string() };
    format!("- Onboarding: ok (bee {shown})")
}

/// The loud gate-bypass banner: `[]` when off, 1 line for normal, 2 for
/// full/total.
pub fn bypass_banner_lines(level: &str) -> Vec<String> {
    if level.is_empty() || level == "off" {
        return Vec::new();
    }
    let mut lines = vec![format!("- {}", bypass_banner(level))];
    if level == "full" || level == "total" {
        let tail = if level == "total" {
            "This includes secret-file reads and review P1 findings: nothing pauses for the human."
        } else {
            "Only reading a secret-shaped file and a review P1 finding still pause for the human."
        };
        lines.push(format!(
            "  The agent does NOT stop for these gates — it records the recommended choice, logs a one-line audit decision, and continues. {tail}"
        ));
    }
    lines
}

/// The wait-and-never-auto-resume HANDOFF block, WITHOUT its leading blank.
///
/// `outcome` is a real parameter, not a formality (D26): the SessionStart
/// hook sets `{ok:false, code:"WRONG_SOURCE"}` whenever a planned-next
/// handoff exists on a non-adopting source — `compact` included — and the
/// refusal REASON is the only thing telling the session why it is waiting
/// instead of starting. Dropping the parameter renders a block that looks
/// verbatim and is not.
pub fn handoff_block_lines(handoff: &Value, outcome: Option<&HandoffOutcome>) -> Vec<String> {
    if !truthy(handoff) {
        return Vec::new();
    }
    let mut lines =
        vec!["### HANDOFF present — present it and WAIT — never auto-resume".to_string()];
    lines.push(format!(
        "- Phase: {} | Feature: {} | Mode: {}",
        tpl_or(vget(handoff, "phase"), "unknown"),
        tpl_or(vget(handoff, "feature"), "unknown"),
        tpl_or(vget(handoff, "mode"), "unknown"),
    ));
    if let Some(Value::Array(cells)) = vget(handoff, "cells_in_flight") {
        if !cells.is_empty() {
            lines.push(format!("- Cells in flight: {}", js_join(cells, ", ")));
        }
    }
    if opt_truthy(vget(handoff, "next_action")) {
        // Clamped: a saved next action is free text and has run to 900+ bytes
        // in practice. The head carries the instruction; `.bee/HANDOFF.json`
        // carries the rest, and the block above already says to present this
        // and wait, so nothing is decided on the truncated half.
        lines.push(format!(
            "- Saved next action: {}",
            clamp_chars(&tpl(vget(handoff, "next_action")), HANDOFF_ACTION_CHARS)
        ));
    }
    if str_eq(vget(handoff, "kind"), "planned-next") {
        if let Some(outcome) = outcome.filter(|o| !o.ok) {
            let reason = outcome
                .reason
                .clone()
                .or_else(|| outcome.code.clone())
                .unwrap_or_else(|| "unknown reason".to_string());
            lines.push(format!("- Adoption not applied: {reason}"));
        }
    }
    lines
}

/// A work item's one canonical location — the same path the retired advice
/// line below named — read as ONE candidate file, never a bundle walk: the
/// cheaper reading `rank_critical_rows` (budget.rs) already takes for the
/// same reason (kl-4 deliberately avoided the 1.24 MB collect_concepts cost
/// just to answer a yes/no). A missing or unparsed candidate reads as an
/// empty-data row — resolve_anchor's WorkItem arm simply will not match it,
/// the same "unreadable file, keep the row" direction collect_concepts
/// itself takes.
fn work_item_candidate(root: &Path, feature: &str) -> Vec<Concept> {
    let rel = format!("work/{feature}/work-item.md");
    let path = join_rel(&bundle_dir(root), &rel);
    let data = read_file_lossy(&path).and_then(|text| parse_frontmatter(&text)).unwrap_or_default();
    vec![Concept { path: rel, data }]
}

impl crate::verbs::knowledge::ConceptLike for Concept {
    fn concept_path(&self) -> &str {
        &self.path
    }
    fn concept_data(&self) -> &Map<String, Value> {
        &self.data
    }
}

/// inject.mjs knowledgeContextLines (okf-8, D38), replaced under D1
/// (kf-1): the gate is now `resolve_anchor` (verbs/knowledge/anchor.rs) —
/// the SAME arbiter `bee knowledge context` itself answers to — instead of
/// a hand-rolled "does a bee.work-item concept exist" check that silently
/// excluded the 162 of 164 anchorable features whose only anchor is
/// docs/history or a scribing stamp. Silence beats a nag: no anchor at all
/// emits nothing — never advice to author a work-item file (D5 made that
/// file optional; the old line now contradicted the shipped design and is
/// deleted outright, not replaced).
pub(crate) fn knowledge_context_lines(root: &Path, record: &JMap) -> Vec<String> {
    let feature = match record.get("feature") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if feature.is_empty() || is_no_work_phase(record) {
        return Vec::new();
    }
    let candidates = work_item_candidate(root, &feature);
    let Some(anchor) = crate::verbs::knowledge::resolve_anchor(&candidates, root, &feature) else {
        return Vec::new();
    };
    let budget = knowledge_context_budget_for_mode(record.get("mode"));
    vec![
        "### Knowledge context — load it before code".to_string(),
        // CUTOVER wording divergence: inject.mjs:260 spelled this
        // `node .bee/bin/bee.mjs knowledge context …`. The anchor kind is
        // named so the reader knows what the manifest was ranked against.
        format!(
            "- `.bee/bin/bee knowledge context --work {feature} --budget {budget}` (anchor: {})",
            anchor.kind()
        ),
        "- Run it and read the manifest's files before touching code — that manifest is this feature's curated context, and it replaces scanning docs/history.".to_string(),
    ]
}

/// inject.mjs projectMapLines (D5/D10 + okf-integration-close-f4 D2).
pub(crate) fn project_map_lines(root: &Path, bundle: bool) -> Vec<String> {
    let mut lines = vec!["### Project map".to_string()];
    let body = if bundle { bundle_project_map_lines(root) } else { spec_project_map_lines(root) };
    lines.extend(body);
    if let Some(backlog) = read_backlog_counts(root) {
        lines.push(format!(
            "- PBI: {} done / {} in-flight / {} proposed",
            tpl(backlog.get("done")),
            tpl(backlog.get("inFlight")),
            tpl(backlog.get("proposed")),
        ));
    }
    lines
}

pub(crate) fn spec_project_map_lines(root: &Path) -> Vec<String> {
    let specs_dir = resolve_product_root(root).join("docs").join("specs");
    let present: Vec<(&str, &str)> = PROJECT_MAP_FILES
        .iter()
        .copied()
        .filter(|(file, _)| specs_dir.join(file).exists())
        .collect();
    if present.is_empty() {
        return vec![
            "- Project map missing (Q1/Q2 unanswerable from repo) — bee-capturing bootstrap available.".to_string(),
        ];
    }
    let mut lines: Vec<String> = present
        .iter()
        .map(|(file, label)| format!("- {label}: docs/specs/{file}"))
        .collect();
    let area_count = std::fs::read_dir(&specs_dir)
        .map(|rd| {
            rd.flatten()
                .filter(|entry| {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                        && name.ends_with(".md")
                        && !PROJECT_MAP_FILES.iter().any(|(file, _)| *file == name)
                })
                .count()
        })
        .unwrap_or(0);
    lines.push(format!(
        "- Specced areas: {area_count} (docs/specs/ — read the spec before the code)"
    ));
    lines
}

pub(crate) fn bundle_project_map_lines(root: &Path) -> Vec<String> {
    let mut lines = vec![
        "- Knowledge bundle: docs/knowledge/ (index: docs/knowledge/index.md) — read the bundle before the code".to_string(),
    ];
    let concepts = collect_concepts(root);
    let mut areas: Vec<String> = Vec::new();
    for concept in &concepts {
        if let Some(rest) = concept.path.strip_prefix("areas/") {
            if let Some(idx) = rest.find('/') {
                let slug = &rest[..idx];
                if !slug.is_empty() && !areas.iter().any(|a| a == slug) {
                    areas.push(slug.to_string());
                }
            }
        }
    }
    lines.push(format!(
        "- Bundle holds: {} area(s), {} concept(s) (docs/specs/ is the read-only compatibility surface)",
        areas.len(),
        concepts.len()
    ));
    lines
}
