---
name: bee-figure-it-out
description: >-
  Design an auditable playbook when no situation fits: a large migration, an ambitious multi-part change, or work a human reviews after stepping away. Scales rigor, keeps a decision trail. Use when the user types /bee-figure-it-out or says figure it out. Not for a task a situation fits.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The judge and the fan-out workers go out through the vendored bee binary's dispatch door. Absent, the leader runs them inline and the account says so.
---

# Figure it out

When the task matches no situation, design one. The deliverable before any code is the workflow itself: a sequence of phases that scales rigor to the task, runs the scientific method, and leaves a decision trail a human can audit after stepping away. Bias toward more rigor. The cost of building the wrong thing dwarfs the cost of being careful.

Don't reinvent a situation you already have. A focused single-unit task that matches a situation in its class index routes there: `bee orient --class <class>` lists the situations. But a large or cross-cutting version of one (a migration across many call sites, an ambitious multi-part change), or work the user reviews after stepping away, belongs here even though a single-unit version would be a feature. The rigor and the audit trail are the point.

## Where each phase lands

- The definition of done, as a falsifiable predicate, goes into `CONTEXT.md` through `bee-shaping`.
- The designed phase list goes into `plan.md` through `bee-planning`, and `bee plan check` holds it.
- The decision trail goes through `bee-show-me-your-work`.

## Start

Open a todolist whose first item is to read the principles `bee orient --class <class>` lists for the task. Then add the phases below as todos.

## Phase A: Frame

Ground first, then commit. Don't start the run until you can state:

- The definition of done as a falsifiable predicate: prove it works, on the real artifact, and "done well" has to be checkable.
- Scope, quantified: rough units and effort, plus the blockers grounding surfaced. Raise them before spending hours, not after fifty doomed commits.
- The rigor level, biased high. One-way doors and high blast radius get more (`bee-blast-radius` sizes the radius); reversible low-stakes steps get less. Rigor is gates and artifacts, not "try harder".

Present the framing and tradeoffs before committing to a long run. Reversible work proceeds without waiting on the human, but a multi-hour run earns one checkpoint.

## Phase B: Design the workflow

Decompose into atomic, independently-landable units. Sequence riskiest-unknown-first so option value stays high. Scaffold and verification come before features: build the foundation the rest stands on first.

- Build the verification harness before the work, with the baseline captured from the pre-change state, so the check reads as "old value vs new value".
- For one-way-door design decisions, run `bee-architect` (it runs `bee-arena`) with diverse, isolated, opinionated candidates. Each runner goes out as `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`, then `--role lane-2` and `--role lane-3`, and `bee blind check` checks each blind brief. The read-only judge goes out as `bee dispatch prepare --runtime claude --kind reviewer --json`. Use each payload exactly as it returns; which model runs a role is a setting of the bee team table, never a choice made here. Skip it for mechanical work whose shape is already concrete. A second arena over a settled design is over-engineering: do the least that settles it.
- Decide what fans out. Parallelize only across genuine seams with `bee-fanout`, and give each worker its own worktree or branch (`bee-principle-separate-before-serializing-shared-state`). Don't over-fan.
- Write the designed phase list down in `plan.md`. That list is what the human reviews.

Then put the design into motion. Add its steps to the todolist as concrete items, after the Phase C entry and before Phase D. Run each under the Phase C loop discipline, and weave the Phase D log through them, a row as each step lands, rather than saving the whole trail for the end.

## Phase C: Run the loop

Each unit is an experiment: state the hypothesis, make the smallest change, measure against the predicate on the real artifact, keep it if it advanced, revert it if it didn't.
Verify each unit before starting the next instead of batching checks at the end.

- Verify by inspecting the artifact, never a self-report. When something passes too easily, suspect the observation method before the system. A blank screenshot passes a lazy gate.
- Pair delegated work with a judge, dispatched as `bee dispatch prepare --runtime claude --kind reviewer --json`, and audit the delegates' artifacts yourself before trusting them. If a worker games the gate, reset and harden the contract. If the gate itself is wrong, fix the gate in its own change rather than routing around it.
- A verdict is VERIFIED, NOT VERIFIED, or INCONCLUSIVE. Inconclusive is not a pass. Don't hide a negative.

## Phase D: Keep the audit trail

Log the run via `bee-show-me-your-work`, one canonical TSV with a row per decision and per unit, evidence as links. A rule that settles also goes through `bee decisions log` the moment it settles; the trail row points at it. figure-it-out's work is usually ambitious enough to commit the trail so the reviewer can read it in the PR; commit it when confidence has to be shown. Prefer evidence produced by committed scripts so a reviewer can re-run it. The trail plus the diff is what lets the human come back and trust the work.

## Phase E: Verify and hand back

Check the whole against the Phase A predicate on the real product, not just the harness. Encode any recurring correction as a gate, a lint rule, a check, or a script, so the win can't silently regress (`bee-principle-encode-lessons-in-structure`).

## Reply

The playbook you designed, the rigor level and why, the decision-trail path, what's verified against the predicate, and what's still open.

## Handoff

Playbook run and reply delivered. Invoke bee-hive skill.
