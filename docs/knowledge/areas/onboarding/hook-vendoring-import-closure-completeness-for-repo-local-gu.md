---
type: bee.area
title: "Onboarding — hook vendoring import closure, closed by the Node runtime's removal"
description: "Why a vendored hook once had to ship with every module it imports, the fresh-install crash that taught it, and why the rule has no live surface since R6: onboarding vendors no .mjs hook, and HOOK_FILENAMES now only names the stale hook files an apply deletes."
timestamp: 2026-07-24
bee:
  id: onboarding-hook-vendoring-import-closure-completeness
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["bee6fcb2 (i54-closeout Gate 3 approval, naming the canary-caught fresh-install write-guard crash)", 103a5608 (i54-closeout scope lock)]
  sources: ["i54-closeout cell i54-closeout-9 (fresh-install write-guard crash fix: vendor tokenize-command.mjs + import-closure regression suite; trace in .bee/cells/, 2026-07-24)", "docs/history/i54-closeout/reports/validation-canary.md section 1 (P5 red, real bug: ERR_MODULE_NOT_FOUND on a fresh repo-hooks install)", "commit 5c62cad02 (R6: delete the Node runtime)", packages/bee-rs/crates/bee/src/onboard/templates.rs, packages/bee-rs/crates/bee/src/onboard/plan.rs]
  authoritative_for: "onboarding: hook vendoring import-closure completeness"
  anchors: [packages/bee-rs/crates/bee/src/onboard/templates.rs, packages/bee-rs/crates/bee/src/onboard/plan.rs]
  verified_at: 2026-11-25
  verified_commit: 1de3a4bfff52f6ac99fdc9e5524e24b24a1f762a
---

# Onboarding — Hook Vendoring Import Closure, Closed by the Node Runtime's Removal

Read when: a host still carries `.bee/bin/hooks/*.mjs`, an onboard plan shows
`remove_repo_hook` items, or someone proposes vendoring a script hook again.

Vendoring a hook file is not the same as vendoring the hook: a hook that imports
a sibling module ships broken the moment that sibling is missing, and the break
only shows up the first time the hook fires. That rule governed the Node hook
set. Since R6 deleted the Node runtime, bee's hooks run inside the `bee` binary,
so no vendored hook has a sibling import to miss.

## Behaviors & Operations

**Today: onboarding vendors no `.mjs` hook.** `HOOK_FILENAMES` in
`onboard/templates.rs` still lists the eleven old hook names, including
`bee-write-guard.mjs` and `tokenize-command.mjs`. `list_plugin_hooks` in
`onboard/plan.rs` keeps only the names that exist in the engine's
`packages/bee/hooks/` directory. That directory now holds no `.mjs` file, so
the list is empty and nothing is copied.

**Today: an apply deletes stale vendored hooks.** The plan takes the hook names
recorded under `managed.repo_hooks` and `managed.codex_hooks` in the previous
`.bee/onboarding.json`, drops the names the current source still lists, and
plans one `remove_repo_hook` item for each one that still exists under
`.bee/bin/hooks/`. A host that never vendored hooks gets no item.

**History: the crash that set the rule (cell i54-closeout-9).** The vendored
list once named `bee-write-guard.mjs` without `tokenize-command.mjs`, a module
that hook imports. A fresh `--repo-hooks` install copied the hook but not its
dependency, and the guard crashed with `ERR_MODULE_NOT_FOUND` the first time it
ran — fail-open, so the write it existed to block went through. The fix made
the list import-closed and added a regression suite that parsed each listed
hook's relative imports and ran the vendored guard as a real child process.
That suite (`scripts/tests/test_hook_vendor_closure.mjs`) was retired with the
Node runtime on 2026-08-02.

## Business Rules

- A vendored script that imports a sibling ships only with that sibling. Any
  future vendored script hook needs a mechanical import-closure check, never a
  hand-kept list.
- A syntax check (`node --check`) is never proof that a module's imports
  resolve; only a real subprocess run or a module-resolution check is. The
  general lesson lives in
  [the vendoring-drift pattern](../../patterns/20260724-canonical-source-tests-cannot-see-vendoring-drift.md).

## Pointers (implementation)

- Legacy hook list: `HOOK_FILENAMES` in `packages/bee-rs/crates/bee/src/onboard/templates.rs`.
- Existence filter: `list_plugin_hooks` in `packages/bee-rs/crates/bee/src/onboard/plan.rs`.
- Stale-hook removal: the `remove_repo_hook` plan item in `onboard/plan.rs`.
- Evidence: cell i54-closeout-9 (trace purged from .bee/cells/),
  `docs/history/i54-closeout/reports/validation-canary.md` (section 1, P5).
