---
type: bee.area
title: "Compiled runtime: how a port is proven faithful"
description: "The house discipline for proving equivalence against a frozen reference — parity legs and scenarios, the single volatility allowlist, per-leg negative controls, oracle rules for importable and command-only units, the conformance rig's five elements, environment pinning, and the byte-versus-parsed comparison rule with the meta-test that guards the instrument itself."
tags: [rust-runtime, proof-discipline, parity, conformance, oracles]
timestamp: 2026-07-26
bee:
  id: area-rust-runtime-proof-discipline
  lifecycle: active
  areas: [rust-runtime]
  required_context: []
  decisions: []
  sources: [docs/history/rust-port/reports/rust-port-15.md, docs/history/rust-port/reports/rust-port-17.md, docs/history/rust-port/reports/rust-port-20.md, docs/history/rust-port/reports/rust-port-21.md]
  authoritative_for: "rust-runtime: how a port is proven faithful against a frozen reference"
---

## Purpose

A port is only as good as the instrument that says it is faithful. This is the house discipline for proving equivalence against a frozen reference: what is compared, at what granularity, against which oracle, and what makes each comparison believable rather than decorative.

## Entry Points & Triggers

- **Whole-command parity** — runs both runtimes over clones of one generated fixture and diffs what a caller sees.
- **Unit-level oracle diffs** — drive the reference's own readers directly and compare a ported reader's result on the same input.
- **Handler conformance** — runs both runtimes' checkpoint handlers over a seeded environment and compares output and side effects.

## Data Dictionary

- **Leg** — one runtime, one output form, one scenario: the unit a diff is taken over.
- **Scenario** — the shape of the fixture a leg runs against. A quiet fixture proves the common path; an enriched fixture seeds the branches a quiet store never reaches.
- **Volatility allowlist** — the single declared place where unavoidable variation is handled. Nothing normalizes anywhere else.
- **Negative control** — a deliberate divergence seeded into one side, which the comparison must detect. Per leg, never once for the whole run.
- **Oracle** — the frozen reference, executed. Never a hand-written expectation standing in for it.
- **Red-first evidence** — the recorded failure a proof produces when the defect it guards is reintroduced.

## Behaviors & Operations

**Whole-command parity.** Both runtimes run over clones of the same fixture in throwaway roots outside the repository. Their output is compared byte for byte after the declared allowlist and nothing else; their exit codes are compared independently of the diff, because two identical failures are not agreement; and the resulting store trees are compared, with only whole-path exclusions for logs, caches, scratch, and the compiled runtime's own additive artifacts. Each leg carries its own seeded divergence which the comparison must catch — asserted on the output diff specifically, since a tree-level or exit-level catch would let a rendering divergence pass unseen.

**Unit-level oracle diffs.** Where a reader is importable, the driver imports the frozen module and compares results on the same fixture. Where the behaviour lives inside a command and has no importable entry, the whole command is the oracle and the comparison is taken over the relevant part of its output. A ported unit with no oracle diff at all is unproven, however obviously correct it looks.

**Handler conformance.** The environment is seeded into a throwaway root and every seeded file is checksum-verified against the repository copy, so the run provably drives the frozen handler rather than a drifted copy. Each silence case is paired with a twin that differs by one field and must produce output, so a handler that silently does nothing cannot pass by accident. An independent verifier confirms an unseeded root is detected as invalid. A meta-proof requires the comparison itself to fail on a deliberately divergent pair.

**Environment pinning.** The harness clears ambient identity from the launching shell and pins its own per scenario, giving both legs identical environments, then asserts a positive marker in the compared output proving the branch it pinned for was actually reached.

## Business Rules

- **R1** — All volatility handling lives in the one declared allowlist. Normalization inside a diff helper or a tree comparison is a defect even when the run is green (D3).
- **R2** — Comparisons of anything under a byte contract are taken over bytes. Parsing to compare discards exactly what the contract is about; where volatile values must be tolerated, they are redacted in the raw text and the redaction fails closed.
- **R3** — Every ported unit named by a cell needs an oracle diff. Correct-by-inspection is not a proof.
- **R4** — Negative controls are per leg and per class, not per run.
- **R5** — A fixture must be shown to reach the branch its test names; the removal proof — neutralize the seeding, require the failure — is what shows it.
- **R6** — A defect fixed in the port ships with red-first evidence: the failure recorded before the fix, and green after.
- **R7** — When a proof cannot reach a budget or a zero-diff result, the run reports the measurement and stops. Widening a tolerance or shrinking a fixture to produce green is refused (D5).

- **R8** — A parity guard outlives its usefulness the moment the reference implementation is removed, and from then on it is pure cost. Such a guard refuses work whose ordering could not be proven identical to the reference; with the reference gone there is nothing left to differ from, the port's own behavior IS the contract, and every refusal the guard still issues is a dead end rather than a fallback. Retiring one is therefore a deletion, not a migration (retire-collation-guard D1/D2, 2026-08-14).

- **R9** — A parity guard must be no narrower than the model it guards. When the guard admits a smaller alphabet than the comparator it protects, it refuses inputs the comparator was deliberately built to handle — and because the refusal travels as a delegate exit rather than a stated error, the mismatch is invisible until a real value trips it. Check the guard against its model, not against intuition about what looks exotic (retire-collation-guard, 2026-08-14).

- **R10** — A test that asserts a guard DISABLES a command is asserting the defect once the guard is retired. Such assertions are inverted, never deleted: the new case asserts the command succeeds AND produces a specific stable order. A test reduced to "it does not fail" has traded coverage for green (retire-collation-guard, cell rcg-1, 2026-08-14).

- **R11** — Repo-relative doc paths in output and stored records are spelled with forward
  slashes on every platform; a Windows-native spelling is normalized at the boundary before
  comparison or storage, never leaked into a record another platform will read. Tests assert
  the forward-slash spelling on every platform, not per-OS variants (windows-ci-path-fixes,
  cells wcpf-1..4, issue #94, 2026-08-16; decision logged same day). The same sweep settled two
  siblings: errno-class checks widened to cover Windows' error codes, and path fixtures
  canonicalized so a symlinked temp dir compares equal to its resolved form.
  R11 governs repo-relative paths in output and stored records. A RENDERED absolute
  filesystem path is the opposite case: it carries the platform's own separator, so its
  assertion derives the expected string from the same `Path::join` the renderer used,
  rather than spelling either separator (windows-suite-green, cell wsg-1, 2026-08-21).
- **Windows test hermeticity is pinned in fixtures, never assumed.** Test fixtures pin
  `core.autocrlf` off — Windows runners default it on, silently breaking exact-LF
  assertions — and path assertions compare canonical identity, never string spelling,
  so an 8.3 short temp path and its long form compare equal
  (windows-ci-test-fix, 2026-08-17).

## Edge Cases Settled

- **A meta-test guards the instrument, not the code.** Because parsed-value equality is order-blind under the serializer configuration this workspace uses, a permanent test asserts both halves — that parsed equality really is order-blind here, and that the byte comparator really does flag the same pair — and says so loudly if the premise ever changes.
- **Reviewers falsify rather than reason where they can.** The strongest verdicts in this port came from reintroducing a defect in a scratch copy and observing the specific test go red; a scratch area inside the working tree exists for this, because writes outside it are refused.
- **A fixture generator has consumers.** Growing the shared fixture can break another cell's proof; fixture changes are checked against every proof that reads them.

## Open Gaps

- **Delegate exits still read as argument errors.** A handler that declines by returning "not handled" — the shape that used to hand work to the reference implementation — now reaches a dispatcher with nowhere to send it, and the user is told the argument shape was wrong. Three commands have been found this way and one was repaired at the handler; the dispatcher message itself is unchanged, so the next one will present the same way (filed 2026-08-14).
- One rendering path has no oracle because the reference derives it from its own working directory; its equality is established by direct comparison of the two literals instead, which no automated run repeats.
- Table-driven cases prove one rung each; a priority-order regression between two competing fields is not detectable unless a case carries both.

## Pointers (implementation)

- Whole-command parity: `crates/bee-parity` (`--self-check`, `--status-check`; allowlist confined to `normalize.rs`; exclusions in `differ.rs`).
- Fixture generation and budgets: `crates/queen-bench`.
- Handler conformance rigs: `crates/queen-bee/tests/hook_conformance.rs` (rig of record), `modelguard_conformance.rs`, `heavyhooks_conformance.rs`; the rig is copied in-file by design, never shared.
- Reader oracles: `crates/bee-core/tests/*_oracle.rs`, `status_readers_*.rs`.
