---
artifact_contract: bee-plan/v1
mode: high-risk
slice: 3
approved_gate2: 2026-07-26  # auto-approved under gate_bypass=total
---

# rust-port Slice 3 — per-invocation store-read dedup on the status path

## Mode gate

Risk flags counted: **public contracts** (bee-core reader signatures consumed by two hooks and the status command), **changes behavior an existing test asserts** (every status-reader oracle test calls today's no-cache signatures), **the change requires replacing existing proof** (the parity suites must keep passing across a signature change), **multi-domain** (bee-core readers + queen-bee status + two hooks + the bench). Four flags → `high-risk`. Product files: `crates/bee-core/src/{decisions,cells,recovery}.rs`, `crates/queen-bee/src/status.rs`, `crates/queen-bee/src/hooks/{chain_nudge,state_sync}.rs`, plus the bench and test targets. Smaller modes are insufficient: `standard` would skip the persona panel on a change that alters signatures two live hooks depend on.

## Why this slice is the dedup and NOT the hook flip

The original Slice 3 intent was "flip the ported hooks live + dedup". Discovery refuses the first half as currently unbuildable, so this is a **SPLIT**, not a scope reduction — every locked decision stands, and the deferred half is named below with its own prerequisites.

Evidence for the split:

1. **No distribution exists.** D8 (CONTEXT.md:28) plans prebuilt per-platform binaries published as release assets and downloaded at install/onboard time. Nothing implements it: `queen-bee` appears nowhere in `packages/bee/scripts/onboard_bee.mjs` or `scripts/install.sh`, no `.bee/bin/queen-bee` exists in any repo including this one, and `.bee/onboarding.json` has no reference. A flip without distribution wires hosts to a binary they do not have.
2. **Only six of nine hooks are ported.** `crates/queen-bee/src/hooks/mod.rs` recognizes `tools-logger`, `codex-subagent-audit`, `write-guard`, `model-guard`, `chain-nudge`, `state-sync`. `bee-session-init.mjs`, `bee-prompt-context.mjs` and `bee-session-close.mjs` have no Rust side, so any flip today is partial by construction and needs a per-hook mjs-vs-binary routing shape the catalog does not have (`packages/bee/hooks/catalog.mjs:84-87` renders only `node <script>.mjs`, with no binary branch).
3. **An open feasibility question gates the platform matrix.** CONTEXT.md's Outstanding Questions still carry the Windows portability spike (D8 contingency) and the spawn-inclusive cold-exec measurement on WSL2/Windows.

Deferred, in order, each its own slice: **(a)** port the three remaining hooks; **(b)** distribution — CI build matrix, release assets, installer/onboarding download, with the Windows spike resolved first; **(c)** the flip itself — a per-hook routing shape in the catalog of record, both wiring files regenerated, and D7's invocation-string recognizer sweep (at minimum `.bee/bin/hooks/bee-write-guard.mjs:499,532,616`, whose `DISPATCHER_RE = /^bee\.mjs$/i` would silently fail open against a renamed entry point while every conformance fixture stayed green).

## Discovery (L1 — repo truth, no external research)

Duplicate reads verified against source, not taken from the decision text:

| What | Times per status run | Anchors |
|---|---|---|
| `decisions.jsonl` parsed | 4 | `bee-core/src/decisions.rs:56` (inside `build_tag_overlay`) + `:120` (inside `active_decisions`), each reached twice — from `recovery.rs:520` and from `queen-bee/src/status.rs:790` |
| `.bee/cells/*.json` directory scanned | 6 | `cells.rs:218` (`ready_cells`), `:415` (`tier_mix`), `:442` (`ceiling_scarcity_warning`, which calls `tier_mix` again), `:332` (`scribing_debt`), `:347` (`global_scribing_debt`), `recovery.rs:522` |
| transcript roots scanned | 2 | `recovery.rs:457` (inside `detect_crash_candidates`) and `:586` (inside `build_recovery_block`, after the first already returned) |

Precedent to follow, not reinvent: `reviews.rs:44`'s `GitMemo` is an explicitly per-pass memo ("never cached across passes"), and `recovery.rs:321-325`'s `SharedInputs` is the same idea scoped to one function's loop. The on-disk half of the review cache is deliberately NOT the model here — decisions, cells and transcripts change on every write, and a persisted cache would carry exactly the staleness risk the object-id-keyed cache was designed to make impossible.

Blast radius beyond status, confirmed: `cells::scribing_debt` is called from `hooks/chain_nudge.rs:133`, and `cells::list_cells` from `hooks/state_sync.rs:71`. Any signature change reaches both hooks.

## Approach

Thread a **per-invocation** shared-read structure, supplied by the caller, through the readers that repeat work — the same "caller-supplied, never re-derived inside bee-core" convention rust-port-16 and rust-port-20 already established for `control_root` and `resolved_session_id`. `build_status` loads the journal, the cell inventory and the transcript-root scan once and passes them down; `build_recovery_block` reuses what `detect_crash_candidates` already scanned instead of scanning again; `ceiling_scarcity_warning` consumes a `tier_mix` result rather than recomputing it.

**Rejected alternatives.** A process-global or lazily-initialized static cache — rejected: it makes test isolation a lie and outlives the invocation whose consistency it is supposed to guarantee. An on-disk cache like the review one — rejected: these stores change on every write, so the staleness guard that makes the review cache safe does not exist here. Reordering callers so each store is read by exactly one function — rejected: it couples unrelated blocks of the status view to each other's call order.

**Mechanism risk sits alone and first.** The proof this slice owes is not "the output still matches" (the parity legs already prove that and will keep passing whether or not the dedup works) — it is "each store is read once". Nothing in the suite counts reads today. Counting reads without polluting production code is the novel mechanism here, so it is its own cell, landed and proven against the *current* 4/6/2 counts before any dedup exists. That ordering is what makes the dedup cell's own proof a genuine red-to-green transition rather than an assertion written after the fact.

## Risk map

| Component | Risk | Proof needed |
|---|---|---|
| Read-accounting seam | **HIGH** — a counter that is not inert in production, or that counts something other than real filesystem reads, is worse than no counter | The seam is proven inert when unarmed (a negative-control test), and the armed baseline reproduces today's 4/6/2 exactly — a count that does not match the source-verified numbers means the instrument is measuring the wrong thing |
| Signature change across two live hooks | **MEDIUM** | `heavyhooks_conformance` and every status-reader oracle target stay green; the hooks' own conformance legs are re-run, not just compiled |
| Byte-parity regression | **MEDIUM** | All six parity legs stay zero-diff; the dedup must not change field order or null-vs-absent anywhere |
| Budget claim | **MEDIUM** | The bench measures the same way it does today (spawn-inclusive, ≥50 runs, size-pinned fixture, cold and warm reported); the budget tightens only if the measurement earns it, and the decision is superseded honestly if it does not |

## Test matrix (edge dimensions touched)

Concurrency (the cache is per invocation, so two invocations must not share), empty/absent stores (a missing journal or an empty cell directory must read once, not zero-then-fallback), malformed content (a corrupt record must degrade identically to today), boundary counts (the cells directory at and above the fixture floor), and failure injection (an unreadable file mid-scan must produce the same degraded block as today).

## Slice cells

Three cells, dependency-ordered, mechanism risk first:

1. **rust-port-22 — read-accounting seam + baseline.** A test-only counting seam over bee-core's store reads, inert unless armed, with a negative control proving inertness; an armed baseline test asserting today's counts are exactly 4 journal parses, 6 cell-directory scans, 2 transcript-root scans per `build_status`. No dedup in this cell.
2. **rust-port-23 — the dedup.** Thread caller-supplied shared inputs through the readers; update the two hook call sites; the same armed test now asserts 1/1/1. Parity legs and all conformance targets stay green.
3. **rust-port-24 — measure and settle the budget.** Re-run the bench, report cold and warm, and either tighten the status budget to 25 ms (superseding e119fc8b's interim 70 ms) or record honestly what was actually reached and why, with the profile.

## Open questions for validating

- Can the read-accounting seam observe real filesystem reads without a production-code branch on every read? If the only honest instrument requires a code seam in the read path, is that seam acceptable permanently, or does it need to be compiled out of release builds?
- Does any consumer depend on `ceiling_scarcity_warning` recomputing `tier_mix` (for example, expecting a fresh read after a concurrent write within one invocation)? Per-invocation consistency is a behaviour change if anything relied on the re-read.
- Is 20 ms actually reachable, or does the ~13 ms in-process floor plus process envelope land above it? The plan must not promise the budget the follow-up named if the measurement refuses it.
