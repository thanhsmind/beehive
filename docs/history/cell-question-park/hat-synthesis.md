# Hat wave synthesis — cell-question-park

Seats: hat-facts-gaps (two runs, conf 85 and 88), hat-alternatives (conf 78),
hat-user-impact (conf 72). Raw findings: `.bee/tmp/hat-wave/cell-question-park.md`
in the main checkout.

## Applied to plan.md

| Finding | Seat | Change |
|---|---|---|
| A dependent of a parked cell lands in wave 1 (`classify` reads parked as pending) | facts-gaps | cqp-1 owns schedule.rs and adds a `parked` arm; the "no scheduler edit" prohibition is replaced |
| "blocked = swept claim" is false | facts-gaps, alternatives | plan Rejected line and CONTEXT.md Why text corrected (narrative only, no decision changed) |
| Answer text never reaches the next brief | facts-gaps, user-impact | park record carries the answer; new cqp-4 renders it in prepare.rs |
| Park must release claim, reservations and trace.worker | facts-gaps, user-impact | cqp-1 action and truths |
| [BLOCKED] clashes with the rescue ladder | user-impact | new token [PARKED]; cqp-3 adds it to the token list and the ladder |
| Every refusal names the remedy | user-impact | cqp-1, cqp-2, cqp-4 truths and actions |
| Status readers mishandle parked | facts-gaps | new cqp-4 (record.rs, merge_ready.rs, build.rs, state_sync.rs, orient.rs, policy.rs) |
| `bee cells reopen` bypasses D2 | facts-gaps | cqp-1 owns handlers_close.rs; reopen refuses a parked cell |
| Tag `answer` would change taxonomy.json | facts-gaps | answer uses `cells` + `slp`, as dissent does |
| One trace.park array instead of two fields | alternatives | adopted |
| collect_needs_you has only the root | facts-gaps | cqp-2 reads `list_cells(root, None, Some("parked"))` |
| cqp-2 tests in close.rs own tests | facts-gaps | cqp-2 action |
| registry_payload.json is hand-edited | facts-gaps | cqp-1 action |
| Attempts row and ownership unstated | facts-gaps | park adds no attempts row and uses the claim-ownership guard; answer needs no ownership |

## Not applied, with reason

- Knowledge and verify-map sync for the new door: the leader does it at
  capture time; it is not a cell.
- Record who answered (`--by`): the decision log already records the actor.
- No locked decision changed, so no supersede decision is needed.
