---
type: bee.area
title: Hook Runtime — the pre-spawn dispatch guard
description: "How a helper dispatch's declared tier and explicit model choice are judged together against configuration alone, which pinned helper type each work tier must ride, and why every evaluated dispatch — allowed or refused — leaves an honest audit line."
timestamp: 2026-08-03
bee:
  id: hook-runtime-dispatch-guard
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [0023, 72f3d6dd (AO5 config is the authority — tier/model agreement and membership at dispatch), "codex-native-transport D3-D5 (3ceba8f5, D3a c0cba64e, Δ2-amended 760e9b05)"]
  sources: ["advisor-and-orchestration Slice 2A-iii cell ao-2aiii-1 (declared-tier-first dispatch guard, 12 verification rows, 2026-07-17)", "advisor-and-orchestration Slice 3B cells ao-3b-1/ao-3b-2 (config-rendered pinned helper types + flat agents sync + generic-type dispatch refusal + drift advisory, 2026-07-17)", "codex-native-transport cell cnt-7 (Claude model-param guard allowlist folds a configured model-shaped adviser's own model, closing a live adviser-dispatch refusal, allow-only-widening; trace in .bee/cells/, report docs/history/codex-native-transport/reports/cnt-7.md, 2026-07-19)", "docs/specs/hook-runtime.md#B16", "docs/specs/hook-runtime.md#B18", "docs/specs/hook-runtime.md#R5", "docs/specs/hook-runtime.md#E17", "docs/specs/hook-runtime.md#P18"]
  authoritative_for: "hook-runtime: pre-spawn dispatch judgement of tier, model, and helper type"
---

# Hook Runtime — the pre-spawn dispatch guard

A dispatch that spawns a helper is the one moment where a stated intention (a
tier) and a concrete choice (a model) can disagree without anyone noticing
afterwards. The guard's job is to make that disagreement impossible to ship
silently — judging both against configuration, which is the only authority
here, and writing one audit line per evaluated dispatch so a refusal can never
read afterwards as a legitimate run.

## Behaviors & Operations

**B16 — The pre-spawn dispatch guard reads the declared tier before judging the
explicit model choice.** A helper dispatch may carry a declared tier, an
explicit model choice, both, or neither; the guard judges them together, in
this order, and configuration is the sole authority throughout — there is no
built-in list of acceptable models.
- *Both present:* they must agree. A tier configured to a model requires the
  explicit choice to equal exactly that model; disagreement is **repaired to
  the configured model** (B16a — it was refused with that model named, until
  2026-08-03). A tier that names no model
  (the session-inherit tier, a budget tier, an external-command tier) must not
  carry an explicit model choice at all — the label would enter the audit
  record while the dispatch actually ran on the choice.
- *Choice only:* the model must be one of the models configured across the
  runtime's tier slots. On Claude, that configured set also includes a
  configured model-shaped adviser's own model — an adviser-kind dispatch is
  judged against the same resolved name the adviser protocol already offers,
  not left out of the allowlist just because the adviser slot is not itself a
  tier slot; a cli-shaped, native, or unconfigured adviser contributes nothing
  to the set. This can only turn a refusal into an allowance, never the
  reverse, and it closed a live gap where a correctly-shaped adviser dispatch
  was still refused as unconfigured (codex-native-transport cnt-7). An
  unconfigured name is refused, and the corrective message teaches every
  legitimate route: the configured models, the session-model marker for a
  dispatch meant to run at the session's own model, and adding the model to a
  configured tier slot. A workspace with no configured tiers is not checked
  (fail-open, unchanged behavior).
- *Tier only:* tiers resolving to a model, a budget, or session-inherit are
  permitted as before. A tier backed by an external command is refused — an
  in-family helper cannot *be* the external command — and the corrective
  message routes to the external-executor gather path without ever naming a
  model that does not exist.
- *Neither:* refused — unless the helper type itself names a tier (B16a). The
  corrective message checks how the default tier is configured first, so it
  never instructs the actor to pass a nonexistent model, and it teaches the
  helper-type route before the marker route.
What each actor observes: the assistant receives the corrective message and can
self-correct on the next attempt; the audit log gains one line per evaluated
dispatch whose transport label states *why* a refusal happened (tier/choice
disagreement, unconfigured choice, external-command tier, bare) — a refused or
misdeclared dispatch can no longer appear in the audit trail as a legitimate
one. Every internal failure of the guard itself fails open.

**B16a — A declaration the guard can derive is never asked for, and a
disagreement config already settles is repaired, not refused.** Three of the
four commonest refusals were the guard demanding something it could work out
itself, which put the choice back on the actor least able to get it right — the
caller — and charged a round trip for every guess.
- *A dispatch that names a rendered helper type and nothing else* declares its
  tier by that fact: each helper file is generated from exactly one tier's
  configured model, so the type IS the tier. Permitted, tier recorded, no
  correction event. This is purely additive — it decides only cases that
  previously had nothing to go on, so no dispatch that passes today changes
  its verdict, and an explicit model choice is still read first.
- *A declared work tier riding the catch-all helper type* is repaired: the
  request is rewritten to that tier's pinned type on a replacement copy and
  proceeds. The tier was already stated; which file carries it is the guard's
  own lookup.
- *A declared tier and an explicit model that disagree* is repaired to the
  tier's configured model — the same authority order the refusal was already
  enforcing (config wins), applied instead of announced.
- What stays refused: a wholly bare dispatch, an unconfigured model name, an
  explicit model on a tier that names none, and any tier resolving to an
  external command. Each is either unknowable or genuinely ambiguous.
- Every repair is announced twice — to the actor next to the tool call, to the
  human as a one-line note — and the audit line's transport label says a
  repair happened and records the value that will actually run. A repair
  carries **no permission verdict**: correcting a field is not approving the
  call (hook-runtime R23). Observed rate before this: 126 refusals in 399
  dispatches, 83% of them the catch-all-type pairing (2026-08-03).

**B18 — Work-tier dispatches ride pinned helper types, not the catch-all.**
Three pinned helper definitions — a gather worker, an extraction worker, and a
review worker — are rendered into the project at onboarding **from the
configured tier models** (never hand-pinned: configuration stays the sole
authority, so re-rendering follows a config change). They are delivered by a
flat managed-file step with its own version marker, deliberately outside the
skill-root sync (a skills-root entry would break onboarding's version preflight
permanently). A tier slot backed by an external command or left empty renders
no file (a helper type must name a real model), and the second runtime gets
none at all — it has no per-helper model selection, a documented asymmetry.
The dispatch guard enforces the pairing: a dispatch declaring a work tier
(gather/extraction/review) while naming the **catch-all generic helper type**
is corrected to that tier's pinned type and proceeds (per B16a; it was refused
with a corrective message until 2026-08-03). The session-model
tier is exempt — it has no pinned type by definition. Drift between a rendered
helper file's model and the configured slot is surfaced as an **advisory** by
the status snapshot and the configuration check (never a refusal — the
dispatch-time agreement rules already protect the dispatch itself). No claim
is made that any of this reduces cost; it makes the tier decision auditable.

## Business Rules

- R6 — A guard refuses only what it cannot derive or resolve. Where the
  request already contains the answer (a helper type that stands for exactly
  one tier) or where an existing decision already names the winner (config over
  an explicit model choice), the guard supplies or corrects it and proceeds,
  announced. Refusal is reserved for the unknowable and the genuinely
  ambiguous — a rule that hands a choice back to whoever just got it wrong
  buys a round trip and a second guess, not safety (2026-08-03).

- R5 — Every dispatch of a subagent carries an explicit model-tier transport
  that **agrees with configuration**, and every evaluated dispatch — allowed or
  refused — is audit-logged with an honest transport label (decision 0023;
  strengthened per 72f3d6dd/AO5 "config is the authority": a declared tier is
  read before the explicit model choice is judged, per B16; the audit
  checkpoint is an allowed difference — it exists only where the runtime
  exposes dispatch).

## Edge Cases Settled

- The second-runtime branch of the anchored tier-marker check alone
  recognizes an advisor role token in addition to the existing tier names
  (the first runtime's branch is byte-unchanged); before this, a confirmed
  native-override advisor payload would have been wrongly denied by the guard
  as unmarked (codex-native-transport D5).

## Pointers (implementation)

- The whole judgement: `evaluate_claude_dispatch` in
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs`, in verdict order —
  pinned-type repair, tier+model repair, model-only membership, marker-only,
  helper-type tier inference (`tier_for_pinned_type`), bare refusal. The
  repair emission is `repair_stdout` (stdout JSON `hookSpecificOutput.
  updatedInput`, no `permissionDecision`, exit 0); refusals stay exit 2 +
  stderr. Tests: `a_rendered_bee_agent_type_declares_its_own_tier`,
  `pinned_type_rule`, `marker_plus_param_agreement_rules`,
  `an_inferred_cli_tier_is_still_refused`.
- Claude model-param allowlist advisor fold (B16, "Choice only"):
  `configured_model_set`, folding the resolved advisor model into the union.
  Evidence: `.bee/cells/cnt-7.json`,
  `docs/history/codex-native-transport/reports/cnt-7.md`.
