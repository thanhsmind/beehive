# decision-attribution — learnings

Harvested 2026-08-25 from CONTEXT.md, plan.md, cells da-1/da-2/da-3 and the
live migration run.

## A shared fallback is only correct for the callers it was designed for

`resolve_mutation_target` answers "which record should I act on?" and falls
back to the shared default `.bee/state.json` when a session has no bound lane.
For its twenty-odd callers that is right: an unbound session *mutating* state
should act on the default record.

`decisions log` was the one caller asking a different question — "what is this
decision **about**?" — and it reused the same helper for it. The fallback
answered the question it was designed for and quietly gave a wrong answer to
the one actually being asked. The default record's `feature` is whatever
another session most recently made active.

The rule: when a helper resolves a fallback, a new caller must re-derive
whether that fallback answers *its* question. Copying a helper copies its
default, and "act here" is not the same as "this is what this is about". Same
shape as the already-promoted pattern that a probe's failure direction belongs
to the consequence, not to the module.

## A bug's first measurement is a sample, not a census

The mis-attribution was found by counting records stamped `model-role-split`
and reading their text: **23**. That number went into the locked decision and
into what the owner approved.

The fix's own dry-run over the whole store found **67**. The same mechanism had
mis-filed decisions under `release-2-11-0`, `uat-gate-before-merge`,
`staging-optional`, `staging-lane` and more. The first count was biased by
exactly the thing that made the bug visible: I only looked where I had already
been blocked, because a close door had stopped me there.

Build the general predicate first, then let a dry run do the counting. Never
carry the discovery sample into the fix's scope — and when the real number
lands, say it out loud rather than letting the approved figure stand.

## The bug lived where the flow has no lane yet

Every one of the 18 `human-mailbox` decisions was logged **before** the
`human-mailbox` lane existed — earliest 10:11:06, latest 10:55:26, lane created
10:57:48. That timing is the whole diagnosis, and it is what turned a vague
"stamped with the session's active lane" into a precise, provable mechanism.

It also explains why the defect was severe rather than cosmetic. The Discovery
flow is where bee logs decisions *most* — a wayfinding map locks them ticket by
ticket — and it is precisely the phase that has no lane to attribute to. The
busiest writer was the one with nothing to write.

Generalises: when a field is populated from context, ask which phase populates
it most, and check whether that phase has the context at all.

## The test that could not fail, again

`log_touches_sweep_own_history_stays_a_citation_when_no_feature_is_bound`
covered the unbound case with a fixture that had **no `.bee/state.json` at
all** — so it passed identically before and after the fix. The real shape, a
state file that exists and names a foreign feature, had no test.

Meanwhile a *different* existing test pinned the bug as correct behaviour: it
wrote a bare `state.json` and asserted the borrowed name rode the event. So the
suite simultaneously failed to cover the defect and asserted it was intended.

Both were caught only by writing the new test first and watching it go red
(`left: Some("someone-elses-feature")`, `right: None`). This is the third
recurrence in two features of the promoted pattern `20260819` — a fixture that
cannot distinguish the broken state from the fixed one. Prose is not holding
it.

## What held

The narrow predicate earned its keep. Correcting 67 records in a store three
sibling sessions were actively appending to could have gone badly; instead the
diff proved exactly 67 lines changed, the line count unchanged, and `feature`
the only field differing anywhere. Two properties did that work: the verb can
only act where a record already contradicts itself, and every untouched line is
carried through byte-for-byte rather than re-serialised. The five records it
declined are the evidence the predicate is real — it left work undone rather
than guess, and that residual is filed rather than hidden.
