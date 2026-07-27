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

Validating is the hard gate between planning and execution. It rejects beautiful fantasy plans by demanding repo/system evidence, feasibility proof, and cells a stranger could pick up cold. Never skip validating — it scales down, it does not disappear.

**Lane scaling.** For `tiny` and `small`, this skill is **not separately invoked**: the reality check runs inline inside bee-planning before the merged shape+execution gate (see bee-planning §5), and no validating subagents are spawned. This skill's full protocol below applies from `standard` upward — a small-diff `standard` (counted touch set ≤5 product files, zero hard-gate flags) runs the review wave **inline on the session model** instead of dispatching it (lane-lean D3, Review Wave below); a larger `standard` dispatches the merged reviewer; `high-risk` always scales it to a persona panel. A `spike` runs whatever single proof its question demands, nothing more.

Start with `node .bee/bin/bee.mjs status --json`. If onboarding is missing or stale, stop and invoke bee-hive.

## Required Inputs

- `docs/history/<feature>/CONTEXT.md`
- `docs/history/<feature>/plan.md` — approved and **frozen at Gate 2** (D1): its content sections are immutable once `approved_gates.shape` is set, so what validating reads is byte-identical to what the human approved
- the discovery and approach content: `docs/history/<feature>/discovery.md` and `approach.md` **if they exist**; otherwise the `## Discovery` and `## Approach` sections folded into `plan.md` (decision 0009 — separate files are written only for L2+ discovery or high-risk lanes)
- current-slice cells exist: `node .bee/bin/bee.mjs cells list --feature <feature>` (D2 — the current slice lives only in cells; there is no separate slice document)

If `plan.md` is absent, unapproved, or the current-slice cells do not exist, stop and return to bee-planning. Never validate an unapproved shape. A missing `discovery.md`/`approach.md` is **not** a failure when `plan.md` carries the equivalent sections — read those; stop only if neither exists.

## Operating Contract

1. **Orient** on state, mode/lane, the approved shape, and the current-work cells. The orient read (CONTEXT.md, plan.md, discovery/approach, cells) delegates as an extraction-tier I/O worker per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`) when the D2 rubric fires — launched in the stage-start wave (§Review Wave), not ahead of it; judgment (mode fit, reality-gate scoring) stays on the session model. **On slice 2+, first run `node .bee/bin/bee.mjs state validation-cache check --json`** — it names which rows are still hash-fresh and which moved.
2. **Reality gate:** MODE FIT / REPO FIT / ASSUMPTIONS / SMALLER PATH / PROOF SURFACE — each scored PASS|FAIL with file/command evidence. Fail on nonexistent code paths, unsupported commands, stale versions, missing credentials, hidden architecture work, or excess ceremony. A failed reality gate halts the pipeline and returns to bee-planning. Dimensions are cacheable on the same terms as matrix rows.
3. **Feasibility matrix:** every blocking assumption gets a row — assumption | risk | proof required | evidence | result | **sources**. Sources are what the row was proven from: `{path, sha256}` for file evidence, `{command, output_sha}` for command evidence. Record them with `bee.mjs state validation-cache record --slice <n> --rows-file <f>` (the verb hashes each path itself). Accepted evidence only (below). Plausibility language is an automatic NOT READY. For multi-cell slices, the matrix includes a schedule row: `bee cells schedule` reports zero cycles and the expected wave shape — required evidence, not optional.
4. **Delta rule (slice 2+).** Re-prove only **stale rows and cells that are new**; carry fresh rows forward verbatim as `evidence: cached (slice N, sources unchanged)`. A row is stale when any source sha256, the newest active decision id, or `sha256(plan.md)` changed — the `advisor_ref` anchors, same never-a-TTL law. **Any cache defect re-proves everything** (`degraded`/`revalidate: full`), as does a row storing no hashes: a cache problem buys more validation, never less. Cached evidence is still Accepted Evidence and still auto-fails on plausibility language.
5. **Spikes** for unproven assumptions that can invalidate the current work.
6. **Review wave** — one merged reviewer (structure + cold-pickup cells) dispatched at stage start while the matrix runs; one shot, then at most one blocker-scoped pass. On slice 2+ its scope is the new/changed cells and stale rows, not the whole frozen plan.
7. **Decide** using the decision vocabulary, then ask Gate 3.

Load `references/validation-reference.md` for report formats, repair routing, and the subagent prompts.

## Accepted Evidence

Existing implementation, file/API/type inspection, command output, build/typecheck/test result, official version/doc proof, runtime probe, or a `.bee/spikes/<feature>/` result. Evidence that is only "should work", "likely", "expected", or model knowledge → **NOT READY**.

**Static evidence for `tiny`/`small` (test-economy D5).** At these lanes, a file:line citation proving the code path exists and behaves as assumed — quoted, not paraphrased — is **sufficient** to pass the reality gate; it does not need a runtime spike. Precedent in-repo → cite it and move on; an API, library, or technique with no comparable prior use here → spike per the rules below. This does not loosen `standard`/`high-risk`, where the reality gate and feasibility matrix run at full weight regardless of lane.

## Spike Rules

- One spike answers exactly one yes/no question.
- Disposable code lives under `.bee/spikes/<feature>/`.
- **NO** → return to bee-planning with the failed assumption and the required plan change.
- **YES** → record the discovered constraints for planning and execution.
- Spike code never silently becomes production code.
- **Debug discipline (test-economy D5): hypothesis before repro, read before rerun.** Before writing any repro script, record the hypothesis in one line plus the file:line evidence from reading the code that grounds it — a repro script with no prior hypothesis is not a spike, it's a guess with extra steps. Cap the loop at **2** failed repro rounds: after the second wrong repro, stop running scripts and go back to reading/instrumenting the actual code path instead of trying a third guess blind. Prose law; the machine-enforced proof-tier matrix lives in `bee-executing/SKILL.md`.

**Verify scripts and any executable code NEVER go in `docs/history/`** (GitHub #17). `docs/history/` is the tech-agnostic knowledge layer — `.md` only (CONTEXT.md, plan.md, reports, walkthrough). A cell's `verify` is a runnable command; when it needs a multi-line harness, that script lives in **the project's own scripts** (committed with the product, so `verify` points at it) or, if disposable, in **`.bee/spikes/<feature>/`** — the disposable-code half of the one canonical scratch home (docs/specs/doctrine-layer.md R17). The write-guard denies a code-extension file (`.sh`, `.mjs`, `.py`, …) written under `docs/history/`, and also denies any scratch-shaped write landing in a tracked directory outside `.bee/tmp/`/`.bee/spikes/`.

## Review Wave

**A wave, not a chain.** At stage start dispatch **simultaneously** the merged reviewer below and — when the D2 rubric fires — the orient/extraction worker, then run the reality gate and feasibility matrix on the session model **while the wave runs**: the stage costs max(reviewer, matrix), not their sum. **Sync point (decision 0017, now wave-wide):** findings block nothing until the Gate 3 presentation — or its bypass self-approval — and neither ever happens while **any** wave member is outstanding.

**One dispatch, two mandates, both vocabularies.** One `bee-review`-class dispatch on the **`review` slot** (decision 0021 — `resolveTier(root, 'review', runtime)`, default opus on Claude, generation fallback; state the model explicitly; if the runtime cannot select per-agent models, cap its reads and output instead) replaces the former plan-checker + cell-reviewer pair and returns **one report, two sections**: **Structure** — the adversarial check over its 5 dimensions, every finding **BLOCKER** or **WARNING**; and **Cells** — the cold-pickup review, every finding **CRITICAL** (all fixed before approval) or **MINOR** (may ship with a recorded note). Merging the dispatches never merges the finding classes. Prompt, dimensions, and both flag lists: the reference.
<!-- bee:only claude -->
On Claude Code, spawn `subagent_type: "bee-review"` when `.claude/agents/bee-review.md` exists (W3, AO5/AO10) — bee's own rendered agent for the review tier, never `general-purpose` (`bee-model-guard` denies that pairing).
<!-- bee:end -->
<!-- bee:only codex -->
Codex has no per-agent subagent type (AO11), so the tier stays enforced as a read budget + output cap only.
<!-- bee:end -->
It is a **read-only gather**, never a cell: a cli-shaped review slot resolves with the purpose-scoped `resolveTier(root, 'review', runtime, {for:'gather'})` — a bare 3-arg resolve of one now refuses (AO12/B1); a model-shaped slot is unaffected by purpose.

**One shot, then at most one blocker pass.** The merged reviewer runs **once**. WARNING-level and mechanically fixable findings (a missing link, a vague verify command, a dependency typo) the orchestrator applies **directly to the cells** — legal because cells are mutable before Gate 3 (D2). Only **unresolved BLOCKERs** trigger a **second and final** pass, scoped to those blockers. No third pass: a BLOCKER open after pass 2 escalates to the user with both positions.

**Small-diff standard: same mandates, no dispatch (lane-lean D3).** When the counted touch set is ≤5 product files with zero hard-gate flags, the merged reviewer is not dispatched — the session model runs both mandates itself: Structure over the same 5 dimensions, Cells as a cold-pickup pass, findings in the same vocabularies, recorded in the validation report. Same sync point, same one-shot-then-one-blocker-pass cap. A hard-gate flag, a 6th product file, or genuine doubt about self-review independence restores the dispatch. `high-risk` never takes this path.

**High-risk lane:** scale this same merged dispatch to a persona panel — coherence + feasibility lenses always, plus conditional lenses (security, product, scope-guardian) chosen by the diff of concerns. Dedupe findings, then synthesize into auto-fix vs present-for-decision buckets.

## Decision Vocabulary

```text
READY
READY WITH CONSTRAINTS
NOT READY - RUN SPIKE
NOT READY - RETURN TO PLANNING
```

READY is a feasibility verdict, not execution approval — Gate 3 still requires the user.

## Gate 3 — Execution Approval

**Advisor consult (AO2b/AO3/AO4) — runs before this gate opens, at every bypass level.** For a high-risk or hard-gate slice, the orchestrator consults the configured advisor **before** presenting Gate 3 to the human, and before self-approving it under any bypass level (`normal`/`full`/`total` lift the *human* checkpoint below — they never lift this mechanical precondition). Resolve the advisor from config (`resolveAdvisor(root, runtime)`):
- **cli-shaped** advisor → run the configured command verbatim, read-only, with an evidence bundle on stdin (plan summary, risk map, validation findings, open questions — never session history, never secrets) and capture the digest.
- **model-shaped** advisor → dispatch a `bee-review`-class read-only run with the same evidence bundle.
- **unconfigured** advisor (`resolveAdvisor` returns `null`) → record that fact and proceed. AO2(b) adds one trigger; it is not a hard dependency on an advisor being configured.

Then record the consult: `node .bee/bin/bee.mjs state advisor-ref record --advisor "<identity>" --digest-file <path>` (the verb stamps the staleness anchors itself — the caller supplies only the advisor identity and the digest file).

**Enforcement is a throw, not a warning.** For high-risk work, `node .bee/bin/bee.mjs state gate --name execution --approved true` refuses — throws, never just warns — when the selected record's `advisor_ref` is missing or stale (AO3/AO13). Nothing is written until a non-stale `advisor_ref` exists; this is CLI-enforced, not optional ceremony. An `advisor_ref` is stale if **any** of (AO13, verbatim):
1. its feature differs from `state.feature`;
2. the newest active decision id changed since the consult;
3. `sha256(plan.md)` changed since the consult;
4. the ref predates the most recent revocation of the execution gate.

Never a time-based TTL — AO13 already burned this feature on one invented number once.

**Advice never approves a gate and never overrides a locked decision.** The consult's digest is data for the human decision, not a decision itself (critical rule 12, existing law); an advisor result that conflicts with a locked `CONTEXT.md` decision is surfaced to the human, never silently followed or used to auto-approve.

Write the full machine report (reality gate, matrix, both reviewer sections, approval block) to `docs/history/<feature>/reports/validation-<slice>.md` — it is evidence, always written immediately. The `bee-briefing` refresh (patching the implement plan's Validation Plan with the evidence links) is **presentation, not evidence**: when the bypass level covers Gate 3, skip it here and run it lazily at the next human-facing event — a real gate presentation, an explicit `bee-briefing` invocation, or feature close-out (spec #77 P6). With bypass off or stopping at this gate, refresh now as before, so the Gate 3 message links a current brief. Then present **only the human layer** in chat per the Gate Presentation Contract (template in the reference): what I'm about to do / why it's trustworthy / if it goes wrong / what you are deciding — in the user's language, jargon-free, implement plan + report linked — then ask verbatim: **"Feasibility validated. Approve execution?"** Optionally offer a cross-model second opinion first (agreement → mention it; disagreement → quote both positions; never auto-resolve). Approval covers the **current work only**; future slices return to planning and validating.

On approval, update state: `node .bee/bin/bee.mjs state gate --name execution --approved true` then `node .bee/bin/bee.mjs state set --owner validating --phase swarming --summary "<summary>" --next-action "Invoke bee-swarming for the validated work."` (`validated` is not a phase — it never was; the approved execution gate is what records that. See chain-integrity D6.)

**Gate bypass (opt-in, decisions 0010/dcf01d7b) — LEVEL-AWARE.** Read the active level first (`node .bee/bin/bee.mjs status --json` → `gate_bypass_level`), then apply the routing contract's level table (`routing-and-contracts.md §Gate bypass`):
- `off` → present Gate 3 to the human as above.
- `normal` → if the lane is `tiny`/`small`/`standard` with no hard-gate flag, do not ask (self-approve, below); a `high-risk` lane or any hard-gate flag (auth, authorization, data loss, audit/security, external provider, validation removal, migration/schema) STILL stops for the human under `normal`.
- `full` / `total` → the human deliberately **lifted the high-risk/hard-gate floor** by choosing the level; do NOT re-erect it — auto-approve Gate 3 at **every** lane, high-risk and hard-gate included.

When the level covers this gate: do not ask — take the recommendation, set `approved_gates.execution: true` yourself (`bee.mjs state gate --name execution --approved true`), still write the machine-layer report, log a one-line audit decision, post a short `⚡ auto-approved Gate 3 (bypass)` line, and hand off to bee-swarming. (`total` only stops for secret-file reads; those are not a gate. Gate 4 UAT/P1 is separate and follows §Gate 4.)

## Headless

With `mode:headless`: run every check, apply unambiguous cell repairs, and defer ambiguous ones to an `Outstanding Questions` section of the structured terminal report. Headless **stops at the Gate 3 question** — it emits the approval block and the READY/NOT READY verdict and exits. It never self-approves execution.

## Red Flags

- skipping the reality gate or feasibility matrix
- spawning the review wave for a tiny/small lane (their reality check lives inline in planning)
- accepting plausibility language as evidence
- carrying a row forward without hash-verified sources, or reading a degraded cache as permission to skip a proof rather than to re-prove it
- continuing after a NO spike because a workaround "probably works"
- running a third reviewer pass instead of escalating; presenting Gate 3 with a wave member outstanding
- splitting the merged reviewer in two, or losing a finding class to the merge
- approving (or letting approval cover) future slices
- CRITICAL cell flags left unfixed at approval time
- a tiny fix wearing epic ceremony; a hard-gate change routed below high-risk
- self-approving Gate 3, in any mode
- presenting or auto-approving Gate 3 for high-risk/hard-gate work without first running the advisor consult and recording a non-stale `advisor_ref` (AO2b/AO3/AO13)
- treating an advisor digest as a decision instead of data, or letting it silently override a locked `CONTEXT.md` decision

Violating the letter of the rules is violating the spirit of the rules.

Validation complete and Gate 3 approved. Invoke bee-swarming skill.

## Reference Files

| File | When to Load |
|---|---|
| `references/validation-reference.md` | Report formats, repair routing, the merged reviewer prompt, approval block |
