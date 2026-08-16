---
type: bee.area
title: "Compiled runtime: purpose, guarantees, and the artifacts it writes"
description: "The compiled runtime replaces the interpreted reference on the paths a session pays for every turn, promising identical output and store writes with no child process spawned. What it guarantees, what stays dark until activation, the one additive artifact it writes, its fail-open crash contract, and the reference defect it reproduces on purpose."
tags: [rust-runtime, byte-compatibility, hooks, status, port]
timestamp: 2026-08-16
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

The interpreted reference is retired: no `.mjs` file remains anywhere in the repository, `.bee/bin/bee` is itself the compiled binary, and the runtime path (CLI, hooks, statusline data) requires no Node — D10's exit criterion (`docs/history/rust-port/CONTEXT.md`). The byte-identical-output and no-child-process guarantees that once anchored a live comparison against the reference are now simply this crate's own behaviour; there is no second implementation left to diverge from.

## Entry Points & Triggers

- **Status assembly** — invoked on demand by an operator or by session orientation; returns the full state view as machine-readable data or as rendered text.
- **Checkpoint handlers** — invoked by the harness at lifecycle moments: a worker finishing, a session heartbeat interval, a write attempt, a subagent dispatch.
- Every ported handler is live and routed: the compiled binary is the dispatch target for the CLI, the hooks, and the statusline data path, with no interpreted handler left behind it to fall back to (rust-port D10).

## Data Dictionary

- **Reference runtime** — the frozen interpreted implementation. The single oracle for every comparison; never modified during the port.
- **Compiled runtime** — the ported binary under construction.
- **Dark** — a term from the port era for a proven-but-unwired handler. Nothing in the current runtime is dark; every ported handler is wired and routing (see Entry Points).
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
- **R4** — Activation is complete: every ported handler is wired into live routing, and no interpreted handler remains to fall back to (D1, D10).
- **R5** — The additive review derivation cache may exist only because it is invisible to the reference runtime and disposable; no existing store's shape changes to accommodate the port (decision a7d7b3d5).

## Edge Cases Settled

- **A defect in the reference is reproduced, not corrected.** One filter compares records by identity across two independently constructed lists, so the comparison never matches and expired-but-unreleased records are over-counted. The compiled runtime reproduces the over-count exactly, documented at the call site as deliberate, because byte-compatibility is the contract and a unilateral fix would diverge. Correcting it is a change to the reference, made deliberately and mirrored — never a silent improvement inside the port.
- **Records are written by patching what is on disk, not by round-tripping through a typed shape.** A typed shape emits every field it declares, including ones the reference never writes; round-tripping therefore grows keys that were never there. Writers read the stored record, set what changed, and write it back.

## Open Gaps

- Status assembly repeats store reads within a single invocation (the same large files parsed several times, the same directories scanned repeatedly), which is what keeps it an order of magnitude above the original target — see the budgets concept and the filed follow-up.

## Pointers (implementation)

- Compiled runtime: one crate, `packages/bee-rs/crates/bee` — commands and hooks under `src/verbs/` and `src/hooks/`, readers/stores/locks under crate root modules (`state.rs`, `roots.rs`, `nested_checkout.rs`, and peers). No `crates/queen-bee` or `crates/bee-core` split exists; the port consolidated into this one crate.
- Live binary: `.bee/bin/bee` IS the compiled binary (an ELF executable) built from that crate — there is no interpreted `.mjs` reference anywhere in the repository (D10's exit criterion; `fd -e mjs .` returns nothing).
- Review derivation cache: `.bee/runtime/review-git-cache.json`; ancestry answered in-process via `gix`.
- Locked decisions: `docs/history/rust-port/CONTEXT.md` (D1, D3, D5, D9, D10); addendum decision `a7d7b3d5`.
