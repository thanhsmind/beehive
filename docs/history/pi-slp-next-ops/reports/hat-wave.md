# Hat wave — pi-slp-next-ops plan check

Seats: hat-facts-gaps (native, opus), hat-alternatives (native, opus),
hat-user-impact (herding, agy-flash; report at main
`.bee/mailbox/job-1791000781137-2787968-1/report-1.md`). All three returned.
All 13 original claim rows matched their bytes (facts-gaps audit).

## Synthesis

| Finding | Seat | Severity | Leader decision |
|---|---|---|---|
| The hint keeps four lines; a run line would drop gate pending or triggers due | facts-gaps, user-impact | BLOCKER | nop-2 raises the cap to five; claim 14 |
| The dedup hash ignores the command, so a new run line never re-injects | facts-gaps, user-impact | BLOCKER | nop-2 adds the command to the hash only when non-null |
| A literal `<runtime>` placeholder is not runnable (shell redirect, value check) | user-impact | BLOCKER | the caller's runtime from `locate_caller()`; unknown gives null; claim 18 |
| orient has no hook payload; session id must come from the environment | facts-gaps, alternatives | WARNING | `env_session_id()` and `session_preamble::state::resolve_pipeline`; claim 15 |
| A broken lane binding must not fall back to the default record | facts-gaps | WARNING | orient blocker and null command |
| The wayfinding branch was missing from the command list | facts-gaps | WARNING | kept: `bee discovery list --json` |
| The hook holds no cells, handoff or grant facts | facts-gaps, alternatives | WARNING | `next_operation` takes a plain-facts struct; the hook reads cells and grant once |
| run_from main vs AGENTS.md "move into the worktree before dispatch" | facts-gaps, user-impact | WARNING | kept main: every named command is a control-plane verb that refuses inside a granted worktree; herding workers take cwd from the payload |
| Plain-text orient omits the command | user-impact | WARNING | nop-1 prints a run line |
| Five hat seats marked required on a standard lane | alternatives | NOTE | role plan now marks hat-risks and hat-value not-applicable |
| orient and status now differ for a bound session | facts-gaps | NOTE | CONTEXT names why: status is the repo-wide view |
| Pi reads the delta every turn; worktree enter's instruction | facts-gaps | NOTE | claims 16, 17 |
| nop-1 and nop-2 could merge | alternatives | NOTE | kept split now that the interface is a plain-facts struct |
| Route block.command is merge while the notice says enter | user-impact | NOTE | kept: the command is the step after the work, the notice the step before; text only |
