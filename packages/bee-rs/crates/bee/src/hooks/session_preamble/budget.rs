// the preamble budget and the block it assembles
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, doc_viewer_prefix, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;
// D3: the digest ranks by relevance using kl-1's shared anchor resolver and
// the same IDF ranker context.rs uses — aliased so they never collide with
// this module's own re-derived Concept/parse_frontmatter/read_file_lossy
// port above (a deliberately separate, cheaper reading path — see the module
// banner's "may not edit that file" note).
use crate::verbs::knowledge::{
    parse_frontmatter as verbs_parse_frontmatter, resolve_anchor as verbs_resolve_anchor,
    score_critical_relevance, Anchor as VerbsAnchor, Concept as VerbsConcept, Fm as VerbsFm,
};
// D2 (kf-1): the SAME lane-first resolution `state gate`/`state set`/
// `state route` already read through (explicit id, then the
// BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID env chain, then single-live-session
// adoption) — reused here rather than re-derived, because the SessionStart
// hook's own `session_id` frequently does not (yet) name a session record
// carrying a `lane` field even when the calling process IS lane-bound by
// this identical chain.
use crate::verbs::state_group::session_binding;

/// inject.mjs criticalPatternsDigest — routes on the ONE bundle predicate
/// (G12), same line cap in both branches.
// ── preamble budget ────────────────────────────────────────────────────────
//
// The preamble is injected into EVERY session, so its bytes are the most
// expensive in the harness — and three of its sections grew with the store
// rather than with what the reader can act on. Measured on this repo before
// these caps: 11,390 bytes, of which 3,169 (28%) was a single line listing
// 176 uncaptured cell ids grouped by feature. No reader acts on 176 ids; the
// COUNT is the signal, and the fix for it is one command.
//
// The rule these constants enforce: a section may carry a number that grows,
// never a list that grows. Everything past the cap is named as a count and
// left where it can be fetched. `the_preamble_stays_inside_its_budget` pins
// the whole block against a deliberately pathological store.
/// Every section switched on at once, over a pathological store, must fit
/// here. Measured: 11,390 bytes before the caps on this repo, 4,841 after.
///
/// NOT the law decision `8f63adb4` (budget-fence-removal D1) abolished. That
/// one forbids a standing size ceiling on bee's AUTHORED INSTRUCTION TEXT —
/// skill bodies, AGENTS.md — because a number there makes people trim for
/// smallness instead of writing for information. This caps a GENERATED
/// payload, the same family as the `status --brief --json` payload cap and
/// `knowledge context --budget`, which that feature's own boundary listed as
/// out of scope and explicitly kept. Nobody edits prose to satisfy it; it
/// bounds how much of the STORE the renderer is allowed to paste.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const PREAMBLE_BUDGET_BYTES: usize = 5120;

pub(crate) const ORPHAN_FEATURES_SHOWN: usize = 3;

pub(crate) const DEBT_CELLS_SHOWN: usize = 8;

/// One header line + this many patterns.
pub(crate) const PATTERN_DIGEST_LINES: usize = 4;

pub(crate) const DECISION_CHARS: usize = 160;

pub(crate) const HANDOFF_ACTION_CHARS: usize = 400;

/// Truncate on a CHARACTER boundary (never mid-UTF-8) and say so, so a reader
/// never mistakes a cut for the end of the text.
pub(crate) fn clamp_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max).collect();
    format!("{} […]", head.trim_end())
}

/// A bundle index row is `- [Title](path) — gloss`, and for most patterns the
/// gloss restates the title verbatim. The title IS the pattern statement, so
/// the gloss is dropped; the linked file holds the full text.
pub(crate) fn strip_row_gloss(row: &str) -> String {
    let after_link = row.find("](").and_then(|i| row[i..].find(')').map(|j| i + j)).unwrap_or(0);
    match row[after_link..].find(" — ") {
        Some(rel) => row[..after_link + rel].to_string(),
        None => row.to_string(),
    }
}

pub(crate) fn critical_patterns_digest(
    root: &Path,
    max_lines: usize,
    bundle: bool,
    feature: Option<&str>,
) -> Option<Vec<String>> {
    if bundle {
        bundle_critical_patterns_digest(root, max_lines, feature)
    } else {
        legacy_critical_patterns_digest(root, max_lines)
    }
}

pub(crate) fn legacy_critical_patterns_digest(root: &Path, max_lines: usize) -> Option<Vec<String>> {
    let file = root
        .join("docs")
        .join("history")
        .join("learnings")
        .join("critical-patterns.md");
    let text = read_file_lossy(&file)?;
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| js_trim(l.strip_suffix('\r').unwrap_or(l)).to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .collect();
    if lines.is_empty() {
        return None;
    }
    Some(lines.into_iter().take(max_lines).collect())
}

/// `.replace(/\]\((?!https?:|\/)/g, '](docs/knowledge/')` — index links are
/// bundle-relative, and the preamble is read from the repo root.
pub(crate) fn rewrite_bundle_links(row: &str) -> String {
    let b = row.as_bytes();
    let mut out = String::with_capacity(row.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b']' && i + 1 < b.len() && b[i + 1] == b'(' {
            let rest = &row[i + 2..];
            let excluded = rest.starts_with('/')
                || rest.starts_with("http:")
                || rest.starts_with("https:");
            out.push_str(if excluded { "](" } else { "](docs/knowledge/" });
            i += 2;
            continue;
        }
        // Push one whole char (indices only advance on char boundaries above).
        let ch = row[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// A bundle index row is `- [Title](target) — gloss`; the target is the raw,
/// bundle-relative link before `rewrite_bundle_links` prefixes it for
/// display — the shape the relevance ranker needs to resolve a row to an
/// on-disk concept file.
pub(crate) fn link_target(row: &str) -> Option<&str> {
    let start = row.find("](")? + 2;
    let end = row[start..].find(')')? + start;
    Some(&row[start..end])
}

/// D3 (decision), phase 4: rank `rows` (raw, unrewritten) by relevance to
/// `work`'s anchor and return the top `keep`, display-formatted, plus how
/// many rows were dropped (target file missing) and how many concept files
/// were actually opened to score them — the cost evidence a caller/test can
/// pin against the whole-bundle count `collect_concepts` would cost.
///
/// `Err(reason)` on any of: no anchor for `work`, or no row resolved to a
/// file — the caller renders today's recency pick instead and names the
/// reason in the header (must always say which mode produced the rows).
pub(crate) fn rank_critical_rows(
    root: &Path,
    rows: &[String],
    work: &str,
    keep: usize,
) -> Result<(Vec<String>, usize, usize), String> {
    let dir = bundle_dir(root);

    // Cost discipline: an empty concepts slice means resolve_anchor can only
    // ever land on the History arm (docs/history/<work>/CONTEXT.md and/or
    // plan.md) or None — never the WorkItem arm, which would need
    // collect_concepts to find. That is the deliberate cheaper reading this
    // digest takes (see the module's "may not edit that file" banner).
    let empty: &[VerbsConcept] = &[];
    let anchor = verbs_resolve_anchor(empty, root, work).ok_or_else(|| format!("no anchor for \"{work}\""))?;
    let VerbsAnchor::History { meta, body, .. } = &anchor else {
        return Err(format!("no anchor for \"{work}\"")); // unreachable with an empty concepts slice
    };

    // Parse every row's target into a candidate path, dropping rows whose
    // file is missing on disk (counted, never silent) and skipping external
    // links (never a bundle concept, so never "dropped").
    let mut candidates: Vec<(String, String)> = Vec::new(); // (raw row, bundle-relative path)
    let mut dropped = 0usize;
    for row in rows {
        let Some(target) = link_target(row) else { continue };
        if target.starts_with('/') || target.starts_with("http:") || target.starts_with("https:") {
            continue;
        }
        if join_rel(&dir, target).is_file() {
            candidates.push((row.clone(), target.to_string()));
        } else {
            dropped += 1;
        }
    }
    if candidates.is_empty() {
        return Err(format!("no critical row resolved to a file for \"{work}\""));
    }

    // Read ONLY the candidate concepts' bodies — never collect_concepts,
    // which would parse every concept in the bundle for data this digest
    // does not use.
    let mut concepts: Vec<VerbsConcept> = Vec::with_capacity(candidates.len());
    for (_, rel) in &candidates {
        let raw = read_file_lossy(&join_rel(&dir, rel)).unwrap_or_default();
        let data = match verbs_parse_frontmatter(&raw) {
            VerbsFm::Parsed { data, .. } => data,
            _ => Map::new(),
        };
        concepts.push(VerbsConcept { path: rel.clone(), data });
    }
    let opened = concepts.len();
    let criticals: Vec<&VerbsConcept> = concepts.iter().collect();
    // A history anchor carries no tags/areas of its own (context.rs D7) — the
    // same IDF ranker, the same empty query sets it uses under that arm.
    let query_tags: HashSet<String> = HashSet::new();
    let query_areas: HashSet<&str> = HashSet::new();
    let scores = score_critical_relevance(&dir, &criticals, meta, body, &query_tags, &query_areas)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("the ranker had nothing to score for \"{work}\""))?;

    let score_of = |path: &str| scores.iter().find(|(p, _)| p == path).map(|(_, s)| *s).unwrap_or(0.0);
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| {
        score_of(&candidates[b].1)
            .partial_cmp(&score_of(&candidates[a].1))
            .unwrap_or(Ordering::Equal)
            .then_with(|| candidates[a].1.cmp(&candidates[b].1))
    });
    let top: Vec<String> = order
        .into_iter()
        .take(keep)
        .map(|i| strip_row_gloss(&rewrite_bundle_links(&candidates[i].0)))
        .collect();
    Ok((top, dropped, opened))
}

fn recency_digest(rows: &[String], keep: usize, reason: &str) -> Vec<String> {
    let mut recent: Vec<String> = rows[rows.len().saturating_sub(keep)..]
        .iter()
        .map(|r| strip_row_gloss(&rewrite_bundle_links(r)))
        .collect();
    recent.reverse();
    let mut out = vec![format!(
        "- {} critical pattern(s) in the bundle — recency fallback ({reason}), the {} most recent below; full list: docs/knowledge/index.md (\"Critical patterns\").",
        rows.len(),
        recent.len()
    )];
    out.extend(recent);
    out
}

pub(crate) fn bundle_critical_patterns_digest(
    root: &Path,
    max_lines: usize,
    feature: Option<&str>,
) -> Option<Vec<String>> {
    let text = read_file_lossy(&bundle_dir(root).join("index.md"))?;
    let all: Vec<String> = text
        .split('\n')
        .map(|l| js_trim(l.strip_suffix('\r').unwrap_or(l)).to_string())
        .collect();
    let start = all.iter().position(|l| l == CRITICAL_PATTERNS_HEADING)?;
    // Raw (unrewritten) rows — the relevance ranker needs the un-prefixed
    // link target; display rewriting happens once a row is actually chosen.
    let mut rows: Vec<String> = Vec::new();
    for line in all.iter().skip(start + 1) {
        if line.starts_with("## ") {
            break;
        }
        if line.starts_with("- ") {
            rows.push(line.clone());
        }
    }
    if rows.is_empty() {
        return None;
    }
    let keep = std::cmp::max(1, max_lines.saturating_sub(1));

    // D3: rank by relevance to the bound feature's anchor when one is bound,
    // resolves, and the ranker has something to score; otherwise the recency
    // pick, with the header always naming which mode produced the rows.
    let work = feature.map(js_trim).filter(|f| !f.is_empty());
    let Some(work) = work else {
        return Some(recency_digest(&rows, keep, "no feature bound"));
    };
    match rank_critical_rows(root, &rows, work, keep) {
        Ok((top, dropped, _opened)) => {
            let mut out = vec![format!(
                "- {} critical pattern(s) in the bundle — ranked by relevance to \"{work}\" ({dropped} row(s) dropped: target file missing), the {} most relevant below; full list: docs/knowledge/index.md (\"Critical patterns\").",
                rows.len(),
                top.len()
            )];
            out.extend(top);
            Some(out)
        }
        Err(reason) => Some(recency_digest(&rows, keep, &reason)),
    }
}

/// D2 (kf-1): the feature `knowledge_context_lines` reads and the feature
/// the critical-pattern digest ranks against must be the session's ACTIVE
/// feature — the bound lane when the session has one, the default record
/// otherwise — never the default record unconditionally. `pipeline_record`
/// above already answers this correctly whenever the hook's own
/// `session_id` names a session record that itself carries a `lane` field;
/// this only ADDS the fallback for the frequent case where it does not
/// (yet) at hook-fire time, by falling through to the same lane-first
/// resolution `state gate`/`state set`/`state route` already read through
/// (`session_binding`) rather than re-deriving a second notion of "active".
/// Measured: a live preamble ranked its digest against a feature closed
/// hours earlier, while three features ran on lane records that same day.
fn active_record(
    root: &Path,
    session_id: Option<&str>,
    pipeline: &Pipeline,
    default_record: &JMap,
) -> JMap {
    if pipeline.ok && pipeline.source == "lane" {
        return pipeline.record.clone();
    }
    let control_root = control_root_for(root);
    let (_sid, bound) = session_binding(session_id, &control_root).unwrap_or((None, None));
    let Some(feature) = bound else { return default_record.clone() };
    let feature_val = json!(feature);
    read_lane(&control_root, Some(&feature_val)).unwrap_or_else(|| default_record.clone())
}

/// inject.mjs `buildSessionPreamble(root, { sessionId, handoffOutcome })`.
/// Pure: reads state, never writes. Fail-open everywhere — orientation is
/// never a place to fail a session.
pub fn build_session_preamble(
    root: &Path,
    session_id: Option<&str>,
    handoff_outcome: Option<&HandoffOutcome>,
) -> String {
    let state = read_state(root);
    let onboarding = read_onboarding(root);
    let handoff = read_handoff(root);
    let pipeline = resolve_pipeline(root, session_id);
    let pipeline_record = if pipeline.ok { pipeline.record.clone() } else { state.clone() };
    // D2 (kf-1): the ACTIVE feature — never the default record
    // unconditionally — that the knowledge bridge and the digest below both
    // read.
    let active = active_record(root, session_id, &pipeline, &pipeline_record);
    // okf-integration-close-f4 D1/D2/D3: the ONE predicate, resolved once and
    // handed to every section that branches on it (G12). Fail-safe direction
    // is the legacy branch — orientation never fails a session.
    let bundle = bundle_mode(root);
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("## bee v{BEE_VERSION}"));
    lines.push(onboarding_line(onboarding.as_ref()));
    lines.push(format!(
        "- Phase: {} | Mode: {} | Feature: {}",
        tpl(pipeline_record.get("phase")),
        tpl_or(pipeline_record.get("mode"), "none"),
        tpl_or(pipeline_record.get("feature"), "none"),
    ));
    lines.push(format!("- Gates: {}", gates_line(&pipeline_record)));
    if pipeline.ok && pipeline.source == "lane" {
        let bound = pipeline.feature.clone().map(Value::String);
        let others: Vec<JMap> = list_lanes(root)
            .into_iter()
            .filter(|lane| {
                !strict_eq(lane.get("feature"), bound.as_ref())
                    && !str_eq(lane.get("phase"), "idle")
                    && !str_eq(lane.get("phase"), "compounding-complete")
            })
            .collect();
        if !others.is_empty() {
            let names: Vec<Value> =
                others.iter().map(|l| l.get("feature").cloned().unwrap_or(Value::Null)).collect();
            lines.push(format!(
                "- {} other active lane(s): {}",
                others.len(),
                js_join(&names, ", ")
            ));
        }
    }
    let config = read_config_raw_open(root);
    for line in bypass_banner_lines(bypass_level(&config)) {
        lines.push(line);
    }
    // spec #81 P1 (sv-1): zero preamble cost when off — only 'draft-pr' adds
    // a line, matching bypassBannerLines' own "omit entirely when nothing to
    // report" convention just above.
    if ship_visibility(&config) == "draft-pr" {
        lines.push(
            "- Ship visibility: draft-pr — first cap opens a draft PR, every cap pushes (routing-and-contracts \"Ship visibility\")".to_string(),
        );
    }
    // explicit-triage D2: zero preamble cost when absent — only a recorded
    // route (bee state route --set) adds a line.
    if let Some(route) = pipeline_record.get("route").filter(|r| truthy(r)) {
        let flags = vget(route, "flags");
        let (flag_count, flag_list) = match flags {
            Some(Value::Array(items)) => (items.len().to_string(), js_join(items, ",")),
            // Node would throw on `.join` here; totality wins (see the module
            // banner's divergence list).
            _ => ("undefined".to_string(), String::new()),
        };
        lines.push(format!(
            "- Route: class={} | lane={} | flags={flag_count} [{flag_list}] | files={}",
            tpl(vget(route, "class")),
            tpl(vget(route, "lane")),
            tpl(vget(route, "product_files")),
        ));
    }
    if handoff_outcome.map(|o| o.ok).unwrap_or(false) {
        // fsh-10 (D1): adoption succeeded — start-now, no confirmation needed.
        // adoptHandoff already cleared .bee/HANDOFF.json, so `handoff` above
        // is already None by the time this renders — handoff_outcome is the
        // only surviving record of what happened.
        let outcome = handoff_outcome.unwrap();
        let next_cell_id = outcome.next_cell.clone().unwrap_or_else(|| "unknown".to_string());
        let next_cell = read_json_open(
            &root.join(".bee").join("cells").join(format!("{next_cell_id}.json")),
        );
        let title = next_cell.as_ref().and_then(|c| vget(c, "title")).filter(|v| truthy(v));
        lines.push(String::new());
        lines.push(
            "### PLANNED-NEXT ADOPTED — starting now, no confirmation needed (D1)".to_string(),
        );
        lines.push(match title {
            Some(t) => format!("- Cell: {next_cell_id} — {}", tpl(Some(t))),
            None => format!("- Cell: {next_cell_id}"),
        });
        lines.push(format!(
            "- Lane: {}",
            tpl_or(next_cell.as_ref().and_then(|c| vget(c, "lane")), "unknown")
        ));
        if let Some(verify) = next_cell.as_ref().and_then(|c| vget(c, "verify")).filter(|v| truthy(v))
        {
            lines.push(format!("- Verify: `{}`", tpl(Some(verify))));
        }
    } else if let Some(handoff) = handoff.as_ref() {
        // The leading blank is the CALLER's separator, never the block's own
        // first byte (see the blank-line ownership note above).
        lines.push(String::new());
        lines.extend(handoff_block_lines(handoff, handoff_outcome));
    }

    let commands = normalize_commands(config.get("commands"));
    let recorded_keys: Vec<&str> = COMMAND_KEYS
        .iter()
        .copied()
        .filter(|key| opt_truthy(commands.get(*key)))
        .collect();
    if !recorded_keys.is_empty() {
        lines.push(String::new());
        lines.push("### Standard commands (host project)".to_string());
        for key in &recorded_keys {
            lines.push(format!("- {key}: `{}`", tpl(commands.get(*key))));
        }
        if str_eq(commands.get("test"), NO_TEST_SENTINEL) {
            // no-test-repos D1 (decision 55b951e1): the sentinel REPLACES the
            // test-gate paragraph outright with one loud line — never a
            // silent drop of the gate.
            lines.push(format!(
                "- Test gates disabled by repo declaration (commands.test: {NO_TEST_SENTINEL}) — cells cap on diff-backed outcomes; re-enable by recording a real commands.test."
            ));
        } else if opt_truthy(commands.get("test")) {
            lines.push(
                // REWRITTEN TWICE. It first read "before your first `cells
                // claim`, check CI INSTEAD of running anything locally",
                // wrong in both directions: CI ran nightly only, so its
                // answer could predate the change by a day, while the
                // declared command it stood in for finishes in seconds. It
                // then keyed on `commands.verify`, which has since been
                // retired — one declared test command now serves the dev
                // loop, the cap door, feature close, and the merge gate, and
                // it IS what CI runs on every push and PR.
                "- Never build on red: run the test command above before your first `cells claim`, and treat a red as its own fix-first cell. CI runs the same command on every push and PR.".to_string(),
            );
        }
    }

    // doc-viewer-links (decision 4205835b): rendered only when the key
    // resolves — an unset doc_viewer leaves the preamble byte-identical to
    // today. Placed right after Standard commands, never appended at the
    // end: the closing trailer's exact bytes are pinned by
    // session_preamble/tests.rs (`ends_with`).
    if let Some(prefix) = doc_viewer_prefix(&config) {
        lines.push(String::new());
        lines.push("### Doc links".to_string());
        lines.push(format!(
            "- Doc viewer: {prefix} — when you point the user at a doc, give this URL with the repo-relative path appended (e.g. {prefix}/docs/history/<feature>/plan.md), never the bare path."
        ));
    }

    // okf-8 (D38): the startup bridge sits ahead of the project map. D2:
    // reads the session's active feature, not pipeline_record unconditionally.
    let knowledge = knowledge_context_lines(root, &active);
    if !knowledge.is_empty() {
        lines.push(String::new());
        lines.extend(knowledge);
    }

    lines.push(String::new());
    lines.extend(project_map_lines(root, bundle));

    // D11: capture-mode spine. okf-integration-close-f4 D3: the nudge names
    // the RESOLVED target rather than hardcoding docs/specs/.
    let (debt_count, debt_cells) = scribing_debt(root);
    if debt_count > 0 {
        lines.push(String::new());
        lines.push(format!(
            "### Scribing debt: {debt_count} behavior_change cell(s) uncaptured"
        ));
        let shown = std::cmp::min(DEBT_CELLS_SHOWN, debt_cells.len());
        let more = debt_cells.len() - shown;
        let ids = js_join(&debt_cells[..shown], ", ");
        let tail = if more > 0 { format!(" +{more} more") } else { String::new() };
        lines.push(format!(
            "- {ids}{tail} capped since the last scribing run — capture pending (decision c8e25271): run bee-capturing when you choose; settled behavior belongs in {}.",
            if bundle { "docs/knowledge/" } else { "docs/specs/" }
        ));
    }

    // scribing-integrity si-1: the orphan sweep, one loud line.
    let (orphan_count, orphan_features) = global_scribing_debt(root);
    if orphan_count > 0 {
        lines.push(String::new());
        lines.push(format!(
            "### Orphaned scribing debt: {orphan_count} cell(s) across {} feature(s)",
            orphan_features.len()
        ));
        // The cell ids used to be spelled out in full — 176 of them, 3,169
        // bytes, 28% of the whole preamble, re-injected every session. Nobody
        // acts on that list; they act on the count, and then on ONE feature.
        // So: the heaviest few features by name, the rest as a number, and
        // the command that prints the whole thing when it is actually wanted.
        let mut heaviest: Vec<&(String, Vec<Value>)> = orphan_features.iter().collect();
        heaviest.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
        let shown = std::cmp::min(ORPHAN_FEATURES_SHOWN, heaviest.len());
        let named = heaviest[..shown]
            .iter()
            .map(|(feature, cells)| format!("{feature} ({} cell(s))", cells.len()))
            .collect::<Vec<_>>()
            .join("; ");
        let more = heaviest.len() - shown;
        let tail = if more > 0 {
            format!(
                " +{more} more feature(s) — the full list is `bee status --json` → scribing_debt.orphaned."
            )
        } else {
            String::new()
        };
        lines.push(format!(
            "- Heaviest: {named}.{tail} Capped with no scribing sync ever recorded for their feature; run bee-capturing for one, then `bee state scribing-run --feature <feature> --areas \"<a,b>\" --next-action \"<n>\"` to stamp the repair."
        ));
    }

    // Decision 0017: capture stubs queued mid-flow, awaiting their flush pass.
    let queue_count = capture_queue_count(root);
    if queue_count > 0 {
        lines.push(String::new());
        lines.push(format!("### Capture queue: {queue_count} stub(s) pending flush"));
        lines.push(
            "- Settlements were stubbed mid-flow (decision 0017) — offer the flush now before new work: bee-capturing drains the queue oldest-first and merges each stub into its area spec.".to_string(),
        );
    }

    // D3 (kf-2): a promote proposal `bee close` wrote and nobody applied.
    let promote_lines = promote_proposal_lines(root);
    if !promote_lines.is_empty() {
        lines.push(String::new());
        lines.extend(promote_lines);
    }

    // D4 (D4a): a worktree `bee worktree merge` never reached and nobody
    // pruned — same blank-line separator as the promote-proposal block above.
    let reclaimable_lines = reclaimable_worktree_lines(root);
    if !reclaimable_lines.is_empty() {
        lines.push(String::new());
        lines.extend(reclaimable_lines);
    }

    // P7: keep the ceiling model scarce.
    if let Some((pct, ceiling, tiered)) = ceiling_scarcity_warning(root) {
        lines.push(String::new());
        lines.push(format!(
            "### Ceiling-model scarcity: {}% of tiered cells on ceiling",
            num_str(pct)
        ));
        lines.push(format!(
            "- {ceiling}/{tiered} cells tiered ceiling (> {}%) — the cost lever erodes when the strongest model touches most dispatches; re-tier routine cells to generation/extraction (decision 0012).",
            num_str(js_round(CEILING_MAX_SHARE * 100.0))
        ));
    }

    // D3/D2: the digest ranks against the session's ACTIVE feature (the
    // same one the knowledge bridge above reads) — never the default
    // record's feature unconditionally.
    let bound_feature = active.get("feature").and_then(Value::as_str).filter(|f| !f.is_empty());
    if let Some(digest) = critical_patterns_digest(root, PATTERN_DIGEST_LINES, bundle, bound_feature) {
        lines.push(String::new());
        lines.push("### Critical patterns (digest)".to_string());
        lines.extend(digest);
    }

    let decisions = active_decisions(root, Some(3));
    if !decisions.is_empty() {
        lines.push(String::new());
        lines.push("### Recent decisions".to_string());
        for event in &decisions {
            // Clamped: a decision's full text runs to a paragraph, and three
            // paragraphs of prose the reader can re-fetch by id is not what
            // an always-on block is for. The headline is the signal.
            //
            // Clamped BEFORE datamark, never after — datamark wraps its
            // result in «…», and cutting the wrapped string would eat the
            // closing guillemet and leave the quote hanging open.
            let clamped = vget(event, "decision")
                .map(|v| Value::String(clamp_chars(&jsjson::js_to_string(v), DECISION_CHARS)));
            lines.push(format!(
                "- {} ({})",
                datamark(clamped.as_ref()),
                tpl(vget(event, "date"))
            ));
        }
    }

    lines.push(String::new());
    // CUTOVER wording divergence: inject.mjs:583 spelled the command
    // `node .bee/bin/bee.mjs status --json`. Everything else is unchanged.
    lines.push("Everything above is already read — do not re-fetch it. Run `.bee/bin/bee status --json` yourself when you ROUTE WORK (claim, plan, change phase) or need detail this block does not carry. Never hand bee commands to the user. Route via bee-hive.".to_string());
    lines.join("\n")
}
