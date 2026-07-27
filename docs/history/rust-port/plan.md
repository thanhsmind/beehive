---
artifact_contract: bee-plan/v1
mode: high-risk
approved_gate2: 2026-07-26 (auto, gate_bypass total)
---

# rust-port — Plan (epic map)

Source of truth: `docs/history/rust-port/CONTEXT.md` (D1–D11). Inventory: `reports/mjs-inventory.md`.

## Mode gate record

Risk flags counted: external systems (CI, GitHub Releases — D8), public contracts (the whole CLI + hook surface, 116 defs/19 groups), cross-platform (5 targets, D8), covered contract must change (mjs verify estate retired group-by-group, D6), proof replaced (mjs suites → Rust suites + parity oracles, D7), multi-domain (CLI, hooks, storage/locks, distribution, test system) = **6 flags → high-risk**. Product-file count: epic-scale (new `crates/` workspace + eventual touch of every runtime entry point). Smaller lanes are insufficient by an order of magnitude: this is a multi-slice epic with hard-gate-class risk (data-store concurrency, D9).

## Discovery

Level: **L2, merged into `approach.md`** (high-risk lane graduates the approach; a separate discovery.md would duplicate it — decision 0009 fan-out). Candidates compared there: CLI framework, JSON layer, git access strategy, static-build/distribution, test-graph engine. Empirical unknowns are deliberately NOT settled by research — they are validating spikes (D5 cold-exec floor, host-real `status` budget, lock interop), per CONTEXT.md Outstanding Questions.

## Epic map (slices)

Flip units are registry group prefixes (D7); slices bundle them by risk and dependency. **Only Slice 0 gets cells now** (current slice); later slices are re-planned at their own turn with evidence from the previous one.

| Slice | Delivers | Flip? |
|---|---|---|
| **0 — Foundation & proof floor (CURRENT)** | `crates/` workspace + queen-bee skeleton; benchmark harness with host-real fixture (D5 instrument); storage core (fsutil semantics + D9 lock protocol) with cross-runtime interop conformance; parity harness with mjs-vs-mjs self-parity baseline | no flips — proofs only |
| 1 — Hot-path hooks | all 9 hook impls + adapter semantics in queen-bee (`queen-bee hook <event>`); hook conformance fixtures (D7b: exit codes, fail-open, hooks.jsonl); flip BOTH wiring files; statusline data path | hooks + statusline |
| 2 — Read spine | `status`, preamble inject, `knowledge context` read path — zero-subprocess design (git via in-process lib/cache per D5); bench gate on host-real fixture | status, knowledge (read verbs) |
| 3 — State & cells core | state (23 defs), cells (20 defs), reservations, claims — the lock-heavy write spine (D9 in anger) | state, cells, reservations |
| 4 — Ledger groups | decisions, backlog, capture, reviews, feedback, intent | 6 groups |
| 5 — Long tail | perf, worktree (spawns verify — special case), config, herding, recovery, doctor, dispatch, tmp | remaining groups |
| 6 — Distribution | CI build matrix (5 targets), GitHub Releases assets, installer binary download (D8); Windows contingency decision point | — |
| 7 — Final flip | managed invocation-string rewrite machinery (D11 deliverable), recognizer sweep (write-guard DISPATCHER_RE), mjs removal from host payload; graph-based test system done-bar (D6 zero-false-negative probe set) | everything |

## Slice 0 — current work shape

Four cells (created after Gate 2, current slice only):

1. **Workspace skeleton** — `crates/` cargo workspace: `queen-bee` bin crate + `bee-core` lib crate; release profile tuned for cold start (lto, panic=abort, stripped); builds and tests green on linux.
2. **Benchmark harness (D5 instrument)** — fixture generator producing a host-real store (decisions.jsonl ≥700 KB, reservations ≥600 KB, backlog ≥250 KB, ≥250 cells) + spawn-inclusive p95 runner (≥50 runs). Proves the floor: no-op queen-bee p95 < 5 ms on WSL2; records node baseline alongside.
3. **Storage core + D9 locks** — fsutil semantics (atomic JSON write, jsonl append) and the full D9 lock protocol in `bee-core`; conformance suite includes cross-runtime interop (node holder vs rust contender and the reverse; staleness takeover; hooks-never-wait).
4. **Parity harness** — runner cloning a fixture store, executing the same command through `bee.mjs` and (later) queen-bee, diffing stdout/exit/side-effect files; smoke-proved now via mjs-vs-mjs self-parity (zero diff) so the rig's detection power is itself tested (red-first: a seeded mutation must produce a diff).

## Test matrix sketch (12 edge dimensions, epic depth)

- **Concurrency/races:** D9 interop suite — two runtimes contending one lock; takeover under stale pid; contention.jsonl writes. (Slice 0)
- **Scale:** host-real fixture sizes pinned in D5; bench refuses smaller stores. (Slice 0)
- **Platform variance:** WSL2 now; Windows spike before Slice 6 commits (D8 contingency); musl static for linux targets.
- **Corrupt/partial input:** truncated jsonl tail, half-written cell JSON — Rust reader must match mjs tolerance semantics (fixtures in parity harness).
- **Error contracts:** hook fail-open (crash→exit 0 + log; denial→exit 2) asserted per fixture (Slice 1).
- **Idempotency/atomicity:** atomic-write crash windows (temp+rename) equivalence.
- **Time:** staleness windows (30 s / 1 h) and clock behavior in lock takeover.
- **Compat/versioning:** mjs↔rust interleaving on one store during the whole port (D3) — every parity run exercises it.
- **I/O failure:** transient-FS retry policy parity (lock protocol).
- **Config absence:** missing/legacy store files (e.g. legacy `.inject-cache.json` fallback).
- **Security/privacy:** write-guard/privacy hook behavior preserved byte-for-byte at Slice 1; no new network paths except installer download (Slice 6).
- **Resource limits:** binary size + memory ceiling recorded by bench (informational, no gate).

## Slice-boundary law

Every slice honors every locked decision it touches; nothing shrinks scope silently (SPLIT is answered at slice boundaries, where re-planning happens). A slice flips only on green parity + conformance + bench gates for its groups (D7).
