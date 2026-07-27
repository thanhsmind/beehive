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

**Lane scaling.** For `tiny` and `small`, this skill is **not separately invoked**: the reality check runs inline inside bee-planning before the merged shape+execution gate (see bee-planning §5), and no validating subagents are spawned. This skill's full protocol below applies from `standard` upward — `standard` runs the plan-checker and cell reviewer; `high-risk` scales the checker to a persona panel. A `spike` runs whatever single proof its question demands, nothing more.

Start with `node .bee/bin/bee.mjs status --json`. If onboarding is missing or stale, stop and invoke bee-hive.

## Required Inputs

- `docs/history/<feature>/CONTEXT.md`
- `docs/history/<feature>/plan.md` — approved and **frozen at Gate 2** (D1): its content sections are immutable once `approved_gates.shape` is set, so what validating reads is byte-identical to what the human approved
- the discovery and approach content: `docs/history/<feature>/discovery.md` and `approach.md` **if they exist**; otherwise the `## Discovery` and `## Approach` sections folded into `plan.md` (decision 0009 — separate files are written only for L2+ discovery or high-risk lanes)
- current-slice cells exist: `node .bee/bin/bee.mjs cells list --feature <feature>` (D2 — the current slice lives only in cells; there is no separate slice document)

If `plan.md` is absent, unapproved, or the current-slice cells do not exist, stop and return to bee-planning. Never validate an unapproved shape. A missing `discovery.md`/`approach.md` is **not** a failure when `plan.md` carries the equivalent sections — read those instead; stop only if neither the files nor the sections exist and the plan genuinely lacks discovery/approach content.

## Operating Contract

1. **Orient** on state, mode/lane, the approved shape, and the current-work cells. The orient read (CONTEXT.md, plan.md, discovery/approach, cells) delegates as an extraction-tier I/O worker per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`) when the D2 rubric fires; judgment (mode fit, reality-gate scoring) stays on the session model. **On slice 2+, first run `node .bee/bin/bee.mjs state validation-cache check --json`** — it names which rows are still hash-fresh and which moved.
2. **Reality gate:** MODE FIT / REPO FIT / ASSUMPTIONS / SMALLER PATH / PROOF SURFACE — each scored PASS|FAIL with file/command evidence. Fail on nonexistent code paths, unsupported commands, stale versions, missing credentials, hidden architecture work, or excess ceremony. A failed reality gate halts the pipeline and returns to bee-planning. Dimensions are cacheable on the same terms as matrix rows.
3. **Feasibility matrix:** every blocking assumption gets a row — assumption | risk | proof required | evidence | result | **sources**. Sources are what the row was proven from: `{path, sha256}` for file evidence, `{command, output_sha}` for command evidence. Record them with `bee.mjs state validation-cache record --slice <n> --rows-file <f>` (the verb hashes each path itself). Accepted evidence only (below). Plausibility language is an automatic NOT READY. For multi-cell slices, the matrix includes a schedule row: `bee cells schedule` reports zero cycles and the expected wave shape — required evidence, not optional.
4. **Delta rule (slice 2+).** Re-prove only **stale rows and cells that are new**; carry fresh rows forward verbatim as `evidence: cached (slice N, sources unchanged)`. A row is stale when any source sha256 changed, the newest active decision id changed, or `sha256(plan.md)` changed — the same anchors, and the same "never a TTL" law, as `advisor_ref` under Gate 3 below. **A missing, unreadable, malformed, or partially-valid cache re-proves everything** (`degraded`/`revalidate: full`), as does any row storing no hashes: a cache problem always buys more validation, never less. Cached evidence is still Accepted Evidence and still auto-fails on plausibility language.
5. **Spikes** for unproven assumptions that can invalidate the current work.
6. **Plan-checker subagent** (adversarial) until structurally clean or escalated — on slice 2+, scoped to the new/changed cells and stale rows, not the whole frozen plan.
7. **Cold-pickup cell review** of the new or changed cells; fix every CRITICAL flag.
8. **Decide** using the decision vocabulary, then ask Gate 3.

Load `references/validation-reference.md` for report formats, repair routing, and the subagent prompts.

## Accepted Evidence

Existing implementation, file/API/type inspection, command output, build/typecheck/test result, official version/doc proof, runtime probe, or a `.bee/spikes/<feature>/` result. Evidence that is only "should work", "likely", "expected", or model knowledge → **NOT READY**.

**Static evidence for `tiny`/`small` (test-economy D5).** At these lanes, a file:line citation proving the code path exists and behaves as assumed — quoted, not paraphrased — is **sufficient** to pass the reality gate; it does not need a runtime spike. Reach for a spike only when the assumption touches an API, library, or technique with no precedent already in this repo (nothing to cite because nothing comparable has been built here yet). Precedent in-repo → cite it and move on; genuinely new technology or API surface → spike per the rules below. This does not loosen `standard`/`high-risk`, where the reality gate and feasibility matrix run at full weight regardless of lane.

## Spike Rules

- One spike answers exactly one yes/no question.
- Disposable code lives under `.bee/spikes/<feature>/`.
- **NO** → return to bee-planning with the failed assumption and the required plan change.
- **YES** → record the discovered constraints for planning and execution.
- Spike code never silently becomes production code.
- **Debug discipline (test-economy D5): hypothesis before repro, read before rerun.** Before writing any repro script, record the hypothesis in one line plus the file:line evidence from reading the code that grounds it — a repro script with no prior hypothesis is not a spike, it's a guess with extra steps. Cap the loop at **2** failed repro rounds: after the second wrong repro, stop running scripts and go back to reading/instrumenting the actual code path instead of trying a third guess blind. This is prose law — see `bee-executing/SKILL.md` for the machine-enforced proof-tier matrix; nothing here changes what capCell accepts.

**Verify scripts and any executable code NEVER go in `docs/history/`** (GitHub #17). `docs/history/` is the tech-agnostic knowledge layer — `.md` only (CONTEXT.md, plan.md, reports, walkthrough). A cell's `verify` is a runnable command; when it needs a multi-line harness, that script lives in **the project's own scripts** (committed with the product, so `verify` points at it) or, if disposable, in **`.bee/spikes/<feature>/`** — the disposable-code half of the one canonical scratch home (docs/specs/doctrine-layer.md R17). The write-guard denies a code-extension file (`.sh`, `.mjs`, `.py`, …) written under `docs/history/`, and also denies any scratch-shaped write landing in a tracked directory outside `.bee/tmp/`/`.bee/spikes/`.

## Plan Checker (adversarial)

Dispatch a subagent on the **`review` slot** (decision 0021 — `resolveTier(root, 'review', runtime)`, default opus on Claude, generation fallback; state the model explicitly; if the runtime cannot select per-agent models, cap its reads and output instead).
Codex has no per-agent subagent type (AO11), so the tier stays enforced as a read budget + output cap only.
The plan-checker is a **read-only gather**, never a cell — when the review slot is cli-shaped, resolve it with the purpose-scoped 4-arg form, `resolveTier(root, 'review', runtime, {for:'gather'})`, per the Delegation contract's cli gather branch (`bee-hive/references/routing-and-contracts.md`); a bare 3-arg resolve of a cli-shaped review slot now refuses (AO12/B1, plan 2A-ii). A model-shaped review slot is unaffected by purpose — dispatch it exactly as before — **in the background where the runtime supports it** (decision 0017): continue the spike/matrix/cell-review work while it runs; its findings block nothing until the Gate 3 presentation, which never happens with the checker still outstanding. It assumes the plan is flawed and verifies 5 dimensions: requirement/decision coverage, cell completeness, dependency correctness, key links, scope sanity. Every finding carries **BLOCKER** or **WARNING**. Maximum 3 structural-verification iterations; a BLOCKER still open after iteration 3 escalates to the user. Never attempt iteration 4.

**High-risk lane:** scale to a persona panel — coherence + feasibility lenses always, plus conditional lenses (security, product, scope-guardian) chosen by the diff of concerns. Dedupe findings, then synthesize into auto-fix vs present-for-decision buckets.

## Cell Review (cold pickup)

Dispatch the cell reviewer (`review` slot, decision 0021).
Could a worker with no session history pick each cell up cold? **CRITICAL** flags — assumed context, vague acceptance, scope overload, unproven feasibility, broken verify — must be fixed before approval. **MINOR** flags may ship with a recorded note.

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

Write the full machine report (reality gate, matrix, plan-checker findings, cell review, approval block) to `docs/history/<feature>/reports/validation-<slice>.md`. For `small`/`standard`/`high-risk`, invoke `bee-briefing` in refresh mode to patch the implement plan's Validation Plan section with the accepted evidence links (and to flip its `status` if a source changed), so the Gate 3 message links a current brief. Then present **only the human layer** in chat per the Gate Presentation Contract (template in the reference): what I'm about to do / why it's trustworthy / if it goes wrong / what you are deciding — in the user's language, jargon-free, implement plan + report linked — then ask verbatim: **"Feasibility validated. Approve execution?"** Optionally offer a cross-model second opinion first (agreement → mention it; disagreement → quote both positions; never auto-resolve). Approval covers the **current work only**; future slices return to planning and validating.

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
- spawning the plan-checker or cell reviewer for a tiny/small lane (their reality check lives inline in planning)
- accepting plausibility language as evidence
- carrying a row forward without hash-verified sources, or reading a degraded cache as permission to skip a proof rather than to re-prove it
- continuing after a NO spike because a workaround "probably works"
- running a 4th plan-checker iteration instead of escalating
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
| `references/validation-reference.md` | Report formats, repair routing, plan-checker and cell-reviewer prompts, approval block |
