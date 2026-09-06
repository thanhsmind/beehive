---
type: bee.pattern
title: A boundary that lists field names will leak the field you forgot
description: A boundary that lists field names will leak the field you forgot
tags: [failure, security, allowlist, trust-boundary]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-a-boundary-that-lists-field-names-will-leak
  lifecycle: active
  areas: [hook-runtime]
  sources: ["docs/history/learnings/critical-patterns.md#PAT10", "original feature: evolving-loop"]
  polarity: pitfall
  critical: true
---

# A boundary that lists field names will leak the field you forgot

The same defect survived three rounds: a validator covered `title`, then `title`+`layer`+`source`,
and each time the next unremembered field was the next hole (`first_seen` rode in on
`Date.parse("Jan 1 2020 (payload)")` — lenient date parsers treat parenthesised text as a comment).
A list of field NAMES cannot make forgetting a field fail. Map each field to its validator and
**derive the field list from the map**, so an unspecced field is a red test, not a vulnerability.
Then write the table-driven test that feeds a payload into *every* field.

**Full entry:** docs/history/evolving-loop/reports/review-slice-a.md
