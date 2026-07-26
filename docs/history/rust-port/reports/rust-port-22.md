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

## Files touched

- `crates/bee-core/src/read_accounting.rs` (new)
- `crates/bee-core/src/lib.rs`
- `crates/bee-core/src/decisions.rs`
- `crates/bee-core/src/cells.rs`
- `crates/bee-core/src/recovery.rs`
- `crates/queen-bee/tests/read_accounting.rs` (new)

Full trace/evidence: `.bee/cells/rust-port-22.json`.
