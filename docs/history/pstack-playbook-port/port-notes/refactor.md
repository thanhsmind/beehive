# Port notes — refactor class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-7. This
file is the only place a pstack-only tool is named; no file under `skills/`
names one. `skills/bee-planning/playbooks/refactor.md` is now the class index
with two situations.

## refactoring

Source `playbooks/refactoring.md`, merged with the bee `refactor` class
playbook (D3) into `skills/bee-planning/playbooks/refactor/refactoring.md`.

Tool swaps:

- The `how` skill (step 1) became `bee-how`, run as a gather through
  `bee dispatch prepare --kind gather`.
- The `subtract-before-you-add` principle citation became
  `bee-principle-subtract-before-you-add`.
- The `migrate-callers-then-delete-legacy-apis` principle citation became
  `bee-principle-migrate-callers-then-delete`.
- Delegating the mechanical edits to a subagent on the named default
  refactoring model became `bee dispatch prepare --kind cell` (the cell goes
  out on the code role's configured model), scope and diff review kept.
- The control skill (the smoke run on the matching surface) became the
  project's `verify-app` skill, or the real interface when the project has
  none.
- The pointer to Feature became "the feature class"; the pointer to the
  figure-it-out skill became "a plan of its own", because the skill does not
  exist in bee yet (restored in ppb-24: the pointer names `bee-figure-it-out`).
- Run Opening a PR became the bee close and worktree merge path, with the
  `uat` acceptance at `bee close` the owner's.

Dropped steps:

- `architect` before a move whose target crosses a function boundary: the
  skill does not exist in bee yet (D5, slice 3). Step 4 keeps the rule: write
  down the contract the target keeps before dispatching. Restored (ppb-24):
  step 4 runs `bee-architect` for the shape before the move.
- The `prove-it-works` principle citation (steps 1 and 6): no bee principle of
  that name exists yet. Step 1 keeps its reason (without a record nothing can
  say the behavior held) and step 8 keeps the real-artifact rule.
- The `model-the-domain` principle citation (step 2, now 3): the same. The
  step keeps the named structures and the "delete branches or invalid states,
  not add indirection" test.
- The `foundational-thinking` and `redesign-from-first-principles` principle
  citations (step 3, now 4): the same. The step keeps "built today".
- The `laziness-protocol` principle citation (step 4, now 5): the same. The
  step keeps "the smallest change that reaches the target shape ships".
- The `minimize-reader-load` principle citation (step 7, now 9): the same. The
  step keeps the reader-load measure.
- The `sequence-verifiable-units` principle skill citation (step 8, now 10): no
  bee principle of that name exists yet. The step keeps "each
  behavior-preserving slice stays green before the next".

Kept from the bee `refactor` playbook (D3): record existing behavior first
with "a type check or a lint is not a record" (step 1), the record proven
green on the unchanged tree (step 2), small steps so a red points at one step
and the record green at every step (step 6), "a behavior change is not a
refactor — it is a separate cell" (opening paragraph), subtract first (step
5), the API move that migrates every caller and deletes the old API in one
slice then searches strings and docs for the old name (step 6), never edit the
record, the harness or the baseline (step 7), keep the diff only if it lowers
what a reader must hold in mind (step 9), the Proof line and the Proof floor
line.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`).

## visual-parity

Source `playbooks/visual-parity.md`, ported without a bee overlap, into
`skills/bee-planning/playbooks/refactor/visual-parity.md`.

Tool swaps:

- The control skill (step 4, image diff on the matching surface) became the
  project's `verify-app` skill, or the real interface when the project has
  none.
- The `separate-before-serializing-shared-state` principle skill became
  `bee-principle-separate-before-serializing-shared-state`.
- Parallelizing across worktrees, one owner per component, became one cell per
  component through `bee dispatch prepare --kind cell`.
- Run Opening a PR per component or per safe batch became the bee close and
  worktree merge path per component or batch, with the `uat` acceptance at
  `bee close` the owner's.

Dropped steps:

- The Cursor `/loop` per component until the diff is zero (step 4): bee has no
  counterpart for a looping command inside a session. The step keeps "rerun
  the diff per component until it is zero".

Added: a Proof line (the image-diff result for each component against its
baseline), because the pstack text had only a Reply; and a reason on every
step where pstack gave none (decision `61b52e96`).

## Steps re-added in ppb-24

Restored: `architect` (refactoring step 4, now `bee-architect`) and
figure-it-out (the pointer for large or cross-cutting structural work, now
`bee-figure-it-out`).

Still waiting, because no skill of that name exists in `skills/`:
`prove-it-works`, `model-the-domain`, `foundational-thinking`,
`redesign-from-first-principles`, `laziness-protocol`, `minimize-reader-load`
and `sequence-verifiable-units` (refactoring). Each keeps its method as a plain
sentence.
