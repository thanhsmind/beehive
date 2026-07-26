//! read_accounting — the read-instrumentation seam this cell (rust-port-22,
//! CONTEXT.md D3/D5, `docs/history/rust-port/plan-slice3.md` slice 3 cell
//! 1 of 3) adds so rust-port-23's per-invocation store-read dedup can be
//! proven as a genuine red-to-green transition rather than an assertion
//! written after the fact. Nothing in the pre-existing suite counted reads
//! before this cell; every parity/oracle test compares final output only,
//! which is exactly why the duplicate reads this instrument now proves
//! survived unnoticed.
//!
//! ## Mechanism (cold-pickup C3, decided — not delegated to this cell)
//!
//! An ALWAYS-COMPILED counter, never `#[cfg(test)]` and never behind a
//! non-default cargo feature:
//!
//! - `#[cfg(test)]` cannot work — `bee-core`'s `cfg(test)` is inactive when
//!   `queen-bee`'s integration tests link it as an ordinary dependency, so a
//!   `cfg(test)`-gated counter would simply not exist in the binary those
//!   tests exercise.
//! - A non-default cargo feature is ruled out by this cell's own proof
//!   surface: `bee-parity` spawns the sibling RELEASE binary
//!   (`bee-parity/src/main.rs:269`), so a seam behind a non-default feature
//!   would be absent from exactly the build the parity legs measure.
//!
//! The counter therefore ships in every build, including release. That is
//! the accepted trade (validation `ASSUMPTIONS` row, repaired): this
//! module's job is to prove the cost is negligible against the status
//! budget (`queen-bench --check`, part of this cell's own `verify`), not to
//! pretend an off state exists.
//!
//! Each counter is a `thread_local!` `AtomicU64` (relaxed ordering): still a
//! genuine atomic type (satisfying the "atomic counter" decision) while
//! giving free per-invocation isolation — every real `queen-bee` process
//! invocation runs its store reads on a single thread from a zeroed static,
//! and every test function in this workspace's default (parallel,
//! thread-per-test) harness gets its own thread-local view, made airtight
//! by calling [`reset`] at the top of any test that reads a [`snapshot`]
//! (the harness may reuse an OS thread across two different test
//! functions, so relying on "this thread has never touched these counters
//! before" would be unsound without an explicit reset).
//!
//! ## Placement (advisor note 2, must-have, NOT negotiable)
//!
//! Every counter below lives at the LOWEST shared read primitive for its
//! store class — the actual file/jsonl read or directory-scan/stat call —
//! never at today's higher-level reader-function entries
//! (`scribing_debt`, `tier_mix`, `ceiling_scarcity_warning`, `ready_cells`,
//! `global_scribing_debt`, `active_decisions` as a single unit, ...). A
//! counter placed at one of those entries would be gameable by the very
//! refactor it is meant to judge: rust-port-23 changes several of those
//! functions to accept a caller-supplied, already-loaded value instead of
//! performing their own read, so a function-entry counter would keep
//! firing once per call regardless of whether a real read still happens
//! underneath — reporting a number disconnected from actual disk I/O. A
//! counter at the shared primitive itself has no such blind spot: when
//! rust-port-23 removes a redundant call to that primitive, the count
//! actually drops.
//!
//! **"Lowest" is not a synonym for "inside the domain module."** The first
//! cut of this seam counted `decisions_journal_parses` inside
//! `decisions.rs`'s own call sites — one level lower than the higher-level
//! readers, but not the FLOOR. A goal-check judge proved that placement
//! gameable: it hand-wrote two extra real
//! `fsutil::read_jsonl(&decisions_path(root))` calls from a brand-new
//! location outside `decisions.rs` entirely (simulating a rust-port-23
//! hoist that pre-loads the journal before calling into `decisions.rs`/
//! `recovery.rs` at all), and the baseline test's count did not move. The
//! counter now lives inside `fsutil::read_jsonl` itself (see that
//! function's doc comment), keyed on the path — the true floor, since
//! `read_jsonl` is the one generic primitive EVERY current and future
//! decisions-journal reader must funnel through, no matter which module
//! writes it. `cells_dir_scans` (inside `cells::list_cells`'s
//! `fs::read_dir`) and `cell_dep_reads` (inside `cells::read_cell`) do not
//! have this problem in the first place: `bee-core` has no generic,
//! multi-store "scan a directory of JSON files" primitive the way it has
//! `read_jsonl` for jsonl files, so `list_cells`/`read_cell` already ARE
//! the lowest shared primitive for the cells store — a hoist would have to
//! reimplement their fail-open/archive-fallback logic from scratch to
//! bypass them, which is a materially different (and easily noticed) thing
//! from calling an existing generic utility function directly.
//!
//! ## BASELINE-ONLY EVIDENCE (must-have truth 8 — read this before trusting
//! ## a reach-proof against POST-dedup code)
//!
//! Every reach-proof in `crates/queen-bee/tests/read_accounting.rs` (and
//! the printed baseline table it produces) is evidence about TODAY'S
//! un-deduped code only. Once rust-port-23 lands, a conditional site that
//! shows as "reached" here can start reporting its count unconditionally
//! for a reason that has NOTHING to do with the original gate: if the
//! dedup hoists a read above the conditional consumer that used to gate
//! it, the hoisted primitive call fires once per invocation regardless of
//! whether that consumer still runs — the count still says "1", but "1"
//! no longer means "the gated consumer ran". A rust-port-23 worker reusing
//! this table must re-derive reachability against the NEW call graph, not
//! assume last cell's reach-proofs still describe it.
//!
//! ## Store classes and their units (validation W1 — DEFINE WHAT YOU COUNT)
//!
//! - **`decisions_journal_parses`** — one increment per actual
//!   `read_jsonl(&decisions_path(root))` call ANYWHERE in the crate (the
//!   `.bee/decisions.jsonl` journal only, never the archive file) —
//!   counted inside `fsutil::read_jsonl` itself (see the placement note
//!   above), not inside `crate::decisions`. Unit: one file read = one
//!   count, unambiguous. Today that fires at the two real call sites
//!   inside `decisions.rs` (`build_tag_overlay`'s own read, and
//!   `active_decisions`'s own read in both its `all` and `!all` branches) —
//!   NOT once per call to `active_decisions` as a whole, because one call
//!   to `active_decisions` performs TWO real reads today (its own event
//!   read plus `build_tag_overlay`'s), and collapsing that into a single
//!   per-call increment would hide exactly the duplication this instrument
//!   exists to prove.
//! - **`cells_dir_scans`** — one increment per `fs::read_dir` attempt
//!   inside `cells::list_cells`, the single funnel every "cells directory
//!   scanned" call site in today's code goes through
//!   (`ready_cells`/`tier_mix`/`ceiling_scarcity_warning`/`scribing_debt`/
//!   `global_scribing_debt` via `list_cells_where`, and
//!   `recovery::detect_crash_candidates`'s own direct `cells::list_cells`
//!   call). Unit: one directory-scan attempt, counted whether or not the
//!   scan succeeds (a failed `read_dir` is still a real syscall cost).
//! - **`cell_dep_reads`** — one increment per `cells::read_cell` call (one
//!   dependency-resolution attempt), counted SEPARATELY from
//!   `cells_dir_scans` (validation W6): `read_cell` is a single active-file
//!   read that only cascades into an archive-directory scan plus
//!   per-candidate file reads on a genuine miss. That archive fallback is
//!   NOT broken out as its own counter — it is inherent to one logical dep
//!   lookup, folded into this same bucket — because rust-port-23's dedup
//!   can move the *cells-directory-scan* cost (batching `ready_cells`'
//!   scan) without moving this per-dep cost at all, so the two must stay
//!   independently observable rather than merged into one number that
//!   would hide which half moved.
//! - **`transcript_root_scan_invocations`** — one increment per call to
//!   `recovery::scan_transcript_roots`, the shared primitive both
//!   `detect_crash_candidates` and `build_recovery_block` call directly.
//!   Unit: one function invocation, deliberately NOT a filesystem-operation
//!   count — this is the number that reconciles with decision e119fc8b's
//!   stated "2" and with `plan-slice3.md`'s discovery table, both of which
//!   meant "how many times was the scan performed", not "how many roots
//!   were stat'd".
//! - **`transcript_root_stat_ops`** — one increment per actual
//!   `fs::metadata` stat inside `scan_transcript_roots`'s per-root loop.
//!   Unit: one filesystem operation. This is DIFFERENT from
//!   `transcript_root_scan_invocations` whenever more than one root is
//!   configured (`.bee/config.json` `recovery.transcript_roots`): the
//!   queen-bench D5 fixture configures exactly one extra root beyond the
//!   always-present default Claude root, so its two invocations cost four
//!   real stats, not two — see
//!   `crates/queen-bee/tests/read_accounting.rs`'s
//!   `transcript_root_stat_ops_scale_with_configured_roots_while_invocations_do_not`
//!   test, which demonstrates the divergence directly rather than asserting
//!   it in prose alone.

use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    static DECISIONS_JOURNAL_PARSES: AtomicU64 = const { AtomicU64::new(0) };
    static CELLS_DIR_SCANS: AtomicU64 = const { AtomicU64::new(0) };
    static CELL_DEP_READS: AtomicU64 = const { AtomicU64::new(0) };
    static TRANSCRIPT_ROOT_SCAN_INVOCATIONS: AtomicU64 = const { AtomicU64::new(0) };
    static TRANSCRIPT_ROOT_STAT_OPS: AtomicU64 = const { AtomicU64::new(0) };
}

/// Record one real parse of `.bee/decisions.jsonl` (never the archive
/// file). Called at each of `decisions.rs`'s own `read_jsonl` call sites —
/// see the module doc comment for why this is not "once per
/// `active_decisions` call".
pub fn record_decisions_journal_parse() {
    DECISIONS_JOURNAL_PARSES.with(|c| c.fetch_add(1, Ordering::Relaxed));
}

/// Record one `.bee/cells/*.json` directory-scan attempt. Called inside
/// `cells::list_cells`, the single shared funnel — see the module doc
/// comment.
pub fn record_cells_dir_scan() {
    CELLS_DIR_SCANS.with(|c| c.fetch_add(1, Ordering::Relaxed));
}

/// Record one `cells::read_cell` dependency-resolution attempt (active
/// file plus, on a miss, its archive-directory fallback — folded into this
/// same bucket, see the module doc comment).
pub fn record_cell_dep_read() {
    CELL_DEP_READS.with(|c| c.fetch_add(1, Ordering::Relaxed));
}

/// Record one call to `recovery::scan_transcript_roots` (function-
/// invocation unit — reconciles with decision e119fc8b's stated "2").
pub fn record_transcript_root_scan_invocation() {
    TRANSCRIPT_ROOT_SCAN_INVOCATIONS.with(|c| c.fetch_add(1, Ordering::Relaxed));
}

/// Record one `fs::metadata` stat performed inside
/// `scan_transcript_roots`'s per-root loop (filesystem-operation unit —
/// scales with configured `recovery.transcript_roots` count, unlike
/// [`record_transcript_root_scan_invocation`]).
pub fn record_transcript_root_stat_op() {
    TRANSCRIPT_ROOT_STAT_OPS.with(|c| c.fetch_add(1, Ordering::Relaxed));
}

/// A per-invocation snapshot of every counter above, read from the CALLING
/// thread only (each counter is `thread_local!`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub decisions_journal_parses: u64,
    pub cells_dir_scans: u64,
    pub cell_dep_reads: u64,
    pub transcript_root_scan_invocations: u64,
    pub transcript_root_stat_ops: u64,
}

/// Read every counter on the calling thread without resetting them.
pub fn snapshot() -> Snapshot {
    Snapshot {
        decisions_journal_parses: DECISIONS_JOURNAL_PARSES.with(|c| c.load(Ordering::Relaxed)),
        cells_dir_scans: CELLS_DIR_SCANS.with(|c| c.load(Ordering::Relaxed)),
        cell_dep_reads: CELL_DEP_READS.with(|c| c.load(Ordering::Relaxed)),
        transcript_root_scan_invocations: TRANSCRIPT_ROOT_SCAN_INVOCATIONS.with(|c| c.load(Ordering::Relaxed)),
        transcript_root_stat_ops: TRANSCRIPT_ROOT_STAT_OPS.with(|c| c.load(Ordering::Relaxed)),
    }
}

/// Zero every counter on the CALLING thread. Never called by production
/// code (a real `queen-bee` process invocation already starts from a
/// zeroed static on a fresh thread) — this exists purely so a test can
/// isolate one invocation's counts from whatever a previous test run on
/// the same reused thread-pool thread left behind.
pub fn reset() {
    DECISIONS_JOURNAL_PARSES.with(|c| c.store(0, Ordering::Relaxed));
    CELLS_DIR_SCANS.with(|c| c.store(0, Ordering::Relaxed));
    CELL_DEP_READS.with(|c| c.store(0, Ordering::Relaxed));
    TRANSCRIPT_ROOT_SCAN_INVOCATIONS.with(|c| c.store(0, Ordering::Relaxed));
    TRANSCRIPT_ROOT_STAT_OPS.with(|c| c.store(0, Ordering::Relaxed));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_zeroes_every_counter_and_snapshot_reflects_it() {
        record_decisions_journal_parse();
        record_cells_dir_scan();
        record_cell_dep_read();
        record_transcript_root_scan_invocation();
        record_transcript_root_stat_op();
        assert_ne!(snapshot(), Snapshot::default());
        reset();
        assert_eq!(snapshot(), Snapshot::default());
    }

    #[test]
    fn each_counter_is_independent() {
        reset();
        record_decisions_journal_parse();
        record_decisions_journal_parse();
        record_cells_dir_scan();
        let snap = snapshot();
        assert_eq!(snap.decisions_journal_parses, 2);
        assert_eq!(snap.cells_dir_scans, 1);
        assert_eq!(snap.cell_dep_reads, 0);
        assert_eq!(snap.transcript_root_scan_invocations, 0);
        assert_eq!(snap.transcript_root_stat_ops, 0);
        reset();
    }
}
