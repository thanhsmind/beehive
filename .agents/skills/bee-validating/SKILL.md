---
name: bee-validating
description: >-
  Prove the plan against repo reality with concrete evidence before any code is written. Use when planning has an approved work shape that needs feasibility validation before swarming, or when a plan smells like plausibility instead of proof.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Validation reads state and cells through the vendored .bee/bin helpers.
---

# Validating — Guard Bees

The hard gate between planning and execution: rejects plausible-sounding plans without repo/system proof. Never skipped — it scales down, never away. Start with `node .bee/bin/bee.mjs status --json`; onboarding missing/stale → stop, invoke bee-hive. Rules stated bare — decision IDs: `references/provenance.md`.

## Lane Scaling

| Lane | What runs |
|---|---|
| `tiny`/`small` | not separately invoked — the reality check runs inline inside bee-planning §5; no validating subagents spawned |
| `standard`, ≤5 files, no hard-gate flag | full protocol; review wave runs **inline on the session model**, no dispatch |
| `standard`, >5 files or a hard-gate flag | full protocol; review wave **dispatches** the merged reviewer |
| `high-risk` | full protocol; review wave scales to a persona panel |
| `spike` | whatever single proof the question demands |

A hard-gate flag, a 6th file, or doubt about self-review independence restores the dispatch.

## Required Inputs

`CONTEXT.md`; `plan.md` (frozen at Gate 2 — byte-identical to what was approved); discovery/approach content (files or `plan.md` sections); current-slice cells. Missing/unapproved `plan.md`, or no current-slice cells → stop, return to bee-planning. Fallback rules: `references/validation-reference.md` ("Required Inputs in full").

## Operating Contract

| Step | Rule |
|---|---|
| 1. Orient | state, mode/lane, approved shape, cells; delegates to an I/O worker per the D2 rubric, launched inside the review wave, never ahead. Slice 2+: `state validation-cache check --json` first. |
| 2. Reality gate | MODE FIT / REPO FIT / ASSUMPTIONS / SMALLER PATH / PROOF SURFACE, each PASS\|FAIL with evidence. A fail halts the pipeline, returns to bee-planning. |
| 3. Feasibility matrix | every blocking assumption: assumption \| risk \| proof required \| evidence \| result \| sources. Multi-cell slices add a schedule row (`cells schedule`, zero cycles). |
| 4. Delta rule (slice 2+) | re-prove only stale rows and new cells; fresh rows carry forward as `cached (slice N, sources unchanged)`. Stale = source sha256, newest active decision id, or `sha256(plan.md)` changed. Any cache defect re-proves everything. |
| 5. Spikes | one yes/no question per spike, for any unproven assumption that can invalidate the current work. |
| 6. Review wave | one merged reviewer (structure + cold-pickup), dispatched at stage start beside the matrix — cost is max(reviewer, matrix), never the sum; sync point holds findings until Gate 3; one shot, then at most one blocker-scoped pass. Full mechanics + runtime dispatch differences: `references/validation-reference.md` ("Review Wave in full", "Merged Reviewer Subagent Prompt"). |
| 7. Decide | decision vocabulary below, then Gate 3. |

## Accepted Evidence

Existing implementation, file/API/type inspection, command output, build/typecheck/test result, official version/doc proof, runtime probe, or a `.bee/spikes/<feature>/` result. "Should work" / "likely" / "expected" / model knowledge → **NOT READY** — plausibility always fails, cached rows included.

**tiny/small exception:** a quoted file:line citation is sufficient — no runtime spike when in-repo precedent exists; an unprecedented API/library/technique still needs one. `standard`/`high-risk` stay at full weight regardless.

Spike rules, debug discipline, docs/history/ code-extension ban: `references/validation-reference.md` ("Spike / Probe Rules").

## Decision Vocabulary

```text
READY
READY WITH CONSTRAINTS
NOT READY - RUN SPIKE
NOT READY - RETURN TO PLANNING
```

READY is a feasibility verdict, not execution approval — Gate 3 still requires the user.

## Gate 3 — Execution Approval

**Advisor consult — before this gate opens, every bypass level.** A high-risk/hard-gate slice consults the configured advisor before presenting or self-approving Gate 3, then records it: `state advisor-ref record --advisor "<identity>" --digest-file <path>`. Resolution recipe: `references/validation-reference.md` ("Advisor Consult in full").

**Enforcement is a throw, not a warning:** `state gate --name execution --approved true` refuses for high-risk work when the selected `advisor_ref` is missing or stale. Stale = its feature differs from `state.feature`, OR the newest active decision id changed since the consult, OR `sha256(plan.md)` changed since the consult, OR the ref predates the most recent execution-gate revocation — never a time-based TTL. Advice is data for the human decision, never a decision itself — a conflict with a locked `CONTEXT.md` decision is surfaced, never silently followed or auto-approved.

Write the full machine report to `docs/history/<feature>/reports/validation-<slice>.md` — always, immediately. Present the human layer (template in the reference), ask verbatim: **"Feasibility validated. Approve execution?"** Approval covers current work only; future slices return to planning and validating. Report/briefing-refresh timing and the optional cross-model second opinion: `references/validation-reference.md` ("Approval Gate Block").

On approval: `state gate --name execution --approved true`, then `state set --owner validating --phase swarming --summary "<summary>" --next-action "Invoke bee-swarming for the validated work."` (`validated` is not a phase — the approved gate records that).

**Gate bypass (opt-in), level-aware:** `normal` self-approves tiny/small/standard with no hard-gate flag; `full`/`total` also self-approve high-risk/hard-gate. When covered: don't ask, set the gate, still write the report, log an audit line, post `⚡ auto-approved Gate 3 (bypass)`, hand off. Level table: `bee-hive/references/routing-and-contracts.md` ("Gate bypass mode").

## Headless

Run every check, apply unambiguous cell repairs, defer ambiguous ones to an `Outstanding Questions` section. Stops at the Gate 3 question — emits the approval block and the READY/NOT READY verdict and exits. Never self-approves execution.

## Red Flags

skipping the reality gate or feasibility matrix · spawning the review wave for a tiny/small lane (their reality check lives inline in planning) · accepting plausibility language as evidence, cached rows included · carrying a row forward without hash-verified sources, or reading a degraded cache as permission to skip a proof rather than re-prove it · continuing after a NO spike because a workaround "probably works" · a third reviewer pass instead of escalating; presenting Gate 3 with a wave member outstanding · splitting the merged reviewer in two, or losing a finding class to the merge · approving (or letting approval cover) future slices · CRITICAL cell flags left unfixed at approval time · a tiny fix wearing epic ceremony; a hard-gate change routed below high-risk · self-approving Gate 3, in any mode · presenting or auto-approving Gate 3 for high-risk/hard-gate work without a recorded non-stale `advisor_ref` · treating an advisor digest as a decision instead of data, or letting it silently override a locked `CONTEXT.md` decision.

Violating the letter of the rules is violating the spirit of the rules.

Validation complete and Gate 3 approved. Invoke bee-swarming skill.

## Reference Files

| File | When to Load |
|---|---|
| `references/validation-reference.md` | Report formats, repair routing, required inputs, review wave + advisor consult mechanics, the merged reviewer prompt, approval block |
| `references/provenance.md` | Decision IDs + rationale for every body rule |
