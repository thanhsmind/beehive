---
type: bee.area
title: "Rust Runtime — measuring text: characters for what people read, code units for what a foreign contract counts"
description: "Why this runtime measures string length two different ways on purpose, which one every display and log cap uses, the two narrow places that keep the other one and what each of them is pinned to, and how the seven scattered copies of that logic became one module with a single rule per caller."
timestamp: 2026-08-06
bee:
  id: rust-runtime-text-measurement-and-the-two-counting-units
  lifecycle: active
  areas: [rust-runtime]
  required_context: [areas/rust-runtime/overview.md]
  decisions: [js-parity-cleanup D3 (helpers consolidate to one module and truncation goes character-based; the code-unit comparator survives as a named exception with its real reproduction rationale), "bc2e2d44 (judge pass over the consolidation: the question-heading guard had been converted with everything else and had to go back, and the surviving exception's stated rationale was inverted)"]
  sources: ["js-parity-cleanup cell jp-5 (seven duplicated helpers plus three inline copies folded into one module; trace .bee/cells/jp-5.json, 2026-08-04 — 1004 passed, 0 failed)", "js-parity-cleanup cell jp-9 (heading guard restored to code units, exception rationale corrected; trace .bee/cells/jp-9.json, 2026-08-04 — 1006 passed, 0 failed)", docs/history/js-parity-cleanup/CONTEXT.md, "packages/bee-rs/crates/bee/src/textutil.rs (the one module: char_len, truncate_chars_head, truncate_chars_tail, utf16_len, code_unit_cmp, js_default_sort)"]
  authoritative_for: "rust-runtime: how string length is counted, where each counting unit applies, and the pinned exceptions"
---

# Rust Runtime — measuring text: characters for what people read, code units for what a foreign contract counts

There are two ways to ask how long a string is, and they disagree exactly on the
characters that live outside the basic range — an emoji, most scripts beyond the
common ones. Counting *characters* answers what a person sees. Counting *code
units* answers what a particular older encoding stores, and it counts those same
characters as two. Neither is right in general; each is right for one kind of
promise. This runtime makes both available in one place and lets a caller pick
deliberately, because the previous arrangement — seven copies of the same
helpers scattered across the tree — meant callers picked by accident.

## Behaviors & Operations

**Characters are the default, and every human-facing cap uses them
(js-parity-cleanup D3, 2026-08-04).** Trigger: any place a value is shortened for
a person to read — a failure excerpt, a decision line, a backlog field. What
happens: the cap counts characters, and truncation takes whole characters from
the head or the tail. A character is therefore never split in half, and a limit
of five hundred means five hundred things a reader can see rather than a number
that shrinks silently when the text stops being plain. What each actor observes:
for ordinary text nothing changed at all; for text carrying characters outside
the basic range the caps became honest, which is the only case where the old and
new numbers differ.

**Two places keep the other unit, and each is pinned to something outside this
runtime's control.** The first is the guard on a question's chip-label heading:
its limit belongs to an external validator that counts code units, so measuring
characters there would let requests through that the validator then rejects —
the guard exists to satisfy that contract, so it must count the way the contract
counts. The second is a sort comparator that orders by code unit: two artefacts
already stamped and stored in the repository reproduce byte for byte only under
that order, so changing it would silently invalidate them. Both sites carry the
reason at the point of use, in terms of what they are pinned to, and neither is
described as a language-compatibility habit — that framing was what made the
distinction invisible in the first place.

**A stated rationale is only as good as the artefact it names.** The comparator's
exception originally claimed it kept the release manifest's own path list
reproducible. A judge pass proved the opposite: that list is ordered by a locale
comparator, and the file has a test asserting the code-unit order would *not*
reproduce it. The real constraints are elsewhere — a fingerprint feeding stored
digest sidecars, and the ordering of one diff report's two lists. The exception
survived; the reason attached to it did not, and was replaced with the one the
tests actually pin.

## Business Rules

- R8 — Text shortened for a person to read is measured and cut in characters, so
  a truncation never splits a character; the counting unit is chosen at the call
  site and never inherited by accident (js-parity-cleanup D3, cell jp-5,
  2026-08-04).
- R9 — Code-unit counting survives in exactly two places, each pinned to
  something this runtime does not own: the question-heading guard, which must
  agree with an external validator's own measure, and the comparator whose order
  already-stamped artefacts reproduce under. Each site states, at the point of
  use, what it is pinned to — never a language-parity rationale (js-parity-cleanup
  D3 as corrected by jp-9, judge finding bc2e2d44, 2026-08-04).
- R10 — An exception's written rationale is verified against the artefact it
  claims to protect before it is trusted; a rationale contradicted by the code's
  own tests is replaced, and the exception is re-justified or removed
  (jp-9, 2026-08-04).

## Edge Cases Settled

- Text made only of plain characters measures identically under both units, which
  is why the consolidation could change the rule everywhere and still leave every
  ordinary output byte-identical.
- The heading guard's automatic repair is narrower than its check: it rewrites
  plain-ASCII headings only, because cutting a paired character to match the
  external validator's own cut has never been proven — see
  `areas/hook-runtime/write-guard-request-shapes.md` B28/R27 and its Open Gap.

## Pointers (implementation)

- One module, one rule per caller: `packages/bee-rs/crates/bee/src/textutil.rs` —
  `char_len`, `truncate_chars_head`, `truncate_chars_tail` (the default path),
  `utf16_len` (the heading guard's single caller, named in its doc comment),
  `code_unit_cmp` and `js_default_sort` (the reproduction exception). It replaced
  seven duplicated helpers and three inline copies across eighteen files.
- The pinned reproduction constraints: `manifest_fingerprint` in
  `devtools/skill_trees.rs` (feeding stored sha256 sidecar digests) and the
  `diff.missing`/`diff.added` ordering in `release_manifest.rs`, locked by
  `changed_uses_locale_order_while_missing_uses_code_units`. The manifest's own
  path list is ordered by `sort_by_locale`, proven by
  `code_unit_sort_would_not_reproduce_the_manifest`.
- The heading guard's call site: `hooks/write_guard/detectors.rs:159`.
- Evidence: traces `.bee/cells/jp-5.json` (1004 passed) and `.bee/cells/jp-9.json`
  (1006 passed), both 2026-08-04; the correction originates in judge decision
  bc2e2d44.
