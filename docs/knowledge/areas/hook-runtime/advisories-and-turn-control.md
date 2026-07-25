---
type: bee.area
title: "Hook Runtime — advisories, session-stop output, and the one deliberate turn-control exception"
description: "Why close-time, compaction and child-stop checkpoints only ever inform, what a session-stop handler is allowed to emit, and the single scoped exception — the gate-bypass net — that turns the close-time checkpoint into a loop-guarded block verdict on purpose."
timestamp: 2026-07-23
bee:
  id: hook-runtime-advisories-and-turn-control
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-runtime-parity D1, D2", "4c1c5921 (GitHub #18 — the gate-bypass net mechanized at runtime as a loop-guarded turn-control block)"]
  sources: ["codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "post-advisor-hardening cells pah-1/pah-3 (onboarding-generator drift check + B15 consult instruction, 2026-07-18)", "docs/specs/hook-runtime.md#B2", "docs/specs/hook-runtime.md#B10", "docs/specs/hook-runtime.md#B15", "docs/specs/hook-runtime.md#R4", "docs/specs/hook-runtime.md#R10", "docs/specs/hook-runtime.md#R14", "docs/specs/hook-runtime.md#P4", "docs/specs/hook-runtime.md#P5", "codex-loop-p0 cell clp-1 (the prompt reminder threads sessionId and stops reporting the on-demand review gate outside a review session; trace in `.bee/cells/`, 2026-07-23)", "loop-seams-1112 cell ls-1 (advisor issue #54 — both surfaces, terminal owes no gate, per-session dedup; trace in `.bee/cells/`, 2026-07-23)"]
  authoritative_for: "hook-runtime: the advisory contract and the gate-bypass turn-control exception"
---

# Hook Runtime — advisories, session-stop output, and the one deliberate turn-control exception

A checkpoint that fires when a turn is ending has a dangerous power: emitting a
turn-control verdict there would resume a stopped child or loop the main turn.
So the rule is that those checkpoints inform and nothing more — and the rule is
stated with its exception attached, because the exception is deliberate,
scoped, and loop-guarded rather than an oversight.

**`R14` here is the surviving half of a duplicated id.** The source carried
`R14` twice; every citation of `hook-runtime R14` — this area's own `R4` and
`R10`, the routing-and-contracts reference, and decision `4c1c5921` — means this
rule, so it keeps the id. The other rule was renumbered `R14a` and lives in
[`write-guard-request-shapes.md`](write-guard-request-shapes.md).

## Data Dictionary

| Element | Meaning |
|---|---|
| advisory | A checkpoint message that informs the assistant without blocking or continuing any turn — delivered as a parseable structured message, never as a turn-control verdict. |

## Behaviors & Operations

**B2 — Advisories never steer the conversation.** Close-time, compaction, and
child-stop checkpoints only inform: their output is a structured message the
runtime displays/parses. They never emit a turn-control verdict, because on
those events a verdict would resume a stopped child or loop the main turn. The
**one** deliberate exception is the gate-bypass net (B15), which is turn-control
by design on the close-time checkpoint only — there, looping the main turn is
exactly the intended continuation.

**B10 — Session-stop handlers speak JSON or say nothing.** Both handlers
wired to the session-stop checkpoint exit success. Whichever of them produces
output at all produces a single non-empty payload that parses as JSON
carrying a human-readable summary field, and never a turn-control block
verdict — consistent with B2 (advisories never steer the conversation). A
handler with nothing to report stays completely silent rather than emit a
placeholder (codex-runtime-parity cell 6b).

**B15 — The gate-bypass net mechanizes "zero stops" at the close-time
checkpoint (GitHub #18).** Honoring the gate-bypass autopilot was previously
prose-only: the level-aware auto-approval rule lived in the planning/validating
skills (and is machine-checked green there), but nothing caught the assistant
when it skipped that rule and stopped at an approval gate anyway — the close-time
checkpoint only warned. This is the "an invariant left in prose WILL be bypassed;
mechanize it" pattern. The close-time checkpoint now emits a **turn-control block
verdict** (the deliberate exception to B2) — forcing the turn to continue — when
ALL hold: the event is the session-stop event itself (never compaction, never a
child-stop, never a missing/ambiguous event); the active phase is one whose
stop is an *approval* gate (plan-shaping → Gate 2, feasibility → Gate 3) with
that gate still pending; and the active bypass level covers that gate for the
lane (`full`/`total` cover every lane; `normal` covers only the non-hard-gate
lanes — a `normal`-lane high-risk change still stops; `off` never fires). The
exploring/Gate-1 phase is **excluded on purpose**: under the highest level a
genuine *information* question still stops for the human — only rubber-stamp
approval gates are mechanized. The block carries an instruction to record the
approval, log the audit decision, and proceed — and when the pending gate is
execution approval on high-risk work, the instruction additionally names the
adviser-consult prerequisite first (run the configured adviser read-only,
record the consult, then approve), so the mechanized instruction can no longer
steer the assistant into the consult-precondition refusal uninformed; every
other gate's instruction is unchanged (cell pah-3, 2026-07-18). **Loop-guard:** it blocks at most
once per (session, phase, gate, level); an immediate re-stop at the same gate is
deduped and degrades to the ordinary advisory, so a turn that genuinely cannot
proceed is never trapped in a loop. Fail-open is unchanged: any internal failure
falls through to the advisory path with a visible log, never a crash.

## Business Rules

- R3a — The prompt reminder speaks for the ACTING SESSION, and reports only
  gates that are genuinely open. Two rules the reminder must hold, each a fixed
  loop-driver: (1) it resolves the pipeline with the caller's own `sessionId`, so
  a session bound to a feature lane reports *its lane's* phase and gate, never the
  default record's — dropping the id made a bound session read the default (idle)
  state and pulled it back toward routing every turn. (2) It reports the on-demand
  **review** gate as pending only inside a review session (phase `reviewing`);
  outside one, review is user-invoked and permanently unapproved, so walking it
  printed a false "gate pending: review" on every turn — a phantom
  unfinished-workflow signal. The reminder walks the pre-execution gates
  (context/shape/execution) always, and review only when a review session is live.

  Two corollaries the first pass missed, both fixed after an external review: the
  rule binds **both surfaces** — the startup/compaction preamble as well as the
  per-turn reminder, and the preamble is the more dangerous one because it is what
  a long session re-reads to re-orient. And a **terminal record owes no gate at
  all**: at idle or after a close there is no feature, so naming any gate pending
  announces an approval owed for work that does not exist. Terminal states also
  never emit a re-route order — a routing instruction on an idle record reads as
  "there is workflow waiting" when there is none, which is the loop it creates.
  Finally the reminder dedup is keyed **per session**, not per repo: a repo-global
  key let two sessions on one checkout invalidate each other and collapsed the
  throttle to nearly every turn.

- R4 — Advisory events never emit turn-control verdicts (codex-runtime-parity D2),
  with the single scoped exception of the gate-bypass net on the session-stop
  event (B15, R14) — turn-control there is the intended, loop-guarded behavior.

- R10 — Every session-stop handler exits success; any non-empty output from
  it parses as a single JSON object carrying a summary field and — except for
  the gate-bypass net (R14) — never a block verdict (codex-runtime-parity cell 6b).

- R14 — The session-stop handler emits a block verdict ONLY for the gate-bypass
  net (B15): only on the session-stop event, only in an approval-gate phase whose
  gate the active bypass level covers and is still pending, and at most once per
  (session, phase, gate, level) — an immediate same-key re-stop degrades to the
  ordinary advisory. It never fires on compaction, child-stop, a missing event,
  the exploring/Gate-1 phase, or a `normal`-lane hard-gate change (GitHub #18).

## Pointers (implementation)

- Shared adapter: `packages/bee/hooks/adapter.mjs` (`encodeAdvisory`; `encodeBlock` — the
  deliberate turn-control inverse used ONLY by the gate-bypass net); the eight
  handlers `packages/bee/hooks/bee-*.mjs`, including the paired Codex native-subagent audit
  handler and its vendored mirror.

- Gate-bypass net (B15, R14): `maybeBypassBlock` in `packages/bee/hooks/bee-session-close.mjs`
  (fire matrix `PHASE_GATE` + `levelCoversGate`; loop-guard via
  `shouldInject`/`markInjected` in `packages/bee/lib/inject.mjs`
  keyed `sessionId:phase:gate:level`; level via `bypassLevel` in
  `.../lib/state.mjs`). Suite: `hooks/test_bypass_stop_net.mjs`.
