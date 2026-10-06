---
date: 2026-10-06
feature: paseo-pi-hardening
categories: [pattern, failure]
severity: standard
tags: [herding, paseo, hat-wave, judge, workers]
---

# Learning: The plan-step hat wave and the slice judge each caught a defect the plan would have shipped

**Category:** pattern
**Severity:** standard
**Tags:** [hat-wave, judge, paseo]
**Applicable-when:** planning or judging work that changes a run loop, a guard, or an event-driven timer.

## What Happened

The five-seat hat wave on `docs/history/paseo-pi-hardening/plan.md` found five blockers before any code existed: worker self-dispatch that herdr pane workers rely on (psd-12), a supervisor allowlist that depended on extension discovery, Paseo's `-`/`auto` inspect sentinels, an invalid Pi `--model` built from a non-pi provider, and an unproven extension load. Six decisions were revised (stores `a8d28361`, `c2a5af66`, `4cb9c644`, `be1699f8`, `976ef5a5`, `e2af1dab`). After the code landed, the opus slice judge returned NEEDS_REVISION on pph-4: the new broker timer bypassed the result drain's turn-start latch (`.pi/extensions/bee-guard/result-inbox.ts`); the fix shipped as `ca64d2cb3`.

## Root Cause

Both defects sat in what the plan did not say: the facts seat read Paseo's own `inspect.ts` and the risks seat read the herding pane code, and the judge read the drain the new timer shared. A plan written from the leader's reading of the touched files alone does not see a neighbouring mechanism.

## Recommendation

When a cell adds a second caller to an existing send path, timer, or latch, name that path's preconditions in the cell's must_haves, and keep the independent slice judge on high-risk behavior cells even when every verify is green.

# Learning: agy-flash herding workers commit but never cap, and a fix-up round can be ignored

**Category:** failure
**Severity:** standard
**Tags:** [herding, workers, agy]
**Applicable-when:** dispatching cells through `bee herding run` to agy-flash.

## What Happened

All seven cells ran on agy-flash through `bee herding run`. Every worker committed with the `cell:` trailer and returned outcome done, and every run ended with "worker reported success … without capping it". The leader ran each verify and capped with `bee cells finish --force-ownership`. On pph-5 a fix-up round (the action text extended with a named one-line change) returned done with no new commit; the leader escalated the cell and made the change. The pph-4 revision round, with a precise failure signature and named tests, was followed.

## Root Cause

The herded brief tells the worker not to run bee commands, so the cap stays with the leader (PBI p-6f2623c9). A fix-up appended to a long original action reads as already done to a cheap model.

## Recommendation

When re-dispatching an agy-flash cell, put the fix first in the action and name the failing check and the test that proves it; verify `git log` for a new commit before trusting the outcome.

# Learning: the desktop AppImage paseo CLI passes doctor but cannot spawn a worker

**Category:** failure
**Severity:** standard
**Tags:** [paseo, doctor, environment]
**Applicable-when:** a machine has both the Paseo desktop AppImage and the npm `@getpaseo/cli`.

## What Happened

In the live proof, `~/.local/bin/paseo` (an AppImage wrapper) was first on `PATH`. `bee doctor --runtime pi` reported `paseo_ready` ok, yet `bee herding run --agent paseo-pi-flash` failed `spawn_failed: could not parse agent id: no JSON object found in output`. Pointing `herding.paseo.command` at the npm CLI made the same run succeed: the worker's `git push` was refused by worker-guard and `job.json` recorded the observed model. Evidence: `~/.local/state/bee-verify/evidence/20261006-135136-2730693/` (006, 008, 011, 012).

## Root Cause

The doctor row checks the version string, which the AppImage also prints, not the JSON shape `paseo run` must return.

## Recommendation

When a Paseo spawn fails to parse an agent id, check which `paseo` is on `PATH` first; the doctor gap is a backlog finding.
