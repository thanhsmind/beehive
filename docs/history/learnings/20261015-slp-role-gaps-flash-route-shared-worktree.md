---
date: 2026-10-15
feature: slp-role-gaps
categories: [failure, pattern]
severity: standard
tags: [herding, dispatch, worktree, planning, prompts]
---

# Learning: Check a role's observed model before giving it multi-site Rust cells

**Category:** failure
**Severity:** standard
**Tags:** [herding, dispatch, planning]
**Applicable-when:** planning assigns cells to a role whose slot is a herding agent

## What Happened

The plan gave rig-1, rig-3 and rig-5 to role `code`, which resolves to `paseo-pi-flash`. Every run of that agent in the session printed `runs model hive/deepseek-ai/deepseek-v4.1-flash (thinking off), configured hive/deepseek-ai/deepseek-v4.1-flash (thinking medium)`, and `bee herding status` showed `model_mismatch=true`. rig-5's run ended `timed_out_idle` after 900 s with no edit to `herding.rs`; rig-1's run ended `timed_out_ceiling` at 1800 s with its work uncommitted. Both were rerouted to role `test` (opus) by decisions `d933d1a2` and `64df01a4`, and rig-3 was rerouted before its first run (`5117fa99`).

## Root Cause

The mismatch was visible before planning, but nothing read it when roles were assigned. A cheap model with thinking off cannot finish a cell that touches two code sites plus tests inside the ceiling.

## Recommendation

When a plan assigns a write cell to a herding role, read that agent's last `bee herding status` line first. If it shows `model_mismatch=true`, assign the cell to a native role, or fix the route. A team that wants the run to stop on that mismatch sets `herding.paseo.on_model_mismatch` to `refuse` (slp-role-gaps D9, `0ccaa387`).

# Learning: Never stash in a worktree that live workers share

**Category:** failure
**Severity:** standard
**Tags:** [worktree, herding]
**Applicable-when:** several workers run in one feature worktree at the same time

## What Happened

Two herding workers ran `git stash` in the shared slp-role-gaps worktree to measure a baseline: rig-1 left entries `rig1-check` and `rig1-baseline`, and rig-2 reported "with it stashed (4231 passed, 12 failed)". While rig-1's work sat in a stash, other workers committed beside it. After one pop, an unrelated untracked draft, `docs/history/cockpit-enumerated-surface/`, appeared in the worktree. It was never committed anywhere, and its only owner is an old lane record.

## Root Cause

The stash stack is shared by every worktree of the repository, and the herding brief does not forbid stash. A baseline measured by stashing also changes the tree that other live workers read.

## Recommendation

When a worker needs a baseline in a shared worktree, run the same command on a separate checkout of the base commit, or skip the baseline. Re-dispatch briefs in this feature carried "Never use `git stash` in this worktree"; that line belongs in the herding brief itself (backlog row filed below).

# Learning: A prompt edit owes the dispatch goldens and a re-vendored binary

**Category:** pattern
**Severity:** standard
**Tags:** [prompts, tests, vendored-binary]
**Applicable-when:** a cell edits a file under `packages/bee/prompts/`

## What Happened

rig-7 added one line to `advisor.md` and two to `reviewer.md`, and proved itself with `--test instruction_laws` (9 passed). The full suite then showed 7 reds in `verbs::drivers::tests` (byte-identical payload goldens), and `bee dispatch prepare` in the worktree refused with "prompt skew" until `.bee/bin/bee` was rebuilt and copied. Pattern `20260821-a-vendored-binary-is-a-second-place-the-feature-must-land` already names the binary half; the skew refusal caught it, so that check works.

## Root Cause

The cell's verify command covered instruction text only, not the payload goldens that pin every prompt byte.

## Recommendation

When a cell edits a dispatch prompt, its verify runs `cargo test -p bee drivers::` as well as `--test instruction_laws`, and the wave barrier rebuilds and re-vendors `.bee/bin/bee`.

# Learning: Plan prose that spells a bee verb and a flag is scanned as a command

**Category:** failure
**Severity:** standard
**Tags:** [planning, tests]
**Applicable-when:** writing cells JSON or key_links inside plan.md

## What Happened

A rig-1 key_link in the frozen plan read `bee cells dissent passes --new-evidence to the sibling`. The test `no_shipped_command_spelling_is_refused_by_the_widened_guard` read it as a runnable command and failed, and the frozen plan could only be pinned in `KNOWN_HISTORICAL_EXCEPTIONS` (now 15 rows, most of them this same kind of plan prose).

## Root Cause

The documented-invocation scan reads every `bee <verb> --flag` span under `docs/`, including plan prose that only names a flag.

## Recommendation

In plan.md, name a flag without the verb in front of it ("the `--new-evidence` flag of the dissent verb"), so the scan has no command to read.
