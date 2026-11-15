# Port notes — principle pointers restored in ppb-37

Each pointer below was dropped by an earlier port cell because its bee
principle did not exist yet, and the method stayed as a plain sentence. The
slice 4 principles exist now, so the pointer is back as `bee-principle-<slug>`
beside the sentence it came from. Each source entry carries a "Restored
(ppb-37)" mark in its port note. No situation or skill was added or removed, and
no step was rewritten: the pointer joined the existing sentence.

Playbook paths are under `skills/bee-planning/playbooks/`. Skill paths are under
`skills/`.

## Restored, by file

| File | Where | Principle slug | Port note that dropped it |
|------|-------|----------------|---------------------------|
| `session/session-pickup.md` | Step 1, the gather that keeps the transcript out of the lead's thread | `bee-principle-guard-the-context-window` | `session-ppb-11.md` |
| `session/session-pickup.md` | Step 5, verify on the real artifact | `bee-principle-prove-it-works` | `session-ppb-11.md` |
| `session/autonomous-run.md` | Step 3, verifiable units | `bee-principle-sequence-verifiable-units` | `session-ppb-11.md` |
| `session/worktree-cleanup.md` | Step 2, the open sessions are the real artifact | `bee-principle-prove-it-works` | `session-ppb-11.md` |
| `session/worktree-cleanup.md` | Step 3, the gather's output kept to a digest | `bee-principle-guard-the-context-window` | `session-ppb-11.md` |
| `session/autopilot-full.md` | Step 2, owner self-proof | `bee-principle-prove-it-works` | `session-ppb-12.md` |
| `bugfix/bug-fix.md` | Step 6, failing repro before the fix in the commits | `bee-principle-sequence-verifiable-units` | `bug-fix` section of `port-notes.md` |
| `feature/feature.md` | Step 2, the organizing structure | `bee-principle-model-the-domain` | `feature.md` |
| `feature/feature.md` | Step 5, delegation is mandatory | `bee-principle-laziness-protocol` | `feature.md` |
| `feature/feature.md` | Step 9, build, verify and commit each unit | `bee-principle-sequence-verifiable-units` | `feature.md` |
| `feature/multi-phase-plan.md` | Step 2, give options | `bee-principle-never-block-on-the-human` | `feature.md` |
| `feature/multi-phase-plan.md` | Step 3, the gather keeps the lead's context | `bee-principle-guard-the-context-window` | `feature.md` |
| `feature/multi-phase-plan.md` | Step 5, one PR is one change with its own evidence | `bee-principle-sequence-verifiable-units` | `feature.md` |
| `feature/multi-phase-plan.md` | Step 7, a rule left to memory is skipped | `bee-principle-encode-lessons-in-structure` | `feature.md` |
| `feature/multi-phase-plan.md` | Verification paragraph | `bee-principle-prove-it-works` | `feature.md` |
| `refactor/refactoring.md` | Step 1, the record makes the refactor checkable | `bee-principle-prove-it-works` | `refactor.md` |
| `refactor/refactoring.md` | Step 3, the missing structure | `bee-principle-model-the-domain` | `refactor.md` |
| `refactor/refactoring.md` | Step 4, the target shape | `bee-principle-foundational-thinking`, `bee-principle-redesign-from-first-principles` | `refactor.md` |
| `refactor/refactoring.md` | Step 5, the smallest change that reaches the target | `bee-principle-laziness-protocol` | `refactor.md` |
| `refactor/refactoring.md` | Step 8, prove behavior unchanged | `bee-principle-prove-it-works` | `refactor.md` |
| `refactor/refactoring.md` | Step 9, reader load | `bee-principle-minimize-reader-load` | `refactor.md` |
| `refactor/refactoring.md` | Step 10, each slice green before the next | `bee-principle-sequence-verifiable-units` | `refactor.md` |
| `perf/perf-issue.md` | Step 4, verify each attempt | `bee-principle-sequence-verifiable-units` | `perf.md` |
| `perf/hillclimb.md` | Core discipline, the data decides | `bee-principle-prove-it-works` | `perf.md` |
| `perf/hillclimb.md` | Step 5, the lead's context stays on the metric | `bee-principle-guard-the-context-window` | `perf.md` |
| `perf/hillclimb.md` | Step 5, each iteration ends in a check | `bee-principle-sequence-verifiable-units` | `perf.md` |
| `perf/hillclimb.md` | Step 6, correctness and simplicity outrank the number | `bee-principle-laziness-protocol` | `perf.md` |
| `research/runtime-forensics.md` | Step 2, the reduced finding in the lead's thread | `bee-principle-guard-the-context-window` | `research.md` |
| `research/trace-forensics.md` | Step 1, the reduced finding in the lead's thread | `bee-principle-guard-the-context-window` | `research.md` |
| `spike/prototype.md` | Intro, the smallest-change rule inverts | `bee-principle-laziness-protocol` | `spike.md` |
| `spike/prototype.md` | Step 5, all variants behind one switcher | `bee-principle-exhaust-the-design-space` | `spike.md` |
| `bee-arena/SKILL.md` | Phase D tie-break | `bee-principle-laziness-protocol` | `skill-bee-arena.md` |
| `bee-arena/SKILL.md` | Phase E, folding in a graft | `bee-principle-redesign-from-first-principles` | `skill-bee-arena.md` |
| `bee-arena/SKILL.md` | Phase F, report what the run showed | `bee-principle-prove-it-works` | `skill-bee-arena.md` |
| `bee-architect/SKILL.md` | Design it twice | `bee-principle-exhaust-the-design-space` | `skill-bee-architect.md` |
| `bee-architect/SKILL.md` | Phase C, scaffold-first | `bee-principle-foundational-thinking` | `skill-bee-architect.md` |
| `bee-architect/SKILL.md` | Phase C, the outcome is held fixed | `bee-principle-outcome-oriented-execution` | `skill-bee-architect.md` |
| `bee-architect/SKILL.md` | Phase E, scrap the sketch | `bee-principle-redesign-from-first-principles`, `bee-principle-crash-site-versus-fault-site` | `skill-bee-architect.md` |
| `bee-architect/references/runner-prompt.md` | Idempotence bullet | `bee-principle-make-operations-idempotent` | `skill-bee-architect.md` |
| `bee-architect/references/runner-prompt.md` | Short call chains bullet | `bee-principle-laziness-protocol`, `bee-principle-minimize-reader-load` | `skill-bee-architect.md` |
| `bee-comment-review/SKILL.md` | Step 4, build the fix as if the requirement had always existed | `bee-principle-redesign-from-first-principles` | `skill-bee-comment-review.md` |
| `bee-figure-it-out/SKILL.md` | Phase A, falsifiable done predicate | `bee-principle-prove-it-works` | `skill-bee-figure-it-out.md` |
| `bee-figure-it-out/SKILL.md` | Phase A, reversible work proceeds | `bee-principle-never-block-on-the-human` | `skill-bee-figure-it-out.md` |
| `bee-figure-it-out/SKILL.md` | Phase B, the foundation first | `bee-principle-foundational-thinking` | `skill-bee-figure-it-out.md` |
| `bee-figure-it-out/SKILL.md` | Phase C, the smallest change per unit | `bee-principle-laziness-protocol` | `skill-bee-figure-it-out.md` |
| `bee-figure-it-out/SKILL.md` | Phase C, verify each unit | `bee-principle-sequence-verifiable-units` | `skill-bee-figure-it-out.md` |

## A skill, not a principle, restored with them

`bee-architect` Phase C names `bee-interrogate` for the hardest-objection pass.
The port note for `bee-architect` held that sentence until ppb-15 landed the
skill, so it came back with the principles.

## Not restored, named here

- `bee-principle-minimize-reader-load` dropped its link to
  `bee-principle-guard-the-context-window` when it was ported
  (`principles-ppb-27.md`). That skill is outside this cell's files, so the link
  stays a plain sentence there until a cell owns the principle.
- `bee-principle-experience-first` keeps "prototype in throwaway markup" as a
  plain sentence, because no `bee-design` skill sits under `skills/`
  (`principles-ppb-29.md`).
- `deslop` has no bee counterpart and no decision brings one, so it stays
  dropped.
- The `arena` pointer in `bee-architect` stays replaced by the blind lanes,
  because `bee-arena` and `bee-architect` share one procedure.

## Leftovers closed in the same cell

These come from `reports/ppb-24c.md` and `reports/ppb-25c.md`.

- `session/worktree-cleanup.md` step 3 now names `bee-arena` (ppb-24c, step 25).
- `feature/multi-phase-plan.md`: owners never rebase under decision `b25486e3`.
  The checklist line and the merge line both send drift against trunk to the
  merger as a cell, and step 3 names the read role for gathers (ppb-25c finding
  A and ppb-9c finding 2).
- The feature port's "Rebase into small, ordered commits" and the "the app is
  small" rebuttal are now written up in `feature.md` under "Fixes in ppb-37".
  The rebuttal is back in step 5 of `feature/feature.md`; the rebase wording
  stays swapped, for the reason given there.
- The restored note in `session-ppb-12.md` (the babysit CI retry) sits after the
  last drop bullet, so the bullets above it read as drops again (ppb-25c
  finding B).
- The four table-only rows ppb-11c marked not noted (the `gh` liveness probes,
  "pushes its branch", the `units.tsv` PR column, "merge order" in
  `preferences.md`) are listed in the orchestrate drop list of
  `session-ppb-11.md`.
