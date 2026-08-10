---
type: bee.pattern
title: "A feature is not a release: re-inspect the whole working tree at close, and never let the version tuple move without explicit release intent"
description: "A feature is not a release: re-inspect the whole working tree at close, and never let the version tuple move without explicit release intent"
tags: [release-safety, version-tuple, silent-drift, glob-scoped-files, orchestrator-reverify, render-manifest-order]
timestamp: 2026-07-20
bee:
  id: pattern-20260720-a-feature-is-not-a-release-re-inspect
  lifecycle: active
  decisions: [0018]
  sources: ["docs/history/learnings/critical-patterns.md#PAT43", "original feature: transcript-recovery"]
  polarity: pitfall
  critical: false
---

# A feature is not a release: re-inspect the whole working tree at close, and never let the version tuple move without explicit release intent

A cell-4 step ran `bump_version.mjs 1.7.6` unrequested; the 4-member release tuple
(`state.mjs` x2 + both `plugin.json`) split — the two plugin manifests matched the cell's
broad `.claude-plugin/*`/`.codex-plugin/*` globs and committed at 1.7.6, the two `state.mjs`
members matched no glob and stayed uncommitted, invisible in both the commit diff and the
worker's `[DONE]` report. Only a fresh orchestrator-run `git status` at feature-close caught
it. Three rules: **(1)** at every feature close, independently re-inspect the FULL working
tree (`git status --short` + a `bump_version.mjs --check`), never trust the worker's file
list — the goal-check discipline (decision 0018) applies to the whole tree, not just the
cell verify. **(2)** the release-version tuple moves ONLY in an explicit `release X.Y.Z`
commit; if a non-release feature drifted it, restore with `bump_version.mjs <last-released>`.
**(3)** regenerate `release-manifest.json` only AFTER the final render — a manifest written
before a `.bee-render.json` sha changes goes red on `plugin_distribution`. Order: edit →
render trees → onboard sync → `release_manifest --write` → `--check`. Mechanization filed
(backlog: cap-time tuple-drift guard).
