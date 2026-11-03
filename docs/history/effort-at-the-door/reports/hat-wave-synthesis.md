# effort-at-the-door — plan-step hat wave, synthesis

Three seats, one draft of `plan.md`, 2026-10-17. All three returned inside
the ceiling; none dropped. The leader folded every finding into the second
draft of `plan.md` and two new decisions (D4a `5009d4f1`, D10 `8233df26`).

## MANDATE 1 — Structure (facts-gaps)

- BLOCKER, fixed by D4a: the codex guard leg denies any `reasoning_effort`
  different from the configured base, so wave 1 alone would deny every codex
  gather/advisor dispatch. ead-2 now moves the codex leg's expected value to
  the ladder-moved one; deny posture unchanged.
- BLOCKER, fixed by D10: `[bee-cell:]` is emitted only under an approved v2
  plan, so the guard could not read the lane for every cell. The door now
  writes `[bee-kind:]` always and `[bee-lane:]` for cells; the guard reads
  them; no token → allowed unchanged.
- BLOCKER, fixed by D10: the guard cannot tell gather from cell by role, and
  `plan`/`lane-*` roles are dispatched as advisor. Kind rides the token.
- WARNING, fixed: the two verify lines pointed at `.bee/bin/bee`; they now
  build and run `packages/bee-rs/target/release/bee`.
- WARNING, fixed: `derive_dispatch_economics` is a hand-rolled twin; ead-2
  adds the four keys there and extends the twin's own order test.
- WARNING, fixed: the Verdict/repair restructure needed for a composed
  repair is named in ead-2; the lane read points at `read_cell`.
- WARNING, fixed: `spike` is a ladder row at 0; the stale doc line at
  model_guard.rs:479 is deleted in ead-2.
- WARNING, fixed: the prepare "text" pointer did not exist; D7 now lands as
  an `--explain` row. `d6ec1958` is in CONTEXT.md as D9.
- Claims 1–12 all confirmed; eight rows added (13–20).

## SMALLER PATH (alternatives)

- Taken: keep 3 cells / 2 waves; `derive_economics` takes an
  `Option<&EffortChoice>` (D5 literal); ead-3 stays its own cell.
- Taken: the guard recomputes (role+kind alone would mis-repair tiny and
  high-risk cells); the codex leg widened (D4a).
- Taken: reuse `read_cell`; allow unchanged when nothing can be read.

## User impact

- Fixed in plan: no role sets effort today, so the Summary says the feature is
  idle until config sets one, and `team show` says so in its ladder line and
  shows `model/effort` per role.
- Fixed: the prepare line lives in `--explain`, never beside the JSON.
- Fixed: `plan`/`lane-*` as advisor would have been "repaired" wrongly — D10.
- Noted: cost is visible only in economics and the dispatch log; the concept
  names the grep. No new cost notice (D1 says auditable, not announced).
- Open for the user after merge: whether to set `effort` on the native roles
  in this repo's `.bee/config.json`.

## Seats and confidence

facts-gaps 92 · alternatives 90 · user-impact 88. Falsifiers recorded in each
return; none run (each needs the ead-1 build).
