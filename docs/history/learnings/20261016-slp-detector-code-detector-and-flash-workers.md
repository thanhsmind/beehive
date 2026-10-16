# slp-detector — learnings (2026-10-16)

Feature: `docs/history/slp-detector/` (CONTEXT.md D1–D7, plan.md, hat-wave-synthesis.md).
Merged into main at d70d327; cells det-1..det-4 capped with leader checks and judge records.

## What the feature taught

1. **A code-only detector covers every SLP signal the spec names.** Each of the
   four signals had a deterministic surface once the changed set was defined
   correctly: a diff against the commit recorded at spawn plus untracked files.
   The first draft read `git diff HEAD`; the hat wave caught that workers commit
   mid-job, which would have made self-correction fire on every commit. Source:
   `docs/history/slp-detector/hat-wave-synthesis.md`, decision 32492fc4.
2. **The plan check pays for itself before any code.** Three advisor seats found
   one real blocker and six warnings, all applied as plan edits; none reached a
   worker. Cost: three opus consults, about four minutes wall clock.
3. **A flash code worker can end a round idle with zero edits.** Two
   `paseo-pi-flash` runs on det-2 drafted the whole module in prose and never
   called an edit tool (jobs job-1792109946704, job-1792111467709, both
   `timed_out_idle`). The same model finished det-1 and det-3 when the cell was
   smaller or the brief carried a "write code from the first step" line. Waste:
   two 30-minute rounds. Backlog friction row filed (2026-10-16).
4. **Workers finish the code and miss the last mechanical step.** det-1 and
   det-3 committed and wrote `result-1.json` but never ran `cells finish`
   (ceiling and transport timeouts); the leader capped from the recorded result
   with a named deviation. The existing pattern
   `pattern-20260827-a-dead-worker-has-the-code-and-is-missing-the-last-mechanical-step`
   fired as written.
5. **Two bee defects surfaced** and were filed: `cells unclaim` leaves the claim
   store row so claim and prepare refuse each other (issue 108); `dispatch
   prepare` ignores a cell's `escalate` flag and still returns the herding
   command (second report, same day). Workaround used: `--force-ownership`
   on prepare, then the session model ran the cell inline with
   `--inline-reason` at finish.
6. **The host worktree guard shapes how a leader works.** Inside a
   worktree-isolated session the Claude Code guard refuses shell variables,
   heredocs, the words `git` and `bind` in arguments, and writes outside the
   worktree. Control-plane verbs (`dispatch prepare`, `dispatch wave`,
   `cells finish`) must run from main; file edits must happen inside the
   worktree with the edit tools. The leader alternated EnterWorktree and
   ExitWorktree five times; a herding worker needs no entry at all.

## Divergent lens

- Proof the agent only self-reported: det-1's "four truths driven live" ran on
  a throwaway repo the worker built; the leader did not re-drive it. The full
  suite (4799/0) and the artifact spot-checks stand as the leader's evidence.
- Second-order effect: `.bee/bin/bee` on main is still the 2.53.0 release
  binary, so the Pi heartbeat's `bee detector tick` will log a not-found error
  until the next release installs the binary. Harmless (the call is swallowed
  in `finally`), but the feature is inert for the heartbeat until then.
- Context the leader fetched that the user could have pasted: none.

## Not promoted

- The flash-idle behaviour is a single feature's observation; promote only if
  a second feature sees it.
- The worktree-guard rules belong to the host, not bee doctrine.
