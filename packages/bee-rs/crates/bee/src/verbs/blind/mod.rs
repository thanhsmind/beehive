// verbs/blind — `bee blind check --dossier <path>`, the convergence door for
// slp-blind-lanes (D2(d), store 5981246b; shape B, decision f0f21142).
//
// A blind-lane run ends in ONE document: the convergence dossier, holding
// every lane's proposal verbatim, the cross-critiques, the chosen answer, the
// rejected set, the citations, and the revisit trigger. That document is the
// record — there is no `.bee/blind/` store, and this verb creates none. The
// caller names the file; bee reads it and answers.
//
// ─── WHY A MALFORMED DOSSIER REFUSES BY NAME ──────────────────────────────
//
// The dossier is a HAND-WRITTEN document, and it is the only place the lane
// proposals live. Every later check — D4's citation check, digest equality
// across lanes, the read-diet check — reads its sections. A parser that
// shrugged at a missing section would run those checks over less material and
// still report green, which is the failure mode the whole convergence step
// exists to prevent: a dossier that LOOKS checked. So the section contract is
// fixed, ordered, and every arm refuses by NAME of the offending section, in
// the `{ok:false, type:"refused", reason, …, fix}` shape `unmapped_kind_refusal`
// and every `--brief-file` refusal already take (verbs/drivers/prepare.rs).
//
// ─── WHAT THIS DOOR CHECKS TODAY ──────────────────────────────────────────
//
//   * the fixed sections, in order, each named when it is missing, extra,
//     repeated or out of place;
//   * every lane section's machine fields (`dispatch_id`, `brief_sha256`,
//     `role`, `paths_read`) and its verbatim proposal block, each named when
//     it is absent;
//   * the recorded brief, re-run through `lint_brief` — the SAME function the
//     dispatch door runs (verbs/drivers/brief_lint.rs), never a second copy of
//     its stems — so a convergence built on an unlinted brief refuses here
//     even though the door it bypassed never saw it;
//   * the three EVIDENCE checks, each reading a source outside the sentence
//     that makes the claim: every citation resolved against the proposal of
//     the lane it names, every lane's brief digest checked against the
//     dispatch log the door wrote, and every reported path held to the diet
//     the brief declared. They are not equally strong, and § "the three
//     evidence checks" below says exactly where each one's word comes from.
//
// `checks_run` on the result names all six, and the pass line carries the
// COUNTS — lanes, citations, paths. A zero count for any of them refuses
// instead: "checked nothing" must never render as "checked".
//
// ─── VERBATIM PAYLOADS RIDE IN FENCED BLOCKS ──────────────────────────────
//
// A lane proposal is arbitrary prose written by an advisor, and prose about
// this repo very plausibly contains a line like `## Chosen`. If the section
// scan could not tell a heading from a quoted heading, a lane's own text would
// move the dossier's section boundaries — the record checking itself against
// whatever the record happened to say. So the brief and every proposal are
// carried inside fenced blocks, and the heading scan ignores fenced lines.
//
// Root: WIDE (`resolve_store_root_any`), the same choice `verbs/discovery`
// makes for `docs/discovery/` — a dossier is plain git-tracked content and a
// worktree session must be able to check one. The root is used only for the
// timing line and the drift notice; the `--dossier` path itself is read AS
// GIVEN, relative to the process cwd, exactly like `--brief-file`.

use super::feedback::{emit_error, emit_success, js_trim, parse_shape, ParsedArgs};
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::drivers::{brief_refusal, lint_brief};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── the contract ───────────────────────────────────────────────────────

/// The dossier's level-1 title, minus the run id: `# Blind lane run <run-id>`.
pub(crate) const DOSSIER_TITLE_PREFIX: &str = "Blind lane run";

/// The fixed level-2 sections, in the order they must appear
/// (`docs/history/slp-blind-lanes/plan.md`, deferred question 3).
pub(crate) const DOSSIER_SECTIONS: [&str; 7] = [
    "Question",
    "Lanes",
    "Cross-critiques",
    "Chosen",
    "Rejected",
    "Citations",
    "Revisit trigger",
];

/// The machine fields every `### <lane-id>` section carries, each on its own
/// line as `<field>: <value>` (a leading `-` or `*` bullet is allowed).
///
/// `dispatch_id` is the chain of custody: without it a later digest check
/// compares the orchestrator's own transcriptions against each other, which
/// verifies the transcriber against itself rather than the run.
pub(crate) const LANE_FIELDS: [&str; 4] =
    ["dispatch_id", "brief_sha256", "role", "paths_read"];

/// What `bee blind check` actually ran, reported on every pass. The list is
/// the honest inventory of this door, not a badge: a caller reading `ok:true`
/// reads exactly which checks stand behind it, and the counts beside it say
/// how much material each one had.
const CHECKS_RUN: [&str; 6] = [
    "sections",
    "lane_fields",
    "brief_lint",
    "citations",
    "digest_equality",
    "read_diet",
];

// ─── the parsed document ────────────────────────────────────────────────

/// One `### <lane-id>` section: its machine fields plus its verbatim proposal.
/// Every field here is read by an evidence check — the document is parsed
/// once, by one contract, and the checks read THIS rather than the text again.
#[derive(Debug, Clone)]
pub(crate) struct LaneSection {
    pub id: String,
    pub dispatch_id: String,
    pub brief_sha256: String,
    pub role: String,
    /// The comma-separated `paths_read` list, split and trimmed. The lane's
    /// OWN confession of what it read — a stated list, never an enforced one.
    pub paths_read: Vec<String>,
    /// The proposal exactly as the lane returned it, fence delimiters removed.
    pub proposal: String,
}

/// A dossier that satisfied the section contract.
#[derive(Debug, Clone)]
pub(crate) struct Dossier {
    pub run_id: String,
    /// The LaneBrief recorded under `## Question`, verbatim.
    pub brief: String,
    pub lanes: Vec<LaneSection>,
    /// Every fixed section's body, in contract order — the material the
    /// citation, digest and diet checks read next.
    pub sections: Vec<(String, String)>,
}

impl Dossier {
    pub(crate) fn section(&self, name: &str) -> &str {
        self.sections
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, body)| body.as_str())
            .unwrap_or("")
    }

    fn to_value(&self, counts: &Counts) -> Value {
        let lanes: Vec<Value> = self
            .lanes
            .iter()
            .map(|l| {
                json!({
                    "id": l.id,
                    "dispatch_id": l.dispatch_id,
                    "brief_sha256": l.brief_sha256,
                    "role": l.role,
                    "paths_read": l.paths_read,
                })
            })
            .collect();
        json!({
            "ok": true,
            "run_id": self.run_id,
            "lanes": lanes,
            "citations_checked": counts.citations,
            "paths_checked": counts.paths,
            "checks_run": CHECKS_RUN,
        })
    }
}

// ─── markdown, read the way a record must be read ───────────────────────

/// Per-line "is this line inside (or itself) a fenced block?".
///
/// Fence delimiters count as fenced, so an opening ```` ``` ```` never reads
/// as prose. A closing delimiter must use the same character as its opener,
/// be at least as long, and carry nothing else — so a proposal may contain
/// its own shorter fences without ending the block that quotes it.
fn fence_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(lines.len());
    let mut open: Option<(char, usize)> = None;
    for line in lines {
        let t = line.trim_start();
        let marker = t.chars().next().filter(|c| *c == '`' || *c == '~');
        let run = marker.map_or(0, |c| t.chars().take_while(|x| *x == c).count());
        match open {
            None => {
                if run >= 3 {
                    open = Some((marker.unwrap_or('`'), run));
                }
                mask.push(run >= 3);
            }
            Some((c, n)) => {
                mask.push(true);
                let closes = marker == Some(c)
                    && run >= n
                    && t.trim_end().chars().all(|x| x == c);
                if closes {
                    open = None;
                }
            }
        }
    }
    mask
}

/// `(level, text)` for an ATX heading line, or `None`. A `#` run with no
/// whitespace after it is not a heading — the same rule `brief_lint`'s own
/// `heading_text` applies, so the two guards read one markdown.
fn heading_of(line: &str) -> Option<(usize, &str)> {
    let t = line.trim_start();
    let level = t.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &t[level..];
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        None
    } else {
        Some((level, text))
    }
}

struct Heading {
    level: usize,
    text: String,
    line: usize,
}

/// Every heading OUTSIDE a fenced block, in document order.
fn headings(lines: &[&str], mask: &[bool]) -> Vec<Heading> {
    lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !mask[*i])
        .filter_map(|(i, l)| heading_of(l).map(|(level, text)| Heading {
            level,
            text: text.to_string(),
            line: i,
        }))
        .collect()
}

/// The first fenced block in `lines[range]`, delimiters removed, or `None`.
fn first_fenced_block(lines: &[&str], mask: &[bool], from: usize, to: usize) -> Option<String> {
    let mut start: Option<usize> = None;
    for i in from..to.min(lines.len()) {
        if mask[i] {
            if start.is_none() {
                start = Some(i + 1); // skip the opening delimiter
            }
        } else if let Some(s) = start {
            // the line before this prose line closed the block
            let body: Vec<&str> = lines[s..i.saturating_sub(1).max(s)].to_vec();
            return Some(body.join("\n"));
        }
    }
    let s = start?;
    let end = to.min(lines.len());
    // an unterminated block runs to the end of the section; its last line is
    // the closing delimiter only when one was actually written.
    let stop = if end > s && lines[end - 1].trim_start().starts_with(['`', '~']) {
        end - 1
    } else {
        end
    };
    Some(lines[s..stop.max(s)].join("\n"))
}

/// `<field>: <value>` on a prose line, with an optional `-`/`*` bullet. The
/// field name folds case; the value is trimmed and may be empty (which every
/// caller treats as absent — a field written with nothing after the colon is
/// a field that was not recorded).
fn field_value(lines: &[&str], mask: &[bool], from: usize, to: usize, field: &str) -> Option<String> {
    for i in from..to.min(lines.len()) {
        if mask[i] {
            continue;
        }
        let mut t = lines[i].trim();
        if let Some(rest) = t.strip_prefix('-').or_else(|| t.strip_prefix('*')) {
            t = rest.trim_start();
        }
        let Some(colon) = t.find(':') else { continue };
        if t[..colon].trim().eq_ignore_ascii_case(field) {
            let value = t[colon + 1..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

// ─── refusals ───────────────────────────────────────────────────────────

/// ONE refusal shape at this door, and it is the SAME one the dispatch door
/// returns — `brief_refusal` is reused rather than re-derived, so a caller
/// handling a `--brief-file` refusal already handles these.
fn refuse(reason: &str, extra: &[(&str, Value)], fix: String) -> Value {
    brief_refusal(reason, extra, fix)
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// ─── the section contract ───────────────────────────────────────────────

/// Parse a dossier, or refuse by name.
///
/// Arm order is deliberate and mirrors `lint_brief_shape`: TITLE first (with
/// no title there is no run to check), then extra/repeated sections (an
/// unknown heading names itself, the most actionable thing to say), then
/// missing, then order, then the per-lane material, then the recorded brief.
pub(crate) fn parse_dossier(text: &str) -> Result<Dossier, Value> {
    let lines: Vec<&str> = text.lines().collect();
    let mask = fence_mask(&lines);
    let heads = headings(&lines, &mask);
    let expected = DOSSIER_SECTIONS.join(", ");

    // ── the title ────────────────────────────────────────────────────────
    let title = heads.first().filter(|h| h.level == 1).ok_or_else(|| {
        let found = heads.first().map(|h| h.text.clone());
        refuse(
            "dossier_title_missing",
            &[("found".into(), found.clone().map_or(Value::Null, Value::String))],
            match found {
                Some(f) => format!(
                    "the dossier's first heading is \"{f}\"; it must be \"# {DOSSIER_TITLE_PREFIX} <run-id>\". Add that title line at the top — the run id is what every later check reports against."
                ),
                None => format!(
                    "the dossier carries no headings at all, so there is nothing to check. It must open with \"# {DOSSIER_TITLE_PREFIX} <run-id>\" and then carry these level-2 sections in order: {expected}."
                ),
            },
        )
    })?;
    let run_id = title
        .text
        .strip_prefix(DOSSIER_TITLE_PREFIX)
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            refuse(
                "dossier_title_missing",
                &[("found".into(), s(&title.text))],
                format!(
                    "the dossier's title reads \"# {}\"; it must read \"# {DOSSIER_TITLE_PREFIX} <run-id>\", naming the run. Rewrite the title with the run id.",
                    title.text
                ),
            )
        })?
        .to_string();

    // ── the fixed sections ───────────────────────────────────────────────
    // A second level-1 heading is an extra SECTION, not a second title: the
    // dossier is one run's record.
    for h in heads.iter().skip(1).filter(|h| h.level == 1) {
        return Err(refuse(
            "dossier_section_unexpected",
            &[("section".into(), s(&h.text))],
            format!(
                "\"# {}\" is a second level-1 heading; a dossier records ONE run and carries exactly one title. Demote it to one of the seven sections, in this order: {expected}.",
                h.text
            ),
        ));
    }
    let sections: Vec<&Heading> = heads.iter().filter(|h| h.level == 2).collect();
    for (i, h) in sections.iter().enumerate() {
        let known = DOSSIER_SECTIONS.iter().any(|w| w.eq_ignore_ascii_case(&h.text));
        let repeated = sections[..i].iter().any(|p| p.text.eq_ignore_ascii_case(&h.text));
        if known && !repeated {
            continue;
        }
        let why = if repeated {
            format!("the section \"{}\" appears more than once", h.text)
        } else {
            format!("\"{}\" is not one of them", h.text)
        };
        return Err(refuse(
            "dossier_section_unexpected",
            &[("section".into(), s(&h.text))],
            format!(
                "a dossier carries exactly seven level-2 sections, in this order: {expected}. Here {why}. Remove that heading, or fold its content into the section it belongs to."
            ),
        ));
    }
    for want in DOSSIER_SECTIONS {
        if !sections.iter().any(|h| h.text.eq_ignore_ascii_case(want)) {
            return Err(refuse(
                "dossier_section_missing",
                &[("section".into(), s(want))],
                format!(
                    "the dossier has no \"## {want}\" section, so nothing can be checked against it. The seven sections read in this order: {expected}. Add \"## {want}\" in that position."
                ),
            ));
        }
    }
    for (i, want) in DOSSIER_SECTIONS.iter().enumerate() {
        let found = &sections[i].text;
        if !found.eq_ignore_ascii_case(want) {
            return Err(refuse(
                "dossier_section_out_of_order",
                &[("section".into(), s(found))],
                format!(
                    "the dossier's sections are out of order: \"{found}\" stands where \"{want}\" is due. They must read in this order: {expected}. Move \"{found}\" back below \"{want}\"."
                ),
            ));
        }
    }

    // ── section bodies, and where the lanes live ─────────────────────────
    let end_of = |idx: usize| -> usize {
        sections.get(idx + 1).map_or(lines.len(), |h| h.line)
    };
    let mut bodies: Vec<(String, String)> = Vec::with_capacity(DOSSIER_SECTIONS.len());
    for (i, h) in sections.iter().enumerate() {
        let from = h.line + 1;
        let to = end_of(i);
        bodies.push((h.text.clone(), lines[from.min(lines.len())..to].join("\n")));
    }
    let lanes_at = DOSSIER_SECTIONS.iter().position(|w| *w == "Lanes").unwrap_or(1);
    let lanes_from = sections[lanes_at].line;
    let lanes_to = end_of(lanes_at);

    // A `### …` heading is a LANE, and lanes live under `## Lanes`. One
    // anywhere else is a section boundary nobody declared.
    for h in heads.iter().filter(|h| h.level == 3) {
        if h.line > lanes_from && h.line < lanes_to {
            continue;
        }
        let owner = sections
            .iter()
            .rev()
            .find(|sh| sh.line < h.line)
            .map_or("(before any section)".to_string(), |sh| sh.text.clone());
        return Err(refuse(
            "dossier_subsection_unexpected",
            &[("section".into(), s(&owner)), ("heading".into(), s(&h.text))],
            format!(
                "\"### {}\" sits under \"## {owner}\"; a level-3 heading in a dossier is a LANE and belongs under \"## Lanes\". Move it there, or write it as prose.",
                h.text
            ),
        ));
    }

    // ── the lane sections ────────────────────────────────────────────────
    let lane_heads: Vec<&Heading> = heads
        .iter()
        .filter(|h| h.level == 3 && h.line > lanes_from && h.line < lanes_to)
        .collect();
    if lane_heads.is_empty() {
        return Err(refuse(
            "dossier_lanes_missing",
            &[("section".into(), s("Lanes"))],
            "the \"## Lanes\" section carries no \"### <lane-id>\" subsection, so the dossier records no proposal to check. A blind run opens 2-3 lanes; write one subsection per lane, each with its dispatch_id, brief_sha256, role, paths_read and its proposal in a fenced block."
                .to_string(),
        ));
    }
    let mut lanes: Vec<LaneSection> = Vec::with_capacity(lane_heads.len());
    for (i, h) in lane_heads.iter().enumerate() {
        if lane_heads[..i].iter().any(|p| p.text.eq_ignore_ascii_case(&h.text)) {
            return Err(refuse(
                "dossier_lane_repeated",
                &[("lane".into(), s(&h.text))],
                format!(
                    "the lane \"{}\" has more than one \"### \" subsection. Each lane appears exactly once — two sections under one id make every later per-lane check ambiguous about which bytes it read. Give the second lane its own id.",
                    h.text
                ),
            ));
        }
        let from = h.line + 1;
        let to = lane_heads.get(i + 1).map_or(lanes_to, |n| n.line);
        let mut values: Vec<String> = Vec::with_capacity(LANE_FIELDS.len());
        for field in LANE_FIELDS {
            let Some(v) = field_value(&lines, &mask, from, to, field) else {
                return Err(refuse(
                    "dossier_lane_field_missing",
                    &[("lane".into(), s(&h.text)), ("field".into(), s(field))],
                    format!(
                        "the lane \"{}\" records no \"{field}\". Every lane section carries {} on its own line, as \"{field}: <value>\". Without {field} this dossier cannot be checked against the run that produced it.",
                        h.text,
                        LANE_FIELDS.join(", ")
                    ),
                ));
            };
            values.push(v);
        }
        let Some(proposal) = first_fenced_block(&lines, &mask, from, to) else {
            return Err(refuse(
                "dossier_lane_proposal_missing",
                &[("lane".into(), s(&h.text))],
                format!(
                    "the lane \"{}\" records no proposal. The proposal is carried VERBATIM in a fenced block inside the lane's section — it is the only copy of the lane's own bytes, and every citation is later resolved against it. Paste it between fence lines.",
                    h.text
                ),
            ));
        };
        lanes.push(LaneSection {
            id: h.text.clone(),
            dispatch_id: values[0].clone(),
            brief_sha256: values[1].clone(),
            role: values[2].clone(),
            paths_read: values[3]
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect(),
            proposal,
        });
    }

    // ── the recorded brief, through the door's OWN guard ─────────────────
    //
    // The brief lives in `## Question` as a fenced block: the whole LaneBrief,
    // its four sections intact, not a paraphrase. That is what makes this a
    // re-run rather than a second, weaker check — `lint_brief` reads the same
    // bytes it would have read at the dispatch door, so a convergence built on
    // a brief that never passed that door refuses here instead.
    let question_at = 0usize;
    let q_from = sections[question_at].line + 1;
    let q_to = end_of(question_at);
    let Some(brief) = first_fenced_block(&lines, &mask, q_from, q_to) else {
        return Err(refuse(
            "dossier_brief_missing",
            &[("section".into(), s("Question"))],
            "the \"## Question\" section carries no fenced block, so the LaneBrief the lanes answered was not recorded. Paste the brief VERBATIM between fence lines under \"## Question\" — it is what the leaning guard re-reads and what every lane's brief_sha256 is checked against."
                .to_string(),
        ));
    };
    if let Err(mut inner) = lint_brief(&brief) {
        if let Some(map) = inner.as_object_mut() {
            map.insert("dossier_section".into(), s("Question"));
        }
        return Err(inner);
    }

    Ok(Dossier { run_id, brief, lanes, sections: bodies })
}

// ─── the three evidence checks ──────────────────────────────────────────
//
// The section contract proves a dossier is SHAPED like a record. These three
// ask whether it is TRUE, and each one answers from a source outside the
// sentence that makes the claim.
//
//   1. CITATIONS (D4). Every `<lane-id> :: <quote>` line resolves against
//      THAT lane's own proposal bytes — never against the concatenated set.
//      Containment over all proposals at once passes a quote attributed to
//      lane A but written only by lane B, and that misattribution IS the
//      fabrication D4 exists to catch: it manufactures agreement between
//      lanes that never agreed. What a resolved citation proves is
//      PROVENANCE — the quote is a whole sentence of the named lane's own
//      bytes — never faithfulness to what that lane meant; `is_sentence_end`
//      states the boundary rule's own limits.
//   2. DIGEST EQUALITY. Every lane's `brief_sha256` is checked against the
//      digest its `dispatch_id` carries in `.bee/logs/dispatch.jsonl`, the
//      record the dispatch door itself wrote. Comparing the dossier's
//      transcribed digests with each other would only verify the transcriber
//      against itself. Then the logged digests must be equal across the
//      lanes: unequal means the brief CHANGED between lane 1 and lane 3 —
//      the cost decision f0f21142 accepted, detected here, not prevented.
//   3. READ DIET. Every path a lane reports sits inside the diet the recorded
//      brief declares, and no path names `.bee/` at all.
//
// ─── THE THREE ARE NOT EQUALLY STRONG, AND THE DOOR SAYS SO ───────────────
//
// Checks 1 and 2 read bytes this process holds: the proposal inside the
// dossier, the line inside the dispatch log. Check 3 reads the lane's OWN
// REPORT of what it read. Nothing here observed a single read. It catches a
// careless breach and a recorded lie; it does NOT catch a lane that read
// `.bee/decisions.jsonl` and then wrote a tidy `paths_read` line. That
// limit is stated in the refusal itself and in this comment, because a check
// whose strength is misread is worse than no check: it retires a suspicion it
// never earned.

/// A quote shorter than this is generic enough to sit inside almost any
/// proposal, so resolving it proves nothing. Both floors must be cleared.
const MIN_CITATION_CHARS: usize = 24;
const MIN_CITATION_WORDS: usize = 5;

/// What a citation line puts between the lane it names and the quote.
const CITATION_SEP: &str = "::";

/// What the evidence pass actually looked at, reported on the result so a
/// reader can tell a checked dossier from a small one.
pub(crate) struct Counts {
    pub citations: usize,
    pub paths: usize,
}

/// `.bee/logs/dispatch.jsonl`, read as the authoritative record of what was
/// dispatched — the chain of custody the dossier's own fields cannot supply.
///
/// A line counts when it carries a `dispatch_id`; `append_prepare_record`
/// writes one per served prepare with `source: "prepare"`, and the digest is
/// taken from whichever line for that id carries one. Broader than
/// prepare-only on purpose: a false "unlogged" refusal would block an honest
/// convergence, while an id that appears NOWHERE is still refused by name.
pub(crate) struct DispatchLog {
    path: String,
    by_id: HashMap<String, Option<String>>,
}

impl DispatchLog {
    pub(crate) fn parse(path: &str, text: &str) -> Self {
        let mut by_id: HashMap<String, Option<String>> = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
            let Some(id) = v.get("dispatch_id").and_then(Value::as_str) else { continue };
            let digest = v.get("brief_sha256").and_then(Value::as_str).map(str::to_string);
            let slot = by_id.entry(id.to_string()).or_insert(None);
            if slot.is_none() {
                *slot = digest;
            }
        }
        DispatchLog { path: path.to_string(), by_id }
    }

    /// The log as it sits on disk. A missing file reads as an EMPTY log, and
    /// an empty log refuses every lane by name — the write side is fail-open
    /// ("a log failure never blocks the payload", `append_prepare_record`),
    /// so the read side must be the opposite or the hole would pass silently.
    pub(crate) fn read(root: &Path) -> Self {
        let file = root.join(".bee").join("logs").join("dispatch.jsonl");
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        DispatchLog::parse(&file.display().to_string(), &text)
    }
}

/// Whitespace runs collapse to one space, edges trim, ASCII case folds. Both
/// sides of every citation comparison go through THIS function — a quote
/// re-wrapped by hand is still the lane's own sentence.
fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            space = !out.is_empty();
        } else {
            if space {
                out.push(' ');
            }
            space = false;
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

fn is_terminator(b: u8) -> bool {
    matches!(b, b'.' | b'!' | b'?')
}

/// Dots that do NOT end a sentence. CLOSED on purpose: a set that grows by
/// guesswork would start refusing honest citations, and every miss here is a
/// dot the boundary rule reads as a sentence end — the safe direction is to
/// list the marks that actually appear in design prose. Lowercase, dot
/// included, because both sides of the scan are `normalize`d.
///
/// One entry per abbreviation, never one per spelling: the lookup strips the
/// leading punctuation off the token first, so `(i.e.`, `"e.g.` and `-cf.`
/// all land on the row already here. Spelling punctuated variants out would
/// be the same miss with more rows.
const NON_TERMINAL_ABBREVIATIONS: [&str; 8] =
    ["i.e.", "e.g.", "etc.", "cf.", "vs.", "no.", "fig.", "approx."];

/// Is the terminator at `at` the END OF A SENTENCE, or only a dot?
///
/// ─── WHAT THIS RULE CATCHES, AND WHAT IT CANNOT ───────────────────────────
///
/// The rule exists for ONE class: a citation that starts mid-sentence and
/// drops the words that reverse it — "we should not cache the token" cited as
/// "cache the token". Requiring the span to sit between two sentence
/// boundaries RAISES THE COST of that strip and refuses the written forms
/// enumerated below. It does NOT make a within-sentence strip impossible: the
/// dropped negation is the text between the boundary and the quote, so the
/// strip survives wherever a dot this rule reads as a boundary stands between
/// them. Read this list as the whole of what is caught, never as a class
/// closed by it.
///
/// A dot alone is not a boundary. "we should not follow lane-b here, i.e.
/// cache the token" has a dot before the clause, and reading it as a sentence
/// end hands back the whole hole this rule closes. So a dot ends a sentence
/// only when three things hold: it is not part of a run (an ellipsis is one
/// mark, not three ends); the WORD it closes — the token with its leading
/// brackets, quotes and dashes stripped, because "(i.e." is the commonest
/// spelling of the abbreviation — is not a listed abbreviation; and that word
/// is not a list marker ("1.", "a.", "ii."), because a numbered list under a
/// negated stem is ordinary design prose and every item is governed by it.
///
/// WHAT IT CANNOT DO. This is a mechanical string check over ONE sentence,
/// and it cannot decide whether a citation is faithful to what the lane
/// meant. Four limits, named rather than papered over:
///   * framing one sentence back — "Never do the following. Cache the token
///     on the worker side." cited as the second sentence alone is a whole
///     sentence of the lane's own bytes and PASSES. That is decided, not
///     missed: refusing it needs the meaning of the previous sentence, and a
///     rule that guessed there would refuse honest citations wholesale;
///   * the abbreviation set is closed, so an unlisted abbreviation ("resp.",
///     "ibid.") still reads as a sentence end;
///   * the list-marker shape is narrow — digits, one letter, or a run of the
///     numeral letters i/v/x — so a marker outside it ("A1.", "step.") still
///     reads as a sentence end;
///   * a real sentence that genuinely ends in a listed abbreviation, in a
///     number, or in a one-letter word is refused. Every miss but the first
///     lands on the strict side, which is why the first is stated everywhere
///     this check is described.
///
/// The true claim, and the only one any refusal or doc may make: a resolved
/// citation is a WHOLE SENTENCE of the named lane's own bytes. Provenance,
/// not faithfulness.
fn is_sentence_end(h: &[u8], at: usize) -> bool {
    if h[at] != b'.' {
        return true;
    }
    // A run of two or more dots is one mark — an ellipsis ends nothing.
    if (at > 0 && h[at - 1] == b'.') || h.get(at + 1) == Some(&b'.') {
        return false;
    }
    let mut k = at;
    while k > 0 && h[k - 1] != b' ' {
        k -= 1;
    }
    // Match on the WORD, not the raw token: an opening bracket, quote or dash
    // rides on the front of "(i.e." and "\"e.g.", and a lookup by whole-token
    // equality is defeated by that one character.
    let token = &h[k..=at];
    let word = match token.iter().position(u8::is_ascii_alphanumeric) {
        Some(i) => &token[i..],
        None => token,
    };
    if NON_TERMINAL_ABBREVIATIONS.iter().any(|a| a.as_bytes() == word) {
        return false;
    }
    !is_enumerator(word)
}

/// Is `word` (dot included) a LIST MARKER rather than a word that ends a
/// sentence? "1.", "a." and "ii." open an item under a stem that governs it —
/// "we must not do any of the following:" — so reading that dot as a sentence
/// end lets a citation start at the item and leave the stem behind.
///
/// The shape is deliberately narrow: all digits, ONE letter, or a run of the
/// numeral letters i/v/x. English has no ordinary word of that shape, so a
/// sentence-final short word ("one.", "it.", "all.") is not mistaken for a
/// marker. Case is not a signal here — both sides of the scan are
/// `normalize`d, so every letter arrives lowercase.
fn is_enumerator(word: &[u8]) -> bool {
    let Some((last, body)) = word.split_last() else { return false };
    if *last != b'.' || body.is_empty() {
        return false;
    }
    if body.iter().all(u8::is_ascii_digit) {
        return true;
    }
    if !body.iter().all(u8::is_ascii_alphabetic) {
        return false;
    }
    body.len() == 1
        || (body.len() <= 3 && body.iter().all(|b| matches!(b, b'i' | b'v' | b'x')))
}

/// Does `needle` stand in `hay` as a WHOLE-SENTENCE span — starting where a
/// sentence starts and ending where one ends?
///
/// Plain containment is what makes the negation strip work: a proposal that
/// says "we should not cache the token" contains "cache the token", and a
/// dossier citing the second reports the lane as saying the opposite of what
/// it said. Requiring the span to begin at a sentence boundary makes that
/// drop cost a boundary the citation has to find — the dropped "we should
/// not" is the text between the sentence start and the quote, so the strip
/// works only where `is_sentence_end` reads a boundary, and that function
/// states exactly which marks it reads and which it does not. Both `hay` and
/// `needle` arrive normalized, so the scan is over ASCII bytes and a byte
/// equal to `.` or ` ` can only be that character.
fn quote_resolves(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let h = hay.as_bytes();
    let mut from = 0usize;
    while let Some(off) = hay[from..].find(needle) {
        let start = from + off;
        let end = start + needle.len();
        let starts_sentence = start == 0 || {
            let mut k = start;
            while k > 0 && h[k - 1] == b' ' {
                k -= 1;
            }
            k < start && k > 0 && is_terminator(h[k - 1]) && is_sentence_end(h, k - 1)
        };
        let ends_sentence = end == h.len() || (is_terminator(h[end]) && is_sentence_end(h, end));
        if starts_sentence && ends_sentence {
            return true;
        }
        from = start + 1;
    }
    false
}

/// The quote as it is compared: normalized, with its own trailing sentence
/// punctuation dropped so citing the closing period is not a mismatch.
fn citation_quote(raw: &str) -> String {
    normalize(raw).trim_end_matches(['.', '!', '?', ' ']).to_string()
}

// ─── CHECK 1 — every citation resolves against the lane it names ────────

fn check_citations(d: &Dossier) -> Result<usize, Value> {
    let proposals: Vec<(&str, String)> =
        d.lanes.iter().map(|l| (l.id.as_str(), normalize(&l.proposal))).collect();
    let known = d.lanes.iter().map(|l| l.id.as_str()).collect::<Vec<_>>().join(", ");
    let mut checked = 0usize;

    for raw in d.section("Citations").lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let parts = line.split_once(CITATION_SEP);
        let (lane_id, quote_raw) = match parts {
            Some((l, q)) if !l.trim().is_empty() && !q.trim().is_empty() => (l.trim(), q.trim()),
            _ => {
                return Err(refuse(
                    "dossier_citation_malformed",
                    &[("line".into(), s(line))],
                    format!(
                        "the citation line \"{line}\" is not \"<lane-id> {CITATION_SEP} <quote>\". Every line under \"## Citations\" names the lane it came from and then quotes it, so the quote can be resolved against that lane's own proposal. Rewrite it in that form, or delete the line."
                    ),
                ));
            }
        };
        let quote = citation_quote(quote_raw);
        let words = quote.split(' ').filter(|w| !w.is_empty()).count();
        if quote.chars().count() < MIN_CITATION_CHARS || words < MIN_CITATION_WORDS {
            return Err(refuse(
                "dossier_citation_too_short",
                &[("lane".into(), s(lane_id)), ("quote".into(), s(quote_raw))],
                format!(
                    "the citation \"{quote_raw}\" is too short to be evidence: a fragment that brief sits inside almost any proposal, so resolving it proves nothing about what {lane_id} actually said. Quote at least {MIN_CITATION_CHARS} characters and {MIN_CITATION_WORDS} words — a whole sentence of the lane's own text."
                ),
            ));
        }
        let Some((_, own)) = proposals.iter().find(|(id, _)| id.eq_ignore_ascii_case(lane_id))
        else {
            return Err(refuse(
                "dossier_citation_lane_unknown",
                &[("lane".into(), s(lane_id)), ("quote".into(), s(quote_raw))],
                format!(
                    "the citation names the lane \"{lane_id}\", and this dossier has no \"### {lane_id}\" section. The lanes it records are: {known}. Fix the lane id, or add the missing lane section with its proposal verbatim."
                ),
            ));
        };
        if quote_resolves(own, &quote) {
            checked += 1;
            continue;
        }
        // The text may be genuine and simply in the WRONG lane. Saying so is
        // both the more actionable message and the louder one: misattribution
        // is a dossier reporting agreement the lanes never reached.
        if let Some((other, _)) = proposals
            .iter()
            .find(|(id, body)| !id.eq_ignore_ascii_case(lane_id) && quote_resolves(body, &quote))
        {
            return Err(refuse(
                "dossier_citation_misattributed",
                &[
                    ("lane".into(), s(lane_id)),
                    ("found_in".into(), s(other)),
                    ("quote".into(), s(quote_raw)),
                ],
                format!(
                    "the citation credits \"{lane_id}\" with a sentence only \"{other}\" wrote. Blind lanes are worth their cost because they answered independently; moving one lane's words onto another manufactures an agreement that never happened. Attribute the quote to {other}, or cite what {lane_id} actually said."
                ),
            ));
        }
        return Err(refuse(
            "dossier_citation_unresolved",
            &[("lane".into(), s(lane_id)), ("quote".into(), s(quote_raw))],
            format!(
                "the citation \"{quote_raw}\" does not resolve against the proposal \"{lane_id}\" recorded here. A quote must stand as a WHOLE SENTENCE of that lane's text: a span that starts mid-sentence can drop the very words that reverse the meaning — \"we should not cache the token\" cited as \"cache the token\" reads as the opposite of the lane's answer. Copy the sentence from the lane's proposal block, or drop the citation."
            ),
        ));
    }

    if checked == 0 {
        return Err(refuse(
            "dossier_citations_empty",
            &[("section".into(), s("Citations"))],
            "the dossier records no citation, so the chosen answer rests on nothing this door can resolve against the lanes. A convergence cites the lanes it synthesized: write one \"<lane-id> :: <quote>\" line per claim the answer leans on. Zero citations is refused rather than reported as checked."
                .to_string(),
        ));
    }
    Ok(checked)
}

// ─── CHECK 2 — one brief, proven against the dispatch log ───────────────

fn check_digests(d: &Dossier, log: &DispatchLog) -> Result<(), Value> {
    let where_ = &log.path;
    let mut logged: Vec<(&str, &str)> = Vec::with_capacity(d.lanes.len());

    for lane in &d.lanes {
        let Some(entry) = log.by_id.get(&lane.dispatch_id) else {
            return Err(refuse(
                "dossier_dispatch_unlogged",
                &[
                    ("lane".into(), s(&lane.id)),
                    ("dispatch_id".into(), s(&lane.dispatch_id)),
                    ("log".into(), s(where_)),
                ],
                format!(
                    "the lane \"{}\" names dispatch_id {} and no line in {where_} carries it, so there is no record that this dispatch ever happened. That log is written fail-open — a log failure never blocks a payload — which is exactly why a missing line is refused here instead of passed: an unlogged dispatch_id is either a dispatch that did not run or a log this check cannot see. Re-run the lane through `bee dispatch prepare`, or point the check at the root whose .bee/logs holds the run.",
                    lane.id, lane.dispatch_id
                ),
            ));
        };
        let Some(digest) = entry else {
            return Err(refuse(
                "dossier_brief_digest_unlogged",
                &[
                    ("lane".into(), s(&lane.id)),
                    ("dispatch_id".into(), s(&lane.dispatch_id)),
                    ("log".into(), s(where_)),
                ],
                format!(
                    "the lane \"{}\" names dispatch_id {}, and that line in {where_} records no brief_sha256 — so it carried no brief, and nothing pins the bytes this lane answered. A blind lane is dispatched with `--brief-file`; a dispatch without one is not a lane. Re-run it with the brief.",
                    lane.id, lane.dispatch_id
                ),
            ));
        };
        if !digest.eq_ignore_ascii_case(&lane.brief_sha256) {
            return Err(refuse(
                "dossier_brief_digest_mismatch",
                &[
                    ("lane".into(), s(&lane.id)),
                    ("dispatch_id".into(), s(&lane.dispatch_id)),
                    ("recorded".into(), s(&lane.brief_sha256)),
                    ("logged".into(), s(digest)),
                    ("log".into(), s(where_)),
                ],
                format!(
                    "the lane \"{}\" records brief_sha256 {}, and {where_} logged {} for its dispatch_id. The log is the record the dispatch door wrote; the dossier line was transcribed by hand. Correct the dossier to the logged digest — and if the logged one is the surprise, the lane did not answer the brief this dossier says it answered.",
                    lane.id, lane.brief_sha256, digest
                ),
            ));
        }
        logged.push((lane.id.as_str(), digest.as_str()));
    }

    let Some((first_lane, first)) = logged.first().copied() else { return Ok(()) };
    for (lane, digest) in &logged[1..] {
        if !digest.eq_ignore_ascii_case(first) {
            return Err(refuse(
                "dossier_brief_digest_divergent",
                &[
                    ("lane".into(), s(lane)),
                    ("other_lane".into(), s(first_lane)),
                    ("digest".into(), s(digest)),
                    ("other_digest".into(), s(first)),
                    ("log".into(), s(where_)),
                ],
                format!(
                    "the lanes did not answer one brief: {where_} logged {digest} for \"{lane}\" and {first} for \"{first_lane}\". Blind lanes are comparable only because the brief is byte-identical across them, so a divergence means the brief changed between one dispatch and the next — this run compared answers to two different questions. Re-run every lane on ONE brief, then rebuild the dossier."
                ),
            ));
        }
    }
    Ok(())
}

// ─── CHECK 3 — the read diet, as far as a self-report can carry ─────────

/// The one honest sentence about this check's strength, carried on every one
/// of its refusals. It is written ONCE so no arm can quietly state a stronger
/// claim than the check earns.
const DIET_TRUST: &str = "This check reads the lane's own self-reported paths_read line — nothing here observed a read — so it is weaker than the citation check, which resolves quotes against proposal bytes this door holds. It catches a careless breach and a recorded lie, never a determined one.";

/// The paths the recorded brief's `## Read diet` section declares. The brief
/// is parsed as its own document — its `##` headings are top-level there.
fn declared_diet(brief: &str) -> Vec<String> {
    let lines: Vec<&str> = brief.lines().collect();
    let mask = fence_mask(&lines);
    let heads = headings(&lines, &mask);
    let Some(at) = heads.iter().position(|h| h.text.eq_ignore_ascii_case("Read diet")) else {
        return Vec::new();
    };
    let from = heads[at].line + 1;
    let to = heads.get(at + 1).map_or(lines.len(), |h| h.line);
    let mut out = Vec::new();
    for i in from..to.min(lines.len()) {
        if mask[i] {
            continue;
        }
        let mut t = lines[i].trim();
        if let Some(rest) = t.strip_prefix('-').or_else(|| t.strip_prefix('*')) {
            t = rest.trim_start();
        }
        // A diet entry is a path; anything after the first whitespace is the
        // note a human wrote beside it.
        if let Some(first) = t.split_whitespace().next() {
            let p = normalize_path(first);
            if !p.is_empty() {
                out.push(p);
            }
        }
    }
    out
}

fn normalize_path(p: &str) -> String {
    p.trim().replace('\\', "/").trim_start_matches("./").to_string()
}

/// A diet entry covers a path when it IS the path or is a directory above it.
fn diet_covers(diet: &[String], path: &str) -> bool {
    diet.iter().any(|entry| {
        let e = entry.trim_end_matches('/');
        if path.eq_ignore_ascii_case(e) {
            return true;
        }
        path.len() > e.len()
            && path.as_bytes()[e.len()] == b'/'
            && path[..e.len()].eq_ignore_ascii_case(e)
    })
}

/// Does any segment of this path name `.bee`? Segment-wise, so a file called
/// `notes-about-bee.md` is not a hit and `docs/.bee/x` is.
fn names_bee_store(path: &str) -> bool {
    path.split('/').any(|seg| seg == ".bee")
}

fn check_read_diet(d: &Dossier) -> Result<usize, Value> {
    let diet = declared_diet(&d.brief);
    if diet.is_empty() {
        return Err(refuse(
            "dossier_read_diet_undeclared",
            &[("section".into(), s("Read diet")), ("trust".into(), s("self_reported"))],
            format!(
                "the recorded brief's \"## Read diet\" section lists no path, so there is no diet to check the lanes against and this check would report success over nothing. A blind lane is dispatched with an explicit read-only path diet (D2(b)); list one path per line under that heading. {DIET_TRUST}"
            ),
        ));
    }

    let mut checked = 0usize;
    for lane in &d.lanes {
        if lane.paths_read.is_empty() {
            return Err(refuse(
                "dossier_read_diet_empty",
                &[("lane".into(), s(&lane.id)), ("trust".into(), s("self_reported"))],
                format!(
                    "the lane \"{}\" reports no path in its paths_read line, so its proposal came from nothing this dossier records reading. An empty report is refused rather than counted as a clean one — zero paths checked is not a diet kept. List the paths the lane read. {DIET_TRUST}",
                    lane.id
                ),
            ));
        }
        for raw in &lane.paths_read {
            let path = normalize_path(raw);
            if names_bee_store(&path) {
                return Err(refuse(
                    "dossier_read_diet_bee_path",
                    &[
                        ("lane".into(), s(&lane.id)),
                        ("path".into(), s(raw)),
                        ("trust".into(), s("self_reported")),
                    ],
                    format!(
                        "the lane \"{}\" reports reading \"{raw}\", inside the bee store. That is where the orchestrator's own leaning lives — D1 forces an open reason, and it lands in .bee/decisions.jsonl on the same disk the lane can read — so a lane that reads .bee/ is no longer blind, whatever the brief allowed. No diet can license a .bee/ path. Re-run the lane without it. {DIET_TRUST}",
                        lane.id
                    ),
                ));
            }
            if !diet_covers(&diet, &path) {
                return Err(refuse(
                    "dossier_read_diet_breach",
                    &[
                        ("lane".into(), s(&lane.id)),
                        ("path".into(), s(raw)),
                        ("trust".into(), s("self_reported")),
                        ("diet".into(), json!(diet)),
                    ],
                    format!(
                        "the lane \"{}\" reports reading \"{raw}\", which the brief's read diet does not declare. The diet is what makes two lanes comparable: a lane that read more answered a different question. The declared diet is: {}. Add the path to the brief and re-run the lanes, or drop the lane from this dossier. {DIET_TRUST}",
                        lane.id,
                        diet.join(", ")
                    ),
                ));
            }
            checked += 1;
        }
    }
    Ok(checked)
}

/// The three evidence checks, in contract order. Any one of them refusing is
/// the whole door refusing: a dossier that fails one check has not been
/// checked, and a partial pass is the "looks checked" failure this verb was
/// written to prevent.
pub(crate) fn check_evidence(d: &Dossier, log: &DispatchLog) -> Result<Counts, Value> {
    if d.lanes.is_empty() {
        return Err(refuse(
            "dossier_lanes_missing",
            &[("section".into(), s("Lanes"))],
            "the dossier records no lane, so there is no proposal to resolve a citation against, no dispatch_id to check the brief digest with, and no reported path to hold to a diet. A run with no lane is refused rather than reported as three checks passed. Record every lane under \"## Lanes\"."
                .to_string(),
        ));
    }
    let citations = check_citations(d)?;
    check_digests(d, log)?;
    let paths = check_read_diet(d)?;
    Ok(Counts { citations, paths })
}

// ─── argv plumbing ──────────────────────────────────────────────────────

struct Ctx {
    root: PathBuf,
    drift: crate::registry::Drift,
}

fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)),
        Roots::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, drift }))
}

fn flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "blind" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "check" => run_check(parse_shape(rest, &["dossier"])?, t0),
        _ => None,
    }
}

/// A refusal is a REFUSAL: the typed object on stdout under `--json`, its
/// `fix` on stderr otherwise, and a non-zero exit either way. `bee blind check
/// && bee decisions log` must not walk past a malformed dossier because the
/// verb answered 0 while saying no.
fn emit_refusal(root: &Path, cmd: &str, use_json: bool, refusal: &Value, t0: Instant) -> ExitCode {
    if use_json {
        println!("{}", jsjson::stringify_pretty(refusal));
    } else {
        let fix = refusal.get("fix").and_then(Value::as_str).unwrap_or("refused");
        let reason = refusal.get("reason").and_then(Value::as_str).unwrap_or("refused");
        eprintln!("bee {cmd}: {reason} — {fix}");
    }
    record_timing(root, cmd, t0, false);
    ExitCode::FAILURE
}

fn run_check(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "blind check";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(path) = flag(&parsed, "dossier") else {
        let msg = format!("bee {cmd}: --dossier is required.");
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    };
    let refusal: Value = match std::fs::read(path) {
        Err(_) => refuse(
            "dossier_unreadable",
            &[("path".into(), s(path))],
            format!(
                "--dossier \"{path}\" could not be read. The path is resolved from the current working directory — check the spelling, or pass an absolute path."
            ),
        ),
        Ok(bytes) => match String::from_utf8(bytes) {
            Err(_) => refuse(
                "dossier_not_utf8",
                &[("path".into(), s(path))],
                format!(
                    "--dossier \"{path}\" is not valid UTF-8. It is refused rather than decoded lossily: a lossy decode would change the very proposal bytes the citation check exists to resolve against. Save the dossier as UTF-8."
                ),
            ),
            Ok(text) => match parse_dossier(&text) {
                Err(mut r) => {
                    // The path rides on every refusal: a convergence run checks
                    // one dossier, but a report that names none is a report the
                    // reader has to guess at.
                    if let Some(map) = r.as_object_mut() {
                        map.insert("path".into(), s(path));
                    }
                    r
                }
                Ok(dossier) => match check_evidence(&dossier, &DispatchLog::read(&ctx.root)) {
                    Err(mut r) => {
                        if let Some(map) = r.as_object_mut() {
                            map.insert("path".into(), s(path));
                        }
                        r
                    }
                    Ok(counts) => {
                        let result = dossier.to_value(&counts);
                        // The counts ride in the line, not only the JSON: a
                        // pass over one lane and one citation is a much
                        // smaller claim than a pass over three of each, and
                        // the reader must be able to tell them apart.
                        let line = format!(
                            "Dossier {} passes: {} lane(s), {} citation(s) and {} reported path(s) checked. Checks run: {}.",
                            dossier.run_id,
                            dossier.lanes.len(),
                            counts.citations,
                            counts.paths,
                            CHECKS_RUN.join(", ")
                        );
                        return Some(emit_success(
                            &ctx.root, cmd, parsed.json, &ctx.drift, &result, &line, t0,
                        ));
                    }
                },
            },
        },
    };
    Some(emit_refusal(&ctx.root, cmd, parsed.json, &refusal, t0))
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::drivers::LEANING_VERDICT_STEMS;

    /// A four-section LaneBrief the dispatch door accepts — the control every
    /// dossier fixture below is built on.
    fn brief() -> String {
        [
            "## Question",
            "Which reader wins when two records claim the same path?",
            "",
            "## Constraints",
            "No new store. The answer stays inside the existing dispatch door.",
            "",
            "## Read diet",
            "- packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs",
            "",
            "## Digest contract",
            "Report the sha256 the dispatch record stamped.",
        ]
        .join("\n")
    }

    /// The digest every lane in the control fixture reports, and the one the
    /// control dispatch log carries for each of their dispatch_ids.
    const FIXTURE_DIGEST: &str = "9c1185a5c5e9fc54612808977ee8f548b2258d31";

    /// A lane's dispatch_id, derived from its id so two lanes never share one.
    /// A shared dispatch_id would make the per-lane log lookup ambiguous, which
    /// is the very ambiguity the chain of custody exists to remove.
    fn dispatch_id_of(id: &str) -> String {
        let n = u32::from(id.bytes().last().unwrap_or(b'a') - b'a') + 1;
        format!("3f1c9a20-0000-4000-8000-{n:012}")
    }

    fn lane(id: &str, proposal: &str) -> String {
        lane_full(id, &dispatch_id_of(id), FIXTURE_DIGEST, DIET_PATH, proposal)
    }

    /// The one path the control brief's read diet declares.
    const DIET_PATH: &str = "packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs";

    const LANE_A_PROPOSAL: &str =
        "The older lease wins on every read, so a stale claim never outranks a live one.";
    const LANE_B_PROPOSAL: &str =
        "A second store keyed by path would answer this without reading the lease at all.";
    const LANE_A_CITATION: &str =
        "lane-a :: The older lease wins on every read, so a stale claim never outranks a live one";

    /// A lane section with every machine field spelled out — the builder the
    /// evidence probes use to break exactly one field at a time.
    fn lane_full(id: &str, dispatch: &str, digest: &str, paths: &str, proposal: &str) -> String {
        format!(
            "### {id}\n\n- dispatch_id: {dispatch}\n- brief_sha256: {digest}\n- role: advisor\n- paths_read: {paths}\n\n```\n{proposal}\n```\n"
        )
    }

    /// A dossier with the control prose and the given lanes and citations.
    fn dossier_of(lanes: &[String], citations: &[String]) -> String {
        format!(
            "# Blind lane run example-0001\n\n## Question\n\n```\n{}\n```\n\n## Lanes\n\n{}\n## Cross-critiques\n\nlane-a read lane-b verbatim and named one missing constraint.\n\n## Chosen\n\nlane-a: the older lease wins.\n\n## Rejected\n\nlane-b: it needs a second store to answer.\n\n## Citations\n\n{}\n\n## Revisit trigger\n\nlease-shape-changes__3f1c9a20\n",
            brief(),
            lanes.join("\n"),
            citations.join("\n"),
        )
    }

    /// The well-formed dossier: every section, in order, with two lanes.
    fn well_formed() -> String {
        dossier_of(
            &[lane("lane-a", LANE_A_PROPOSAL), lane("lane-b", LANE_B_PROPOSAL)],
            &[LANE_A_CITATION.to_string()],
        )
    }

    /// The control dossier with its citation list swapped out.
    fn cited(citations: &[&str]) -> String {
        let lanes = [lane("lane-a", LANE_A_PROPOSAL), lane("lane-b", LANE_B_PROPOSAL)];
        dossier_of(&lanes, &citations.iter().map(|c| (*c).to_string()).collect::<Vec<_>>())
    }

    /// A dispatch log built from `(dispatch_id, brief_sha256)` pairs — the
    /// authoritative record the digest check reads, never the dossier itself.
    fn log_of(entries: &[(&str, Option<&str>)]) -> DispatchLog {
        let text = entries
            .iter()
            .map(|(id, digest)| {
                let mut line = json!({
                    "ts": "2026-08-28T09:00:00.000Z",
                    "source": "prepare",
                    "dispatch_id": id,
                    "kind": "advisor",
                });
                if let Some(d) = digest {
                    line["brief_sha256"] = json!(d);
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        DispatchLog::parse(".bee/logs/dispatch.jsonl", &text)
    }

    /// The log that matches the control fixture: both lanes, one brief.
    fn control_log() -> DispatchLog {
        log_of(&[
            (&dispatch_id_of("lane-a"), Some(FIXTURE_DIGEST)),
            (&dispatch_id_of("lane-b"), Some(FIXTURE_DIGEST)),
        ])
    }

    /// Parse, then run the three evidence checks over the result.
    fn checked(doc: &str, log: &DispatchLog) -> Result<Counts, Value> {
        let d = parse_dossier(doc).unwrap_or_else(|r| panic!("this fixture must parse: {r}"));
        check_evidence(&d, log)
    }

    fn evidence_refusal(doc: &str, log: &DispatchLog) -> Value {
        checked(doc, log).err().expect("this dossier must be refused by an evidence check")
    }

    fn refusal_of(doc: &str) -> Value {
        parse_dossier(doc).expect_err("this dossier must be refused")
    }

    fn reason(v: &Value) -> String {
        v.get("reason").and_then(Value::as_str).unwrap_or_default().to_string()
    }

    fn field(v: &Value, key: &str) -> String {
        v.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
    }

    /// Every refusal at this door is the SAME object shape the dispatch door
    /// returns, and every one of them names its remedy.
    fn assert_refusal_shape(v: &Value) {
        assert_eq!(v.get("ok"), Some(&json!(false)), "{v}");
        assert_eq!(v.get("type"), Some(&json!("refused")), "{v}");
        assert!(!reason(v).is_empty(), "a refusal without a reason: {v}");
        assert!(
            v.get("fix").and_then(Value::as_str).is_some_and(|f| f.len() > 40),
            "a refusal must name its remedy: {v}"
        );
    }

    #[test]
    fn a_well_formed_dossier_passes_and_reports_every_lane() {
        let d = parse_dossier(&well_formed()).expect("the control dossier must pass");
        assert_eq!(d.run_id, "example-0001");
        assert_eq!(d.lanes.len(), 2);
        assert_eq!(d.lanes[0].id, "lane-a");
        assert_eq!(d.lanes[0].role, "advisor");
        assert_eq!(d.lanes[0].brief_sha256.len(), 40);
        assert_eq!(
            d.lanes[0].paths_read,
            vec!["packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs".to_string()]
        );
        assert!(d.lanes[0].proposal.contains("older lease wins"), "{:?}", d.lanes[0].proposal);
        // The brief travels whole, so the digest check has real bytes later.
        assert_eq!(d.brief.trim(), brief().trim());
        assert!(d.section("Citations").contains("lane-a ::"), "{:?}", d.section("Citations"));
    }

    #[test]
    fn a_document_with_no_headings_at_all_is_refused_by_the_missing_title() {
        let v = refusal_of("just some prose, and not a dossier at all\n");
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_title_missing");
        assert!(
            field(&v, "fix").contains("Blind lane run"),
            "the refusal must name the title it wants: {v}"
        );
    }

    #[test]
    fn a_title_without_a_run_id_is_refused() {
        let doc = well_formed().replace("# Blind lane run example-0001", "# Blind lane run");
        let v = refusal_of(&doc);
        assert_eq!(reason(&v), "dossier_title_missing");
    }

    #[test]
    fn every_missing_section_is_refused_by_its_own_name() {
        for want in DOSSIER_SECTIONS {
            let doc = well_formed().replace(&format!("## {want}\n"), "");
            let v = refusal_of(&doc);
            assert_refusal_shape(&v);
            assert_eq!(reason(&v), "dossier_section_missing", "dropping {want}: {v}");
            assert_eq!(field(&v, "section"), want, "dropping {want} named the wrong section: {v}");
        }
    }

    #[test]
    fn an_extra_or_repeated_section_is_refused_by_its_own_name() {
        let extra = well_formed().replace("## Chosen", "## Notes\n\nstray.\n\n## Chosen");
        let v = refusal_of(&extra);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_section_unexpected");
        assert_eq!(field(&v, "section"), "Notes");

        let repeated = well_formed().replace("## Chosen", "## Rejected\n\nagain.\n\n## Chosen");
        let v = refusal_of(&repeated);
        assert_eq!(reason(&v), "dossier_section_unexpected");
        assert_eq!(field(&v, "section"), "Rejected");

        let second_title = well_formed().replace("## Citations", "# Blind lane run other\n\n## Citations");
        let v = refusal_of(&second_title);
        assert_eq!(reason(&v), "dossier_section_unexpected");
        assert_eq!(field(&v, "section"), "Blind lane run other");
    }

    #[test]
    fn a_misordered_section_is_refused_by_the_name_that_stands_out_of_place() {
        // Swap Chosen and Rejected — both present, both known, wrong order.
        let doc = well_formed()
            .replace("## Chosen\n\nlane-a: the older lease wins.", "@@CHOSEN@@")
            .replace(
                "## Rejected\n\nlane-b: it needs a second store to answer.",
                "## Chosen\n\nlane-a: the older lease wins.",
            )
            .replace("@@CHOSEN@@", "## Rejected\n\nlane-b: it needs a second store to answer.");
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_section_out_of_order");
        assert_eq!(field(&v, "section"), "Rejected");
    }

    #[test]
    fn a_lane_missing_any_machine_field_is_refused_by_lane_and_field_name() {
        for f in LANE_FIELDS {
            let doc = well_formed().replacen(&format!("- {f}: "), "- dropped: ", 1);
            let v = refusal_of(&doc);
            assert_refusal_shape(&v);
            assert_eq!(reason(&v), "dossier_lane_field_missing", "dropping {f}: {v}");
            assert_eq!(field(&v, "field"), f, "{v}");
            assert_eq!(field(&v, "lane"), "lane-a", "{v}");
        }
    }

    #[test]
    fn a_field_written_with_no_value_counts_as_absent() {
        let doc = well_formed().replacen(
            &format!("- dispatch_id: {}", dispatch_id_of("lane-a")),
            "- dispatch_id:",
            1,
        );
        let v = refusal_of(&doc);
        assert_eq!(reason(&v), "dossier_lane_field_missing");
        assert_eq!(field(&v, "field"), "dispatch_id");
    }

    #[test]
    fn a_lane_with_no_proposal_block_is_refused_by_lane_name() {
        let doc = well_formed().replace(
            "```\nThe older lease wins on every read, so a stale claim never outranks a live one.\n```\n",
            "",
        );
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_lane_proposal_missing");
        assert_eq!(field(&v, "lane"), "lane-a");
    }

    #[test]
    fn a_lanes_section_with_no_lane_is_refused_by_the_section_name() {
        let doc = format!(
            "# Blind lane run example-0001\n\n## Question\n\n```\n{}\n```\n\n## Lanes\n\nnone yet.\n\n## Cross-critiques\n\n-\n\n## Chosen\n\n-\n\n## Rejected\n\n-\n\n## Citations\n\n-\n\n## Revisit trigger\n\n-\n",
            brief()
        );
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_lanes_missing");
        assert_eq!(field(&v, "section"), "Lanes");
    }

    #[test]
    fn the_same_lane_id_twice_is_refused_by_that_id() {
        let doc = well_formed().replace("### lane-b", "### lane-a");
        let v = refusal_of(&doc);
        assert_eq!(reason(&v), "dossier_lane_repeated");
        assert_eq!(field(&v, "lane"), "lane-a");
    }

    #[test]
    fn a_lane_shaped_heading_outside_the_lanes_section_is_refused() {
        let doc = well_formed().replace("## Chosen", "## Chosen\n\n### lane-c");
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_subsection_unexpected");
        assert_eq!(field(&v, "heading"), "lane-c");
        assert_eq!(field(&v, "section"), "Chosen");
    }

    #[test]
    fn a_question_section_with_no_recorded_brief_is_refused_by_name() {
        let doc = well_formed().replace(&format!("```\n{}\n```", brief()), "see the brief file.");
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_brief_missing");
        assert_eq!(field(&v, "section"), "Question");
    }

    /// D2(a) reaching the convergence door: the recorded brief goes through
    /// the guard the DISPATCH door runs, so the refusal reason is that door's
    /// own — not a second vocabulary invented here.
    #[test]
    fn a_recorded_brief_that_leans_is_refused_by_the_dispatch_doors_own_guard() {
        // The stem is taken from the frozen list rather than typed, so this
        // file never carries a copy of it — see the both-callers test below.
        let stem = LEANING_VERDICT_STEMS[0];
        let leaning = brief().replace(
            "No new store. The answer stays inside the existing dispatch door.",
            &format!("{stem} the second reader."),
        );
        let doc = well_formed().replace(&brief(), &leaning);
        let v = refusal_of(&doc);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "brief_leaning_language", "{v}");
        assert_eq!(field(&v, "phrase"), stem, "{v}");
        assert_eq!(field(&v, "dossier_section"), "Question", "{v}");
    }

    /// Both of the guard's arms reach the dossier, not just the lexical one:
    /// a brief that never had four sections could never have passed the door.
    #[test]
    fn a_recorded_brief_that_lost_a_section_is_refused_by_the_same_shape_arm() {
        let short = brief().replace(
            "## Constraints\nNo new store. The answer stays inside the existing dispatch door.\n\n",
            "",
        );
        let doc = well_formed().replace(&brief(), &short);
        let v = refusal_of(&doc);
        assert_eq!(reason(&v), "brief_section_missing", "{v}");
        assert_eq!(field(&v, "section"), "Constraints", "{v}");
    }

    /// The rule lives at two doors, so ONE test reads both of them. Neither
    /// door may carry its own copy of the stems: a second list is a second
    /// answer, and the two would drift the first time one was tuned.
    #[test]
    fn both_doors_call_the_one_guard_and_neither_carries_its_stems() {
        const BLIND_SOURCE: &str = include_str!("mod.rs");
        const PREPARE_SOURCE: &str = include_str!("../drivers/prepare.rs");

        assert_eq!(LEANING_VERDICT_STEMS.len(), 17, "the frozen list is seventeen stems");
        for (what, src) in [("verbs/blind/mod.rs", BLIND_SOURCE), ("verbs/drivers/prepare.rs", PREPARE_SOURCE)] {
            assert!(src.contains("lint_brief("), "{what} must call the shared guard");
            let lower = src.to_ascii_lowercase();
            for stem in LEANING_VERDICT_STEMS {
                assert!(
                    !lower.contains(stem),
                    "{what} carries its own copy of {stem:?} — the list lives in \
                     verbs/drivers/brief_lint.rs alone"
                );
            }
        }
    }

    /// A proposal is arbitrary prose. If a quoted heading could move a section
    /// boundary, a lane's own text would decide what the record says it is.
    #[test]
    fn a_proposal_that_quotes_headings_cannot_move_the_section_boundaries() {
        let sneaky = "Compare the two records:\n\n## Chosen\n\n### lane-z\n\nThat is prose, not structure.";
        let doc = well_formed().replace(
            "The older lease wins on every read, so a stale claim never outranks a live one.",
            sneaky,
        );
        let d = parse_dossier(&doc).expect("a quoted heading is prose, not a section");
        assert_eq!(d.lanes.len(), 2, "the fake lane heading must not become a lane");
        assert!(d.lanes[0].proposal.contains("## Chosen"), "the quote survives verbatim");
        assert!(d.section("Chosen").contains("the older lease wins"), "{:?}", d.section("Chosen"));
    }

    /// A longer fence closes on a longer fence only, so a proposal may quote
    /// its own fenced code.
    #[test]
    fn a_proposal_may_carry_its_own_shorter_fence() {
        let inner = "Use this call:\n\n```\nbee dispatch prepare --kind advisor\n```\n\nand nothing else.";
        let doc = well_formed().replace(
            "```\nThe older lease wins on every read, so a stale claim never outranks a live one.\n```",
            &format!("````\n{inner}\n````"),
        );
        let d = parse_dossier(&doc).expect("a nested fence is still one proposal");
        assert!(d.lanes[0].proposal.contains("bee dispatch prepare"), "{:?}", d.lanes[0].proposal);
        assert_eq!(d.lanes.len(), 2);
    }

    /// The honest inventory. `checks_run` names every check that stands
    /// behind `ok:true` — no more, and no fewer. The three evidence checks
    /// join it in the SAME change that made them run: a list that named a
    /// check nobody ran, or omitted one that did, is how a dossier comes to
    /// look checked without being checked.
    #[test]
    fn the_pass_result_names_every_check_this_door_ran() {
        let d = parse_dossier(&well_formed()).unwrap();
        let counts = check_evidence(&d, &control_log()).expect("the control passes");
        let v = d.to_value(&counts);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["run_id"], json!("example-0001"));
        assert_eq!(v["lanes"].as_array().map(Vec::len), Some(2));
        let checks: Vec<&str> =
            v["checks_run"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
        assert_eq!(
            checks,
            vec!["sections", "lane_fields", "brief_lint", "citations", "digest_equality", "read_diet"]
        );
        // The counts say how much material the pass covered — a pass over
        // nothing is refused, so a reported count is always at least one.
        assert_eq!(v["citations_checked"], json!(1));
        assert_eq!(v["paths_checked"], json!(2));
    }

    /// The example the registry ships must be the shape this parser accepts —
    /// an example that cannot pass is a doc that teaches a wrong record.
    #[test]
    fn the_shipped_example_dossier_passes_this_door() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .join("docs/history/slp-blind-lanes/blind/example-run.md");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let d = parse_dossier(&text)
            .unwrap_or_else(|r| panic!("the shipped example must pass: {r}"));
        assert_eq!(d.lanes.len(), 2, "the example shows a real 2-lane run");
    }

    // ── CHECK 1 — citations resolve against the lane they name ──────────
    //
    // Four holes, four probes. Plain containment against the concatenated
    // proposal set passes every one of them, which is why none of them is a
    // variation of the others.

    /// HOLE a — a short generic quote sits inside almost any proposal. This
    /// one IS contained in lane-a's bytes, so only the minimum span refuses it.
    #[test]
    fn a_short_generic_citation_is_refused_for_failing_the_minimum_span() {
        let lanes = [
            lane("lane-a", "The answer stays at the dispatch door, so no new machinery is needed."),
            lane("lane-b", LANE_B_PROPOSAL),
        ];
        let doc = dossier_of(&lanes, &["lane-a :: the dispatch door".to_string()]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_too_short", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
        assert!(field(&v, "quote").contains("dispatch door"), "{v}");
    }

    /// HOLE b — a quote in no proposal at all. The plain fabrication.
    #[test]
    fn a_fabricated_citation_found_in_no_proposal_is_refused() {
        let doc = cited(&["lane-a :: A third store settles the tie between two live leases"]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
    }

    /// HOLE c — the negation strip. The quote IS a substring of the proposal
    /// and it is long, but it starts mid-sentence, past the word that reverses
    /// the meaning. Containment says yes; the record would then cite the lane
    /// as saying the opposite of what it said.
    #[test]
    fn a_citation_that_drops_a_negation_and_inverts_the_meaning_is_refused() {
        let proposal = "We should not cache the token on the worker side, because a cached token outlives the lease that granted it.";
        let quote = "cache the token on the worker side, because a cached token outlives the lease that granted it";
        assert!(proposal.contains(quote), "the probe is only about the sentence rule");
        let lanes = [lane("lane-a", proposal), lane("lane-b", LANE_B_PROPOSAL)];
        let doc = dossier_of(&lanes, &[format!("lane-a :: {quote}")]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
        assert!(
            field(&v, "fix").to_ascii_lowercase().contains("sentence"),
            "the refusal must name the sentence rule that caught it: {v}"
        );
    }

    /// HOLE d — misattribution. The text is genuine, in the WRONG lane. This
    /// is the fabrication D4 exists to catch: a dossier that reads as three
    /// lanes agreeing when one lane said it and the citation moved it.
    #[test]
    fn a_citation_attributed_to_one_lane_but_present_only_in_another_is_refused() {
        let doc = cited(&[
            "lane-b :: The older lease wins on every read, so a stale claim never outranks a live one",
        ]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_misattributed", "{v}");
        assert_eq!(field(&v, "lane"), "lane-b", "{v}");
        assert_eq!(field(&v, "found_in"), "lane-a", "the refusal names where it really lives: {v}");
    }

    /// The prohibition itself: never check against the concatenated set. This
    /// quote is two whole sentences that exist only ACROSS the lane boundary,
    /// so it resolves against the joined proposals and against neither lane.
    #[test]
    fn the_citation_check_never_matches_against_the_concatenated_proposal_set() {
        let quote = format!("{LANE_A_PROPOSAL} {LANE_B_PROPOSAL}");
        let joined = format!("{LANE_A_PROPOSAL}\n{LANE_B_PROPOSAL}");
        assert!(
            quote_resolves(&normalize(&joined), &normalize(quote.trim_end_matches('.'))),
            "the probe is only meaningful while the concatenation WOULD match"
        );
        let doc = cited(&[&format!("lane-a :: {quote}")]);
        let v = evidence_refusal(&doc, &control_log());
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// The green probe: same bytes, different whitespace and case. A citation
    /// is transcribed by hand; re-wrapping it is not fabricating it.
    #[test]
    fn a_real_citation_differing_only_in_whitespace_and_case_passes() {
        let doc = cited(&[
            "lane-a ::   The  OLDER lease   wins on every read,  so a stale claim never outranks a LIVE one.",
        ]);
        let counts = checked(&doc, &control_log()).expect("a re-wrapped real quote is not a fake");
        assert_eq!(counts.citations, 1);
    }

    /// The judge's verbatim counterexample quote: 34 characters and 7 words,
    /// so it clears BOTH floors and only the sentence rule can refuse it.
    const GOVERNED_CLAUSE: &str = "cache the token on the worker side";

    /// A proposal whose negation is separated from `GOVERNED_CLAUSE` by `sep`
    /// alone, cited as if the clause were a sentence of its own.
    fn governed_clause_case(sep: &str) -> String {
        let proposal = format!("We should not follow lane-b here{sep} {GOVERNED_CLAUSE}.");
        assert!(
            proposal.contains(GOVERNED_CLAUSE),
            "the probe is only about the sentence rule: {proposal}"
        );
        let lanes = [lane("lane-a", &proposal), lane("lane-b", LANE_B_PROPOSAL)];
        dossier_of(&lanes, &[format!("lane-a :: {GOVERNED_CLAUSE}")])
    }

    /// HOLE e — an abbreviation's dot faking a sentence start. This is the
    /// judge's counterexample against the first fix, verbatim: "i.e." ends in
    /// a dot, so a rule that trusts ANY dot lets the citation begin past the
    /// "should not" that governs the clause. The lane then reads as
    /// recommending exactly what it refused.
    #[test]
    fn a_citation_starting_after_an_i_e_abbreviation_is_refused() {
        let doc = governed_clause_case(", i.e.");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
        assert!(
            field(&v, "fix").to_ascii_lowercase().contains("sentence"),
            "the refusal must name the sentence rule that caught it: {v}"
        );
    }

    /// The same hole through the other everyday abbreviation.
    #[test]
    fn a_citation_starting_after_an_e_g_abbreviation_is_refused() {
        let doc = governed_clause_case(", e.g.");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// The same hole through an ASCII ellipsis: three dots are one mark, and
    /// the last of them is not the end of a sentence.
    #[test]
    fn a_citation_starting_after_an_ascii_ellipsis_is_refused() {
        let doc = governed_clause_case("...");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// HOLE e, THE BRACKETED FORM. "(i.e." is the single most common way the
    /// abbreviation is actually written, and a set matched by byte equality on
    /// the whole space-delimited token never sees it — the leading bracket
    /// alone defeats the lookup. Same strip, same inversion.
    #[test]
    fn a_citation_starting_after_a_bracketed_abbreviation_is_refused() {
        let doc = governed_clause_case(" (i.e.");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// The same defeat through a leading quote mark. One adjacent character is
    /// the whole attack, so the lookup must read the WORD, not the token.
    #[test]
    fn a_citation_starting_after_a_quoted_abbreviation_is_refused() {
        let doc = governed_clause_case(" \"e.g.");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// HOLE f — the enumerator dot. A markdown numbered list under a negated
    /// stem is ordinary design prose: the "must not" governs every item, and
    /// the dot after "1" is a list marker, never a sentence end. Reading it as
    /// one lets the citation start at item 1 and report the lane as
    /// recommending what it forbade.
    #[test]
    fn a_citation_starting_after_a_numeric_enumerator_is_refused() {
        let proposal = format!(
            "We must not do any of the following:\n\n1. {GOVERNED_CLAUSE}.\n2. Skip the lease check."
        );
        let lanes = [lane("lane-a", &proposal), lane("lane-b", LANE_B_PROPOSAL)];
        let doc = dossier_of(&lanes, &[format!("lane-a :: {GOVERNED_CLAUSE}")]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// The lettered spelling of the same list. "a." is an enumerator wherever
    /// "1." is one, and a fix that reads only digits leaves the hole open.
    #[test]
    fn a_citation_starting_after_an_alpha_enumerator_is_refused() {
        let proposal = format!(
            "We must not do any of the following:\n\na. {GOVERNED_CLAUSE}.\nb. Skip the lease check."
        );
        let lanes = [lane("lane-a", &proposal), lane("lane-b", LANE_B_PROPOSAL)];
        let doc = dossier_of(&lanes, &[format!("lane-a :: {GOVERNED_CLAUSE}")]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// The END side of the same hole, mirrored. Here the enumerator dot sits
    /// where the quote STOPS, so a citation can drop the clause that qualifies
    /// it — "only when the lease is dead" — and still look whole.
    #[test]
    fn a_citation_ending_on_an_enumerator_dot_is_refused() {
        let proposal =
            format!("{GOVERNED_CLAUSE} 1. only when the lease is dead and gone.");
        let lanes = [lane("lane-a", &proposal), lane("lane-b", LANE_B_PROPOSAL)];
        let doc = dossier_of(&lanes, &[format!("lane-a :: {GOVERNED_CLAUSE} 1.")]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_unresolved", "{v}");
    }

    /// THE LIMIT, PINNED. Cross-sentence framing passes and is MEANT to: the
    /// quote IS a whole sentence of lane-a's own bytes, and the words that
    /// reverse it live in the sentence BEFORE it. Deciding it explicitly
    /// (carried, not refused) keeps the contract honest — this check reads one
    /// sentence, so a mechanical rule cannot see a negation one sentence back
    /// without refusing honest citations wholesale. This probe exists so the
    /// green is a recorded limit and never mistaken for proof of faithfulness.
    #[test]
    fn a_whole_sentence_quote_framed_by_the_sentence_before_it_still_passes() {
        let proposal = "Never do the following. Cache the token on the worker side.";
        let lanes = [lane("lane-a", proposal), lane("lane-b", LANE_B_PROPOSAL)];
        let doc = dossier_of(&lanes, &[format!("lane-a :: {GOVERNED_CLAUSE}")]);
        let counts = checked(&doc, &control_log())
            .expect("carried by decision: cross-sentence framing is outside the within-sentence contract");
        assert_eq!(counts.citations, 1);
    }

    #[test]
    fn a_citation_naming_a_lane_the_dossier_has_no_section_for_is_refused() {
        let doc = cited(&["lane-z :: The older lease wins on every read, so a stale claim never outranks a live one"]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_lane_unknown", "{v}");
        assert_eq!(field(&v, "lane"), "lane-z", "{v}");
    }

    #[test]
    fn a_citation_line_that_carries_no_lane_id_is_refused_as_malformed() {
        let doc = cited(&["the older lease wins on every read"]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citation_malformed", "{v}");
    }

    /// A zero count is not a pass. A dossier citing nothing has synthesized
    /// its chosen answer out of nothing this door can resolve.
    #[test]
    fn a_dossier_with_no_citation_at_all_refuses_rather_than_reporting_success() {
        let doc = cited(&[]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_citations_empty", "{v}");
    }

    // ── CHECK 2 — the brief digest, read from the authoritative log ──────

    #[test]
    fn a_lane_whose_recorded_digest_differs_from_the_logged_one_is_refused() {
        let other = "0000000000000000000000000000000000000000";
        let log = log_of(&[
            (&dispatch_id_of("lane-a"), Some(other)),
            (&dispatch_id_of("lane-b"), Some(FIXTURE_DIGEST)),
        ]);
        let v = evidence_refusal(&well_formed(), &log);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_brief_digest_mismatch", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
        assert_eq!(field(&v, "recorded"), FIXTURE_DIGEST, "{v}");
        assert_eq!(field(&v, "logged"), other, "{v}");
    }

    /// The dispatch log is FAIL-OPEN at write (`append_prepare_record`: "a log
    /// failure never blocks the payload"). So an absent line is a hole, and a
    /// hole must be NAMED — a silent pass here would let any dossier claim a
    /// dispatch that never happened.
    #[test]
    fn a_dispatch_id_with_no_line_in_the_log_is_refused_by_name() {
        let log = log_of(&[(&dispatch_id_of("lane-a"), Some(FIXTURE_DIGEST))]);
        let v = evidence_refusal(&well_formed(), &log);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_dispatch_unlogged", "{v}");
        assert_eq!(field(&v, "lane"), "lane-b", "{v}");
        assert_eq!(field(&v, "dispatch_id"), dispatch_id_of("lane-b"), "{v}");
        assert!(field(&v, "log").contains("dispatch.jsonl"), "the refusal names the log: {v}");

        // An empty or missing log is the same hole for every lane, never a pass.
        let v = evidence_refusal(&well_formed(), &log_of(&[]));
        assert_eq!(reason(&v), "dossier_dispatch_unlogged", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
    }

    #[test]
    fn a_logged_dispatch_that_records_no_digest_is_refused_rather_than_skipped() {
        let log = log_of(&[
            (&dispatch_id_of("lane-a"), None),
            (&dispatch_id_of("lane-b"), Some(FIXTURE_DIGEST)),
        ]);
        let v = evidence_refusal(&well_formed(), &log);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_brief_digest_unlogged", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
    }

    /// Byte-identical briefs is what makes the lanes comparable (D2(b)). Two
    /// different logged digests mean the brief CHANGED between lane 1 and
    /// lane 3 — the cost decision f0f21142 accepted, detected here rather
    /// than prevented at the door.
    #[test]
    fn logged_digests_that_are_not_all_equal_are_refused_naming_the_divergence() {
        let other = "1111111111111111111111111111111111111111";
        let lanes = [
            lane("lane-a", LANE_A_PROPOSAL),
            lane_full("lane-b", &dispatch_id_of("lane-b"), other, DIET_PATH, LANE_B_PROPOSAL),
        ];
        let doc = dossier_of(&lanes, &[LANE_A_CITATION.to_string()]);
        let log = log_of(&[
            (&dispatch_id_of("lane-a"), Some(FIXTURE_DIGEST)),
            (&dispatch_id_of("lane-b"), Some(other)),
        ]);
        let v = evidence_refusal(&doc, &log);
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_brief_digest_divergent", "{v}");
        assert_eq!(field(&v, "lane"), "lane-b", "{v}");
        assert_eq!(field(&v, "other_lane"), "lane-a", "{v}");
    }

    // ── CHECK 3 — the read diet, and how far it can be trusted ───────────

    #[test]
    fn a_lane_reporting_a_path_outside_the_declared_diet_is_refused() {
        let stray = "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs";
        let lanes = [
            lane("lane-a", LANE_A_PROPOSAL),
            lane_full(
                "lane-b",
                &dispatch_id_of("lane-b"),
                FIXTURE_DIGEST,
                &format!("{DIET_PATH}, {stray}"),
                LANE_B_PROPOSAL,
            ),
        ];
        let doc = dossier_of(&lanes, &[LANE_A_CITATION.to_string()]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_read_diet_breach", "{v}");
        assert_eq!(field(&v, "lane"), "lane-b", "{v}");
        assert_eq!(field(&v, "path"), stray, "{v}");
    }

    /// `.bee/` is where the orchestrator's own leaning lives — the open reason
    /// D1 forces lands in `.bee/decisions.jsonl`. So a `.bee/` read refuses
    /// even when the brief itself listed it: a diet cannot license it.
    #[test]
    fn any_path_naming_dot_bee_is_refused_even_when_the_diet_declares_it() {
        let lanes = [
            lane_full(
                "lane-a",
                &dispatch_id_of("lane-a"),
                FIXTURE_DIGEST,
                ".bee/decisions.jsonl",
                LANE_A_PROPOSAL,
            ),
            lane("lane-b", LANE_B_PROPOSAL),
        ];
        let doc = dossier_of(&lanes, &[LANE_A_CITATION.to_string()]).replace(
            &format!("## Read diet\n- {DIET_PATH}"),
            &format!("## Read diet\n- {DIET_PATH}\n- .bee/decisions.jsonl"),
        );
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_read_diet_bee_path", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
    }

    /// The honesty rule. This check reads the lane's OWN report; checks 1 and
    /// 2 read bytes this door holds. The refusal says so, because a check
    /// whose strength is misread retires a suspicion it never earned.
    #[test]
    fn the_read_diet_refusal_states_that_it_is_self_reported_and_weaker() {
        let lanes = [
            lane_full(
                "lane-a",
                &dispatch_id_of("lane-a"),
                FIXTURE_DIGEST,
                "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
                LANE_A_PROPOSAL,
            ),
            lane("lane-b", LANE_B_PROPOSAL),
        ];
        let doc = dossier_of(&lanes, &[LANE_A_CITATION.to_string()]);
        let v = evidence_refusal(&doc, &control_log());
        assert_eq!(field(&v, "trust"), "self_reported", "{v}");
        let fix = field(&v, "fix").to_ascii_lowercase();
        assert!(fix.contains("self-report"), "{v}");
        assert!(fix.contains("weaker"), "the refusal must not read as strong as the citation check: {v}");
    }

    #[test]
    fn a_lane_reporting_no_path_at_all_refuses_rather_than_checking_nothing() {
        let lanes = [
            lane_full("lane-a", &dispatch_id_of("lane-a"), FIXTURE_DIGEST, ",", LANE_A_PROPOSAL),
            lane("lane-b", LANE_B_PROPOSAL),
        ];
        let doc = dossier_of(&lanes, &[LANE_A_CITATION.to_string()]);
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_read_diet_empty", "{v}");
        assert_eq!(field(&v, "lane"), "lane-a", "{v}");
    }

    #[test]
    fn a_brief_declaring_no_diet_leaves_nothing_to_check_and_refuses() {
        let doc = well_formed().replace(&format!("## Read diet\n- {DIET_PATH}\n"), "## Read diet\n");
        let v = evidence_refusal(&doc, &control_log());
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_read_diet_undeclared", "{v}");
    }

    // ── the whole door ──────────────────────────────────────────────────

    /// Zero lanes is zero evidence. Parsing already refuses it, and so does
    /// the evidence pass — a check that can be handed nothing must never
    /// answer "checked".
    #[test]
    fn an_evidence_pass_over_no_lane_refuses_rather_than_reporting_success() {
        let mut d = parse_dossier(&well_formed()).unwrap();
        d.lanes.clear();
        let v = check_evidence(&d, &control_log()).err().expect("no lane is no evidence");
        assert_refusal_shape(&v);
        assert_eq!(reason(&v), "dossier_lanes_missing", "{v}");
    }

    #[test]
    fn the_control_dossier_passes_all_three_evidence_checks() {
        let counts = checked(&well_formed(), &control_log()).expect("the control must pass");
        assert_eq!(counts.citations, 1);
        assert_eq!(counts.paths, 2, "one reported path per lane");
    }

    /// The shipped example is a worked record, so it must survive the EVIDENCE
    /// checks too — not only the section contract. An example that could not
    /// pass would teach a dossier the door refuses.
    #[test]
    fn the_shipped_example_dossier_passes_every_evidence_check() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .join("docs/history/slp-blind-lanes/blind/example-run.md");
        let text = std::fs::read_to_string(&path).unwrap();
        let d = parse_dossier(&text).unwrap();
        let log = log_of(&d
            .lanes
            .iter()
            .map(|l| (l.dispatch_id.as_str(), Some(l.brief_sha256.as_str())))
            .collect::<Vec<_>>());
        let counts = check_evidence(&d, &log)
            .unwrap_or_else(|r| panic!("the shipped example must pass every check: {r}"));
        assert_eq!(counts.citations, 2, "the example cites both lanes");
        assert_eq!(counts.paths, 2);
    }

    #[test]
    fn the_argv_probe_claims_only_its_own_namespace() {
        let argv = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
        assert!(try_native(&argv(&["triggers", "list"]), Instant::now()).is_none());
        assert!(try_native(&argv(&["blind", "nope"]), Instant::now()).is_none());
        // An unknown flag shape is Node-era conservatism: no output, no claim.
        assert!(try_native(&argv(&["blind", "check", "--wat", "x"]), Instant::now()).is_none());
    }
}
