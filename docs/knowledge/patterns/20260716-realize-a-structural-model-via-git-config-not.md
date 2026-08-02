---
type: bee.pattern
title: "Realize a structural model via git config, not a file migration, when the boundaries already exist"
description: "Realize a structural model via git config, not a file migration, when the boundaries already exist"
tags: [pattern, tiering, gitignore, gitattributes, no-migration]
timestamp: 2026-07-16
bee:
  id: pattern-20260716-realize-a-structural-model-via-git-config-not
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT3", "original feature: worktree-feature-parallelism"]
  polarity: practice
  critical: true
---

# Realize a structural model via git config, not a file migration, when the boundaries already exist

The "three-tier `.bee/` store" (log / cache / runtime) sounded like a directory restructure, but
beehive's flat store already had the boundaries: logs tracked, cache/runtime gitignored. The tiers
were realized as a LOGICAL classification — `.gitattributes merge=union` on the tracked log jsonl
(so worktree branches union-merge provenance) plus gitignore entries for the runtime/cache dirs —
moving zero files. Before migrating a layout to match a model, check whether the model is already
expressible as config over the existing layout. Corollary (list-rot, AGAIN): the onboarding
gitignore block has a hand-kept twin in `test_onboard_bee` (an independent sha256 reconstruction);
adding one pattern to the source silently reddened the test until the twin was updated — the same
"hardcoded fixture list rots" failure from 20260714/20260715, third recurrence. Derive the twin
from the source, or expect to update both every time.
