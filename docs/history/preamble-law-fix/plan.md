---
artifact_contract: bee-plan/v2
mode: tiny
---

# Plan: Preamble law fix

## Summary

Make the instruction-law test green again on `main` before release 2.57.0, by
reordering two doc paragraphs above `PREAMBLE_BUDGET_BYTES`.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one file, no behavior
change, the red law test is the proof.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The law test is red on the constant | ran | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml no_size_ceiling_on_instruction_text_survives_in_any_shipped_tooling_tree` | `budget.rs:66 (ceiling-shaped identifier "PREAMBLE_BUDGET_BYTES")` |
| 2 | The scan looks 6 lines back for a subject word | read | packages/bee-rs/crates/bee/tests/instruction_laws.rs:549 | `const SUBJECT_COMMENT_LOOKBACK: usize = 6;` |
| 3 | The NOT-instruction-text paragraph sits inside that window | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:59 | `/// one forbids a standing size ceiling on bee's AUTHORED INSTRUCTION TEXT —` |

## Cells (current slice)

```json
[
  {
    "id": "plf-1",
    "feature": "preamble-law-fix",
    "title": "Keep the preamble budget doc outside the instruction-law scan window",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": [],
    "files": ["packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs"],
    "read_first": ["docs/history/preamble-law-fix/CONTEXT.md", "packages/bee-rs/crates/bee/tests/instruction_laws.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In budget.rs, move the 8-line `/// NOT the law decision 8f63adb4 ...` paragraph above the 5-line `/// Every section switched on at once ...` paragraph, keeping one `///` separator line between them, so the line right above PREAMBLE_BUDGET_BYTES is the generated-payload paragraph. Add or remove no comment line.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee comment_baseline",
    "must_haves": {
      "truths": ["instruction_laws is green", "the comment count of budget.rs is unchanged"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs", "substantive": "reordered doc paragraphs"}],
      "key_links": [],
      "prohibitions": ["No change to PREAMBLE_BUDGET_BYTES", "No new comment lines"]
    },
    "behavior_change": false,
    "judge_obligation_ack": "doc-paragraph reorder only, no code or guard behavior changes; the independent instruction_laws test is the proof"
  }
]
```

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"One doc reorder in budget.rs; tiny runs inline."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The release is its own lane."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"This page is the plan."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"The law test is the proof."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"No docs change."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Nothing to survey."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No lookup."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"User-invoked only."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Tiny."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Tiny lane: no hat wave."}
  ]
}
```
