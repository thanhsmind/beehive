---
type: bee.pattern
title: Inside a worktree the tracked copy of a runtime store answers stale
description: "A feature worktree carries its own git-tracked copy of .bee/decisions.jsonl, frozen at whatever the branch last committed while the live store is the control root's, so a reader standing inside the worktree gets a confidently wrong answer to \"does decision X exist\" — two independent judges hit this within one hour, and the second issued a false NEEDS_REVISION finding that three architectural decisions had no record behind their cited hashes, when the decisions had existed the whole time."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-worktree-tracked-store-answers-stale
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  sources: [capture stub 8f89c13e (herding-orchestration), docs/history/herding-orchestration/CONTEXT.md]
---

A feature worktree carries its own git-tracked copy of
`.bee/decisions.jsonl`, frozen at whatever the branch last committed,
while the live store is the control root's. A reader standing inside
the worktree gets a confidently wrong answer to "does decision X
exist": the file is present, parses fine, and simply lacks every
decision logged since the branch point. The staleness is by
construction, not by accident — the tracked copy cannot contain what
was logged after the branch diverged, so the ambient answer inside a
worktree is wrong exactly when the question matters.

Two independent judges hit this within one hour. The first noticed
the worktree boundary and re-checked the control root before ruling.
The second did not, and issued a false NEEDS_REVISION finding that
three architectural decisions had no record behind their cited
hashes. The decisions existed the whole time, in the live store the
judge never looked at.

The sting in the tail: the false finding still pointed at a true
defect. The three decisions were in the store but had never been
added to CONTEXT.md's locked table — and the locked table is the
record downstream cells actually cite. Logging a decision and locking
it are two different acts, and only the second is visible to the next
worker. Clearing the false half of the finding without asking what it
was pointing at would have left the real gap in place.

**The rule:** any check of the form "does this recorded thing exist"
resolves the control root explicitly — `git rev-parse
--git-common-dir` — and reads the store there, never through the
ambient cwd, because inside a worktree the ambient answer is stale by
construction. And when a finding proves false, ask what true defect
it was aimed at before dismissing it: here the record existed but was
never locked into CONTEXT.md, and locking, not logging, is what the
next worker sees.
