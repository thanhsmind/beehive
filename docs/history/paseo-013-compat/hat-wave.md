# Plan check — hat wave synthesis (paseo-013-compat)

PLAN CHECK
Work: current slice (p13-1 code, p13-2 docs)

Seats: hat-facts-gaps, hat-alternatives, hat-user-impact (opus, advisor kind), 2026-10-27.

STRUCTURE
BLOCKERS (all fixed in plan.md rev 2):
- Coverage: a second Paseo version floor in `herding/run.rs` (two `version_at_least(&out, (0, 10, 3))` sites, three 0.10.3 FIX strings, ~42 fixtures) had no owner — folded into p13-1; new decision D5 (`5ccde9fb`).
- Testable exit: the `paseo_steer` search in the old p13-2/p13-3 verify matched the kept error codes — narrowed to `paseo_steer::|mod paseo_steer|paseo_steer\.(rs|mjs)`.
- Claims: the prose cited a live `--no-wait` run that was never held — run live (claim 4) and recorded.
WARNINGS (applied):
- Permit uses a 5 s timeout, not 15 s; `interrupt` already builds the real CLI lazily behind `Option<&dyn PaseoCli>` — steer copies that pattern (claim 21).
- The fake in paseo.rs is `ScriptedCli`; job_verbs.rs already owns `FakePaseoCli` (claim 22) — reused.
- No test covers the no-config `paseo_steer_unsupported` case today — named as new.
- Doctor fixtures: 13 at 0.10.3 move to 0.12.0; the old-version fixture is 0.10.2 → 0.11.4.
- Doc line 285 ("`paseo send` has no steer flag … WebSocket interface") named in p13-2 and its verify.
- Dash-leading steer text is read as an option — probed live (claims 6-7); `--` separator added, decision D6 (`11719b08`).
- Dropping the provider allow-list touches paseo-pi D5 `b6dd8b33` — decision D7 (`87defaa3`) logged with `touches:b6dd8b33`; idle-worker new-turn behavior documented.
- SMALLER PATH: p13-1 and p13-2 (old) merged into one code cell; docs cell runs in parallel — three cells became two.

CELLS (reviewed: 2)
CRITICAL FLAGS: none open.
MINOR FLAGS: p13-1 — the steer counter on Paseo jobs now always prints 1 (cosmetic, out of scope, recorded). Environment: npm `@getpaseo/cli` on this machine is 0.10.3; bee resolves the bundled 0.13.2 because `herding.paseo.command` is unset — recorded in CONTEXT.md.
CLEAN CELLS: p13-2

SUMMARY: Three seats found one real scope gap (the run.rs floor), one live-verified argv defect (`--`), and a cheaper two-cell shape. All folded into plan.md and three new decisions (D5-D7). No blocker remains.
