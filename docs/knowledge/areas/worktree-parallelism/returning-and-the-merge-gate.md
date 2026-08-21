---
type: bee.area
title: "Worktree Parallelism — returning: the staged merge, its verify gate, and the integration queue that serializes concurrent merges"
description: "Why a feature worktree returns through a merge that is staged but never committed until the configured verify passes, why the coordination lock releases around that verify child and re-acquires behind a fence before any commit, how a second concurrent merge against the same main checkout now queues and bounded-waits behind a single processor lease instead of racing the lock, and when post-commit cleanup runs and when it refuses."
timestamp: 2026-08-14
bee:
  id: worktree-parallelism-returning-and-the-merge-gate
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/entering-creating-and-registering.md]
  decisions: [worktree-session-routing D8 (worktree merge --id <id> is the return path), D2-REVISED (the merge is a staged transaction — user review P1-2), D8a (dirty is git status --porcelain without --ignored), "D8b/D8c (--cleanup ran post-commit only, opt-in, before worktree-reclaim D1 made cleanup the default outcome)", "I47 (issues-46-53 — cleanup on ALREADY_UP_TO_DATE, superseded by worktree-reclaim D1a below)", "multisession-native D10b (issue #56 3.9 — the worktree-admin lock releases around the verify child and re-acquires behind a four-part fence before any commit)", "multisession-native D8 stage 5 / D9 invariant 12 (issue #56 3.9/mục queue — bee worktree merge requests against the same main checkout serialize through a durable integration queue and a single processor lease instead of racing the coordination lock; a busy processor bounded-waits and a timeout returns a typed, unambiguous not-run result)", "worktree-reclaim D1 (cleanup is the default outcome of a merge that merged something, not a favour a caller has to ask for)", "worktree-reclaim D1a (cleanup-by-default fires only on a merge that actually merged something, so the ALREADY_UP_TO_DATE arm removes nothing; a non-boolean --no-cleanup value is refused outright, never silently read either way)", "c117994b (traceable-runs trun-4, logged at capture 2026-08-14 — a discovered-live deadlock fix: the dirty-MAIN precondition auto-commits path-scoped .bee/ and the merging feature's own docs/history/<feature>/ before refusing, closing the deadlock against the worktree-first guard, reusing bee close's own bookkeeping-commit helper and opt-out key; cell trun-4, commit 9e01807d)", "worktree-keep-on-merge D1 (2026-08-17, supersedes worktree-reclaim D1 — a green merge KEEPS the worktree by default and queues a worktree-cleanup entry in the pending-work ledger; --cleanup re-armed as the per-merge immediate-teardown opt-in, worktree_cleanup_on_merge: true as the repo-wide opt-in, --no-cleanup an explicit keep that wins over config; prune drains and resolves the entry; cells wkm-1..3, commits f1b6a19f/3e32e605/6ff041f8)", "merge-closes-the-lane D1 (b61d41ac, 2026-08-18 — a green worktree merge that actually merged something clears the merged feature's lane waiting_on/run_state pair and rewrites next_action to name bee close --feature <feature>, but never writes phase, since a merge can land one slice of several; best-effort and post-commit inside merge_finish, gated on the same actually-merged condition the default cleanup outcome uses; commit 28928490)", "uat-stop-placement D1 (2026-08-18 — a new .bee/config.json key uat_stop picks where the uat acceptance door sits: \"merge\" (default, absent means this — today's behavior), \"close\" (merge first, accept after), or \"off\"; the existing uat_before_merge boolean stays readable as its back-compat alias, true reads as merge, false reads as off; a value outside the three, or a non-boolean alias, refuses rather than guessing; cell usp-1, commit 6b2340af)", "uat-stop-placement D2 (2026-08-18 — the close-time uat door keeps the merge-time door's lane rule verbatim, standard/high-risk only, fail-closed on a missing or unrecognized lane, and is escapable by a logged uat-deferral decision naming the feature, the same shape judge-debt already established; cell usp-3, commit ec1717ff)", "uat-stop-placement D4 (2026-08-18 — under uat_stop: close exactly four things change: worktree merge stops refusing WORKTREE_MERGE_UAT_PENDING; the post-merge lane write INVERTS from clearing the waiting_on/next_action pair to SETTING a gate wait naming the reload-test-approve-or-fix road; --cleanup and worktree_cleanup_on_merge: true are both ignored while that wait is live, reported as WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING; and bee close grows the new blocking uat door from D2; cells usp-2/usp-3, commits e94f62fb/ec1717ff)", "uat-stop-placement D5 (2026-08-18 — nothing else moves: a merge under close still never writes lane phase, keeps the worktree by default, allows repeat merges normally, keeps uat user-only at either end via bee state gate --name uat's actor-auto refusal, and leaves GATE_NAMES untouched)", "uat-stop-placement D2 revision (2026-08-18, usp-3/usp-6 — the close-time uat door and the merge-time precondition both classify a feature's lane through crate::uat::uat_lane_mode now, the one shared read; the door used to read feature_route (prefers route.lane) while the merge side read mode, and the two disagree on 12/95 real .bee/lanes records, silently dropping the uat stop; found by an independent semantic judge after a green round, since the old exempt-lane fixture wrote a lane record with mode only, the one shape that cannot express the disagreement)", "uat-stop-placement D4.3 revision (2026-08-18, usp-5 — merge_finish computes the close-time cleanup-suppression bit directly from the fail-closed uat_merge_precheck, never from whether set_lane_uat_wait_on_merge found a lane file to rewrite, so a feature with no .bee/lanes/<feature>.json on disk still keeps its worktree while a uat is owed)", "uat-approval-reaches-the-door D1 (8ca2378f, 2026-08-18 — with no live workflow record, the uat resolver reads the lane record OR the default state record, not the lane record in strict precedence over it; the live record still short-circuits, its answer final either way; a lane-side revocation cannot veto a stale default-record approval for the same feature, filed as its own open item; cell uad-1, commit 71ef5359)"]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-returning-worktree-merge-id-id-d8", "issues-46-53 cell i-2 (GH #47 — the safety property is \"nothing would be lost\", not \"a commit happened\"; trace in `.bee/cells/`, 2026-07-23)", "multisession-native cell multisession-native-2 (three-phase lock split around the verify child, four-part fence, WORKTREE_MERGE_FENCE_DRIFT; trace .bee/cells/multisession-native-2.json, commit b8fc926, 2026-07-24)", "multisession-native cell multisession-native-22 (integration-queue.mjs: durable queue + processor lease serializing worktree merge; async verify child (runVerifyChild) replacing spawnSync so a heartbeat can interleave; checkProcessorLease as the P3 fence's first line; trace .bee/cells/multisession-native-22.json, commit 546d532, 2026-07-25)", "multisession-native cell multisession-native-23 (test_msn_invariants.mjs, invariant 7's fresh two-worktree merge-time MERGE_CONFLICT proof chained to the write-time advisory-allow+warning; trace .bee/cells/multisession-native-23.json, commit 06cd209, 2026-07-25)", "docs/history/multisession-native/reports/advisor-digest-slice5.md (conditions A/B/C, verdict proceed-with-conditions)", "docs/history/worktree-reclaim/CONTEXT.md and plan.md (D1, D1a, wr-4); commit e9fe0fd8 (cleanup by default, on a real merge only); packages/bee-rs/crates/bee/src/verbs/worktree/{handlers.rs,merge.rs,phases.rs}", "traceable-runs cell trun-4 (trace .bee/cells/trun-4.json, commit 9e01807d, capped 2026-08-14 — worktree/phases.rs, git.rs, tests.rs, drivers/close.rs)", "docs/history/uat-stop-placement/CONTEXT.md (D1-D5, locked); docs/discovery/uat-after-merge/MAP.md (charting)", "packages/bee-rs/crates/bee/src/uat.rs (UatStop, uat_stop_config, uat_gate_applies_to_lane; cell usp-1, commit 6b2340af)", "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (Staged.uat_stop, close_the_lane_on_merge, set_lane_uat_wait_on_merge, effective_cleanup; cell usp-2, commit e94f62fb)", "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs (build_close_report_doors uat door, has_uat_deferral_decision, uat_gate_approved, CLOSE_UAT_PREFIX; cell usp-3, commit ec1717ff)", "docs/handbook/register.md (cell usp-4, commit 05b37c6f)", "docs/config-reference.md (cell usp-4, commit 05b37c6f)", ".bee/config-sample.json (cell usp-4, commit 05b37c6f)", "skills/bee-swarming/SKILL.md (cell usp-4, commit 05b37c6f)", "skills/bee-hive/references/gates-and-delegation.md (cell usp-4, commit 05b37c6f)", "packages/bee-rs/crates/bee/src/uat.rs (uat_lane_mode; cell usp-3, commit 01f29a61)", "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs (uat door reads uat_lane_mode; cell usp-3, commit 01f29a61)", "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (uat_wait_set from uat_merge_precheck; cell usp-5, commit 65b0520a; uat_merge_precheck reads uat_lane_mode; cell usp-6, commit 59de087c)", "docs/knowledge/patterns/20260818-a-rule-checked-at-two-points-needs-one-shared.md", "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (merge_finish passes the precondition's own Option<&str> to uat_merge_precheck, so an unresolvable feature suppresses cleanup; make_feature_unresolvable fixture + 7 tests in worktree/tests.rs; cell usp-7, commit da336b41)", "docs/history/learnings/20260818-uat-approval-reaches-the-door.md (root cause, resolution, and the deviation from the frozen plan's strict cascade); packages/bee-rs/crates/bee/src/uat.rs (uat_gate_approved; cell uad-1, commit 71ef5359)"]
  authoritative_for: "worktree-parallelism: the return path, the merge verify gate, the integration queue that serializes concurrent merges, and cleanup"
---

# Worktree Parallelism — Returning and the Merge Gate

The return path is where an isolated feature worktree becomes ordinary history on main.
Its whole design follows from one property: nothing is committed to main until the merged
tree has been proven green, so there is never a merge commit to roll back.

## Returning: `worktree merge --id <id>` (D8)

Run from the ordinary MAIN checkout (never from inside a worktree — that includes merging
"yourself"):

- Typed zero-mutation refusals first: unknown/ungranted id, dirty MAIN tree, dirty WORKTREE
  tree, detached HEAD or branch mismatch in the worktree. **Dirty** (D8a) =
  `git status --porcelain` without `--ignored`: the worktree's gitignored `.bee` store never
  counts as dirt.
- **The last zero-mutation refusal is the user's acceptance stop (uat-gate-before-merge D1,
  cells ug-1..3, 2026-08-17).** A standard or high-risk feature whose `uat` gate is
  unapproved refuses `WORKTREE_MERGE_UAT_PENDING` before any mutation, the message naming
  its three exits: the user approves (`bee gate --name uat --approved true` — user actor
  only, never auto), a one-merge `--skip-uat`, or repo config `uat_before_merge: false`
  (a non-boolean value refuses; since defaults-and-agent-env D1, 2026-08-20, both keys
  absent reads as the CLOSE placement — see "Where the uat door sits" below — so this
  merge-time refusal fires only under an explicit `uat_stop: "merge"` or
  `uat_before_merge: true`). Tiny/small/docs lanes are exempt; a
  missing or unreadable lane reads as standard — fail closed.
- **A `--with-companion` mount survives every zero-mutation refusal** (GH #84, gh-fix-batch
  cell gfb-3, 2026-07-28 — two prior live incidents where a refused merge destroyed a
  healthy, in-use mount): the companion teardown (symlink + marker removal + best-effort
  session end) runs only AFTER all four refusal checks pass, immediately before the first
  mutation. The worktree dirty check stays honest without pre-deleting the mount by
  excluding the companion's `mountPath` AND the marker file via git pathspec
  `:(exclude)…` — never by text-filtering porcelain output, which collapses a nested
  mount to its parent directory line (`?? vendor/`) and would refuse forever. Dirt other
  than the mount still refuses. Residual, accepted: post-staging outcomes (textual
  conflict, red verify) still tear the companion down — a session must not outlive a
  merge that genuinely proceeded.
- The merge itself is a **staged transaction** (D2-REVISED, user review P1-2): `git merge
  --no-ff --no-commit <branch>` stages the merge WITHOUT committing it. Already up to date
  (nothing staged) returns a typed no-op result and never touches `git commit`. A textual
  conflict runs `git merge --abort`, then PROVES main is untouched (HEAD unchanged, no
  `.git/MERGE_HEAD`, clean tracked status) before returning typed `MERGE_CONFLICT` — bee
  still does not auto-resolve a textual conflict, it just no longer leaves conflict state
  sitting on main. A clean stage runs the configured `commands.test` (none recorded →
  `verify: skipped`) against the merged-but-**uncommitted** tree.
- **The transaction runs in three phases so the coordination lock is never held across the
  verify child (multisession-native D10b, issue #56 3.9).** Every pre-check plus the stage
  itself (capturing pre-merge HEAD and a `git write-tree` hash of the freshly staged index)
  runs under the shared coordination lock — **phase 1, locked**. When a verify command is
  configured, the lock is then RELEASED and the verify child runs against the staged-but-
  uncommitted tree with **no filesystem lock held at all** — **phase 2, unlocked** — so a
  verify that takes minutes no longer blocks every other worktree-admin operation for that
  whole window (a second, unrelated merge attempt is still free to try; it simply self-blocks
  on the ordinary dirty-main-tree check, since a real staged merge is genuinely sitting
  there). The lock is then RE-ACQUIRED before anything is ever committed — **phase 3,
  re-locked** — and only after re-checking a **four-part fence** against everything that could
  have drifted while the lock was up for grabs: HEAD is still exactly what phase 1 captured;
  `.git/MERGE_HEAD` is still present; the staged tree's identity (a fresh `git write-tree`)
  still matches phase 1's captured hash — HEAD alone would miss tampering with the staged
  index itself, since HEAD never moves for an uncommitted merge; and the worktree's grant is
  still intact. A merge with no verify command configured skips phase 2 entirely and runs
  phases 1 and 3 inside one unbroken lock hold — byte-identical to the pre-D10b single-lock
  behavior, since there is no long child to protect against.
- **Any fence mismatch aborts with main proven untouched, typed `WORKTREE_MERGE_FENCE_DRIFT`.**
  Drift means something changed main, the staged tree, or the grant during the unlocked verify
  window — main or the stage can no longer be vouched for, so bee refuses to build a commit on
  top of it: `git merge --abort` runs, main-untouched is proven the same way as every other
  abort path, and the typed drift result is returned rather than a commit.
- **Superseded (D7/D8, td-3): the verify-child arm described in this bullet no longer runs.**
  `bee worktree merge` spawns no verify command at all now; the D8 proof check (every capped
  cell for the feature already carries a recorded proof line) runs as a zero-mutation
  precondition BEFORE `git merge` is even attempted (`merge_stage`, P1) — a proof-less merge
  refuses, typed `WORKTREE_MERGE_PROOF_DEBT`, before main is touched. What follows records the
  now-historical verify-child design for archaeology only. A red verify after a textually clean
  merge and a clean fence was the semantic-conflict alarm the command existed to raise:
  `git merge --abort` ran, main-untouched was proven the same way, and the result was typed
  `MERGE_VERIFY_RED` with the output tail — fix-first before release. Because the merge was
  never committed until verify passed, **no merge commit ever existed to roll back**; this
  superseded the old "merge commit is never rolled back" contract. Only once verify was green
  AND the fence was clean did bee run `git commit` (message names the id). A post-commit guard
  checked `git status --porcelain --untracked-files=no` was clean; if the verify command itself
  left tracked files modified, the result carried a typed `warning.code:
  'verify_mutated_tracked_files'` instead of silently treating the tree as equivalent to the
  commit. Recovery for a merge commit that only failed a LATER independent verify: `git revert
  -m 1 <merge-commit>` (documented, not automated).
- **Keeping the worktree is the default outcome now; teardown is the opt-in
  (worktree-keep-on-merge D1, supersedes worktree-reclaim D1).** On a merge that stages and
  commits something, the worktree, its branch, and its registration all stay in place, and the
  merge appends exactly one `worktree-cleanup` entry to the pending-work queue ledger
  (`.bee/deferred-queue.jsonl`) naming the worktree id, branch, merge commit, worktree root, and
  `bee worktree prune` as the remove command. That entry is the owner's cross-check record: the
  merged tree stays on disk for comparison and audit until an explicit prune drains it, and
  pruning resolves the entry (see `pruning-dead-worktrees.md`). The merge result still carries
  `cleanup_suggested_command` for an immediate manual teardown.
- **Three switches, one clear precedence (D1).** `--cleanup` — re-armed from its former no-op —
  forces the old immediate teardown on one merge: worktree remove, then `git branch -d` (never
  `-D`), then grant removal, then workspace-record removal, through the same shared teardown
  helper the return path and `worktree unregister` both call (see "one teardown, explicit
  removal" in `entering-creating-and-registering.md`); a forced teardown refuses (typed; the
  merge result stays ok) when the worktree still holds tracked-modified or untracked files, and
  a teardown after a skipped verify carries a warning that nothing was checked. Repo-wide,
  `.bee/config.json`'s `worktree_cleanup_on_merge: true` — now an explicit opt-IN, where it used
  to be the opt-out — restores always-teardown; absent or any other value means keep.
  `--no-cleanup` stays an explicit per-merge keep and wins over a `true` config. A non-boolean
  `--no-cleanup` value is still refused outright, never silently read toward the destructive
  direction.
- **The safety property is still "nothing would be lost", not "a commit happened."** Cleanup never
  runs after a textual conflict or a red verify: on those paths the branch's work is **not
  integrated**, so removing the worktree would destroy the only copy of it. Every refusal that
  guarded cleanup under the old opt-in flag still guards it under the new default — nothing about
  making cleanup automatic loosened what it is willing to remove.
- **The already-up-to-date no-op removes nothing, on purpose (D1a).** This reverses the flag-era
  reading, where `--cleanup` on an up-to-date no-op still ran: that arm merges nothing, so there is
  nothing for cleanup to have integrated, and it now hardcodes cleanup off regardless of the
  default, the flag, or the config — never the passed-through decision. The no-op reports what
  would remove it instead: `cleanup_suggested_command` (`bee worktree merge --id <id> --json`
  again). It carries no "cleaned up unchecked" warning either, because that warning means *no
  verify command is recorded*, which would be a lie where verify was skipped only because nothing
  was merged.
- **A merge that actually merged something clears the merged feature's stuck "waiting on
  you" mark, but never writes its phase (merge-closes-the-lane D1, b61d41ac).** A green
  `bee worktree merge` clears the merged feature's lane `waiting_on` and `run_state` pair
  and rewrites `next_action` to name `bee close --feature <feature>` as the next step —
  the return path itself never writes `phase`, since a merge can land one slice of several
  and a phase write from merge would claim a feature is finished while it is still
  mid-flight. This is best-effort and runs only after the merge commit lands: a failure
  warns on its own line and the merge stays green. It is gated on the exact same "actually
  merged something" condition the default-outcome cleanup above already uses — an
  already-up-to-date merge, a `MERGE_CONFLICT`, and a `WORKTREE_MERGE_PROOF_DEBT` each
  leave the lane untouched.
- **The lane rewrite happens after the post-commit tracked-files reading, never before
  (merge-closes-the-lane D4).** The return path checks, once the merge commit has landed,
  whether anything else modified tracked files afterwards, and warns when something did.
  Because the lane record is itself a tracked file in a repo that keeps its workflow
  records under version control, rewriting it before that reading made every clean merge
  accuse itself. The reading therefore comes first and the lane rewrite second; the check
  is never narrowed and no path is exempted from it. Its own regression test keeps the
  lane record tracked, because an untracked one is invisible to the check and hid this
  exact fault once.
- **The lane rewrite is committed, not left as dirt (merge-commits-the-lane D1).**
  Once the rewrite succeeds, the return path commits that one lane record on its own,
  scoped to it alone, so a green merge leaves the main checkout clean. The merge commit
  itself is never rewritten to absorb it: that commit is already published as landed, and
  a bookkeeping row is no reason to rewrite history. A second, plainly named commit
  removes the real harm — a pending change that an operator cannot tell apart from
  genuine drift. Like every bookkeeping write on this path it is best-effort: a commit
  that fails warns on its own line and the merge stays green.

## Main's own `.bee` and closing-feature `docs/history/` dirt auto-commits before the dirty-MAIN refusal fires (trun-4, 2026-08-14)

**The dirty-MAIN precondition and the worktree-first write guard used to
deadlock against each other.** The dirt blocking a merge is routinely bee's
own bookkeeping — cell traces under `.bee/cells/`, `.bee/decisions.jsonl`,
`.bee/backlog.jsonl` — written by the orchestrator's ordinary state calls
during the slice. Committing exactly that bookkeeping in MAIN was itself
refused by the worktree-first guard (`AGENTS.md` "Code-touching feature work
lives in its feature worktree"), which classified a bare `git commit` in the
MAIN checkout as a feature source write while the active feature held a
granted worktree. Neither door could be satisfied without disabling the
other, so a green slice could not land at all.

- **Before the dirty-MAIN refusal fires, `worktree merge` now auto-commits —
  path-scoped, never wholesale.** When every dirty path in MAIN is under one
  of the swept roots — `.bee/` and `docs/decisions` (wholesale), the *merging
  feature's own* `docs/history/<feature>/` (never any other feature's
  history), and the exact `docs/knowledge/` files that feature's own capped
  cells recorded — the merge
  commits exactly those paths and proceeds; the same `git add -A --
  <pathspecs>` / commit helper `bee close`'s own bookkeeping commit uses
  (`R81` in `gates.md`), widened here to accept the feature's docs-history
  root as a second scoped pathspec alongside `.bee/`.
- **Any dirty path outside those two roots still refuses exactly as
  before**, `WORKTREE_MERGE_MAIN_DIRTY`, and the refusal message now names
  the offending non-`.bee`/non-history paths so the operator knows what to
  clear by hand — the worktree-first guard on genuine feature source is
  never widened or bypassed.
- **A pathspec that matches nothing is tolerated, not a hard failure.**
  `git add -A -- <pathspecs>` (unlike `git status`) errors outright with
  "pathspec did not match any files" when a root matches nothing at all —
  the ordinary case for `docs/history/<feature>/` on a worktree that never
  wrote there. The auto-commit filters pathspecs to roots that exist on disk
  or are already tracked (`git ls-files`) before `add`/`commit` runs.
- **A failing auto-commit warns; it never turns a green merge red** — same
  discipline as `bee close`'s own bookkeeping commit, including the unsigned
  (`--no-gpg-sign`, stdin-nulled) commit so a signing repo's pinentry can
  never hang the merge.
- **`.bee/config.json`'s opt-out for `bee close`'s bookkeeping commit now
  also silences this auto-commit** — no separate key: turning bookkeeping
  auto-commit off is one repo-wide choice, not two.
- **Named cost, not a defect:** the sweep is `.bee`-wide, so a *concurrent*
  session's in-flight tracked bee-store dirt can ride into this merge's
  bookkeeping commit under this feature's message — misattributed history,
  never data loss, the same tradeoff `gates.md` R81 already accepts for
  `bee close`'s own sweep.
- **Authored prose is never swept blanket (knowledge-sweep-scope, cells
  kss-1/kss-2).** `.bee` and `docs/decisions` stay wholesale — they are the
  machine-written control plane and its rendered index — but `docs/knowledge/`
  enters the root list only as the exact paths the merging feature's own
  capped cells recorded as changed, the same scoping `docs/history/<feature>/`
  already had. A sibling session's knowledge dirt therefore stays
  uncommitted, and the ordinary dirty-MAIN refusal names it, which is the
  honest instruction: commit your own capture. A merge that cannot resolve
  WHICH feature it is landing sweeps no `docs/knowledge` at all, matching what
  that same unresolved arm already did for `docs/history` — having no data to
  scope a sweep argues for not sweeping, never for sweeping wholesale. Cause:
  one merge's bookkeeping commit swallowed 21 insertions of a sibling
  session's spec sync; nothing was lost, but both the authorship and the
  feature attribution were wrong, and a peer session found it, not a test.

## Concurrent merges serialize through an integration queue, never the lock (multisession-native D8 stage 5, D9 invariant 12, msn-22)

Phase 2's lock release (D10b, above) is correct for letting other worktree-admin
operations run during a multi-minute verify, but it opened a gap: a **second**
`worktree merge` against the *same* main checkout no longer waits politely on the lock
— it slips straight into phase 1's own dirty-tree pre-check and gets a hard
`WORKTREE_MERGE_MAIN_DIRTY` refusal, honest but unfriendly, because the first merge's
staged-but-uncommitted tree genuinely *is* dirty. `integration-queue.mjs` closes that
gap:

- **Only `bee worktree merge` (`handleWorktreeMerge`) becomes queue-aware** (advisor
  condition A) — `dispatch-interlock.mjs` and `herding.mjs` are untouched; the queue is
  a merge-serialization concern only, never wired into the herding cockpit's own
  enable/disable gesture.
- Every merge request enqueues a durable record first, at
  `controlRoot/.bee/runtime/integration/queue/<seq>.json`. **When the single processor
  lease is free, the requester becomes processor and merges directly** — an empty
  queue resolves on the first iteration with no real sleep at all, so a solo merge is
  **byte-identical** to pre-D9 behavior: proven by the full 159-case
  `scripts/tests/test_worktree_cli.mjs` regression suite passing unmodified. **When the lease
  is held, the requester enqueues and bounded-waits** (`--queue-wait-ms`, default
  180s), polling every 500ms. **A timeout returns a typed
  `{ok: false, code: 'INTEGRATION_QUEUE_TIMEOUT', merged: false}` result whose text
  unambiguously says the merge did NOT run** (advisor condition B) — never a shape a
  caller could mistake for success, the same truth-telling discipline
  `MERGE_CONFLICT`/`WORKTREE_MERGE_PROOF_DEBT` already use.
- **The processor lease** is a `path` resource in lease-store.mjs
  (`"path:integration-processor"`, under `controlRoot`), acquired with a **strictly
  positive TTL** — a non-positive one is refused before ever calling lease-store,
  because lease-store treats it as "never expires" and would deadlock the queue behind
  a crashed processor forever. **Heartbeat-renewed through the verify child**: phase 2's
  verify now runs via async `spawn` (`runVerifyChild`) instead of `spawnSync`
  specifically so a timer can interleave with a multi-minute child without being
  starved by a blocked event loop; `onVerifyTick` fires on that timer (default every
  30s) and attempts a best-effort lease renewal each time — a missed renewal is never
  silently fatal to correctness, because the epoch re-check below is the authoritative
  gate, not this heartbeat. A dead processor's lease is swept before every acquire
  attempt, and the next caller's takeover bumps `epoch` by exactly 1 — a real takeover,
  never a silently reused epoch.
- **Phase 3's fence gains a first line of defense, ahead of the existing
  `checkMergeFence` staged-tree/HEAD check**: immediately after `'worktree-admin'` is
  re-acquired and before ever committing, `checkProcessorLease` re-checks the acquired
  epoch against the on-disk lease record. A **zombie processor whose lease was already
  taken over aborts here** — `git merge --abort`, main proven untouched, typed
  `WORKTREE_MERGE_FENCE_DRIFT` — exactly like any other fence-drift abort. A merge with
  no queue contention (or any caller that predates this cell and passes no
  `checkProcessorLease` at all) reaches phase 3 with both checks trivially clean.
- A caller with no resolvable session identity (no `BEE_SESSION_ID`, no live session
  record) still gets a stable sessionless lease identity rather than a refusal — merge
  has never required session identity to run solo, and this cell does not change that.
- Tests: `test_integration_queue.mjs` (14 checks, deterministic seams — a virtual `now`
  drives dead-processor takeover, no real sleeps), `test_worktree_store.mjs` (+4, the
  `checkProcessorLease`/`onVerifyTick` wiring), and `scripts/tests/test_worktree_merge_queue.mjs`
  (15 checks, real two-OS-process CLI dispatch proving serialization, no-lock-held-
  across-verify, the timeout path, and the byte-identical solo surface). Evidence: trace
  `.bee/cells/multisession-native-22.json`, commit 546d532.

**The issue-#56 acceptance suite (msn-23) indexes this and every other
multisession-native invariant.** `test_msn_invariants.mjs` is one numbered, named
entry per invariant 1-15 — an INDEX, not a from-scratch reproof: a reused entry fails
loud (never a silent pass) if its underlying suite file goes missing, its assertion
text is renamed, the underlying suite goes red as a whole, or its specific PASS line
vanishes from actual runtime output. Invariant 7 is fresh for this concept: a real
two-worktree git fixture where both sides edit the same file differently — `checkWrite`
proves the write-time advisory-allow-plus-warning (see `cross-worktree-holds.md`), then
`mergeFeatureWorktree` proves the typed `MERGE_CONFLICT` catches the same conflict at
merge time — chaining two facts no existing suite chained together. Invariant 12
(no lock held across a long op) is deliberately scoped to the two enumerated long
ops — queue processing and the merge-verify child — not a blanket "zero `spawnSync`
under any lock" claim, which is false today (short git plumbing legitimately runs via
`spawnSync` while holding `'worktree-admin'`). The suite prints a verbatim
`"15/15 PASS: invariants 1,2,...,15 all green"` summary line. Evidence: trace
`.bee/cells/multisession-native-23.json`, commit 06cd209; advisor digest
`docs/history/multisession-native/reports/advisor-digest-slice5.md`.

## Where the uat door sits: `uat_stop` (uat-stop-placement D1-D5, 2026-08-18)

The bullet above (uat-gate-before-merge D1) describes the door's ONE placement bee
shipped first: at `bee worktree merge`. That placement does not fit every topology —
a small project whose product serves out of the MAIN checkout cannot test anything
until the code is ON main, so stopping at merge asks the owner to accept work they
physically cannot run yet. `.bee/config.json` key `uat_stop` picks WHERE the door
sits instead of whether it exists at all:

| Value | Behavior |
|---|---|
| `"merge"` | `bee worktree merge` refuses `WORKTREE_MERGE_UAT_PENDING` until the gate is approved (the original shipped default, until defaults-and-agent-env D1) |
| `"close"` (the default since defaults-and-agent-env D1, 2026-08-20; absent means this) | the merge lands first — the product becomes testable on main, the `wt/<feature>` branch is held for convenient testing — and `bee close` carries the door instead |
| `"off"` | no uat stop anywhere |

defaults-and-agent-env D2 (2026-08-20) flipped the sibling `staging_before_merge`
absent-key default the same day: absent now reads FALSE — staging is opt-in
(`bee staging add`/`rebuild` refuse `STAGING_DISABLED` unless the key is an explicit
`true`), so an unconfigured repo runs worktree → merge to main → uat at close.
Explicit values on either key keep their meanings.

Any other string, or a non-string shape, refuses rather than guessing (`None` from
`uat_stop_config`, surfaced as `WORKTREE_MERGE_UAT_CONFIG_INVALID` at merge or a
blocking `uat` door with an invalid-config detail at close — never a silent pick
either way). The pre-existing `uat_before_merge` boolean stays readable as this
key's back-compat alias, read only when `uat_stop` itself is absent: `true` → `"merge"`,
`false` → `"off"`, and a non-boolean value still refuses. Said plainly: main plays
the role staging plays for bigger projects, when a repo turns this key to `"close"`.

**Under `"close"`, exactly four things change (D4).**

1. `bee worktree merge` stops refusing `WORKTREE_MERGE_UAT_PENDING` — the last
   zero-mutation precondition in the bullet above never fires under this placement.
2. The post-merge lane write from `close_the_lane_on_merge` (above, "a merge that
   actually merged something clears…") INVERTS for a merged feature whose lane cares
   (`uat_gate_applies_to_lane`) and whose `uat` gate is still unapproved: instead of
   clearing `waiting_on`/`run_state` and pointing `next_action` at `bee close`, it
   SETS `waiting_on` to a `"gate"` mark naming `"uat: <feature>"` and points
   `next_action` at the reload-test-approve-or-fix road. Every other case — `"merge"`,
   `"off"`, an exempt lane, or an already-approved gate under `"close"` — keeps the
   ordinary clearing write byte-for-byte.
3. Cleanup is forced OFF while that wait is live: `--cleanup` and
   `worktree_cleanup_on_merge: true` are both ignored, and the merge result reports
   `WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING` in place of the ordinary cleanup
   outcome — the worktree is the only place a failed uat can be fixed, and tearing it
   down would drop the grant a second merge needs (the no-granted-worktree refusal
   this file already documents). The worktree still queues the same
   `worktree-cleanup` pending-work entry the default-keep outcome above already
   writes, since it was kept either way.
4. `bee close` grows a new blocking `uat` door — present only under `"close"`,
   lane-scoped exactly like the merge-time door (`standard`/`high-risk` only, a
   missing or unrecognized lane fails closed as standard), and escapable by a logged
   `uat-deferral` decision naming the feature — the same shape `judge-debt`
   established for a blocking close door.

**Nothing else moves (D5).** A merge under `"close"` is still, in every other
respect, the same non-terminal event this whole file already describes: it never
writes lane `phase`; the worktree is kept by default; repeat merges of the same
feature run normally, a second merge with no new commits is still
`ALREADY_UP_TO_DATE`; `bee state gate --name uat` keeps refusing `--actor auto` —
`uat` stays user-only at either end of the road; `GATE_NAMES` is untouched (`uat`
was already in it); `waiting_on` kinds stay the closed `["gate", "question"]` pair
(the post-merge wait is kind `"gate"`, the right one already); and
`staging_before_merge` is independent and unaffected — opting out of staging never
opts out of `uat` at either placement, and vice versa.

A failed uat after a `"close"` merge is fixed forward: a new cell on the same
worktree (still granted, never torn down while the wait is live), merged again.
bee grows no revert or rollback mechanism for a merge already on main — main may be
broken for a while, and that is accepted for a project this small
(docs/history/uat-stop-placement/CONTEXT.md D3).

**The lane classification behind the uat stop has ONE home now
(uat-stop-placement D2 revision, usp-3/usp-6).** D4.2 above and the merge-time
precondition (uat-gate-before-merge D1) both need the same fact — "what lane is
this feature in" — to decide whether the standard/high-risk-only rule applies at
all. They used to derive it two different ways: the merge side read a live
workflow's (or the lane record's) `mode`; the close-time door read
`feature_route`, which prefers a lane record's `route.lane`. Those two fields
disagree on 12 of 95 real records in `.bee/lanes` (worked example:
`knowledge-loop`, `mode: "standard"`, `route.lane: "small"`). Under `uat_stop:
"close"` that meant a merge could SET `waiting_on` gate `uat: <feature>` while
`bee close` on the very same feature read it as exempt — the uat stop vanished
silently, the exact failure this feature exists to prevent. An independent
semantic judge found it after the round shipped green; nothing in the suite
caught it, because the suite's own exempt-lane fixture wrote a lane record
carrying only `mode` and no `route` — the one shape where the two reads cannot
possibly disagree, since the field `feature_route` prefers does not exist to
disagree with. `crate::uat::uat_lane_mode` is now the ONE read (live workflow's
`mode`, falling back to the lane record's `mode`; `route.lane` never
consulted) — the close door reads it (usp-3), and the merge side's own former
inline copy of the identical read was deleted so it reads it too rather than
keeping two driftable copies (usp-6). Only `feature_route` itself is
untouched — `judge-debt` and its other callers still read `route.lane` on
purpose, this fix narrows to the uat door alone.

**Cleanup suppression under `"close"` is computed from the fail-closed
precheck, never from whether a bookkeeping write happened to succeed
(usp-5).** D4.3 above ("cleanup is forced OFF while that wait is live") used to
read its `uat_wait_set` bit off whether `set_lane_uat_wait_on_merge` actually
found a `.bee/lanes/<feature>.json` to rewrite — so a feature with a live
workflow but no lane file on disk got `uat_wait_set: false` by omission, and
`merge_finish` tore the worktree down anyway with a uat still owed, taking away
the only place the fix could be written and dropping the grant the repeat
merge needs. `merge_finish` now computes `uat_wait_set` itself, straight from
the same `uat_merge_precheck` fail-closed read the merge-time precondition
already trusts, independent of whether the lane-record write below it
succeeds, finds nothing to touch, or fails outright — a missing or unwritable
lane file can never silently un-suppress cleanup.

**An unresolvable feature suppresses cleanup too (usp-7).** The same asymmetry
survived one more place. `merge_finish` used to gate the precheck behind
`feature.as_deref().is_some_and(...)`, so a merge that could not resolve WHICH
feature it was landing short-circuited to `uat_wait_set: false` and tore the
worktree down — while the merge-time precondition had always failed CLOSED for
the same `None`, since `uat_gate_applies_to_lane(None)` is `true`. The precheck
is now called with the same `Option<&str>` the precondition passes, so the two
agree. The judgement behind it generalizes past this door: keeping a worktree
that could have been removed costs one `bee worktree prune`; removing one that
should have been kept costs the only place the fix can be written, plus the
grant the second merge needs. When two errors are that unequal, the safe one is
the default. Three cells of this feature — usp-3, usp-5, usp-7 — each closed one
instance of "the precondition fails closed, its downstream consequence fails
open"; the general rule is recorded at
`docs/knowledge/patterns/20260818-a-rule-checked-at-two-points-needs-one-shared.md`.

**An approval now reaches the door even after the feature's workflow record
has already closed (uat-approval-reaches-the-door D1, 8ca2378f,
2026-08-18).** The `uat` approval used to have a home neither door read.
The merge-time precondition and the close-time door each consulted the
live workflow record and, failing that, the default state record filtered
to the same feature — never the lane record. `bee gate --name uat
--approved true --lane <f>` writes the approval onto the lane record
whenever the feature's workflow record is already closed (ordinary
housekeeping closes any record that is not the closing session's own), so
an owner's genuine approval landed on a file no door ever opened, and the
only visible exit stayed the one-merge skip flag — on the one gate no
bypass level may auto-approve.

One resolver now answers the question for both doors. It reads the live
workflow record first and alone: if one exists, its own answer is final,
approved or not, and nothing else is consulted. Only when no live record
stands does it look further, and there it reads as approved when EITHER
the lane record's approval or the default record's approval is a literal
true — an OR, not a fallback chain — counting the default record only
while it is presently tracking this same feature, so a foreign feature's
approval can never leak in. Every source still fails closed on anything
but a literal true: a missing gate, a missing approval map, an explicit
false, and a non-boolean value all read as not approved, exactly as
before this fix — the fix widens WHERE an approval may be found, never
WHAT counts as one.

The OR is deliberate, not a looser stand-in for a cleaner cascade. A
strict "lane wins, else default" precedence cannot be built honestly on
top of this store: the shared gate defaults stamp "not approved" onto
every merged record, so a lane record that never mentioned `uat` reads
back byte-identical to one that explicitly refused it — a precedence rule
built on presence would treat every silent lane record as a veto over a
genuine default-record approval, breaking the very case this fix exists
to unblock. The cost this accepts, named so it is never rediscovered as a
surprise: an explicit lane-side revocation cannot veto a stale
default-record approval for the same feature. That gap already existed
before this fix and is not a regression it introduces; it is tracked as
its own open item.

## Pointers (implementation)

- Lane-closing on merge (merge-closes-the-lane D1): implemented inside
  `merge_finish` in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`, best-effort and
  post-commit, gated on the same condition `attach_cleanup_outcome` /
  `enqueue_worktree_cleanup_deferral` already use. `merge_text_lines` gained
  one line naming the next action. Evidence: commit `28928490`; full suite
  `cargo test --release --manifest-path packages/bee-rs/Cargo.toml` — 2033
  passed, 0 failed.
- The three-phase merge (D10b): `mergeFeatureWorktree` / `mergeFeatureWorktreeStage`
  (phase 1) / `mergeFeatureWorktreeFinish` (phase 3) / `checkMergeFence` (the four-part
  check, advisor condition C2) in `packages/bee/lib/worktree-store.mjs`.
  Deterministic-seam regression tests replace the old sleep-based lock-duration proof: an
  injected verify script self-checks the lock's absence during phase 2, and another
  self-tampers the staged tree mid-verify to prove the fence catches it, both red-first
  against the pre-fix code. Evidence: trace `.bee/cells/multisession-native-2.json`,
  commit b8fc926.
- The integration queue (D9 invariant 12, msn-22): `runThroughQueue` / `tryBecomeProcessor`
  / `checkProcessorLeaseEpoch` in `packages/bee/lib/integration-queue.mjs`;
  `runVerifyChild` (the async verify-child replacement for `spawnSync`) and the
  `onVerifyTick`/`checkProcessorLease` hooks in `worktree-store.mjs`; CLI wiring
  (`--queue-wait-ms`, the `INTEGRATION_QUEUE_TIMEOUT` text) in `handleWorktreeMerge`,
  `the bee binary`. Evidence: trace
  `.bee/cells/multisession-native-22.json`, commit 546d532.
- The acceptance suite (D9, msn-23): `packages/bee/tests/test_msn_invariants.mjs`
  (index, 15 numbered entries) plus its two fresh Worker-concurrency race harnesses
  (`race_lease_child.mjs`, invariants 5/6). Evidence: trace
  `.bee/cells/multisession-native-23.json`, commit 06cd209.
- The `uat_stop` placement policy (uat-stop-placement D1, D2): `UatStop`,
  `uat_stop_config`, `uat_gate_applies_to_lane` in
  `packages/bee-rs/crates/bee/src/uat.rs` — one module the merge side and the
  close side both read, so the read order (`uat_stop` wins, `uat_before_merge`
  is the fallback alias, absent means `Merge`) and the lane rule live in
  exactly one place. Evidence: cell `usp-1`, commit `6b2340af`.
- The inverted post-merge lane write and forced-off cleanup under `"close"`
  (D4.2, D4.3): `close_the_lane_on_merge` (now branching on the carried
  `Staged.uat_stop`), `set_lane_uat_wait_on_merge`, and the
  `CloseLaneOutcome`/`uat_wait_set` plumbing through `merge_finish`'s
  `effective_cleanup` in `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`.
  Evidence: cell `usp-2`, commit `e94f62fb`.
- The close-time blocking `uat` door (D4.4, D2): `build_close_report_doors`'s
  `uat` door, `has_uat_deferral_decision`, `uat_gate_approved`, and the
  `CLOSE_UAT_PREFIX` constant in
  `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` — built beside the
  `judge-debt` door with the same lane-scoped, blocking, deferral-escapable
  shape. Evidence: cell `usp-3`, commit `ec1717ff`.
- The one shared lane-classification read (D2 revision, mode-vs-route.lane
  drift fix): `crate::uat::uat_lane_mode` in
  `packages/bee-rs/crates/bee/src/uat.rs`, read by the close-time `uat` door
  in `close.rs` (cell `usp-3`, commit `01f29a61`) and by
  `uat_merge_precheck` in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`, whose own
  former inline copy of the same read was deleted (cell `usp-6`, commit
  `59de087c`).
- The one shared approval resolver (uat-approval-reaches-the-door D1,
  8ca2378f): `crate::uat::uat_gate_approved` in
  `packages/bee-rs/crates/bee/src/uat.rs` — live workflow record first and
  alone, else an OR of the lane record's and the (feature-matched) default
  state record's `approved_gates.uat`. Read by the merge-time precondition
  in `uat_merge_precheck` (`verbs/worktree/phases.rs`) and by the
  close-time `uat` door in `close.rs`, replacing each door's own former
  byte-identical copy. Evidence: cell `uad-1`, commit `71ef5359`. The gate
  command's own half-write note is documented in
  `areas/workflow-state/workflow-records-and-projections.md` (R133, cell
  `uad-2`, commit `fd6529ec`).
- Fail-closed cleanup suppression under `"close"` (D4.3 revision): the
  `uat_wait_set` local inside `merge_finish`, computed straight from
  `uat_merge_precheck` rather than from `CloseLaneOutcome`, in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`. Evidence: cell
  `usp-5`, commit `65b0520a`.
- The generalizable lesson — one rule enforced at two points needs one
  shared read, and a fixture that cannot express the disagreement proves
  nothing about it: `docs/knowledge/patterns/20260818-a-rule-checked-at-two-points-needs-one-shared.md`.
- Docs for both placements: `docs/handbook/register.md`,
  `docs/config-reference.md`, `.bee/config-sample.json`,
  `skills/bee-swarming/SKILL.md`, `skills/bee-hive/references/gates-and-delegation.md`.
  Evidence: cell `usp-4`, commit `05b37c6f`.
- Pre-refusal bookkeeping auto-commit (trun-4): the dirty-MAIN check and its
  auto-commit precede in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs` (the real refusal
  site, not `merge.rs`); the shared unsigned-commit helper (widened to accept
  multiple pathspecs) is `commit_unsigned` in
  `packages/bee-rs/crates/bee/src/verbs/worktree/git.rs`; `bee close`'s own
  bookkeeping commit (`R81`, `gates.md`) is
  `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`. Tests: pathspec
  filtering, mixed-dirt refusal naming the offending paths, commit-failure
  warning, and the shared opt-out, in
  `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs`. Evidence: trace
  `.bee/cells/trun-4.json`, commit `9e01807d`; backlog row that named the
  deadlock: close-lands-bookkeeping-20260810 P2 row 708.
