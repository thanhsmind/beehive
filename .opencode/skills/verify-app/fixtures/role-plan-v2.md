---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: demo-note
## Summary
Verification fixture plan.
## Load-bearing claims
| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The sandbox has a README | read | README.md:1 | # sandbox |
## Cells, current slice preview
```json
[
  {
    "id": "demo-note-2",
    "feature": "demo-note",
    "title": "Add a second note",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [],
    "files": [
      "NOTE2.md"
    ],
    "read_first": [
      "README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Write NOTE2.md",
    "verify": "test -f NOTE2.md",
    "must_haves": {
      "truths": [
        "NOTE2.md exists"
      ],
      "artifacts": [
        {
          "path": "NOTE2.md",
          "substantive": "holds a note"
        }
      ],
      "key_links": [],
      "prohibitions": []
    },
    "behavior_change": true
  }
]
```

## Role assignments

```json
{
 "schema_version": "1.0",
 "runtime": "pi",
 "roster_sha256": "REPLACE_FROM_ROSTER_STALE_ERROR",
 "stages": [
  {
   "stage": "implementation",
   "classification": "required",
   "role": "code",
   "reason": "demo-note-2 writes a file."
  },
  {
   "stage": "test-and-live-proof",
   "classification": "not-applicable",
   "role": "test",
   "reason": "Docs only; the manifest check is the proof."
  },
  {
   "stage": "documentation-and-capture",
   "classification": "required",
   "role": "docs",
   "reason": "hfd-1 adds the flag to the rule homes."
  },
  {
   "stage": "read-only-gather",
   "classification": "not-applicable",
   "role": "read",
   "reason": "The leader already read every anchor."
  },
  {
   "stage": "hat-facts-gaps",
   "classification": "not-applicable",
   "role": "hat-facts-gaps",
   "reason": "Small lane: no hat wave."
  },
  {
   "stage": "hat-alternatives",
   "classification": "not-applicable",
   "role": "hat-alternatives",
   "reason": "Small lane: no hat wave."
  },
  {
   "stage": "hat-user-impact",
   "classification": "not-applicable",
   "role": "hat-user-impact",
   "reason": "Small lane: no hat wave."
  },
  {
   "stage": "hat-risks",
   "classification": "not-applicable",
   "role": "hat-risks",
   "reason": "Small lane."
  },
  {
   "stage": "hat-value",
   "classification": "not-applicable",
   "role": "hat-value",
   "reason": "Small lane."
  },
  {
   "stage": "planning",
   "classification": "not-applicable",
   "role": "plan",
   "reason": "The leader writes this plan."
  },
  {
   "stage": "fact-extraction",
   "classification": "not-applicable",
   "role": "extraction",
   "reason": "No extraction."
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
   "reason": "No consult needed."
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
   "reason": "One clear design."
  },
  {
   "stage": "blind-lane-2",
   "classification": "not-applicable",
   "role": "lane-2",
   "reason": "One clear design."
  },
  {
   "stage": "blind-lane-3",
   "classification": "not-applicable",
   "role": "lane-3",
   "reason": "One clear design."
  },
  {
   "stage": "design-ui",
   "classification": "not-applicable",
   "role": "design",
   "reason": "No rendered UI."
  }
 ]
}
```
