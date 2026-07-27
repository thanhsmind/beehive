---
type: bee.pattern
title: "A fresh worktree inherits every other feature's stale claimed cells because .bee/cells/ is git-tracked"
description: "A fresh worktree inherits every other feature's stale claimed cells because .bee/cells/ is git-tracked"
tags: [worktree, cells, git-tracked-state, scaffolding-bug]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-worktree-inherits-stale-cells
  lifecycle: active
  sources: ["worktree-concurrency-guard session (2026-07-24): rel1150-1 (release-1-15-0, unrelated feature) blocked state start-feature in TWO separately created fresh worktrees before this feature could begin; filed as its own PBI p-9c48a67c, feature worktree-scaffolding-cell-leak"]
  polarity: pitfall
  critical: true
---

# A fresh worktree inherits every other feature's stale claimed cells because .bee/cells/ is git-tracked

`bee worktree new` creates a fresh linked worktree via an ordinary
`git worktree add`. Because `.bee/cells/` is a git-tracked directory, that
checkout brings over every cell file that existed in the source commit —
including a cell from a completely unrelated feature that was claimed but
never capped. The new worktree's own `state start-feature` call then refuses
immediately: it sees a claimed cell it knows nothing about and has no
mandate to touch. This is not a rare edge case — it happened twice in one
session, blocking a brand-new feature from even starting, purely because an
older feature elsewhere in the repo's history had left one cell claimed and
uncapped at the moment a worktree happened to branch from that commit.

**The tell:** a freshly created worktree's very first `state start-feature`
call refuses with "claimed cell(s) remain," naming a cell whose `feature`
field belongs to something the new session never touched.

**The workaround used both times:** `bee cells drop --id <stale-id> --reason
"..."` in the new worktree, citing that the cell is already resolved/committed
in the worktree's own base commit and is simply a stale local-state artifact,
not real remaining work. This is safe but manual, and it only fixes the
symptom in the one worktree it's run in — the same stale cell reappears in
the next worktree branched from a commit where it's still claimed.

**The generalizable gap:** `bootstrapWorktreeStore`
(`worktree-store.mjs:334-368`) only copy-if-absents `onboarding.json`,
`config.json`, and `state.json` when seeding a new worktree's `.bee/` store —
it never scopes, filters, or clears `cells/`. The real fix (tracked as its
own backlog item, `p-9c48a67c`, feature `worktree-scaffolding-cell-leak`) is
in git-tracking or checkout-scoping of `.bee/cells/`, not a one-off `cells
drop` — until it ships, every `worktree new` remains liable to re-surface
this exact block whenever the source commit has any claimed-but-uncapped
cell anywhere in the repo's history.
