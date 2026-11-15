---
name: bee-fanout
description: >-
  Fan out N parallel workers across slices or on one brief, drain them, and return one report. Use when the user types /bee-fanout or asks for parallel coverage, a race or a gauntlet. Not bee-swarming, which runs approved cells to done.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The workers go out through the vendored bee binary's dispatch door. Absent, the leader covers the slices itself, one after the other, and the report says so.
---

# Fanout

Fan out N parallel workers. They may cover separate slices, race the same brief, or mix both. The parent waits, aggregates, and returns one report.

This is not `bee-swarming`. Swarming runs approved cells to done: each worker owns one claimed cell, writes code and commits. A fanout worker owns a slice of a question or an arm of a race and returns a report. Work that changes files in a worktree is a cell and goes to `bee-swarming`.

## Start

Open a task list with one entry per phase before launching anything.

1. Frame
2. Fan out
3. Aggregate
4. Report

## Phase A: Frame

1. State the done predicate and the artifact or report the fanout must return.
2. Choose the shape. Partition into slices, race N workers on identical briefs, or mix both. For a race or mixed shape, declare `first pass`, `rank all`, or `best-of` before dispatching.
3. Set N from the user or derive it from the shape. N is total workers.
4. Pick the seat each worker goes out through. A worker that only reads and reports (coverage, exploration, a gauntlet of checks) goes out as `bee dispatch prepare --runtime claude --kind gather --json`. A worker whose answer is judged or compared (a race, a competing take) goes out as `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`, then `--role lane-2` and `--role lane-3`; a fourth worker or more reuses a lane role. Which model a role runs on is a setting of the bee team table, so a model race is a team setting: name each arm's role up front, and name the model each payload returns. Use every payload exactly as it returns. Never pick a model or a subagent type by hand.
5. Give each worker its own output. A gather or advisor worker writes no file: its answer is its output, and the leader saves each answer verbatim at that worker's own path, `/tmp/fanout-<slug>/worker-<n>/`. N workers sharing one path is shared mutable state and fails the `bee-principle-separate-before-serializing-shared-state` test.

## Phase B: Fan out

Issue all N `bee dispatch prepare` calls in the same turn and run every returned payload in the background.

Every brief stands alone. Include the goal, scope, exact slice or race arm, how to verify, and what to report. Reports use `PASS`, `ISSUES`, or `BLOCKED` with evidence. In a race every arm gets the same brief bytes. The brief carries the artifact, the constraints and the open question, never your own leaning or expected findings.

If a worker drops out, proceed with N-1 and note it.

A design question that must end in a dossier and `bee blind check` is not a fanout: run the blind-lane procedure in `bee-hive/references/gates-and-delegation.md`, section "Blind lanes and convergence".

## Phase C: Aggregate

Read the terminal results. For coverage, every required slice needs a result. For a race, apply the selection rule declared up front. Use first pass, rank all, or best-of. A judge for `rank all` or `best-of` goes out as `bee dispatch prepare --runtime claude --kind reviewer --json` with no `--role`, sees the arms by path label with no role or model names, and is read-only. Do not paste raw worker dumps.

Keep a compact result table, one-line evidenced issues, and explicit gaps or dropouts. A worker's `PASS` stands once you have seen the evidence it points at, per `bee-principle-verify-before-reporting`.

When every worker came back on the same model, the report says so: workers on one model share its blind spots, and their agreement is not independent corroboration.

## Phase D: Report

Return one consolidated in-chat report with the table, issue one-liners, gaps or dropouts, and the race rule when used.

## Handoff

Report delivered. Invoke bee-hive skill.
