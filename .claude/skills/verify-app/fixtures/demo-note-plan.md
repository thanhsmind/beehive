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
    "id": "demo-note-1",
    "feature": "demo-note",
    "title": "Add a note file",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [],
    "files": [
      "NOTE.md"
    ],
    "read_first": [
      "README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Write NOTE.md",
    "verify": "test -f NOTE.md",
    "must_haves": {
      "truths": [
        "NOTE.md exists"
      ],
      "artifacts": [
        {
          "path": "NOTE.md",
          "substantive": "holds a note"
        }
      ],
      "key_links": [],
      "prohibitions": []
    },
    "behavior_change": true
  },
  {
    "id": "demo-note-2",
    "feature": "demo-note",
    "title": "Verify NOTE.md content",
    "lane": "small",
    "role": "test",
    "deps": [
      "demo-note-1"
    ],
    "decisions": [],
    "files": [
      "NOTE.md"
    ],
    "read_first": [
      "NOTE.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Verify NOTE.md contains note",
    "verify": "grep -q note NOTE.md",
    "must_haves": {
      "truths": [
        "NOTE.md contains note"
      ],
      "artifacts": [
        {
          "path": "NOTE.md",
          "substantive": "holds a note"
        }
      ],
      "key_links": [],
      "prohibitions": []
    },
    "behavior_change": false
  }
]
```
