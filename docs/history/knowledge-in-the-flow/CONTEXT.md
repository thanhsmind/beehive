# Knowledge in the feature flow — Context

**Feature slug:** knowledge-in-the-flow
**Date:** 2026-11-09
**Shaping session:** complete (owner decisions settled upstream in spec drop `docs/discovery/spec-drops/sup-20261109-57a6.md`, PBI sup-20261109-57a6)
**Scope:** Deep
**Domain types:** READ | RUN | ORGANIZE

## Feature Boundary

bee loads one feature pack (knowledge plus the code it anchors) at shaping,
planning and the plan-step hat wave from the same assembler cells use, each
cell commits its own knowledge delta with its code, a writer-role worker
consolidates the feature's knowledge before the merge so code and knowledge
land in main together, close stops leaving promote-proposals.md behind, the
pack loads are measured with a feature-level holdout, and this repo's
existing knowledge debt is drained once through that flow. It ends at this
repo: host repos drain their own debt with the shipped flow after they
upgrade.

## Locked Decisions

These are the owner's decisions from the spec drop, rendered as written.
Planning implements them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Flow, not gate. No new gate, no mandatory knowledge-ref check. Loading and writing knowledge are ordinary steps of the flow. (9dca3714) | No new refusal on any verb. Existing doors (the read-set check of knowledge-docs-in-the-flow D2, the cap sync door of knowledge-freshness-mechanic D4) are not new and stay. |
| D2 | At shaping (context lock) and planning, bee assembles one feature pack = knowledge context (areas, patterns, past decisions) + the code files those concepts anchor (`bee.anchors`). The shaping interview, CONTEXT.md, plan.md load-bearing claims and the hat-wave briefs all use the same pack. Code and knowledge complement each other; neither repeats the other. (09762a2e, supersedes 2b11f025) | Retires "pull only, nothing pushed into briefs" (knowledge-freshness-mechanic D1, knowledge-docs-in-the-flow D5) for the shaping, planning and hat-wave moments. |
| D3 | One assembler: reuse `learned_context_lines`, so shaping, planning and cells read the same pack. (d8407506) | `verbs/drivers/prepare.rs:1075`. |
| D4 | Each cell writes its knowledge delta (area update, a pattern when it hit a pitfall, anchors to the files it changed) in the same commit as its code at cap. Knowledge never trails code. (5eaca955) | Carried by the cell worker brief and recorded on the cap; never a refusal (D1). |
| D5 | Close and merge only consolidate: a writer-role worker dedupes and writes the delivery, and the knowledge lands in main in the same merge as the code. The promote-proposals.md backlog goes away. (89b6a3b9) | With `uat_stop: "close"` the merge comes before close, so the consolidation runs in the worktree before `bee worktree merge`. |
| D6 | No human review step, critical patterns included; fix later like code. (75ef006e) | — |
| D7 | Keep the layer thin: every entry anchors to code and never restates what code shows; the close-time writer trims repetition. (4ef03a29) | — |
| D8 | Keep measurement: record the loaded pack paths in dispatch.jsonl; keep the 10% hash holdout of knowledge-outcome-measure at feature level as well as cell level. (6d239a54) | The spec names it "D4 of knowledge-outcome-measure"; that feature's CONTEXT.md numbers the holdout D2 (decision dafd05a1). Same holdout. |
| D9 | One-time drain of existing debt (proposals, capture stubs, orphaned scribing) through the same flow. (c29c2535) | Measured here 2026-11-09: 45 unapplied proposals, 50 capture stubs, 114 orphaned scribing cells across 33 features. |

### Agent's Discretion

Named guesses where the spec is silent; each is planning's to confirm from the code.

- "Feature-level holdout" (D8): a feature whose slug hashes into the holdout
  bucket gets an empty pack at shaping, planning and the hat wave, with the
  arm recorded; its cells keep their own cell-level arm unchanged.
- "Writes the delivery" (D5): the writer records what knowledge the feature
  changed in `docs/history/<feature>/delivery.md`, inside the merge.
- The verb names, flags and record field names.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| feature pack | The knowledge paths (areas, patterns, decisions) from the one assembler plus the code paths those concepts anchor, never file bodies. |
| knowledge delta | The `docs/knowledge/` changes a cell makes in its own code commit: area update, pitfall pattern, anchors to changed files. |
| consolidation | The writer-role step before merge that dedupes and trims the feature's deltas and writes the delivery. |
| feature arm | `knowledge` or `holdout`, chosen by a hash of the feature slug, recorded on every pack load. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/okf-profile/context-and-promote.md | suspect | The manifest builder, the anchor resolver, promote's propose-never-write contract (B5) that D5 changes at close. |
| docs/knowledge/areas/workflow-state/dispatch.md | suspect | Prepare records, the Learned context block and the cell holdout arm. |
| docs/knowledge/areas/workflow-state/deferred-work-queue.md | suspect | The promote stub and deferred record close enqueues today. |
| docs/knowledge/areas/workflow-state/gates.md | suspect | Close doors and the scribing-run stamp. |
| docs/knowledge/areas/okf-profile/critical-bar.md | suspect | Critical pattern selection inside the pack. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | `learned_context_lines` (:1075), `bundle_learned_lines` (:962), `knowledge_arm` (:1118), `cell_learned_context` (:1124) — the one assembler and the cell holdout. |
| `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` | Close writes promote-proposals.md and enqueues the stub and deferred record (:3323-3353, :770, :798). |
| `packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs` | The merge steps; no knowledge step today. |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` | Cell-arm outcomes from dispatch.jsonl prepare rows (:340, :374). |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs` | `build_promotion`, the proposal the consolidation reads. |

## Specific Ideas And References

- Evidence from waggledance and 11 host repos on bee 2.88.0 (spec drop):
  leader pull rate 0.002 in waggledance and 0 in 9 of 11 repos; 0
  `knowledge_arm` rows in four host repos; 114 promote-proposals files in
  waggledance; 46 of 226 commits since 2026-10-01 touch docs/knowledge.
- This repo, 2026-11-09: `bee knowledge report` shows 20 knowledge-arm and
  1 holdout-arm cells, 1287 closed cells with no recorded arm (older than the
  holdout).

## Canonical References

- `docs/discovery/spec-drops/sup-20261109-57a6.md` — the spec drop.
- `docs/history/knowledge-outcome-measure/CONTEXT.md` — the cell holdout (D2).
- `docs/history/knowledge-freshness-mechanic/CONTEXT.md` — anchors, the cap sync door (D4), pull-only (D1, superseded here for shaping and planning).
- `docs/history/knowledge-docs-in-the-flow/CONTEXT.md` — the read set (D2), kept.

## Outstanding Questions

<!-- bee:not-a-deferral: template section names; the questions are planning's to answer from the code, not promises to act later -->
### Resolve Before Planning

- (none)

### Deferred To Planning

- [ ] Where the hat-wave dispatch picks up the pack: the advisor prompt for `hat-*` roles, or the `--brief-file` the leader passes.
- [ ] How the consolidation step is reached in the flow without a refusal: a planned writer cell, a merge report line, or both.
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template prose describing what planning reads, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, the read set, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
