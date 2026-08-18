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
  decisions: [worktree-session-routing D8 (worktree merge --id <id> is the return path), D2-REVISED (the merge is a staged transaction — user review P1-2), D8a (dirty is git status --porcelain without --ignored), "D8b/D8c (--cleanup ran post-commit only, opt-in, before worktree-reclaim D1 made cleanup the default outcome)", "I47 (issues-46-53 — cleanup on ALREADY_UP_TO_DATE, superseded by worktree-reclaim D1a below)", "multisession-native D10b (issue #56 3.9 — the worktree-admin lock releases around the verify child and re-acquires behind a four-part fence before any commit)", "multisession-native D8 stage 5 / D9 invariant 12 (issue #56 3.9/mục queue — bee worktree merge requests against the same main checkout serialize through a durable integration queue and a single processor lease instead of racing the coordination lock; a busy processor bounded-waits and a timeout returns a typed, unambiguous not-run result)", "worktree-reclaim D1 (cleanup is the default outcome of a merge that merged something, not a favour a caller has to ask for)", "worktree-reclaim D1a (cleanup-by-default fires only on a merge that actually merged something, so the ALREADY_UP_TO_DATE arm removes nothing; a non-boolean --no-cleanup value is refused outright, never silently read either way)", "c117994b (traceable-runs trun-4, logged at capture 2026-08-14 — a discovered-live deadlock fix: the dirty-MAIN precondition auto-commits path-scoped .bee/ and the merging feature's own docs/history/<feature>/ before refusing, closing the deadlock against the worktree-first guard, reusing bee close's own bookkeeping-commit helper and opt-out key; cell trun-4, commit 9e01807d)", "worktree-keep-on-merge D1 (2026-08-17, supersedes worktree-reclaim D1 — a green merge KEEPS the worktree by default and queues a worktree-cleanup entry in the pending-work ledger; --cleanup re-armed as the per-merge immediate-teardown opt-in, worktree_cleanup_on_merge: true as the repo-wide opt-in, --no-cleanup an explicit keep that wins over config; prune drains and resolves the entry; cells wkm-1..3, commits f1b6a19f/3e32e605/6ff041f8)", "merge-closes-the-lane D1 (b61d41ac, 2026-08-18 — a green worktree merge that actually merged something clears the merged feature's lane waiting_on/run_state pair and rewrites next_action to name bee close --feature <feature>, but never writes phase, since a merge can land one slice of several; best-effort and post-commit inside merge_finish, gated on the same actually-merged condition the default cleanup outcome uses; commit 28928490)"]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-returning-worktree-merge-id-id-d8", "issues-46-53 cell i-2 (GH #47 — the safety property is \"nothing would be lost\", not \"a commit happened\"; trace in `.bee/cells/`, 2026-07-23)", "multisession-native cell multisession-native-2 (three-phase lock split around the verify child, four-part fence, WORKTREE_MERGE_FENCE_DRIFT; trace .bee/cells/multisession-native-2.json, commit b8fc926, 2026-07-24)", "multisession-native cell multisession-native-22 (integration-queue.mjs: durable queue + processor lease serializing worktree merge; async verify child (runVerifyChild) replacing spawnSync so a heartbeat can interleave; checkProcessorLease as the P3 fence's first line; trace .bee/cells/multisession-native-22.json, commit 546d532, 2026-07-25)", "multisession-native cell multisession-native-23 (test_msn_invariants.mjs, invariant 7's fresh two-worktree merge-time MERGE_CONFLICT proof chained to the write-time advisory-allow+warning; trace .bee/cells/multisession-native-23.json, commit 06cd209, 2026-07-25)", "docs/history/multisession-native/reports/advisor-digest-slice5.md (conditions A/B/C, verdict proceed-with-conditions)", "docs/history/worktree-reclaim/CONTEXT.md and plan.md (D1, D1a, wr-4); commit e9fe0fd8 (cleanup by default, on a real merge only); packages/bee-rs/crates/bee/src/verbs/worktree/{handlers.rs,merge.rs,phases.rs}", "traceable-runs cell trun-4 (trace .bee/cells/trun-4.json, commit 9e01807d, capped 2026-08-14 — worktree/phases.rs, git.rs, tests.rs, drivers/close.rs)"]
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
  (absent means on; a non-boolean value refuses). Tiny/small/docs lanes are exempt; a
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
  path-scoped to two roots only.** When every dirty path in MAIN is under
  `.bee/` (wholesale) or under the *merging feature's own*
  `docs/history/<feature>/` (never any other feature's history), the merge
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
