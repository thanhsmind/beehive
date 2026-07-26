---
type: bee.pattern
title: "A migration's own regen and proof tooling is a consumer of the migration — inventory it before the move"
description: "The first mandatory regen command of the move cell statically imported from the tree being moved; unfixed, the cell's own regen obligation would have crashed at its first step. ESM static imports resolve at load time, so a generator that proves a move can itself be broken by the move — and the companion rule: what a manifest enumerates must change in the same cell as the move, because every cell boundary is a potential ship point."
tags: [migration, path-move, regen, manifest, tooling]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-migration-tooling-is-a-consumer-of-the-migration
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["packages-restructure (validation B1/C1: render_plugin_skill_trees.mjs imported ../skills/bee-hive/templates/lib/lock.mjs while cell 1 moved that tree; C5: package_payload manifest role originally one cell late)", docs/history/packages-restructure/reports/validation-slice1.md, docs/history/learnings/20260726-packages-restructure.md]
---

## The pattern

When a cell moves or renames a tree, two classes of dependents hide outside the obvious sweep:

1. **The tooling the cell itself must run.** Regen obligations and verify commands invoke generators; a generator that statically imports from the moved tree crashes at load time, before any fix-forward can run. The generator's source and its import closure belong in the cell's files list, checked for old-path references before the `git mv` step is even written.
2. **What a manifest or registry enumerates.** If the enumeration change lands a cell later than the move, every build cut between the two cells ships a payload-less manifest. The enumeration rides the same cell as the move, always.

Acceptance for the whole move is a repo-wide `rg` of the literal old path returning zero hits — never a named-file checklist, which is inventory-by-sample.
