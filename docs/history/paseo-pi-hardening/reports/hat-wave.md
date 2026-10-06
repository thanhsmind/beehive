# Plan check — paseo-pi-hardening (hat wave, 2026-10-06)

Seats: hat-facts-gaps (opus), hat-risks (fable), hat-value (agy-flash),
hat-alternatives (opus), hat-user-impact (agy-flash). Synthesis by the leader.

```text
PLAN CHECK
Work: slice 1, cells pph-1..pph-7

STRUCTURE
BLOCKERS:
- coverage / worker self-dispatch: D5 denied bee dispatch for every herded worker, but herdr pane workers may stack their own workers (psd-12) / run.rs caller_is_worker / FIXED: D5 revised (be1699f8) to Paseo-carried workers only.
- key link / supervisor allowlist: enforcement depended on extension discovery in an unset cwd / control_loop.rs:600 spawn seam / FIXED: D9 revised (e2af1dab) with --no-extensions -e <bee-guard>, main-root cwd, refusal when absent; claims 20-22.
- facts / Paseo sentinels: Model "-" and Thinking "auto" would read as mismatches / inspect.ts:134-135 / FIXED: D4 revised (4cb9c644); claim 19.
- facts / Pi --model: provider/model built from a non-pi Paseo provider / paseo.rs:91-92 / FIXED: D9 revised, pi-provider or plain model slot only.
- completeness / silent-idle message: no path from the tick to the outcome message / run.rs:1177, 3428-3430 / FIXED: D2 revised (a8d28361), PollTick flag, immediate TimedOutIdle, message passed to diagnose_giveup_paseo.
WARNINGS (applied):
- D3 ended healthy long tool calls (UpdatedAt moves only on stream events) / D3 revised (c2a5af66) to observe-only.
- pph-1 and pph-2 both rewrote the wait loop / model recording moved into pph-1; pph-2 is herding.rs only.
- second broker timer duplicated the drain's busy/steer logic / one shared helper in result-inbox.ts; one in-flight flag with the heartbeat tick.
- reused inspect state defeated the died debounce / non-inspect ticks report no liveness.
- spawn timeout orphan / failed spawn looks up the job label and names the agent.
- old binary short-circuits an unknown hook to allow / belt checks bee hook --help.
- Pi dispatch tools bypass the shell guard / not registered for a Paseo worker.
- async heartbeat delete races the next start; failed delete hid the orphan; existing leaders kept */5 / D6 revised (976ef5a5).
- doctor orphan check via plain ls is cwd-scoped / per-marker inspect.
- supervisor allowlist needed bee head normalization / applied in pph-3.
- 30 s second-idle constant was not in D2 / replaced by one inspect interval.
WARNINGS (noted, not applied):
- broker claim is not exclusive across several leaders / Out of scope: one leader per repo.

CELLS  (reviewed: 7)
CRITICAL FLAGS: none open after the fixes above.
MINOR FLAGS: pph-3 helper locations corrected (paths.rs, guards.rs).
CLEAN CELLS: pph-5, pph-7

SUMMARY: Five blockers, all fixed in CONTEXT.md and plan.md by six revised
decisions; hat-value found every item material. No blocker is open.
```
