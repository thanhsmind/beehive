# rust-port-22 — read-accounting seam + today's baseline

[DONE] — Added `bee_core::read_accounting`: always-compiled, thread-local
relaxed-atomic counters placed at the LOWEST shared read primitives
(`decisions.jsonl` parse in `decisions.rs`'s two own call sites,
`.bee/cells/*.json` directory scan in `cells::list_cells`, per-dep
`cells::read_cell`, and both a function-invocation and an
`fs::metadata`-operation counter in `recovery::scan_transcript_roots`) —
never at today's higher-level reader-function entries (`ready_cells`,
`tier_mix`, `ceiling_scarcity_warning`, `scribing_debt`,
`global_scribing_debt`), so rust-port-23's dedup can't fool the instrument
by moving the read without moving the count. No dedup performed in this
cell — instrument only.

New `crates/queen-bee/tests/read_accounting.rs` (12 tests, this cell's
`verify` target) builds a derived, site-labelled per-fixture baseline
table as the test's own output:

- reconciles decision e119fc8b's stated "4/6/2" exactly against the real
  `queen-bench --generate` D5 fixture, and separately shows the
  filesystem-operation-unit transcript counter correctly diverging to 4
  (2 invocations x 2 roots) — explained, not silently asserted;
- four reach-proof-by-removal tests: `scribing_debt`'s feature gate,
  recovery's `SharedInputs` crash-candidate-track gate (strictly stronger
  than the transcript scan's weaker any-session gate — both tiers proven
  separately), `ready_cells`' dep-read gate, and the archive-fallback path
  (validation-slice3.md's repaired `archive/<feature>/<id>.json` layout);
- both hook entry points advisor note 1 flagged (chain-nudge: 0 cells
  scans with no active feature, exactly 1 with one; state-sync: exactly 1
  every run, unconditionally — a different shape from chain-nudge's own
  gate);
- a negative control proving `build_status`'s stdout payload is
  byte-identical regardless of prior counter state.

A genuine red-to-green transition was captured for this behavior_change/
high-risk cell's red-first proof tier: `git stash` reverted only the three
counter call-sites (keeping the new module/test file in place), the same
`cargo test` command then failed 10 of 12 tests with every counter reading
0, and `git stash pop` restored green — recorded in the cap's
`red_failure_evidence`.

Full verify chain green: `cargo test -p queen-bee --test read_accounting`
(12 passed) → `cargo build --release` → `bee-parity --status-check` (all
6 legs zero-diff) → `bee-parity --self-check` (PASS) → `queen-bench
--check` (pass:true; status warm p95 ~52-54 ms against the 70 ms e119fc8b
interim budget — the always-compiled counter's cost is negligible).

## Rework round (goal-check NEEDS_REVISION, 6/9 checks passed, fixability: automatic)

The judge verified by EXECUTION, not reading: it reproduced the red-first
run exactly, confirmed every reach-proof discriminates 0-vs-N, confirmed
the unit split, and established the release-profile cost claim more
strongly (1.65 ns/increment via a release-profile micro-benchmark — 16
increments ≈ 26 ns against a ~53 ms p95).

**The blocker:** `decisions_journal_parses` was counted at `decisions.rs`'s
own call sites (beside each `read_jsonl(&decisions_path(root))` call), not
inside the truly shared primitive. The judge proved this gameable: it
hand-wrote two extra real `read_jsonl(&decisions_path(root))` calls from a
brand-new location outside `decisions.rs` entirely, immediately above the
`active_decisions` call the bench-fixture baseline test exercises,
simulating exactly the shape a rust-port-23 hoist could take — the
baseline test's count did not move; three real store reads at a
`build_status`-level load point were completely invisible. `cells_dir_scans`/
`cell_dep_reads`/both transcript counters survive the same class of attack
because `bee-core` has no generic, multi-store directory-scan primitive
analogous to `fsutil::read_jsonl` for those stores to bypass through —
`list_cells`/`read_cell`/`scan_transcript_roots` already ARE the floor.

**Fix:** moved the counter into `fsutil::read_jsonl` itself (`crates/
bee-core/src/fsutil.rs`), incrementing only when the path resolves to
`.bee/decisions.jsonl` (never the archive file or any other jsonl store
sharing that generic function) — so ANY call site anywhere in the crate,
present or future, is counted. Removed the three now-redundant ad-hoc
calls from `decisions.rs`. Added a permanent regression test,
`injected_reads_at_a_build_status_level_load_point_are_still_counted_for_both_stores`,
reproducing the judge's own experiment for BOTH the decisions store and
the cells store (not argued for one), plus a `fsutil.rs`-local unit test
proving the path filter is narrow (does not miscount
`decisions-archive.jsonl`/`backlog.jsonl`).

A second genuine red-to-green transition was captured for THIS fix
specifically: `git stash` reverted only `fsutil.rs`/`decisions.rs` (back to
the pre-rework, judge-broken placement) while keeping the new regression
test in place — the new test failed (`left: 4, right: 6`) reproducing the
judge's exact finding — and `git stash pop` + rerun went green again.

Two smaller findings also addressed: (1) the "baseline-only evidence"
caveat (must-have truth 8) is now stated in the test file's header doc
comment, in `read_accounting`'s module doc comment, AND printed as part of
the baseline table's own stdout output — not only in the cell record; (2)
`queen_bench_bin()` no longer requires a prebuilt debug binary — it builds
`queen-bench` itself (`cargo build -p queen-bench`) the first time it's
needed, verified by deleting `target/debug/queen-bench` and re-running the
leg (it self-healed, 13/13 green).

Full verify chain re-run green after the fix (13 tests now; all other legs
unchanged in shape): `cargo test -p queen-bee --test read_accounting` →
`cargo build --release` → `bee-parity --status-check` (6/6 zero-diff) →
`bee-parity --self-check` (PASS) → `queen-bench --check` (pass:true,
status warm p95 ~52.6 ms).

## Files touched

- `crates/bee-core/src/read_accounting.rs` (new)
- `crates/bee-core/src/lib.rs`
- `crates/bee-core/src/decisions.rs`
- `crates/bee-core/src/cells.rs`
- `crates/bee-core/src/recovery.rs`
- `crates/bee-core/src/fsutil.rs` (rework: decisions-journal counter's real home)
- `crates/queen-bee/tests/read_accounting.rs` (new)

Full trace/evidence: `.bee/cells/rust-port-22.json`.
