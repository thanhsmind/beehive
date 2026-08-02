---
type: bee.area
title: "Feedback Digest — Generating and Refreshing a Repository's Own Digest"
description: "How a repository turns its own scattered friction, findings, debt, and lessons into one safe, portable snapshot — as a side effect of closing a feature, on request, or as a count only — and how that snapshot regenerates from scratch, never accumulates."
timestamp: 2026-07-22
bee:
  id: feedback-digest-generation-and-refresh
  lifecycle: active
  areas: [feedback-digest]
  required_context: [areas/feedback-digest/data-model.md]
  decisions: [D1 8cd4c84e, D2 8cd4c84e]
  sources: ["docs/history/evolving-loop/ (cells evolving-1 … evolving-11, capped)", docs/history/evolving-loop/reports/review-slice-a.md, docs/history/evolving-loop/reports/review-slice-b.md, "docs/history/cli-mutations/ (cell cli-mutations-2, capped; walkthrough.md)", "docs/specs/feedback-digest.md#R1", "docs/specs/feedback-digest.md#R10", "docs/specs/feedback-digest.md#R11", "docs/specs/feedback-digest.md#E1", "docs/specs/feedback-digest.md#E2", "docs/specs/feedback-digest.md#E3", "docs/specs/feedback-digest.md#E6", "docs/specs/feedback-digest.md#P2", "docs/specs/feedback-digest.md#P6", "docs/specs/feedback-digest.md#P7"]
  authoritative_for: "feedback-digest: generation and refresh"
---

# Feedback Digest — Generating and Refreshing a Repository's Own Digest

Each repository that runs the workflow accumulates a private record of how the work actually went:
friction that was hit, findings that were filed, debt that was named, cells that blocked, deviations
from plan, and lessons written down at the end.

The feedback digest turns that scattered record into **one safe, portable snapshot per repository**,
so the workflow's own maintainers can learn from real usage without ever reading the participating
projects' code.

Two properties define the area, and everything else follows from them:

1. **Producing a digest costs a project nothing.** It happens as a side effect of closing a feature.
   Nobody schedules it, and a failure to produce it never stops anyone's work.
2. **A digest is the only thing that crosses a repository boundary**, and the reader trusts none of
   it. The maintainers' repository reads other repositories' digests to decide changes to the
   workflow's own instructions — so a digest is treated as hostile input, not as a friendly export.
   Safely reading it is `cross-repo-trust-boundary.md`; what it may contain is `data-model.md`.

## Entry Points & Triggers

The digest is **regenerated, never appended to**. It is a photograph of standing friction, not a
ledger. An append log would count the same friction once for every time it was re-observed and so
corrupt any measure of how often something hurts.

| Trigger | Who fires it | Result |
|---|---|---|
| A feature closes (the lessons record is written) | The closing routine, unprompted, every time | The repository's digest is regenerated from scratch |
| An operator asks for the digest directly | Operator | Same regeneration, written to a chosen location |
| An operator asks for a count only | Operator | Entry and drop counts reported, nothing written |

## Behaviors & Operations

### B1 — Generating a repository's digest

**Triggers:** a feature closes; or an operator asks.

**What is read:** only the workflow's own records inside the repository — its friction and findings
list, its decision log, its work-item records, and its lessons documents. Every read is resolved
through a single gate that refuses any location outside the two workflow-owned areas, after
following every link to its true destination. A link that *appears* to sit inside a permitted area
but truly points elsewhere is refused and reported. A record location that does not exist is
**absent, not forbidden**: it is skipped, counted, and never treated as an error.

**What is emitted:** one entry per record, carrying only the six allowed fields.

**What blocks it:** nothing. A malformed record, an unreadable file, a work item with no execution
trace — each is skipped and counted. Generation cannot fail on bad input.

**What each actor observes:** the operator sees the entry count and the drop count broken down by
reason. The closing routine sees a warning if generation failed, and continues.

**Reproducibility:** generating twice from unchanged records yields an identical digest, except for
the generation moment. Ordering never depends on the order the storage happened to enumerate records.

### B3 — Refreshing the digest when a feature closes

The closing routine regenerates the digest immediately after writing the lessons record, on every
run, without being asked and without anyone mentioning it.

If regeneration fails for any reason — the tool is missing, the command errors, the file cannot be
written — the routine emits **one warning line and continues**. This holds *regardless of whether the
failure is understood*. An unfamiliar error is still only a telemetry failure: the digest is a
read-only side effect that runs after all the feature's work is already finished and recorded, so it
cannot damage the feature.

If the refresh is skipped for any reason, the skip is **stated out loud** in the closing summary. A
silent omission is a violation even when the summary has no obvious place to put it.

## Business Rules

- **R1** (D1) — Producing a digest is a side effect of closing a feature. A failing or absent
  refresh warns and never blocks, delays, or reverses a feature's close. Telemetry never stops
  someone else's work, and this holds even when the failure is not understood.
- **R10** — A digest is a snapshot, regenerated whole. It is never appended to.
- **R11** — Generation never fails on bad input. Malformed, unreadable, and absent records are
  skipped and counted.

## Edge Cases Settled

- **E1** — A repository with no friction, no findings, and no lessons produces a valid, empty
  digest — it does not fail.
- **E2** — A record location that does not exist is *absent*, and absence is not a containment
  violation. A location that exists but cannot be read, or that resolves outside its permitted area,
  still is.
- **E3** — A work item with no execution trace is skipped and counted.
- **E6** — Two generations from unchanged records differ only in the recorded generation moment.

## Pointers (implementation)

- **P2** — Command surface: `the bee binary` (`feedback` group: `digest`,
  `count`, `collect`, `rank`)
- **P6** — Close-time refresh: `skills/bee-capturing/SKILL.md`, "Compound" step 5 (housekeeping digest refresh)
- **P7** — Written artifact: `.bee/feedback-digest.json`
