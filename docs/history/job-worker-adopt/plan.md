---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: job-worker-adopt

## Summary

Four follow-ups from the agy-as-leader spike, one cell pair each for the
code fixes and one docs cell:

- F11 (D1): `bee cells finish --from-job` adopts a finished job's worker, so a
  cell whose leader died before cap can still be capped.
- F13 (D2): the `bee herding run` settle line names the cell's proof floor.
- F10 (D3): the Stop owner-ask check passes a reply whose step a guard refused
  as owner-only.
- F9 (D4): the multi-session playbook covers parallel leaders.

Each code cell is written by one dispatched implementer and checked by a
separate dispatched tester cell that adds the tests. The full suite runs once
through `bee test` before the merge.

Situation: skills/bee-planning/playbooks/feature/feature.md

architect skipped: each change is a local edit on an existing record (state
workers row, settle line, transcript scan); no new core type.
tutorial skipped: no new API for other code to call.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | cap refuses when trace.worker has no registered row for the cell | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:545 | no registered execution worker |
| 2 | finish --from-job reads the job record and checks its cell id | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1823 | job_val.get("cell_id") |
| 3 | settle_command does not read the cell proof floor | read | packages/bee-rs/crates/bee/src/herding/run.rs:6157 | fn settle_command |
| 4 | the owner-ask check is a phrase match only | read | packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs:39 | fn owner_ask_match |

## Throughput checkpoint

- Blocking first steps: none past the gate.
- Independent workstreams: jwa-1, jwa-3, jwa-5 and jwa-7 touch disjoint files and run at once.
- Shared mutable state: jwa-3 and jwa-4 share run.rs, so jwa-4 waits on jwa-3; jwa-5 and jwa-6 share owner_ask_net.rs the same way.
- Smallest safe decomposition: one implementer and one tester per fix.

## Cells — current slice (preview)

```json
[
  {
    "id": "jwa-1",
    "feature": "job-worker-adopt",
    "title": "Adopt a finished herding job's worker in cells finish --from-job",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "ac1b3903-0767-4985-82ec-665082cae61f"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply D1 in the finish --from-job path of handlers_close.rs: when registered_worker_for_cell fails for trace.worker, and the job record names the cell and its result outcome is done, register a worker row (nickname = job id) for the cell, set trace.worker to it, append an audit row of kind worker-adopt, then cap as before. A job whose result is not done keeps today's refusal.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- from_job",
    "must_haves": {
      "truths": [
        "finish --from-job caps a cell whose trace.worker has no registered row when the job for that cell finished done",
        "finish --from-job keeps the no-registered-worker refusal when the job result is not done"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
          "substantive": "worker adoption in the from-job path"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments",
        "cells cap without --from-job keeps the registered-worker check unchanged"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "jwa-2",
    "feature": "job-worker-adopt",
    "title": "Test that finish --from-job adopts a dead leader's job worker",
    "lane": "small",
    "role": "test",
    "deps": [
      "jwa-1"
    ],
    "decisions": [
      "ac1b3903-0767-4985-82ec-665082cae61f"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests in verbs/cells/tests.rs: a small-lane cell claimed with trace.worker unknown and no state worker row, a job.json naming the cell, a done result: finish --from-job caps it, state.json workers holds the job id row for the cell, and an audit row of kind worker-adopt exists. A second test: same set-up with a result outcome other than done is refused with the no-registered-worker text and the cell stays claimed.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- from_job",
    "must_haves": {
      "truths": [
        "a test drives leader-died-before-cap through finish --from-job and the cell caps",
        "a test proves a not-done job is still refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
          "substantive": "from_job adoption tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments",
        "no edit outside tests.rs unless a defect is found"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "jwa-3",
    "feature": "job-worker-adopt",
    "title": "Name the cell's proof floor in the herding run settle line",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "22979883-1118-442b-ad6c-b9669d572c34"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply D2 in herding/run.rs: where settle_command is called, load the cell, compute cell_proof_floor with the route class, and when the worker's proof result does not meet the floor (proof_meets_floor), print --proof-result <floor> and one more line that says to pass that floor only with its evidence (red-first evidence for green:red-first).",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- settle",
    "must_haves": {
      "truths": [
        "the settle line for a test-role bugfix cell with a green:unit worker proof prints --proof-result green:red-first",
        "the settle line is unchanged when the worker proof meets the floor"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "floor-aware settle line"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "jwa-4",
    "feature": "job-worker-adopt",
    "title": "Test the floor-aware settle line",
    "lane": "small",
    "role": "test",
    "deps": [
      "jwa-3"
    ],
    "decisions": [
      "22979883-1118-442b-ad6c-b9669d572c34"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests in the run.rs test module: a test-role cell under a bugfix route whose worker proof is green:unit gets a settle line naming green:red-first plus the evidence line; a code-role cell with green:unit gets the old line unchanged.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- settle",
    "must_haves": {
      "truths": [
        "a test drives the settle line with a below-floor proof and sees the floor"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "settle floor tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments",
        "no change to production code unless a defect is found"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "jwa-5",
    "feature": "job-worker-adopt",
    "title": "Let the Stop owner-ask check pass a step a guard refused as owner-only",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "b4e747ac-8eb3-47e0-bee2-bcb48c1a0056"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply D3: add the marker [owner-only: <target>] to the write-guard refusals for a governed config key, the team layer file (.bee/team.json and its global twin) and a write under the home dir outside the repo. In owner_ask_net.rs, collect the targets of [owner-only: <target>] markers from tool results in the transcript events, and return Pass when the matched reply names one of those targets. All other replies keep today's outcome.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- owner_ask_net write_guard",
    "must_haves": {
      "truths": [
        "a reply asking the owner to run a write the guard refused as owner-only ends the turn with no refusal",
        "a reply asking the owner to run a step with no owner-only refusal behind it is still refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs",
          "substantive": "owner-only pass"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments",
        "existing guard refusals keep their remedy text"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "jwa-6",
    "feature": "job-worker-adopt",
    "title": "Test the owner-only pass of the Stop owner-ask check",
    "lane": "standard",
    "role": "test",
    "deps": [
      "jwa-5"
    ],
    "decisions": [
      "b4e747ac-8eb3-47e0-bee2-bcb48c1a0056"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests in owner_ask_net.rs: transcript events holding a tool result with [owner-only: .bee/team.json] and a final reply 'bạn chạy cp .bee/team-only-agy.json .bee/team.json' pass with the door present; the same reply with no marker blocks; a marker for another target still blocks.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- owner_ask_net",
    "must_haves": {
      "truths": [
        "a test drives the session-close check with an owner-only reply and gets no refusal"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs",
          "substantive": "owner-only tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no code comments"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "jwa-7",
    "feature": "job-worker-adopt",
    "title": "Doctrine for parallel leaders and knowledge sync",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": [
      "9dc59f64-3371-4c00-93b4-087c869effb2",
      "ac1b3903-0767-4985-82ec-665082cae61f",
      "22979883-1118-442b-ad6c-b9669d572c34"
    ],
    "files": [
      "packages/bee/playbooks/multi-session.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/job-worker-adopt/plan.md",
      "docs/history/job-worker-adopt/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add D4 to packages/bee/playbooks/multi-session.md as one short paragraph in STE (check waggledance state for a run on the same PBI and steer it with --follow-up-of; otherwise dispatch the parallel leader with --no-handoff). Add to the-run-verb-and-worker-outcomes.md the D1 worker adoption of finish --from-job and the D2 floor-aware settle line, citing job-worker-adopt D1 and D2.",
    "verify": "rg -n -- --no-handoff packages/bee/playbooks/multi-session.md && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the multi-session playbook names --no-handoff and the owning-leader check",
        "the run-verb concept names worker adoption and the floor-aware settle line"
      ],
      "artifacts": [
        {
          "path": "packages/bee/playbooks/multi-session.md",
          "substantive": "parallel leader paragraph"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no rule markers changed"
      ]
    },
    "behavior_change": false
  }
]
```
