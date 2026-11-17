# gis-3t: independent check of gis-3 (issues 161, 167)

Cell under test: gis-3, commit 5933d6721. Parent commit: 0607c94d2. Decision: 3ee2db95 (CONTEXT D4).

## Truths

| Truth | Result | Fix site | Tests |
| --- | --- | --- | --- |
| After cells cap the cell holds no path lease and another cell can claim the same path | PASS | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:920` | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:2489` (gis-3), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:2520` (gis-3t edge) |
| reservations list labels a live lease active and a past-expiry lease expired | PASS | `packages/bee-rs/crates/bee/src/verbs/reservations/emit.rs:188`, `packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs:274` | `packages/bee-rs/crates/bee/src/verbs/reservations/tests.rs:990` (gis-3), `packages/bee-rs/crates/bee/src/verbs/reservations/tests.rs:1006` (gis-3t edge) |

## Red on the parent commit

Run in a detached worktree at 0607c94d2, with this commit's test files copied in.

- Cap truth. Both cap tests compile and fail at the lease assertion. The capped cell's leases are still held:
  `left: [("src/gtc_helper.rs", "wk-helper", "gtc-a"), ("src/gtc_kept.rs", "wk-1", "gtc-unrelated"), ("src/gtc_one.rs", "wk-1", "gtc-a")]`,
  `right: [("src/gtc_kept.rs", "wk-1", "gtc-unrelated")]`. Result: `0 passed; 2 failed`.
- Label truth. The label tests do not compile on the parent (`error[E0425]: cannot find function reservation_line`), because the parent has no per-lease label to call. This is a compile failure, not an assertion failure. The real binary shows the behavior difference instead.

## Symptom reproduced with the real binary

Scratch repo: one lease reserved with `--ttl 1` and one with `--ttl 3600`, then `bee reservations list` after 2 seconds.

Parent binary:

```
a | cell c | src/long.rs | reserved ... | active/expired by TTL
a | cell c | src/short.rs | reserved ... | active/expired by TTL
```

Fixed binary (worktree HEAD):

```
a | cell c | src/long.rs | reserved ... | active
a | cell c | src/short.rs | reserved ... | expired
```

## Edge cases added by gis-3t

- `a_plain_cap_frees_every_agents_lease_on_the_cell_and_keeps_the_same_agents_other_cell_lease`: the cap frees a lease that a second agent took for the capped cell. A lease the same worker holds for an unrelated cell is kept. A later cell claims both freed paths.
- `the_list_line_flips_to_expired_one_second_past_expiry_and_active_only_shows_no_expired_label`: a lease that expired one second ago reads `expired` and one a minute ahead reads `active`. `--active-only` returns only the live lease, labelled `active`.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::reservations verbs::cells`

Re-run on the worktree at ffd637047: `test result: ok. 568 passed; 0 failed; 10 ignored; 0 measured; 4833 filtered out; finished in 5.16s`. The filtered-out count grew from 4828 because other cells added tests outside these modules.

The four lease tests ran by name and passed:

- `verbs::cells::tests::a_plain_cap_releases_only_its_own_cells_leases_and_frees_the_path` (gis-3)
- `verbs::cells::tests::a_plain_cap_frees_every_agents_lease_on_the_cell_and_keeps_the_same_agents_other_cell_lease` (gis-3t)
- `verbs::reservations::tests::the_list_line_labels_a_live_lease_active_and_a_past_expiry_lease_expired` (gis-3)
- `verbs::reservations::tests::the_list_line_flips_to_expired_one_second_past_expiry_and_active_only_shows_no_expired_label` (gis-3t)

## Notes

- `lease_to_reservation` computes `expired` from the wall clock (`now_ms()`, leases.rs:274), not from the `now` passed to `list_reservations`. Labels match the list filter at real time. A caller that passes a synthetic `now` gets a label that ignores it.
- A lease file whose `expires_at` is an unparseable string makes `list_reservations` return an error rather than a row. That predates gis-3 and is not tested here.
- D4 also names holds. The cell's truths cover path leases only, so the cross-worktree hold mirror after a cap was not checked.
