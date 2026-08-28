// the LaneBrief leaning guard — `dispatch prepare --brief-file`'s door check
//
// Split from prepare.rs so ONE function owns the rule: the door below calls
// it, and a later `bee blind check` re-runs it over a dossier's recorded
// brief, so a convergence built on an unlinted brief refuses too. A rule that
// lives in two places needs one implementation, or the two answers drift.
//
// slp-blind-lanes D2(a) (store 5981246b): "one neutrality-linted LaneBrief,
// the lint enforced at the dispatch door as a lexical prose-guard refusal".
//
// ─── NAMED HONESTLY ────────────────────────────────────────────────────────
//
// This guard refuses LEANING LANGUAGE. It does NOT certify neutrality, and no
// refusal text, doc line or field name here may say it does. A word list
// cannot certify neutrality (the advisor consult's finding), and calling it
// that would convert "unlinted" into "certified" — false confidence at the
// one door the whole feature rests on.
//
// ─── SCOPE, and it is the highest-risk property of the feature ─────────────
//
// The guard reads THE BRIEF BYTES and nothing else. It never sees `--purpose`,
// `--expertise`, the cell record, or any other dispatch text: a false fire on
// those would refuse the advisor consult Gate 3 itself requires
// (`high_risk_advisor_refusal`, verbs/state_group/set_gate.rs) and deadlock
// the high-risk workflow that approves guards. `lint_brief` therefore takes
// one `&str` — the brief — and the door hands it nothing else.
//
// Word-bounded and ASCII case-folded, on the scanners.rs primitives the two
// decisions prose guards already use (`is_word`, `boundary_before`,
// `starts_with_ci`, `ws_run`). There is no regex crate in this workspace and
// no shared word-list constant: `matches_supersession_prose` and
// `matches_deferral_prose` each inline their own literals at the use site,
// and this guard does the same — three refusals behind one list would be one
// list nobody dares to change.
#![allow(unused_imports)]

use super::*;
use crate::verbs::decisions::{boundary_before, is_word, starts_with_ci, ws_run};
use serde_json::{Map, Value};

/// The FROZEN verdict-stem set — seventeen phrases, arm 1.
///
/// Frozen means frozen in both directions: an addition needs its own recorded
/// reason, and it may NEVER be shrunk to make a test pass. Shrinking a guard's
/// list to satisfy its corpus is the guard agreeing with itself
/// (`docs/knowledge/patterns/20260812-a-guard-and-its-tests-are-one-model-so-green-proves-only-that-the-model-agrees-with-itself.md`).
///
/// `the right answer` and `the right approach` were CUT at the re-consult:
/// they fire on neutral interrogative phrasing ("What is the right approach
/// for X?"), which the impersonal stems here have no natural use for.
///
/// Lowercase and single-space separated: `phrase_at` folds case and accepts
/// any whitespace run between the words.
pub(crate) const LEANING_VERDICT_STEMS: [&str; 17] = [
    "i recommend",
    "we recommend",
    "my recommendation",
    "i prefer",
    "we prefer",
    "i lean",
    "leaning toward",
    "leaning towards",
    "the correct answer",
    "the obvious answer",
    "the obvious choice",
    "clearly the best",
    "obviously better",
    "we should pick",
    "we should use",
    "you should pick",
    "you should use",
];

/// The four level-2 sections a LaneBrief must carry, in this order — arm 2.
///
/// This arm carries the real load: leaning is mostly structural, not lexical.
/// The stem arm catches only the lazy leak.
pub(crate) const BRIEF_SECTIONS: [&str; 4] =
    ["Question", "Constraints", "Read diet", "Digest contract"];

/// Does `phrase` (lowercase, single-space separated) sit at `pos`, word-bounded
/// on both edges, with any whitespace run standing in for each space? The same
/// hand-scanned shape `matches_deferral_prose` uses for "for now" / "revisit
/// when".
fn phrase_at(chars: &[char], pos: usize, phrase: &str) -> bool {
    if !boundary_before(chars, pos) {
        return false;
    }
    let mut at = pos;
    for (i, word) in phrase.split(' ').enumerate() {
        if i > 0 {
            let w = ws_run(chars, at);
            if w == 0 {
                return false;
            }
            at += w;
        }
        if !starts_with_ci(chars, at, word) {
            return false;
        }
        at += word.chars().count();
    }
    at == chars.len() || !is_word(chars[at])
}

/// ARM 1 alone, exposed on its own so the zero-false-fire corpus test can run
/// THIS arm over the shipped prompts without the shape arm — which would fire
/// on every one of them, none carrying the four required sections, and force a
/// quiet re-scoping of the guard. Excluding it is deliberate: it is what stops
/// the corpus and the stem list being co-tuned into agreement.
///
/// Returns the first stem that fires, in text order; at one position the
/// longer stem wins, so "leaning towards" reports itself rather than
/// "leaning toward".
pub(crate) fn first_leaning_stem(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        let mut hit: Option<&'static str> = None;
        for stem in LEANING_VERDICT_STEMS {
            if phrase_at(&chars, i, stem)
                && hit.map(|h| stem.len() > h.len()).unwrap_or(true)
            {
                hit = Some(stem);
            }
        }
        if hit.is_some() {
            return hit;
        }
    }
    None
}

/// The trimmed text of a level-2 (`##`) heading line, or `None`. `###` is a
/// sub-heading inside a section, never a section of its own.
fn heading_text(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("##")?;
    if rest.starts_with('#') {
        return None;
    }
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// An enumerated list line: the first non-space character is `-` or `*`, or a
/// digit run followed by `.`.
fn is_enumerated(line: &str) -> bool {
    let t = line.trim_start();
    let mut cs = t.chars();
    match cs.next() {
        Some('-') | Some('*') => true,
        Some(c) if c.is_ascii_digit() => {
            let rest = t.trim_start_matches(|c: char| c.is_ascii_digit());
            rest.starts_with('.')
        }
        _ => false,
    }
}

/// Refuse the brief this dispatch would carry, or accept it.
///
/// `Ok(())` — no leaning language and no leading shape was found. That is
/// NOT a claim that the brief is neutral; see the module banner.
/// `Err(refusal)` — the `{ok:false, type:"refused", reason, …, fix}` shape
/// every other `--brief-file` refusal takes (`brief_refusal`), with a `fix`
/// naming which arm fired, which phrase or section, and what to change.
///
/// No refusal text here uses the word "neutral" in any form, and a probe
/// pins that: the nearest wrong sentence to write is a reassuring one, and
/// a door that mentions neutrality while refusing a phrase reads as a door
/// that certifies it when it stays quiet.
///
/// Arms run stem-first, then shape: the stem refusal quotes a phrase the
/// author can delete outright, which is the cheaper fix of the two.
pub(crate) fn lint_brief(brief: &str) -> Result<(), Value> {
    if let Some(stem) = first_leaning_stem(brief) {
        return Err(brief_refusal(
            "brief_leaning_language",
            &[("phrase".into(), Value::String(stem.to_string()))],
            format!(
                "the brief carries leaning language — the phrase \"{stem}\" states a verdict, and a lane handed a verdict returns it back. Cut that phrase and let the question stand on its own, then re-run the dispatch."
            ),
        ));
    }
    lint_brief_shape(brief)
}

/// ARM 2 — the four sections, in order, and no enumerated candidates under
/// Question.
fn lint_brief_shape(brief: &str) -> Result<(), Value> {
    let headings: Vec<&str> = brief.lines().filter_map(heading_text).collect();
    let expected = BRIEF_SECTIONS.join(", ");

    // EXTRA first (an unknown or repeated heading names ITSELF, which is the
    // most actionable thing to say); then MISSING; then ORDER.
    for (i, found) in headings.iter().enumerate() {
        let known = BRIEF_SECTIONS.iter().any(|s| s.eq_ignore_ascii_case(found));
        let repeated = headings[..i].iter().any(|h| h.eq_ignore_ascii_case(found));
        if known && !repeated {
            continue;
        }
        let why = if repeated {
            format!("the section \"{found}\" appears more than once")
        } else {
            format!("\"{found}\" is not one of them")
        };
        return Err(brief_refusal(
            "brief_section_unexpected",
            &[("section".into(), Value::String((*found).to_string()))],
            format!(
                "the brief must carry exactly four level-2 sections, in this order: {expected}. Here {why}. Remove that section, or fold its content into the one of the four it belongs to."
            ),
        ));
    }
    for want in BRIEF_SECTIONS {
        if !headings.iter().any(|h| h.eq_ignore_ascii_case(want)) {
            return Err(brief_refusal(
                "brief_section_missing",
                &[("section".into(), Value::String(want.to_string()))],
                format!(
                    "the brief has no \"## {want}\" section. A LaneBrief carries exactly four level-2 sections, in this order: {expected}. Add \"## {want}\" in that position and re-run the dispatch."
                ),
            ));
        }
    }
    for (i, want) in BRIEF_SECTIONS.iter().enumerate() {
        let found = headings[i];
        if !found.eq_ignore_ascii_case(want) {
            return Err(brief_refusal(
                "brief_section_out_of_order",
                &[("section".into(), Value::String(found.to_string()))],
                format!(
                    "the brief's sections are out of order: \"{found}\" stands where \"{want}\" is due. The four sections must read in this order: {expected}. Move \"{found}\" back below \"{want}\"."
                ),
            ));
        }
    }

    // The Question section may not ENUMERATE candidate answers: a brief that
    // lists them has already led the witness, and lanes exist to generate the
    // options. Scoped to Question — a read diet is a list by nature.
    let mut in_question = false;
    for line in brief.lines() {
        if let Some(head) = heading_text(line) {
            in_question = head.eq_ignore_ascii_case(BRIEF_SECTIONS[0]);
            continue;
        }
        if in_question && is_enumerated(line) {
            let quoted = line.trim();
            return Err(brief_refusal(
                "brief_question_enumerates_answers",
                &[
                    ("section".into(), Value::String(BRIEF_SECTIONS[0].to_string())),
                    ("line".into(), Value::String(quoted.to_string())),
                ],
                format!(
                    "the Question section enumerates candidate answers — the line \"{quoted}\" starts a list, and a brief that lists the options has already led the witness. Lanes exist to GENERATE the options: state the question as prose, and move any material the lane needs into \"## Constraints\" or \"## Read diet\"."
                ),
            ));
        }
    }
    Ok(())
}
