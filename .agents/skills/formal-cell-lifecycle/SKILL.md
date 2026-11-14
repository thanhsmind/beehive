---
name: formal-cell-lifecycle
description: "Model-check how bee cell claims race the cell-closing verbs (cap, block, unclaim, reopen, drop) and the expired-claim sweep, with TLA+ and TLC. Use when a change touches the claim file, the claim gate, the per-cell lock, the claim clear, the ownership check or the sweep in packages/bee-rs/crates/bee/src/verbs/cells/, or when a cell shows as claimed with no claim file."
---

# Formal check: cell claims against the closing verbs

Several sessions claim and close the same cells at once. A claim writes a
claim file first and then takes the cell lock. Every closing verb changes
the cell under the lock and then clears the claim file after it lets the
lock go. Tests do not reach the step orders in between. This check walks
every order with TLC.

## The two rules

1. **A claimed cell holds its owner's claim file.** If the store says a
   cell is `claimed` by session S, the claim file for that cell names S,
   unless a sweep has just freed S's stale claim and is parking the cell.
   Harm when broken: no claim file means the ownership check lets any
   session cap, block, unclaim or reopen S's live work, and the sweep never
   frees the cell if S dies, because the sweep only walks claim files.
   (`docs/knowledge/areas/workflow-state/claims-and-ownership.md` B23, R36.)
2. **No close of a live claim by another session.** `cap`, `block`,
   `unclaim`, `reopen` and `drop` never succeed against another session's claim that was never
   swept or let go stale. Harm when broken: one session finishes or throws
   away another session's work in progress without `--force-ownership`.
   (Same concept, B23 and R39.)

## Run it

From the repo root of the checkout or of a worktree:

```bash
bash .bee/verify/formal-cell-lifecycle/check
bash .bee/verify/formal-cell-lifecycle/check --variants
bash .bee/verify/formal-cell-lifecycle/check --trace
```

The plain run checks the model of the current code, in under a second.
`--variants` also checks two variants, about 10 seconds in total; run it
after every change to the model, to show that each rule can still pass
and still fail. `--trace` prints TLC's full counterexample.

The script needs mise, or `TLA2TOOLS` set to a `tla2tools.jar` (and
`JAVA` to a Java 11+ binary). With mise it pins TLC 1.7.4 and Java
Temurin 25, and installs them on the first run (this needs the network).
Exit 0: every expected result came true. Exit 1: a result differs from
what is expected (today the plain run exits 1 on the known finding
below). Exit 2: TLC did not finish, or a usage error.

Read a counterexample with the `statechart` skill when the host has it:
it shows each broken rule as its shortest path in plain words, and draws
the chart of states when the check passes. Use it by name; do not copy
its commands here.

## What the result means today

- **current code** fails rule 1. This is a known bug, not drift. Shortest
  path, in the code's words:
  1. Session b runs `bee cells cap` on an open cell nobody claimed. The
     ownership check passes, because no claim file exists
     (`trace.rs` `check_claim_ownership`), and b writes `capped` under the
     cell lock (`handlers_close.rs` `cap_cell_from_flags`).
  2. Session a runs `bee cells claim` and writes its claim file
     (`claims.rs` `claim_cell_file`), before it takes the cell lock
     (`handlers_write.rs` `claim_cell_cross_session_ex`).
  3. Session b, after it released the lock, clears the claim file. The
     clear removes whatever claim file is there, a's included
     (`claims.rs` `release_claim_file_best_effort`).
  4. Session c runs `bee cells reopen`; the cell is `open` again.
  5. Session a takes the cell lock, sees `open`, and writes `claimed` with
     `trace.claim_session` = a. a's claim file is gone.
  TLC can also print a sibling path: a closer's late clear lands right
  after a's claim succeeded. Both are the same bug. With the hole open,
  rule 2 also fails: a third session caps a's live cell and no ownership
  refusal fires.
- **fixed variant** passes both rules over 3,762,606 states. It changes
  one thing: the clear removes a claim file only if it is the exact claim
  (by claim number, not by owner name) that the closer saw under the
  lock. An owner-name check is not enough: the same session can claim
  again in between. Before `drop` checked ownership, this fix also needed
  the claim to re-check its own claim file under the cell lock; now it
  does not, and that re-check alone still fails. This variant is a design
  note for whoever fixes the bug; it is not code.
- **no-ownership-check variant** fails rule 2 in a short path (claim,
  then a foreign close). It proves rule 2 can catch a missing ownership
  check. Path lengths can differ by a step between runs, because TLC runs
  with several workers.

When the bug is fixed in the code, change the model's `code` behavior to
match the new code, run `--variants`, and update this section: the plain
run must then exit 0.

## Coverage and sync

Sync commit: `9df0c5ea3fb99d26bdbd5684f6cf269cc2c22606`. At this commit
the model was confirmed to describe every function below.

| Model action | Code it stands for |
|---|---|
| `ClaimFile` | `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs` `claim_cell_file` (exclusive create of `.bee/claims/<cell>.json`) |
| `ClaimCellUnderLock` | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs` `claim_cell_cross_session_ex`, the `cells:<id>` lock block; the refusal path's `release_claim` |
| `CloseCellUnderLock` | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` `cap_cell_from_flags` (lock block), `mutate_cell` as used by `block_cell`, `run_drop`, `unclaim_cell`, `reopen_cell`; ownership from `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs` `check_claim_ownership` and `guard_claim_ownership` |
| `ClearClaimFile` | `claims.rs` `release_claim_file_best_effort`, called after the lock in `mutate_cell` and in `cap_cell_from_flags` |
| `SweepClaimFile` | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs` `sweep_expired_claims`, the gated removal |
| `SweepResetCell` | `handlers_select.rs` `sweep_reset_cell` |
| `ClaimGoesStale` | the TTL in `claims.rs` `claim_expired` together with the 900-second `heartbeat_stale` |

A change touches this check when it edits any function above, adds a path
that writes or removes `.bee/claims/<cell>.json`, adds a verb that writes
a cell's `status` or `trace.claim_session`, or moves the claim clear
inside or outside the cell lock.

Find what changed since the sync commit:

```bash
git diff 9df0c5ea3fb99d26bdbd5684f6cf269cc2c22606 -- packages/bee-rs/crates/bee/src/verbs/cells/claims.rs packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs packages/bee-rs/crates/bee/src/verbs/cells/trace.rs
```

Move the sync commit forward only after you have read each change, made
the model describe the code at the new commit, and run `--variants`.

## Bounds and assumptions

- One cell, three sessions, at most four claims in one run, every session
  with a session id. Each command is one process step per lock boundary:
  everything done under the cell lock or under the claim gate is one step.
- A claim goes stale only while its owner is between commands. In the code
  the sweep needs the claim TTL to pass and a 900-second stale heartbeat;
  no command runs that long between its claim file and its lock.
- Not in the model: adoption and renewal and their fence epochs,
  reservations and holds, budgets, gates, dependencies, judge reopens,
  archive, `claim-next` picking, sessionless claims, `--force-ownership`,
  a crash between two steps, and more than one cell.
- A pass covers this model within these bounds, not the whole system.

## A failure: bug or model error?

Replay the path step by step against the code in the table above. If a
step cannot happen in the code, the model is wrong: fix the model and
say what was wrong. If every step can happen, it is a bug: name the
commit that brought it in, tell the path in the code's words, and state
the harm in one sentence. Never weaken a rule or the model to make the
check pass, and never change product code from this skill unless the
user asks.

## Changing the model

Edit `CellLifecycle.tla` from the code, not from the result: one step in
the code under one lock is one action in the model. Change a rule only
when the promise changed on purpose, and cite the evidence (the concept,
a test, a decision). Then run `--variants`, update "What the result
means today" and the coverage table, and move the sync commit.

Drift that built up across many commits, from changes that skipped this
skill, takes the `maintain-formal-verification-skill` skill.
