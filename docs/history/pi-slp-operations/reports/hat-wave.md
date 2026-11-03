# Hat wave — pi-slp-operations plan check

Seats: hat-facts-gaps (native, opus), hat-alternatives (native, opus),
hat-user-impact (herding, agy-flash; report at main
`.bee/mailbox/job-1790996088528-2417533-1/report-1.md`). All three returned.
All 9 original claim rows matched their bytes (facts-gaps audit).

## Synthesis

| Finding | Seat | Severity | Leader decision |
|---|---|---|---|
| CLAIMED-by-caller fix `bee dispatch prepare --cell <id>` is not runnable: `--runtime`, `--kind`, `--worker` are required | facts-gaps, alternatives, user-impact | BLOCKER | rfx-2 names the full form with placeholders for unheld values; claims 13, 14 added |
| Deployment prepare fix lacked `--runtime` and `--kind`; the first consumed site does not hold the feature | facts-gaps, user-impact | WARNING | rfx-1 names the full form, fills held values, placeholders for the rest, checked against prepare help |
| Fix commands asserted only in prose | facts-gaps | WARNING | `ran` rows 13-16 and 18 added |
| Worktree root is the raw `root` parameter, not resolved | facts-gaps | WARNING | rfx-3 canonicalizes `root`; claim 12 added |
| A `.bee/` main path would steer a hand edit of bee state | facts-gaps | WARNING | D3 amended (`aa4db1c5`): `.bee/` paths name `bee --help --json` |
| Telling an agent to merge another worktree risks merging in-flight sibling work | user-impact | WARNING | D3 amended: leave the file, write in the caller's own worktree root |
| Two product docs pin the CLAIMED text | facts-gaps | WARNING | rfx-2 owns both docs |
| `bee state lanes` refuses inside a worktree | alternatives | WARNING | rfx-1's fix says "run from main" |
| Several fixes offered two commands | alternatives, user-impact | WARNING | one command per refusal, listed in Approach |
| Preview-mismatch value echo is beyond D1 and breaks on multi-line fields | alternatives, user-impact | WARNING | dropped; rfx-4 names the plan file only |
| The expired-authorization and missing-feature fixes were uncovered | facts-gaps | NOTE | added to rfx-1; claims 10, 11 |
| Sessionless claims must not read as "you hold it" | facts-gaps | NOTE | rfx-2: equal only when both sessions are present |
| `bee gate --preview` may need `--lane` in multi-lane setups | user-impact | NOTE | kept: a bound session resolves its lane |
| Shape: four disjoint cells is the cheapest honest split | alternatives | NOTE | stands |
