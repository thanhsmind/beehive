# Port notes — perf class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-5. This
file is the only place a pstack-only tool is named; no ported file names one. `skills/bee-planning/playbooks/perf.md` is now the class index with
two situations.

## perf-issue

Source `playbooks/perf-issue.md`, merged with the bee `perf` class playbook
(D3) into `skills/bee-planning/playbooks/perf/perf-issue.md`.

Tool swaps:

- The control skill (baseline trace, Capture a baseline) became the project's
  `verify-app` skill, or the real interface when the project has none.
- The `how` skill became `bee-how`, run as a gather through
  `bee dispatch prepare --kind gather`; the Elimination family keeps its
  line that it needs the `bee-how` pass, not the profiler.
- Delegating the implementation to a subagent on the named default perf-issue
  model became `bee dispatch prepare --kind cell` (the cell goes out on the
  code role's configured model).
- Cite the measurement in the PR, then Opening a PR, became one step: cite the
  measurement in the capped result and land it through the bee close and
  worktree merge path, with the `uat` acceptance at `bee close` the owner's.
- The pointer to the Hillclimb playbook (`playbooks/hillclimb.md`) is now the
  class index line for `hillclimb`, so the situation file no longer carries it.
- The "decision trail" of the old step 6 (record both numbers) now names the
  feature's trail under `docs/history/<feature>/`.

Dropped steps:

- `architect` before a fix that crosses a function boundary: the skill does
  not exist in bee yet (D5, slice 3). The step keeps the rule: write down the
  contract the fix keeps before dispatching. Restored (ppb-24): step 4 runs
  `bee-architect` first.
- The `sequence-verifiable-units` principle skill citation: no bee principle
  of that name exists yet. The sentence stays without the name: verify each attempt before trying the next.
  Restored (ppb-37): perf-issue step 4 names `bee-principle-sequence-verifiable-units`.

Kept from the bee `perf` playbook (D3): the decision-and-noise-floor step
written before the first run, the baseline step, the mechanism hypothesis with
the deleted-path question, the one-change step, the same-way re-measure, keep
the win and record both numbers, the "It feels faster" line with decision
`1593e365`, the Proof line and the Proof floor line.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), and a Reply line kept from pstack.

## hillclimb

Source `playbooks/hillclimb.md`, merged with the sustained-target paragraph of
the bee `perf` playbook (D3) into
`skills/bee-planning/playbooks/perf/hillclimb.md`.

Tool swaps:

- The `how` skill became `bee-how`, run as a gather.
- The `build-the-lever` principle skill became `bee-principle-build-the-lever`.
- The `separate-before-serializing-shared-state` principle skill became
  `bee-principle-separate-before-serializing-shared-state`.
- The `show-me-your-work` decision log (`decision.tsv`, gitignored) became the
  decision trail under `docs/history/<feature>/`, with the same columns
  (restored in ppb-24: step 3 opens it with `bee-show-me-your-work`); the
  gitignore rule became "keep the trail file outside the files an attempt
  touches", so a revert still never removes it.
- Handing the change to a subagent on the named default hillclimb model became
  `bee dispatch prepare --kind cell` on the code role's configured model;
  parallel hypotheses fan to parallel cells, each in its own worktree.
- The Autonomous run playbook pointer became
  `playbooks/session/autonomous-run.md`.
- Run Opening a PR with the stacked commits became the bee close and worktree
  merge path, commits in the order they landed.

Dropped steps:

- The `prove-it-works` principle citation (Core discipline): no bee principle
  of that name exists yet (D5, slice 3). The sentence keeps its reason: the data decides.
  Restored (ppb-37): hillclimb names `bee-principle-prove-it-works`.
- The `guard-the-context-window` principle citation (step 5): the same. The step keeps the reason: the lead's context stays on the metric.
  Restored (ppb-37): hillclimb step 5 names `bee-principle-guard-the-context-window`.
- The `sequence-verifiable-units` principle citation (step 5): the same. The step keeps "each iteration ends in a check before the next begins".
  Restored (ppb-37): hillclimb step 5 names `bee-principle-sequence-verifiable-units`.
- The `laziness-protocol` principle citation (step 6): the same. The step keeps the correctness-and-simplicity rule.
  Restored (ppb-37): hillclimb step 6 names `bee-principle-laziness-protocol`.

Moved in from the bee `perf` playbook (D3): the sustained-target paragraph's
rules (harness separates the target case then freezes, median of several runs,
stop rule set before the first attempt, one row per attempt, revert an attempt
that does not clear the noise, never loosen the stop rule) are steps 1 to 3, 5
and 7; the plateau sentence "A plateau is not a stop — it means pivot the
hypothesis, not give up." is in step 6; the "It feels faster" line, the Proof
line and the Proof floor line carry over.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`).

## Steps re-added in ppb-24

Restored: `architect` (perf-issue step 4, now `bee-architect`) and
`show-me-your-work` (hillclimb step 3, now `bee-show-me-your-work`).

Still waiting, because no skill of that name exists in `skills/`:
`prove-it-works`, `guard-the-context-window`, `sequence-verifiable-units` and
`laziness-protocol` (hillclimb and perf-issue). Each keeps its method as a
plain sentence.
