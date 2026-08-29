# slp-human-up — discovery map

## Destination

A locked, buildable feature set that elevates the human above day-to-day
orchestration: supervisor at the waggledance layer with a delegated-decision
tier, cross-repo spec handoff with dissent rights, herdr ignition, and a
weekly cross-project review — each rung of the escalation ladder typed,
recorded, and door-enforced.

## Notes

Session 2026-08-29: user wants the human elevated above day-to-day
orchestration. Supervisor moves to the waggledance layer (cross-project
cockpit); herdr stays the execution base; paseo is optional transport only.
Cross-project work flows: intake triage → planning lead A interviews peers
(facts/hats) and territory leads (constraints/commitments) → spec delivered
async into the receiving repo → herdr ignites the executing lead → bee chain
as-is → dissent rung back to A → supervisor/human escalation. Everything
recorded; a weekly cross-project review compounds knowledge.

Sources: docs/history/research/demonthorn-deep-dive-vs-bee-slp.md,
docs/history/research/paseo-pi-team-human-elevation.md. Doctrine lineage:
the shipped slp-* clusters (docs/discovery/slp-supervisor-lead-peer/MAP.md).

Build-order sketch (settled direction, exact clustering falls out of the
tickets): (1) spec handoff + ladder rung 1 (cross-repo dissent), (2)
waggledance supervisor (observe/connect/relay only) + weekly review, (3) hat
wave procedure (docs-only). The delegated-decision tier was dropped by
704b691c — human elevation now rides on question quality, queueing, and
per-repo gate_bypass levels. Waggledance-side work lands in the waggledance
repo's own backlog when its cluster shapes.

## Decisions so far

- 2f4bf3b1: supervisor's cross-project home is the waggledance layer; herdr
  stays the base; paseo optional — settled in session, no ticket
- 704b691c (supersedes 83baf03f): supervisor assumed cheap-model — observes,
  connects, packages questions; decides NOTHING; rung 2 = one clear question
  to the human — tickets/001-delegated-decision-boundary.md
- 58796a73: human↔lead direct work is a first-class channel; the supervisor
  is never a mandatory middleman — settled in session, no ticket
- 6f039742: repos fully separate, spec delivered INTO the receiving repo,
  async + correlation id, no cross-repo storage — settled in session, no ticket
- c39ced6c: A is binding arbiter; peers = facts/perspectives (hat wave on big
  draft specs only), leads = binding constraints; spec locks decisions, not
  implementation — settled in session, no ticket
- 5bed1c01: A delivers, receiving repo's gates authorize, herdr ignites cold;
  executing lead holds dissent rights, verdict owed by A — settled in session
- fbf06b0d: ladder completeness rule — typed + recorded + door-enforced per
  rung; unclear → up; repeat offender → human — settled in session, no ticket
- d2701784: intake triage — one-project work takes existing lanes; only
  multi-project/contract work takes the A-spec path; ceremony scales —
  tickets/007-triage-owner.md
- 4dda03e0: every human ask names its target project explicitly; the named
  repo's lead is A by default; one-vs-many is A's planning discovery —
  tickets/007-triage-owner.md
- 8fea3561: cross-repo dissent builds nothing new — B records locally, the
  supervisor's rollup is the only cross-project awareness, replies travel as
  data over the existing transport — tickets/003-cross-repo-dissent-bookkeeping.md
- 28a75c87: weekly cross-project review at waggledance layer, report is a
  print, learnings capture back per-repo — shape: tickets/005-weekly-review-shape.md
- 12deaa34: spec-drop transport = waggledance_dispatch + corr-id-as-PBI-id
  (`pbi add --id`, idempotent), deliberately not herding-ignitable until
  Qualify triage — tickets/002-spec-handoff-transport.md
- 12be1c0b: waggledance supervisor = external tick + ask_state rollup reads +
  per-repo bee-CLI writes + a dedicated cockpit repo for its own records —
  tickets/004-waggledance-supervisor-feasibility.md
- 07328333: hat wave = docs-only procedure over the dispatch door, seam is
  bee-shaping pre-Lock, five advisor dispatches, blue hat = orchestrator —
  tickets/006-hat-wave-seam.md

## Not yet specified

- HUMAN_DECISION_REQUIRED bit on every queued ask and report line
  (agent-suspected — recommended in the pi-team distill, not yet
  user-confirmed)
- Whether the per-repo herding supervisor role stays alongside the
  waggledance-layer supervisor or folds into it (agent-suspected)
- Integration ownership when a multi-repo feature needs final assembly and a
  cross-repo contract proof (agent-suspected)

## Out of scope

- Hard coupling to paseo (user: "không gắn quá chặt với paseo")
- A standing lead↔lead conversation channel (source design has none; bee
  coordinates through stores)
- A V3-style authority-brief parser (bee's hooks/reservations are stronger)
- Any shared cross-repo store or registry, contracts included (user: repos
  stay separate)
