# prune-queue-copy — plan check (hat wave synthesis)

Seats: all five (high-risk). facts-gaps, alternatives, user-impact, value on opus; risks resolved to fable by the model guard. No seat dropped.

PLAN CHECK
Work: one cell, pqc-1

STRUCTURE
BLOCKERS:
- facts-gaps: the plan said the queue reaches a worktree by a bootstrap copy; the code copies only onboarding and config, and the queue is git-tracked in this repo → fixed: Discovery and CONTEXT boundary corrected, claims 6 and 7 added.
WARNINGS:
- Line splitting (blank lines, CRLF) unstated → fixed in the cell action and tests.
- Id equality undefined → fixed: exact strings, non-string id keeps.
- Claims rows 1 and 2 quoted the signature, not the keep line → fixed (rows now anchor prune.rs:136 and :413).
- No proof of the "you see" column → fixed: a live dry-run after merge, shown before any real run.
- Concept row 4 and the doc comment say gitignored → the cell updates both.

CELLS (reviewed: 1)
CRITICAL FLAGS: none after the fixes.
CLEAN CELLS: pqc-1

ACCEPTED
- alternatives: a strict local reader (existing readers are fail-open); main's side is a plain id set.
- user-impact: the keep message names `bee capture flush --id <id>` inside the worktree; "merge" dropped (no command carries a gitignored queue).
- risks: tests for "main names the id only in a flush line" and "host-shaped repo with no main queue keeps"; exact string compare.
- value: worth it (32 worktrees, about 3.6 GiB, prune does nothing today).

SUMMARY: One blocker on a wrong mechanism claim, fixed in the plan text; the cell now names the strict reader, line handling, id equality and the missing tests.

Dismissed:
- user-impact (b) a text summary line for prune — new output scope beyond D1-D4; filed as backlog.
- value: stop the bootstrap copy — there is no bootstrap copy (claim 6).
