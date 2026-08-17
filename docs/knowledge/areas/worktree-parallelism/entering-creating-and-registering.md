---
type: bee.area
title: "Worktree Parallelism — entering: creating a feature worktree and registering it"
description: "The paved road that creates and grants a feature worktree in one move, the adoption command that registers a hand-made one, the fresh lifecycle state a bootstrap writes, a concurrency-aware refusal when the source checkout holds a shared nested checkout without a declared companion mount, and the typed zero-mutation refusals and best-effort rollback that guard all of it."
timestamp: 2026-08-05
bee:
  id: worktree-parallelism-entering-creating-and-registering
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/the-trust-model.md]
  decisions: ["worktree-session-routing D7 (worktree new is the paved road for STARTING a feature worktree, GH #21)", worktree-feature-parallelism (register/list/unregister and the bootstrap contract), I46 (issues-46-53 — immutable creation slug), worktree-concurrency-guard D1(a)/D3/D4/D6 (docs/history/worktree-concurrency-guard/CONTEXT.md; supersession 0ccc1cf3), worktree-reclaim D3 (a teardown either finishes or does not start — unregister drops the workspace record in the same call it drops the grant), worktree-reclaim D3a (the shared teardown helper takes directory removal as an explicit parameter; unregister never reaches it)]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-registering-a-worktree-the-cli", "docs/specs/worktree-parallelism.md#S-entering-worktree-new-feature-slug-d7-gh-21", "issues-46-53 cell i-2 (GH #46 — the creation slug is recorded immutably because the feature name is not; the refusal names the drifted field instead of the branch; trace in .bee/cells/, 2026-07-23)", "worktree-concurrency-guard cell wcg-3 (capped trace and report, 2026-07-24 — worktree-new concurrency-aware refusal)", "worktree-concurrency-guard cell wcg-fix-1 (capped trace and report, 2026-07-26 — acting-session self-exclusion fix, review finding #1)", "docs/history/worktree-reclaim/CONTEXT.md and plan.md (D3, D3a, wr-1); commit 430ec952 (lift teardown into one helper; unregister drops its workspace record); packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs (run_unregister), merge.rs (teardown_worktree)"]
  authoritative_for: "worktree-parallelism: creating, granting and bootstrapping a feature worktree"
---

# Worktree Parallelism — Entering: Creating and Registering

Two ways in, and they differ only in who made the worktree. `register` adopts a worktree
that already exists; `new` creates one and adopts it in the same move. Both end in the same
place: a grant in the main store's registry, keyed by the git-verified id, and a freshly
bootstrapped store inside the worktree.

## Registering a worktree (the CLI)

- `worktree register --feature <slug>` — run from inside a linked-valid worktree. Writes the
  grant into the main store's registry (keyed by the git-verified id) and **bootstraps** the
  worktree's own store: copies the main store's onboarding + config, writes a FRESH lifecycle
  state (the named feature, phase idle, all gates unapproved). An independent-feature worktree
  runs its OWN feature, so it inherits none of main's state/gates/log.
- `worktree list` also names which granted worktrees are already merged and only waiting for
  cleanup (worktree-keep-on-merge D1): an id whose root still has a pending `worktree-cleanup`
  entry in the queue ledger renders `(granted, merged — pending cleanup)` (JSON: a
  `merged_pending` map), a plain grant stays `(granted)` — so the list answers "what is still on
  disk only for cross-checking" without opening the ledger.
- `worktree list` / `worktree unregister [--id <id>]` — read the grants in the main store; unregister drops
  the grant AND the workspace record in the same call (worktree-reclaim D3), never one without the
  other, through the same shared teardown helper `worktree merge` cleanup and `worktree prune` call
  — see `returning-and-the-merge-gate.md` and `pruning-dead-worktrees.md`. `unregister` reaches only
  that registry half: it never removes the worktree's directory or its branch, because it cannot know
  whether the caller still wants those — a partial teardown that removed a grant but left the
  workspace record behind was exactly the bug this closes: 13 orphan records with no matching grant,
  invisible to a grant-driven scan.

## Entering: `worktree new --feature <slug>` (D7, GH #21)

The paved road for STARTING a feature worktree — create and register in one move, run from
the ordinary main checkout:

- Creates the sibling `../<repo-basename>--wt--<slug>` on branch `wt/<slug>` (optional
  `--base-ref`, resolved as a commit-ish via `git rev-parse --verify --end-of-options
  "<ref>^{commit}"` — accepts HEAD, HEAD~1, short shas, tag^{commit}; the RESOLVED sha is
  what the worktree is created from, and anything unresolvable is one typed
  `WORKTREE_BASE_NOT_FOUND` refusal, the old separate invalid-syntax code retired), then
  grants + bootstraps exactly as `register` does. The grant id is read back from the worktree's git metadata after creation,
  never assumed from the directory name. Output names the created path and tells the human to
  open their next session there — a running session is never auto-teleported.
- Slug allowlist `^[a-z0-9][a-z0-9-]*$`; every git call is an argv array (no shell), `--`
  before user-derived values.
- **A worktree carries no runtime of its own, and must not need one.** A linked worktree
  materialises TRACKED files only, and the vendored binary is deliberately untracked
  (machine-local, decision 1f4262ca) — so a worktree's `.bee/bin/` holds prompts and nothing
  else, while its `.claude/settings.json` (tracked) wires all eleven hooks at a path that is
  not there. Every hook then took its visible-fail-open arm and exited 0: no gate enforcement,
  no reservation check, no tier guard, no session preamble. Silent, and unreportable from the
  inside — the component that would have raised the alarm is the one that is missing. Worse,
  the worktree-first routing rule sends code-touching work *into* that state by design.
  The wiring therefore resolves the binary at HOOK time, not at onboard time, and falls back
  to the MAIN checkout via `git rev-parse --path-format=absolute --git-common-dir` — the one
  question git answers usefully from both sides (`--show-toplevel` returns the worktree
  itself). The probe runs only after the direct candidates miss, so an ordinary checkout
  spawns no extra process, and every worktree runs the same binary the main checkout does —
  never a stale copy. Found and closed 2026-08-03.
- **The creation slug is recorded immutably, because the feature name is not.** Directory, branch
  and the worktree's own feature field all derive from one slug at creation — so at that instant
  they agree by construction. The feature field is then freely rewritten afterwards by ordinary
  state handlers that have no worktree awareness at all, and the paved road makes a rename
  near-certain: the worktree-first routing rule (docs/specs/worktree-first.md) tells the agent to
  create the worktree at **feature start**, often before exploring has settled what the feature is
  actually called. Bootstrapping therefore writes
  the creation slug once, write-if-absent, to a record the return path reads **in preference to**
  the mutable field.

  Two properties matter. The record lives on an already-ignored runtime path, deliberately: a
  tracked one would make every worktree read *dirty* to the return path's own uncommitted-work
  pre-check — the same trap that forces the companion-session record to be torn down first. And a
  worktree created before this record existed behaves exactly as it always did: absence degrades
  to reading the mutable field, never to a crash and never to a new refusal.

  Without the record, the return path derived its expected branch from the drifted field and
  refused by naming the **branch** — which is correct, fixed at creation, and the one thing the
  operator must not change. A refusal that names the only unchangeable thing is a dead end, so the
  refusal now names the field that actually drifted and says outright not to rename the branch.
- **A live concurrent session and an undeclared shared nested checkout together refuse the
  creation.** When another session's heartbeat is live for the source checkout, and that
  checkout holds a nested checkout another session could also reach, `worktree new` refuses
  — typed, zero-mutation, no override — unless the call declares its own companion mount for
  that nested checkout. The refusal names declaring a companion mount as the fix. With no other
  session live, or with no such nested checkout present in the source checkout, creation
  proceeds exactly as it always has; declaring a companion mount is likewise never refused by
  this check regardless of concurrency, since a declared mount is itself the fix.
  The concurrency check excludes the acting session's own heartbeat when deciding whether
  another session is live — a solo agent whose own session record is the only one present is
  never mistaken for its own concurrent peer.
- Every refusal is **typed and zero-mutation**: invalid slug/base-ref, caller not an ordinary
  checkout, target path / branch / grant already exists, the live-concurrent-and-undeclared-
  shared-checkout condition above, and git's own `worktree add` failure (the pre-checks are
  advisory; git's atomic failure is authoritative). A failure AFTER the worktree was created
  rolls back best-effort (worktree, branch, grant) and reports typed; if even rollback fails,
  the error names `worktree register` as the adoption path.
- `register` remains for adopting a hand-made worktree; `new` is the paved road.
