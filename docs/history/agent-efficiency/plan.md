---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Agent efficiency

## Summary

Seven cells, one per item of D1 (`c4da7108`): a one-verb feature start and
finish plus a flag-less route; herding run caps a finished cell; dispatch
prepare survives a prompt change with no hand rebuild; a goal-check miss is a
gap verdict that reopens; Prior rounds keeps reasons whole; a shorter worker
prompt; and the capture and promote debt drained. Cells 1-5 and 7 run in
parallel; cell 6 waits for cell 5 (both edit the drivers tests).

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Playbook: feature — `.claude/skills/bee-planning/playbooks/feature.md`
Deviation: the plan-step hat wave is skipped. Every cell answers a friction
the leader hit and traced this session (CONTEXT.md, Evidence); each code cell
proves itself red then green. Cells 4 and 6 defer regen to the wave close
(`regen_obligation_ack: wave-barrier`), so the leader runs bee dev regen once.

## Requirements (from CONTEXT.md)

- D1 (`c4da7108`): the seven items, one cell each; no verb approves a gate.
- D2 (`69e23234`): old orphaned scribing debt stays out.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | herding run leaves a done cell uncapped | ran | packages/bee-rs/crates/bee/src/herding/run.rs:5682 | `left its cell claimed without capping it` |
| 2 | prepare refuses on prompt skew | ran | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:140 | `prompt skew — {} does not match the` |
| 3 | the Prior rounds block lives in the cell prompt | ran | packages/bee/prompts/worker-cell.md:55 | `Prior rounds (machine-assembled from the cell record):` |
| 4 | leader-check already accepts a gap verdict | ran | packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:95 | `if verdict != "ok" && verdict != "gap" {` |
| 5 | route --set refuses a missing --flags | ran | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:596 | `None \| Some(FlagV::Present) => missing.push("--flags"),` |

## Cells (current slice)

```json
[
  {
    "id": "aef-1",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "Open and land a feature with one verb each, and route without placeholder flags",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/router.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
    ],
    "action": "`bee feature start --feature <slug> --class <c> --lane <l> [--flags <a,b>] --files <n>` leaves the same state the separate verbs leave today: a granted feature worktree, a lane record for the feature, and the recorded route, and prints the worktree path and the instruction line `bee worktree new` prints. `bee feature finish --feature <slug> --areas <a,b>` runs worktree merge, then the scribing stamp, then close, stops at the first refusal with that verb's own message, and never approves a gate or the uat door. `bee route --set` (and `bee state route --set`) accepts a route with no risk flags when --flags is omitted or empty, recording flags=0. Each step's refusal passes through unchanged. Item 1 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-1`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- feature_group route",
    "must_haves": {
      "truths": [
        "bee feature start leaves a worktree, a lane record and the route, equal to the separate verbs",
        "bee feature finish runs merge, scribing stamp and close in order and stops at the first refusal with its message",
        "bee route --set with no --flags or empty --flags records flags=0",
        "bee feature --help lists start and finish"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
          "substantive": "Open and land a feature with one verb each, and route without placeholder flags"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate"
      ],
      "open_choices": [
        "A new verbs/feature_group.rs module"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "aef-2",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "bee herding run caps a cell the worker finished but did not cap",
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md"
    ],
    "action": "When a herding worker reports outcome done for a cell with a commit and a proof line whose result is green, `bee herding run` caps the cell through the same finish door `bee cells finish --from-job` uses, and its output says it capped. A red or missing proof, no commit, or a refusal from finish leaves today's message naming the settle command, and the run exit code tells the two apart. A worker that capped itself is unchanged. Item 2 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-2`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- herding",
    "must_haves": {
      "truths": [
        "a done result with a commit and a green proof is capped by herding run itself",
        "a red or missing proof or no commit leaves the cell uncapped with the settle command named",
        "an already-capped cell is not capped twice"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "bee herding run caps a cell the worker finished but did not cap"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate",
        "Never cap on a red, missing or unparseable proof"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "aef-3",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "dispatch prepare works after a vendored prompt changes, with no hand rebuild",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee-rs/crates/bee/src/devtools/prompts.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md"
    ],
    "action": "Today `bee dispatch prepare` refuses with prompt skew when packages/bee/prompts/*.md differs from the template compiled into the binary, and the fix is a hand rebuild and copy of .bee/bin/bee. After this cell, the flow that lands a prompt change (bee dev regen, or the merge that carries it) leaves a state where dispatch prepare renders the on-disk prompt it would render after a fresh build, with no manual step; the skew refusal stays for a binary that truly cannot render the on-disk prompt. Choose the mechanism; name it in your report. Keep tests out of verbs/drivers/tests.rs, which cells aef-5 and aef-6 edit. Item 3 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-3`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt",
    "must_haves": {
      "truths": [
        "after a prompt source change lands through the normal flow, dispatch prepare succeeds with no hand rebuild",
        "the rendered prompt equals the on-disk prompt source",
        "a binary that cannot render the on-disk prompt still refuses with prompt skew"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
          "substantive": "dispatch prepare works after a vendored prompt changes, with no hand rebuild"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate",
        "Never render a prompt that differs from the on-disk source without saying so"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "aef-4",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_specs": [],
    "title": "A leader goal-check miss is a gap verdict that reopens the cell",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs",
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs"
    ],
    "action": "`bee cells leader-check --id <id> --verdict gap --file <payload>` records the gap and returns the capped cell to open for rework, with the gap answers as the reopen reason, so the next dispatch's Prior rounds shows them. A gap payload names each missing requirement; an artifact path is not required for a gap answer. The ok verdict is unchanged. The goal-check step in skills/bee-swarming/references/swarming-reference.md tells the leader to record a miss this way. bee report outcomes then counts the miss in the leader check row with no change of its own. Item 4 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-4`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- leader_check",
    "must_haves": {
      "truths": [
        "leader-check --verdict gap reopens a capped cell with the gap answers as the reason",
        "a gap answer needs no existing artifact path",
        "leader-check --verdict ok behaves as before",
        "swarming-reference.md goal-check step records a miss with leader-check --verdict gap"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
          "substantive": "A leader goal-check miss is a gap verdict that reopens the cell"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate"
      ],
      "open_choices": []
    },
    "behavior_change": true,
    "regen_obligation_ack": "wave-barrier"
  },
  {
    "id": "aef-5",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "The Prior rounds digest carries a block or reopen reason in full",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md"
    ],
    "action": "The Prior rounds block of a cell dispatch prompt today cuts a block or reopen reason after about 150 chars, so a re-dispatched worker loses the fix the leader wrote. After this cell, block and reopen reasons appear in full; other history kinds keep their current shape; the block as a whole still has a size bound, and when that bound cuts, the newest round is kept whole and the cut is marked. Item 5 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-5`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prior_rounds prepare",
    "must_haves": {
      "truths": [
        "a 600-char block reason reaches the prompt in full",
        "a 600-char reopen reason reaches the prompt in full",
        "the digest still has a total bound that keeps the newest round whole and marks a cut"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
          "substantive": "The Prior rounds digest carries a block or reopen reason in full"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "aef-6",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "code",
    "deps": [
      "aef-5"
    ],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "Trim the worker cell prompt to what the skill does not already carry",
    "files": [
      "packages/bee/prompts/worker-cell.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "action": "The cell prompt rendered by `bee dispatch prepare --kind cell` repeats rules the bee-swarming Execute section already gives the worker. After this cell the prompt is materially shorter, and a worker still meets in it, or in a skill the prompt tells it to load, every contract it must keep: identity and location, the inlined cell, the three-part brief and the better-route line, the status tokens, the Result form fields and the closed tests result set, the finish command, and the no-comment rule. Report the before and after line counts. Update the golden prompt expectations to the new text only. Item 6 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-6`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers prompts",
    "must_haves": {
      "truths": [
        "the rendered cell prompt is materially shorter than before, with line counts reported",
        "every listed contract is in the prompt or in a skill the prompt names",
        "the herding variant still holds no bee command"
      ],
      "artifacts": [
        {
          "path": "packages/bee/prompts/worker-cell.md",
          "substantive": "Trim the worker cell prompt to what the skill does not already carry"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate",
        "No text that orders a worker to find faults, doubt the lead, or dissent"
      ],
      "open_choices": []
    },
    "behavior_change": true,
    "regen_obligation_ack": "wave-barrier"
  },
  {
    "id": "aef-7",
    "feature": "agent-efficiency",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": [
      "c4da7108-e15f-492f-94f1-1b9445e5eec7"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "Flush the capture queue and settle the unapplied promote proposals",
    "files": [
      "docs/knowledge/*",
      "docs/history/*"
    ],
    "read_first": [
      "docs/history/agent-efficiency/CONTEXT.md",
      ".claude/skills/bee-capturing/SKILL.md"
    ],
    "action": "Load bee-capturing. Drain the pending capture stubs (22 at plan time) oldest first into their area concepts, and for each unapplied promote proposal (20 at plan time) either apply what belongs in docs/knowledge or record why not, then mark it complete with bee deferred-queue complete --kind promote --feature <f>. A stub or proposal that is already reflected in a concept is recorded as such, never duplicated. Report counts applied, skipped with reason, and left. Item 7 of D1. Parallel cells share this worktree: stage and commit only your own paths by name. No code comments. One commit with the trailer `cell: aef-7`.",
    "verify": ".bee/bin/bee status --json",
    "must_haves": {
      "truths": [
        "the capture queue holds no pending stub, or each one left is named with its reason",
        "every promote proposal is applied or recorded as skipped, and marked complete",
        "no concept gains a near-duplicate of an entry it already holds"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/*",
          "substantive": "Flush the capture queue and settle the unapplied promote proposals"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "No verb approves a gate",
        "Never hand-edit .bee state; use the bee CLI"
      ],
      "open_choices": []
    },
    "behavior_change": true
  }
]
```

## Proof

Each code cell runs its own filtered cargo test, red first. The leader runs
bee dev regen once at wave close, then the full declared suite before merge.
