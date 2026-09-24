---
artifact_contract: bee-research/v1
topic: pstack-playbooks-gap-check
depth: standard
date: 2026-09-24
---

## Bottom Line

- bee's class playbooks (`skills/bee-planning/references/planning-reference.md`
  § "Class playbooks") plus its skill catalog already cover most of pstack's
  poteto-mode playbooks, several of them more strongly.
- Tally over the 23 playbooks at the pinned commit: 7 covered-stronger,
  10 covered-adequately, 3 deliberate-non-match, 2 genuine-gap-deferred,
  1 genuine-gap-closed.
- The two deferred gaps (eval's blind-judge protocol, trace-forensics'
  captured-artifact depth) are backlogged by name, layers `eval-blind-judge`
  and `trace-forensics-depth`.
- The one closed gap is Hillclimb's plateau-means-pivot rule, added as one
  sentence to the `perf` class in the same wave (cell bpb-2).
- Confidence: **85%**. The pstack side is a full read of each playbook file;
  the verdicts are judgment against the anchors named per row.

## Repo Snapshot

- Source: `cursor/plugins`, the `pstack` plugin, commit
  `b9ddc83c32972210b8a94d389130713e8eed346e` (the same pin as
  `docs/history/research/pstack-xia.md`), `skills/poteto-mode/SKILL.md`
  § "Playbooks" (lines 112-140).
- The feature plan counts 22 playbooks. The pinned commit lists 23: the
  plan's count left out **Bug fix**. This artifact adds it as row 23 rather
  than drop a listed playbook.
- bee side: `skills/bee-planning/references/planning-reference.md` class
  playbooks (`perf`, `bugfix`, `refactor`, `research`, `feature`, `docs`,
  `release`, `spike`), the `bee-*` skills, and `AGENTS.md`.

## Findings

Verdicts: **covered-stronger** (bee does the same job with more enforcement
or less manual work), **covered-adequately** (bee does the job to a similar
depth), **deliberate-non-match** (no matching surface in bee, on purpose),
**genuine-gap-deferred** (real gap, backlogged), **genuine-gap-closed**
(real gap, closed in this feature).

| # | pstack playbook | Purpose | Verdict | bee-side anchor |
|---|---|---|---|---|
| 1 | Investigation | Read-only question: how X works, why Y was built, X or Y | covered-adequately | `research` class, `planning-reference.md:302-313` |
| 2 | Perf issue | Trace and improve a measured slowness against a baseline | covered-adequately | `perf` class, `planning-reference.md:244-262` |
| 3 | Hillclimb | Sustained metric improvement loop with a decision log | genuine-gap-closed | `perf` class sustained-target paragraph, `planning-reference.md:253-257`, plus the plateau-pivot sentence bpb-2 adds there |
| 4 | Runtime forensics | Diagnose a live runtime symptom from instrumentation | covered-adequately | `research` class "When the symptom is live at runtime", `planning-reference.md:315-318` |
| 5 | Trace forensics | Diagnose a pre-captured profiling artifact | genuine-gap-deferred | none exists: `research` class, `planning-reference.md:302-324`, has no artifact-parsing, sqlite or symbol-resolution depth; backlog layer `trace-forensics-depth` |
| 6 | Feature | New or changed behavior built from a named data shape | covered-stronger | `feature` class, `planning-reference.md:326-339`, plus `AGENTS.md` § "Work in parallel, coordinate through the store" dispatch mandate |
| 7 | Refactoring | Behavior-preserving structure change | covered-stronger | `refactor` class, `planning-reference.md:286-300` |
| 8 | Prototype | Throwaway sketch to settle a design fork cheaply | covered-stronger | `spike` class, `planning-reference.md:371-390` |
| 9 | Visual parity | Pixel-exact UI equivalence or styling migration | deliberate-non-match | not applicable: this repo has no design-parity surface |
| 10 | Authoring or modifying a skill | Write or edit a SKILL.md | covered-stronger | `skills/bee-writing-skills/SKILL.md:18-29`, Iron Law and RED/GREEN/REFACTOR/VALIDATE cycle |
| 11 | Eval | Test how a skill or prompt change moves agent behavior | genuine-gap-deferred | `skills/bee-evolving/SKILL.md` Gate A/B (lines 63, 104) has no blind-judge protocol; backlog layer `eval-blind-judge` |
| 12 | Babysit | Drive a PR or stack to merge-ready | covered-adequately | `skills/bee-herding/SKILL.md:58-64`, route role |
| 13 | Shipping | Verify a green stack, then land it | covered-adequately | `skills/bee-herding/SKILL.md:50-56`, merge role, plus the `bee worktree merge` proof check |
| 14 | Autonomous run | Drive a long task to completion without stopping | covered-adequately | `AGENTS.md` § "Act. Don't ask." (line 385) plus the bee-herding dispatch loop, `skills/bee-herding/SKILL.md:38` |
| 15 | Orchestrate | Standing multi-day program under one coordinator | covered-adequately | bee-herding dispatch role (`skills/bee-herding/SKILL.md:38`) plus the worker-brief contract, `skills/bee-swarming/SKILL.md:144` ("Execute (worker)") |
| 16 | Autopilot-full | Queue of PRs, one owner per PR, run to merged | deliberate-non-match | not applicable: bee has no Graphite or owner-per-PR concept; merge stays human, `skills/bee-herding/SKILL.md:84` |
| 17 | Autopilot-stack | Queue built into one Graphite stack the operator lands | deliberate-non-match | not applicable: bee has no Graphite stacked-PR concept; one worktree per feature, `AGENTS.md` § "Bee workflow" |
| 18 | Session pickup | Resume a prior agent's in-flight work | covered-stronger | `AGENTS.md:95-114`, `.bee/HANDOFF.json` plus `bee state handoff adopt` |
| 19 | Pause safely | Suspend work so it can be resumed | covered-stronger | `AGENTS.md:297` § "Care for the session", the same HANDOFF.json mechanism |
| 20 | Multi-phase or multi-PR plan | Work that spans phases or stacked PRs | covered-adequately | `planning-reference.md:211-218` § "Phase plan vs epic map" |
| 21 | Worktree and simulator cleanup | Prune merged worktrees and stale simulators | covered-stronger | `bee worktree prune` (automatic sweep, not a manual audit); simulators not applicable |
| 22 | Opening a PR | Open the PR at the end of every playbook | covered-adequately, by deliberate design difference | `skills/bee-hive/references/scout-and-ticks.md:134-149`, `ship_visibility` default `draft-pr` |
| 23 | Bug fix | Reproduce, root-cause and fix a defect with runtime evidence | covered-adequately | `bugfix` class, `planning-reference.md:264-284` |

Row 22 is a considered divergence, not a gap. pstack opens every PR ready,
never as a draft (`playbooks/opening-a-pr.md:25`). bee opens a draft PR on
the first cap and keeps it draft as a window on the work, because the
user's `uat` door, not the PR state, is the acceptance point.

## Inference

- The two genuine-gap-deferred rows each carry one `bee backlog add` entry
  (type `debt`, severity `P3`, feature `bee-playbooks`):
  - `eval-blind-judge` — bee-evolving needs a blind-judge protocol like
    pstack's eval playbook.
  - `trace-forensics-depth` — no captured-artifact research depth for trace
    forensics; no current need in this Rust-CLI repo.
- Every other row is covered or out of scope on purpose. This artifact
  records findings only; it proposes no new class, playbook, or code change.
