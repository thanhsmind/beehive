# Plan check — knowledge-skill-pair

One wave, three seats, one round, on 2026-11-13. All three seats returned.
Seat reports: `.bee/mailbox/job-1794580684873-3653796-1/report-1.md` (facts and gaps),
`.bee/mailbox/job-1794580686404-3654017-1/report-1.md` (alternatives),
`.bee/mailbox/job-1794580687910-3654303-1/report-1.md` (user impact).

## Accepted

| # | Seat | Finding | Where it landed |
|---|------|---------|-----------------|
| 1 | facts and gaps | Claim 10 quoted `five rules applied.`, which counts the reply rules, and the rules cell would have rewritten it | Claim 10 re-anchored; the file left the cell; the action says the hit stays |
| 2 | facts and gaps | The scope sentence in Discovery had no claims row | Claim 13 |
| 3 | facts and gaps, user impact | The herded brief paragraph in `herding/mailbox.rs` never spells `knowledge_used`, so the sweep missed it | Claim 14; anchor and test in ksp-1 |
| 4 | facts and gaps | The count sweep reached a concept description and its generated index | All `docs/knowledge/` edits moved to ksp-3 with an index check |
| 5 | facts and gaps | The two questions CONTEXT.md left to planning had no written answer | Discovery |
| 6 | alternatives | The rules cell's Rust files were a subset of the wrong-answer cell's | Merged into ksp-1; three cells |
| 7 | alternatives | The skills cell did not depend on the rules cell | ksp-2 depends on ksp-1 only |
| 8 | alternatives | The SMALLER PATH line compared shapes that dropped D5 | Rewritten to compare shapes that keep every decision |
| 9 | user impact | The due table named grades and load counts the report does not print | D8(a); claims 15 and 16 |
| 10 | user impact | Loaded-never-used is mostly noise at 9 answered caps against 1359 silent | D8(b) |
| 11 | user impact | Two worker lines disagreed on when to report a wrong concept | D8(c) |
| 12 | user impact | The orientation file had no path | D8(e) |
| 13 | user impact | The report text gave a count with no note | D8(d) |

## Dismissed

- facts and gaps 3, a claims row for the rejected `--deviation` fold — the fold is no longer argued in the plan, so no shape rests on it.

## Not checked by a seat

- Claim 11 (`bee knowledge report`): the read-only guard refused the command for the seat; the leader ran it.
