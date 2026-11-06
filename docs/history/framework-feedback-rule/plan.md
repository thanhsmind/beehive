---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: framework-feedback-rule

## Summary

Owner feedback given in this repo changes the framework, not only an
agent's private memory. One rule lands in CLAUDE.md, the AGENTS block (and
so AGENTS.md), the rule index and the bee-capturing skill.

Mode: `small` — one rule in four homes, then regen.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`7d5b60ab`): owner feedback here is a framework change request.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The capture rule lives in the block source | read | packages/bee/AGENTS.block.md:214 | `## Capture what settles` |
| 2 | The capture skill section exists | read | skills/bee-capturing/SKILL.md:27 | `## Capture the moment it settles` |

## Approach

One docs cell writes the four homes and runs `bee dev regen`.

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| A home misses the rule | LOW | ffr-1 | rg finds the rule id in every home |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"not-applicable","role":"code","reason":"No code changes."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Docs only; the manifest check is the proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"ffr-1 adds the rule to its four homes."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader already read every anchor."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The next release publishes it."},
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
| ffr-1 | Route owner feedback in this repo into the framework | four homes, AGENTS.md, regen output | — | the rule in CLAUDE.md and AGENTS.md | release-manifest check green |

```json
[
  {
    "id": "ffr-1",
    "feature": "framework-feedback-rule",
    "title": "Route owner feedback in this repo into the framework",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": [
      "7d5b60ab-7a0c-460b-9ed1-b0508cdcee63"
    ],
    "files": [
      "CLAUDE.md",
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "skills/bee-capturing/SKILL.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/framework-feedback-rule/CONTEXT.md"
    ],
    "affects_skills": [
      "skills/bee-capturing/SKILL.md"
    ],
    "affects_specs": [],
    "action": "Per decision 7d5b60ab: add a CLAUDE.md section 'Feedback here changes the framework'; add rule agents-feedback-changes-framework to packages/bee/AGENTS.block.md under Capture what settles; add its rule index entry with applied_at CLAUDE.md and skills/bee-capturing/SKILL.md; add one sentence to skills/bee-capturing/SKILL.md Capture the moment it settles. Run `.bee/bin/bee dev regen`.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "owner feedback about how agents work in this repo goes to the owning doctrine, hook or skill through the bee chain and a release; a memory note alone is a miss"
      ],
      "artifacts": [
        {
          "path": "packages/bee/AGENTS.block.md",
          "substantive": "rule agents-feedback-changes-framework"
        }
      ],
      "key_links": [
        "AGENTS.md matches the block after bee dev regen"
      ],
      "prohibitions": [
        "Do not delete agent memory files"
      ]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Docs | every home | Pass when each names the rule |

## Open Questions

(none)

## Out of scope

- Moving existing agent memory notes into the framework.
