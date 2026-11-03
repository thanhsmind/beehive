# SLP Blind Lanes — Procedure (slices 2–5)

**Feature slug:** slp-blind-lanes-procedure
**Date:** 2026-08-28
**Shaping session:** none — this feature locks NO new decisions
**Scope:** Deep
**Domain types:** RUN | ORGANIZE

## Feature Boundary

The remaining four slices of the closed feature `slp-blind-lanes`: the
cross-critique round and the read diet, the deadlock hand-off and the
structured rejected set, the procedure prose that tells an agent when to open
lanes, and the reviewer/judge checklist material. It ends where the parent's
boundary ended — no standalone agent layer, and no relaxation of the merge,
interlock or permission-posture rules.

What is already shipped and merged (do not rebuild it): the dispatch door's
`--brief-file` arm with its leaning guard and stamped digest, and
`bee blind check` with the dossier section contract and its three evidence
checks.

## Locked Decisions

**This feature locks nothing new.** Its decisions are the parent's, cited and
never reinterpreted. The authoritative text is
`docs/history/slp-blind-lanes/CONTEXT.md` — D1 through D7, with their store
ids. Read that table; it is not restated here, because a second copy is a
second thing to drift.

Two decisions taken during the parent's execution bind this feature just as
hard:

| Store id | What it settles |
|----------|-----------------|
| `f0f21142` | No new store and no new command family. The brief rides the existing dispatch door, the lane-opening reason rides the existing decision log, the dossier document holds every proposal verbatim, and ONE verb checks that document. Anything this feature adds obeys the same rule. |
| `79b5437b` | The citation check claims PROVENANCE, never faithfulness. No prose this feature writes may describe it as an anti-fabrication guarantee, and the cross-sentence framing gap stays a named limit. |

Which locked decision each slice serves:

| Slice | Serves |
|-------|--------|
| 2 — cross-critique and the read diet | D2(b), D2(c), D3 |
| 3 — deadlock hand-off and the rejected set | D2(d), D2(e) |
| 4 — procedure prose | D1, D5, D7, plus the rule that convergence runs the checker green before it logs the decision |
| 5 — reviewer/judge checklist material | D6 |

## Terms

The parent's Terms table governs — LaneBrief, neutrality lint, read diet,
LaneProposal, cross-critique, convergence dossier, deadlock, hat. One
addition, needed only by slice 3:

| Term | Meaning in this feature |
|------|-------------------------|
| Rejected set | The lanes a convergence did NOT choose, each with its reason, carried as a structured field on the convergence decision rather than as free prose |

## Canonical References

- `docs/history/slp-blind-lanes/CONTEXT.md` — the locked decision set (D1–D7)
- `docs/history/slp-blind-lanes/plan.md` — the slice queue this feature
  executes, at "Slice queue" (slices 2, 3, 4, 5)
- `docs/knowledge/areas/advisor-protocol/blind-lanes-and-the-convergence-dossier.md`
  — what shipped, and the Open Gaps section naming exactly this work
- `docs/history/slp-blind-lanes/blind/example-run.md` — the worked dossier
  shape the checker accepts

## Outstanding Questions

### Resolve Before Planning

None. The parent's decisions are locked and the slice queue is written.

### Answered by the parent's plan, restated so planning does not re-open them

- The dossier lives at `docs/history/<feature>/blind/<run-id>.md`.
- The lane-opening reason is one decision-log entry, not a new record type.
- The structured `--rejected` flag is an upgrade of the same field the
  parent's flat reason string already carries, never a competing plan.

## Handoff Note

This CONTEXT.md is a pointer, not a decision set. Planning reads the parent's
locked decisions directly and cites them by their D-ids and store ids. If a
gray area appears that the parent did not settle, it goes back to shaping as a
new decision — it is never resolved inside this plan.
