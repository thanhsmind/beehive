# Stage: hive (`bee-hive`)

**Purpose** — The bootstrap meta-skill and router. It verifies onboarding, reads
runtime state, classifies the request into a lane/mode, routes to the next skill,
and presents and protects the three human approval gates.

**When it runs** — First, in every bee session, and again after any context
compaction. Re-entered whenever a routing or mode-gate decision is needed.

## Inputs
- The session preamble (phase, mode, feature, gate states, cell/PBI counts, the
  critical-patterns digest, recent decisions) — read it, don't re-fetch it.
- [`state.json`](../register.md#beestatejson), [`onboarding.json`](../register.md#beeonboardingjson),
  [`HANDOFF.json`](../register.md#beehandoffjson).
- `.bee/bin/bee status --json` — only when about to *route work*.
- `docs/knowledge/index.md` (critical patterns) or `docs/history/learnings/critical-patterns.md`.

## Outputs
- A routing decision (which stage skill to load next).
- Onboarding mutations (via `packages/bee/scripts/onboard_bee.mjs --apply`).
- Gate presentations. hive owns no feature artifacts of its own.

## Gate
Presents all three verbatim but structurally owns none — it is the presenter and
enforcer. Gate 1 "Decisions locked. Approve CONTEXT.md before planning?" · Gate 2
"Work shape is ready. Approve before current-work preparation?" — folding the old
standalone Gate 3 into the same approval, shape and execution together via
`bee state gate --merge` · Gate 4 (P1>0) "P1 findings block merge. Fix before
proceeding?" / (P1=0) "Review complete. Approve merge?".

## State touched
Reads [`state.json`](../register.md#beestatejson),
[`onboarding.json`](../register.md#beeonboardingjson),
[`HANDOFF.json`](../register.md#beehandoffjson),
[`config.json`](../register.md#beeconfigjson) (bypass level, CI/verify gate).
Writes onboarding state and gate approvals (`state gate`).

## Key rules
- **Gates are never skipped, batched, or self-approved** — including go mode and
  headless — except the opt-in `gate_bypass` level in `config.json`.
- **Classification is mechanical** — count risk flags and product files; never by
  vibe. Uncertainty resolves *downward* into more ceremony, never upward into less.
- **The hook is a safety net, not the authority** — an unblocked write is not an
  approved write. Route through hive *before* touching source, every time.
- **Green base before the first `cells claim`** — run `commands.verify` when it is
  cheap (it is the same command CI runs), read CI when it is not; red becomes a
  fix-first tiny cell, never a base to build on.
- **Multisession etiquette** — coordinate through lanes, claims, and holds, never
  around them. New feature work in an occupied checkout routes through
  `bee worktree new` / `bee worktree merge`; docs, tiny, and release work stay in
  the main checkout.

## Source
`skills/bee-hive/SKILL.md`
