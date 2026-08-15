---
artifact_contract: bee-implement-plan/v1
feature: gate-help-drift
lane: small
status: Ready for Review
updated: 2026-08-16
sources: [CONTEXT.md]
decisions: [D1, D2]
---

# Implementation Plan: gate-help-drift

**Goal** — `bee --help --json` and `bee state gate --help --json` list `--actor`, `--bypass-level`, `--reason`, matching the binary (D1), with a drift test so a new flag cannot ship unlisted (D2).

**In scope** — hand-edit the generated help payload for `state.gate` and `gate`; one drift test.
**Out of scope** — restoring the deleted generator script.

**Affected files**
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — add the three parameter entries to both commands (D1)
- `packages/bee-rs/crates/bee/tests/registry_contracts.rs` — pin payload parameters against set_gate.rs known-flags list (D2)

**Validation** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml` → expected: green; `bee state gate --help --json` lists actor, bypass-level, reason. Evidence: pending

**Risk** — none — help-surface only, no behavior change.
**Rollback** — revert the cell's commit.

**Open questions** — none — ready for review
