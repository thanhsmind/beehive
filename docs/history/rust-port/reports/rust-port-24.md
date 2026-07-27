# rust-port-24 — status budget settled after the read dedup: target missed by ~4.5 ms, budget set to 30 ms

**[DONE]** — outcome (b): re-measuring `queen-bee status --json` after rust-port-23's
per-invocation read dedup shows warm p95 landing at **~24.5 ms**, above decision
e119fc8b's 20 ms follow-up target. The headroom rule (this cell's own text) applied
to the measured number, not the unmet 25 ms promise: `STATUS_BUDGET_MS` is now
**30 ms** dev p95 (CI perf smoke 90 ms, D5's 3x runner-variance ratio preserved).
`ping` keeps its untouched 5 ms floor. Decision **58ad0b5c** supersedes e119fc8b
with the measurement and the per-block profile.

**Worker:** Phil · **Lane:** high-risk · **Decisions:** D5, e119fc8b (superseded),
58ad0b5c (new).

Full trace, verify command and recorded output: `.bee/cells/rust-port-24.json`.

## Measurements (all spawn-inclusive, pinned D5 host-real fixture, warm cache gate)

| pass | runs | warm p95 | cold p95 | node baseline p95 |
|---|---|---|---|---|
| 1 | 50 | 24.737 ms | 26.258 ms | 178.850 ms |
| 2 | 50 | 25.302 ms | 25.679 ms | 184.217 ms |
| 3 | 50 | 23.666 ms | 28.180 ms | 184.455 ms |
| 4 | 200 (confirmatory) | 24.467 ms | 25.274 ms | 183.355 ms |
| 5 | 50 (post-edit, against new 30ms gate) | 24.756 ms | 25.685 ms | 180.911 ms |

Mean of passes 1-4: 24.543 ms. Cold and warm remain within run-to-run noise of
each other on this fixture, same secondary finding as e119fc8b/a7d7b3d5 — store
I/O, not the review-git cache, still dominates. Node baseline is stable at
~179-192 ms across all passes (unaffected — no node-side changes).

**Canonical figure:** pass 4 (200 runs) was chosen over any single 50-run pass
because it has the tightest percentile estimate and lands almost exactly on the
4-pass mean (24.467 vs 24.543 ms) — not the lowest of the observed values, so
not a favorably cherry-picked number. Using the single highest 50-run replicate
(25.302 ms, pass 2) instead would derive 35 ms by the same rule; that number is
reported here for transparency rather than silently discarded.

## Headroom-rule arithmetic

`ceil(24.467 / 5) * 5 = 25`, `+ 5 ms headroom = 30 ms`. Per the cell's worked
examples this matches the "22.0 ms -> 30 ms" case. **30 ms** is the new
`STATUS_BUDGET_MS`. CI perf smoke keeps D5's 3x runner-variance ratio:
`30 * 3 = 90 ms`.

Both outcomes named in the cell were live options; this is honestly outcome
**(b)** — the 20 ms target from e119fc8b is not reached (missed by ~4.5 ms),
so 58ad0b5c supersedes e119fc8b's unmet 25 ms promise with the measured number
rather than leaving it standing.

## Profile (warm run, `queen-bee status --json --profile` against a fresh D5 fixture)

19.352 ms in-process, 17.683 ms in measured blocks, 1.669 ms untimed envelope.
Dominant blocks, unchanged in identity from rust-port-15/23 but each individually
cheaper post-dedup:

| ms | % | block |
|---|---|---|
| 9.463 | 53.5 | `build_recovery_block` (transcripts) |
| 4.291 | 24.3 | `list_cells` (counts) |
| 1.767 | 10.0 | `read_backlog_counts` |
| 0.678 | 3.8 | `build_review_block` (gix, warm) |
| 0.518 | 2.9 | `tier_mix` |
| 0.465 | 2.6 | `global_scribing_debt` |
| everything else | < 0.6 | lanes, workers, state, reservations, config, contention, drift, capture, serialize+write |

The green-gate profile (main.rs:402 only auto-emits on RED) was captured
manually: `queen-bee status --json --profile`, run twice against a fresh
`queen-bench --generate` fixture — once to warm the review-git cache, once to
record. `build_recovery_block` and `list_cells` remain the two largest costs;
both are outside this cell's file scope (`crates/queen-bench/*`) to reduce
further, same constraint rust-port-15/23 already recorded.

## What landed

| Artifact | Substance |
|---|---|
| `crates/queen-bench/src/main.rs` | `STATUS_BUDGET_MS` 70.0 -> 30.0; `budget_source` JSON field and doc comments updated to cite decision 58ad0b5c; module doc header rewritten with the new number and CI figure |
| `crates/queen-bench/src/bench.rs` | `CacheState` doc comment updated: gate budget now 30 ms/58ad0b5c, secondary cold/warm-indistinguishable finding reconfirmed post-dedup |
| `docs/history/rust-port/reports/rust-port-24.md` (this file) | Measurement record, headroom arithmetic, profile |

No changes to `.bee/bin/` or `packages/bee/` (D1 freeze honored). Fixture floors
and measurement method unchanged (D5 escape honored — nothing shrunk, nothing
widened).

## Decision logged

**58ad0b5c-43b0-48a5-b6b4-06d3429e0e86** (supersedes e119fc8b): full text records
the four measurement passes, the headroom-rule arithmetic, the new 30 ms/90 ms
budget pair, and the warm profile. Rationale and rejected alternatives (leaving
the unmet target standing; rounding down to 25 ms; using the single highest
replicate for 35 ms) are recorded in the decision itself.

## Verify

`cargo build --release --manifest-path crates/Cargo.toml && cargo run --release
--manifest-path crates/Cargo.toml -p queen-bench -- --check` — exit 0, `pass:true`,
warm p95 24.756 ms against the new 30 ms budget. `cargo test --release -p
queen-bench`: 4 passed (1 suite). Full output recorded on the cell's verify
trace.
