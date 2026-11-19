# oox-t — prove the orient overflow line and the knowledge notices

Checked by the leader after three herded runs ended timed out (one orphan kept
mutating the worktree beside a re-dispatch; it was cancelled with
`bee herding cancel`). Commit under check: ef313d976 (oox-2). oox-1 was dropped
as a NOOP: D1 already shipped in 3cceba454 and 9f519d4e9, and its test
`at_most_principles_cap_principle_rows_print_and_the_rest_share_one_overflow_line`
pins the overflow slugs.

## Runs

Every run is `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- <filter>`.

| # | Scratch edit | Filter | Result |
|---|--------------|--------|--------|
| 1 | `knowledge_notices` setup branch and upkeep branch return `Vec::new()` (notices.rs:448, :451) | `knowledge_notice` | `knowledge_notice_offers_setup_when_the_bundle_is_absent ... FAILED`, `knowledge_notice_points_at_upkeep_when_due ... FAILED`, `test result: FAILED. 1 passed; 2 failed` |
| 2 | `upkeep_line` says `"upkeep: due"` for the not-due case (report.rs:752) | `report_upkeep_line` | `report_upkeep_line_names_the_due_reasons_or_says_not_due ... FAILED`, `test result: FAILED. 0 passed; 1 failed` |
| 3 | none (restored with `git restore`) | `principles notice knowledge::` | `test result: ok. 332 passed; 0 failed` |
| 4 | the `notices::knowledge_notices(&repo_root)` call in onboard/mod.rs replaced by an empty list | `onboard notice knowledge::` | `test result: ok. 560 passed; 0 failed` — **gap** |

## Verdict

The notice function and the report line function are pinned (runs 1 and 2).
The wiring is not: run 4 shows that `bee onboard` can drop the notice with every
test green, and the same holds for the `upkeep_line` push in `run_report`
(found by the cancelled round's plan, M7). Cell oox-3 adds a test per wiring
site and oox-3t checks them.
