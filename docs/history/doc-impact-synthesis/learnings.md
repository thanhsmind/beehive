# Learnings — doc-impact-synthesis (2026-08-16)

4 capped cells, plan v1 rejected by plan-check, 1 recorded escape.

- The plan-check pass again paid for itself before a line of code: a live
  probe showed the generated decisions index would have been 2-of-2 sweep
  hits, so v1's impact door blocked 100% of closes with a remedy the user
  is forbidden to perform (fixing a do-not-hand-edit file). Exclusion
  lists are part of a sweep's contract, not an afterthought.
- A door aimed at other people's rot fires first on its own feature: the
  dry-run probe after merge flagged this feature's own unrouted D2/D3/D4
  and 10 deferral-shaped lines — 9 of which were spec prose DESCRIBING
  deferral mechanisms, not deferrals. A word-list matcher over docs needs
  the rephrase-or-register triage a human judgment call makes; the door's
  recorded-deferral escape absorbed the one line frozen inside a
  gate-locked plan.md.
- Structured beats windowed: giving decide events a `feature` field made
  the close-walk exact and killed the unboundable time-window fallback;
  the pre-field decisions stay owned by the backfill/campaign rows, named.
- First promote-proposal review where a bullet survived: the three new
  doors' own spec home (gates.md close section) lagged one cell behind
  the code — merged at review instead of dismissed. The review step
  exists for exactly this case.
- Verify-your-own-refusals held: checking `bee status` after every phase
  set caught nothing this time because the checks were made — the phase-1
  lesson (piped `tail` swallowing typed refusals) applied.

Promoted: nothing beyond the in-feature spec syncs; no pattern candidates
were proposed and none were found by hand.
