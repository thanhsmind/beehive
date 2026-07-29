# lane-plan-unconditional — Learnings

**Date:** 2026-07-29
**Lane:** small · **Cells:** lpu-1 capped
**Close:** full verify green, 117/117 · feature-verify `48dbd311`
**Origin:** a user observation, not a backlog row — "the rule for deciding parallel lanes isn't wired into the flow as strongly as parallel cells"

## The finding

bee's concurrency law is correct where it is stated, and was wrong in both places
that operationalized it.

`routing-and-contracts.md`'s **MANDATORY CONCURRENCY PLAN** reads *"before dispatching anything,
the orchestrator states in one line what runs concurrently and what is forced serial and why"* —
unconditional. The knowledge concept's R86 says the same. But the two documents an agent actually
acts from each restated it with a precondition the law never carried:

| Site | Restated as |
|---|---|
| `skills/bee-hive/SKILL.md` Routing list | `busy + disjoint paths → lane not wait` |
| `routing-and-contracts.md` LANES, FIRST-CLASS | "when new feature work is ready **while another feature is live**…" |

Both only fire once something is *already* busy. The case the law exists for — two independent
ready features and nobody busy — triggered nothing at all.

## Why it went unnoticed: the tier asymmetry

| | Cells | Lanes |
|---|---|---|
| Computed default | `cells schedule --json`, **required step 1** of bee-swarming; real algorithm (`packages/bee/lib/schedule.mjs:151-246`, Kahn layering + greedy overlap packing) | none |
| What the agent must do | argue *against* the computed parallel grouping to go serial | remember to ask a question nothing asks it |
| Conflict enforcement | reservations/holds deny the write | `--paths` refusal names the holder |
| Missed-opportunity detector | absent | absent |

Cell parallelism is the **output of a command**. Lane parallelism was a sentence. That is the whole
gap, and it is why the sentence's drift survived: nothing recomputed it.

## The evidence that made it measurable

17 lane records on disk; **two** occasions of genuinely concurrent features, both 2026-07-28
(`concurrency-first` + `verify-owner-signal`, `skill-diet-wave2` + `workflow-lifecycle`). The two
most recent features before this one — including the byte-fence removal that ran all day — went
serially through the default pipeline.

This session is itself a data point: `--as-lane` was used, but only because `start-feature` refused
the default path over a stale handoff. The concurrency question was never asked.

## What shipped

Both restatements now state the lane decision as a step taken **before every feature start**. The
substance is untouched — disjoint paths still required, the `--paths` refusal still names its
holder, a worktree is still only for work needing its own checkout. Only the trigger moved from
conditional to unconditional.

Pinned in `scripts/tests/test_gate_bypass_doctrine.mjs`, extending the suite that already asserts
doctrine wording against this exact file in absent-old / present-new pairs. **Proven to bite:**
reverting the two lines produced 5 failures naming each retired phrase and each missing token;
restoring produced 0.

## The generalizable rule — promoted as R88

**A restatement can be born stricter than the law it restates, and the reader obeys the
restatement.**

R87 in the same concept already covers a copy that *survives a law change* and keeps teaching the
old rule. This is its mirror, and it needs no law change at all: the restatement was narrower from
birth. A rule is only as strong as the narrowest restatement an agent reads first, so a law
expressed in more than one place is audited against **its own statement**, never against its
neighbours.

Companion clause, from the tier asymmetry: **where a tier can be checked, check it.** A rule whose
compliance is the output of a required command survives drift that a rule living in prose does not.

## Relationship to the feature that preceded it

`budget-fence-removal` closed an hour earlier with a structurally identical defect running the
other direction: there, a concept kept asserting an **abolished** law in four registers after the
law was retired. Here, two documents asserted a **narrower** law than the one in force, from the
start.

Same mechanism — one rule stated in several places, statements drifting apart, the reader meeting
the wrong one first. Two directions:

- **Stale copy after a change** (R87, and `budget-fence-removal`'s doctrine concept)
- **Narrow copy from birth** (R88, this feature)

The two features together are the argument for auditing a multi-stated rule as a set, not
statement by statement.

## What the user's intervention did

The scope call was theirs and it was right twice over. First, they rejected new machinery (a
`bee lanes plan` verb, a status suggestion line) in favour of fixing the flow — which turned out to
be the correct diagnosis, since the law was already right and only its restatements were wrong;
adding a verb would have built machinery around a defect that was purely textual. Second, the whole
feature exists because they noticed an asymmetry the agent had been living inside all session
without registering it.

## Open friction

Unchanged from `budget-fence-removal`'s close: no detector exists, at either tier, for "these could
have run in parallel and did not". Filed there; not re-filed here.
