---
artifact_contract: bee-implement-plan/v1
feature: guard-refusal-wording
lane: small
status: Ready for Review
updated: 2026-08-16
sources: [CONTEXT.md]
decisions: [D1, D2, D3]
---

# Implementation Plan: guard-refusal-wording

**Goal** — a write-guard Bash refusal whose target cannot be resolved names the resolution failure and the raw token, never a fake path in the gate sentence (D1, D2).

**In scope** — classify unexpanded-shell-syntax tokens as unresolvable; sharpen the unresolvable-target refusal wording; regression tests (D3).
**Out of scope** — tokenizer/extractor rewrite.

**Affected files**
- `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` — classification before `canonical_rel_path` (D1), refusal branch wording (D2)
- `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs` — message text (D2)
- write-guard tests (same crate) — D3 regressions

**Validation** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml` → expected: green, with new regression tests. Evidence: pending

**Risk** — guard is fail-closed; classification change only widens the "unresolvable" bucket, never allows a write.
**Rollback** — revert the cell's commit.

**Open questions** — none — ready for review
