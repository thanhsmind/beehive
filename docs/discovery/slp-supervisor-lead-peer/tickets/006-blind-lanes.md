---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

When does bee open 2–3 BLIND design sessions, and how does
convergence land? Sub-questions: which decisions qualify (gate-risk
rung of the ladder? shaping's hard gray areas?), how isolation is
enforced (no lane sees another or the orchestrator's leaning before
cross-critique), how the convergence record maps onto `bee decisions
log` (chosen + rejected[] + revisit trigger), and what deadlock hands
the user (spec: the dossier, never a coin flip).

## Answer

User half (D 9cffdfb5): the agent opens 2–3 blind lanes on ITS OWN
judgment when a decision is high-stakes AND ambiguous, logging the
reason at open time; the user may also order lanes directly; deadlock
always hands the user the dossier. Mechanism half (D 5981246b): lanes
are a procedure over the existing dispatch door — one
neutrality-linted LaneBrief (prose-guard-style lint at the door), 2–3
parallel advisor-kind dispatches with a byte-identical brief and an
explicit read diet (advisor dispatches are already isolated by
construction), cross-critique as a second advisor round, convergence
as a dossier doc + one decisions-log entry with a registered revisit
trigger. Blind lanes never run as cell-kind (learned_context leaks).
Heterogeneous lane models moved to Out of scope (breaks the one-name
advisor slot, decision 4faf1de9). Findings:
docs/history/research/slp-blind-lanes-surfaces.md.

Post-close shipped shape (feature slp-blind-lanes, 2026-08-28).
f0f21142: no new store and no new command family — the LaneBrief rides
the existing dispatch door, the lane-opening reason rides the existing
decision log, the dossier document itself holds every lane proposal
verbatim, and ONE new verb runs the citation, brief-digest and
read-diet checks over that document. 79b5437b: the citation check
claims PROVENANCE, never faithfulness — it proves a quoted span is a
whole sentence of the named lane's own bytes, and cross-sentence
framing is a recorded limit rather than a caught fault. The brief lint
is a leaning guard, not a neutrality proof.
