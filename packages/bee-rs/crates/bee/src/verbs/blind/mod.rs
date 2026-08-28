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
//     even though the door it bypassed never saw it.
//
// The three EVIDENCE checks — citations resolving against the proposals they
// cite, one brief digest across every lane, and the read diet — are NOT here
// yet, and the result object says so in `checks_run` rather than implying a
// completeness it does not have. Parsing is written so they can be added
// without re-reading the document: `Dossier` hands them the lane bytes, the
// brief bytes and each section's body already separated.
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
/// can see that citations were never looked at.
const CHECKS_RUN: [&str; 3] = ["sections", "lane_fields", "brief_lint"];

// ─── the parsed document ────────────────────────────────────────────────

/// One `### <lane-id>` section: its machine fields plus its verbatim proposal.
///
/// `allow(dead_code)`: `proposal` and `paths_read` are parsed and PINNED by the
/// tests here, and read for real by the three evidence checks that land next
/// (citations against the lane's own bytes, one brief digest across the lanes,
/// the read diet). Parsing them here rather than later is what keeps the
/// document read once, by one contract.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub(crate) fn section(&self, name: &str) -> &str {
        self.sections
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, body)| body.as_str())
            .unwrap_or("")
    }

    fn to_value(&self) -> Value {
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
                Ok(dossier) => {
                    let result = dossier.to_value();
                    let line = format!(
                        "Dossier {} passes the section contract, {} lane section(s) and the brief lint. Checks run: {}.",
                        dossier.run_id,
                        dossier.lanes.len(),
                        CHECKS_RUN.join(", ")
                    );
                    return Some(emit_success(
                        &ctx.root, cmd, parsed.json, &ctx.drift, &result, &line, t0,
                    ));
                }
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

    fn lane(id: &str, proposal: &str) -> String {
        format!(
            "### {id}\n\n- dispatch_id: 3f1c9a20-0000-4000-8000-00000000000{}\n- brief_sha256: 9c1185a5c5e9fc54612808977ee8f548b2258d31\n- role: advisor\n- paths_read: packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs\n\n```\n{proposal}\n```\n",
            id.len() % 10
        )
    }

    /// The well-formed dossier: every section, in order, with two lanes.
    fn well_formed() -> String {
        format!(
            "# Blind lane run example-0001\n\n## Question\n\n```\n{}\n```\n\n## Lanes\n\n{}\n{}\n## Cross-critiques\n\nlane-a read lane-b verbatim and named one missing constraint.\n\n## Chosen\n\nlane-a: the older lease wins.\n\n## Rejected\n\nlane-b: it needs a second store to answer.\n\n## Citations\n\nlane-a :: the older lease wins on every read\n\n## Revisit trigger\n\nlease-shape-changes__3f1c9a20\n",
            brief(),
            lane("lane-a", "The older lease wins on every read, so a stale claim never outranks a live one."),
            lane("lane-b", "A second store keyed by path would answer this without reading the lease at all."),
        )
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
            "- dispatch_id: 3f1c9a20-0000-4000-8000-000000000006",
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

    /// The honest inventory. This door does NOT check citations, digest
    /// equality or the read diet, and the result must not let a reader think
    /// it did — those are the next cell's, and a silent `ok:true` is exactly
    /// how a dossier comes to look checked without being checked.
    #[test]
    fn the_pass_result_names_only_the_checks_this_door_ran() {
        let d = parse_dossier(&well_formed()).unwrap();
        let v = d.to_value();
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["run_id"], json!("example-0001"));
        assert_eq!(v["lanes"].as_array().map(Vec::len), Some(2));
        let checks: Vec<&str> =
            v["checks_run"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
        assert_eq!(checks, vec!["sections", "lane_fields", "brief_lint"]);
        for later in ["citations", "digest_equality", "read_diet"] {
            assert!(!checks.contains(&later), "{later} is not checked here yet");
        }
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

    #[test]
    fn the_argv_probe_claims_only_its_own_namespace() {
        let argv = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
        assert!(try_native(&argv(&["triggers", "list"]), Instant::now()).is_none());
        assert!(try_native(&argv(&["blind", "nope"]), Instant::now()).is_none());
        // An unknown flag shape is Node-era conservatism: no output, no claim.
        assert!(try_native(&argv(&["blind", "check", "--wat", "x"]), Instant::now()).is_none());
    }
}
