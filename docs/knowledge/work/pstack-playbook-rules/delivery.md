---
type: bee.delivery
title: pstack-playbook-rules — delivery
description: "Delivery record proposed by bee knowledge promote for work item pstack-playbook-rules: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-17
bee:
  id: pstack-playbook-rules-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/pstack-playbook-rules/CONTEXT.md, docs/history/pstack-playbook-rules/plan.md]
  sources: [docs/history/pstack-playbook-rules/CONTEXT.md, docs/history/pstack-playbook-rules/plan.md, .bee/cells/ppr-1.json]
---

# pstack-playbook-rules — Delivery

## What shipped

- **ppr-1** — Failure-table rows, CI-red classification and PR shape landed in their skill homes; regen clean (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ppr-1** — ``rg -c 'Late return' skills/bee-swarming/references/swarming-reference.md` prints 1. `rg -c -i 'flake' skills/bee-herding/references/role-merge.md` prints at least 1. `rg -c 'Blast Radius' skills/bee-technical-writing/SKILL.md` prints 1 and `rg -c 'bee-technical-writing' skills/bee-hive/references/scout-and-ticks.md` prints at least 1. `git diff --stat main -- skills/bee-planning/playbooks/perf.md` is empty. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.` — docs-only cell: pointer and parity checks re-run by the leader in the worktree (1, 2, 1, 1, perf diff empty, manifest 419 match; worker's regen 3/3 green)

## Deviations

- **ppr-1** — Left the regen-bumped .bee/onboarding.json timestamp uncommitted — a timestamp-only bee state change outside the cell files — found a better route
- **ppr-1** — Cap run by the leader from main — the worktree host guard refuses a bee argument holding the word git, and the verify text holds it — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work pstack-playbook-rules` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pstack-playbook-rules/CONTEXT.md`, `docs/history/pstack-playbook-rules/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
