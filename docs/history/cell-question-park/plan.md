---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: cell-question-park

## Summary

Today one open product question stops the whole lane. With this change a
worker that has a question parks only its own cell. The other cells keep
running. The human answers with one command, and the cell goes back to work
with the answer in its brief. `bee close` and `bee worktree merge` refuse
while a question is still open, and the supervisor view lists every open
question.

Mode: `standard` — 3 risk flags: data-model, public-contracts, multi-domain
Why this is the least workflow that protects the work: a new cell state and
two new verbs are public contracts, but no hard-gate flag applies.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`b7b5b3c7`): a worker parks only its own cell; the parked cell carries
  the question, leaves the ready set, its dependents wait, siblings stay
  schedulable.
- D2 (`8ed6f4a2`): one verb answers; the answer is a decision cited by the
  cell; the cell returns to open.
- D3 (`e2e0e77e`): the leader sets `waiting_on` question only when no ready
  cell remains; parked questions show in the reply and the supervisor
  what-needs-you view.
- D4 (`2fe6f2a3`): `bee close` and `bee worktree merge` refuse a feature with
  a parked cell, naming the cell and its question.

## Load-bearing claims

Labels are `read` (file opened at that line) or `ran`; evidence is the exact
line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Only open or claimed cells enter the schedulable set, so a parked cell leaves it with no scheduler edit | read | packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs:89 | `matches!(status_of(c).as_deref(), Some("open")` |
| 2 | A dependency counts as met only when it is capped, so dependents of a parked cell wait | read | packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs:97 | `Some("capped") => "satisfied",` |
| 3 | Cell verbs are served from one match in util.rs, the place a park and an answer arm go | read | packages/bee-rs/crates/bee/src/verbs/cells/util.rs:74 | `"block" => run_block(flags, use_json, t0),` |
| 4 | The block mutation is the status-write pattern to copy | read | packages/bee-rs/crates/bee/src/verbs/cells/util.rs:380 | `cell_map.insert("status".into(), Value::String("blocked".into()));` |
| 5 | The Rust decision logger returns no id today, so the answer verb needs a variant that does | read | packages/bee-rs/crates/bee/src/verbs/cells/audit.rs:439 | `pub(crate) fn log_decision(root: &Path, decision: &str, rationale: &str, tags: &[&str]) -> MR<()> {` |
| 6 | bee close already has a per-feature cell debt door to copy | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1873 | `let dissent = crate::verbs::cells::feature_dissent_debt(root, feature)?;` |
| 7 | bee worktree merge already refuses on the same kind of cell debt before it writes | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:358 | `let dissent = crate::verbs::cells::feature_dissent_debt(main_root, feature)` |
| 8 | The supervisor what-needs-you view builds its rows in one place | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:2223 | `if let Some((kind, subject)) = live_waiting_on(control) {` |

## Discovery

A read-only gather walked the cell status sets, the scheduler, the block
verb, the decision logger, the close and merge debt doors and the
supervisor view (anchors above). Finding: no central status enum exists;
the scheduler already does the right thing for an unknown status, so D1
needs a new status value plus verbs, not a scheduler rewrite.

## Approach

Recommended path: a new status value `parked` (D1), written by a new
`bee cells park` verb and cleared by a new `bee cells answer` verb (D2), per
contract decision `16031715`. A reader of a feature's parked cells serves the
close door, the merge refusal and the supervisor rows (D3, D4), per contract
decision `3041908c`. The leader rule in D3 (set `waiting_on` only when no
ready cell remains) is doctrine and lands in the swarming skill.

Rejected: reuse `blocked` with a question field — `blocked` means a swept
claim today and the merge and close paths treat it differently; a lane-wide
mark — the stall this feature removes.

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| New status value meets old status checks (update, claim, counts) | MEDIUM | cqp-1 | unit tests: park, answer, schedule excludes a parked cell, dependents wait |
| Registry entry for two new verbs | LOW | cqp-1 | registry tests green |
| Close and merge refusal | LOW | cqp-2 | unit tests on both refusals |
| Skill text | LOW | cqp-3 | release-manifest check |

Waves: cqp-1 and cqp-3 run in parallel (disjoint files). cqp-2 waits for
cqp-1 because it reads the parked status and question that cqp-1 writes.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"test","reason":"Rerouted from code: herding agent paseo-pi-agy-flash is out of quota; decision 62df1653 (role-reroute)."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"Each cell writes its tests red-first."},
    {"stage":"documentation-and-capture","classification":"required","role":"test","reason":"cqp-3 changes skill text; rerouted from docs for the same quota reason."},
    {"stage":"read-only-gather","classification":"required","role":"review","reason":"Gathers run on --kind gather --role review while read is out of quota."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane with more than five product files: the plan-step hat wave runs."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Out of quota; gathers use review."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."},
    {"stage":"herding-code","classification":"not-applicable","role":"code","reason":"Out of quota; rerouted to test."},
    {"stage":"herding-read","classification":"not-applicable","role":"read","reason":"Out of quota; rerouted to review."},
    {"stage":"herding-docs","classification":"not-applicable","role":"docs","reason":"Out of quota; rerouted to test."}
  ]
}
```

## Shape

Phase plan (one slice):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | park and answer verbs, close and merge refusal, supervisor rows, skill text | the whole ask fits one slice | park a cell, see a sibling still claimable, answer it, see it open again; close refuses while parked | nothing later |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cqp-1 | Park a cell on a question and answer it | cells/park.rs (new), cells/util.rs, cells/audit.rs, cells/mod.rs, cells/tests.rs, generated/registry_payload.json | — | `bee cells park` sets a cell aside with its question; siblings stay claimable; `bee cells answer` logs the answer and reopens it | cells and registry tests green |
| cqp-2 | Refuse close and merge on a parked cell and list it for the human | drivers/close.rs, worktree/phases.rs, worktree/tests.rs, supervisor.rs | cqp-1 | `bee close` and `bee worktree merge` name the parked cell and its question; the supervisor view lists it | close, worktree and supervisor tests green |
| cqp-3 | Teach workers to park and leaders to keep working | skills/bee-swarming/SKILL.md, skills/bee-hive/references/gates-and-delegation.md, release manifest | — | the worker contract says park the cell, not stop the lane; the leader sets the wait mark only with no ready cell | release-manifest check green |

```json
[
  {
    "id": "cqp-1",
    "feature": "cell-question-park",
    "title": "Park a cell on a question and answer it",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["b7b5b3c7-2ca2-47ff-b2f2-ff6e2561bf53", "8ed6f4a2-7d01-4b4c-95fd-ac08f408196e", "16031715-955e-41ea-832f-7c7b1ebc64fc"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/park.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/util.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/audit.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/cell-question-park/CONTEXT.md",
      "docs/history/cell-question-park/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add two cell verbs per contract decision 16031715 (D1 b7b5b3c7, D2 8ed6f4a2). `bee cells park --id <cell> --question <text>` sets an open or claimed cell to status `parked`, writes trace.parked_question and trace.parked_at, and refuses an empty question or a cell that is capped, dropped or already parked. `bee cells answer --id <cell> --answer <text>` refuses a cell that is not parked; otherwise it logs the answer as a decision (tags cells and answer; the decision text names the cell id and quotes the question) BEFORE it writes the cell, appends the new decision id to the cell's `decisions`, moves the question into trace.park_history (question, answer decision id, time), clears trace.parked_question, and sets status `open`. A failed decision log leaves the cell unchanged. Copy the shape of `run_block` / `block_cell` in handlers_close.rs (rg anchor: `\"block\" => run_block(flags, use_json, t0),` in util.rs) and of `record_dissent_verdict` in dissent.rs for log-then-write. audit.rs `log_decision` returns no id today (`pub(crate) fn log_decision(root: &Path, decision: &str, rationale: &str, tags: &[&str]) -> MR<()> {`): add a variant that returns the id; keep the old signature for its callers. Register both verbs in the util.rs try_mutating match AND in generated/registry_payload.json (the registry tests fail if a served verb is undeclared); if the registry file is produced by a regen step, run that step. Also export a reader `feature_parked_cells(root, feature) -> Vec<(cell id, question)>` from the cells module for cqp-2. Check the existing status checks for a status they now meet: handlers_write.rs `let status_ok = matches!(cell_map.get(\"status\"), Some(Value::String(s)) if s == \"open\" || s == \"blocked\");` and `if status != \"open\" && status != \"blocked\" {` — a parked cell must not be claimable and must not be updatable into a running state; leave them unchanged unless a test shows a parked cell slips through. Tests red-first: park sets status and question; a parked cell is absent from compute_schedule's ready set while its open sibling is present; a cell that depends on a parked cell is not ready; answer logs a decision, cites it on the cell and reopens it; the refusals named above.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells registry",
    "must_haves": {
      "truths": [
        "bee cells park sets the cell status to parked and stores the question in trace.parked_question",
        "a parked cell is not in the ready set while its open sibling is",
        "a cell that depends on a parked cell is not ready",
        "bee cells answer logs a decision, appends its id to the cell decisions, and sets the cell status to open",
        "park refuses an empty question and a capped, dropped or already-parked cell; answer refuses a cell that is not parked"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/park.rs", "substantive": "park and answer verbs and the feature_parked_cells reader"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "cells.park and cells.answer declared with their params"}
      ],
      "key_links": [
        "util.rs try_mutating serves park and answer",
        "answer calls the id-returning decision logger before it writes the cell"
      ],
      "prohibitions": [
        "Do not change the meaning of status blocked",
        "Do not change compute_schedule's ready filter",
        "No code comments"
      ],
      "open_choices": ["Put the verbs in a new park.rs file instead of handlers_close.rs"]
    },
    "behavior_change": true
  },
  {
    "id": "cqp-2",
    "feature": "cell-question-park",
    "title": "Refuse close and merge on a parked cell and list it for the human",
    "lane": "standard",
    "role": "test",
    "deps": ["cqp-1"],
    "decisions": ["e2e0e77e-099b-41ef-952a-ee2e98ba951b", "2fe6f2a3-816f-4001-bc6e-2b306c12c488", "3041908c-1baa-462a-a171-12e787688c90"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/supervisor.rs"
    ],
    "read_first": [
      "docs/history/cell-question-park/CONTEXT.md",
      "docs/history/cell-question-park/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/park.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per contract decision 3041908c (D3 e2e0e77e, D4 2fe6f2a3), using the `feature_parked_cells` reader cqp-1 added in verbs/cells/park.rs. (1) bee close: add a door named `parked-cell` beside the dissent-debt door in drivers/close.rs (rg anchor: `let dissent = crate::verbs::cells::feature_dissent_debt(root, feature)?;`); it blocks while the feature has a parked cell and its message names every parked cell id and its question. Check both close paths that read dissent debt (close_handler reads it a second time) so a dry run and a real close agree. (2) bee worktree merge: in merge_stage in worktree/phases.rs, beside `let dissent = crate::verbs::cells::feature_dissent_debt(main_root, feature)`, refuse with code WORKTREE_MERGE_PARKED_CELL before any write, naming every parked cell and its question. (3) supervisor.rs collect_needs_you: after the waiting-on row (`if let Some((kind, subject)) = live_waiting_on(control) {`), add one ReportItem per parked cell across live features, text naming the cell id and the clipped question. Tests red-first: close refuses with the parked-cell door; merge refuses with WORKTREE_MERGE_PARKED_CELL and writes nothing (copy the write_dissenting_cell fixture style in worktree/tests.rs); the needs-you view lists a parked cell; after answer reopens the cell, the door and the refusal no longer fire for parking.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- close worktree supervisor",
    "must_haves": {
      "truths": [
        "bee close blocks on a door named parked-cell that names each parked cell id and question",
        "bee worktree merge refuses with WORKTREE_MERGE_PARKED_CELL before any write",
        "the supervisor what-needs-you view lists one row per parked cell with its question",
        "a feature with no parked cell closes and merges as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "substantive": "parked-cell door"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs", "substantive": "WORKTREE_MERGE_PARKED_CELL refusal"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/supervisor.rs", "substantive": "parked question rows in collect_needs_you"}
      ],
      "key_links": [
        "close.rs and phases.rs both call feature_parked_cells",
        "collect_needs_you calls feature_parked_cells"
      ],
      "prohibitions": [
        "Do not change the dissent-debt door or refusal",
        "No code comments"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "cqp-3",
    "feature": "cell-question-park",
    "title": "Teach workers to park and leaders to keep working",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["b7b5b3c7-2ca2-47ff-b2f2-ff6e2561bf53", "8ed6f4a2-7d01-4b4c-95fd-ac08f408196e", "e2e0e77e-099b-41ef-952a-ee2e98ba951b"],
    "files": [
      "skills/bee-swarming/SKILL.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/cell-question-park/CONTEXT.md",
      "docs/history/cell-question-park/plan.md",
      "skills/bee-swarming/SKILL.md"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md", "skills/bee-hive/references/gates-and-delegation.md"],
    "affects_specs": [],
    "action": "Doctrine for D1, D2 and D3. In skills/bee-swarming/SKILL.md (the worker contract, section Execute): a worker with a product question it cannot answer runs `bee cells park --id <cell> --question \"<question>\"` and returns [BLOCKED]; it never stops the lane. On the leader side: when a worker parks, the leader keeps dispatching ready cells; it sets `bee state waiting-on set --kind question` only when no ready cell remains in the lane, and lists every open parked question in its reply; the human's answer is recorded with `bee cells answer --id <cell> --answer \"<text>\"`, and the cell runs again with the answer in its brief. In skills/bee-hive/references/gates-and-delegation.md, where the waiting-on mark and questions are described, add one line that points to this rule (one home: the swarming skill). Keep each addition short. Then run the regen chain (`bee dev regen`) so rendered skill trees and the release manifest follow, and run the manifest check.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the worker contract tells a worker with a question to park its cell, not stop the lane",
        "the leader rule sets the waiting-on question mark only when no ready cell remains",
        "the answer path names bee cells answer"
      ],
      "artifacts": [
        {"path": "skills/bee-swarming/SKILL.md", "substantive": "park and answer rule"}
      ],
      "key_links": ["gates-and-delegation.md points to the swarming rule"],
      "prohibitions": ["Do not restate the rule in a second home"]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Happy | park, sibling claimable, answer, reopen | Pass when the parked cell is absent from the ready set, the sibling is present, and after answer the cell is open with the decision id in `decisions` |
| Edge | dependent of a parked cell | Pass when the dependent is not ready while its dep is parked |
| Edge | close and merge with no parked cell | Pass when both behave as on main |
| Error | park with empty question, park a capped cell, answer an open cell | Pass when each refuses and the cell file is unchanged |
| Error | close and merge with a parked cell | Pass when close blocks on door `parked-cell` and merge refuses `WORKTREE_MERGE_PARKED_CELL`, each naming the question |

## Open Questions

(none)

## Out of scope

- A per-cell attempt limit (forgent-xia item 6).
- A content check on owner questions (forgent-xia item 5).
