---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Finish phase

## Summary

One cell. `bee feature finish` moves a lane that still sits at exploring or
planning to swarming once its merge succeeds, so the scribing stamp and
close run in the same call (D1).

Mode: `small`. Playbook: bugfix.
Deviation: no hat wave; one narrow code cell with its own red-then-green test.

## Requirements (from CONTEXT.md)

- D1 (`e715bc3a`): phase move after a good merge only.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | finish runs merge, then scribing-run | ran | packages/bee-rs/crates/bee/src/verbs/feature_group.rs:316 | `OsString::from("scribing-run"),` |
| 2 | scribing-run refuses before execution | ran | bee state scribing-run | `Legal from: swarming, reviewing, scribing.` |

## Cells (current slice)

```json
[
  {
    "id": "fph-1",
    "feature": "finish-phase",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "e715bc3a-5c13-47b5-b110-8f1da00cb9a6"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "title": "bee feature finish moves a pre-execution lane to swarming after a good merge",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/feature_group.rs"
    ],
    "read_first": [
      "docs/history/finish-phase/CONTEXT.md"
    ],
    "action": "After `bee worktree merge` succeeds inside `bee feature finish --feature <f> --areas <a>`, when the lane record .bee/lanes/<f>.json has phase exploring or planning, finish sets it to swarming through the same state door `bee state set --lane <f> --owner <old> --phase swarming` uses, prints one line `feature finish: lane <f> phase <old> -> swarming (merge proved the cells)`, then runs the scribing stamp and close as today. A lane already at swarming, reviewing or scribing is not touched. A failed merge returns before any phase write. No code comments. One commit with the trailer `cell: fph-1`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- feature_group",
    "must_haves": {
      "truths": [
        "a lane at exploring with a mergeable worktree ends past the scribing stamp, and the output names the exploring -> swarming move",
        "a lane already at swarming gets no phase line and behaves as before",
        "a refused merge leaves the lane phase unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
          "substantive": "phase move after merge"
        }
      ],
      "key_links": [
        "a test fails before the change and passes after"
      ],
      "prohibitions": [
        "Do not weaken an existing assertion",
        "No code comments",
        "Never write the phase before the merge succeeds",
        "Never approve a gate"
      ],
      "open_choices": []
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
      "reason": "Cell fph-1 changes feature finish."
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
      "reason": "Cell fph-1 carries its own red-then-green unit test."
    },
    {
      "stage": "documentation-and-capture",
      "classification": "required",
      "role": "docs",
      "reason": "No doc cell; fph-1 records its concept line."
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
