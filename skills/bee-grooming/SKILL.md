---
name: bee-grooming
description: >-
  Hunt and kill tech debt IN THE CURRENT PROJECT — dead code, stale docs, TODO/stubs, duplication, drifted specs — reported in plain project language. bee's own housekeeping (the entropy score) is a short side-note, and `.bee/`, `.claude/`, `.codex/` are never treated as project debt. Use when the user asks to clean up, find debt, or audit the repo.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Computes the entropy score from bee records via the vendored .bee/bin helpers.
---

# Grooming (undertaker bees)

On-demand hygiene pass, run when the hive is idle. Fixed cycle: **hunt the project → propose → execute → close the loop**, plus a quick hive-housekeeping check on the side. Grooming decides nothing alone and deletes nothing alone.

## Scope — the project, not the harness

Grooming cleans the **current project** (its source, docs, tests). Report findings in **plain language a non-bee user understands** ("three unused functions in the export module" — never "orphaned cells at trace level").

Out of scope, never kill/move candidates: `.bee/`, `.claude/`, `.codex/`, the `AGENTS.md` bee block, bee's vendored helpers (`.bee/bin/`), `node_modules/`, build output, and generated directories.

A genuine bee/harness bug found during the hunt is NOT a project kill — note it in **one line** as a *harness issue to report upstream to bee* and move on.

## 1. Hive housekeeping — entropy score

Bee's own tidiness (loose cells, stale reservations, un-synced specs) — a few lines, never the headline; the project hunt below is the main event. `broken_tools` and any bee-lib bug are **harness** — route upstream, never a project proposal.

```
ENTROPY SCORE = orphaned cells ×10 + unverified cells ×5 + stale decisions ×5
              + stale specs ×5 + backlog-without-outcome ×2 + stale work ×3
              + broken tools ×8, cap 100
```

**0** perfect · **1–25** healthy · **26–50** attention · **51–100** action required.

Counting rules and score sources: `references/grooming-reference.md` ("Entropy Computation"). Report the score AND the trend vs. the last run (`entropy-audit` entries in `.bee/backlog.jsonl`) — a rising trend at "healthy" still deserves a sentence.

## 2. Hunt the project's debt

Exclude `.bee/`, `.claude/`, `.codex/`, `node_modules/`, build output (see Scope). Per-source checklists: `references/grooming-reference.md` ("Hunt Checklists").

- friction clusters (cell traces, `.bee/backlog.jsonl`)
- dead code and unused exports
- stale docs that contradict the code
- stale, missing, or duplicated area truth — bundle-aware (`knowledge check --json`) or spec-frontmatter-based; areas with code but no concept/spec
- TODO/stub debris
- verify-commands that no longer run
- superseded-but-still-cited decisions
- slop patterns in recent diffs (empty catches, redundant `return await`, dead flags, copy-paste drift)
- test-prune: duplicate-logic or low-validation-value tests, surfaced with evidence of *why*

Prove non-use before calling anything dead: dynamic imports, reflection, config-driven loading, and external callers all count as use. "Obviously dead" without evidence is a red flag, not a finding.

**Test-prune's hard gate:** the action is merging near-duplicate cases into one table-driven test, or deleting a genuinely dead case — never a raw line-count cut. Deleting a test changes guard *behavior*: every suite touched by the prune must run and show **green AFTER the prune, in the same batch** — proposed-now-verified-later is not ready to execute. The surviving case(s) must still demonstrably catch what the pruned duplicates caught.

## 3. Propose

Each kill candidate: **pain** (what it costs today) / **predicted impact** (what removal buys) / **risk lane** (tiny or small). Rank by pain × impact, present the top few — never dump every candidate.

**MANDATORY user approval before any deletion. Grooming never deletes on its own initiative.** No approval, no kill — regardless of how obvious the candidate looks.

## 4. Execute

Approved kills run as normal tiny/small cells through the bee-swarming worker loop ("Execute") — reserve, verify, cap. Grooming never edits files directly. §1/§2 mechanical scans delegate as extraction/generation-tier I/O workers per the Delegation contract (`bee-hive/references/routing-and-contracts.md`); dead-code proof stays generation; any other ad-hoc dispatch defaults to the generation slot model, and ceiling requires the `[bee-tier: ceiling]` marker plus a one-line justification.

One approved kill per cell. Approval of one kill is not approval of its "related" neighbors — never batch unapproved kills into an approved cell.

## 5. Close the Loop

After execution, record the actual outcome against the prediction: `node .bee/bin/bee.mjs backlog add --type kill-outcome --severity <P1|P2|P3> --layer <layer> --title "<outcome>" --detail "<predicted vs actual>" --feature <feature>` (field guidance: `references/grooming-reference.md` ("Outcome Template")). Prediction wrong? That is signal, not embarrassment. Feed durable lessons to `bee-compounding` — grooming that never learns just mows the same grass.

## Headless

`mode:headless` = audit + propose only: compute the score and trend, run the hunt, emit ranked proposals in a structured terminal report with approvals deferred to an `Outstanding Questions` section. Headless NEVER executes kills and never deletes anything.

## Red Flags

- treating `.bee/`, `.claude/`, or `.codex/` (or bee's vendored helpers) as project debt — the harness is out of scope
- presenting a bee/harness bug as a project kill instead of a one-line "report upstream to bee" note
- findings written in bee-jargon (cells, traces, capCell) instead of plain project language
- letting the entropy score / hive housekeeping dominate the report — the project hunt is the main event
- deleting anything without recorded user approval
- "obviously dead" claimed without proof of non-use
- batching multiple kills into one approved cell
- executing a test-prune kill without the touched suites showing green *after* the prune, in the same batch
- grooming editing files directly instead of dispatching cells
- dumping every candidate instead of ranking by pain × impact
- skipping the actual-outcome record after execution
- reporting the score without the trend

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Grooming pass complete: entropy score reported, approved kills executed, outcomes recorded. Invoke bee-compounding skill.

| Reference | When to Load |
|---|---|
| `references/grooming-reference.md` | entropy counting rules, hunt checklists, proposal/outcome templates, slop-pattern list |
