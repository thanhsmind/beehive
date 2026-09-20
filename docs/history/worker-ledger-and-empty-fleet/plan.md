---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: worker-ledger-and-empty-fleet

Revision 2 — slice 1 has landed in main; slice 2 is the current slice.

## Summary

bee keeps a receipt book of which worker was sent to which cell. Every receipt
says `running`, and none of them ever stops saying it, however long ago the work
finished. There are 190 receipts and 188 of them are lying.

Slice 1 stopped that lie from doing damage: the file-cleanup job no longer
trusts the receipt book, so finished work can be cleaned up again. Slice 2 stops
the lie itself. When a cell is finished, its receipt is marked finished too.

Mode: `standard` — 1 risk flag: multi-domain (state store, worktree merge).
Why this is the least workflow that protects the work: the write itself is four
lines, but it lands inside `bee worktree merge` — a verb that must never turn a
green merge red — so it takes a plan, a red-first test, and a claim per
assumption.

**Honest sizing, stated up front.** No code reads a receipt's `status` today
(claim 4). Slice 2 therefore fixes no current bug. What it buys is a store that
tells the truth, which is the same property slice 1's defect came from
violating, and an input that slice 3 can read. It is hygiene with a purpose, not
a fix — and it is worth saying so rather than dressing it up.

## Requirements

No `CONTEXT.md` — this feature came from research, not shaping. Its source of
truth is `docs/history/research/pi-dynamic-workflows-xia.md` (§ R2) and backlog
row `p-7242d305`. The source technique is `recoverStaleRuns` in
`pi-dynamic-workflows` at commit `e29dbcae` (`workflow-manager.ts:545-576`),
whose rule is that a stale record is retired to a NON-failure status so its
history survives.

- R1: *(slice 1, landed)* a receipt for a capped cell must not protect that
  cell's transient files from `bee state worker prune`.
- R2: no currently-passing test may change its assertion to accommodate a fix.
- R3: a receipt whose cell is capped reads `capped`, not `running`.
- R4: the reconciliation never turns a green `bee worktree merge` red.
- R5: the receipt is marked, never deleted — the cap guard reads the row's
  existence, so removing it would break re-capping.
- R6: *(slice 3, not started)* a wave where every worker returned nothing must be
  reportable, and distinguishable from a wave that launched nothing.

## Load-bearing claims

Labels: `read` = the file was opened at that line and the bytes copied; `ran` =
the command was run in this session and its output copied; `guessed` = neither.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | *(slice 1, landed)* The keep set protected every cell named by a worker row, unconditionally | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs` (pre-slice-1) | `Some(cell) => push_unique(js_disp(cell)),` |
| 2 | Slice 1 left a shared predicate behind, and slice 2 reuses it rather than writing a second one | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:300-302` | `fn is_cell_file_capped(path: &Path) -> Result<bool, Err2> { match std::fs::read(path) { Err(_) => Ok(false), // JSON.parse throws → cell null → keep` |
| 3 | Nothing retires a receipt on a normal cap: the row is written at claim with a hard-coded `running`, and the only removal is the FAILED-claim unwind | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3333, 3723, 3845-3858, 3898` | `register_worker_for_cell(root, worker, &claimed_id, tier.as_deref())` … `fn unregister_worker_for_cell(root: &Path, nickname: &str, cell: &str)` called only from `unwind_wave_claim` |
| 4 | No consumer reads a receipt's `status`, so setting it breaks nothing and fixes no current bug | ran | `rg -n '"workers"\|\bworkers\b' packages/bee-rs/crates/bee/src --type rust -g '!**/tests.rs'` | three readers — `handlers_close.rs:150-162` (nickname+cell), `hooks/chain_nudge.rs:123-128` (name only), `state_group/workers.rs` keep set (`cell` only); none reads `status` |
| 5 | The cap guard reads the row's EXISTENCE, which is why R5 marks instead of deletes | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:157-161` | `matches!(w.get("nickname"), Some(Value::String(n)) if n == worker) && matches!(w.get("cell"), Some(Value::String(c)) if c == id)` |
| 6 | `push_worker_record` upserts `status` in place for the same `(nickname, cell)` pair, and two tests already pin `"capped"` as the value | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:655-663, 682-700` | `push_worker_record(&mut workers, "w1", "c1", Some("review"), Some("capped")).unwrap();` … `assert_eq!(workers[0].get("status"), Some(&json!("capped")));` |
| 7 | Every write to the receipt book funnels through `worker_mutate`, and only two files call it | ran | `rg -n 'worker_mutate' packages/bee-rs/crates/bee/src --type rust -g '!**/tests.rs'` | call sites only in `verbs/state_group/workers.rs:142, 169, 189, 231, 254` and `verbs/drivers/prepare.rs:3848` |
| 8 | The write is control-plane and is refused inside a granted worktree, which is why the reconcile cannot live at cap | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:449` | `bee state worker add is control-plane and refuses there, so it is the ORCHESTRATOR's move from the main checkout, not yours.` |
| 9 | `bee worktree merge` already has a post-merge block that runs in main, writes `.bee` state, commits it path-scoped, and is best-effort warn-never-block — the natural host | read | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:964-1000` | `close_the_lane_on_merge(main_root, feature, &merge_commit_sha, *uat_stop)` … `BEST-EFFORT, same warn-never-block contract as the lane write and the pre-merge bookkeeping commit: a failure here warns on its own line and never turns this green merge red.` |
| 10 | The defect's scale in this repo | ran | `python3` over `.bee/state.json` | `count 190` / `Counter({'running': 188, 'done': 2})` |
| 11 | *(slice 3)* `fleet` already separates wave outcomes four ways | read | `packages/bee-rs/crates/fleet/src/choreography.rs:278-280` | `TargetOutcome::Succeeded => succeeded.push(name), TargetOutcome::TimedOut => timed_out.push(name), TargetOutcome::UnverifiableAfterSend => unverifiable_after_send.push(name),` |

## Discovery

Slice 2's one open question — where in main the orchestrator retires a receipt —
was closed by reading, not guessing. Three candidates were named in revision 1:
`bee worktree merge`, `bee close`, and a sweep inside `dispatch prepare`.
`phases.rs:964-1000` settles it (claim 9): merge already owns a post-merge,
in-main, best-effort bookkeeping block that writes `.bee` and commits it
path-scoped, and it already carries the warn-never-block contract R4 needs. The
other two would each need that contract built from scratch.

The reads also produced the honest-sizing finding in the Summary: claim 4 says
no consumer reads `status`, so this slice buys truthfulness and a slice-3 input,
not a bug fix.

## Approach

**Recommended**: add `reconcile_capped_workers(root) -> usize` beside the keep
set in `state_group/workers.rs`. It walks the receipt book inside one
`worker_mutate` frame, and for each row whose cell reads capped — using slice
1's `is_cell_file_capped` (claim 2) — upserts that row's `status` to `capped`
through `push_worker_record` (claim 6). It returns how many rows it changed, and
changes nothing else: no row is added, no row is removed (R5, claim 5).

Call it once from the post-merge block in `worktree/phases.rs`, inside the same
best-effort arm that already writes and commits the lane file (claim 9). A
failure warns on its own line and never fails the merge (R4).

The pass is global, not feature-scoped: a row is marked when its own cell file
says capped, whatever feature that cell belongs to. That is what clears the 188
legacy rows without a migration, and it cannot mis-mark, because the cell file
is the authority for its own status.

Rejected alternatives:

- *Retire at cap.* The cap runs in the worktree; the write is control-plane and
  refused there (claim 8).
- *A new `bee state worker reconcile` verb.* It would need registration, help
  text and its own tests, and nothing would call it automatically — the ask is
  that the orchestrator does this, not that a human remembers to.
- *Delete the row instead of marking it.* Breaks the cap guard, which reads
  existence (claim 5), and throws away the record pi's `recoverStaleRuns`
  deliberately preserves.
- *Host it in `bee close`.* Correct checkout, but it runs once per feature at the
  end, so a long feature's receipts stay stale throughout; and it has no existing
  warn-never-block bookkeeping arm to borrow.

Smaller path check: is there a cheaper shape that still honours R3–R5? The
cheapest imaginable is to mark rows only for the merging feature, which skips the
`is_cell_file_capped` call per row. It is not cheaper in code and it leaves the
188 legacy rows untouched, so it fails R3 for everything but the current feature.
Evidence: claim 10 — 188 of 190 rows predate this feature.

Named deviation, recorded here rather than left silent: the approved role plan
classifies the `research` stage as not-applicable, so a gather dispatch for
slice 2's open question was refused by the door. Rather than log a role-reroute
to dispatch one, the orchestrator did the six reads itself. Decide-altitude
reading is never delegated-only, and the evidence landed in claims 2, 3, 5, 7
and 9.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| `reconcile_capped_workers` | LOW — it writes one field on rows whose cell is already terminal, and no consumer reads that field | wlf-2 | a red-first test that a `running` receipt for a capped cell becomes `capped`, and that a receipt for an open, missing or corrupt cell is left alone |
| The merge call site | MEDIUM — `bee worktree merge` must never go red on this | wlf-2 | a test that a reconcile failure leaves the merge green, driven through the same best-effort arm the lane write already uses |
| Row count drift | LOW — the pass must add and remove nothing | wlf-2 | an assertion that the array length is unchanged across a reconcile |

Waves: wlf-2 alone.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "one Rust function plus one call site and their tests, in one crate"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "the red-first reconcile tests and the merge-stays-green test are the whole proof of this slice"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "slice 2's open question was closed by the orchestrator's own reads; the evidence is in claims 2, 3, 5, 7 and 9 and the deviation is named in Approach"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no narrow fact lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers the only generation work"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked, never automatic"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "one function, one call site, eleven recorded claims"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "the knowledge area for the state store needs the receipt lifecycle recorded", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "single-cell slice, no swarm to supervise"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: one cell, eleven recorded claims"},
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

Slice 1 — **landed** in main at `67b52d5`. The file-cleanup job no longer trusts
the receipt book.
Slice 2 (current) — a receipt for a finished cell reads `capped`, reconciled by
`bee worktree merge` in the main checkout.
Slice 3 (headline) — a wave where every worker returned nothing is reported as
an empty fleet, distinct from a wave that launched nothing.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wlf-2 | Mark a receipt capped when its cell is capped, from the post-merge block | `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs`, `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs`, `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`, `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs` | — | after a merge, `.bee/state.json` stops claiming that 188 finished workers are still running | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml reconcile` — green:unit — every new test carries `reconcile` in its name, so one filter covers both the function and its merge call site |

```json
[
  {
    "id": "wlf-2",
    "feature": "worker-ledger-and-empty-fleet",
    "title": "Mark a receipt capped when its cell is capped, from the post-merge block",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["602e80ae-6431-4da7-8fc3-d7a2828c9778", "6d752b0e-9de7-4cf7-a90e-12d36ac2d0d7"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first, and name EVERY new test with `reconcile` in it so one filter covers the slice. Step 1, in packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs: add tests that pin the wanted behaviour and watch them fail. A receipt row {nickname, cell, status: \"running\"} whose .bee/cells/<cell>.json reads status \"capped\" must end up with status \"capped\"; a receipt whose cell reads \"open\", whose cell file is missing, or whose cell file is unparseable must be left exactly as it was; the array length must be unchanged; and a row that is already \"capped\" must stay \"capped\" without being counted as changed. Step 2, in state_group/workers.rs: add `pub(crate) fn reconcile_capped_workers(root: &Path) -> Result<usize, Err2>` beside read_prune_keep_set. It runs inside ONE worker_mutate frame, walks the rows, and for each row whose cell is capped per the existing is_cell_file_capped predicate (added in slice 1 — reuse it, do not write a second one) upserts that row through push_worker_record with status \"capped\", preserving the row's existing nickname, cell and tier. It returns the number of rows it actually changed. It must add no row and remove no row. Step 3, in verbs/worktree/phases.rs: call it once from the post-merge block, inside the same best-effort arm that already writes and commits the lane file around lines 964-1000. Follow that arm's existing contract exactly — a failure warns on its own line and NEVER turns a green merge red, the same way the lane write and the bookkeeping commit already behave. Do not invent a new error path and do not add a new refusal. Step 4, in verbs/worktree/tests.rs: add a test proving a merge stays green when the reconcile fails, driven the same way the existing lane-write tests drive that arm. Do not change any existing assertion, do not touch registered_worker_for_cell, and do not delete any receipt row.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml reconcile",
    "must_haves": {
      "truths": [
        "A running receipt whose cell is capped ends up capped",
        "A receipt whose cell is open, missing or unparseable is left untouched",
        "The receipt array's length is unchanged across a reconcile",
        "A failing reconcile leaves bee worktree merge green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs", "substantive": "reconcile_capped_workers, one worker_mutate frame, reusing is_cell_file_capped and push_worker_record; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs", "substantive": "one call inside the existing best-effort post-merge arm, warn-never-block"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs", "substantive": "reconcile tests covering capped, open, missing, unparseable, already-capped and array length"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "substantive": "a test that a failing reconcile keeps the merge green"}
      ],
      "key_links": [
        "reconcile_capped_workers calls the same is_cell_file_capped the keep set uses",
        "the post-merge arm calls reconcile_capped_workers and swallows its failure into a warning"
      ],
      "prohibitions": [
        "No new CLI verb and no registry change",
        "No row added and no row removed by the reconcile",
        "No change to registered_worker_for_cell, the cap guard, or the SubagentStop nudge",
        "No edit to any existing assertion",
        "No new refusal or error path in bee worktree merge"
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
| Happy path | Receipt `{w1, c1, running}`; `c1.json` reads `capped` | the row reads `capped`; the count returned is 1 |
| Edge — open cell | Receipt `{w2, c2, running}`; `c2.json` reads `open` | the row still reads `running` |
| Edge — missing cell file | Receipt `{w3, c3, running}`; no `c3.json` | the row still reads `running` |
| Edge — corrupt cell file | Receipt `{w4, c4, running}`; `c4.json` is `{nope` | the row still reads `running` |
| Edge — already capped | Receipt `{w5, c5, capped}`; `c5.json` reads `capped` | the row still reads `capped` and is not counted as changed |
| Edge — length | Any mix of the above | the array length before equals the length after |
| Error path | The reconcile fails inside the post-merge arm | the merge result is still green and carries a warning line |
| Behaviour change | The happy-path case run on `main` and on this branch | `main` leaves the row `running`; this branch marks it `capped` |

## Open Questions

- Slice 3's Agent-tool path: `fleet` carries wave outcomes (claim 11), but Claude
  Agent-tool fan-out has no aggregation point, so the empty-fleet signal there
  would have to be read from cells or from the receipts slice 2 now keeps
  honest. Unverified.
- A cell capped inline in the main checkout, with no worktree to merge, has its
  receipt reconciled only at the next merge. Accepted for this slice and recorded
  here rather than solved.

## Out of scope

- Any change to how bee decides who is live. That is already derived correctly
  from heartbeats and claims.
- Removing the `status` field, or removing a receipt row (R5, claim 5).
- A manual reconcile verb.
- The other six recommendations in the research brief (R1, R3–R7).
