---
type: bee.area
title: "Worktree Parallelism — returning: the staged merge, its verify gate, and the integration queue that serializes concurrent merges"
description: "Why a feature worktree returns through a merge that is staged but never committed until the configured verify passes, why the coordination lock releases around that verify child and re-acquires behind a fence before any commit, how a second concurrent merge against the same main checkout now queues and bounded-waits behind a single processor lease instead of racing the lock, and when post-commit cleanup runs and when it refuses."
timestamp: 2026-07-25
bee:
  id: worktree-parallelism-returning-and-the-merge-gate
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/entering-creating-and-registering.md]
  decisions: [worktree-session-routing D8 (worktree merge --id <id> is the return path), D2-REVISED (the merge is a staged transaction — user review P1-2), D8a (dirty is git status --porcelain without --ignored), D8b/D8c (--cleanup is strictly post-commit), I47 (issues-46-53 — cleanup on ALREADY_UP_TO_DATE), "multisession-native D10b (issue #56 3.9 — the worktree-admin lock releases around the verify child and re-acquires behind a four-part fence before any commit)", "multisession-native D8 stage 5 / D9 invariant 12 (issue #56 3.9/mục queue — bee worktree merge requests against the same main checkout serialize through a durable integration queue and a single processor lease instead of racing the coordination lock; a busy processor bounded-waits and a timeout returns a typed, unambiguous not-run result)"]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-returning-worktree-merge-id-id-d8", "issues-46-53 cell i-2 (GH #47 — the safety property is \"nothing would be lost\", not \"a commit happened\"; --cleanup runs on the no-op and still refuses on conflict and red verify; trace in `.bee/cells/`, 2026-07-23)", "multisession-native cell multisession-native-2 (three-phase lock split around the verify child, four-part fence, WORKTREE_MERGE_FENCE_DRIFT; trace .bee/cells/multisession-native-2.json, commit b8fc926, 2026-07-24)", "multisession-native cell multisession-native-22 (integration-queue.mjs: durable queue + processor lease serializing worktree merge; async verify child (runVerifyChild) replacing spawnSync so a heartbeat can interleave; checkProcessorLease as the P3 fence's first line; trace .bee/cells/multisession-native-22.json, commit 546d532, 2026-07-25)", "multisession-native cell multisession-native-23 (test_msn_invariants.mjs, invariant 7's fresh two-worktree merge-time MERGE_CONFLICT proof chained to the write-time advisory-allow+warning; trace .bee/cells/multisession-native-23.json, commit 06cd209, 2026-07-25)", "docs/history/multisession-native/reports/advisor-digest-slice5.md (conditions A/B/C, verdict proceed-with-conditions)"]
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
- The merge itself is a **staged transaction** (D2-REVISED, user review P1-2): `git merge
  --no-ff --no-commit <branch>` stages the merge WITHOUT committing it. Already up to date
  (nothing staged) returns a typed no-op result and never touches `git commit`. A textual
  conflict runs `git merge --abort`, then PROVES main is untouched (HEAD unchanged, no
  `.git/MERGE_HEAD`, clean tracked status) before returning typed `MERGE_CONFLICT` — bee
  still does not auto-resolve a textual conflict, it just no longer leaves conflict state
  sitting on main. A clean stage runs the configured `commands.verify` (none recorded →
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
- **A red verify after a textually clean merge and a clean fence is the semantic-conflict
  alarm** the command exists to raise: `git merge --abort` runs, main-untouched is proven the
  same way, and the result is typed `MERGE_VERIFY_RED` with the output tail — fix-first before
  release. Because the merge was never committed until verify passed, **no merge commit ever
  existed to roll back**; this supersedes the old "merge commit is never rolled back" contract.
  Only once verify is green AND the fence is clean does bee run `git commit` (message names the
  id). A post-commit guard checks `git status --porcelain --untracked-files=no` is clean; if the
  verify command itself left tracked files modified, the result carries a typed `warning.code:
  'verify_mutated_tracked_files'` instead of silently treating the tree as equivalent to the
  commit. Recovery for a merge commit that only fails a LATER independent verify: `git revert
  -m 1 <merge-commit>` (documented, not automated).
- `--cleanup` (D8b/D8c): on green (or skipped) verify it runs unconditionally — worktree remove,
  then `git branch -d` (never `-D`), then grant removal, in that order. It refuses (typed; the
  merge result stays ok) when the worktree still holds tracked-modified or untracked files.
  Skipped-verify cleanup always carries a warning that nothing was checked.
- **The safety property is "nothing would be lost", not "a commit happened."** Cleanup never runs
  after a textual conflict or a red verify: on those paths the branch's work is **not integrated**,
  so removing the worktree would destroy the only copy of it. It **does** run on the
  already-up-to-date no-op, where no commit is made either — because that outcome means the target
  already holds everything the branch has, and the dirty-tree refusal above has already proved the
  worktree carries nothing uncommitted. Reading the rule as "strictly post-commit" conflates the two
  and made the flag evaporate silently on the no-op: accepted, never acted on, exit zero, no
  message. A flag the caller passed is either honoured or explained; it is never dropped.
- The no-op path therefore reports what cleanup did, and a no-op **without** the flag removes
  nothing and only suggests the command — the flag, not the path, is what removes. The no-op
  carries no "cleaned up unchecked" warning: that warning means *no verify command is recorded*,
  which would be a lie where verify was skipped only because nothing was merged.

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
  `MERGE_CONFLICT`/`MERGE_VERIFY_RED` already use.
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
  `packages/bee/bee.mjs`. Evidence: trace
  `.bee/cells/multisession-native-22.json`, commit 546d532.
- The acceptance suite (D9, msn-23): `packages/bee/tests/test_msn_invariants.mjs`
  (index, 15 numbered entries) plus its two fresh Worker-concurrency race harnesses
  (`race_lease_child.mjs`, invariants 5/6). Evidence: trace
  `.bee/cells/multisession-native-23.json`, commit 06cd209.
