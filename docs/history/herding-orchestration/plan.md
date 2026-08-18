---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Herding Orchestration

Mode: `standard` — 3 risk flags: cross-platform, multi-domain, public-contracts.

Why this is the least workflow that protects the work: one blocking repair
lands first and is proven live, then a core whose only hard part is an
ordering gets that ordering pinned by failing-first tests against a fake
backend, and only then does anything touch a real herdr or the script that
carries the permission posture.

## Requirements (from CONTEXT.md)

- **D1** — done means ONE real coordination scenario running end to end on
  Linux and Windows.
- **D2** — a generic core: workers, tasks, waiting, collection, aggregation.
  No bee concepts inside it.
- **D3** — the specification is the ORDERING of the herdr-agent-comms
  choreography and its test corpus. No code ported.
- **D4** — Windows is a required outcome, not a follow-up.
- **D5** — own crate in `packages/bee-rs`, linked into the `bee` binary; the
  core crate never depends on the `bee` crate.
- **D6** — the scenario is a spawn-and-brief wave with collection, exercising
  at least one failure path.
- **D7** — a worker-backend trait (start, status, send, read) with herdr
  first; status model is ready / working / blocked / finished /
  unverifiable, with `unverifiable` a first-class value.
- **D8** — `control-loop.sh` becomes Rust here; `bootstrap-cockpit.sh` stays
  bash as a recorded gap.
- **D9** — `std::thread` + `mpsc`. No async runtime. No tmpfile handoff layer.
- **D10** — one append-only wave ledger on the bee side; the core records
  nothing.
- **D11** — a wave is a VALUE with a failure-policy enum from day one; no
  recipe file format.
- **D12** — repair the dead spawn line first (split-then-start).
- **D13** — the Rust control loop's argv is byte-identical to the bash
  default, proven by test.

## Discovery

- `herdr agent start testslug --cwd /tmp --workspace w4 --tab w4:t1 --split right --no-focus -- claude --model sonnet`
  → `unknown option: --cwd`. A parse failure, so nothing was mutated. The
  0.8.0 signature is `agent start <NAME> --kind <KIND> --pane <ID>
  [--timeout MS]`, and its own docs say it "never creates, splits, or moves
  layout".
- `herdr --skill` prints a 195-line agent skill from the installed binary —
  the authority for CLI shape. `herdr agent prompt --wait`,
  `herdr agent wait --until`, and `herdr pane wait-output` all exist in
  0.8.0; `herdr wait agent-status`, which the source scripts call, does not.
- `.github/workflows/windows.yml:4-5` states win32 is bee's primary platform
  and runs the full unexcluded `cargo test --release` on `windows-latest`;
  `release-binaries.yml:44-51` builds `x86_64-pc-windows-msvc`.
- The `bee` crate has no async runtime, no thread-pool crate, and no seam for
  testing a command that shells out — every `git` call is a bare
  `std::process::Command`.
- `control-loop.sh` is 438 lines whose only Windows-fatal parts are GNU
  coreutils `timeout` and a bash-4.3 nameref; it uses no signals and no job
  control.
- Read as a state machine rather than as API calls, the source choreography
  is five ordered phases carrying eight properties no send/wait primitive
  pair provides (CONTEXT.md § Ordering Invariants), pinned by 29 tests
  several of which encode recorded regressions.

## Approach

**Recommended path.** Repair the spawn line and re-record its proof (D12)
before anything else, because no scenario that opens an agent can run until
it is fixed and the existing proof document actively forbids the correct
form. Then build the core crate against a fake backend only (D2, D5, D7,
D9, D11), with each of the eight ordering invariants introduced by a test
that fails first — the choreography is the whole risk, and a fake backend is
the only way to make its races deterministic. Then wire the herdr backend
and run the real spawn-and-brief wave with its ledger (D6, D10). Then
replace `control-loop.sh` (D8) behind an argv byte-equivalence test (D13).
Finally sync the knowledge area, whose Open Gaps and Pointers both move.

**Rejected alternatives.**

- Port the source's Python and bash — rejected in the distill: pinned to
  herdr 0.7.4 and calling a verb 0.8.0 removed.
- Build the core inside the `bee` crate — D5 forbids it; the crate boundary
  is the only thing enforcing D2.
- Go straight to a herdr backend and skip the fake — the eight invariants
  are races; without deterministic fault injection they can only be asserted,
  not proven.
- Replace `bootstrap-cockpit.sh` too — D8 scopes it out.
- Invent the recipe file format now — D11 defers it until a second scenario
  shows what is genuinely a parameter.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Spawn repair (D12) | LOW | Two commands; the failure is loud | A live split-then-start run, recorded in `spawn-proof.md` |
| Choreography ordering (D3) | HIGH | Every defect here is a race that passes by luck | Eight failing-first tests, one per invariant, over a fault-injecting fake backend |
| Fake-backend seam (D7) | MEDIUM | The crate has no precedent for it | The core's whole test suite runs green with no herdr on PATH |
| Windows parity (D4) | MEDIUM | Nothing new is pioneered, but nothing is proven either | The existing `windows.yml` job green with the new crate's tests included |
| Control-loop argv (D13) | HIGH | Silent drift breaks the recorded permission-posture split | A test asserting byte-identical argv against the bash default |
| Wave ledger (D10) | LOW | Append-only, shape already used in `.bee/` | A wave run leaves one readable row; occupancy reads it |

## Shape

Phase plan. Each phase is demoable in order; no phase is a technical bucket.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — Spawn repair | `role-dispatch.md` §8 and its quick reference move to split-then-start; `spawn-proof.md` is re-recorded from a live run | Nothing that opens an agent works until this lands, and the current proof forbids the correct form | An agent actually starts in a worktree pane again | Everything else |
| 2 — Core crate, fake backend | New crate in the workspace linked into `bee`; `Wave` value with the failure-policy enum; the backend trait; the five-phase state machine; `std::thread` fan-out; a fault-injecting fake backend; eight invariant tests | The ordering is the whole risk and it can only be proven deterministically | `cargo test` green on Linux and Windows with no herdr installed | Phases 3 and 4 |
| 3 — herdr backend + real wave | The herdr implementation of the trait; a `bee herding wave` verb; the wave ledger | D1 is not met until a real wave runs; the ledger is what makes it readable afterwards | A real spawn-and-brief wave over N worktrees, one failure path included, one ledger row | D1 satisfied |
| 4 — control-loop in Rust | `control-loop.sh` replaced; `bootstrap-cockpit.sh` updated to invoke it | The last Windows blocker, and the one place a silent permission drift is possible | The dispatch loop running on Windows | D4 fully satisfied |

**Slice queue and dependencies.** 1 → 2 → 3; 4 depends on 1 only and could
run beside 2/3, but is sequenced after 3 so the permission-sensitive change
lands on a base whose choreography is already proven.

**Knowledge sync is not a phase.** `docs/knowledge/areas/bee-herding/overview.md`
carries two changes — its Open Gaps (occupancy stops being counted; the
unpinned-herdr-shape gap has now actually fired) and its stale Node-era
Pointers. Both belong to the close of the phase that caused them, per the
capture discipline, not to a phase of their own. Phase 1 fixes the Pointers
it touches; phase 3 rewrites the occupancy gap; phase 4 closes the herdr-shape
gap with whatever capability check it lands.

**SMALLER PATH check — PASS.** One cheaper shape was found and taken:
knowledge sync was drafted as a fifth phase and folded into the closes above,
because AGENTS.md already makes capture part of closing a task and a phase for
it is ceremony. Two other merges were considered and rejected on evidence:
folding phase 2 into phase 3 would remove the only milestone that proves the
ordering with no herdr installed (and the fake backend is required by D7
regardless, so nothing is saved); folding phase 1 into phase 3 would leave the
blocking repair unproven and `spawn-proof.md` still forbidding the correct form.

**Current slice to prepare: Phase 1.** Later phases carry headlines only,
not cells.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges
existing coverage first and authors only the gap.

**Happy path.**
- A wave over N healthy workers dispatches to all, waits concurrently, and
  aggregates to success.
- An agent that settles on `finished` rather than `ready` is accepted
  without waiting out its timeout (invariant 7).
- A name and its pane id naming one target produce exactly one send
  (invariant 8).

**Edge cases.**
- A worker that completes before its waiter starts is still detected
  (invariant 1) — the baseline-before-dispatch case.
- A marker present in the baseline is rejected as proof of this send
  (invariant 2).
- A target that flips to working between the preflight pass and its own
  dispatch is skipped (invariant 3).
- A wave in which every SENT worker succeeded but one was dropped still
  reports failure (invariant 6).

**Error paths.**
- A status lookup that fails, returns a null field, or returns a value
  outside the enum is `unverifiable` — never safe, never a panic
  (invariant 4).
- One send failing mid-fan-out leaves earlier workers running and still
  collects them (invariant 5).
- A blocked worker whose status lookup ALSO fails does not stabilise into
  success.
- Phase 4 only: the constructed control-loop argv is byte-identical to the
  bash default when no `herding.control_command` is configured (D13).

## Out of scope

- Orchestrator succession — a wave surviving the orchestrator's own context
  running out.
- A declarative recipe file format for scenarios (D11).
- `bootstrap-cockpit.sh` rewritten in Rust (D8).
- Re-runnable eval suites for bee skills — backlog `p-cf66d519`.
- Any change to the merge gesture (R2), the dispatch interlock (R3), or the
  permission-posture split (R4).
