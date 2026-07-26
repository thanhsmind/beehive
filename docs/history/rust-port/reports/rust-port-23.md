# rust-port-23 — per-invocation shared reads

[DONE] — `build_status` now reads each store **once per invocation**:
decisions-journal parses **4 → 1**, cells-directory scans **6 → 1**,
transcript-root scans **2 → 1** (queen-bench D5 fixture, same test and same
counter units that produced rust-port-22's baseline). `status` warm p95 fell
from ~52.6 ms to **26.96 ms** against the 70 ms interim budget.

The seam is `bee_core::shared_reads::SharedReads`: a caller-supplied,
**lazy** per-invocation memo (`OnceCell`, `!Sync` on purpose) whose accessors
load through the *existing* readers — `cells::list_cells`,
`decisions::active_decisions` (via `fsutil::read_jsonl`) — never around them,
so every load stays visible to rust-port-22's counters. `recovery`'s
competing `SharedInputs` was folded into it (validation W4), and
`last_durable_settlement` lost its `Option` injection argument: a fresh memo
*is* the old self-loading behavior.

`active_decisions` also dropped from two journal reads to one internally —
`build_tag_overlay` now shares the caller's read instead of re-reading the
same file.

## The three named failure modes

1. **The archive trap — guarded, and the guard proven discriminating.**
   Dep resolution deliberately still goes through `read_cell`
   (`ready_cells_from` takes the shared inventory for the *listing* only).
   `ready_cells_from_a_shared_inventory_still_resolves_an_archived_capped_dep`
   asserts the dep is genuinely absent from the shared inventory, then that
   the cell is still ready. It was proven to **fail (0 vs 1)** under a
   temporary mutation that resolved deps against the active-only inventory —
   the exact regression the cell warned would ship silently. Read counts
   cannot catch it (`cell_dep_reads` bundles the archive fallback), which is
   why the table itself now says so in its own stdout.
2. **Instrument gaming — avoided by construction and re-proven.**
   rust-port-22's anti-gaming regression test survives the dedup with its
   arithmetic rebased (baseline 1/1, injected 1+2 / 1+1): real reads injected
   from outside the domain modules still move the counters, so the 1/1/1
   claim stays falsifiable.
3. **The hook read profile — preserved by laziness, not by luck.** Both hook
   call sites moved to the shared-read signature, and all three rust-port-22
   hook baselines were re-run green: chain-nudge still scans **zero** cells
   with no active feature (it early-returns before touching `shared.cells()`),
   exactly one with a feature; state-sync stays unconditionally one. An eager
   shared struct would have made that scan unconditional on every
   SubagentStop event with every correctness test still green — this is why
   the memo is lazy.

The transcript-root hoist sits in `build_recovery_block`, **above**
`detect_crash_candidates`' no-sessions early return. Returning roots
alongside candidates would have rendered an empty `roots` block on exactly
the fixtures with no session records; `detect_crash_candidates` keeps its own
below-the-gate scan for standalone callers.

## Deviations

The cap's `deviations` array is empty; they are recorded here.

- **stderr warning count on a bad configured transcript root.**
  `scan_transcript_roots` emits one stderr warning per bad *configured* root
  per invocation, so halving the scans halves those warnings. This is off the
  D7a oracle surface — `bee-parity` compares stdout, exit code and the
  post-run store tree, and its `RunResult` has no stderr field at all
  (`crates/bee-parity/src/differ.rs`). The duplicated warning was an artifact
  of the duplicated scan, not a contract.
- **Key insertion order preserved deliberately.** `ceiling_scarcity_warning`
  now consumes the `tier_mix` result computed just above it, so the two values
  are computed in dependency order but *inserted* in bee.mjs's source order
  (`tier_mix` then `ceiling_scarcity`) — `JSON.stringify` emits insertion
  order, and that is byte-parity contract.
- **Public root-taking wrappers kept.** `ready_cells`, `tier_mix`,
  `scribing_debt`, `global_scribing_debt`, `ceiling_scarcity_warning` and
  `detect_crash_candidates` still exist with their original signatures, each
  now a one-line wrapper over the shared shape. There is one implementation
  per reader, not two competing ones.

## Verify

Full chain green, exit 0 — `cargo test -p queen-bee` (137 passed / 11 suites)
→ `cargo test -p bee-core` (176 passed / 9 suites) → `cargo build --release`
→ `bee-parity --status-check` (6/6 legs zero-diff, per-leg seeded-mutation
detected on every leg) → `bee-parity --self-check` (PASS).
`heavyhooks_conformance` was **actually re-run** (19 passed, 0.81 s), not
merely recompiled. Beyond the chain: `queen-bench --check` `pass:true`.

Two genuine red-to-green transitions were captured (the dedup itself, and the
archive-trap guard under mutation) — both in the cap's
`red_failure_evidence`.

Consistency finding logged as decision `19ee5bf2-6ca0-4ec6-a254-51fb48f3afc8`:
*each store is read at ONE instant per invocation; cross-store consistency
remains unguaranteed, exactly as today* — explicitly **not** snapshot
semantics, and noting that `status` takes no D9 locks.

## Files touched

- `crates/bee-core/src/shared_reads.rs` (new), `lib.rs`
- `crates/bee-core/src/cells.rs`, `decisions.rs`, `recovery.rs`
- `crates/queen-bee/src/status.rs`, `hooks/chain_nudge.rs`, `hooks/state_sync.rs`
- `crates/queen-bee/tests/read_accounting.rs`, `crates/bee-core/tests/status_readers_b2.rs`

Full trace/evidence: `.bee/cells/rust-port-23.json`.
