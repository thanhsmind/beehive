# Knowledge Loop — Context

**Feature slug:** knowledge-loop
**Date:** 2026-08-05
**Shaping session:** complete
**Scope:** Standard
**Domain types:** READ | CALL | ORGANIZE

## Feature Boundary

Make bee's knowledge read path reachable: retrieval and promotion stop requiring a
`bee.work-item` concept that almost no feature has, feature close proposes the knowledge
it earned, and the session preamble's critical-pattern digest ranks by relevance instead
of recency. It ends at the proposal — nothing in this feature writes into
`docs/knowledge/`, and no retirement machinery ships here.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `bee knowledge context --work <id>` and `bee knowledge promote --work <id>` accept a feature slug with NO `bee.work-item` concept. When no work-item concept resolves, both fall back to `docs/history/<slug>/CONTEXT.md` plus `plan.md` (when present) as the anchor text used for relevance ranking and for promote's resolution. The output NAMES which anchor was used. No work-item file is ever auto-created. `unknown_work` remains the typed error only when neither a work-item concept nor any history anchor exists. | Auto-authoring a work-item stub would fabricate `title`/`description` prose, which D10 forbids. `CONTEXT.md` already carries the feature's locked decisions in the feature's own words and already exists for every feature. |
| D2 | `bee close` runs `bee knowledge promote` for the closing feature after the tests door reports green AND after the scribing-debt door passes. It prints a proposal headline into close output and writes the full proposals to `docs/history/<slug>/promote-proposals.md`. SOFT door: it never refuses close; a promote failure degrades to one named warning line. Nothing is written into `docs/knowledge/`. | D38 (`promote` proposes, it never writes) must survive intact. `docs/history/` is outside the bundle, so writing the proposals there is not a bundle write. A second refusing door on close was rejected. |
| D3 | The session-preamble critical-pattern digest ranks by RELEVANCE to the bound feature, reusing the IDF ranker in `verbs/knowledge/context.rs` against the D1 anchor, replacing `budget.rs`'s last-N-rows-in-path-order pick. With no bound feature or no resolvable anchor it falls back to the current recency behaviour and SAYS SO in the header line. | The digest is the only knowledge surface every session actually reads. Re-grading 87 concepts by hand was rejected in favour of ranking within the flag. |
| D4 | `bee.critical` re-grading, `bee.lifecycle` retirement, and the `bee knowledge stale` / `knowledge links` verbs (PBI P68) are OUT of this feature. | Named so planning does not smuggle them in; P68 stays a separate backlog row. |
| D5 | No `docs/knowledge/work/<slug>/` file is created, moved, or deleted by this feature. The two existing work items keep working through the unchanged work-item resolution path — the D1 fallback fires only when that path finds nothing. | Guarantees the change is additive: an existing work item always wins over the fallback. |

### Agent's Discretion

Planning owns: where the fallback anchor resolver lives (a shared helper versus one per
verb), the exact `promote-proposals.md` layout, how the digest header names its ranking
mode, and whether the preamble's ranker call is budget-capped separately from
`knowledge context`.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Anchor | The text a ranker measures relevance against: the `bee.work-item` concept when one resolves, otherwise `docs/history/<slug>/CONTEXT.md` (+ `plan.md`). |
| Join key | The `--work <id>` argument that both `knowledge context` and `knowledge promote` require today, and that only 2 of ~41 features can satisfy. |
| Soft door | A close step that reports and continues; contrast the existing tests door and scribing-debt door, which refuse with exit 1. |

## Specific Ideas And References

- Live measurement taken during shaping: `bee knowledge context --work okf-foundation
  --budget 20000` returns 26 entries, 0 truncated, 55 excluded, `critical_total` 78,
  `zero_signal` 0 — the ranker is healthy; only the join key is missing.
- Bundle census at shaping time: 95 area concepts, 87 patterns, 4 work concepts;
  78 of 87 patterns carry `bee.critical: true`; every concept is `lifecycle: active`.
- Capture queue is DRAINED (0 pending; 25 stubs, 25 flushes, last flush
  2026-08-04T16:05Z). The debt is in promotion and retrieval, not in capture.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/knowledge/context.rs:26` — `FLOOR = 3`, `KEEP = 20`,
  and the IDF relevance ranker plus its budget-reservation phase (`:402-427`). D3 reuses this.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs` — the full proposal builder
  (delivery draft, area bullets, pattern candidates); `writes: []` always.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:653-678` — the scribing-debt door,
  the shape a new close door follows (D2 diverges by being soft).

### Established Patterns

- Typed refusals with a named remedy — `unknown_work` in the knowledge verbs, the
  close doors' exit-1 messages. D1's error path keeps that shape.
- Read the runtime store, never write it: `promote` already reads `.bee/cells/*.json`
  as a permitted read (`context-and-promote.md`). The anchor read follows the same rule.

### Integration Points

- `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:127-157`
  (`bundle_critical_patterns_digest`, the `keep` pick at `:146`) — D3's edit site.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:680-738` — the green path where
  D2's soft door lands, after `auto_archive_on_close`'s preconditions are known.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs` — where `--work` resolution
  is routed for both verbs.

## Canonical References

- `docs/knowledge/areas/okf-profile/context-and-promote.md:147` — D38, "`promote` proposes;
  it never writes". Binding on D2.
- `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md:55` — D10, never invent a
  value that cannot be derived. Binding on D1.
- `docs/backlog.md:67` — PBI P67, the row D2 satisfies.
- `docs/backlog.md:68` — PBI P68, explicitly deferred by D4.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] Does the preamble's ranker call fit the session-start latency budget? — measure the
  IDF pass over the live bundle inside `bundle_critical_patterns_digest`'s call path; the
  fallback to recency is the answer if it does not.
- [ ] Does `promote` on a fallback anchor produce a usable delivery draft when the feature
  has no `bee.areas` list to key area bullets from? — run `promote` against one orphaned
  feature (exec-speed) and read the output.

## Deferred Ideas

- `bee knowledge stale` / `knowledge links` (PBI P68) — separate scope per D4.
- Re-grading the 78 `critical: true` patterns and introducing real `lifecycle`
  transitions — needs a per-concept human call; D3 makes it non-urgent.
- Backfilling `bee.work-item` concepts for the 5 orphaned-scribing features — D1 removes
  the reason to.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
