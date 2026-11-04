---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: bee-report-fixes

## Summary

Four open bee-report issues are still live in the current code. Each one
blocks a normal flow in a host project:

- #154 — after `bee cells unclaim`, re-claiming the same cell with the same
  worker through `dispatch prepare --claim` refuses on the worker's own hold.
- #155 — `bee worktree merge` fails with `WORKTREE_MERGE_ABORT_FAILED` when
  main has uncommitted `.bee` store changes.
- #156 — `bee onboard --apply` rewrites `.bee/bin/prompts/*` to the new
  version but leaves an old `.bee/bin/bee`, so every `dispatch prepare`
  refuses with prompt skew.
- #157 — a native (Agent tool) deploy-stage dispatch gets the read-only
  gather prompt and no `BEE_DISPATCH_ID`, so the deploy worker cannot run
  `scripts/release.sh` under its permit.

Issue #69 (a cross-project dashboard) is a new product, not a bug. It is out
of scope here and needs its own shaping.

Mode: `standard` — flags: multi-domain (reservations, worktree merge,
onboarding, dispatch), public-contracts (dispatch payload, onboard plan).
Four disjoint cells run in parallel; each writes its red test first.

## Evidence

- #154: `reserve_exec` (`verbs/reservations/reserve.rs`) filters
  `find_conflicts` by agent, then the O_EXCL lease create meets the
  agent's own live lease for the same cell and reports it as a conflict.
  `unclaim_cell` (`verbs/cells/handlers_close.rs`) flips the cell to open
  and never releases the cell's reservations.
- #155: issue output shows `M .bee/backlog.jsonl A .bee/cells/...` staged
  in the index when `git merge --no-ff --no-commit` runs; ort refuses.
  Code home: `verbs/worktree/` (staging and `bookkeeping_commit`).
- #156: onboard plans `copy_prompt` (`onboard/plan.rs`, `onboard/apply.rs`)
  but has no step for `.bee/bin/bee`; the skew guard is in
  `verbs/drivers/prompt.rs`.
- #157: `verbs/drivers/prepare.rs` builds `pane_prompt` from
  `deployment_prompt(ver)` for the herding and cli arms only; the claude
  Agent arm and the codex arms use `prompt_body`, and the
  `BEE_DISPATCH_ID` export is added only to a `command` payload. Cell
  rel730-1 recorded this deviation during the 2.73.0 release.

## Cells

- **brf-1 (#154)** — contract decision 7d3c57e8.
- **brf-2 (#155)** — contract decision 51fe42f6.
- **brf-3 (#156)** — contract decision 2b906517.
- **brf-4 (#157)** — contract decision 93ec1728.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "TBD",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "four Rust fixes with red-first tests"},
    {"stage": "tests", "classification": "not-applicable", "role": "test", "reason": "each code cell writes its own tests red-first"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "the reads were done at plan time; see Evidence"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers it"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "four reproduced bugs with located causes"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "a concept doc names the changed behavior", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "four parallel disjoint cells"},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "no rendered UI changes"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: reproduced bugs with located causes"},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "as above"},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "as above"},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "as above"},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "as above"},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "as above"},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "as above"}
  ]
}
```
