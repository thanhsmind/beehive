# Knowledge Distill + Trigger — Context

**Feature slug:** knowledge-distill-trigger
**Date:** 2026-08-16
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Turn the knowledge/decision layer from append-and-forget into
maintain-and-fire: stale pointers and unsynced docs block close, deferred
decisions live in a trigger registry that cannot sink, decision reversals
must be declared at write time, and the existing debt is cleaned by the
new mechanisms themselves. It ends at the bee CLI + docs layer — no
change to cells, gates, or worktree mechanics.

## Evidence (audit 2026-08-16, this session)

- `bee knowledge check`: 38 `dangling_source` + 13 `dangling_required_context`
  warnings, ignored — warn-only failed.
- 2051 decisions, 32 formal supersedes (~1.6%); rust-port (D1–D8,
  `.bee/decisions.jsonl:1545-1547`) reversed cli-performance's rejection
  (`.bee/decisions.jsonl:1231`) with zero cross-reference.
- 5 confirmed orphan deferred conditions (e.g. "if budgets still miss",
  "when upstream lands anomalyco/opencode#29638") — nothing watches them.
- Note-accretion: `docs/knowledge/areas/workflow-state/gates.md:143-306` is
  changelog-in-prose; `verify-pipeline/concurrency-and-hermetic-runs.md:93-97`
  keeps a dead pointer under its own erratum;
  `workflow-state/worktree-isolation.md` is `lifecycle: active` while citing
  four JS files the rust-port deleted; `rust-runtime/overview.md` still calls
  live Rust guards "dark".
- Zero cross-feature supersession markers across 110+ `docs/history/*/CONTEXT.md`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Dangling knowledge pointers and unsynced docs in areas a feature touches BLOCK `bee close` — a hard door, like tests. Escape valve: an explicit recorded deferral with reason, never a silent pass. | Warn-only already failed (38+13 warnings ignored). |
| D2 | Deferred decisions get a two-tier trigger registry: machine-checkable conditions auto-fire when due; free-text conditions are registered and surfaced at orient/close for human confirmation. No deferred condition may exist outside the registry. | Pure machine-checkable rejected: real conditions (upstream fixes) are not measurable; pure prose sinks. |
| D3 | Every `bee decisions log` requires a relation declaration: `supersedes <id>` / `touches <id>` / `none`. The system proposes candidates from same area+tags; the writer confirms. After-the-fact scanning is never the mechanism. | 32/2051 formal supersedes proves opt-in fails; user's framing: synthesis at write time, not scanning. |
| D4 | Slice 1 builds the mechanisms; slice 2 runs those same mechanisms to clean the existing debt (38+13 dangling pointers, unmarked reversals, changelog-prose distill of the worst files) inside this same feature. The backfill is the mechanism's proof. | — |

### Agent's Discretion

Exact CLI verb names, flag shapes, registry file format, and which door
(close vs a new doctor check) hosts each guard — planning's call, within
the D1–D3 behaviors. The candidate-suggestion ranking for D3 may reuse
the existing IDF ranker (`verbs/knowledge/context.rs`).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| dangling pointer | A `sources` or `required_context` entry naming a path that does not resolve on disk |
| trigger registry | The persistent store of deferred decisions + their firing conditions (both tiers) |
| distill | Replacing changelog-in-prose accretion with one present-tense current-state description; contradicted lines replaced, never kept alongside |

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `bee knowledge check` — already detects `dangling_source`,
  `dangling_required_context`, `not_canonical`, `invalid_evidence_state`;
  D1 promotes its findings from warning to door.
- `bee decisions supersede` — the formal supersede verb exists; D3 makes
  the relation declaration mandatory at log time.
- `bee close` door architecture (`test-simple`, doors report) — D1 adds a
  door to an existing frame.
- IDF relevance ranker in `verbs/knowledge/context.rs` — D3 candidate
  suggestions.

### Integration Points

- `.bee/decisions.jsonl` schema — D3 adds relation fields.
- `bee orient` / close output — D2 surfacing point for human-tier triggers.
- `docs/knowledge/` frontmatter (`lifecycle`, `sources`) — D1's subjects.

## Canonical References

- `docs/history/knowledge-loop/CONTEXT.md` — D2/D4 there deferred
  `bee knowledge stale` / `knowledge links` as PBI P68; this feature
  revives and absorbs that scope.
- `docs/knowledge/patterns/20260723-an-append-only-learning-artifact-transmits-obsolete-advice.md`
  and `20260722-a-migration-is-not-done-until-its-instructions-are.md` — the
  repo's own prior diagnosis of this failure class.

## Outstanding Questions

### Deferred To Planning

- [ ] Which close door hosts D1 (extend scribing-debt vs new door) — read
  the close driver's door table first.
- [ ] Trigger registry storage: new `.bee/` store vs decision-record
  fields — check projection/rebuild implications.
- [ ] D3 migration: how existing 2051 relation-less decisions read under
  the new required field (grandfather them; only new writes owe relations).
- [ ] Backfill slice batching: 38+13 pointers is multi-cell — split by area.

## Deferred Ideas

- Auto-distill of every accreted knowledge file (beyond the worst files in
  slice 2) — full-bundle distill is its own grooming campaign; backlogged.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and reviewing
use locked decisions for coverage and UAT.
