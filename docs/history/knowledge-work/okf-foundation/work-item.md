---
type: bee.work-item
title: okf-foundation — Bee OKF Profile foundation
description: "Replace bee's document model with an OKF v0.1 bundle: docs/knowledge/, a validator, an index generator, a budget-aware context consumer, and the full migration loop proven end-to-end on one area."
tags: [okf, knowledge-bundle, migration, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-foundation
  lifecycle: active
  required_context: [areas/advisor-protocol/overview.md, patterns/20260715-shipping-a-lib-file-means-shipping-the-manifest.md, patterns/20260716-a-tolerant-regression-net-frozen-green-before-the.md, patterns/20260712-cross-cell-contracts-and-census-carriers-are-plan.md]
  decisions: [D17-D38 active set (docs/history/okf-foundation/CONTEXT.md), "D29 (F1 proof area: docs/specs/advisor-protocol.md)", D30 (workflow-state.md decomposition locked as F2 input), D34 (ships list + guard propagation — ledger/manifest/plugin trees/session-close hook), "D35 (coverage report law: every numbered source anchor lands in exactly one concept)", D37 (pointer-stub anchor map — citations rewired in the same cell as the stub)]
  sources: [docs/history/okf-foundation/CONTEXT.md, docs/history/okf-foundation/plan.md]
  lane: high-risk
---

# okf-foundation — Bee OKF Profile Foundation

## Outcome

Replace bee's document model with an OKF v0.1 bundle governed by a **Bee OKF Profile**: a new
`docs/knowledge/` bundle holding only curated current truth, a validator, an index generator, a
budget-aware context consumer, and the full migration loop proven end-to-end on one area —
`docs/specs/advisor-protocol.md` — plus this feature's own work item and the critical patterns, so
the consumer has real data to resolve on day one. Legacy trees are retired by migration behind
pointer stubs, never annotated in place. `docs/specs/workflow-state.md` is **not** migrated here;
its decomposition map is locked as F2's input (D30).

## Scope

- Bundle root is `docs/knowledge/`: `index.md`, `log.md`, `areas/<area>/`, `features/`,
  `work/<id>/`, `decisions/`, `patterns/`, `runbooks/` (D17). Legacy trees — `docs/specs/`,
  `docs/history/`, `docs/REFs/`, root-level `docs/*.md` — stay outside and are retired area-by-area.
- Vocabulary closed at nine types, slug-cased (D18): `bee.area`, `bee.feature`,
  `bee.work-item`, `bee.plan`, `bee.delivery`, `bee.decision`, `bee.pattern`, `bee.runbook`,
  `bee.evidence`. "Pitfall" is `bee.pattern` with `bee.polarity: pitfall` (D18).
- Concept frontmatter: `type` (OKF-required) + `title`/`description`/`tags`/`timestamp`
  (+`resource`) + a nested `bee:` map carrying `id`, `lifecycle`, `areas`, `required_context`,
  `decisions`, `sources`, and per-type fields (D19, corrected by D32: an id is never computed from
  a path; the path is derived from the id).
- Migration authors new concepts and retires the source via a pointer stub carrying an anchor map;
  legacy files are **not deleted** in this feature (D20, D37).
- A concept is any non-reserved `.md` inside `docs/knowledge/` (D23) — nothing outside the bundle
  is checked or carries a frontmatter obligation.
- Scope includes the consumer (`bee knowledge context`, D27); the full ships list is D34.
- F1's proof area is `docs/specs/advisor-protocol.md` (D29, 202 lines, same nine-section BA
  template as `workflow-state.md`). `workflow-state.md`'s locked nine-concept decomposition
  (D30) is F2's input, not built here.
- `.bee/*.json(l)` stays authoritative for runtime state; the bundle is the knowledge layer and is
  never a write path into it (D2). Reads are permitted.

## Acceptance

Condensed from `docs/history/okf-foundation/plan.md`'s slice exit states:

- **S1** — `bee knowledge check --json` exits 0 on the skeleton; full chain green.
- **S2** — `knowledge check` joins `packages/bee-rs/crates/bee/src/verbs/test_runner.rs` (formerly scripts/run_verify.mjs) as a chain suite.
- **S3** — `index`/`list` generators: regenerating the bundle's indexes changes zero bytes;
  a stale index is detected by `index --check`.
- **S4** — Data: `advisor-protocol` migrated (coverage report D35, stub + anchor map D37),
  `critical-patterns.md` migrated into `patterns/`, canonical templates authored, and this
  feature's own `work/okf-foundation/` work item exists. Coverage report 100%, 0 duplicated,
  0 lost; zero dangling anchors; chain green.
- **S5** — `bee knowledge context --work <id> --budget <tokens>` returns a manifest a cold
  session can act on.
- **S6** — Startup bridge (D38): a fresh session on this repo loads `okf-foundation` from the
  manifest, ≤6 files.
- **S7** — `bee knowledge promote --work <id>` (D38) proposes concept diffs from cell traces and
  the delivery record — proposals only, never a silent truth-write.

## Decisions

Active set D17-D38 (`docs/history/okf-foundation/CONTEXT.md`), most load-bearing for this work
item: D2 (bundle never a runtime write path), D17 (bundle root), D18 (closed vocabulary), D19/D32
(field set, id/path direction), D20/D37 (migration-retires-source, pointer stub + anchor map),
D22/D34 (consumer in scope; the full ships list, including guard propagation — ledger, manifest,
plugin trees, session-close hook), D23 (concept = non-reserved `.md` inside the bundle), D27
(`context` verb contract), D29/D30 (proof area vs. `workflow-state.md` deferral), D35 (coverage
report law).

## Chosen Approach

Format core first with the chain kept green at every slice boundary (the ledger self-onboard rides
the same cell that adds `knowledge.mjs`); generators next; then data (one migrated area plus this
feature's own work item and the critical patterns); the consumer proven against that real data;
then the two loop-closing slices D38 ordered ahead of wide migration (startup bridge, promote v1).
Rejected: a big-bang retrofit of `docs/` in place (593 files, three frontmatter schemas);
`workflow-state.md` as the proof area (1464 lines, ~700 lines homeless under a heading split); a
report-only chain entry (precedent miscited, withdrawn in D22).
