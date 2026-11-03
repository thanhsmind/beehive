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
  owns.code: [packages/bee-rs/crates/bee/src/main.rs, packages/bee-rs/crates/bee/src/catalog.rs, packages/bee-rs/crates/bee/src/doctor.rs, packages/bee-rs/crates/bee/src/fsutil.rs, packages/bee-rs/crates/bee/src/jsjson.rs, packages/bee-rs/crates/bee/src/lease_store.rs, packages/bee-rs/crates/bee/src/link_invalid.rs, packages/bee-rs/crates/bee/src/lock.rs, packages/bee-rs/crates/bee/src/nested_checkout.rs, packages/bee-rs/crates/bee/src/path_identity.rs, packages/bee-rs/crates/bee/src/registry.rs, packages/bee-rs/crates/bee/src/roots.rs, packages/bee-rs/crates/bee/src/router.rs, packages/bee-rs/crates/bee/src/shell.rs, packages/bee-rs/crates/bee/src/state.rs, packages/bee-rs/crates/bee/src/textutil.rs, packages/bee-rs/crates/bee/src/uat.rs, packages/bee-rs/crates/bee/src/version.rs, "packages/bee-rs/crates/bee/src/devtools/*", "packages/bee-rs/crates/bee/src/verbs/status_full/*", packages/bee-rs/crates/bee/src/verbs/status_brief.rs, packages/bee-rs/crates/bee/src/verbs/help.rs]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/src/doctor/tests.rs, packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs, packages/bee-rs/crates/bee/tests/front_door.rs, packages/bee-rs/crates/bee/tests/registry_contracts.rs, packages/bee-rs/crates/bee/tests/registry_dispatch.rs]
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

## Concepts in this area

- [Compiled runtime: the command surface — flow verbs, aliases, and the plumbing namespace](command-surface.md) — Why the CLI shows a small default surface instead of its whole registry, which verbs earn a place in it, how a flow verb is an alias rather than a second implementation, what every porcelain verb owes its caller at the point of contact, and why drift detection still hashes the full registry the split hides.
- [Compiled runtime: how a port is proven faithful](parity-and-conformance-proof-discipline.md) — The house discipline for proving equivalence against a frozen reference — parity legs and scenarios, the single volatility allowlist, per-leg negative controls, oracle rules for importable and command-only units, the conformance rig's five elements, environment pinning, and the byte-versus-parsed comparison rule with the meta-test that guards the instrument itself.
- [Compiled runtime: per-command performance budgets and the host-real fixture floors](performance-budgets-and-fixture-floors.md) — Speed is a gated contract, not an aspiration: per-command budgets measured spawn-inclusive over a store pinned to real sizes, both cache states reported, the reference figure recorded beside every result. Includes the status supersession to an interim 70 ms, its measured cause, and the mandatory follow-up that tightens it.
- [Compiled runtime: prompt files, and the learned-context block that closes the learn-to-use loop](prompt-files-and-learned-context.md) — Why every machine-assembled prompt body lives in a file rather than in code, what the split between template wording and computing logic buys, how a dispatched worker is handed the project's own learned context instead of re-deriving it, and why every source in that resolution chain fails silently.

## Patterns in this area

- [A counter placed at today's reader is gameable by the refactor it was built to judge](../../patterns/20260727-a-counter-at-the-reader-entry-is-gameable-by-the-refactor-it-judges.md) — Read counters built to measure a deduplication sat beside one store's call sites rather than inside the shared primitive; injecting two real reads at the level the refactor would hoist to left the count unchanged and the test green. The dedup would have reported the target number while twice as many reads happened.
- [A sweeping consolidation eats the call site that was different on purpose](../../patterns/20260806-a-sweeping-consolidation-eats-the-call-site-that-was-different-on-purpose.md) — Folding seven copies of a helper into one converted every caller to the new default — including the one guard whose measure is dictated by an external validator, which the sweep could not tell apart from drift; the same sweep carried a comment whose stated reason for the surviving exception was contradicted by the code's own test.
- [An optimization inherits the lifetime of what it optimizes](../../patterns/20260806-an-optimization-inherits-the-lifetime-of-what-it-optimizes.md) — exec-speed measured, shipped and proved five real speedups in the runtime bee then ran on; five days later that runtime was retired and four of the five have no surviving code at all — the wins were real, the substrate was not, and nothing in the feature's record shows the substrate's remaining lifetime being weighed before the work was scoped.
- [A generated view in a cell's scope puts its source record in scope too](../../patterns/20260818-a-generated-view-in-scope-puts-its-source-record-in-scope.md) — When a cell names a file whose content is derived from a data record (a catalog rendered off a registry payload, a projection off a store), the record is the real edit surface and belongs in the cell's files — the derived file alone cannot change. Three deliveries hit this on the same pair (catalog off the command registry) before it was named: each cell listed only the derived file and then touched, and had to reserve, the source record as unplanned fallout.
- [A green count is not evidence that your new test ran](../../patterns/20260818-a-green-count-is-not-evidence-that-your-new-test-ran.md) — A green count is not evidence that your new test ran
- [A seam added for testability can relocate the untested join instead of closing it](../../patterns/20260819-a-seam-added-for-testability-can-relocate-the-untested-join-instead-of-closing-it.md) — Threading construction through a new seam makes the inner hop testable while pushing the real production argument one level up, out of every test's reach — a closure literal written at a call site is reachable by no test, so the untested join relocates instead of closing. Measured on the herding backend seam: the whole workspace stayed green under a mutation that made the production closure ignore both of its arguments and construct with constants, while the doc comment added alongside asserted the seam was the only place a backend is built from a resolved pair.
- [When a guard cannot be made to fail, find which check already owns that half](../../patterns/20260819-when-a-guard-cannot-be-made-to-fail-find-which-check-already-owns-that-half.md) — A red-first proof that cannot go red the obvious way is information about the system, not a licence to weaken the guard — the fleet/bee crate boundary turned out to have two enforcement mechanisms, cargo's cycle check refusing any NORMAL `bee` dependency before a test body runs, and the new manifest test owning the dev, build, and target-conditional half cargo does not, so the red-first proof had to use a dev-dependency and the decision's 'only mechanism' wording is true of the boundary but false of any single check.
- [A faked seam hides the parse](../../patterns/20260821-a-faked-seam-hides-the-parse.md) — A trait seam that every test fakes proves the seam is exercised, never that the parse behind it is right; a fake returns whatever the test configured, so the one thing only a real process reply can falsify — the extraction of a live response — stays unchecked while the suite is green.
- [A vendored binary is a second place the feature must land](../../patterns/20260821-a-vendored-binary-is-a-second-place-the-feature-must-land.md) — A vendored binary is a second place the feature must land
- [Instruction text is an untested code path](../../patterns/20260821-instruction-text-is-an-untested-code-path.md) — Instruction text is an untested code path
- [A flake is a defect in the test's mechanism, not noise to retry past](../../patterns/20260825-a-flake-is-a-defect-in-the-tests-mechanism.md) — A flake is a defect in the test's mechanism, not noise to retry past
- [Shape-driven validation silently dissolves the must-not rules](../../patterns/20260826-shape-driven-validation-dissolves-the-must-not-rules.md) — Rebuilding a validator from allow-list to walk-what-is-configured is right for open sets, but every existing MUST-NOT rule needs an explicit deny arm carried over — otherwise the forbidden key becomes legal without anyone deciding it.
- [A comparison blind to the contract's own dimension passes the regression it exists to catch](../../patterns/20260726-a-comparison-blind-to-the-contracts-own-dimension.md) — Assertions compared parsed values under a map type whose equality ignores key order, while the contract was byte-for-byte file equality. Two real key-order breaks lived in that blind spot simultaneously and every assertion stayed green; the fix was byte comparison plus a permanent meta-test asserting both that parsed equality is order-blind on this build and that the byte comparator flags the same pair.
- [A dependency feature flag can re-alias an operation existing code already calls](../../patterns/20260726-a-dependency-feature-flag-can-alias-an-existing-call.md) — Enabling a serializer's order-preserving feature also re-aliased its map removal to a swap-with-last removal, silently reordering two files written by code the change never touched — one of them under a byte-for-byte compatibility contract. The diff was one manifest line; the affected call sites were nowhere in it.
- [Moving a shared type down a crate picks the form that keeps an in-flight sibling's call sites byte-identical](../../patterns/20260823-moving-a-shared-type-down-a-crate-keeps-sibling-call-sites-byte-identical.md) — In a parallel wave, one cell moved the screen classifier into a lower crate and re-exported it under an alias; a sibling cell editing call sites of the old path at the same moment broke at merge. The move is a cross-cutting change inside a wave: keep the old path valid (a re-export at the old name, same signatures) until every in-flight sibling has landed, then retire it in its own cell.
- [A shared struct's new field is a whole-crate change, not a local one](../../patterns/letter-reflection-lr-1-pitfall.md) — Adding a field to a record shared across the crate means every struct literal that builds it needs the field, whether or not the cell's file list named that path
- [A cell's named file may not be where the text it targets actually lives](../../patterns/slp-advisor-nudge-an-5-pitfall.md) — Help/prose text can live only in a hand-edited generated payload that regen does not overwrite from the source file the plan named
