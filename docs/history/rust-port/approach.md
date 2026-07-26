# rust-port — Approach

Chosen path, rejected alternatives, and risk map for the queen-bee port. Cites CONTEXT.md D-IDs; empirical claims marked [measured] came from the exploring review passes on this machine, [ecosystem] from tried-and-true ecosystem knowledge (verify versions at implementation time), [spike] means validating must prove it.

## Candidates and choices (three-layers framing)

### CLI layer
- **clap v4** [ecosystem, tried-and-true] — chosen. Derive API maps cleanly onto the 116-def registry; parse cost is microseconds, irrelevant against the 5 ms budget; binary-size cost acceptable.
- pico-args / hand-rolled — rejected: saves compile time and a few hundred KB, loses typed subcommand structure and help generation across 19 groups; maintenance cost dominates.
- Registry authority note: the Rust command tree must be **generated or conformance-checked against `command-registry.mjs`** (the enumeration authority, per boundary) so drift is mechanical to catch — a `registry-parity` test, not a human promise.

### JSON / storage
- **serde + serde_json** [ecosystem, tried-and-true] — chosen for all `.bee/` stores (D3). 724 KB decisions.jsonl parses in low single-digit ms [ecosystem; bench confirms, Slice 0].
- simd-json — deferred: only if the Slice 0 bench shows serde_json blowing the budget on host-real stores; not worth the unsafe surface by default.
- **Key discipline:** structs must round-trip unknown fields (`#[serde(flatten)]` capture) — mjs writers may carry fields Rust doesn't model yet; dropping them on rewrite would violate D3. This is the shim-drops-unnamed-side-effect pattern from the critical patterns list applied to data.

### Git access on hot paths (D5: zero subprocess)
- **gix (gitoxide)** [ecosystem, new-and-popular but mature for read paths] — chosen for read-only queries (branch, dirty state, recent commits) that today cost ~97 ms of `spawnSync git` in `status` [measured].
- libgit2 bindings — rejected: C dependency complicates musl static builds (D8) for no capability we need.
- Subprocess + cache — fallback only: acceptable for cold paths (worktree merge already spawns the host verify by design), never for `status`/inject/statusline.
- [spike] Slice 2 bench must show gix query set on this repo < 2 ms; if not, an mtime-keyed cache layer fronts it (still zero-subprocess on the hot read).

### Static builds & distribution (D8)
- **Plain GitHub Actions matrix + `cargo build --release` per target** (linux x64/arm64 musl, macOS x64/arm64, windows x64), assets on GitHub Releases — chosen. Boring and auditable.
- cargo-dist — noted: nice packaging, but the installer (D10, stays Node) only needs predictable asset URLs; adding a packaging framework adds moving parts without removing any.
- Release profile: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` [ecosystem] — cold-start and size both benefit.

### Test system (D6)
- **cargo-nextest** for parallel execution [ecosystem] + **impacted selection from `cargo metadata`'s crate graph, refined by a module-level graph** where crates are too coarse — chosen.
- Bare `cargo test` — rejected: process-per-binary model is slower and has no partitioning story.
- Selection-correctness bar: mutation probes per crate recorded as the D6 zero-false-negative probe set; the impacted selector must catch every probe a full run catches. The mjs registry's spawn-argv edge type has no crate-graph analog — cross-process edges (queen-bee spawning verify, parity harness spawning both runtimes) get **explicit declared edges** in a small manifest, mirroring today's regex-scanned registry doctrine.

### Lock protocol (D9)
- Direct semantic port of `lock.mjs` — no design freedom here; the contract is frozen (path scheme, body, staleness, takeover identity, retry, hooks-never-wait `maxAttempts: 1`, contention log). The only Rust-side choice is proving it: the interop conformance suite (node holder vs rust contender and the reverse) is the oracle output-diff parity cannot be (D7c).
- **Contract is wider than lock.mjs alone** (bundle: `workflow-state/holds-and-the-coordination-lock.md`): sharded per-resource lease records under `.bee/runtime/leases/`, batch acquire with canonicalized hash-sorted ordering + first-collision full rollback (deadlock avoidance), `workspace_id`-scoped conflicts (same-workspace hard-deny, cross-workspace advisory), per-workflow locks never held together with the sessions lock, liveness-probe takeover (no heartbeat renewal across blocking spawns). The D9 conformance suite asserts the concept, not just the one file.

## Risk map

| Component | Risk | Proof needed |
|---|---|---|
| Lock interop (mjs and Rust contending one store mid-port) | **HIGH** | Slice 0 cross-runtime conformance suite incl. staleness takeover, two-holder negative test (D9) |
| 5 ms p95 for `status` on host-real store | **HIGH** | Slice 0 floor bench (no-op binary) + Slice 2 status bench; per D5, proven-unreachable → explicit per-command budget via supersession, never a shrunk fixture |
| Parity fixture representativeness | MEDIUM | Slice 0 self-parity baseline + seeded-mutation red-first check of the harness itself |
| Unknown-field data loss on rewrite (D3) | MEDIUM | round-trip property tests over real store samples |
| Windows delivery (D8 contingency) | MEDIUM | spike before Slice 6 commits; failure re-scopes Windows to its own PBI |
| Hook fail-open semantics drift | MEDIUM | D7b conformance fixtures assert crash→0/denial→2 explicitly |
| Recognizer sweep at final flip (write-guard `DISPATCHER_RE`) | MEDIUM | flip checklist item per D7; grep-audit cell in Slice 7 |
| clap/serde ecosystem choices | LOW | registry-parity test; bench |

## Likely files and order (Slice 0)

1. `crates/Cargo.toml` (workspace), `crates/queen-bee/` (bin), `crates/bee-core/` (lib) — skeleton first.
2. `crates/bee-core/src/fsutil.rs`, `lock.rs` + `crates/bee-core/tests/lock_interop/` (drives node as subprocess for interop).
3. `crates/bench/` or `crates/queen-bee/benches/` + fixture generator (`crates/bee-fixtures/`).
4. `crates/parity/` runner + first fixtures.

## Open questions for validating

- Cold-exec floor on WSL2 for a static musl binary (expected 1–3 ms [ecosystem]; must be [measured] before the D5 target is treated as feasible).
- exact gix API coverage for the `status` query set (Slice 2, but a 10-minute validating probe de-risks the choice now).
- interop test harness shape: can a node child reliably hold/renew a lock under test control (deterministic, no sleeps > staleness windows)?
