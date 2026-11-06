---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: handoff-flag-doctrine

## Summary

A handoff dispatch now passes `handoff: true` beside `leader: true` and
`follow_up_of`. waggledance 0.5.3 starts the successor at once and retires
the old leader. Six homes of rule agents-context-handoff-65 gain the flag.

Mode: `small` — one phrase in six doctrine homes, then regen.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`3eddff18`): the handoff dispatch passes `handoff: true`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The block source names follow_up_of for the handoff | read | packages/bee/AGENTS.block.md:298 | `` `leader: true` and `follow_up_of` naming the current run, so the `` |
| 2 | waggledance takes the flag beside leader and follow_up_of | read | /mnt/data/Projects/goglbe/waggledance/docs/history/leader-lifecycle/CONTEXT.md:15 | `` - **D1** (`06ae7954`): `waggledance_dispatch` takes `handoff: true` beside `` |

## Approach

One docs cell adds the flag in each home and runs `bee dev regen`.

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| A home misses the flag | LOW | hfd-1 | rg finds `handoff: true` in every home |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"not-applicable","role":"code","reason":"No code changes."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Docs only; the manifest check is the proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"hfd-1 adds the flag to the rule homes."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader already read every anchor."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"release-2-79-0 publishes it."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No extraction."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No consult needed."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| hfd-1 | Pass handoff: true on a handoff dispatch | six rule homes, AGENTS.md, regen output | — | every handoff instruction names the flag | release-manifest check green |

```json
[
  {
    "id": "hfd-1",
    "feature": "handoff-flag-doctrine",
    "title": "Pass handoff: true on a handoff dispatch",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["3eddff18-250e-4ec0-a0af-b3db5b9868ce", "a23d677e-69b2-4d7c-b1a3-06e4dd9068f8"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/handbook/stages/swarming.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": ["docs/history/handoff-flag-doctrine/CONTEXT.md"],
    "affects_skills": ["skills/bee-swarming/SKILL.md"],
    "affects_specs": [],
    "action": "Per decision 3eddff18, wherever rule agents-context-handoff-65 names `follow_up_of` naming the current run, add `handoff: true` beside it: packages/bee/AGENTS.block.md, skills/bee-swarming/SKILL.md, skills/bee-hive/references/go-mode.md, skills/bee-swarming/references/swarming-reference.md, docs/handbook/stages/swarming.md and the rule index entry in router-triage-and-the-agents-md-duplication-boundary.md. Then run `.bee/bin/bee dev regen` so AGENTS.md and the rendered skill copies follow. List every file regen changed in the cap and name the rendered copies as a scope deviation.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": ["every handoff dispatch instruction names leader: true, follow_up_of and handoff: true"],
      "artifacts": [{"path": "packages/bee/AGENTS.block.md", "substantive": "handoff: true in the rule"}],
      "key_links": ["AGENTS.md matches the block after bee dev regen"],
      "prohibitions": ["Do not change when a handoff happens"]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Docs | every rule home | Pass when each handoff dispatch line names `handoff: true` |

## Open Questions

(none)

## Out of scope

- When a handoff happens (context-mark-advisory D1, D3).
