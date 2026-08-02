# Test-simple — one declared test path, one result record

Status: owner-approved direction (2026-07-31). Supersedes the
proof-economy tier system wholesale.

## The model (three pieces, like fluent's)

1. **Declaration.** `.bee/config.json` `commands.test` is the single place
   a project declares how it is tested (string or array of commands) — the
   ONE command every door runs. `commands.verify` is retired. Nothing else
   declares test obligations — no per-cell proof tiers, no per-slice
   test-cell mandate.
2. **Deterministic runner.** `bee test` (porcelain) runs the declared
   commands in order, captures per-command output, and writes ONE
   normalized record: `.bee/logs/test-results.json` —
   `{ran_at, green, commands: [{command, exit, duration_ms,
   failure_excerpt}]}` where `failure_excerpt` is the last ≤500 chars of
   a failing command's output. The runner is a program; an agent's word
   is never the record.
3. **The record is the evidence.** `bee cells finish` runs `bee test`
   when `commands.test` is declared: green → cap records
   `{tests: green, results: <pointer>}`; red → cap refused, the refusal
   carries the failure_excerpt, the red becomes the work. A cell in a
   repo with no declared `commands.test` caps with `tests: undeclared`.
   Re-dispatch prompts (Prior rounds) cite the failure_excerpt directly.

## What is deleted (machinery and prose)

- The proof-tier matrix: `requiredProofTier`, change_class × lane tiers,
  red-first evidence flags (`--behavior-change`, `--evidence-stdin/file`,
  `red_failure_evidence`), evidence-tier trace fields.
- `--feature-verify-pending` and the whole deferred-proof path:
  `featureVerifyDebt`, `testCellDebt` (both kinds), the feature
  close-door ladder tied to them, `trace.proof: "unrecorded"` arming.
- Test-volume brakes: ratio ceiling, `new_suite_reason`, the
  refactor-plus-new-test-file refusal.
- The trailing-test-cell-per-slice planning mandate (coverage judgment
  survives as craft in `.bee/expertise/tests.md`; TDD-first survives as
  worker prose — discipline, not machinery).
- Verified-transcripts-as-proof special path.
- Classic per-cell `cells verify` as a taught path (the verb may remain
  plumbing; prose stops teaching it).

## What stays

- `bee close`: still the close driver — now its doors are: `bee test`
  green (full declared run) + capture reminder. Scribing/capture doors
  are capture-side, unchanged.
- Merge gate: `bee worktree merge` re-runs `commands.test` against the
  staged merge — the last net.
- "Never build on red": a red result is the next work item, never a base.
- Communication rule: "done/green/fixed" only beside fresh output — now
  always satisfiable by quoting the record.
- Coverage judgment, triad shape, red-before-green: prose craft in
  `.bee/expertise/tests.md`, applied by judgment, enforced by review —
  not by cap doors.

## Cost note

This trades the deferred-proof economy for per-finish test runs, exactly
fluent's trade. A host keeps it fast by pointing `commands.test` at a
suite it is willing to run on every cap — there is no second, slower
command to hide the full chain behind. bee's own repo declares its
impacted-cap runner.
