---
name: bee-scribing
description: >-
  Keep technology-agnostic BA specs of every area current, so a human understands the system without the code and an agent can rebuild it on another stack. SELF-TRIGGERING: invoke this yourself, unprompted, the moment any discussion-test-adjust loop settles a rule, behavior, or value — the user should never have to ask for knowledge to be recorded. Also use when execution completes (chain), when the user asks to document a screen/API/job/area, or when a legacy area has code but no spec.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads cell traces and logs decisions via the vendored .bee/bin helpers.
---

# Scribing (scribe bees)

Scribing is bee's BA — it owns the state layer. Rules stated bare — decision IDs: `references/provenance.md`; quoted headings resolve in `references/scribing-reference.md`.

An **area is domain-general**: a screen, API, job, integration, pipeline, CLI command, or process — any unit with observable behavior that outlives features. Code is the implementation; the spec is the *meaning*, surviving a full rewrite on a different stack.

**Where meaning is written: `bundleMode(root)`** (`.bee/bin/lib/knowledge.mjs`) — true only when `docs/knowledge/` exists AND a concept in it parses (a `.gitkeep`-only dir is NOT a bundle). Bundle → `docs/knowledge/areas/<area>/*.md`, one `bee.area` concept per subject. No bundle → `docs/specs/<area>.md` + `system-overview.md` + `visuals/` + `reading-map.md`. Same rebuild bar, tech-agnostic rule, nine sections, modes, never-invent in both — only layout/frontmatter mechanics differ. Full routing (`scribingTarget()`, refusal answers, anti-fork gate, `emitFrontmatter`): `references/scribing-reference.md ("Map Deltas", "Bundle-mode gate and frontmatter")`.

**Rebuild bar:** given ONLY the spec (Pointers deleted), a stranger rebuilds the same behavior, no code needed. **Tech-agnostic rule:** outside Pointers, no language/framework/library/class/table/file name — business vocabulary only.

## Modes

| Mode | Trigger | Does |
|---|---|---|
| **sync** (chain default) | execution completed, `behavior_change` cells capped | merge behavior deltas into the touched areas' specs |
| **capture** | a discuss→build→test→adjust loop **settles an outcome** — rule, tested behavior, tuning value, policy; a spoken settlement ("chốt"/"final"/"ok ship it") is mandatory same turn | log same turn, then per lane below |
| **flush** | capture queue non-empty at a flush point (wrap-up, PreCompact warning, session-start offer) | drain oldest-first: full merge + `capture flush --id <id> --into <spec>` |
| **harvest** | user asks to document an area, or grooming files a missing-spec item | write the first spec for a pre-bee area; interview for what code can't prove |
| **bootstrap** | no-bundle repos only, `docs/specs/` lacks its map files | offer, never auto-run — a skeleton from provable facts only |

Bootstrap is inventory, harvest is meaning. Sync runs AFTER goal-check — the semantic judge already verified `standard`/`high-risk` `behavior_change` cells — never instead of it.

## Capture — the self-triggering law

**Detection is the scribe's own duty, unprompted.** Most settlements are silent — the user confirms a behavior, accepts an explanation, picks an option, moves on — and the agent watches for these itself, every turn, unasked. Do not ask "should I document this?" — announce what settled and where in one line, then do it same turn. **Close-audit, zero exceptions:** at every task close (cell, docs write, quick fix) ask "what settled here?" and either capture it or state "nothing settled" — smallness is never the answer. A user saying "ghi lại" means detection already failed once.

1. Log: `node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..."` — same turn, every lane.
2. Lane-scaled (never memory-scaled): **high-risk** → full spec merge now, never queued. **Everything else** → one-line stub, `bee.mjs capture add --outcome "..." --did <D-IDs>`, merge deferred to flush.
3. Contradicts shipped behavior → record as "not yet implemented — see backlog" + file it; never state as current.

Litmus: would this outcome survive outside the chat if the session ended now? A queued stub passes. Deferred requests ("để sau", "not now") get the same duty as a `proposed` backlog row. Scribing debt backstops detection, never replaces it. Full protocol: `references/scribing-reference.md ("Capture Mode in full", "Deferred requests")`.

## Merge — nine BA-grade sections

**Purpose → Entry Points & Triggers → Data Dictionary → Behaviors & Operations → Actors & Access → Business Rules → Edge Cases Settled → Open Gaps → Pointers (implementation).** Same nine, same order, for every area shape and for a `bee.area` concept — a concept covers what its subject has content for, never invents a different heading set. Deltas come from evidence only — capped cells, UAT records, worker reports — never `plan.md`, never memory; an unbacked claim is an Open Gap. Before finishing, cover Pointers: could a stranger rebuild this elsewhere? "You'd have to look at the code" is a hole — fix it or file the gap. Full template, rules, gather-sources: `references/scribing-reference.md`.

## Update State

`node .bee/bin/bee.mjs state scribing-run --feature <feature> --areas "<a,b>" --next-action "<...>"` — the `at` stamp clears scribing debt. Nothing to sync/capture → still run it (`--areas "none"`) so debt resets.

## Hard Gates

- Never skip scribing when `behavior_change` cells were capped — any lane, tiny included.
- Never name technology outside Pointers; never state an unverified claim as behavior (evidence→behavior, decision→rule, neither→Open Gap).
- Never create a second spec/concept for a covered area/subject — one truth in place forever, ownership checked bundle-wide, never by eye.
- Never hand-write concept frontmatter (`emitFrontmatter` produces every block); never decide bundle mode by eye or `existsSync`.
- Secrets/PII never appear in specs.

## Headless

`mode:headless`: apply mechanical merges (deltas from cells + evidence) and reading-map fixes; log capture decisions only when the wording is verbatim-quotable. Harvest questions, ambiguous merges, rewording → `Outstanding Questions`.

## Red Flags

- a framework/library/file path above Pointers; an enum without its meaning; a Behavior block that never says what an actor observes
- spec content copied from plan.md or memory; "I'll write it after compounding" — scribing runs first, while evidence is fresh
- a `-v2`/`-new`/date-suffixed spec, or a fresh spec made without checking the reading map
- a new concept for a claimed subject, or `authoritative_for` copied from a sibling; an area left without regenerating indexes
- bundle mode decided by eye or `existsSync`; frontmatter typed by hand; harvest answers invented instead of asked
- a settled outcome nowhere but the chat; a high-risk settlement queued as a stub; a stub or deferred request surviving a flush point unflushed/unfiled
- a capture that ran only because the user asked, instead of the agent catching it itself; asking "should I document this?" instead of announcing and doing it
- a UI screen visibly changed with its snapshot unchanged and no Open Gap; an area added/removed with `system-overview.md` unsynced
- treating scribing as UI-only — backend jobs, APIs, integrations, processes are areas too

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Scribing complete: <N> area specs synced (<coverage>), <M> open gaps, reading map refreshed. Invoke bee-compounding skill.

| Reference | When to Load |
|---|---|
| `references/scribing-reference.md` | spec template, merge rules, bundleMode routing, gather-sources, deferred-requests/harvest/bootstrap protocols, system-overview/reading-map/backlog machinery |
| `references/provenance.md` | Decision IDs + rationale for every body rule |
