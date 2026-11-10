---
type: bee.area
title: "Hook Runtime — advisories, session-stop output, and the one deliberate turn-control exception"
description: "Why close-time, compaction and child-stop checkpoints only ever inform, what a session-stop handler is allowed to emit, and the single scoped exception — the gate-bypass net — that turns the close-time checkpoint into a loop-guarded block verdict on purpose."
timestamp: 2026-07-28
bee:
  id: hook-runtime-advisories-and-turn-control
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-runtime-parity D1, D2", "4c1c5921 (GitHub #18 — the gate-bypass net mechanized at runtime as a loop-guarded turn-control block)", "validation-diet D1-D3 (docs/history/validation-diet/CONTEXT.md, 2026-07-28)", "knowledge-first-lean-decisions D1, D2 (docs/history/knowledge-first-lean-decisions/plan.md, 2026-11-07)"]
  sources: ["codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "post-advisor-hardening cells pah-1/pah-3 (onboarding-generator drift check + B15 consult instruction, 2026-07-18)", "docs/specs/hook-runtime.md#B2", "docs/specs/hook-runtime.md#B10", "docs/specs/hook-runtime.md#B15", "docs/specs/hook-runtime.md#R4", "docs/specs/hook-runtime.md#R10", "docs/specs/hook-runtime.md#R14", "docs/specs/hook-runtime.md#P4", "docs/specs/hook-runtime.md#P5", "codex-loop-p0 cell clp-1 (the prompt reminder threads sessionId and stops reporting the on-demand review gate outside a review session; trace in `.bee/cells/`, 2026-07-23)", "loop-seams-1112 cell ls-1 (advisor issue #54 — both surfaces, terminal owes no gate, per-session dedup; trace in `.bee/cells/`, 2026-07-23)", "validation-diet cell vd-1 (trace in .bee/cells/, report docs/history/validation-diet/reports/vd-1.md, 2026-07-28 — the bypass net's fire matrix narrowed to the single phase now carrying the merged approval, since the standalone feasibility phase it used to also cover is retired)", "knowledge-first-lean-decisions cell kbc-1 (trace in .bee/cells/, 2026-11-07 — the knowledge-before-code nudge)"]
  authoritative_for: "hook-runtime: the advisory contract and the gate-bypass turn-control exception"
  anchors: [packages/bee/LEADER.md, packages/, packages/bee-rs/crates/bee/src/, skills/, packages/bee-rs/crates/bee/src/hooks/write_guard/knowledge_first.rs, packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, packages/bee-rs/crates/bee/tests/knowledge_first_nudge.rs, packages/bee-rs/crates/bee/src/hooks/adapter.rs, packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs]
  verified_at: 2026-11-10
  verified_commit: 471430df16c5b9bc6af2a3f891ee1180729d1916
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
prose-only: the level-aware auto-approval rule lived in the planning skill
(and is machine-checked green there), but nothing caught the assistant
when it skipped that rule and stopped at an approval gate anyway — the close-time
checkpoint only warned. This is the "an invariant left in prose WILL be bypassed;
mechanize it" pattern. The close-time checkpoint now emits a **turn-control block
verdict** (the deliberate exception to B2) — forcing the turn to continue — when
ALL hold: the event is the session-stop event itself (never compaction, never a
child-stop, never a missing/ambiguous event); the active phase is the one phase
whose stop is a pending *approval* gate — shape and execution approved together
as a single question, with no separate later stop for either half — and that
gate is still pending; and the active bypass level covers that gate for the
lane (`full`/`total` cover every lane; `normal` covers only the non-hard-gate
lanes — a `normal`-lane high-risk change still stops; `off` never fires). The
exploring/Gate-1 phase is **excluded on purpose**: under the highest level a
genuine *information* question still stops for the human — only the
rubber-stamp approval gate is mechanized, and there is no longer a separate
pre-code feasibility phase to mechanize a stop for. The block carries an
instruction to record the approval, log the audit decision, and proceed — and
when the pending gate covers high-risk work, the instruction additionally
names the adviser-consult prerequisite first (run the configured adviser
read-only, record the consult, then approve), so the mechanized instruction
can no longer steer the assistant into the consult-precondition refusal
uninformed; the same gate's instruction is the plain version whenever the
change is not high-risk (cell pah-3, 2026-07-18; validation-diet D1-D3,
2026-07-28). **Loop-guard:** it blocks at most
once per (session, phase, gate, level); an immediate re-stop at the same gate is
deduped and degrades to the ordinary advisory, so a turn that genuinely cannot
proceed is never trapped in a loop. Fail-open is unchanged: any internal failure
falls through to the advisory path with a visible log, never a crash.

**B16 — A leader's first code read in a turn gets one knowledge-first nudge
(knowledge-first-lean-decisions D1, D2).** The rule lives in one place, the rule
block `agents-knowledge-before-code` in `packages/bee/LEADER.md` § Start a
session; the hook only points to it. On a leader's PreToolUse, the write-guard
checks whether the call is a code read: Read, Glob or Grep whose path is under
`packages/` or `packages/bee-rs/crates/bee/src/` (formerly src/) or is a source file, or a Bash command (`rg`, `grep`,
`cat`, `sed`, `head`, `tail`, `fd`, `find`, `less`, `bat`) whose path argument
is. Anything under `docs/`, `.bee/` or `skills/`, and any `*.md` file, is not a
code read. When the session's current turn has neither a knowledge lookup nor a
nudge yet, the guard emits one advisory — `additionalContext` and
`systemMessage`, no `permissionDecision` — naming the rule id and the command
`.bee/bin/bee knowledge search --text "<what you need>"`, and marks the turn
nudged. A Bash command that runs `bee knowledge search` or `bee knowledge
context` marks the turn searched instead, and the rest of the turn stays silent.
The nudge never denies and fires at most once per turn. The turn state is a
per-session file under `.bee/cache/`; the prompt-context hook clears it at every
UserPromptSubmit, so the next turn re-arms the nudge, and a session with no state
counts as a fresh turn. A worker never hears it: a herded pane
(`BEE_HERDING_WORKER`) never reaches this path, and a payload carrying a
non-empty `agent_id` or `agent_type` (a native subagent) is skipped.

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

## Edge Cases Settled

- On Codex, a parent session `Stop` event with a persisted transcript updates
  repository `state.json.waiting_on` to `turn-end` (stored in repository state,
  not per-session records) and transitions activity state to `idle`. Child subagent
  completion (`SubagentStop`) is isolated from the parent turn and does not alter the
  parent session's waiting state or activity journal.

## Pointers (implementation)

- Knowledge-first nudge (B16): `packages/bee-rs/crates/bee/src/hooks/write_guard/knowledge_first.rs`
  (`observe`, `reset_turn`), called from `run_native_with_roots` in
  `.../write_guard/main.rs`; the turn reset is in
  `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs`. Suite:
  `packages/bee-rs/crates/bee/tests/knowledge_first_nudge.rs`.

- Shared adapter: `packages/bee-rs/crates/bee/src/hooks/adapter.rs` (formerly packages/bee/hooks/adapter.mjs) (`encodeAdvisory`; `encodeBlock` — the
  deliberate turn-control inverse used ONLY by the gate-bypass net); the eight
  handlers `packages/bee/hooks/bee-*.mjs`, including the paired Codex native-subagent audit
  handler and its vendored mirror.

- Gate-bypass net (B15, R14): `maybeBypassBlock` in `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs` (formerly packages/bee/hooks/bee-session-close.mjs)
  (fire matrix `PHASE_GATE` + `levelCoversGate`; loop-guard via
  `shouldInject`/`markInjected` in packages/bee/lib/inject.mjs (retired with the Node runtime, 2026-08-02)
  keyed `sessionId:phase:gate:level`; level via `bypassLevel` in
  `.../lib/state.mjs`). Suite: hooks/test_bypass_stop_net.mjs (retired with the Node runtime, 2026-08-02).
