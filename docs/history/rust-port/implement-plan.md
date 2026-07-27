---
feature: rust-port
lane: high-risk
status: Approved
gate2: 2026-07-26 (auto, gate_bypass total)
sources: CONTEXT.md (D1-D11), plan.md (frozen), approach.md, cells rust-port-1..4
---

# rust-port — Implement Plan (Slice 0)

## Review Status

Gates 1–3 approved 2026-07-26 under gate_bypass `total`. Advisor consult (fable): PROCEED WITH NOTES, all 7 notes applied to cells (`reports/advisor-slice0.md`). Validation: READY WITH CONSTRAINTS (`reports/validation-slice0.md`).

## Goal / Success

Freeze the mjs mechanics (D1) and port bee to a single compiled Rust binary `queen-bee` that runs the whole CLI and all hooks in host repos (D2), with every `.bee/` storage format unchanged (D3), Rust source staying in this repo only (D4), p95 < 5 ms spawn-inclusive on hot paths (D5), a graph-based parallel test system (D6), an incremental group-by-group flip behind parity/conformance harnesses (D7), and per-platform binaries via GitHub Releases (D8). **Slice 0 (this brief)** delivers no flips — it builds the proof floor: workspace, D5 bench instrument, storage+lock core with cross-runtime conformance (D9), and the parity harness.

## Current State

- Runtime surface to reproduce: ~38,300 dedup lines (`bee.mjs` 7,259 + 35 lib modules + 11 hook files), 116 command defs / 19 group prefixes, hooks wired in `.codex/hooks.json` AND `.claude/settings.json` (reports/mjs-inventory.md).
- Measured hot-path reality (WSL2): hooks 90–160 ms end-to-end; `status` p95 486 ms (git spawnSync ~97 ms, transcript tail ~37 ms, JSON reads ~65 ms — cold start only ~15%). The 5 ms target requires work elimination (zero subprocess), not just translation (D5).
- No Rust code exists in the repo yet; mjs verify estate = 105 suites (the freeze-era parity oracle, D6).

## Scope

**In (Slice 0):** `crates/` workspace; bench instrument + host-real fixture generator; bee-core fsutil + full D9 lock/lease protocol with node↔rust interop conformance; parity harness with self-parity + red-first smoke.
**Out:** any group flip, hook port, distribution, installer changes, managed-text rewrite (later slices per plan.md epic map); pure-binary installer and host-project verify port (deferred PBIs p-81a97109, p-726a4881).

## Proposed Approach

Per approach.md: clap v4 + serde/serde_json (tried-and-true), gix for zero-subprocess git reads (Slice 2), plain GH Actions matrix + GitHub Releases for distribution, cargo-nextest + crate/module graph with declared cross-process edges for the test system, direct semantic port of the lock/lease contract. Rejected: pico-args (loses structure), libgit2 (C dep vs musl static), cargo-dist (extra moving parts), bare cargo test (no partitioning). Empirical unknowns go to validating spikes, not research.

## Technical Design

Slice 0 creates a cargo workspace `crates/` with four members: `queen-bee` (bin — only `ping`/`--version` this slice, the bench target), `bee-core` (lib — fsutil atomic-write/jsonl primitives and the D9 lock/lease implementation), a bench crate (`queen-bench`: fixture generator that refuses sub-pinned-size stores + spawn-inclusive p95 runner emitting JSON), and `bee-parity` (store-cloning dual-runtime differ with a volatile-field allowlist). Data shapes are the frozen `.bee/` formats; the serde layer must round-trip unknown fields so an mjs-written record is never narrowed by a Rust rewrite (D3). The lock module implements the concept-level contract (sharded leases under `.bee/runtime/leases/`, hash-sorted batch acquire with rollback, workspace-scoped conflict semantics, liveness-probe takeover) and is proven by conformance tests that drive the *real* `.bee/bin/lib/lock.mjs` in a node child process — the one oracle output-diffing cannot provide. Nothing in this slice touches host wiring, onboarding payloads, or any mjs file.

## Affected Files (from cells)

- `crates/*` (all four cells) — new workspace; the only product surface this slice.
- `.gitignore` (rust-port-1) — `target/`.
- Prohibited everywhere: `packages/bee/`, `.bee/bin/` (mjs frozen, D1); onboarding/vendoring manifests (D4).

## Implementation Steps (cells — reshaped by validation, 6 cells)

Wave order (zero cycles): 1 → (2, 5) → (3, 4) → 6.

1. `rust-port-1` — workspace skeleton, ALL four members scaffolded, members list final (B1 fix).
2. `rust-port-2` — queen-bench: fixture generator + p95 runner, `--budget-ms` (5 dev / 15 CI), `--self-test` covers refusal AND happy-path + bee.mjs sanity read.
5. `rust-port-5` — bee-core fsutil + unknown-field round-trip, corrupt-tail parity proven against the real mjs reader as oracle.
3. `rust-port-3` — bee-core D9 lock protocol + node↔rust interop conformance (lease-store deferred to Slice 3 by validation decision; tier: ceiling).
4. `rust-port-4` — bee-parity harness (deps: 1, 2): self-sufficient temp roots outside repo tree, resolved-root assertion, exclusion set {logs, cache, tmp}, root-path rewrite in allowlist, self-check requires inner exits 0 + fixture sanity.
6. `rust-port-6` — CI-full-run wrapper suite `scripts/tests/test_rust_workspace.mjs` (impacted-run blind spot named until D6).

## Validation Plan (evidence recorded)

Full report: `reports/validation-slice0.md`. Reality gate 5/5 PASS. Spikes both YES: cold-exec floor p95 **2.63 ms** (100 runs, WSL2, spawn-inclusive — 47% headroom under the 5 ms budget, provisional on gnu target); lock holder control deterministic (`{acquired, release}` / `{acquired: false, holder}`, body `{pid, session, ts, token}`). Panel: 5 BLOCKERs repaired at cell level (no plan.md edits); cold-pickup CRITICALs fixed; advisor notes 1–7 applied. Per-cell verify commands as listed in the cells; none run yet — execution proves them.

## Risks & Mitigation

From approach.md risk map: lock interop HIGH (Slice 0 conformance suite is the mitigation); 5 ms on host-real `status` HIGH (floor bench now, status bench Slice 2, supersession-not-shrunk-fixture rule per D5); fixture representativeness, unknown-field loss, Windows delivery, hook fail-open drift, recognizer sweep — MEDIUM, each with a named proof in the plan.

## Rollback Plan

Slice 0 is purely additive: no host wiring, no mjs edits, no onboarding changes. Rollback = revert the slice's commits (one commit per cell, cell id in message) and delete `crates/` — zero effect on any host project or on the running mjs toolchain. From Slice 1 onward every flip is reversible by pointing the wiring entry back at the mjs implementation (both wiring files, per D7); that per-slice rollback line is re-authored at each slice's brief refresh.

## Open Questions

- Cold-exec floor on WSL2 (validating spike; expected 1–3 ms, must be measured).
- gix API coverage for the `status` query set (10-minute validating probe; Slice 2 dependency).
- Deterministic node-child lock control in interop tests (validating proves the harness shape).
- Windows binary + hooks wiring feasibility (before Slice 6; D8 contingency).
