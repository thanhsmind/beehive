---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: pstack-verify-trust

## Summary

Three short rules that make bee's own proof harder to fake: a risk row must
name the one fact it rests on and prove it by running code; a sandbox is
"ready" only when it is usable; the close report says which model judged.
One docs cell, one regen.

Mode: `small` — 0 risk flags (lane by recorded deviation D4).

## Requirements (from CONTEXT.md)

- D1: one fact + its proof per MEDIUM/HIGH risk row. D2: ready means usable in verify-app Launch. D3: `judged by <model>` in the slice-close report. D4: lane small, one cell.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The risk map table has a Proof needed column but no "one fact" rule | read | skills/bee-planning/references/planning-reference.md:199-202 | `## Risk map` / `\| Component \| Risk \| Reason \| Lands in \| Proof needed \|` |
| 2 | Launch's readiness is "exits 0" | read | .bee/verify/verify-app/SKILL.md:53 | `` `launch` prints the sandbox path and is ready when it exits 0. It: `` |
| 3 | The slice-close tick names the sample choice but not the judge model | read | skills/bee-swarming/references/swarming-reference.md:274-275 | `~1-in-3 sample (choice stated in the slice-close tick, never a silent` / `skip)` |
| 4 | judge-record already stamps model_independence from the caller's pinned models | ran | `.bee/bin/bee cells judge-record --help` | `appends it, stamped with model_independence` |

## Cells — current slice

```json
[
  {
    "id": "pvt-1",
    "feature": "pstack-verify-trust",
    "title": "Name the one fact and its proof, make ready mean usable, and name the judge at slice close",
    "lane": "small",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["b2ded703-c795-4f07-8386-d974371d6cc9", "fcc20192-1cfa-4da5-ae6e-a17cd2216648", "05a27950-292c-4887-a012-1f2f7eb95012"],
    "files": [
      "skills/bee-planning/references/planning-reference.md",
      ".bee/verify/verify-app/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pstack-verify-trust/CONTEXT.md",
      "skills/bee-planning/references/planning-reference.md",
      ".bee/verify/verify-app/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_skills": ["skills/bee-planning/references/planning-reference.md", ".bee/verify/verify-app/SKILL.md", "skills/bee-swarming/references/swarming-reference.md"],
    "affects_specs": [],
    "behavior_change": true,
    "action": "Three prose edits, per pstack-verify-trust D1, D2, D3; each in its file's voice, short sentences, no copied pstack prose. EDIT 1 (D1) — skills/bee-planning/references/planning-reference.md, section `## Risk map` (search the heading): keep the table; add one short paragraph under it: every MEDIUM or HIGH row names the ONE fact the change is safe because of (for example: this call only drops entries already dead) and the command or small script that proves that fact by running the real code — the planner runs it before the gate and pastes the output as the row's evidence; a row whose reason only sounds safe is moved to Open Questions, because a writeup reads the same whether it is true or not. Also add the phrase `the one fact: …` as the recommended shape of the Reason cell for those rows. Mirror the same sentence, in one line, in the `## Approach` guidance of the plan.md template (search `compact risk map`), as a pointer, never a second copy. EDIT 2 (D2) — .bee/verify/verify-app/SKILL.md, section `## Launch` (search the heading), right after the sentence `` `launch` prints the sandbox path and is ready when it exits 0. ``: add two or three sentences: ready means usable — the sandbox came up from an empty data dir with every migration and seed step done, and the first mapped feature can be driven; a process that answers, or a doctor row that passes before init finished, is not ready; evidence captured before that point is discarded. Keep the numbered list after it unchanged. EDIT 3 (D3) — skills/bee-swarming/references/swarming-reference.md, in the semantic-judge paragraph (search `choice stated in the slice-close tick`): extend that parenthesis or add one sentence after it: the slice-close tick also names the judge — `judged by <model>` taken from the judge-record's recorded judge model and `model_independence`, or `frozen judge only` on tiny/small — so the human sees who verified without asking; say in one clause that this is the goal-check judge, never the user-invoked review. Do NOT touch bee-reviewing. REGEN inside this cell: `.bee/bin/bee dev regen` (it ends with release-manifest --write); commit the regenerated copies with the change, path-scoped (`git commit -- <paths>`); use the Edit tool for edits.",
    "verify": "`rg -c 'the one fact' skills/bee-planning/references/planning-reference.md` prints at least 2. `rg -c -i 'ready means usable' .bee/verify/verify-app/SKILL.md` prints 1. `rg -c 'judged by' skills/bee-swarming/references/swarming-reference.md` prints at least 1. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.",
    "must_haves": {
      "truths": [
        "A MEDIUM or HIGH risk row cannot pass the gate on a reason alone; it names its one fact and the run that proved it",
        "verify-app's Launch defines ready as usable and discards evidence captured earlier",
        "The slice-close tick names the judge model or says frozen judge only"
      ],
      "artifacts": [
        {"path": "skills/bee-planning/references/planning-reference.md", "substantive": "one-fact-and-proof paragraph under Risk map plus a one-line pointer in the Approach guidance"},
        {"path": ".bee/verify/verify-app/SKILL.md", "substantive": "ready-means-usable sentences in Launch"},
        {"path": "skills/bee-swarming/references/swarming-reference.md", "substantive": "judged-by line in the semantic-judge paragraph"}
      ],
      "key_links": ["rendered copies under .claude/skills match skills/ and .bee/verify after regen"],
      "prohibitions": ["No edit to bee-reviewing", "No copied pstack sentence", "No edit to AGENTS.block.md"]
    }
  }
]
```

## Proof

- docs → pointer checks named in the cell's verify; `bee dev regen` and `release-manifest --check` clean.
