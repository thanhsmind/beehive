# shipping

You own what lands. Verify each unit independently, land only the verified
run, one merge at a time from the root, then stop at the uat door.

This is for "land the stack", "ship it", "merge the verified units" or the
second half of a run that `babysit` already drove to green. Babysit makes units
mergeable. Shipping decides what is actually safe to merge and lands it. Green
is not safe, and the gap between those two words is where this playbook lives.

A unit is one worktree branch with its own cap proof, usually with a draft PR.
Units land in the order the program fixed; the lowest unmerged one is the
bottom.

1. **Verify every unit independently before landing anything.** One reviewer
   per unit, not batched, each started through `bee dispatch prepare --kind
   reviewer`, each exercising the real surface against the base and against the
   head: the project's `verify-app` skill when it has one, otherwise the real
   interface itself. Each returns `PASS`, `PASS+NOTES` or `FAIL`, posts that
   verdict as a comment on its own PR, and adds a row to the decision trail
   under `docs/history/<feature>/`, so the record outlives the chat. Safe means
   a verdict from an agent that did not write the code. CI green is not a
   verdict, and an approving automated review is not a verdict.
2. **Land only the contiguous verified run rooted at the bottom.** Walk up from
   the lowest unmerged unit and stop at the first one without a passing
   verdict, where both `PASS` and `PASS+NOTES` pass. A verified unit sitting
   above an unverified one is not landable, because merging it would pull the
   gap in underneath it. Report the ceiling as a unit id and say what breaks
   the chain.
3. **Re-check that the verdicts still describe the code.** A rebase rewrites
   every SHA above it and silently invalidates every verdict without touching a
   single check. Compare `git patch-id` at the verdict SHA against the current
   head before trusting an older verdict, and re-verify anything that actually
   drifted. Twenty-one verdicts went stale this way in one run with no signal at
   all.
4. **Land with `bee worktree merge`, one unit at a time, without asking.** Run
   `bee worktree merge --id <unit> --cleanup` from the main checkout, bottom
   unit first, as `uat_stop = "close"` already does. Exactly one merger runs it
   at a time, because two merges racing for main lose a commit. The verb checks
   the cap's proof line itself, so no second verify runs here. Stop at the first
   refusal with the verb's own message: `MERGE_CONFLICT` stops the run and the
   conflict goes to a cell, never to a retry, and no gate is approved to get
   past a refusal.
5. **Never enable GitHub auto-merge, and never run `gh pr merge`.** A PR that
   merges itself skips the verb's proof-line and uat checks and lands in an
   order nobody chose. If a previous agent armed it, disarm with `gh pr merge
   <n> --disable-auto` and confirm the field is back off.
6. **Confirm each landing from git, not from the PR page.** A PR can read
   merged, or a row can read landed, while the commit never reached main. After
   every merge run `git merge-base --is-ancestor <head> main`, and recompute
   what is merged from `git log` on main, never from narrative. If you cannot
   confirm a landing, say so rather than inferring it.
7. **While a merge runs, touch nothing else.** No push to the unit being
   merged, no rebase, no second merger, no merge of a unit above the ceiling.
   Independent work not yet verified waits for its own pass through step 1.
8. **Report each merge and the new ceiling as it lands.** If a merge stalls or
   refuses, diagnose before mutating, because a refused merge and a broken
   branch look identical from the outside. A merge that lands is progress, not
   termination; keep walking up until the ceiling.
9. **Stop at the ceiling and at the uat door.** When the verified run is
   merged, stop at the `uat` door of `bee close`, which stays the owner's: uat
   is never auto-approved at any bypass level. Report what landed, what the
   next unverified unit is, and what verifying it would take. Extending the run
   is a new pass through step 1, not a judgment call you make at 3am.

Reply: the verified run and its ceiling, each unit's verdict and who produced
it, what you merged and how you confirmed it, what landed, and what the next
gap needs.

Proof line: `git merge-base --is-ancestor <head> main` for each landed unit,
and the verdict comment link per unit, verbatim.
