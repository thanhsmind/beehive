---
type: bee.area
title: Onboarding — hook vendoring import-closure completeness
description: "Why a fresh repo-hooks onboard must vendor every module a vendored hook imports, transitively, not merely the hook entrypoints themselves, and the regression suite that guards the class of bug rather than the one instance."
timestamp: 2026-07-24
bee:
  id: onboarding-hook-vendoring-import-closure-completeness
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["bee6fcb2 (i54-closeout Gate 3 approval, naming the canary-caught fresh-install write-guard crash)", 103a5608 (i54-closeout scope lock)]
  sources: ["i54-closeout cell i54-closeout-9 (fresh-install write-guard crash fix: vendor tokenize-command.mjs + import-closure regression suite; trace in .bee/cells/, 2026-07-24)", "docs/history/i54-closeout/reports/validation-canary.md section 1 (P5 red, real bug: ERR_MODULE_NOT_FOUND on a fresh repo-hooks install)"]
  authoritative_for: "onboarding: hook vendoring import-closure completeness"
---

# Onboarding — Hook Vendoring Import-Closure Completeness

Vendoring a hook file is not the same as vendoring the hook: a hook that imports
a sibling module ships broken the moment that sibling is missing, and the break
only shows up the first time the hook actually fires. This concept owns the rule
that closes that gap and the suite that keeps it closed.

## Behaviors & Operations

**A fresh repo-hooks onboard vendors every module a vendored hook imports,
transitively, not just the hook entrypoints named in the vendored-hook list.**
The vendored-hook list used to name `bee-write-guard.mjs` without naming
`tokenize-command.mjs`, a module that hook imports directly. A fresh
`--repo-hooks` install therefore copied the hook file but not its dependency,
and the vendored guard crashed with a module-not-found error the first time it
ran — silently fail-open, so the write it existed to block went through
unguarded. The fix is not "add the one missing filename": the vendored-hook
list must now be import-closed — every relative import of a listed hook is
itself listed — so the same class of gap (a hook gains a new sibling import
tomorrow and nobody remembers to vendor it) is caught the next time it happens,
not just this one instance.

**The regression suite proves the class, not the instance.** A static
import-closure parse walks the vendored-hook list's declared entrypoints and
asserts every relative import they make is itself a listed entry — this is what
would have caught the gap before a fresh install ever hit it. A companion
fresh-install/subprocess proof actually runs the vendored guard as a real child
process and asserts the specific crash signature (`ERR_MODULE_NOT_FOUND`)
is absent — a syntax check alone (`node --check`) is not sufficient evidence,
because it is syntax-only and does not detect a missing relative-import module.

## Business Rules

- The vendored-hook list must be import-closed: every relative import made by a
  listed hook file is itself a listed entry, checked mechanically rather than
  trusted to a human remembering to update it by hand.
- A syntax check (`node --check`) is never accepted as proof that a vendored
  module's imports resolve; only a real subprocess run (or an equivalent
  module-resolution check) proves an import closure complete.

## Pointers (implementation)

- Vendored-hook list: `HOOK_FILENAMES` in
  `packages/bee/scripts/onboard_bee.mjs`.
- Regression suite: `scripts/tests/test_hook_vendor_closure.mjs` (static
  import-closure parse + fresh-install/subprocess proof).
- Evidence: `.bee/cells/i54-closeout-9.json`,
  `docs/history/i54-closeout/reports/validation-canary.md` (section 1, P5).

