# ppb-28c: check of the principles ported in ppb-28

Checked against commit `f82a8ebbd` (ppb-28). The sources are the pstack
`principle-prove-it-works`, `principle-sequence-verifiable-units` and
`principle-make-operations-idempotent` `SKILL.md` files (each directory holds
only that one file). No file other than this report was edited.

## Verdict

Every source rule of the three principles is present in bee. None is left out
and none is missing. No pstack-only tool name appears in the new skills or in
their expertise sections, and no bee rule was removed: the commit's only
deleted lines are hashes and a timestamp in `.bee/onboarding.json` and the
release manifest, replaced by the regenerated values.

"Present" means the rule is in the bee skill body, its depth section, or both.
Depth sections: `expertise/tests.md` § Prove it works (line 352),
`expertise/planning.md` § Sequence work into verifiable units (line 176),
`expertise/operations.md` § Make operations idempotent (line 303). The
vendored copies under `.bee/expertise/` match these files byte for byte, the
three `SKILL.md` files are identical in all five rendered trees
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`), and
each `LICENSE.pstack` matches the pstack `LICENSE` byte for byte.

## prove-it-works

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Verify every output by checking the real thing; never infer from proxies, self-reports or "it compiles" | present | skill body, first paragraph; depth opening |
| 2 | Why: unverified work has unknown correctness; indirect checks (mtimes, output freshness, self-reports, cached screenshots) feel cheaper; a wrong inference costs more | present | skill body and Why line; depth opening |
| 3 | Pattern: after any task, ask "how do I prove this actually works?" | present | skill body first sentence; skill Check line; depth opening |
| 4 | Check process liveness directly, not through derived state | present | depth bullet only |
| 5 | Read the actual value, not a cached or derived representation | present | skill description; depth bullet |
| 6 | When verification fails, suspect the observation method first | present | skill body; depth bullet |
| 7 | Code: build it (necessary, not sufficient) | present | skill body; depth step 1 |
| 8 | Code: run it and exercise the actual feature path | present | skill body; depth step 2 |
| 9 | Code: check the full chain, input to output | present | skill body; depth step 3 |
| 10 | Code: integrations, test the full communication path end to end | present | skill body; depth step 4 |
| 11 | Delegation: inspect the artifact (diff, file, runtime behavior), not the delegate's summary; agents report intent | present | skill body; depth paragraph, with the bee reviewer spelling `bee dispatch prepare --runtime claude --kind reviewer` added (D2) |
| 12 | Script the check: a deterministic re-runnable comparison beats a one-time eyeball; keep its output as an artifact | present | skill body; depth paragraph |
| 13 | A script comparing old and new compiled output catches what a glance misses | present | depth paragraph only |
| 14 | Keep the artifact visible; commit it only for large or complex work, via show-me-your-work | present | depth paragraph, as `bee-show-me-your-work`, which exists in `skills/` |

## sequence-verifiable-units

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Order work as small units each ending in a checkable state; do not advance until green | present | skill body; depth opening |
| 2 | One discipline at two altitudes: execution and delivery | present | skill body; depth opening |
| 3 | Why: a break caught at its unit is cheap to localize; after a batch it is buried under work on a broken base | present | skill Why line; depth Why |
| 4 | Why: a replayable delivery turns "trust me" into "watch it go red, then green" | present | depth Why only |
| 5 | Execution: verify each change before the next; never batch and verify at the end | present | skill body; depth Execution |
| 6 | Each unit is a before/after bracket: known-good state, one change, check, proceed | present | skill body; depth Execution |
| 7 | Rebase onto clean trunk first so checks measure the real baseline | present | as "start from a clean base" in skill body and depth Execution; the trunk-rebase wording is the D2 swap, named in the port notes |
| 8 | When a lever does the edits the per-unit check is nearly free; run it anyway | present | depth Execution only, pointing at Build the lever |
| 9 | Delivery: stack commits and PRs in the order that proves the work | present | as "order commits, and the cells that produce them" in skill body and depth Delivery |
| 10 | Canonical shape: failing test first, fix on top (red, then green) | present | skill body; depth Delivery, pointing at Red before green |
| 11 | Other orders: subtraction before reshape, baseline before treatment, scaffold before feature | present | skill body; depth Delivery |
| 12 | Each commit lands on its own; the sequence reads as an argument | present | skill body; depth Delivery |
| 13 | Pattern: pick the smallest unit that ends in a check | present | skill Check line; depth three-line pattern |
| 14 | Pattern: verify before advancing, red to green per unit | present | skill Check line; depth three-line pattern |
| 15 | Pattern: order units so the sequence builds confidence for you and the reviewer | present | depth three-line pattern |
| 16 | Complement to prove-it-works and build-the-lever | present | depth closing paragraph; both skills exist in `skills/` |

## make-operations-idempotent

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Converge on the correct state however many times it runs and wherever it starts | present | skill body; depth opening |
| 2 | Every state-mutating operation answers: runs twice? previous run crashed halfway? | present | skill body; depth opening |
| 3 | Why: crashes, restarts and retries are normal; partial state turns every restart into a debugging session | present | skill Why line; depth opening |
| 4 | Convergent startup: scan existing state, clean stale artifacts, adopt live sessions | present | skill body; depth bullet |
| 5 | Content-based cleanup: compare by content, not creation order | present | skill body; depth bullet |
| 6 | Self-healing locks: PID-based stale lock detection | present | skill body (holder gone, no PID word); depth bullet names the PID |
| 7 | Idempotent scheduling: failed work respawns cleanly, fresh input each cycle | present | skill body; depth bullet |
| 8 | Test: runs twice in a row? | present | skill Check line; depth test 1 |
| 9 | Test: previous run crashed at every possible point? | present | skill Check line; depth test 2 |
| 10 | Test: does re-execution converge to the same end state? | present | skill Check line; depth test 3 |
| 11 | "It depends on what state was left behind" means the operation needs a reconciliation step | present | depth closing paragraph only |

## pstack-only tool names

None in `skills/bee-principle-{prove-it-works,sequence-verifiable-units,make-operations-idempotent}/SKILL.md`
or in the three added depth sections. A search for Graphite, stacked-PR,
Cursor, subagent and pstack skill slugs found nothing; the only skill named,
`bee-show-me-your-work`, is a bee skill.

One remark on the port notes, not a defect in the port: the notes say the
sequence-verifiable-units source "stacks commits and PRs with a stacked-PR
tool". The source text names no tool; it says "stack commits and PRs". The
bee wording (ordered commits and cells, a clean base) is still the right D2
reading, since bee lands one commit per cell.

## Bee rules removed

None. `git show f82a8ebbd` deletes only the old expertise hashes and the
`updated_at` timestamp in `.bee/onboarding.json` and the release manifest.
No existing index row under `## Principle homes` changed; three rows were
added.

## Additions beyond the source (not checked as defects)

A Not when line on each skill, situation rows in each guide's routing table,
cross-references to existing sections, and one sentence tying prove-it-works
to the cap's proof line. All are listed in
`docs/history/pstack-playbook-port/port-notes/principles-ppb-28.md`.

## Verify

`cargo test --release -p bee --test principle_index_parity`: 5 passed, 0 failed.
