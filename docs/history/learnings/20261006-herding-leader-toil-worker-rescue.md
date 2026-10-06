---
date: 2026-10-06
feature: herding-leader-toil
categories: [failure, pattern]
severity: standard
tags: [herding, workers, tests, judge]
---

# Learning: cheap herded workers stall waiting on a background test run

**Category:** failure
**Severity:** standard
**Tags:** [herding, agy, tests]
**Applicable-when:** dispatching code cells to agy-flash through `bee herding run`.

## What Happened

Two agy-flash workers (hlt-2 first round, hlt-5) wrote their code, started the whole crate suite in the background, and then waited on it with no further screen change. One run ended at the idle timeout, one at the 30-minute ceiling, both uncommitted. A third (hlt-2 revision) hit the ceiling mid-fix. The leader closed the panes, escalated the cells, and finished them: hlt-5 needed only a test fix and a commit, hlt-2 needed a redesign of the worker's stdout-capture approach.

## Root Cause

The worker treats "run the full suite" as part of done and waits for a background task that never wakes it; the herding idle signal reads the unchanged screen as idle.

## Recommendation

When a herded code cell is dispatched, its action says to run only the cell's own verify in the foreground and commit when it is green; when a pane shows no edits for about 10 minutes, close it and take the cell over rather than waiting for the ceiling.

# Learning: tests that run an external release binary or mutate process state test the wrong thing

**Category:** pattern
**Severity:** standard
**Tags:** [tests, isolation]
**Applicable-when:** a Rust unit test needs a command's printed output or a different cwd or HOME.

## What Happened

Workers wrote tests that spawned `target/release/bee` (stale when another worktree built last, skipped on a clean CI), changed process-wide `HOME`, or wrote to the real main store through the real cwd. The opus slice judge caught each one. The fixes: a child run of the test binary itself (`current_exe` with `--exact <child> --ignored`), a cfg(test) thread-local for the home folder, and temp repos under the shared cwd lock.

## Recommendation

When a test needs printed output, run an ignored child test of the same test binary; never call a separately built binary and never change process-wide env.
