# rrt-10 — check of rrt-9, the rrt-2 rework and rrt-8

Verify at HEAD, `CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt`, exit 0:
- `cargo test --release --bin bee -- handoff_after_merge_door catalog audit`: 41 passed, 0 failed.
- `cargo test --release --test proof_gate`: 8 passed, 0 failed.

## rrt-9 coverage

- Flag count pin: `packages/bee-rs/crates/bee/src/catalog.rs:289` `catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision`, `PINNED_FLAG_COUNT = 259` at `catalog.rs:823`. Passes at HEAD.
- Proof-line anchor: `packages/bee-rs/crates/bee/tests/proof_gate.rs:412` `every_site_teaching_the_proof_line_shows_a_qualified_result`, site `("packages/bee/prompts/worker-cell.md", "cargo test -p bee")` at `proof_gate.rs:354`. Passes at HEAD. The anchor line is `packages/bee/prompts/worker-cell.md:100`.
- Judgment: both are held by existing tests. No new test was written for rrt-9.

## rrt-2 rework (a85c938ac) tests

Two tests added in `packages/bee-rs/crates/bee/src/verbs/feature_group.rs`, inside its existing test module. `audit.rs` and `verbs/audit.rs` have no test module, so the audit-kind test sits beside the start tests and calls `crate::verbs::audit::try_native`.

| Truth | Test | Red at the parent of a85c938ac |
|---|---|---|
| `handoff-waived` is an accepted audit kind | `handoff_after_merge_door_audit_log_accepts_the_handoff_waived_kind` (new) | yes |
| a refused start writes no row | `handoff_after_merge_door_refuses_a_second_feature_and_writes_nothing` (existing) | no, it passes at the parent because the refusal comes before any write |
| a failed start writes no row | `handoff_after_merge_door_a_failed_start_with_same_session_writes_no_audit_row` (new; the worktree path is occupied, start fails with WORKTREE_TARGET_EXISTS) | yes |
| row exists after a successful `--same-session` start | `handoff_after_merge_door_same_session_with_a_reason_succeeds_and_is_audited` (existing) | no, it passes at the parent, the row was already written there |

Red method: `git show a85c938ac -- audit.rs feature_group.rs | git apply -R`, run `-- handoff_after_merge_door`, restore the files from copies. Result at the parent: 4 passed, 2 failed.

- `audit log: unknown kind "handoff-waived" — must be one of: gate-bypass, gate, close, merge, release, scribing, promote, handoff, judge-deferral, uat-deferral, capture-deferral, knowledge-freshness-deferral, impact-deferral, routing-deferral, doc-deferral, leader-check-deferral, dissent-deferral, knowledge-upkeep, verify-upkeep, fix-at-upkeep.` then `assertion left == right failed, left: Some(ExitCode(1)), right: Some(ExitCode(0))` at `feature_group.rs:757`.
- `[WORKTREE_TARGET_EXISTS] .../testrepo--wt--second-feat already exists.` then the panic at `feature_group.rs:777` printing `[Object {"kind": String("handoff-waived"), "feature": String("second-feat"), "source": String("feature start"), ...}]`, a row written for a start that failed.

At HEAD all six `handoff_after_merge_door` tests pass. Files restored; the diff is the two added tests only.

## rrt-8 must_haves

1. Consolidate ran and merged with no near-duplicate concept. The merged content is `docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:175` (thin switches) and `:176` (handoff door), each extending the existing concept. No second concept holds the handoff door or thin switches (`rg` over `docs/knowledge` finds `reread-thrift` in five other concepts, none repeats these lines). That the `consolidate` command itself ran cannot be seen in files, so that part rests on the cell trace. Not rechecked.
2. `delivery.md` per D-number: `docs/history/reread-thrift/delivery.md:5` D1, `:6` D2, `:7` D3, `:8` D4, `:9` D5, `:10` D6, `:11` D7. Measured numbers `:15` to `:33`. Out of scope `:35`. All present.

Numbers:
- 20739, 14803, 8887: `delivery.md:21` to `:23`, equal to `CONTEXT.md:62`, `:63`, `:66`. The 8868 row in CONTEXT is the two-flag-only row, not used by the delivery table.
- 21685 to 14915: `delivery.md:29`. 21685 is in `CONTEXT.md:42`. 14915 equals `wc -c packages/bee/LEADER.md` = 14915 now, and `reports/rrt-7.md:67`.
- 15227 to 11413: `delivery.md:30`. CONTEXT.md holds only the 25 percent rule (`:43`), not these two numbers. They come from the rrt-9 cell result (5768 + 5645 = 11413) and `reports/rrt-7.md:120`. 1 - 11413/15227 = 0.25048, which matches 25.05 percent.
- 258 to 259: `delivery.md:31`, equals `catalog.rs:823`.

Gap: CONTEXT.md does not state 15227 or 11413, so those two are checked against the cell result and rrt-7, not CONTEXT.

## Verdict

```json rrt-9
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "rrt9-flag-pin-held", "status": "PASS", "evidence": "catalog.rs:289 distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision, PINNED_FLAG_COUNT 259 at catalog.rs:823, passes at HEAD"},
    {"id": "rrt9-proof-anchor-held", "status": "PASS", "evidence": "tests/proof_gate.rs:412 every_site_teaching_the_proof_line_shows_a_qualified_result, anchor at proof_gate.rs:354, passes at HEAD; worker-cell.md:100 holds cargo test -p bee"},
    {"id": "rrt2-audit-kind-accepted", "status": "PASS", "evidence": "new handoff_after_merge_door_audit_log_accepts_the_handoff_waived_kind red at the parent with unknown kind handoff-waived, green at HEAD"},
    {"id": "rrt2-failed-start-no-row", "status": "PASS", "evidence": "new handoff_after_merge_door_a_failed_start_with_same_session_writes_no_audit_row red at the parent with a handoff-waived row after WORKTREE_TARGET_EXISTS, green at HEAD"},
    {"id": "rrt2-refused-start-no-row-and-success-row", "status": "PASS", "evidence": "existing refuses_a_second_feature_and_writes_nothing and same_session_with_a_reason_succeeds_and_is_audited pass at the parent and at HEAD; they already covered these two truths"},
    {"id": "rrt8-must-haves", "status": "PASS", "evidence": "delivery.md:5-11 D1 to D7, :15-33 numbers, :35 out of scope; concept lines token-economy-of-injected-context.md:175-176; the consolidate command run itself is not visible in files"},
    {"id": "rrt8-numbers-match", "status": "PASS", "evidence": "20739, 14803, 8887, 21685 match CONTEXT.md:62,63,66,42; 14915 equals wc -c; 15227 to 11413 match the rrt-9 result and rrt-7.md:120, not CONTEXT.md"}
  ],
  "fixability": "automatic",
  "confidence": "medium"
}
```
