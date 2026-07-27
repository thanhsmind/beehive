# rust-port Slice 2 — Advisor Consult (pre-Gate-3)

Advisor: fable (model-shaped, `models.claude.advisor`), read-only, 2026-07-26.
Evidence bundle: CONTEXT.md, validation-slice2.md, cells rust-port-13..20.

## Verdict

**PROCEED WITH NOTES** — repaired shape sound; premises independently verified (`crates/Cargo.toml:17` panic=abort, `write_guard.rs:684/:950` catch_unwind, `fixture.rs` hollow-fixture claim, `bee-parity/src/main.rs:34-43` single arm).

## Notes and disposition

1. **Cell 18's crash trigger does not exist yet** — the only Rust "crash" fixture (`hook_conformance.rs:568-586`) is a *handled* I/O fault absorbed via `Result`; it never unwinds and exits 0 under both profiles, so it can neither satisfy red-first nor exercise `catch_unwind`. → **applied**: cell 18 now names the seam decision explicitly (inert, env-gated, documented fixture-only panic seam, or a documented genuinely-panicking input) and adds a truth that the trigger must genuinely unwind.
2. **Cell 19's non-triviality covered the git cost but not the transcript cost** — a transcript-bytes floor can be met by files `detectCrashCandidates` never matches, so the ~37 ms cost could still measure zero (B1 recurring one block over). → **applied**: the self-test must now also require a non-empty recovery/crash-candidate block, and the fixture must match the encoded-project-dir shape.
3. **Allowlist containment leak** — a worker could leave the declared allowlist untouched and normalize inside diff helpers or the tree compare, passing the prohibition's letter. → **applied**: cell 15 now requires all volatility handling to live in the one declared allowlist, and the seeded-mutation negative control to run **per leg** (json and text).
4. **`[BLOCKED]`-with-profile is the right shape** for the expected-red bench (D5 licenses a per-command budget only after measured proof; the supersession is decide-altitude). Structural caution accepted: if cell 15 blocks, the orchestrator splits it — parity caps, bench becomes its own cell carrying the budget supersession — rather than leaving it open across a decision cycle.
5. **Re-rank applied**: cell 16's dep on 14 was artificial (workflow-store reuses `lock.rs`/`workspace.rs`, imports nothing gix) — deps narrowed to 13 so a crates.io block on 14 cannot hold 16/17 hostage.

Advice is data, not approval; no locked decision overridden.
