# Port notes — principles prove-it-works, sequence-verifiable-units, make-operations-idempotent, cell ppb-28

Source: pstack principles `principle-prove-it-works`,
`principle-sequence-verifiable-units` and `principle-make-operations-idempotent`
(one `SKILL.md` each, no reference files), MIT, copyright 2026 Lauren Tan. The
notice is `LICENSE.pstack` in each new skill directory, a verbatim copy of the
pstack `LICENSE`. This file is the only place a pstack-only tool is named; no
file under `skills/` or `expertise/` names one.

Names: `bee-principle-<slug>`, the pstack slug with the prefix that both
distribution pipes discover principle skills by.

Shape: the exact shape of `bee-principle-subtract-before-you-add`: a
`description` opening "Apply", a body, then Check, Not when, Why and Depth.
The source's `disable-model-invocation: true` is dropped, as in every bee
principle skill.

## Tool swaps

- sequence-verifiable-units, delivery half: the source stacks commits and PRs
  with a stacked-PR tool, and its "rebase onto clean trunk" step assumes that
  tool's flow. The swap: the units are ordered commits and the cells that
  produce them, and the clean-base step reads "start from a clean base". The
  stacked-PR tool itself has no bee counterpart and is dropped; bee lands one
  commit per cell and merges with `bee worktree merge`, so the ordering
  argument (failing test first, fix on top) survives without it.
- prove-it-works, delegation: the source says to inspect the delegate's
  artifact, not its summary. Kept, and the bee spelling added where a check is
  itself handed to someone: a reviewer goes out as
  `bee dispatch prepare --runtime claude --kind reviewer` and is handed the
  artifact. No model or subagent type is named.
- prove-it-works, "commit the script for large work, the show-me-your-work
  skill": kept; `bee-show-me-your-work` exists in `skills/`.
- make-operations-idempotent: the source names no pstack tool. Its examples
  (adopting live sessions, a PID check on a lock) are kept as generic
  examples; nothing was swapped.

No step was dropped other than the stacked-PR mechanics above.

## Where each source part went

prove-it-works, depth in `expertise/tests.md` § Prove it works (verification
is the testing guide's craft, next to Red before green):

- The opening, the Why and the "how do I prove this" question: kept in the
  skill and the section.
- The three check-the-real-thing bullets (liveness, actual value, suspect the
  observation method) and the four code steps (build, run, whole chain,
  integration end to end): kept, in the source's order.
- Delegation, trust artifacts not self-reports: kept.
- Script the check, with the keep-it-visible and commit-only-for-large-work
  rule: kept.

sequence-verifiable-units, depth in `expertise/planning.md` § Sequence work
into verifiable units (ordering units is planning craft, beside Build order
versus discovery order):

- The two altitudes (execution, delivery), the Why, the before and after
  bracket, "never batch and verify once", the clean-base step, the
  lever-makes-the-check-cheap remark: kept.
- The delivery orders (failing test first, subtraction before reshape,
  baseline before treatment, scaffold before feature) and the three-line
  pattern: kept.
- The closing pointers to prove-it-works and build-the-lever: kept; both
  exist in `skills/` (`bee-principle-build-the-lever` and the new
  prove-it-works).

make-operations-idempotent, depth in `expertise/operations.md` § Make
operations idempotent (a command, lifecycle step or loop is operations craft):

- The opening question pair, the Why, the four pattern bullets (convergent
  startup, content-based cleanup, self-healing locks, idempotent scheduling)
  and the three-question test with the reconciliation-step conclusion: all
  kept.

## Added, not in the source

- The Not when line on each skill: the source has no limit. prove-it-works is
  limited by a real artifact that cannot be reached (name the closest check as
  a proxy and write no "done"); sequence-verifiable-units by a single small
  change that already is the unit, with a step that cannot be checked alone
  folded into the one before (split-on-a-real-dependency);
  make-operations-idempotent by a pure read or an already atomic single
  effect, where a reconciliation step guards nothing
  (subtract-before-you-add).
- A situation row in the routing table at the top of each guide's file.
- In each section, cross-references to existing sections and skills: Red
  before green and Test the real code, never the twin (tests); Build the
  lever, Subtract before you add, Split only on a real dependency (planning);
  Scheduled work is a system, `data.md` Backfills are jobs, and `apis.md`
  Every unsafe operation needs a retry story (operations).
- One sentence in prove-it-works tying the principle to the cap's proof line.

## Index rows

Three rows added to `## Principle homes` after the ppb-27 rows; no existing
row changed. Classes come from the pstack "when it applies" line:

- prove-it-works: `feature, bugfix, refactor, perf, release, docs` — "after
  completing a task, before declaring done" fires wherever work lands, so
  every class that lands work except research and spike (their deliverable
  is a written answer) and content.
- sequence-verifiable-units: `refactor, feature, bugfix` — "multi-step work
  (sweeps, migrations, runs of similar edits)" and the failing-test-then-fix
  commit order.
- make-operations-idempotent: `feature, refactor, release` — "commands,
  lifecycle steps, or processing loops" that run amid crashes and retries;
  release because deploy and migration steps are re-run.

## Pointers kept as plain sentences

None. Every skill or principle the new text names exists in `skills/`.

## Regen note

`bee dev regen` also re-rendered unrelated `.claude/skills/` entries in this
checkout, as in ppb-27; those stay uncommitted. Only the three new
directories in each tree, the render manifests, the vendored expertise
copies, the onboarding hashes and the release manifest are committed.
