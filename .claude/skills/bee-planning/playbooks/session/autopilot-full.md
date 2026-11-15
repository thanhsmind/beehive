# autopilot-full

You own the verdicts, never the units. One owner runs each unit from build to
merge-ready, and nothing merges without your clean verdict.

This is for "autopilot this queue", "full autopilot" and one-owner-per-unit
programs: a queue of independent units handed over to drive to merged with full
autonomy. `orchestrate` runs a standing program whose coordinator lands
verified work itself and whose workers never merge. Here each unit's owner
carries the whole lifecycle up to the merge, and the root keeps only
verification, countersigns and audits. A unit is one cell in its own worktree.

1. **Mark the owner's items and honor state-then-wait.** Items the owner names
   stay theirs: they review and they decide, and nothing here merges one. When
   the owner asks for the protocol or the plan to be stated, deliver the
   statement and stop, because a request to state the plan is not a go.
   Execution starts only on their explicit go. On that go, write the program
   objective as a checkable exit condition and drive it as `autonomous-run`
   does, so the run continues across turns until the queue is done.
2. **Start one owner per unit with the full lifecycle.** One worker per unit,
   started through `bee dispatch prepare --kind cell` in its own worktree, owns
   build, self-proof on the real artifact (the project's `verify-app` skill, or
   the real interface), skeptical triage of every review comment per
   `babysit`, a `bee-comment-review` pass over the comments the diff added,
   because bee writes no comment in code, then `babysit` to green. An owner never rebases:
   a conflict or drift against main goes to the merger as a cell, per
   `orchestrate` § Merge safety. Every owner keeps a decision trail per
   `bee-show-me-your-work` under `docs/history/<feature>/` and returns it with
   its report. Owners push their
   own branch and never merge, because exactly one merger runs `bee worktree
   merge` and step 5 gates it.
3. **Run owners in true parallel and never chain them.** Many owners at once
   when units are self-contained: one writer per branch, disjoint files
   (`bee-principle-separate-before-serializing-shared-state`), cross-unit drift
   absorbed by the merger. Only genuinely overlapping work serializes. Self-contained
   units branch straight off main, and sequenced work is merge-then-branch. One
   exception: an owner that must split a genuinely dependent change may hold a
   short private chain of its own branches.
4. **Verify every merge-ready head before its merge.** At the owner's
   merge-ready head SHA, fan out independent reviewers per `bee-fanout`, each started through
   `bee dispatch prepare --kind reviewer`, and aggregate their results to one
   verdict. The lanes: re-run the gates at that SHA; prove the load-bearing
   behavior live on the real surface the change touches; audit the receipts and
   the diff, distrusting the PR body. The live lane is the floor, and a verdict
   without it is not clean, because a gate that passes says the code compiles
   and not that the behavior holds. No merge without the root's clean verdict.
   Findings go back to the owner for fix-forward, and the new head gets fresh
   reviewers and a fresh verdict.
5. **On a clean verdict the merge runs and the owner takes the next item.** The
   merge is `bee worktree merge --id <unit> --cleanup` from the main checkout,
   run by the root, without asking, as `uat_stop = "close"` already does. Merge
   only from a head that contains current main
   (`git merge-base --is-ancestor main <head>`), because the verdict pins that
   SHA. A head that does not is a cell for the merger, never an owner's rebase.
   If main moves again before the merge, `shipping` step 3 governs
   re-verification: a new head voids the verdict unless its `git patch-id` is
   unchanged. The owner's full-autonomy grant plus the root's clean verdict is
   the merge authorization that `babysit` alone never has. The owner picks up
   the next self-contained unit from the queue. Owner-named items stop at
   merge-ready and wait for the owner. Stop at the first refusal of the verb,
   with its own message. The `uat` door of `bee close` stays the owner's: uat
   is never auto-approved at any bypass level.
6. **Run the root layer.** A genuinely new raise of a pinned gate or budget
   value, a limit CI only lets tighten, needs your fresh countersign, granted
   only after reviewer proof. Absorbing values that already landed on main is
   drift, not a raise. Run an audit tick over all owners at a fixed interval,
   armed under `bee-herding` (`bee herding control-loop`), never left to memory
   or to lossy completion notices, because a cadence nobody armed stops without
   anyone noticing. At each tick, re-read this playbook from main with
   `git show main:skills/bee-planning/playbooks/session/autopilot-full.md`,
   then re-read the exit condition from step 1. Audit the operation against
   both. Fix drift during that tick and treat it as urgent. Probe each owner
   read-only: the cell records, `git log` and the worktree list, and collect
   each owner's decision trail. Never resume
   an agent to ask how it is, because a resume restarts an idle agent. Count
   only side effects as progress: commits, pushes, PR or check deltas and
   reports. Treat a lane that passes its expected runtime without a side effect
   as stuck: stand it down and start a replacement at once, because waiting for
   a polite return is how a queue idles. When merges batch, run a retro pass
   and a sweep of the review comments left on the merged PRs.
7. **Stand down instantly on the owner's stop.** Their hold or stand-down
   reaches every owner as a zero-writes order immediately. Owners hold their
   briefs until the owner releases them, because a write after a stop is the
   one thing the owner cannot take back.

Reply: the queue with each unit's owner, state and head SHA; each verdict and
the reviewers that produced it; what merged and what each owner took next;
countersigns granted and why; open owner gates; where the collected decision
trails live.

Proof line: the queue's merged count checked against `git log` on main,
verbatim.
