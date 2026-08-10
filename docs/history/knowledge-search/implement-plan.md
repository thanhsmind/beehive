---
artifact_contract: bee-implement-plan/v1
feature: knowledge-search
lane: small
status: Approved
updated: 2026-08-10
sources: [CONTEXT.md]
decisions: [D1, D2, D3, D4, D5, D6, D7]
---

# Implementation Plan: Knowledge Search

**Goal** — An agent mid-debug pulls matching patterns and area concepts out of the bundle by symptom text (`bee knowledge search --text`), at any phase, in any checkout (D1, D2, D7).

**In scope** — New read-only verb in the knowledge group (D1-D5, D7); skill wiring at the two debug moments (D6).
**Out of scope** — Any write path, new index files, changes to preamble digest or cell-dispatch learned context; decisions corpus (owned by `decisions search`, D2).

**Affected files**
- `packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs` — new verb: term-split, rank, why-matched rows (D3, D4)
- `packages/bee-rs/crates/bee/src/verbs/knowledge/{routing.rs,mod.rs}` — dispatch + read-only writes:[] contract (D7)
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — registry entry (hand-maintained payload; no regen chain exists — ks-1 trace records the gap)
- `skills/bee-swarming/SKILL.md` ("Execute (worker)"), `skills/bee-hive/references/scout-and-ticks.md` — name the pull move (D6, site erratum: bee-executing retired into bee-swarming 2026-07-31, commit 12ccd460)

**Validation** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml` → expected: green incl. registry contract suite. Evidence: ks-1 capped green (1381 passed, 3 ignored); ks-2 pending.

**Risk** — Registry payload is hand-edited JSON; drift check (tests/registry_contracts.rs) is the net.
**Rollback** — Revert the two cell commits; the verb is additive, nothing depends on it.

**Open questions** — none — ready for review
