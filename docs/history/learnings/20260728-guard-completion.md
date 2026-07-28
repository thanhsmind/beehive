---
date: 2026-07-28
feature: guard-completion
categories: [orchestration, workflow-state, verify-pipeline]
severity: high
tags: [unbypassable-guards, git-safety, boundary-set, ci-division-of-labour]
---

# guard-completion — the last open P1, and prose becoming enforcement

## What Happened

Three things closed here, all of them "a rule that existed but nothing enforced":

1. **The last open P1 from the review session.** The feature-swap door asked two of
   three debt questions. The fix was not a third call: `FEATURE_DEBT_KINDS` +
   `guardFeatureDebt` now own the whole debt set, every door supplies only a refusal
   head, and the tests iterate `DEBT_DOORS × FEATURE_DEBT_KINDS` — a debt kind added
   later is inherited by every door and by the coverage. A structural check now refuses
   any per-kind detector call in the caller, so the old shape cannot come back.
2. **Worker git safety became a hook.** Whole-tree verbs (reset/stash/clean/checkout/
   restore/revert/rebase/merge/cherry-pick/apply), plus `git add` and an unscoped
   `git commit`, are refused while more than one worker is live. Read-only git, a
   path-scoped `git commit -- <paths>`, `git add -N`, and the temp-index route stay
   allowed; solo sessions and the orchestrator's release work are untouched.
3. **Two bee-scribing routing rules restored** after a body compression dropped them.

## The finding inside the finding

The git-guard cell was specified — by me — to key on the live-worker count. The worker
found that would not have caught the incident it exists to prevent: **subagents in one
session share a session id and heartbeat, so a three-worker wave reports as one.** The
shipped guard unions active reservation agents with live sessions, de-duped and filtered
by workspace. A guard written to my spec would have passed its tests and stopped nothing.

*Rule: when specifying a guard, name the signal it must detect, not the field you think
carries it — and expect the implementer to check whether that field actually moves.*

## Where the boundary verify failed, and CI did not

Windows CI caught a prose regression (`test_bundle_mode`) that the feature verify missed,
because **the boundary set is chosen by orchestrator judgment** and that suite was not in
it. The split of labour worked exactly as designed — the full chain is CI-owned and it
found what the local subset could not — but the local subset should not be a guess.
Filed as friction: derive the boundary set from the feature's changed paths through the
impact registry, plus every prose-pinning suite whenever `skills/**` or `AGENTS.md`
changed.

Separately confirmed: splitting `test_worktree_store` **did** fix the Windows timeout —
the run completed in 11m34s with no TIMEOUT, and the only failure was the unrelated prose
regression above.

## Recommendation

- **A guard that keeps needing another call site is the wrong shape.** Three rounds on
  this surface ended only when the question became singular and the tests derived from
  the registry rather than a hand-written list.
- **Prose rules survive exactly as long as nothing enforces them.** The worker git rules
  were written after three incidents, were obeyed by every worker that read them, and
  still needed a hook — because the next worker has not read this session's history.
