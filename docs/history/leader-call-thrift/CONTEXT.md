# Leader call thrift — Context

**Feature slug:** leader-call-thrift
**Date:** 2026-11-21
**Scope:** Standard
**Domain types:** RUN

## What was asked

After token-spend-cuts, the owner asked which other parts of the flow repeat
cached context and cost tokens without improving the work. The leader's
answer named seven; the owner said "làm đi" to the recommended three
(items 1, 2 and 6).

Evidence from the token-spend-cuts run (`bee report usage --corr
sup-20261121-7c4e`): 42.9M tokens, 35.7M on the opus leader, almost all cache
read. About 20 leader calls were a refusal and a retry. `cells finish` refused
one missing answer per call. 6 of 8 herded workers returned done without a
cap. The tester ran the full suite after its verify and went idle twice.

## What will be done

1. A cap names every missing answer in one refusal.
2. `herding run` caps a done cell job with the worker's own principle and
   sync answers, and when it still cannot, prints the exact finish command.
3. A herded cell worker cannot start the full suite unless its cell says why.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | A cap gathers the missing-answer refusals (mistakes answer, principles answer, principles-arm answer, proof floor, finish commit trailer, sync door) into one refusal that names each fix. State refusals still refuse first and alone. The judge override decision is written only after every check passed. | 4597b92e |
| D2 | `herding run` auto-settle passes the worker result's `principles_used` and `sync_ack` to the cap. When it still refuses, `herding run` prints the cap refusal and a finish command carrying every flag the refusal names. The cell brief asks the worker for those two result fields. | 2b81baba |
| D3 | In a herded job that carries a cell id, the worker guard refuses a Bash command with a full-suite segment (a `commands.test` entry after env stripping, `bee test`, `.bee/bin/bee test`) unless the cell has a non-blank `full_suite_reason`. The refusal names the cell verify. Jobs with no cell id are unchanged. | 9b3a677f |
| C1 | Test contract cap-one-refusal. | fab7d838 |
| C2 | Test contract auto-settle-answers. | 42961013 |
| C3 | Test contract worker-full-suite-guard. | 6e253441 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | suspect | the cap doors |
| docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md | suspect | auto-settle and the uncapped hint |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | `cap_cell_from_flags`, the cap check order |
| `packages/bee-rs/crates/bee/src/herding/run.rs` | `try_cap_finished_cell`, `postflight_cap_check`, the uncapped hint |
| `packages/bee-rs/crates/bee/src/hooks/worker_guard.rs` | the herded worker guard |
| `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs` | `full_suite_verify_problem`, the full-suite segment test |

## Out of scope

The other four ideas from the same answer: moving the mechanical loop off the
opus leader, trimming the leader's standing context, narrowing plan-revision
conflict candidates, and shorter default output.
