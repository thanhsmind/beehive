# Hat wave synthesis — claude-read-only

Five seats ran once on the first draft (2026-10-12): hat-facts-gaps (opus),
hat-risks (fable), hat-value (opus), hat-alternatives (opus), hat-user-impact
(opus). Findings are folded into `CONTEXT.md` (D3 note, D4, D5) and `plan.md`.

## PLAN CHECK

Work: current slice (cro-1)

### STRUCTURE

BLOCKERS (fixed):
- hat-risks: the Claude guard fails open when no binary resolves or an old binary short-circuits the verb. Fixed by D4 (store f84f697c): a read-only claude run fails closed before spawn.

WARNINGS (applied):
- Write tools outside the matcher (Monitor, EnterWorktree, ExitWorktree, mcp__). Fixed by D5 (store 8ea1fa19).
- No status message on the new group; `cd` allowed in read-only.
- `.claude/settings.json` refresh named in the cell (onboard --apply).
- Script name `bee-worker-guard.mjs` named in the action.
- D3 proof rows added (mode A git push; mode B on a Claude supervisor recorded as intended).
- hooks.json dropped from the cell files (no-diff check stays in the tests).
- SMALLER PATH: hat-value proposed folding into write-guard; hat-alternatives showed it is not cheaper (marker short-circuit, shared Claude+Codex group). Kept the new group; the reason is in plan.md Approach.
- Stale-binary hook error in non-herded sessions: accepted and recorded as a known limit.

### CELLS (reviewed: 1)

CRITICAL FLAGS: none open.
MINOR FLAGS: none.
CLEAN CELLS: cro-1

SUMMARY: One blocker (fail-open) is fixed by a pre-spawn check. The seats
split on folding into write-guard; the locked marker short-circuit decides it.
