# escalate-off-disarm — CONTEXT

**Route.** class `bugfix` · lane `standard` · flag `covered-contract-change` ·
4 product files. Asked: the user said "tiếp tục" to my named next action —
fix the r3 delta review's top P2.

## What was asked

`bee cells escalate --off` reported success but disarmed nothing on a
migrated ceiling cell. Both r3 reviewers found it independently: `--off`
removed only the `escalate` key (`handlers_close.rs:1186-1190`), and
`cell_is_escalated` (`validate.rs:108-113`) then re-answered true off the
legacy `tier: "ceiling"` string — which nothing in the tree ever clears. The
cell kept burning the 40% ration, kept dispatching on the session model, and
the preamble kept counting it. This hit exactly the 20 live cells the D9
backfill converted. Second facet: a later `backfill-roles` run re-derived the
flag from the tier string and re-armed even a hypothetically effective disarm.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | An explicit `escalate: false` is the third spelling of the escalation answer and means **disarmed** — it overrides the legacy tier read in every reader: `cell_is_escalated`, the preamble's `map_is_escalated` copy, and the backfill's derivation, which treats a **present** flag key of either value as "this record is done". A bare `tier: "ceiling"` with no flag key stays escalated, so pre-backfill stores read unchanged. | Clearing the tier string would rewrite history D4 deliberately keeps; an explicit false preserves the string, preserves compat, and records the operator's act on the cell itself. |
| D2 | The disarm writes `escalate: false` only when the cell carries the legacy tier spelling; every other cell keeps today's key-removal. | Writing false everywhere would make absent and false two spellings of one state on ordinary cells — the exact two-answers defect this feature family keeps removing. |

## What was done

Red-first: two tests written against unfixed code and shown failing — the
disarm test (`left: None, right: Some(false)`) and the re-arm test, whose
failure output shows the pass flipping a recorded `false` back to `true`.
Then the four sites: the explicit-false arm in `cell_is_escalated` and
`map_is_escalated`, the legacy-aware disarm in `set_escalation`, and the
present-key skip in the backfill derivation. One existing assertion pinned
the old removal contract and was retargeted with its reason in place.
