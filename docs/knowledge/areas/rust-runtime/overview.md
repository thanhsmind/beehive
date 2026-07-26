---
type: bee.area
title: "Compiled runtime: purpose, guarantees, and the artifacts it writes"
description: "The compiled runtime replaces the interpreted reference on the paths a session pays for every turn, promising identical output and store writes with no child process spawned. What it guarantees, what stays dark until activation, the one additive artifact it writes, its fail-open crash contract, and the reference defect it reproduces on purpose."
tags: [rust-runtime, byte-compatibility, hooks, status, port]
timestamp: 2026-07-26
bee:
  id: area-rust-runtime-overview
  lifecycle: active
  areas: [rust-runtime]
  required_context: []
  decisions: [a7d7b3d5]
  sources: [docs/history/rust-port/CONTEXT.md, docs/history/rust-port/reports/rust-port-13.md, docs/history/rust-port/reports/rust-port-14.md, docs/history/rust-port/reports/rust-port-17.md, docs/history/rust-port/reports/rust-port-20.md]
  authoritative_for: "rust-runtime: purpose, guarantees, and the artifacts of the ported runtime"
---

## Purpose

The compiled runtime replaces the interpreted reference runtime on the paths a session pays for on every turn — status assembly, checkpoint handlers, orientation data — without changing a single thing an operator or an agent observes. Its promise is narrow and absolute: the same output, the same store writes, the same exit codes, at a fraction of the wall time and with no child process spawned on those paths.

The reference runtime remains frozen and authoritative for the whole port. It is never edited to make the compiled runtime agree; where the two differ, the compiled runtime is wrong by definition — including where the reference behaviour is itself a defect (see Edge Cases Settled).

## Entry Points & Triggers

- **Status assembly** — invoked on demand by an operator or by session orientation; returns the full state view as machine-readable data or as rendered text.
- **Checkpoint handlers** — invoked by the harness at lifecycle moments: a worker finishing, a session heartbeat interval, a write attempt, a subagent dispatch.
- Every ported handler is currently **dark**: the compiled binary carries it, and nothing routes to it. Activation is a separate, later step; until then the interpreted handlers remain the ones that run.

## Data Dictionary

- **Reference runtime** — the frozen interpreted implementation. The single oracle for every comparison; never modified during the port.
- **Compiled runtime** — the ported binary under construction.
- **Dark** — present and proven, but not wired: no dispatch path reaches it. A dark unit's proof is complete; only its activation is pending.
- **Review derivation cache** — an additive artifact the compiled runtime writes and the reference runtime never reads or writes. Keyed by the resolved repository head plus the conditions that change ancestry answers (shallow state, replacement refs, alternate object stores). Holds only definite answers. Deleting it costs speed and nothing else; a stale or corrupt file is discarded rather than trusted.

## Behaviors & Operations

**Status assembly.** Composes the state view from readers over the existing stores. Field order, the distinction between a null value and an absent key, and degraded markers for unreadable blocks all match the reference byte for byte, in both the machine-readable and the human-rendered form. Ancestry questions for the review block are answered in-process rather than by invoking the version-control tool, so the path spawns nothing.

**Checkpoint handlers.** The advisory handler emits its advisory when a worker is registered or the phase is one that expects worker returns, and stays silent otherwise — silence being an observable behaviour, proven on both runtimes. The synchronization handler is silent always, exits successfully always, refreshes heartbeat and lease and cross-checkout hold records, and rebuilds projected state under the shared lock; when the lock is held it skips the rebuild without complaint rather than waiting or failing.

**Fail-open on crash.** Any handler that panics internally exits successfully anyway and records one crash line naming the handler it came from, its fault, and when. A crash never blocks the operator's action. The crash line names the handler that actually crashed — not the wrapper that caught it.

## Actors & Access

- **Operator** — runs status directly; sees identical output from either runtime.
- **Session harness** — invokes checkpoint handlers; must never be blocked by one, including a crashing one.
- **Later slices** — consume the ported readers to assemble further commands; they compose the readers rather than re-parsing stores.

## Business Rules

- **R1** — The reference runtime is frozen for the duration of the port; no change to it is part of any port cell (D1).
- **R2** — Output and store writes must be byte-identical to the reference on the same inputs, including key order (D3).
- **R3** — The hot paths spawn no child process (D5).
- **R4** — A ported handler stays dark until an explicit activation step; a port cell never edits routing (D1).
- **R5** — The additive review derivation cache may exist only because it is invisible to the reference runtime and disposable; no existing store's shape changes to accommodate the port (decision a7d7b3d5).

## Edge Cases Settled

- **A defect in the reference is reproduced, not corrected.** One filter compares records by identity across two independently constructed lists, so the comparison never matches and expired-but-unreleased records are over-counted. The compiled runtime reproduces the over-count exactly, documented at the call site as deliberate, because byte-compatibility is the contract and a unilateral fix would diverge. Correcting it is a change to the reference, made deliberately and mirrored — never a silent improvement inside the port.
- **Records are written by patching what is on disk, not by round-tripping through a typed shape.** A typed shape emits every field it declares, including ones the reference never writes; round-tripping therefore grows keys that were never there. Writers read the stored record, set what changed, and write it back.

## Open Gaps

- Activation of the ported handlers is not yet designed; until then every proof is of behaviour nothing routes to.
- Status assembly repeats store reads within a single invocation (the same large files parsed several times, the same directories scanned repeatedly), which is what keeps it an order of magnitude above the original target — see the budgets concept and the filed follow-up.

## Pointers (implementation)

- Compiled runtime: `crates/queen-bee` (commands and hooks), `crates/bee-core` (readers, stores, locks).
- Frozen reference: `.bee/bin/bee.mjs`, `.bee/bin/lib/*.mjs`, `.bee/bin/hooks/*.mjs`.
- Review derivation cache: `.bee/runtime/review-git-cache.json`; ancestry answered in-process via `gix`.
- Locked decisions: `docs/history/rust-port/CONTEXT.md` (D1, D3, D5, D9, D10); addendum decision `a7d7b3d5`.
