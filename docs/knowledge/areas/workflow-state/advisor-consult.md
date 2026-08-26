---
type: bee.area
title: "Workflow State — the worker's adviser consult, its budget, and the cost pattern behind it"
description: "The one place a dispatched worker may reach for a second opinion: who the adviser is (configuration alone decides), when a consult may fire (after a first serious failed verification, twice per claim), what advice may never do, and how a workspace with no adviser behaves identically to one before advisers existed."
timestamp: 2026-07-22
bee:
  id: workflow-state-advisor-consult
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [de967733-00c8-48b3-b154-68397faf7b5f (cost pattern; advisor config tolerance; refines decision 0015; amended by advisor D1 — worker-level on-failure consult), "72f3d6dd (AO5 config is the authority — the adviser strength ladder was removed, cells ao-2b-1/ao-2b-2 2026-07-17)", advisor D1-D3 (docs/history/advisor/CONTEXT.md; logged 3a794918/6841bfcb/34514a8b)]
  sources: ["advisor cells adv-1..adv-3 (traces in .bee/cells/, reports docs/history/advisor/reports/, 2026-07-13)", "advisor-and-orchestration Slice 2B cells ao-2b-1/ao-2b-2 (AO5 adviser form — ladder and ceiling-skip removed, same-model no-op only, 2026-07-17)", "fanout-delegation D1 (cells fanout-1/fanout-4, 2026-07-12 — advisor config tolerance)", "docs/specs/workflow-state.md#B9", "docs/specs/workflow-state.md#R7", "docs/specs/workflow-state.md#R8", "docs/specs/workflow-state.md#R14", "docs/specs/workflow-state.md#R15", "docs/specs/workflow-state.md#R16", "docs/specs/workflow-state.md#R24", "docs/specs/workflow-state.md#E3", "docs/specs/workflow-state.md#E12", "docs/specs/workflow-state.md#E13", "docs/specs/workflow-state.md#P7", "docs/specs/workflow-state.md#P8", "docs/specs/workflow-state.md#P11"]
  authoritative_for: "workflow-state: the worker adviser consult loop, its budget, and adviser configuration tolerance"
---

# Workflow State — the worker's adviser consult, its budget, and the cost pattern behind it

A stuck worker is the one actor in the workflow allowed to ask for help
mid-turn — and the whole design is about keeping that door narrow. Configuration
alone says who the adviser is; an objective failure, never a self-assessment,
says when the door opens; a budget says how often; and advice, however good,
never approves anything.

## Behaviors & Operations

**B9 — A stuck worker may consult a configured adviser, inside its own turn.**
When the dispatcher assigns a work unit to a worker, it first resolves the
configured adviser. The configured adviser IS the adviser — no family test, no
strength test, no self-judged skip (72f3d6dd/AO5); the dispatch omits it only
when the adviser resolves to literally the same model the worker runs on (the
one honest no-op — an external-command adviser is never the same model, so it
is always offered, workers on the session's strongest tier included). If an
adviser is offered, the dispatch names it and exactly how to reach it. The worker may consult
only after its first serious failed verification attempt, sending an evidence
bundle: the exact check it ran, the failing output, its diagnosis, the relevant
excerpts, and where the locked feature decisions live — never secrets or
credential values. The canonical loop: first failure → consult → advised retry
→ (second failure) → one follow-up consult → final retry → blocked, with every
consult summarized in the worker's report (count, adviser identity, one-line
question/answer digest). Each consult also lands one recognizable, attributable
entry in the dispatch audit log naming the work unit and the adviser. What
observers see: a worker with no adviser named in its dispatch behaves exactly
as before (two failures → blocked); the orchestrator's rescue ladder is
unchanged except that it knows an arriving blocked unit already spent its
consult budget.

## Business Rules

- R7 — The workflow runs one cost pattern: the session's own model
  orchestrates every phase and is always the ceiling tier, never a configured
  value; the cheaper configured tiers (extraction, generation, review) take
  retrieval, implementation, and review work; steps that are mostly gathering
  content dispatch down-tier and return digests rather than raw content
  (decision de967733; the ceiling-is-the-session-model principle it refines
  stands unchanged, decision 0015). Amended, not reversed, by advisor D1
  (2026-07-13): a dispatched worker that fails its first serious verification
  attempt may consult a configured stronger adviser from inside its own turn —
  advice only, on failure only; the orchestration pattern and the retired
  gate-time advisory mode stay exactly as this rule states them.
- R8 — A workflow configuration file that still carries the retired advisor
  setting loads successfully: the setting is stripped from the parsed view
  and surfaced as one warning by both the status command and the onboarding
  report; it never errors, and the status display renders no advisor line
  (decision de967733).
- R14 — The adviser is a per-runtime configured role beside the reviewer role;
  it may name a different provider. Unset, invalid, or not stronger than the
  worker means no adviser: nothing is silently substituted, and no fallback to
  another configured role ever happens (advisor D2).
- R15 — Consult triggers are objective, never self-assessed: only after the
  first serious failed verification attempt, at most two consults per claim,
  and blocks the adviser has no standing to resolve — an ambiguous work unit,
  unmet dependencies, an architectural change, a software installation, a
  conflict with a locked decision — block immediately without consulting
  (advisor D3).
- R16 — Advice is advice: it never approves gates or installations, never
  widens a worker's file scope, and never substitutes for fresh verification
  output; advice that contradicts a locked decision turns into a block citing
  both the decision and the advice (advisor D1/A1; goal-check unchanged).
- R24 — A configured external assistant whose reply is free prose is proven live
  only by a **known-answer probe** — a question whose correct answer is already
  known — never by its exit status alone. The command string is a contract with
  an argument grammar: whether the assistant can even *receive* the question is
  part of what must be validated. An assistant that exits zero while never having
  been handed the prompt looks healthy and is silently useless, forever.

## Edge Cases Settled

- A configuration file carrying the retired advisor setting → parses
  normally with the setting stripped from the parsed view; the status
  command and the onboarding report each surface one identical warning line
  naming it safe to delete, never an error (decision de967733).
- A consult attempt that fails at the transport level (the adviser is
  unreachable, errors, or hangs past the external-work timeout discipline) is
  not advice: it spends at most one budget slot in total and is never retried
  in a storm; the worker proceeds toward blocked exactly as it would without an
  adviser.
- A workflow whose configuration names no adviser dispatches byte-identical
  worker instructions to before the adviser existed.

## Pointers (implementation)

- Cost pattern / role resolution: `resolve_role`, `resolve_tier` and
  `resolve_advisor` in
  `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` (the model-role
  split retired `MODEL_TIERS`/`CONFIGURABLE_TIERS` and the Node
  `state.mjs` reader; `ceiling` is still never configured — the validator
  now names the key on sight — and any role `models.<runtime>` carries is
  configurable).
- Adviser (worker consult): `resolveAdvisor` + `MODEL_NORMALIZE_SLOTS` in
  `packages/bee/lib/state.mjs` (byte-mirrored to `.bee/bin/lib/`);
  slot `models.<runtime>.advisor` in `.bee/config.json`; worker protocol in
  `skills/bee-swarming/references/worker-details.md` ("Advisor consult in
  full"); dispatch-time
  same-model no-op + Advisor line (AO5 form, ladder removed by ao-2b-1) in
  `skills/bee-swarming/SKILL.md`
  and `references/swarming-reference.md`; consult record = `advisor-consult
  <cell-id>: <advisor>` description prefix in `.bee/logs/dispatch.jsonl`.
  Evidence: commits 14e0e1b, 68d3a0d, 33aaad7; traces `.bee/cells/adv-{1,2,3}.json`;
  transport proofs `docs/history/advisor/reports/validation-advisor-consult.md`.
- Advisor config tolerance: `STALE_ADVISOR_KEY_WARNING` (copy names the
  top-level key; the nested `models.<runtime>.advisor` slot is separate and
  valid), `hasStaleAdvisorKey`
  in `packages/bee/lib/state.mjs` (byte-mirrored to
  `.bee/bin/lib/state.mjs`); surfaced by `the bee binary`
  (`status` group) and `packages/bee/scripts/bee onboard` (`staleAdvisorNotices`).
  Evidence: fanout-delegation cells fanout-1/fanout-4 (commits 0056eda,
  79d96df).
