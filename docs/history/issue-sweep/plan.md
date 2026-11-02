---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Issue sweep

## Summary

Fix the 12 open bee-report GitHub issues in nine cells, one per disjoint
cluster. Each cell reproduces its issue as a failing unit test, then fixes it.
Close #145 as stale (main CI green). Leave #69 open (feature request).

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Playbook: bugfix — `.claude/skills/bee-planning/playbooks/bugfix.md`
Deviation: the plan-step hat wave is skipped. Every cell fixes a reported,
traced defect whose fix shape the report already names; red-then-green tests
are the check.

## Requirements (from CONTEXT.md)

- D1 (`4cb2e63a`): one cell per disjoint cluster; red-then-green proof per fix.

## Discovery

Each issue's error text was found in live 2.70.0 code (claims below). The
`can't be verified to stay inside the worktree` text of #147 is the host
guard, not bee; the bee-side fix is an alias verb without the refused word.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The plan-freeze guard text lives in the write guard | ran | packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:436 | `plan.md freezes once shape is locked and no longer accepts direct edits.` |
| 2 | leader-check compares by equality | ran | packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:272 | `files_changed.iter().any(\|fc\| normalize_cell_path(fc) == artifact)` |
| 3 | gate approval re-hashes plan.md | ran | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:605 | `let current_sha256 = compute_plan_sha256(&bytes);` |
| 4 | close sets lane phase idle only | ran | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3336 | `Lane phase set to \"idle\" for \"{feature}\" — close is the terminal write.` |
| 5 | merge dirty refusal cites D8a | ran | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:556 | `A bootstrapped, gitignored .bee store alone is NOT dirty, per decision D8a.` |
| 6 | plan-conflicts workflow-changed refusal | ran | packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs:523 | `so the workflow lock this call holds does not protect it` |
| 7 | the temp-index recipe lives in the write guard paths | ran | packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:383 | `GIT_INDEX_FILE=<tmp> git read-tree HEAD` |
| 8 | FIX lines name the session binding verb | ran | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:831 | `bee state session bind --lane <feature>` |

## Cells (current slice)

```json
[
  {
    "id": "swp-1",
    "feature": "issue-sweep",
    "title": "Let the plan-freeze guard allow staging an unchanged plan.md",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #149 #146 #142 (read them with `gh issue view <n>`). The Bash write guard treats `git add <path>/plan.md` (and add then commit) as an edit of a frozen plan.md. Staging does not change bytes: the guard must let a VCS add/commit of plan.md pass, in main and inside a feature worktree, while a real write (redirect, sed -i, Edit/Write tool) to a frozen plan.md stays refused. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-1`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- write_guard",
    "must_haves": {
      "truths": [
        "git add of a frozen, unchanged plan.md is allowed by the plan-freeze guard",
        "a real write to a frozen plan.md is still refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #149 #146 #142 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-2",
    "feature": "issue-sweep",
    "title": "Accept leader-check artifacts that start with a changed path plus evidence",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #143 #138 (read them with `gh issue view <n>`). recordLeaderCheck compares the whole artifact string to each trace.files_changed path (leader_check.rs:272). Accept an artifact whose leading path token (up to the first whitespace, ' (' or ' —') equals a files_changed path, so evidence text may follow the path. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-2`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- leader_check",
    "must_haves": {
      "truths": [
        "an artifact that starts with a files_changed path followed by evidence text is accepted",
        "an artifact that names no files_changed path is still refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #143 #138 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-3",
    "feature": "issue-sweep",
    "title": "Accept a bare --detached flag on worktree merge",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #137 (read them with `gh issue view <n>`). `bee worktree merge --id X --detached` is refused as an unsupported argument shape while `--detached true` works. Make the bare boolean flag parse as true, matching how other boolean flags parse. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-3`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- worktree",
    "must_haves": {
      "truths": [
        "bee worktree merge --detached with no value parses as detached=true",
        "--detached true still works"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #137 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-4",
    "feature": "issue-sweep",
    "title": "Hash the lane worktree plan.md at gate --merge approval",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #148 (read them with `gh issue view <n>`). bee gate --preview hashes the feature worktree's plan.md but approval (plan_packets.rs around lines 605 and 702) hashes the control-root copy, so approval refuses a fresh preview as stale. Resolve plan.md for approval the same way preview does (lane worktree_root when the lane has a granted worktree). Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-4`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- plan_packets",
    "must_haves": {
      "truths": [
        "gate approval hashes the same plan.md path the preview hashed",
        "a really changed plan.md still refuses as stale"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #148 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-5",
    "feature": "issue-sweep",
    "title": "Mark the workflow record closed when bee close finishes a lane",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #140 (read them with `gh issue view <n>`). A green bee close on a lane feature sets the lane phase to idle (close.rs:3336) but leaves the workflow record status active. Close the workflow record as part of the terminal write, reusing close_workflow_records in verbs/state_group/workflows.rs. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-5`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- close",
    "must_haves": {
      "truths": [
        "after a green bee close on a lane, workflows list reports that feature with status closed",
        "close on the default record keeps its present behavior"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #140 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-6",
    "feature": "issue-sweep",
    "title": "Refresh the shared index after the temp-index commit recipe",
    "lane": "standard",
    "role": "code",
    "deps": [
      "swp-1"
    ],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #144 (read them with `gh issue view <n>`). The concurrent-worker git guard prints a GIT_INDEX_FILE temp-index recipe (paths.rs) that advances HEAD without updating the shared index, so the next shared-index commit reverts sibling commits. Make the recipe end by refreshing the shared index for the committed paths, and let the guard allow that path-scoped refresh while workers are live. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-6`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- write_guard",
    "must_haves": {
      "truths": [
        "the printed temp-index recipe refreshes the shared index for the committed paths",
        "the guard allows the path-scoped index refresh while workers are live"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #144 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-7",
    "feature": "issue-sweep",
    "title": "Do not read bee-written cell snapshots as a dirty worktree at merge",
    "lane": "standard",
    "role": "code",
    "deps": [
      "swp-3"
    ],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #141 (read them with `gh issue view <n>`). bee cells add from main mirrors .bee/cells/<id>.json into a granted feature worktree where they are untracked, so worktree merge refuses WORKTREE_MERGE_WORKTREE_DIRTY (phases.rs:556) and no CLI verb removes them. Treat bee-written .bee/ store files as part of the bootstrapped store under decision D8a in the dirty check. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-7`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- worktree",
    "must_haves": {
      "truths": [
        "untracked .bee/cells snapshots alone do not make worktree merge refuse as dirty",
        "a real untracked product file still makes it refuse"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #141 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-8",
    "feature": "issue-sweep",
    "title": "Resolve the implicit lane the same way for plan-conflicts derive and its lock",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #139 (read them with `gh issue view <n>`). plan-conflicts derive without --lane from a session bound to a lane resolves the workflow id from a stale binding while the lock resolves the live lane record, so it refuses 'workflow changed' on every retry (plan_conflicts.rs:523). Resolve both from the same live lane record. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-8`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- plan_conflicts",
    "must_haves": {
      "truths": [
        "plan-conflicts derive with an implicit lane succeeds when the binding names a live lane",
        "a real workflow change during the call still refuses"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #139 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "swp-9",
    "feature": "issue-sweep",
    "title": "Add an attach alias for state session bind and print it in FIX lines",
    "lane": "standard",
    "role": "code",
    "deps": [
      "swp-3",
      "swp-8"
    ],
    "decisions": [
      "4cb2e63a-7396-4a5d-9bec-dd2a4386a642"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/router.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"
    ],
    "read_first": [
      "docs/history/issue-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix GitHub issue(s) #147 (read them with `gh issue view <n>`). The host worktree guard refuses any shell text holding the word for the session binding verb. Add `bee state session attach` as an alias of that verb (same flags, same effect), list it in the catalog, and make every FIX line bee prints name `attach`. Red first: add a unit test that reproduces the reported behavior and watch it fail, then fix, then run verify. No code comments. One commit with the trailer `cell: swp-9`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- sessions",
    "must_haves": {
      "truths": [
        "bee state session attach --lane X --session-id Y does what the old binding verb does",
        "FIX lines name state session attach"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs",
          "substantive": "the fix"
        }
      ],
      "key_links": [
        "issue #147 reproduces as a failing test before the fix"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "Do not touch files outside this cell"
      ]
    },
    "behavior_change": true
  }
]
```

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "Each cell is a code fix in bee-rs."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "No release rides this lane."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The plan is this page."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "not-applicable",
      "role": "test",
      "reason": "Each code cell carries its own red-then-green unit test."
    },
    {
      "stage": "documentation-and-capture",
      "classification": "not-applicable",
      "role": "docs",
      "reason": "Capture runs at close by the leader."
    },
    {
      "stage": "read-only-gather",
      "classification": "not-applicable",
      "role": "read",
      "reason": "The leader traced each issue already."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No narrow lookup is needed."
    },
    {
      "stage": "generation-fallback",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Every job has a specific role."
    },
    {
      "stage": "independent-review",
      "classification": "not-applicable",
      "role": "review",
      "reason": "Independent review stays user-invoked."
    },
    {
      "stage": "generic-advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "Not high-risk."
    },
    {
      "stage": "supervision",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "No unattended loop."
    },
    {
      "stage": "blind-lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "No competing designs."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "No competing designs."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "No competing designs."
    },
    {
      "stage": "design-ui",
      "classification": "not-applicable",
      "role": "design",
      "reason": "No rendered UI."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "not-applicable",
      "role": "hat-facts-gaps",
      "reason": "Hat wave skipped: each cell fixes a reported, traced bug with the fix shape in the report."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    }
  ]
}
```

## Proof

- Each cell: its verify command green after a recorded red.
- Lane close: the declared suite green on the merged branch.
