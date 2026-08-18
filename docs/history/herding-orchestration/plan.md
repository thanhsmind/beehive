---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Herding Orchestration

Mode: `standard` — 3 risk flags: cross-platform, multi-domain, public-contracts.

Why this is the least workflow that protects the work: one blocking repair
lands first and is proven live — including a real round trip, not just a
spawn — then a core whose only hard part is an ordering gets that ordering
pinned by failing-first tests against a fake backend, and only then does
anything touch the script that carries the permission posture.

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
- **D14** — `herding.agent_command` keeps its shape; bee splits it at spawn
  (token 0 → `--kind`, the rest after `--`). The documented default is
  unchanged; an unrecognised token 0 is a typed error naming the key.

## Discovery

- `herdr agent start testslug --cwd /tmp --workspace w4 --tab w4:t1 --split right --no-focus -- claude --model sonnet`
  → `unknown option: --cwd`. A parse failure, so nothing was mutated. The
  0.8.0 signature is `agent start <NAME> --kind <KIND> --pane <ID>
  [--timeout MS] [-- <AGENT_ARG>...]`, with `--timeout` the only other
  option — no `--cwd`, `--workspace`, `--tab`, `--split`, or `--no-focus`.
  Its own docs say it "never creates, splits, or moves layout".
- `herdr --skill` prints a 195-line agent skill from the installed binary —
  the authority for CLI shape. `herdr agent prompt --wait`,
  `herdr agent wait --until`, and `herdr pane wait-output` all exist in
  0.8.0; `herdr wait agent-status`, which the source scripts call, does not.
- `bootstrap-cockpit.sh:206` already parses `pane.pane_id` out of a split
  response, so D12's `.result.pane.pane_id` path has in-repo corroboration.
- `.github/workflows/windows.yml:4-5` states win32 is bee's primary platform
  and runs the full unexcluded `cargo test --release` on `windows-latest`;
  `release-binaries.yml:44-51` builds `x86_64-pc-windows-msvc`.
- The `bee` crate has no async runtime, no thread-pool crate, and no seam for
  testing a command that shells out — every `git` call is a bare
  `std::process::Command`.
- `control-loop.sh` is 438 lines whose only Windows-fatal parts are GNU
  coreutils `timeout` and a bash-4.3 nameref; it uses no signals and no job
  control.
- `role-dispatch.md` and `spawn-proof.md` each exist in SIX tracked copies:
  the canonical `skills/bee-herding/`, two regen targets
  (`.claude-plugin/`, `.codex-plugin/` — `devtools/skill_trees.rs:62-74`),
  and three onboarding-sync renders (`.claude/`, `.agents/`, `.opencode/` —
  `onboard/skills.rs`). None are gitignored. The renders carry per-runtime
  `bee:only` marker arms, so hand-editing any mirror is the wrong repair.
- Read as a state machine rather than as API calls, the source choreography
  is five ordered phases carrying eight properties no send/wait primitive
  pair provides (CONTEXT.md § Ordering Invariants), pinned by 29 tests
  several of which encode recorded regressions.

## Settled at review: the config contract (D14)

`operational-invariants.md:69-71` records the default as the argv-token array
`["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"]`,
and `role-dispatch.md:282-287` orders the dispatcher to pass it verbatim
after `--`. On herdr 0.8.0 the executable is named by `--kind` and everything
after `--` is handed **to** the agent, so token 0 (`claude`) would be passed
to claude as a positional argument. `operational-invariants.md:60-64` also
promises that with no `herding` keys set, "every spawned command is
BYTE-EQUIVALENT to what this skill has always run" — a promise herdr's own
change has already voided.

Owner's answer, locked as D14: the config keeps its shape and bee does the
splitting. Phase 1 therefore carries three obligations — derive `--kind` from
token 0, pass the remainder after `--`, and refuse an unrecognised token 0
with a typed error that names the key rather than a generic start failure —
plus the reworded promise in `operational-invariants.md`.

## Approach

**Recommended path.** Repair the spawn line, and prove it with a real ROUND
TRIP rather than a bare spawn (D12) — start, prompt, read — because that
round trip is also the only evidence that the D7 status model can absorb
herdr's real one before the trait shape is fixed in phase 2. Then build the
core crate against a fake backend only (D2, D5, D7, D9, D11), with each of
the eight ordering invariants introduced by a test that fails first — the
choreography is the whole risk, and a fake backend is the only way to make
its races deterministic. Then wire the herdr backend and run the real
spawn-and-brief wave with its ledger, on both platforms (D6, D10, D1). Then
replace `control-loop.sh` (D8) behind an argv byte-equivalence test (D13).

**Rejected alternatives.**

- Port the source's Python and bash — rejected in the distill: pinned to
  herdr 0.7.4 and calling a verb 0.8.0 removed.
- Build the core inside the `bee` crate — D5 forbids it; the crate boundary
  is the only thing enforcing D2.
- Go straight to a herdr backend and skip the fake — the eight invariants
  are races; without deterministic fault injection they can only be asserted,
  not proven.
- Prove the trait shape only against the fake (the first draft of this plan)
  — rejected on review: a fake whose `finished` semantics the author invents
  cannot reproduce herdr's focus-dependent `idle` vs `done`, which is
  invariant 7's exact case, so a green phase 2 would not be evidence phase 3
  works. Hence the round trip moves into phase 1.
- Replace `bootstrap-cockpit.sh` too — D8 scopes it out.
- Invent the recipe file format now — D11 defers it until a second scenario
  shows what is genuinely a parameter.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Spawn repair (D12) | MEDIUM | The herdr call fails loudly, but the rest of the repair fails silently: six tracked copies must move together, a documented config default is invalidated (D14), and §4's anomaly rule changes meaning | A live round trip recorded in `spawn-proof.md`, capturing the argv echo and the bypass-permissions status bar; a regen/sync diff showing all six copies moved |
| Choreography ordering (D3) | HIGH | Every defect here is a race that passes by luck | Eight failing-first tests, one per invariant, over a fault-injecting fake backend |
| herdr backend (D7) | HIGH | Three recorded hazards on this exact path: `agent prompt --wait` can return on the wrong turn if the agent is already working; `--lines` cannot recover rows lost to the alternate screen; `idle` vs `done` depends on UI focus, not on work. None has been run end to end by anyone | The phase-1 round trip, plus a phase-3 test per hazard: a prompt sent to an already-working agent, a response too long for the read window, and a status read without focusing |
| Crate boundary (D5) | MEDIUM | Review-by-eye is not a mechanism — one `bee` import compiles green and every behavioural test still passes | A test that parses the core crate's manifest and fails if it declares any dependency on the `bee` crate |
| Fake-backend seam (D7) | MEDIUM | The crate has no precedent for it | The core's whole test suite runs green with no herdr on PATH |
| Windows parity (D4) | MEDIUM | Nothing new is pioneered, but nothing is proven either | The existing `windows.yml` job green with the new crate's tests; the phase-3 wave re-run on Windows |
| Control-loop argv (D13) | HIGH | Silent drift breaks the recorded permission-posture split | A test asserting byte-identical argv against the bash default, plus a test that the wall-clock ceiling survives |
| Wave ledger (D10) | MEDIUM | It replaces a containment listed in `operational-invariants.md:143-145`, and its sweep/retention behaviour is deferred — a stale or orphaned row reproduces the very over-spawn failure D10 exists to remove | A wave run leaves one readable row; occupancy reads it; a stale row is proven to be swept or proven to be ignored |

## Shape

Phase plan. Each phase is demoable in order; no phase is a technical bucket.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — Spawn repair, proven by a round trip | `role-dispatch.md` §8 and its quick reference move to split-then-start, across all six tracked copies via regen/sync; `spawn-proof.md` re-recorded from a live run; `operational-invariants.md` step-number and `agent_command` mapping updated per D14 | Nothing that opens an agent works until this lands, the current proof forbids the correct form, and the round trip is the only pre-phase-2 evidence about herdr's real status semantics | An agent starts in a worktree pane, takes a prompt, and its reply is read back — with the argv echo and the bypass-permissions indicator captured | Everything else |
| 2 — Core crate, fake backend | New crate in the workspace linked into `bee`; `Wave` value with the failure-policy enum; the backend trait; the five-phase state machine; `std::thread` fan-out; a fault-injecting fake backend; eight invariant tests; the manifest-boundary test | The ordering is the whole risk and it can only be proven deterministically | `cargo test` green on Linux and Windows with no herdr installed | Phases 3 and 4 |
| 3 — herdr backend + real wave | The herdr implementation of the trait; the bee-side entry point (surface deferred — see below); the wave ledger; §4's occupancy check moves to the ledger | D1 is not met until a real wave runs on both platforms; the ledger is what makes it readable afterwards, and §4 is where it is read | A real spawn-and-brief wave over N worktrees with one failure path, one ledger row — run on Linux AND on Windows | D1 satisfied |
| 4 — control-loop in Rust | `control-loop.sh` replaced; `bootstrap-cockpit.sh:231` updated to invoke it | The last Windows blocker, and the one place a silent permission drift is possible | The dispatch loop running on Windows | D4 fully satisfied |

**Slice queue and dependencies.** 1 → 2 → 3; 4 depends on 1 only and could
run beside 2/3, but is sequenced after 3 so the permission-sensitive change
lands on a base whose choreography is already proven.

**The bee-side entry point is not named here.** An earlier draft wrote
"a `bee herding wave` verb". CONTEXT.md contains no such decision — D11 says
a scenario is reached "through a Rust API", and the delegated list covers
naming and trait shape, not a new CLI verb. Adding a verb to the `bee herding`
group is a public-contract change and is deferred to planning's phase-3
preparation, recorded as an open question rather than smuggled in as shape.

**Who owns `role-dispatch.md` §4.** CONTEXT.md leaves open whether the ledger
replaces §4's pane-count check in this feature. Answered here as sequencing,
not as product: **phase 1 does not touch §4**; phase 3 amends it when the
ledger exists to read. Phase 1 touches §8, its quick-reference row, and — per
CRITICAL 3 below — adds §8's own cleanup rule for a pane it created.

**Knowledge sync is not a phase.** `docs/knowledge/areas/bee-herding/overview.md`
carries two changes — its Open Gaps (occupancy stops being counted; the
unpinned-herdr-shape gap has now actually fired) and its stale Node-era
Pointers. Both belong to the close of the phase that caused them, per the
capture discipline, not to a phase of their own. Phase 1 fixes the Pointers
it touches; phase 3 rewrites the occupancy gap; phase 4 closes the
herdr-shape gap with whatever capability check it lands.

**SMALLER PATH check — PASS.** One cheaper shape was found and taken:
knowledge sync was drafted as a fifth phase and folded into the closes above,
because AGENTS.md already makes capture part of closing a task and a phase for
it is ceremony. Two other merges were considered and rejected on evidence:
folding phase 2 into phase 3 would remove the only milestone that proves the
ordering with no herdr installed (and the fake backend is required by D7
regardless, so nothing is saved); folding phase 1 into phase 3 would leave the
blocking repair unproven and `spawn-proof.md` still forbidding the correct form.

**Current slice to prepare: Phase 1**, once D14 is locked. Later phases carry
headlines only, not cells.

## Phase 1 — rules a cold worker cannot otherwise determine

Surfaced by the review wave; recorded here so the cells inherit them.

1. **Edit the canonical tree only.** Change `skills/bee-herding/**`, then run
   `bee dev regen` for the two plugin trees and the onboarding sync path for
   the three render trees. Never hand-edit `.claude/`, `.claude-plugin/`,
   `.codex-plugin/`, `.agents/`, or `.opencode/` — they carry per-runtime
   `bee:only` marker arms. Both the §8 body AND the quick-reference row must
   move; the old form appears twice per copy.
2. **The stray pane belongs to §8.** Under split-then-start, an `agent start`
   failure leaves an unlabelled pane whose `foreground_cwd` is the worktree —
   exactly what §4 classifies as an anomaly candidate. §8 created that pane,
   so §8 closes it on any start failure, before reporting. The confirm step
   ("exactly one new pane") is re-read against the new two-command order.
3. **Geometry tie-break survives; the empty branch dies.** Keep the existing
   largest-rect rule for choosing `right` vs `down` — it was written for
   `agent start --split` but reads identically for `pane split`. Delete the
   "No panes yet → `--split right`" branch: it is now impossible, because
   there is always a runtime root pane to split
   (`bootstrap-cockpit.sh:211-213`).
4. **The re-record captures R4's evidence.** The old proof observed the
   permission posture two ways — `result.argv` echoed back verbatim, and the
   pane status bar reading `⏵⏵ bypass permissions on`. Under `--kind` plus
   `AGENT_ARG` passthrough nobody has observed either, so the re-record MUST
   capture both. Without them the claim that R4 is untouched is unevidenced.
5. **Where the proof runs.** `bee worktree new` refuses from inside a linked
   worktree, so the proof's worktree step runs from the MAIN checkout while
   the cell lives in the feature worktree. Name the workspace, tab and pane
   ids used, and never split this session's own pane.
6. **Shell-prompt readiness is a new race.** `agent start` requires "the pane
   must be at its interactive shell prompt"; `pane split` returns as soon as
   the pane exists, and `--timeout` covers agent detection, not shell
   readiness. Record a settle-and-retry rule. This is the failure most likely
   to fire on Windows, where ConPTY starts slower.
7. **`--ratio` is kept**, matching the recorded proof's `--ratio 0.5`, unless
   the geometry rule computes otherwise.
8. **`operational-invariants.md` moves too** — `agent_command` is pinned there
   as "the tail of `herdr agent start ... --`, Dispatch role §8 **step 2**";
   after the repair it is step 3, and its content changes per D14.

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

**Structural.**
- The core crate's manifest declares no dependency on the `bee` crate (D5).
  This test is the only mechanism enforcing D2; without it the boundary is
  a convention.

**Phase 3 — the real backend.**
- The D6 failure path: a wave in which one worker is unreachable or refuses
  still collects the rest and reports a failed verdict.
- A prompt sent to an agent that is already working does not report the
  previous turn's completion as this send's.
- A response longer than the read window is still recovered.
- A status read does not depend on the pane having been focused.
- One wave leaves exactly one ledger row, and occupancy read from that row
  matches the panes actually open.

**Phase 4 — the control loop.**
- Constructed argv is byte-identical to the bash default when no
  `herding.control_command` is configured (D13).
- The wall-clock ceiling GNU `timeout` provided survives on both platforms:
  an iteration that overruns is killed, not left hanging.

## Review wave

Ran on the first draft (standard lane, 8 product files, dispatched). It
returned 4 BLOCKER, 5 WARNING, 5 CRITICAL, 3 MINOR. Applied in this
revision: the Windows leg added to phase 3's demo (BLOCKER 1); the crate
boundary given a test and a risk row (BLOCKER 2); the CLI verb removed and
deferred (BLOCKER 3); §4 ownership answered as sequencing (BLOCKER 4); the
round trip moved into phase 1 (WARNING 1); a herdr-backend risk row with its
three hazards (WARNING 2); the ledger and the spawn repair re-rated MEDIUM
(WARNING 3, 4); phase-3 and phase-4 test rows added (WARNING 5); and the
eight phase-1 rules above (CRITICAL 2-5, MINOR 1-3). Presented rather than
applied: CRITICAL 1, the `agent_command` config contract — see § Open
Decision.

## Out of scope

- Orchestrator succession — a wave surviving the orchestrator's own context
  running out.
- A declarative recipe file format for scenarios (D11).
- `bootstrap-cockpit.sh` rewritten in Rust (D8).
- Re-runnable eval suites for bee skills — backlog `p-cf66d519`.
- Any change to the merge gesture (R2), the dispatch interlock (R3), or the
  permission-posture split (R4).
