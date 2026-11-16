# autopilot-stack

You own the chain, never an owner's merge. Build and verify the queue with full
autonomy, then land the verified chain in one ordered pass and stop at the uat
door, where the owner reviews it.

This is for "autopilot-stack", "stack them, don't ship", "build the stack, I'll
land it". The sibling of `autopilot-full`. The owner loop and the verification
gate are the same; only the terminal differs. There a clean verdict authorizes
that unit's own merge. Here it appends a link to one verified chain, and the
chain lands as a whole.

1. **Run the owner loop unchanged.** One worker per unit, started through `bee
   dispatch prepare --kind cell` in its own worktree, owns its change end to
   end: build, self-proof on the real artifact (the project's `verify-app`
   skill, or the real interface), skeptical triage of every review comment per
   `babysit`, a `bee-comment-review` pass over the comments the diff added,
   because bee writes no comment in code, and babysit to green. Owners parallelize when the
   work is self-contained. Every owner keeps a decision trail per
   `bee-show-me-your-work` under `docs/history/<feature>/` and returns it in its
   report.
2. **Audit on a fixed cadence.** The root runs an audit tick at a fixed
   interval, armed under `bee-herding` (`bee herding control-loop`), never left
   to memory or to lossy completion notices, because a cadence nobody armed
   stops without anyone noticing. At each tick, re-read this playbook from main
   with `git show main:skills/bee-planning/playbooks/session/autopilot-stack.md`,
   then re-read the exit condition from step 3. Audit the operation against
   both. Fix drift during that tick and treat it as urgent. Probe each owner
   read-only: the cell records, `git log` and the worktree list; a resume
   restarts an idle agent. Count only side effects as progress: commits,
   pushes, PR or check deltas and reports. Treat a lane that passes its
   expected runtime without a side effect as stuck: stand it down and start a
   replacement at once, because waiting for a polite return is how a queue
   idles.
3. **Hold the owner's gates.** State-then-wait, so a request to state the plan
   is not a go. On the owner's explicit go, write the program objective as a
   checkable exit condition and drive it as `autonomous-run` does, so the run
   continues across turns until the chain is done. On the owner's stop, every
   owner takes an immediate zero-writes hold.
4. **Verify at STACK-READY.** The owner reports STACK-READY with the exact head
   SHA. The root verifies that SHA with independent reviewers per `bee-fanout`,
   each started through `bee dispatch prepare --kind reviewer`: gates re-run at that SHA, a
   live runtime floor over the load-bearing behavior, and a receipts-and-diff
   audit that distrusts the PR body. The results aggregate to one verdict.
   Findings go back to the owner, and nothing enters the chain unverified,
   because an unverified link lands its flaws along with everything above it.
5. **Append on a clean verdict, never land early.** No owner merges, arms
   auto-merge or closes: exactly one merger runs `bee worktree merge`, and only
   at step 8. A clean verdict appends the unit to the one chain, in verified
   order or an order the owner specified.
6. **Single writer on order, parallel writers on builds.** An owner pushes only
   its own branch, `git push --force-with-lease` after an `ls-remote` check, and
   reports its tip and the unit it follows. The root owns the chain's order and
   records it in the decision trail. Nobody else reorders it, because two
   writers on the order produce a chain neither of them can explain.
7. **Absorb drift at the root, then re-verify what moved.** The root absorbs
   movement of main through the merger: owners never rebase, and a conflict
   goes to the merger as a cell, per `orchestrate` § Merge safety. A rebase
   rewrites every SHA above it and voids the verdicts at the old SHAs.
   Compare `git patch-id` at each verdict SHA against the new head. Anything that
   actually drifted goes back through step 4 before delivery. The countersign
   rule is unchanged from `autopilot-full`: a genuinely new pin raises a stop
   for the root's fresh countersign, and absorbing drift of landed values is not
   a raise.
8. **Land the chain in order, then stop at the uat door.** When every link is
   verified at its current head, land bottom first with `bee worktree merge
   --id <unit>` from the main checkout, without asking, as
   `uat_stop = "close"` already does; the landing is one pass over the chain,
   one merge at a time. A project that sets `uat_stop` to `"merge"` presents the
   chain for the owner's yes first, as `bee-swarming` describes, and this
   playbook does not override that choice. Stop at the `uat` door of `bee
   close`, which stays the owner's: uat is never auto-approved at any bypass
   level. Stop at the first refusal of the verb, with its own message.
9. **Deliver the chain.** The deliverable is one linear chain of verified units,
   reviewable bottom-up, every link carrying its verifier verdict in the PR body
   or a comment. The owner reviews it at the uat door.

**Choosing between the autopilots.** `autopilot-full` when the units are
independent and each may land as its verdict arrives. `autopilot-stack` when
the owner wants to see the verified chain whole, the work is sequenced or
coupled, or a single landing is wanted.

Reply: the first and last unit of the chain, a one-line verdict summary per
link, and anything parked or excluded with the reason.

Proof line: `git merge-base --is-ancestor <head> main` for each landed link, and
the verdict comment link per link, verbatim.
