---
date: 2026-07-26
feature: packages-restructure
categories: [refactor, migration, validation, cli]
severity: mixed
tags: [path-migration, manifest-roles, adversarial-validation, regen-tooling, cli-flags]
---

# packages-restructure — learnings

## What Happened

The bee vendor payload moved out of `skills/bee-hive/templates/` (and repo-root `hooks/`) into `packages/bee/`, making `packages/` the standard install source and `skills/` instruction-only (engine excepted, D4). 4 serial cells; the adversarial validation pass caught 8 BLOCKER + 8 CRITICAL findings *before* any code moved, all repaired in-place via `cells update`. Execution then landed clean: 4/4 cells capped, full suite green after every cell, zero-diff proof cell confirmed the distribution surface settled.

## Root Causes and Findings

1. **Inventory-by-sample vs the real surface.** Plan discovery named 4 "misc" test files with hand counts; the true surface was ~48 files with live path resolution (validation B2/C2), and cell 1's cascade ultimately touched 493 files. Manual ref-counts on the files one *thought of* are not a surface; only a repo-wide `rg` of the literal old path is.
2. **The migration's own proof tooling was an unlisted consumer of the migration.** `render_plugin_skill_trees.mjs` — the first mandatory regen command of cell 1 — statically imported from the tree cell 1 moves; unfixed, the cell's own regen obligation would have crashed at step 1 (B1/C1). ESM static imports resolve at load time: a generator that proves a move can itself be broken by the move.
3. **Payload must never vanish between cells.** The manifest role for the moved tree originally landed one cell after the move (C5); any release cut between cells would have shipped an empty payload. Role/registry changes ride the same cell as the move they describe (logged as D6).
4. **Second-order break found mid-cell, repaired in-scope.** D1 (skills instruction-only) silently broke onboarding's three-version preflight for already-onboarded hosts. Worker kevin surfaced it through the cell's own verify, consulted the advisor once, and added `SKILLS_VERSION_STAMP` (`.bee-skills-version.json`, legacy-marker fallback, malformed ⇒ unknown never-fallback). Legitimate deviation because: the cell's verify forced it, the fix is in the direct closure of the change, and the worker escalated instead of improvising past 2 failed mechanical attempts.
5. **CLI verb accepted an unknown flag and exited 0.** `capture add --text ...` silently wrote nothing — two settlements nearly died in chat. Strict-flag validation exists in the codebase (`update`, `worker prune` throw on unknown flags) but is opt-in per verb. Friction filed for allow-list enforcement across all verbs.
6. **Artifact drift between the cell store and its draft file.** Validation repairs went through `cells update` (store correct), but the draft `cells-slice1.json` in docs/history kept the old narrow verify — an analyst later misread it as the live contract. The store is authoritative; a draft that stops being maintained should say so or be deleted.

## Recommendations

- **When X = renaming/moving a path prefix**, do: record repo-wide `rg '<old-path>'` output in discovery *before* locking the plan, and write every related cell's acceptance as "repo-wide rg returns 0 hits" — never a named-file checklist.
- **When X = a cell whose regen/verify obligation invokes a generator**, do: put the generator's source (and its import closure) in the cell's `files` and check it for references to the tree being moved before writing the `git mv` step.
- **When X = moving a tree that a manifest/registry enumerates**, do: land the enumeration change in the same cell as the move — every cell boundary is a potential ship point.
- **When X = a worker hits breakage caused by the cell's own change**, do: allow the in-scope repair only when the cell's verify forces it AND the fix is in the change's direct closure AND the worker escalates after 2 failed attempts; otherwise it is scope creep.
- **When X = adding any CLI verb**, do: validate flags against an allow-list and throw on unknown flags — exit 0 on an ignored flag converts caller typos into silent data loss.

## Suite census (test-economy D4)

- Suites in registry: 105 (unchanged through the feature).
- The feature added no new suites; test-line delta is path-repoints only (+ fixture restructuring in `test_onboard_bee.mjs`).
