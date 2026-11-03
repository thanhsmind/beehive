# Proactive Leader Intake — Context

**Feature slug:** proactive-leader-intake
**Date:** 2026-08-30
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

The leader becomes proactive after spec clarification: on a human
request it clarifies the spec as today, and once the spec is clear
enough it opens the hat wave itself to build the implementation plan —
the wave absorbs the internal advisor consult and the plan-checker,
while the user-invoked independent review stays untouched. The law
lands in bee-shaping, bee-planning, the hat-wave procedure home
(bee-hive gates-and-delegation), and AGENTS; it ends before any change
to bee-reviewing or Gate 3.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Store ids in parentheses are the decision-log events.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The wave runs at the PLAN step, not at raw intake: the leader clarifies the spec first (interview/scout as today); once the spec is clear enough, the leader proactively opens the hat wave to build the implementation plan, and the synthesized answers feed the plan. (store: supersession of 8fb1e0da by the 2026-08-30 refinement) | The clarified spec is the draft the hats anchor on — no D7 collision with "hats critique one draft". |
| D2 | The big-or-vague threshold holds: a clear/tiny ask keeps today's fast path with no wave; big, vague, or high-risk work gets the wave. The unit is once per feature, never per message. (store: 8fb1e0da trigger law, timing refined by D1) | Five dispatches on a typo fix is the named ceremony-capture failure. |
| D3 | Seat count: 3 seats by default (hat-facts-gaps, hat-alternatives, hat-user-impact); all 5 seats on high-risk work. (store: 423e1664) | — |
| D4 | The wave absorbs the INTERNAL consults only: the high-risk advisor gate consult and the plan-checker run as the hat wave from now on. bee-reviewing and Gate 3 stay untouched and user-invoked. (store: 2026-08-30 scope-of-duty decision) | Two consult surfaces collapse into one; the review law (agents-review-user-invoked) is out of scope. |
| D5 | The discretionary pre-Lock spec-critique wave is KEPT as today — absorb, never retire. Two windows, two jobs: pre-Lock critiques a drafted big spec; the plan-step wave builds the plan. (store: 98ac20a1) | Intake/plan hats cannot critique a spec draft; retiring the window leaves big specs uncritiqued. |
| D6 | Headless/Qualify: the wave runs unattended and its questions land as Outstanding Questions on the record — never blocking, never self-answering, never simulating the interview. (store: f73d6c49) | — |
| D7 | Communication: during the wave the user sees ONE plain state line, no hat vocabulary; the wave's output reaches the user as one leader voice; findings are filtered against the request text before anything surfaces. (from the intake wave's user-impact digest, accepted in interview) | — |

### Agent's Discretion

- Exact prompt shape per hat seat, budget/timeout numbers, and where
  the wave record file lives — within the recorded laws above.
- Whether the alternatives seat's SMALLER PATH question and
  bee-planning's inline SMALLER PATH mandate merge into one home
  (duplication named by the wave; planning picks the home).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| hat wave | Parallel silent dispatch of the configured `hat-*` seats, one open question each, leader-synthesized; procedure home is bee-hive gates-and-delegation ("Hat wave"). |
| plan step | The moment shaping has locked the spec and bee-planning drafts the work shape — the wave's new firing point. |
| internal consults | The advisor gate consult and the plan-checker — leader-called checks, as opposed to the user-invoked review. |

## Specific Ideas And References

- The intake hat wave dogfood run for this very feature:
  docs/history/proactive-leader-intake/hat-wave-intake.md — five-seat
  digests and the synthesis that fed this interview.
- Direction decision b348489e (user, 2026-08-30) — the wave-at-intake
  direction this shaping refined into D1.

## Existing Code Context

### Integration Points

Path correction after plan review (no decision changed): the editable
source is `skills/**`, NEVER `.claude/skills/**` — the vendored
`.claude` tree is rewritten by `bee dev regen`/onboard.

- `skills/bee-hive/references/gates-and-delegation.md` — "Hat wave"
  section (lines 179-230) is the single procedure home (decision
  07328333); the new plan-step law amends it here, pointers elsewhere.
  Lines 56 and 133 also name the plan-checker/consult law.
- `skills/bee-planning/SKILL.md` — the plan step the wave now opens;
  carries the inline SMALLER PATH mandate (duplication note).
- `skills/bee-shaping/SKILL.md` — carries the one-line hat-wave
  trigger pointer; the discretionary tail-check wording updates.
- `packages/bee/AGENTS.block.md` — the SOURCE of the AGENTS.md
  BEE:START/END block; edit here, then `bee dev regen` re-renders
  AGENTS.md. A direct AGENTS.md edit inside the block is destroyed.
- `docs/knowledge/areas/advisor-protocol/triggers.md` — authoritative
  for consult triggers; the high-risk pre-gate consult law rewrites
  here, not as a stub.
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` —
  `SEAT_ROLES` constant of record for the `hat-*` seats.

## Canonical References

- docs/history/proactive-leader-intake/hat-wave-intake.md — wave record.
- Decision store events: b348489e (direction), 322d9d54 (wave-open
  reason), 8fb1e0da, 423e1664, 98ac20a1, f73d6c49, plus the two
  2026-08-30 refinement events (plan-step timing; scope of duty).

## Outstanding Questions

<!-- bee:not-a-deferral: every item below was answered in plan.md ("Plan-step law details") and shipped in cells pli-1..pli-4; this section is the historical record of the handoff, not a promise to act later -->
### Resolved In Planning (was: Deferred To Planning)

- [x] Per-wave wall-clock and token ceiling, and the partial-return
  rule when hit — set in plan.md: 10-minute ceiling, dropped seats
  named, partial synthesis never blocks the gate.
- [x] Seat quorum when hats are null/unconfigured — set in plan.md:
  no hard quorum; degradation named in the record; doctor advisory
  stays the config nag.
- [x] Idempotence on resume/compaction — set in plan.md: the recorded
  advisor-ref is the once-per-feature carrier.
- [x] Gate-bypass `full`/`total` reading of wave questions — set in
  plan.md: recorded as plan Open Questions, nothing new stops.
- [x] The one home for the SMALLER PATH question — set in plan.md:
  bee-planning's inline check; the hat-alternatives seat cites it.
<!-- /bee:not-a-deferral -->

## Declined Ideas

<!-- bee:not-a-deferral: recorded as declined, not promised — a future owner starts its own shaping -->
- Hats as the user-invoked review's engine (retiring Gate 3's separate
  flow) — user declined for this feature; would supersede
  agents-review-user-invoked and needs its own shaping.
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
read the locked decisions and integration points; every open question
above was resolved in plan.md before the gate. Planning's Gate 2 shape
stage and reviewing use locked decisions for coverage and UAT.
