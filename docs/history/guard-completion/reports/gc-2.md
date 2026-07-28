# gc-2 — Deny whole-tree git verbs while more than one worker is live

**[DONE]**

Whole-tree git verbs (`reset`, `stash`, `clean`, `checkout`, `restore`, `revert`,
`rebase`, `merge`, `cherry-pick`, `apply`, plus `add` and un-path-scoped
`commit`) are now refused with a typed `git-concurrent-tree` verdict whenever the
computed live-worker view shows more than one worker in this checkout — in every
phase, `swarming` included, with no config opt-out. Read-only inspection, a
path-scoped `git commit -- <paths>`, `git add -N`, `git stash list/show`,
`git apply --check`, and the whole temp-index route stay allowed; a solo session
and the orchestrator's own release/merge work are untouched; an unresolvable
worker count refuses.

Files touched:

- `packages/bee/lib/guards.mjs`
- `packages/bee/tests/test_guards.mjs`
- `packages/bee/hooks/bee-write-guard.mjs` (stale comment only — deviation)

Full trace, deviations, and verification evidence: `.bee/cells/gc-2.json`.
