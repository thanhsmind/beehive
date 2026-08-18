---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset>
---

# Plan: merge-closes-the-lane

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the change edits three
separate CLI contracts (`worktree merge`, `close`, `state waiting-on clear`)
whose current shapes are pinned by existing tests, so the shape needs writing
down before any cell touches them.

## Requirements

Derived from a verified live defect, not from a prior CONTEXT.md. The four
statements below are what this feature must make true.

- **R1** — A green `bee worktree merge` that actually merged something clears
  the merged feature's lane `waiting_on` mark and rewrites its `next_action`.
  The uat gate mark is *answered by the merge itself*; leaving it live makes a
  shipped feature read as "waiting on the human" forever.
- **R2** — A green, non-dry-run `bee close --feature <f>` moves that feature's
  lane to the terminal phase `idle`. `close` already retires the cells; the
  phase is the half it never wrote.
- **R3** — `bee state waiting-on clear` is a no-op, never a refusal, when the
  named feature's workflow record is already `closed` — its own help text
  already promises exactly this. Today it refuses with
  `names no live workflow (… status !== closed)`, so a stale mark on a closed
  lane has no CLI repair path at all.
- **R4** — Clearing a wait on a **lane** record nulls the pair
  `waiting_on` + `run_state` (`run_state` only when it reads
  `awaiting-approval`), the same rule decision `f9fd9d46` already fixed for
  the default `state.json` record. Today the lane path clears only half, so
  `run_state: awaiting-approval` sticks after the mark is gone.

## Discovery

Inspected `verbs/worktree/{merge,phases,handlers}.rs`, `verbs/state_group/`,
and the `close` driver. Findings:

- `bee worktree merge` performs **zero** lane-record writes. Its only durable
  self-record is a `worktree-cleanup` row appended to `.bee/deferred-queue.jsonl`.
  The one lane touch in the whole path is read-only: the uat-gate precheck
  (`phases.rs`, `read_lane_display`).
- `merge_text_lines` (`handlers.rs`) emits merge, verify, cleanup and staging
  lines — no line naming the lane, `bee close`, or any next action.
- `bee close` archives cells on a green close and auto-commits `.bee`
  bookkeeping. It writes no phase.
- Live evidence: 11 features on `main` were fully merged (`git rev-list --count
  main..wt/<f>` = 0), their cells capped or archived, yet every lane still read
  a non-terminal phase and 5 still held a live gate/question mark. Repairing
  them by hand needed `state start-feature` → `waiting-on clear` →
  `state set --phase idle` → `workflows close`, and even that left
  `run_state: awaiting-approval` behind (R4).

## Approach

**Recommended.** Split the repair across the three commands that each own one
half of the truth, and change nothing else:

| Command | New write | Why it, and not another |
|---|---|---|
| `worktree merge` | clear `waiting_on` (+ `run_state` pair), rewrite `next_action` | the merge is the event that answers the uat mark |
| `close` | set `phase: idle` on a green, non-dry-run close | `close` is already the "this feature is done" driver |
| `state waiting-on clear` | accept a closed workflow; clear the lane pair | the repair path, and its help already promises it |

**Merge does NOT set a phase.** A merge can land slice 1 of 3, so any phase
write from `merge` would lie about a feature still mid-flight. Phase is the
close driver's word alone. This is the load-bearing constraint of the shape.

**Rejected — merge sets `phase: scribing`.** Reads well for a one-slice
feature, lies for every multi-slice one. Rejected on the same constraint.

**Rejected — a new `bee lane close` verb.** A fourth command for a job two
existing commands already half-own; more surface, same outcome.

**Rejected — teach `rebuild-projections` to sweep closed-workflow lanes.**
Fixes the symptom on already-broken records without stopping new ones being
made, and rewriting closed history on every rebuild is a wider blast radius
than the three targeted writes.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| `worktree merge` lane write | MEDIUM — merge is transactional and queue-leased; a lane write must never turn a green merge red | write is best-effort and post-commit: a failure warns, never fails the merge |
| `close` phase write | LOW — green path only, terminal value only | test: green close sets `idle`; blocked close writes nothing |
| `waiting-on clear` widening | MEDIUM — an existing test pins the refusal | that test is inverted deliberately, named in the cell |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | `state waiting-on clear` accepts a closed workflow and clears the lane `waiting_on`+`run_state` pair (R3, R4) | the shared clear helper both other phases lean on | clear a mark on a closed lane; `run_state` goes null too | phases 2 and 3 call one correct helper |
| 2 | `worktree merge` clears the merged feature's mark and rewrites `next_action`; merge text names the close road (R1) | the event that strands the mark | merge a feature; its lane shows no wait and a next action naming `bee close` | a merged feature stops reading "waiting on you" |
| 3 | `bee close` green, non-dry-run sets lane `phase: idle` (R2) | the only writer entitled to a terminal phase | green close; lane reads `idle` | a shipped feature reads terminal on any dashboard |

Single slice — all three phases are the current slice.

## Test matrix

Triad per phase, at its smallest demonstrating size. Each writer judges
existing coverage first and authors only the gap.

- **Happy path** — clear on a closed workflow succeeds and nulls the pair; a
  green merge leaves `waiting_on: null` and a `bee close`-naming `next_action`;
  a green close leaves `phase: idle`.
- **Edge** — clear on a feature with no mark at all stays a no-op; a merge that
  merged nothing (already up to date) writes no lane change; a close that is
  `--dry-run` writes no phase; a close blocked by a door writes no phase.
- **Error** — clear on a genuinely unknown feature still refuses (the refusal
  narrows, it does not vanish); a lane-write failure during merge warns and
  leaves the merge green.

## Out of scope

- The 33 historical lanes with no worktree that still read a non-terminal
  phase. They predate this defect and are cleanup, not behavior.
- Capture debt on `staging-lane` and `uat-gate-before-merge` (blocked `close`
  doors). Real debt, its own work.
- Worktree retention after merge. `worktree-keep-on-merge D1` is deliberate
  and stays; `bee worktree prune` remains the drain.
