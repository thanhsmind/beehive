# Hat wave — pi-slp-dispatch-return plan check

Seats: hat-facts-gaps (native, opus), hat-alternatives (native, opus),
hat-user-impact (herding, agy-flash; report at main
`.bee/mailbox/job-1790991453296-2136087-1/report-1.md`). Standard lane, three
default seats; all three returned, one past the 10-minute budget.

## Synthesis

| Finding | Seat | Severity | Leader decision |
|---|---|---|---|
| D1 makes every done herded cell exit through herding-cap-check D1's non-zero refusal (`run.rs:4197`); the plan did not say so | facts-gaps, alternatives | BLOCKER | D5 (`28db2359`): keep the refusal as the leader's cap signal; `run.rs` unchanged; Pi exit notice reworded in psd-2; claims 4, 12, 13 added or reworded |
| Template has no else; native-only text needs its own var, and every declared var must be supplied | facts-gaps, alternatives | WARNING | psd-1 names two vars: `native_bookkeeping` and `herding` |
| "Byte-identical" was proven only by a substring | facts-gaps | WARNING | psd-1 adds a golden `assert_eq!` captured before the template edit |
| Wrapping the whole Contract list duplicates shared rules | alternatives | WARNING | psd-1 wraps only the bookkeeping bullets, Result form and Finish; the trailer rule keeps one home |
| Template edit causes prompt skew until the binary is rebuilt | alternatives | NOTE | plan gains an "After merge" step |
| Nothing makes the leader check the commit exists before capping | facts-gaps | NOTE | the leader completeness check at cap covers it; no cell change |
| CLI-transport cell dispatch keeps the cap order | facts-gaps | NOTE | named in CONTEXT as out of D1's scope |
| No cheaper overall shape | alternatives | NOTE | two disjoint cells stand |
| Every good herded cell raises a Pi error notice (exit 1) | user-impact | BLOCKER | same as row 1: D5, psd-2 rewords the exit notice |
| The planned cap line `cells finish --id <id>` is refused: `--outcome`, `--files`, `--report` are required | user-impact | BLOCKER | psd-2's line names every required flag |
| The worktree cwd self-check ends in a `[BLOCKED]` text token a herded worker must not emit | user-impact | WARNING | psd-1 moves it into a third var, `native_worktree_check`; herding block says blocked work writes a blocked result |
| A summary of only the sha hides what changed | user-impact | WARNING | herding block: one-line summary ending with the sha |
| `claim` is boolean and flagArgs drops non-strings | user-impact | NOTE | already in psd-2 (bare `--claim`) |
| AGENTS.md as a worker input names bee commands | user-impact | NOTE | kept: the wrapper already says to ignore workflow instructions, and AGENTS.md carries code conventions the worker needs |

All 11 original claim rows matched their bytes (facts-gaps audit).
