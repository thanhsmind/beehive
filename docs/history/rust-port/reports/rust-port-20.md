# rust-port-20 — status readers B2

[DONE] — Ported the recovery/contention readers, the five remaining
`bee.mjs`-private status helpers (`buildLaneRows`/`buildLaneSummary`,
`computeRuntimeDrift`, `findRepoHive`, `ungrantedWorktreeNotice`), and the
remaining importable `state.mjs`/`source-identity.mjs` helpers into
`bee-core`, plus a lease-backed `listReservations`/
`expiredUnreleasedReservations` reader that deliberately replicates
`bee.mjs`'s reference-identity reservation-staleness bug (D1 freeze).

All 23 tests green in one target: `cargo test --manifest-path
crates/Cargo.toml -p bee-core --test status_readers_b2`.

## Rework round (goal-check NEEDS_REVISION, 9/11 checks passed)

Two test-only gaps, no production change needed:

1. **oracle-coverage-every-ported-reader** — `read_raw_config_for_validation`
   had zero call sites in any test. Added
   `read_raw_config_for_validation_matches_mjs_absent_malformed_and_present`:
   oracle-diffs the Absent / Value(null)-malformed / Value(object)-present
   shapes against the real `bee.mjs status --json`'s `staleness_warnings`
   `config validate [...]` line.
2. **reference-identity-fixture-discriminating** — the original fixture's
   two leases were distinguishable (different path/workflow_id/
   workspace_id/timestamps), so its comment's "deeply equal" claim was
   inaccurate. Fixed the comment, and added
   `reservation_reference_identity_bug_with_deeply_equal_projected_rows`:
   two leases sharing the same `resource`, no `acquired_at`, only
   `expires_at` differing (2020 vs 2099, a field the projection drops
   entirely) — proving the two `LeaseReservationView` rows are
   `serde_json::Value`-equal, and that both the port and the real CLI
   still report 2, never a deduped 1.

Both deviations from the first pass (`read_lane`'s missing `created_at`
default merge, `build_lane_rows`' `workers: []` leak) are now recorded as
structured `trace.deviations` on the cell, not just prose. Capped with
`--override-judge` (a worker cannot itself dispatch a fresh judge pass;
the coordinator relayed the verdict and instructed capping after rework)
and a `ratio_waiver` (this round's diff is test-only, so the ratio gate's
source-line denominator is legitimately zero).

## Files touched

- `crates/bee-core/src/recovery.rs` (new)
- `crates/bee-core/src/source_identity.rs` (new)
- `crates/bee-core/src/state.rs`
- `crates/bee-core/src/claims.rs`
- `crates/bee-core/src/config.rs`
- `crates/bee-core/src/fsutil.rs`
- `crates/bee-core/src/reservations.rs`
- `crates/bee-core/src/lib.rs`
- `crates/bee-core/tests/status_readers_b2.rs` (new)
- `crates/bee-core/tests/support/status_readers_b2_oracle.mjs` (new)
- `crates/bee-core/tests/guard_support.rs` (updated for the new `Session`
  fields)

Full trace/evidence: `.bee/cells/rust-port-20.json`.

## Deviations (auto-fixed, found via oracle diff)

1. `state::read_lane` was missing `laneRecordFrom`'s `created_at: null`
   default merge — fixed by injecting the default into `extra`, scoped to
   lane reads only (never `state.json`).
2. `state::build_lane_rows` was leaking a `state.json`-only `workers: []`
   field the shared `State` struct always carries — no real lane record
   ever has this key — fixed by stripping it in the lane-row projection.
