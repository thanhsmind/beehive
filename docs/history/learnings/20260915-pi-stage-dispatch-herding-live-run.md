---
date: 2026-09-15
feature: pi-stage-dispatch
categories: [failure, pattern, decision]
severity: standard
tags: [pi, herding, dispatch, hat-wave, live-run, judge]
---

# Learning: A detached run launched from a tool with a timeout dies with the tool's process group

**Category:** failure
**Severity:** standard
**Tags:** [pi, herding, process-group]
**Applicable-when:** a bee verb must finish work (close a pane, write a record) after the command that launched it returns, and the launcher runs inside an agent tool that can time out.

## What Happened

The first live Pi hat wave launched three `herding run` jobs with `&` and a `wait` under a 30-second bash timeout. Pi runs each bash command in its own process group and on a timeout sends SIGKILL to the whole group, so the three runners died; the workers still wrote `result-1.json`, but panes `w1:p9V`, `w1:p9T`, `w1:p9W` stayed open and job `…91088-1` kept `pane_id: null`. The pane close lived only inside the runner process.

## Root Cause

Cleanup that belongs to a long-lived job was placed in the short-lived launcher's process. `nohup` cannot help against SIGKILL to the group. The second live run (psd-11: the launcher re-launches the runner with `process_group(0)` and returns `outcome detached`) closed every pane.

## Recommendation

When a verb promises work after its launcher returns, run that work in a process outside the launcher's process group, and prove it with a live run whose launcher is killed — a unit test cannot kill a real tool's group.

# Learning: The largest-pane split rule assumes the caller is the human's pane

**Category:** failure
**Severity:** standard
**Tags:** [herding, pane-layout]
**Applicable-when:** a rule chooses a target by excluding "the caller" and the caller can itself be a worker.

## What Happened

`resolve_split_parent` excluded only the caller's own pane and picked the largest remaining pane. A Pi leader that was itself a herding worker ran a hat wave, so the human's main pane `w1:p3A` won on area and every hat split it. psd-12 made a worker caller (`BEE_HERDING_WORKER=1`) split its own pane downward; the second live run kept `w1:p3A` at full height and all hats in the worker column.

## Root Cause

The rule encoded "the caller is the main pane" as an unstated premise. A nested caller broke it with no error.

## Recommendation

When a placement rule protects a pane by excluding the caller, check whether the caller can be a worker, and read the worker marker once at option parse so tests inside a worker pane stay hermetic. Capture the pane layout before, during and after a live run as the proof. A worker caller's downward splits still need a minimum height (backlog).

# Learning: A cheap herding model stalls on large Rust files, and escalation does not reroute a v2-planned cell

**Category:** failure
**Severity:** standard
**Tags:** [dispatch, role-routing, agy-flash]
**Applicable-when:** an approved bee-plan/v2 routes code, test or docs cells on large Rust modules to the agy-flash herding role.

## What Happened

Five cells on agy-flash (psd-1, psd-2, psd-4, psd-5, psd-7) stalled with zero edits or hit the 1800 s ceiling. `bee cells escalate` set the flag, but `dispatch prepare` still returned agy-flash, because prepare.rs reads the escalation flag only when no role is set and a v2 plan always supplies the planned role. Every stalled cell finished only after a recorded `role-reroute` decision and `bee cells reroute --role plan`.

## Root Cause

The escalation door predates role-planned dispatch, and the two paths were never reconciled (backlog). The planned role for large-module cells was chosen by job name, not by the file size the job touches.

## Recommendation

When planning cells that edit large Rust modules, assign the native plan role in the approved plan rather than the cheap herding role. When a planned cell stalls, reroute it with a logged role-reroute decision; do not rely on `bee cells escalate` under a v2 plan.

# Learning: Two gate-adjacent checks read artifacts the author did not think of as input

**Category:** pattern
**Severity:** standard
**Tags:** [plan, tests, judge]
**Applicable-when:** writing plan.md cell JSON, or a judge-verdict prompt.

## What Happened

The full suite went red once because the documented-invocations scanner reads fenced code in `docs/history/**`, and the psd-1 cell action in plan.md quoted a `bee state advisor-ref` verb span; psd-10 pinned it in `KNOWN_HISTORICAL_EXCEPTIONS`. Separately, the first semantic judge run returned verdicts that `bee cells judge-record` refused, because the judge prompt listed allowed values (`PASS|FAIL`, `mechanical|design|none`) that did not match `verbs/cells/judge.rs` (`PASS|NEEDS_REVISION`, `automatic|authority`).

## Root Cause

Both artifacts had a hidden consumer with a strict grammar: plan cell JSON is fenced text a test scans, and a judge prompt's enum list is a contract with the validator.

## Recommendation

When a cell action names a bee verb, describe it in words instead of quoting a runnable spelling with flags. When writing a judge prompt, copy the allowed values from the validator source, and chain the audit decision after a gate call with `&&` so a refused gate writes no approval record.
