---
type: bee.area
title: Feedback Digest — Ranking and the Self-Improvement Process
description: "Grouping the collected view by what a title means, scoring each group deterministically, and the gated process that turns the ranked result into a shipped, human-approved improvement of the workflow itself."
timestamp: 2026-07-22
bee:
  id: feedback-digest-ranking-and-self-improvement
  lifecycle: active
  areas: [feedback-digest]
  required_context: [areas/feedback-digest/cross-repo-trust-boundary.md]
  decisions: [0022, c75fed88]
  sources: ["docs/history/evolving-loop/ (cells evolving-1 … evolving-11, capped)", docs/history/evolving-loop/reports/review-slice-a.md, docs/history/evolving-loop/reports/review-slice-b.md, "docs/history/cli-mutations/ (cell cli-mutations-2, capped; walkthrough.md)", "docs/specs/feedback-digest.md#R12", "docs/specs/feedback-digest.md#R13", "docs/specs/feedback-digest.md#R14", "docs/specs/feedback-digest.md#R15", "docs/specs/feedback-digest.md#P3"]
  authoritative_for: "feedback-digest: ranking and self-improvement"
---

# Feedback Digest — Ranking and the Self-Improvement Process

This concept owns what happens after the collected view (`cross-repo-trust-boundary.md`) exists:
grouping it into pressing findings, and — in the maintainers' repository only — turning the
ranking into a shipped, human-approved change to the workflow itself.

## Behaviors & Operations

### B4 — Ranking the collected view

**Triggers:** an operator asks for a ranking; or the self-improvement process (below) starts.

**What it does.** All entries — local and collected — are grouped by what their title *means*, then
each group is scored and the groups are returned most-pressing-first.

- **Grouping ignores the safety wrapping.** A collected entry's title arrives wrapped in
  neutralization marks; the same title recorded locally is bare. Grouping compares an internal
  cleaned form of the title (wrapping removed to a fixed point, the same neutralization cleanups
  applied, case and spacing normalized), so wrapped and bare twins — and even double-wrapped ones —
  land in one group. The stored titles themselves stay wrapped; only the invisible comparison form
  is cleaned.
- **Score** = the group's highest severity × how many entries it holds × how many distinct
  repositories contributed (the local repository counts as one). Ties are broken by earliest first
  observation, then by the comparison form — so the same records always rank in the same order.
- **A group whose entries carry no severity scores at the floor value (one), never at zero** — a
  hole in the data must not bury a group.
- **A group containing any CLOSED-kind entry retires from the ranking (evolving-rank-freshness
  cell erf-1, 2026-08-11).** Retired groups leave the ranked list entirely and are reported in a
  separate `retired` list (representative stored title + entry count) so nothing silently
  vanishes; the text render names the retired count in its header. The retire convention is
  append-only and needs no linkage field: appending a closed-kind row bearing the SAME title as
  the group it retires lands in that group by the normalized-title comparison above and
  tombstones it. Motivation: the loop's own top-ranked item was a P1 fixed and closed days
  earlier — closed rows existed under different titles, so the dead item kept topping the agenda.
  NOTE the payload shape change that rode this: the machine-readable ranking is now an object
  `{ranked, retired}`, no longer a bare array.

**What blocks it:** nothing new — it consumes only the already-validated collected view (B2) and
reads nothing itself.

**What each actor observes:** the operator sees each group's representative title (**still wrapped
exactly as stored**), its score and score components, and where its entries came from. The internal
comparison form exists only to group; it is never meant for display, because displaying it would
undo the neutralization the reader applied (see Open Gaps — today the machine-readable output still
carries it).

### B5 — The self-improvement process

The maintainers' repository can run a gated process that turns the ranked groups into a shipped
improvement of the workflow itself: rank → a human chooses **what** to fix → the fix is built under
the discipline that a failing check exists before any content → the checks pass → a human approves
**the exact change** → publication as a deliberate, named, manual step.

The process refuses to run anywhere but the maintainers' repository, refuses to skip either human
choice, and never publishes on its own. Approval of a plan, a schedule, a standing rule, or a
previous change never counts as approval of the next publication.

## Business Rules

- **R12** (0022) — A neutralization-wrapped title and its bare twin are **the same title** for
  grouping. The comparison uses an internal cleaned form; the stored value keeps its wrapping.
- **R13** (0022) — The internal comparison form is never displayed and never placed where something
  that acts on instructions can read it — showing it would strip the reader's own neutralization.
  (Enforced in the process's instructions today, not yet in the tool — see Open Gaps.)
- **R14** (0022) — Ranking is deterministic: the same collected view always yields the same order.
  Severity × occurrences × distinct contributing repositories, ties broken by earliest first
  observation then the comparison form.
- **R15** (0022, D3, D5) — The self-improvement process runs only in the maintainers' repository,
  only on demand, with two human decisions (what to fix; approve the exact change), and never
  publishes automatically. No standing rule or prior approval transfers.

## Open Gaps

- **Ordering across machines.** Entries are ordered using a comparison that depends on the reading
  machine's language settings, while real record titles are not all in one language. Two machines may
  therefore order the same records differently. Reproducibility is proven only within one machine.
- **The internal comparison form leaks into the machine-readable ranking output** (acknowledged at
  review close, 2026-07-10, decision `c75fed88`): the ranked groups carry the cleaned, unwrapped
  form of every title alongside the wrapped one, so a consumer that dumps the raw output re-exposes
  what the reader neutralized. Keeping it out of a prompt currently rests on the process's written
  instructions, not on the tool. Fix filed in the friction backlog.
- **The grouping's cleanup rules are a hand-copied twin of the neutralizer's** (same acknowledgment):
  if the neutralizer's cleanups are ever extended, wrapped and bare twins silently stop grouping and
  nothing goes red. Fix (one shared cleanup, plus a coupling check) filed in the friction backlog.
- **Accent-form twins do not group.** The same title written with composed vs decomposed accented
  characters (common across editors and platforms, and this corpus is bilingual) produces two groups.
- **A title that is only wrapping, or a title legitimately quoted with the wrapping marks, may
  collapse toward the empty comparison form** and falsely group.
- **The severity floor and the missing-date tie-break are believed but untested** — the behavior is
  read from the implementation; no check pins it.
- **Cross-repository corroboration is real but inert in practice**: measured on the live
  two-repository corpus, no group spans both repositories (titles are in different languages), so
  the distinct-repository factor is one everywhere until two repositories share a friction.

## Pointers (implementation)

- **P3** — The self-improvement process: `docs/handbook/evolving.md`; contract in
  `docs/07-contracts.md`; decision record `docs/decisions/0022-evolving-loop.md`
