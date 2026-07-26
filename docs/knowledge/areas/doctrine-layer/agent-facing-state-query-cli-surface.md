---
type: bee.area
title: Doctrine Layer — agent-facing state query CLI surface
description: "The three read-only query verbs bee exposes over its own state store, so agents stop grepping .bee/*.jsonl or importing internals — decisions active/search --cell/--feature, backlog findings --feature, and state scribing-run --show — plus the word-boundary discipline that makes the text-matching verbs correct."
timestamp: 2026-07-24
bee:
  id: doctrine-layer-agent-facing-state-query-cli-surface
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [5ca69717 state-query-surface B1 -- decisions --cell/--feature word-boundary token match not a structural field, e0d130c1 state-query-surface META -- a harness must own its own state-query contract]
  sources: [state-query-surface feature close 2026-07-24, .bee/bin/bee.mjs handleDecisionsActive/handleDecisionsSearch/handleBacklogFindings/scribing-run --show branch, .bee/bin/lib/decisions.mjs matchesWholeToken word-boundary helper]
  authoritative_for: "doctrine-layer: agent-facing state query CLI surface"
---

# Doctrine Layer — Agent-Facing State Query CLI Surface

## Purpose

Bee owns a contract for querying its own state. An agent that wants to know
what decisions touched a cell, what friction a feature has accumulated, or
when a feature last had its knowledge synced does not grep `.bee/*.jsonl` by
hand and does not import internal library modules to compute the answer
itself. It calls one of three CLI query verbs instead. The verbs exist
because a structure-blind text search over the raw store is silently wrong —
it looks like it worked and returns the wrong rows — and because grepping
internals every time a question comes up is a workaround for a missing verb,
not a habit to keep.

## Entry Points & Triggers

- `bee decisions active --cell <id>` / `bee decisions active --feature <slug>`
  — list currently-active decisions touching a cell or feature.
- `bee decisions search --cell <id>` / `bee decisions search --feature <slug>`
  — the same matching discipline over the full (not just active) decision
  history.
- `bee backlog findings --feature <slug> [--text <terms>]` — list friction/
  finding backlog rows recorded against a feature.
- `bee state scribing-run --show [--feature <slug>]` — read the most recent
  scribing-run stamp, overall or for one feature.

## Data Dictionary

- **decision/rationale/alternatives text** — the free-text fields on a
  `decide` event in the decisions store. There is no structural `cell` or
  `feature` field on a decide event; cell and feature identifiers live only
  as prose inside these three fields.
- **cell id / feature slug (as a query token)** — the string passed to
  `--cell` or `--feature`; matched as a whole token, never as a substring.
- **backlog finding row** — a friction or finding record in the backlog
  store, distinct from a product-backlog (pbi) row. Carries a `feature`
  field (structural, unlike decisions) plus `title`/`detail` free text.
- **scribing-run stamp** — the timestamp (and originating feature, when
  scoped) of the most recent scribing sync recorded for the workflow overall
  or for one feature.

## Behaviors & Operations

**Decisions active/search --cell / --feature.** Both verbs filter by
matching the given id or slug as a **whole token** inside the decision,
rationale, and alternatives text — a word-boundary text match, not a
structural field lookup, because a decide event carries no cell/feature
field at all. `--cell si-1` returns decisions whose text mentions "si-1" and
must **exclude** ones that only mention "si-10"; `--feature billing-export`
excludes text that only mentions "billing-export-v2". `active` scopes to
decisions still in force; `search` scopes to the full history.

**Backlog findings --feature [--text].** Lists friction/finding rows for a
feature, skipping product-backlog (pbi) rows entirely. The feature match is
a word-boundary match on the row's structural `feature` field (exact-modulo-
case, not substring — same collision-avoidance discipline as the decisions
verbs, applied to a real field this time rather than free text). The
optional `--text` filter is a substring match over the row's title and
detail. The verb reads **both** the legacy row-schema field name and the
current one for "what kind of row is this" — two schema generations coexist
in the backlog store, and a row counts as a finding if either the old or the
new field name says so.

**State scribing-run --show [--feature].** A read-only query returning the
most recent scribing-run stamp — overall, or for the named feature. `--show`
performs no write and does not advance the workflow phase: it is evaluated
above the write-path validation, so it never demands the write-only flags
(`--areas`, `--next-action`) a real scribing-run call requires.

## Actors & Access

Any agent operating in the bee workflow (orchestrator or worker) is the
caller. All three verbs are read-only queries — no gate, claim, or
reservation governs calling them.

## Business Rules

1. `--cell`/`--feature` on `decisions active`/`decisions search` is a
   word-boundary text match over decision/rationale/alternatives, never a
   structural field lookup: a decide event has no cell/feature field, so
   matching must exclude a token that is merely a substring of another
   (`si-1` must not match `si-10`) (D 5ca69717).
2. `backlog findings --feature` reads both the legacy and current
   row-schema field names for row kind, because two schema generations
   coexist in the backlog store; a row counts as a finding under either
   name, and `pbi` rows are always excluded first.
3. `state scribing-run --show` is read-only: it never writes to the
   scribing ledger and never advances the phase, and it is validated above
   the write-path checks so it never requires write-only flags.
4. **Meta rule:** a structure-blind text search over the raw `.bee/*.jsonl`
   store is silently wrong — it returns substring collisions without
   signaling an error — so the harness, not each agent ad hoc, owns the
   query contract over its own state (D e0d130c1).

## Edge Cases Settled

- `--cell si-1` must exclude a decision that mentions only `si-10` in its
  text, and `--feature billing-export` must exclude one that mentions only
  `billing-export-v2` — both are word-boundary exclusions, not substring
  hits.
- A backlog row tagged under either the legacy `kind` field or the current
  `type` field is recognized as a finding; a row explicitly kinded `pbi` is
  never treated as a finding under either schema.
- `state scribing-run --show --feature <slug>` with no scribing run ever
  recorded for that feature returns an explicit "no run recorded" result
  rather than falling back to the global stamp.

## Open Gaps

None identified for this surface as of the state-query-surface close.

## Pointers (implementation)

- The three verbs live in the bee CLI dispatcher (`bee.mjs`) and its command
  registry, alongside the workflow's other query and mutation verbs.
- Word-boundary matching form: `(?<![\w-])<id>(?![\w-])` (case-insensitive),
  applied identically wherever a cell id or feature slug is matched as a
  whole token rather than a substring.
