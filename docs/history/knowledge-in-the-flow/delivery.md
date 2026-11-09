---
type: bee.delivery
title: knowledge-in-the-flow — delivery
description: "Delivery record for feature knowledge-in-the-flow: feature pack assembler, cell knowledge delta, and worktree consolidation before merge."
timestamp: 2026-11-09
bee:
  id: knowledge-in-the-flow-delivery
  lifecycle: active
  areas: [okf-profile, workflow-state]
  required_context: [docs/history/knowledge-in-the-flow/CONTEXT.md, docs/history/knowledge-in-the-flow/plan.md]
  sources: [docs/history/knowledge-in-the-flow/CONTEXT.md, docs/history/knowledge-in-the-flow/plan.md]
---

# knowledge-in-the-flow — Delivery

## Concepts changed

- **docs/knowledge/areas/okf-profile/context-and-promote.md** — Documents the feature pack assembler (`learned_context_lines` code anchors), the `bee knowledge pack` verb, and consolidation replacing close-time proposal generation (D2, D3, D5, D7).
- **docs/knowledge/areas/workflow-state/dispatch.md** — Documents feature-level holdout hashing, pack rows, and advisor pack delivery in dispatch logging (D2, D8).
- **docs/knowledge/areas/workflow-state/deferred-work-queue.md** — Documents that close no longer enqueues promote-proposal records into the deferred work queue (D5).
- **docs/knowledge/areas/okf-profile/critical-bar.md** — Documents feature-level outcomes by arm and pack command pull-rate tracking in knowledge report (D8).
- **docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md** — Documents cell-level knowledge_delta recording on cap and finish (D4).
- **docs/knowledge/areas/okf-profile/concept-model-and-authoring.md** — Synced during debt drain to reflect current active concept authoring model (D9).
- **docs/knowledge/areas/workflow-state/gates.md** — Synced during debt drain to record feature start and finish gate operations (D9).

## What shipped

- **kitf-1** — Assembled one feature pack with `learned_context_lines` code anchors and loaded it with `bee knowledge pack` (commit b8b65b828).
- **kitf-2** — Gave hat-wave and advisor briefs the feature pack and logged feature arm in dispatch records (commit c28c5c490).
- **kitf-3** — Compared features with and without the pack in `bee knowledge report` and counted pack in pull rate (commit 8794b2917).
- **kitf-4** — Consolidated knowledge before merge with `bee knowledge consolidate` and stopped writing proposals at close (commit aa41de11e).
- **kitf-5** — Recorded `trace.knowledge_delta` on every cap and finish and printed delta in completion text (commit becfaccd6).
- **kitf-6** — Taught shaping, planning, workers, and capturing the pack and deltas, and regenerated skill trees (commit d600e3a55).
- **kitf-8a** — Drained 45 unapplied promote proposals into knowledge bundle and recorded drain ledger (commit ee827c864).
- **kitf-8b** — Flushed 50 pending capture stubs and recorded drain ledger (commit 0eec59068).
- **kitf-8c** — Drained orphaned scribing debt across 33 features into drain ledger (commit 205714cdb).
- **kitf-9** — Proved knowledge in the flow end to end in a throwaway repo (commit 6b33ca6f9).
- **kitf-7** — Consolidated feature knowledge, synced area concepts, and wrote delivery record (commit pending).
