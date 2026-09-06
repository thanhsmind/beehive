---
type: bee.area
title: "Worktree Parallelism — the two kinds of parallelism, and where this area stops"
description: "The difference between swarm-worker worktrees that only remove git-index contention and independent-feature worktrees that each run their own full bee lifecycle, plus the surfaces this area deliberately leaves out of scope."
timestamp: 2026-07-22
bee:
  id: worktree-parallelism-overview
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: []
  decisions: ["worktree-feature-parallelism (shipped 2026-07-16, unreviewed)", "worktree-session-routing D7/D8/D9 (enter/return commands + routing rule, 2026-07-18, GH #21, unreviewed)", "cross-worktree-holds D1-D6 (the shared holds ledger, 2026-07-20)", "hardening-1-7-10 (atomic hold acquisition + heartbeat renewal, 2026-07-21, unreviewed)"]
  sources: [docs/history/worktree-feature-parallelism/, docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-what-problem-this-solves", "docs/specs/worktree-parallelism.md#S-boundary-out-of-scope"]
  authoritative_for: "worktree-parallelism: purpose, the two kinds of parallelism, and the area boundary"
  owns.code: ["packages/bee-rs/crates/bee/src/verbs/worktree/*", "packages/bee-rs/crates/bee/src/verbs/staging/*"]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs]
---

# Worktree Parallelism — Purpose and Boundary

**Area:** how one session fans independent work into git worktrees, each running its
own bee lifecycle, reconciled to the main checkout on `git merge`.

**Status:** shipped 2026-07-16 (unreviewed); enter/return commands + routing rule added
2026-07-18 (worktree-session-routing, GH #21, unreviewed); cross-worktree hold acquisition
made atomic (single-lock conflict-check + reserve + ledger-insert) and holds gained
heartbeat-renewal on top of their TTL ceiling, 2026-07-21 (hardening-1-7-10, unreviewed).
History: `docs/history/worktree-feature-parallelism/`, `docs/history/worktree-session-routing/`.

Every other concept in this area describes one mechanism — the trust model, the
commands that enter and return, the routing rule, the shared holds ledger, the
store's lifecycle tiers. This one describes what the whole thing is for, and
which neighbouring problems it deliberately does not solve.

## What problem this solves

Two different kinds of parallelism exist in bee:

- **Swarm-worker worktrees (P40, pre-existing):** an orchestrator dispatches many workers
  into worktrees that all share ONE coordination store at the main checkout and work under
  ONE feature's single gate. Worktrees only remove git-index contention.
- **Independent-feature worktrees (this area):** a worktree runs its OWN full bee lifecycle
  — its own phase, gates, and store — so a session can advance several independent features
  at once and merge each back. This is what P40 deliberately did NOT provide.

The distinction is load-bearing everywhere else in this area. P40's worktrees are a
performance device: many workers, one feature, one gate, one store. This area's worktrees
are an isolation device: one feature each, its own gate, its own store — and therefore its
own trust question, its own return path, and its own way of staying visible to the
checkouts beside it.

## Boundary (out of scope)

- Same-feature / same-file concurrent WRITE-INTENT across worktrees is now covered by the
  shared holds ledger (above); concurrent read visibility across worktrees stays out of
  scope by design — digests and the merge gate remain the interface.
- Rollout to onboarded host repos — deferred; proven in bee's own repo first.
- P40 swarm-worker behavior — unchanged; this area coexists beside it.

## Concepts in this area

- [Worktree Parallelism — the shared holds ledger that makes an island's writes visible](cross-worktree-holds.md) — One path-keyed ledger in the main store that mirrors every reservation, acquired under a single lock so two checkouts can never both believe they hold a path, renewed by a live session's heartbeat, read by three taps whose write-time answer now splits by resource — exclusive resources still hard-deny, every other path is advisory — and released by cell rather than by holder.
- [Deciding whether two paths name the same location](path-identity.md) — Separator meaning is a platform property, case behaviour is a per-volume one, and a zero device or index is absent rather than a value. Every ambiguity resolves to 'different', because a refused legitimate operation is a retry while an accepted wrong location is not recoverable.
- [Worktree Parallelism — pruning: bee worktree prune and the fail-closed dead-worktree classifier](pruning-dead-worktrees.md) — Why a worktree that never crossed the merge return path needs its own reclaim path, the eight independent conditions every one of which must hold before a worktree is judged dead, why every condition keeps rather than guesses on any doubt, and why liveness reads live session records instead of the workspace record's own ownership fields.
- [Worktree Parallelism — the staging mixing ground the user tests at](staging-mixing-ground.md) — The invariant 'staging = main + Σ features awaiting UAT', the three triggers that keep it true, why staging is disposable and its history is garbage, and the teeth that keep it a mixing ground instead of a back door into main.

## Patterns in this area

- [A fix for a too-strict comparison becomes a too-loose one, and every test written for the fix points the wrong way](../../patterns/20260727-a-fix-for-a-too-strict-comparison-becomes-a-too-loose-one.md) — Normalizing both sides of a path comparison folded a character that is legal inside a filename, so two genuinely different directories compared equal and the identity check examined the wrong location. Three reviews and a red-first proof missed it; a reviewer asked to construct a false-equal found it immediately.
- [A guard on the shared index does not stop a path-scoped commit naming someone else's file](../../patterns/20260806-a-guard-on-the-shared-index-does-not-stop-a-path-scoped-commit-naming-someone-elses-file.md) — The escape a guard recommends inherits the guard's job: the concurrent-worker git guard closes the staging path that swept siblings' work into the wrong commit, then allows a path-scoped commit to name any path, including a file another live worker is still editing.
- [A pathspec-less commit in a shared checkout takes a sibling session's staged work](../../patterns/20260806-a-pathspec-less-commit-in-a-shared-checkout-takes-a-sibling-session-s-work.md) — One session ran an ordinary commit with a message and no paths while another session had unrelated files already staged in the same index; the commit swept them in, so a cell's diff carried a stranger's changes and neither session's history says what it looks like it says.
- [A refusal from one verb of a family does not speak for the verb the contract rests on](../../patterns/20260813-a-refusal-from-one-verb-of-a-family-does-not-speak-for-the-verb-the-contract-rests-on.md) — Four sibling verbs refused from a granted worktree, so the fifth was assumed to refuse too — and on that assumption a user-authored decision was superseded, a grant was dropped, and a defect was filed for a problem the dropped grant created. The one verb the worker contract actually depends on had been widened nine days earlier, and testing it would have cost one command.
- [A fresh feature worktree is not ready-to-run — budget its bootstrap gaps](../../patterns/20260818-a-fresh-feature-worktree-is-not-ready-to-run.md) — A worktree created by `bee worktree new` used to arrive with two known gaps. The binary gap is CLOSED as of 2026-08-21 — the bootstrap now provisions `.bee/bin/bee` itself. The dirty-copy gap is still live: the bootstrap drops untracked cell-record copies under `.bee/cells/` on a tracked path, which later blocks `bee worktree merge` as WORKTREE_DIRTY (remove the copy via git clean before merging — the main store holds the authoritative record). Three deliveries hit the binary gap and two hit the dirty-copy gap on the same day before this was named.
- [A write placed before a self-check makes the check accuse itself](../../patterns/20260818-a-write-placed-before-a-self-check-makes-the.md) — A write placed before a self-check makes the check accuse itself
- [Inside a worktree the tracked copy of a runtime store answers stale](../../patterns/20260819-inside-a-worktree-the-tracked-copy-of-a-runtime-store-answers-stale.md) — A feature worktree carries its own git-tracked copy of .bee/decisions.jsonl, frozen at whatever the branch last committed while the live store is the control root's, so a reader standing inside the worktree gets a confidently wrong answer to "does decision X exist" — two independent judges hit this within one hour, and the second issued a false NEEDS_REVISION finding that three architectural decisions had no record behind their cited hashes, when the decisions had existed the whole time.
- [A stated ancestry direction is a coin flip until one real pair of commits is measured against it](../../patterns/20260901-a-stated-ancestry-direction-is-a-coin-flip-until-it-is-measured.md) — An ordering written in prose — ancestor of, before, predates — reads equally plausible both ways round, so a decision that states one ships a 50% bug into a guard that then fires on every run and is tuned out
- [Never release another agent's reservations on a stall signal](../../patterns/20260710-never-release-another-agents-reservations-on-a-stall.md) — Never release another agent's reservations on a stall signal
- [A shared-suite red is not yours while a sibling cell is in flight](../../patterns/20260713-a-shared-suite-red-is-not-yours-while.md) — A shared-suite red is not yours while a sibling cell is in flight
- [Realize a structural model via git config, not a file migration, when the boundaries already exist](../../patterns/20260716-realize-a-structural-model-via-git-config-not.md) — Realize a structural model via git config, not a file migration, when the boundaries already exist
- [A shared checkout with a second live session: check the out-of-scope tree before a blocking verify, and diff before diagnosing "flaky"](../../patterns/20260719-a-shared-checkout-with-a-second-live-session.md) — A shared checkout with a second live session: check the out-of-scope tree before a blocking verify, and diff before diagnosing "flaky"
- [With concurrent sessions possible, the claim precedes the spawn — and session ids are self-derived, never handed down](../../patterns/20260719-with-concurrent-sessions-possible-the-claim-precedes-the.md) — The orchestrator claims a cell atomically before spawning its worker; session ids are read from the worker’s own runtime, never handed down
- [Measure the contention topology before adding coordination machinery](../../patterns/20260720-measure-the-contention-topology-before-adding-coordination-machinery.md) — Measure the contention topology before adding coordination machinery
- [Locks guarding long synchronous child spawns cannot be heartbeat-renewed — probe owner liveness instead](../../patterns/20260721-locks-guarding-long-synchronous-child-spawns-cannot-be.md) — A lock held across a blocking spawnSync cannot heartbeat-renew its mtime; stale takeover must probe owner liveness instead
- [Shared-file axes must be sequenced at dispatch time; a worker's "watcher" dies with its turn](../../patterns/20260721-shared-file-axes-must-be-sequenced-at-dispatch.md) — Shared-file axes must be sequenced at dispatch time; a worker's "watcher" dies with its turn
- [A fresh worktree inherits every other feature's stale claimed cells because .bee/cells/ is git-tracked](../../patterns/20260724-worktree-inherits-stale-cells.md) — A fresh worktree inherits every other feature's stale claimed cells because .bee/cells/ is git-tracked
- [A resolver added beside the old one is dead code until every call site is swept — the seam ships when the LAST consumer moves, not when the new function lands](../../patterns/20260725-a-resolver-added-beside-the-old-one-is-dead-code.md) — A cell that adds a new function beside an old one and calls the migration started ships nothing observable; an honest worker block that names the real sweep size beats a false-green cap that hides it.
- [A shim that preserves a CLI surface can still drop a side-effect that surface never named](../../patterns/20260725-a-shim-can-drop-an-unnamed-side-effect.md) — The cross-worktree mirror write lived beside the reservation store's write, not inside it — a neighbor, not a return value. Retiring the store's own implementation without deliberately carrying that neighbor along would have lost cross-worktree coordination silently, because every visible test of the shim's own contract (reserve/release/renew) would still pass.
- [git add with a pathspec fails where git status succeeds when nothing matches](../../patterns/20260814-git-add-with-a-pathspec-fails-where-git-status-succeeds-when-nothing-matches.md) — `git status --porcelain -- <root>` reports quietly on a root that matches nothing, but `git add -A -- <root>` exits non-zero with 'pathspec did not match any files' — a multi-root bookkeeping commit must filter its pathspecs to ones that exist on disk or are tracked before add/commit.
- [Parallel workers in one worktree share one git index — file-disjointness does not protect commits](../../patterns/20260817-parallel-workers-in-one-worktree-share-one-git-index.md) — Two execution workers editing disjoint files inside the SAME worktree checkout still collide on the shared git index: one worker's stage/commit/amend cycle can sweep the sibling's staged edits into its own commit or wipe an in-progress edit. Disjoint files protect the working tree, never the index. Parallel cells in one worktree need serial commit windows, a per-worker GIT_INDEX_FILE, or one-worker-at-a-time dispatch.
