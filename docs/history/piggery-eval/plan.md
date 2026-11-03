# Plan: piggery-eval

## Summary

One xia brief checks six piggery hypotheses against bee's own code and docs,
and lands the spec drop that asked for it. The leader then files one proposed
PBI per item worth doing.

Mode: `small` — 0 risk flags. Docs only: no product code, no hook, no CLI shape.
Why this is the least workflow that protects the work: a wrong verdict costs one
PBI triage later; nothing ships to users.

## Requirements (from CONTEXT.md)

- D1: Claude Code and Paseo only.
- D2: bee gate and piggery gate stay separate concepts.
- D3: brief at `docs/history/research/piggery-xia.md`, request at `docs/discovery/spec-drops/sup-20261016-a506.md`.
- D4: one proposed PBI per worthwhile item.
- D5: piggery-borrow is checked as prior art first.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A claim has a default TTL, so a dead worker's cell is swept, not finished | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:34 | DEFAULT_CLAIM_TTL_SECONDS: f64 = 3600.0 |
| 2 | Cancel already kills the process tree with a 2 s grace | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:279 | const TREE_KILL_GRACE: Duration = Duration::from_millis(2000); |
| 3 | The concurrency limit is refused in code | read | packages/bee-rs/crates/bee/src/herding/wave.rs:1141 | "reason": "limits.concurrency", |
| 4 | The prose worker cap is a literal number | read | skills/bee-swarming/references/swarming-reference.md:33 | the worker count is a default, ~3-4 live |
| 5 | Review corroboration names different models | read | skills/bee-reviewing/SKILL.md:84 | independent only across different models or disjoint lenses |

## Approach

Three read-only gathers run first in parallel: piggery itself, bee's worker
path (swarming, herding, limits, lifecycle), and bee's mail, worktree and
review surfaces. The leader writes the verdicts from the digests and spot-checks
each anchor. One docs cell writes the two files from the leader's synthesis.
The PBIs are CLI state writes and stay with the leader.

## Cells — current slice

- `pe-1` (docs): write `docs/history/research/piggery-xia.md` and
  `docs/discovery/spec-drops/sup-20261016-a506.md` from the leader's brief.
  Verify: both files exist; every verdict row carries a repo anchor; no bee
  "gate" is described with piggery's meaning.

## Proof

- docs → parity/pointer check: `rg` each cited path exists in the worktree.
- `bee backlog pbi list` shows each new id with status `proposed`.
