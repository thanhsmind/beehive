# Class Playbooks — Brief

**Feature slug:** class-playbooks
**Date:** 2026-09-02
**Lane:** small — short brief, no plan.md

## What was asked

Write the four missing class playbooks. `ROUTE_CLASS_VALUES`
(`packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:287-288`)
holds eight classes; only `perf`, `bugfix`, `refactor` and `research` have a
`###` section under `## Class playbooks` in
`skills/bee-planning/references/planning-reference.md`. `feature`, `docs`,
`release` and `spike` have none.

## What was found

Nothing pins the playbook set. `rg` over `packages/bee-rs/crates/bee/tests/`
and `src/` returns no file reading the playbook headings, and
`route_class_parity.rs` pins the class VOCABULARY across four documents but
never asks whether each class has a procedure. That is why the hole was
invisible: `expertise-principles` routed as `class=feature`, found no
playbook to cite, and had to record a named deviation instead.

## What will be done

One cell, red first.

1. Add `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs`: every
   value in `ROUTE_CLASS_VALUES` — read as TEXT from `workflows.rs`, never
   copied — has exactly one `### <class>` section under `## Class playbooks`,
   and every such section names a real class. Both directions.
2. Run it and watch it fail, naming `feature`, `docs`, `release`, `spike`.
3. Write those four playbooks in the voice and shape of the four that exist:
   a short numbered step list, each step an action, and a closing line
   naming the thing that is not a result.
4. Run it green.

## Boundaries

- No new route class, lane, CLI verb or skill.
- No existing playbook is reworded — this fills holes, it does not edit
  what is already written.
- The fence declares no list of its own; the class values are read from
  `workflows.rs` as text (the `route_class_parity.rs` technique).

## Nothing settled beyond this

The four uncovered pstack task shapes — hillclimb, runtime-forensics,
trace-forensics, visual-parity — stay out. Each needs a new class or skill
and is its own decision.
