# Wave chain thrift — Context

**Feature slug:** wave-chain-thrift
**Date:** 2026-11-21
**Scope:** Standard
**Domain types:** RUN

## What was asked

The owner asked the advisor where bee can still cut tokens while working
better. The advisor ranked six proposals; the owner said "làm đi" to the top
three: run the per-cell chain without leader calls, carry plan-conflict
verdicts across a plan revision, and state the read-only shell rules in the
brief.

Evidence: in token-spend-cuts the opus leader was 35.7M of 42.9M tokens,
nearly all cache read; each cell cost several leader calls between the
implementer's result and the cap. Adding 2 small cells reopened 62 conflict
candidates. In the advisor's read-only job 5 of about 18 Bash calls were
refused by the guard.

## What will be done

1. `bee dispatch wave --run` records the tester's judge blocks itself, and
   bee-swarming makes `--rolling --run` the default loop.
2. A plan-conflicts re-derive keeps prior verdicts for candidates the new or
   changed cells do not touch.
3. A read-only job's brief lists the shell rules its guard enforces.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | `dispatch wave --run` (plain and `--rolling`) reads a test-role cell's report for `json <cell-id>` judge blocks and records each block whose cell is a dep, through the `judge-record --from-text` function; invalid blocks are reported on the row. Rows show capped and judged. bee-swarming names `--rolling --run` the default execution loop; the leader reviews diffs and runs leader-check once per wave. | 426a6af1 |
| D2 | The derive stores a digest per plan cell; a re-derive carries a prior verdict for an unchanged candidate id unless the new or changed cells also derive it. Output names carried and open counts; carried rows gain `carried_from`. | 78112206 |
| D3 | A read-only job's brief carries a shell-rules block built from `READ_ONLY_ALLOWED` and the refused forms; the read-only ack step needs no command substitution. | e9cd3a25 |
| C1 | Test contract wave-judge-record. | a8b6b9f4 |
| C2 | Test contract conflict-verdict-carry. | e8e0b78c |
| C3 | Test contract read-only-shell-rules. | 1bf12c30 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/bee-herding/waves-and-occupancy.md | suspect | wave and rolling runs |
| docs/knowledge/areas/workflow-state/gates.md | suspect | plan-conflicts derive and verdicts |
| docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md | suspect | read-only guard |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs` | `run_wave`, `run_rolling`, rows |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | `run_dispatch_wave`, the rolling fill |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs` | `process_judge_from_text` |
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs` | derive, `build_conflict_review` |
| `packages/bee-rs/crates/bee/src/herding/mailbox.rs` | `render_brief` |
| `packages/bee-rs/crates/bee/src/hooks/worker_guard.rs` | `READ_ONLY_ALLOWED`, read-only checks |

## Out of scope

Short default verb output, moving the result contract out of the brief, and
trimming standing context (advisor proposals 4 to 6).
