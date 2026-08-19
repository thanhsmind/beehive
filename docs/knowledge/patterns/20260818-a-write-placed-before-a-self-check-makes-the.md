---
type: bee.pattern
title: A write placed before a self-check makes the check accuse itself
description: A write placed before a self-check makes the check accuse itself
tags: [failure, guards, tests, fixtures]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-a-write-placed-before-a-self-check-makes-the
  lifecycle: active
  areas: [worktree-parallelism]
  sources: [".bee/cells/mcl-2.json", "original feature: merge-closes-the-lane"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs (a_green_merge_of_a_tracked_lane_file_emits_no_mutated_tracked_files_warning — the fixture force-tracks its lane file, because an untracked one is invisible to the guard)"
---

# A write placed before a self-check makes the check accuse itself

`bee worktree merge` gained a deliberate lane write, placed just after the merge
commit — and just BEFORE the post-commit guard that reads `git status --porcelain
--untracked-files=no` and warns when something modified tracked files after the commit
landed. Lane records are tracked in this repo, so the guard read the merge's own write
and every clean merge printed `verify_mutated_tracked_files` at itself. The suite was
green the whole time.

Two lessons, one cheap and one expensive.

Cheap: a self-check measures what OTHERS did. Every deliberate write of your own belongs
after it, never before — otherwise the check's first finding is always you, and the one
real signal it exists to catch is buried under a warning that fires every run.

Expensive: the guard filtered on a property — tracked, not untracked. The cell's three
new tests all built their fixture lane file as an untracked file, so the guard could not
see it and the defect was invisible to a passing suite. When a check selects by a
property (tracked, ignored, staged, non-empty, above a threshold), a fixture that lacks
that property is not a weak test — it is a test of nothing, and it reads as proof. The
fixture's own state is part of the subject, not setup detail. It took an independent
semantic judge reading the production path to find this; no amount of green told anyone.
