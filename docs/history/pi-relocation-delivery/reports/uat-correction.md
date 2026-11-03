# Correction to the live acceptance verdict (2026-09-16)

The acceptance leader's verdict
(`/tmp/bee-verify/evidence/20260915-224413-891129/uat-verdict.md`) marks step 5
PASS and concludes that "the result launched before relocation reached the
session after relocation". That conclusion does not hold: the evidence it rests
on was produced by the operator's own probe, not by the carry under test.

## Timeline, from file mtimes and the session transcript

| UTC | Fact |
|---|---|
| 01:47:04 | Detached job `job-1789523224362-2419582-1` launched under OLD token `01a0a7e4…`; marker written to `.bee/result-inbox/01a0a7e4…/` |
| 01:47:30 | Session replaced; new session `01a0a7e5…` starts in `repo--wt--uat-carry` |
| 01:47:36 | The job finishes (`result-1.json`, status `done`) |
| 01:47–01:55:55 | The marker stays under the OLD token, never renamed to `.processing`; the new session's transcript holds no injection header |
| ~01:57 | Operator replaces the worktree's `.pi/extensions/bee-guard.ts` (84089 bytes, pre-fix) with the fixed belt (90560 bytes) |
| 01:58:01 | **Operator copies the marker into the NEW session's own token folder** as a probe |
| 01:58:27 | The injection appears in the new session |

## Why the PASS is wrong

1. Until ~01:57 the relocated session ran the worktree's OWN belt copy, which
   predates this feature and holds no carry logic. Nothing could have carried
   the token during the window the verdict cites.
2. The injection at 01:58:27 followed the operator's probe at 01:58:01 by 26
   seconds, and the probe placed the marker under the session's own token — the
   ordinary path that worked before this feature existed.
3. The carried folder was never drained: the marker under `01a0a7e4…` was still
   unclaimed at 01:55:55, eight minutes after the job had finished.

## What the run did establish

- The transition records the carry pair: `.bee/relocation-carry.json` holds
  `{"01a0a7e5…": ["01a0a7e4…"]}`.
- The drain is alive in a relocated session for its OWN token (the probe was
  injected within about half a minute).
- **A real defect (prd-5):** the belt calls `bee cells rebind-session` with a
  worktree cwd, and that verb refuses inside a granted worktree. Reproduced
  directly — from the worktree it exits 1 ("refused inside a granted feature
  worktree"), from the main checkout the same call exits 0. The leader's pane
  carried the matching warning.
- **A second finding (backlog):** a feature worktree carries its own belt copy,
  so a belt change that is not committed and synced is invisible to a session
  that relocates into it.

## Status

The carry half of this feature is UNPROVEN live. It is proven only by the
contract test, which drives the real belt under node. A clean re-run is owed
after prd-5 lands, with the fixed belt present in both the sandbox main checkout
and its worktree, and with no operator-placed markers.
