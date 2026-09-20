---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: worker-ledger-and-empty-fleet

## Summary

bee keeps a receipt book of which worker was sent to which cell. Rows go in and
never come out — 190 of them today, 188 still marked "running" long after their
work finished.

The receipt book is not how bee decides who is working right now; that comes
from heartbeats and claims, and it is correct. But one cleanup job trusts the
receipt book, and that job is now stuck: `bee state worker prune` protects every
cell that appears in the book, so once a cell has a receipt it can never have
its leftover files cleaned up. Finished work is protected forever.

This slice fixes the stuck cleanup. A receipt for a finished cell stops
protecting it. Two later slices retire the rows themselves and add a warning for
a batch of workers that all came back empty.

Mode: `standard` — 1 risk flag: multi-domain (cell close, state store, fleet).
Why this is the least workflow that protects the work: the fix is one predicate
in one function, but it changes what a destructive verb deletes, so it takes a
plan, a red-first test, and a recorded claim per assumption.

## Requirements

No `CONTEXT.md` — this feature came from research, not shaping. Its source of
truth is `docs/history/research/pi-dynamic-workflows-xia.md` (§ R2) and backlog
row `p-7242d305`. The source technique is `emptyFleetSummary` and
`recoverStaleRuns` in `pi-dynamic-workflows` at commit `e29dbcae`.

- R1: a receipt row for a capped cell must not protect that cell's transient
  files from `bee state worker prune`.
- R2: no currently-passing test may change its assertion to accommodate the fix.
- R3: retiring the rows themselves is a separate slice, because the write is
  control-plane and the cap that would trigger it runs in the worktree.
- R4: a wave where every worker returned nothing must be reportable, and
  distinguishable from a wave that launched nothing.

## Load-bearing claims

Labels: `read` = the file was opened at that line and the bytes copied; `ran` =
the command was run in this session and its output copied; `guessed` = neither.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The keep set protects every cell named by a worker row, unconditionally — this is the defect | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:318-324` | `for w in &workers { if !truthy(w) { continue; } match jget(w, "cell") { None \| Some(Value::Null) => {} Some(cell) => push_unique(js_disp(cell)), } }` |
| 2 | The same function already computes "is this cell capped" from the cell file, so the predicate the fix needs exists in scope | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:337-344` | `truthy(&v) && matches!(jget(&v, "status"), Some(Value::String(s)) if s == "capped")` |
| 3 | The existing keep-set test does NOT pin the defect — its worker row carries no `status` and its capped cell has no worker row, so the fix leaves it green | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1058-1074` | `r#"{"workers":[{"nickname":"w1","cell":"c-keep"},null,"junk"]}"#` … `assert!(keep.contains(&"c-keep".to_string()));` … `assert!(!keep.contains(&"capped".to_string()));` |
| 4 | Nothing reads a worker row's `status`, so changing or setting it breaks no reader | ran | `rg -n '"workers"\|\bworkers\b' packages/bee-rs/crates/bee/src --type rust -g '!**/tests.rs'` | three readers found — `handlers_close.rs:150-162` (matches nickname+cell), `hooks/chain_nudge.rs:123-128` (matches name), `state_group/workers.rs:299-350` (reads `cell`); none reads `status` |
| 5 | The cap that would naturally retire a row runs inside the granted worktree, where the ledger write is refused — so the retire cannot live at cap | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:449` | `A worker reading this from inside a granted worktree cannot self-register: bee state worker add is control-plane and refuses there, so it is the ORCHESTRATOR's move from the main checkout, not yours.` |
| 6 | `status` is a designed field with existing tests pinning `"capped"` as its post-cap value, so it is set rather than dropped | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:655-663, 682-700` | `push_worker_record(&mut workers, "w1", "c1", Some("review"), Some("capped")).unwrap();` … `assert_eq!(workers[0].get("status"), Some(&json!("capped")));` |
| 7 | `bee state worker prune` targets `.bee/workers/` transient files, not the `state.json` array | ran | `.bee/bin/bee state worker prune --dry-run` | `Would prune 0 worker transient(s) from .bee/workers/ (kept 0 still-active).` |
| 8 | The defect's scale in this repo | ran | `python3` over `.bee/state.json` | `count 190` / `Counter({'running': 188, 'done': 2})` |
| 9 | `fleet` already separates wave outcomes four ways, so slice 3 reads existing structure rather than inventing one | read | `packages/bee-rs/crates/fleet/src/choreography.rs:278-280` | `TargetOutcome::Succeeded => succeeded.push(name), TargetOutcome::TimedOut => timed_out.push(name), TargetOutcome::UnverifiableAfterSend => unverifiable_after_send.push(name),` |

## Discovery

Two reality touches changed the shape twice. The first read who consumes
`state.json workers[]` and found liveness is derived elsewhere and correct — so
the original "bee cannot tell a live from a dead worker" framing was wrong, and
is corrected in the research brief. The second read `read_prune_keep_set` and
its test, and found the real harm: the keep set trusts the receipt book
unconditionally, so a capped cell with a receipt is protected forever. Evidence
commands are in the claims table, rows 1, 3, 7 and 8.

## Approach

**Recommended**: fix the keep set's predicate in place. A worker row stops
contributing its cell to the keep set when that cell's own file says `capped` —
reusing the capped check the same function already performs eleven lines below
(claim 2). The receipt book stays untouched, so the cap guard
(`handlers_close.rs:150-162`) and the SubagentStop nudge keep working unchanged.

Rejected alternatives:

- *Retire the row at cap.* Impossible where it belongs — the cap runs in the
  worktree and the ledger write is control-plane (claim 5). Deferred to slice 2,
  where its home is an open question.
- *Delete the worker-row source from the keep set entirely.* The deletion test
  says it is nearly redundant with the cell-file pass, but not fully: a row
  whose cell file is missing would lose its protection. Keeping the source and
  adding the predicate is the smaller change.
- *Drop the `status` field.* Overturned by claim 6 — the field has a designed
  shape and two tests. Chesterton's fence: set it, later, in slice 2.

Smaller path check: is there a cheaper shape that still honours R1–R2? No — a
one-line predicate plus one red-first test is the floor, and R2 forbids editing
the existing test to get there. Evidence: claim 3 shows the existing test stays
green untouched, so no cheaper "just change the test" path exists.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| `read_prune_keep_set` predicate | MEDIUM — it gates a destructive verb; a wrong predicate deletes live files | wlf-1 | a red-first test that a capped cell WITH a worker row is not kept, plus the existing keep-set tests still green |
| Corrupt/missing cell file | LOW — the function already falls open to "keep" on unparseable | wlf-1 | the existing `corrupt` assertion in the same test stays green |

Waves: wlf-1 alone. Slices 2 and 3 are headlines, not cells.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "one Rust predicate plus its red-first test in one crate"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "the red-first keep-set test is the whole proof of this slice"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "the two reality touches are already recorded in the claims table"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no narrow fact lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers the only generation work"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked, never automatic"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "a one-predicate fix with nine recorded claims needs no second opinion"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "the knowledge area for the state store needs the keep-set rule recorded", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "single-cell slice, no swarm to supervise"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: one cell, one file, nine recorded claims"},
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

Slice 1 (current) — the stuck cleanup is unstuck.
Slice 2 (headline) — the orchestrator retires a receipt row when its cell caps,
and sets `status` to `capped`; its home is an open question (see below).
Slice 3 (headline) — a wave where every worker returned nothing is reported as
an empty fleet, distinct from a wave that launched nothing.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wlf-1 | Stop protecting a capped cell that still has a worker receipt | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs`, `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs` | — | `bee state worker prune` cleans the leftover files of finished cells instead of keeping every cell it ever saw | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml prune_` — green — the keep-set behaviour is the whole change and those tests are its full surface |

```json
[
  {
    "id": "wlf-1",
    "feature": "worker-ledger-and-empty-fleet",
    "title": "Stop protecting a capped cell that still has a worker receipt",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["602e80ae-6431-4da7-8fc3-d7a2828c9778"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first. In packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs, beside prune_keep_set_protects_non_capped_and_corrupt_cells (around line 1058), add a new test that pins the defect: a worker row naming cell c1 PLUS a .bee/cells/c1.json whose status is \"capped\" must NOT appear in read_prune_keep_set's result. Run it and watch it fail because c1 IS currently kept — that is the reported reason. Then, in read_prune_keep_set (state_group/workers.rs:299-350), change the worker-row pass at lines 318-324 so a row contributes its cell to the keep set only when that cell is not capped, reusing the same capped predicate the cell-file pass already applies at lines 337-344 (per decision 602e80ae). Factor the predicate into one helper rather than duplicating the matches! expression, so the two passes can never disagree. A missing or unparseable cell file still protects: the function falls open on unknown state, and the existing corrupt assertion must stay green. Do not touch the receipt book itself, do not add or remove a worker row, and do not change any existing assertion — the existing keep-set tests must pass untouched.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml prune_",
    "must_haves": {
      "truths": [
        "A worker row whose cell file reads capped does not keep that cell",
        "A worker row whose cell file is missing still keeps that cell",
        "A worker row whose cell file is unparseable still keeps that cell",
        "The typed refusal for a non-array state.workers still fires unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs", "substantive": "read_prune_keep_set's worker-row pass consults one shared capped predicate; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs", "substantive": "a new test pinning a capped cell that carries a worker row; existing keep-set assertions unedited"}
      ],
      "key_links": [
        "the worker-row pass and the cell-file pass call the same capped predicate"
      ],
      "prohibitions": [
        "No edit to any existing assertion in prune_keep_set_protects_non_capped_and_corrupt_cells or prune_refuses_malformed_workers",
        "No write to state.json workers[] — retiring rows is slice 2",
        "No change to registered_worker_for_cell or the SubagentStop nudge"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Test matrix

| Case | Scenario | Pass when |
|---|---|---|
| Happy path | A worker row names cell `c1`; `.bee/cells/c1.json` has `status: "capped"` | `read_prune_keep_set` does NOT contain `c1` |
| Edge — no cell file | A worker row names `c-keep`; no `c-keep.json` exists | `keep` contains `c-keep` (existing assertion, unchanged) |
| Edge — open cell | A worker row names `c2`; `c2.json` has `status: "open"` | `keep` contains `c2` |
| Edge — corrupt cell | A worker row names `c3`; `c3.json` is `{nope` | `keep` contains `c3` — unparseable falls open |
| Error path | `state.workers` is not an array | the existing typed refusal still fires: `worker prune: state.workers is not an array` |
| Behaviour change | The happy-path case run on `main` and on this branch | `main` keeps `c1`; this branch does not |

## Open Questions

- Slice 2's home: where does the orchestrator retire the row? Candidates are
  `bee worktree merge`, `bee close`, and a sweep inside `dispatch prepare`. Each
  runs in main; none has been read yet. This is a `guessed` claim and is
  deliberately parked here rather than carried into slice 1.
- Slice 3's Agent-tool path: `fleet` carries wave outcomes (claim 9), but
  Claude Agent-tool fan-out has no aggregation point, so the empty-fleet signal
  there would have to be read from cells. Unverified.

## Out of scope

- Any change to how bee decides who is live. That is already derived correctly
  from heartbeats and claims, and the research brief's first draft was wrong to
  say otherwise.
- Removing the `status` field (overturned by claim 6).
- The other six recommendations in the research brief (R1, R3–R7).
