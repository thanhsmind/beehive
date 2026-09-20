---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: worker-ledger-and-empty-fleet

Revision 3 — slices 1 and 2 have landed in main; slice 3 is the current slice.

## Summary

Two records in bee were being opened and never closed. Slice 1 stopped the first
one from doing damage, slice 2 closed it. Slice 3 is the second one, and it is
the bigger of the two.

Every dispatched worker gets a row in the wave ledger, written the moment it
starts, with its outcome left blank to be filled in when it ends. **The filling
in never happens.** In the live ledger, 431 of 437 worker rows still carry a
blank outcome, and the only six that do not are failures recorded by a different
verb. The verb that bee actually dispatches through — `bee herding run` — knows
exactly how its worker ended, prints it to the caller, and never writes it back.

This matters beyond tidiness. The ledger is what answers "how many workers are
occupying a slot right now", which is how the four-slot cap is meant to be
enforced. With every row blank, that question falls back to cross-checking a
live pane list, and when no pane list is available it falls back again to a
one-hour timer the code itself calls "a strictly worse answer".

Slice 3 closes that record, and then adds the thing the closed record makes
possible: a wave where every worker came back with nothing is named as such.

Mode: `standard` — 1 risk flag: multi-domain (herding run, wave ledger).
Why this is the least workflow that protects the work: two small cells in one
file each, but one of them writes to an append-only ledger that occupancy reads,
so it takes a plan, red-first tests, and a claim per assumption.

## Requirements

No `CONTEXT.md` — this feature came from research, not shaping. Its source of
truth is `docs/history/research/pi-dynamic-workflows-xia.md` (§ R2) and backlog
row `p-7242d305`. The source techniques are `recoverStaleRuns` and
`emptyFleetSummary` in `pi-dynamic-workflows` at commit `e29dbcae`.

- R1: *(slice 1, landed)* a receipt for a capped cell must not protect that
  cell's transient files from `bee state worker prune`.
- R2: no currently-passing test may change its assertion to accommodate a fix.
- R3: *(slice 2, landed)* a receipt whose cell is capped reads `capped`.
- R4: a wave-ledger row written by `bee herding run` carries that run's outcome
  once the run ends.
- R5: the ledger is only ever appended to — no line is rewritten or deleted.
- R6: a closed wave in which no worker succeeded is reportable as an empty fleet,
  and is distinguishable from a wave that launched nothing, from a wave still
  running, and from a wave whose workers only did a dry run.
- R7: neither change may fail a run or a wave that would otherwise have passed.

## Load-bearing claims

Labels: `read` = the file was opened at that line and the bytes copied; `ran` =
the command was run in this session and its output copied; `guessed` = neither.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The ledger's own design expects a second, outcome-carrying append — this slice is finishing a design, not inventing one | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:26-31` | `A caller MAY append more than once for the same wave_id — once at dispatch time with outcome: None per worker (so occupancy is visible while the wave is still running), then again later once every outcome is known` |
| 2 | `bee herding run` writes its row with a blank outcome and never appends again | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2178-2194` | `outcome: None, evidence: None, retryable: None,` … `if let Err(e) = wave_ledger::append_wave(main_root, &row)` |
| 3 | Only `bee herding wave` fills outcomes in, and bee's dispatch door returns `herding run`, not `herding wave` | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:527` | `let outcome = classify_outcome(&result, &pane_id).map(str::to_string);` |
| 4 | The gap's real scale in this repo | ran | `python3` over `.bee/wave-ledger.jsonl` | `rows 432` / `outcomes: [(None, 431), ('resolution_failed', 4), ('unverifiable_after_send', 2)]` / `wave sizes: [(1, 429), (3, 2), (2, 1)]` |
| 5 | `bee herding run` already computes the outcome it fails to record, as a stable string | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3435-3449, 3682` | `fn outcome_label(o: &RunOutcome) -> &'static str` … `MailboxStatus::Done => "done", MailboxStatus::Blocked => "blocked"` … `m.insert("outcome".into(), Value::String(outcome_label(&result.outcome).to_string()));` |
| 6 | A blank outcome degrades occupancy to a fallback the code itself calls worse, which is why R4 is worth doing on its own | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:54-58` | `The one-hour DEFAULT_STALE_AFTER_MS timer is FALLBACK ONLY, used when the caller could not obtain a live pane list at all … It is a strictly worse answer than the pane-list cross-check` |
| 7 | The read side already folds repeated `wave_id` rows so a later row supersedes an earlier one — the append-only rule survives R4 | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:30-33, 227` | `When a wave_id appears more than once, the LATER row supersedes the earlier one; the fold happens entirely at read time (fold_waves_by_wave_id below)` |
| 8 | "Every worker reported" already has a predicate, so R6 reuses it rather than writing a second one | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:196-201` | `A wave whose workers have ALL reported an outcome is closed` … `row.workers.iter().all(\|w\| w.outcome.is_some())` |
| 9 | The ledger write is already best-effort in `herding run`, so R7's contract exists to copy | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2192-2194` | `eprintln!("bee herding run: could not append the wave ledger row: {e}");` |
| 10 | *(slice 2, landed)* the receipt reconcile is in main and its tests are green | ran | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml reconcile` | `test result: ok. 4 passed; 0 failed` |

## Discovery

Revision 2 left slice 3 as "add an empty-fleet flag" with an open question about
the Claude subagent path. Reading the real ledger answered a different and more
important question first (claim 4): there is no outcome data to judge. 431 of 437
worker rows are blank, because the one verb bee dispatches through never closes
its own row (claims 2, 3, 5).

That reorders the slice. Closing the record comes first and stands on its own
merit (claim 6); the empty-fleet verdict is what the closed record then makes
possible. Both halves turned out to be finishing a design the ledger's own
header already describes (claims 1, 7, 8) rather than adding a new one.

## Approach

**Two cells, no dependency between them, run in parallel.** Their files are
disjoint, and the verdict is testable against synthetic ledger rows, so it needs
no real outcome data to be written first.

*wlf-3 — close the row.* When `bee herding run` finishes, append a second row
for the same `wave_id` carrying `outcome: outcome_label(...)` (claim 5) and the
evidence path it already knows. Appending is the whole mechanism: the read-time
fold makes the later row win (claim 7), so R5 holds with no rewrite. The write
copies the existing best-effort contract (claim 9) — a failure warns and the run
keeps its exit code, satisfying R7.

*wlf-4 — the verdict.* A pure function over the folded waves. A wave is an empty
fleet when it is closed (claim 8), has at least one worker, and no worker's
outcome is `done`. A wave with no workers is "launched nothing", not an empty
fleet. A wave with any unreported worker is still running, not an empty fleet. A
wave whose workers are all `dry_run` did nothing real and is not an empty fleet
either — the same three exclusions pi's own `emptyFleetSummary` makes.

Rejected alternatives:

- *Rewrite the original row in place.* Breaks R5 and the ledger's stated
  must-have that the file is only ever appended to.
- *Make `herding run` call `herding wave`'s `classify_outcome`.* That function
  classifies a fleet-level `WaveResult`, which a single run does not produce;
  `outcome_label` is already the right vocabulary for this path (claim 5).
- *Wait for wlf-3 before writing wlf-4.* Unnecessary: the verdict reads rows,
  and a test can write rows directly. Serialising them would cost a round trip
  and buy nothing.
- *Surface the verdict in `bee status` or the session preamble.* Out of scope
  for this slice — the function and its tests are the deliverable, and where it
  is displayed is a separate decision with its own reader.

Smaller path check: is there a cheaper shape that still honours R4 and R6? The
cheapest imaginable is wlf-3 alone, leaving the verdict unwritten. It fails R6,
which the user asked for by name. The next cheapest is wlf-4 alone, which is
what revision 2 proposed — it fails on claim 4: a verdict over blank outcomes
can only ever answer "still running".

Named deviation, carried forward from revision 2 and still true: the approved
role plan classifies the `research` stage as not-applicable, so the dispatch door
refuses a gather. The orchestrator did these reads itself at decide-altitude;
the evidence is claims 1 through 9.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| The second append in `herding run` | MEDIUM — it writes to a ledger occupancy reads, and a wrong row could make a live worker look finished | wlf-3 | a red-first test that a finished run appends a second row whose outcome matches the envelope's, and that the first row's bytes are untouched |
| Best-effort contract | LOW — copied from the existing append at the same site | wlf-3 | a test that an append failure leaves the run's outcome and exit code unchanged |
| The verdict's exclusions | LOW — pure function, no I/O beyond the read the fold already does | wlf-4 | tests for each of the four cases: empty fleet, at least one success, no workers, still running, all dry-run |

Waves: wlf-3 and wlf-4 together, in parallel — disjoint files, no dependency.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "two small Rust changes, one file each, in one crate"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "red-first tests are the whole proof of both cells"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "the ledger reads that reordered this slice were done by the orchestrator; the evidence is claims 1 through 9 and the deviation is named in Approach"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no narrow fact lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers the only generation work"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked, never automatic"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "two small cells with ten recorded claims"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "the bee-herding knowledge area needs the closed-row rule recorded", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "two cells, no swarm to supervise"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: ten recorded claims and two small cells"},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "as above"},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "as above"},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "the smaller-path check is recorded inline in Approach"},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "as above"},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes in this slice"},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "as above"},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "as above"}
  ]
}
```

## Shape

Slice 1 — **landed** at `67b52d5`. The file cleanup no longer trusts the receipt.
Slice 2 — **landed** at `90683ab`. A receipt for a finished cell reads `capped`.
Slice 3 (current) — the wave ledger's rows get closed, and a wave where nobody
came back with anything gets named.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wlf-3 | Append the run's outcome to its wave-ledger row when the run ends | `packages/bee-rs/crates/bee/src/herding/run.rs` | — | the wave ledger stops showing every worker bee ever dispatched as still unreported | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml ledger_outcome` — green:unit — every new test carries `ledger_outcome` in its name |
| wlf-4 | Name a closed wave in which no worker succeeded an empty fleet | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs` | — | a wave where every worker came back with nothing can be told apart from one that launched nothing and one still running | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml empty_fleet` — green:unit — every new test carries `empty_fleet` in its name |

```json
[
  {
    "id": "wlf-3",
    "feature": "worker-ledger-and-empty-fleet",
    "title": "Append the run's outcome to its wave-ledger row when the run ends",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["6d752b0e-9de7-4cf7-a90e-12d36ac2d0d7"],
    "files": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first, and name EVERY new test with `ledger_outcome` in it so one filter covers the cell. Today herding/run.rs appends a wave-ledger row at dispatch time with `outcome: None` (around line 2178-2194) and never appends again, so 431 of 437 worker rows in the live ledger are still blank. The ledger's own header (wave_ledger.rs:26-31) says a caller MAY append a second time for the same wave_id once the outcome is known, and the read-time fold (fold_waves_by_wave_id) makes the later row supersede the earlier one — so this is an APPEND, never a rewrite. Step 1, add tests that fail: a run that finishes appends a SECOND wave-ledger row for the same wave_id whose worker outcome equals the same string the result envelope reports (outcome_label, run.rs:3435-3449); the first row's bytes are still present and unchanged on disk; and an append failure leaves the run's own outcome and exit code exactly as they were. Step 2, make them pass: where the run's outcome becomes known, build a WaveRow with the same wave_id, started_at and worker identity as the dispatch-time row, set the worker's outcome to outcome_label(&result.outcome), fill evidence with the report or log path the run already knows (or None when it has none), and append it through wave_ledger::append_wave. Copy the existing best-effort contract at run.rs:2192-2194 exactly — a failure prints its own warning line and NEVER changes the run's outcome or exit code. Do not rewrite or delete any ledger line, do not change outcome_label's vocabulary, do not touch herding/wave.rs, and do not change any existing assertion.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml ledger_outcome",
    "must_haves": {
      "truths": [
        "A finished run appends a second ledger row for its wave_id carrying the same outcome string the envelope reports",
        "The dispatch-time row's bytes are still on disk unchanged after the second append",
        "A failing ledger append leaves the run's outcome and exit code unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "a second append_wave call carrying outcome_label's value, under the existing best-effort contract; no TODO stubs"}
      ],
      "key_links": [
        "the appended outcome string is the same one outcome_label produces for the result envelope"
      ],
      "prohibitions": [
        "No rewrite or deletion of any ledger line",
        "No change to outcome_label's vocabulary",
        "No change to herding/wave.rs or wave_ledger.rs",
        "No new failure path in bee herding run",
        "No edit to any existing assertion"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "wlf-4",
    "feature": "worker-ledger-and-empty-fleet",
    "title": "Name a closed wave in which no worker succeeded an empty fleet",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["6d752b0e-9de7-4cf7-a90e-12d36ac2d0d7"],
    "files": ["packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first, and name EVERY new test with `empty_fleet` in it so one filter covers the cell. Add a pure verdict over the folded waves in wave_ledger.rs, beside the existing is_closed and live_worker_count. A wave is an EMPTY FLEET when all three hold: it is closed by the existing is_closed predicate (every worker reported an outcome — reuse it, do not write a second one), it has at least one worker, and no worker's outcome is \"done\". Three cases are explicitly NOT an empty fleet and each needs its own test: a wave with zero workers launched nothing; a wave with any worker still carrying a null outcome is still running; and a wave whose workers all carry \"dry_run\" did nothing real. A wave with at least one \"done\" is not an empty fleet however many others failed. Return enough for a caller to report it — the wave_id and the worker names with their outcomes — and cap the number of names returned so a huge wave cannot produce an unbounded string. This cell adds a function and its tests only: do not wire it into any command, any output or any hook, and do not change is_closed, fold_waves_by_wave_id, live_worker_count or any existing assertion.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml empty_fleet",
    "must_haves": {
      "truths": [
        "A closed wave whose workers all failed is an empty fleet",
        "A closed wave with at least one done outcome is not an empty fleet",
        "A wave with no workers is not an empty fleet",
        "A wave with any unreported worker is not an empty fleet",
        "A closed wave whose workers are all dry_run is not an empty fleet"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/wave_ledger.rs", "substantive": "an empty-fleet verdict reusing is_closed, with a cap on the number of worker names returned; no TODO stubs"}
      ],
      "key_links": [
        "the verdict calls the existing is_closed rather than re-deriving closedness"
      ],
      "prohibitions": [
        "No wiring into any command, output or hook in this cell",
        "No change to is_closed, fold_waves_by_wave_id or live_worker_count",
        "No change to herding/run.rs or herding/wave.rs",
        "No edit to any existing assertion"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

| Case | Scenario | Pass when |
|---|---|---|
| Happy path (wlf-3) | A run finishes with status Done | a second ledger row for that `wave_id` carries outcome `done` |
| Edge (wlf-3) | A run finishes Blocked | the second row carries `blocked`, matching the envelope |
| Edge (wlf-3) | The first row is already on disk | its bytes are unchanged after the second append |
| Error path (wlf-3) | The ledger append fails | the run's outcome and exit code are unchanged, and a warning line is printed |
| Happy path (wlf-4) | Closed wave, workers `blocked` and `died` | the verdict names it an empty fleet |
| Edge (wlf-4) | Closed wave, workers `done` and `died` | not an empty fleet |
| Edge (wlf-4) | Wave with zero workers | not an empty fleet |
| Edge (wlf-4) | Wave with one worker still unreported | not an empty fleet |
| Edge (wlf-4) | Closed wave, every worker `dry_run` | not an empty fleet |
| Edge (wlf-4) | A wave with more workers than the name cap | the returned names are capped |
| Behaviour change | The happy path of wlf-3 run on `main` and on this branch | `main` appends one row; this branch appends two |

## Open Questions

- Where the empty-fleet verdict is displayed. wlf-4 deliberately wires it into
  nothing, because the reader decides the surface and no reader has asked yet.
- The Claude subagent fan-out path still has no aggregation point of its own. It
  leaves no ledger row at all, so neither cell reaches it. Naming that gap is
  this slice's honest limit, not something it closes.
- A cell capped inline in the main checkout has its receipt reconciled only at
  the next merge (carried from revision 2).

## Out of scope

- Any change to how bee decides who is live.
- Removing the `status` field or any receipt row.
- Displaying the empty-fleet verdict anywhere.
- The other six recommendations in the research brief (R1, R3–R7).
