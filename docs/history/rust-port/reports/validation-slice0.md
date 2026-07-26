# rust-port — Validation Report (Slice 0)

Date: 2026-07-26 · Lane: high-risk · Verdict: **READY WITH CONSTRAINTS**

## Reality gate

| Check | Result | Evidence |
|---|---|---|
| MODE FIT | PASS | 6 risk flags (plan.md mode-gate record); epic multi-slice shape; smaller lanes dishonest by an order of magnitude |
| REPO FIT | PASS | no `crates/` exists (clean ground); cargo 1.97.1 + rustc 1.97.1 on PATH; node v24; live store sizes match D5 pins (decisions.jsonl 737,068 B; reservations 617,717 B; backlog 256,356 B; 259 cells) |
| ASSUMPTIONS | PASS | both spikes YES (below); lock.mjs exports root-parameterized (temp-root interop feasible); workspace target dir = `crates/target/` confirmed via cargo metadata |
| SMALLER PATH | PASS | freeze+incremental-flip is already the minimal honest path for a 38 kLOC runtime port; big-bang rejected by D7 |
| PROOF SURFACE | PASS | every cell verify runnable as written post-repair; slice-0 proofs are the harnesses themselves |

## Spikes (`.bee/spikes/rust-port/`)

1. **Cold-exec floor (D5)** — YES. Minimal Rust binary (tuned release profile), 100 spawn-inclusive runs on WSL2: **p50 1.76 ms / p95 2.63 ms / p99 3.33 ms / max 3.33 ms**. The 5 ms budget has 47% headroom at the floor. Constraint discovered: only `x86_64-unknown-linux-gnu` installed — floor is provisional on gnu; musl proof deferred to Slice 6 (logged decision).
2. **Lock holder control (D9 interop shape)** — YES. `lock_probe.mjs`: `acquireStoreLockOnceSync(root, name)` returns `{acquired, release}`; second acquire returns `{acquired: false, holder}`; body keys exactly `{pid, session, ts, token}`; path `<root>/.bee/locks/<name>-<hash8>.lock`. Deterministic node-child holder control confirmed; staleness controllable via backdated `ts`.

## Feasibility matrix

| Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|
| Rust toolchain on dev machine | blocking | command output | cargo/rustc 1.97.1 | PASS |
| <5 ms spawn-inclusive possible on WSL2 | blocking (D5) | runtime spike | p95 2.63 ms, n=100 | PASS (provisional gnu) |
| node child can hold/deny lock deterministically | blocking (D9 interop) | runtime probe | lock_probe.mjs output | PASS |
| temp-root store isolation possible | blocking (B5) | file inspection | `state.mjs:760-816` resolveRootsCore — root = walk-up for `.bee/onboarding.json`, no `.git`; cell 4 now pins self-sufficient outside-repo roots + resolved-root assertion | PASS |
| workspace verify paths correct | blocking | cargo metadata | target dir `crates/target/` confirmed | PASS |
| schedule sane | required | `cells schedule` | 4 waves, zero cycles: 1 → (2,5) → (3,4) → 6 | PASS |

## Plan-checker (persona panel) — findings and disposition

Verdict was CONDITIONAL with 5 BLOCKERs + 10 warnings; all BLOCKERs repaired at cell level, zero plan.md edits (plan stays frozen):

- **B1** reservation collision (`crates/*` on 2/3/4) → cell 1 scaffolds all four members with a FINAL members list; cells 2/3/4/5 narrowed to per-crate globs. Schedule now parallelizes (2,5) and (3,4).
- **B2** hidden 4→2 dependency (no fixture source) → `rust-port-4.deps = [1, 2]`; fixture explicitly from queen-bench generator.
- **B3** bench verify couldn't build queen-bee → verify now builds the workspace before running the bench.
- **B4** lease contract source missing from reads → resolved by scope decision: lease-store port deferred to Slice 3 (its consumers); Slice-0 D9 proof = lock-file protocol exactly as frozen plan.md names.
- **B5** parity harness could resolve onto the live store → cell 4 action + truths now pin self-sufficient temp roots outside the repo tree with resolved-root assertion.
- Warnings W1 (gitignore fence), W5 (budget flag), W6 (generator refusal untested), W7 (exclusion set), W8 (`node -e` guard), W9 (driver path anchor), W10 (crate names in actions) → folded into cell text. W2/W3/W4 → logged decisions (provisional gnu floor; lease deferral; rust-port-6 CI wrapper). Cross-cell CI-invisibility note → cell rust-port-6.

## Cold-pickup cell review — disposition

CRITICALs on cells 2 (crate unnamed, unbuildable verify, unverified generator), 3 (missing lease source → mooted by deferral; scope overload → split into 3+5, lease out; live-store guard; time control), 4 (root-resolution trap; self-check semantics) all fixed in the patched cells. MINORs folded (gitignore fence wording, --version truth, budget note, implement-plan in read_first).

## Constraints carried into execution

1. Perf floor is provisional on gnu; musl static is a Slice-6 obligation (D8).
2. Lease-store port is a named Slice-3 obligation — deferred, not dropped (D9 scope decision).
3. `crates/Cargo.toml` members list is final after cell 1; later cells never touch it.
4. Interop/parity work never touches the live `.bee/` store; drivers are files, never `node -e`.

## Approval block

Gates 1–2 approved (bypass total). Advisor consult: see `reports/advisor-slice0.md` (recorded via `state advisor-ref record` before the execution gate). Gate 3: auto-approved under bypass total after advisor-ref recording; audit decision logged.
