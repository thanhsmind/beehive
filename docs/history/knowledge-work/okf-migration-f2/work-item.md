---
type: bee.work-item
title: okf-migration-f2 — migrate every remaining area into the bundle
description: "Migrate the remaining ten docs/specs areas into docs/knowledge/areas/ under the coverage law, after first making the coverage gate derive its own ground truth from a git-pinned blob."
tags: [okf, knowledge-bundle, migration, coverage, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-migration-f2
  lifecycle: active
  required_context: [areas/doctrine-layer/overview.md, areas/doctrine-layer/placement-and-anchoring.md, areas/hook-runtime/index.md, patterns/20260722-surface-structure-planning-is-reviewed-before-scope-locks.md, patterns/20260722-a-coverage-gate-derives-ground-truth-it-never-compares-two-hand-lists.md]
  decisions: ["F8 (the extractor is shape-aware and self-reporting; pins are content-addressed {commit, path, blob_sha, expected_counts})", "F9 (order by spec SHAPE, not size; shapeless areas get an explicit scheme, never a forced fit)", "F10 (workflow-state is split across multiple cells, never one commit)", "F11 (per-anchor fidelity floor: >=0.60 normalized token overlap against the pinned blob)", "F12 (per-cell drift telemetry, comparability keyed by scheme)", F13 (implement-plan may not retire until Technical Design and Rollback Plan have a home in the plan concept), "D20 (author-then-stub, never annotate in place)", D35 (every numbered source anchor lands in exactly one concept), D37 (pointer stubs carry the full anchor map)]
  sources: [docs/history/okf-migration-f2/CONTEXT.md, docs/history/okf-migration-f2/plan.md, docs/history/okf-migration-f2/reports/advisor-digest-f2.md, docs/history/okf-migration-f2/reports/inventory-sweep.md]
  lane: high-risk
---

# okf-migration-f2 — Migrate every remaining area into the bundle

## Outcome

Every area that lived in `docs/specs/` is a set of typed concepts in `docs/knowledge/areas/`,
each source retired behind a pointer stub carrying its full anchor map, and each migration
guarded by a chain suite that proves no anchor was lost, duplicated, or summarised away.

## Scope

**In:** ten remaining areas, the honest-gate work that had to precede them (content-addressed
pins, shape-aware extraction with unparsed-line reporting, the per-anchor fidelity floor, drift
telemetry), and one new anchor scheme for the single genuinely shapeless area.

**Out (F3):** deleting the pointer stubs, rewiring the skills to read the bundle, migrating the
27 long-form decision records, and moving `okf-profile.md` itself into the bundle it describes.

## Acceptance

- Every area's gate is a chain suite; the full chain is green.
- Coverage per area: every anchor owned exactly once, zero duplicated, zero lost.
- Fidelity per area: minimum overlap at or above the 0.60 floor, achieved without editing a
  concept or lowering a threshold.
- Every pin is content-addressed and verifiable in a shallow clone.
- No source path is ever deleted; every stub resolves the old anchors.

## Chosen Approach

Fix the gate's ground truth first, then migrate ordered by spec **shape** rather than size,
proving the loop on a clean nine-section area before the large and irregular ones. Each area is
one cell, one commit, independently revertible. The largest area is split across cells.

## What went differently than planned

The first plan was revoked before any cell ran: its premise — that the shipped extractor could be
reused as-is — was falsified by measurement. The extractor was blind to bold-wrapped rule ids,
which hid 86 anchors across five areas and made two areas look "shapeless" when they were merely
unreadable. Ordering by size would have put those two first. Both corrections are recorded as
F8/F9, and the pattern behind them is `patterns/20260722-surface-structure-planning-is-reviewed-before-scope-locks.md`.

## Remaining work

`workflow-state` (1464 lines, ~140 anchors, duplicate rule ids `R19`/`R20`/`R21` in source) is the
last area, split by the decomposition locked in okf-foundation D30. The duplicate ids must be
disambiguated in the source before pinning — the precedent is `hook-runtime`'s `R14` repair.
