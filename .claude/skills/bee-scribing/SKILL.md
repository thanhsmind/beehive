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

Scribing is bee's BA — it owns the state layer. Quoted headings resolve in `references/scribing-reference.md`.

An **area is domain-general**: a screen, API, job, integration, pipeline, CLI command, or process — any unit with observable behavior. Code is the implementation; the spec is the *meaning*.

## Where Meaning Is Written (bundleMode Routing)

`bundleMode(root)` — true only when `docs/knowledge/` exists AND a concept in it parses (a `.gitkeep`-only dir is NOT a bundle) — is the single predicate for where new area truth is written.

### 2a. Bundle branch

`docs/knowledge/areas/<area>/*.md`, one `bee.area` concept per subject, via `scribingTarget()`. Refusals: `subject_required` (empty/blank/punctuation-only subject on a new concept — never routed to `overview.md`); `duplicate_authority` (two+ concepts already claim the subject, `owner.conflicts` — collapse to one authority, re-ask). Backstop for a paraphrase neither refusal catches: the bundle-wide `duplicate_authoritative_for` chain-fail in `bee knowledge check`.

### 2b. No-bundle branch

`docs/specs/<area>.md` + `system-overview.md` + `visuals/` + `reading-map.md`. **One area = one file, forever.**

Same rebuild bar, tech-agnostic rule, nine sections, modes, never-invent in both. Full routing and worked examples: `references/scribing-reference.md ("Map Deltas", "Bundle-mode gate and frontmatter")`.

**Rebuild bar:** given ONLY the spec (Pointers deleted), a stranger rebuilds the same behavior, no code needed. **Tech-agnostic rule:** outside Pointers, no language/framework/library/class/table/file name — business vocabulary only.

## Modes

| Mode | Trigger | Does |
|---|---|---|
| **sync** (chain default) | feature close, capped cells incl. `behavior_change` | merge feature's behavior deltas into touched specs, once |
| **capture** | a discuss→build→test→adjust loop **settles an outcome** — rule, tested behavior, tuning value, policy; a spoken settlement ("chốt"/"final"/"ok ship it") is mandatory same turn | log same turn, then per lane below |
| **flush** | capture queue non-empty at a flush point (wrap-up, PreCompact warning, session-start offer) | drain oldest-first: full merge + `capture flush --id <id> --into <spec>` |
| **harvest** | user asks to document an area, or grooming files a missing-spec item | write the first spec for a pre-bee area; interview for what code can't prove |
| **bootstrap** | no-bundle repos only, `docs/specs/` lacks its map files | offer, never auto-run — a skeleton from provable facts only |

Bootstrap is inventory, harvest is meaning. Sync runs AFTER goal-check — the semantic judge already verified `standard`/`high-risk` `behavior_change` cells — never instead of it.

## Capture — the self-triggering law

**Detection is the scribe's own duty, unprompted.** Most settlements are silent — the user confirms a behavior, accepts an explanation, picks an option, moves on — and the agent watches for these itself, every turn, unasked. Do not ask "should I document this?" — announce what settled, then do it same turn. **Close-audit, zero exceptions:** at every task close (cell, docs write, quick fix) ask "what settled here?" and either capture it or state "nothing settled" — smallness is never the answer.

1. Log: `node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..."` — same turn, every lane.
2. Lane-scaled (never memory-scaled): **high-risk** → full spec merge now, never queued. **Everything else** → one-line stub, `bee.mjs capture add --outcome "..." --did <D-IDs>`, merge deferred to flush.
3. Contradicts shipped behavior → record as "not yet implemented — see backlog" + file it; never state as current.

Litmus: would this outcome survive outside the chat if the session ended now? A queued stub passes. Deferred requests ("để sau", "not now") get the same duty as a `proposed` backlog row. Scribing debt backstops detection, never replaces it. Full protocol: `references/scribing-reference.md ("Capture Mode in full", "Deferred requests")`.

## Merge — nine BA-grade sections

**Purpose → Entry Points & Triggers → Data Dictionary → Behaviors & Operations → Actors & Access → Business Rules → Edge Cases Settled → Open Gaps → Pointers (implementation).** Same nine, same order, for every area shape and for a `bee.area` concept — a concept covers what its subject has content for, never invents a different heading set. Deltas come from evidence only — capped cells, UAT records, worker reports — never `plan.md`, never memory; an unbacked claim is an Open Gap. Before finishing: could a stranger rebuild this from Pointers alone? "Look at the code" is a hole — fix it or file the gap. Full template, rules, gather-sources: `references/scribing-reference.md`.

## Update State

`node .bee/bin/bee.mjs state scribing-run --feature <feature> --areas "<a,b>" --next-action "<...>"` — the `at` stamp clears scribing debt. Nothing to sync/capture → still run it (`--areas "none"`) so debt resets.

## Hard Gates

- Never skip feature-close sync when capped cells hold `behavior_change` — any lane, tiny included.
- Never name technology outside Pointers; never state an unverified claim as behavior (evidence→behavior, decision→rule, neither→Open Gap).
- Never create a second spec/concept for a covered area/subject — one truth in place forever, ownership checked bundle-wide, never by eye.
- Never hand-write concept frontmatter (`emitFrontmatter` produces every block); never decide bundle mode by eye or `existsSync`.
- Secrets/PII never appear in specs.

## Headless

`mode:headless`: apply mechanical merges (deltas from cells + evidence) and reading-map fixes; log capture decisions only when the wording is verbatim-quotable. Harvest questions, ambiguous merges, rewording → `Outstanding Questions`.

## Red Flags

- a framework/library/file path above Pointers; a Behavior block that never says what an actor observes
- spec content copied from plan.md or memory; "I'll write it after compounding" instead of now
- a `-v2`/`-new`/date-suffixed spec, or a fresh spec made without checking the reading map
- a new concept for a claimed subject, or `authoritative_for` copied from a sibling; an area left without regenerating indexes
- harvest answers invented instead of asked
- a settled outcome nowhere but the chat; a stub or deferred request surviving a flush point unfiled
- a capture triggered only because the user asked, not caught by the agent itself
- a UI screen visibly changed with its snapshot unchanged and no Open Gap; an area added/removed with `system-overview.md` unsynced
- treating scribing as UI-only — backend jobs, APIs, integrations, processes are areas too

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Scribing complete: <N> area specs synced (<coverage>), <M> open gaps, reading map refreshed. Invoke bee-compounding skill.

| Reference | When to Load |
|---|---|
| `references/scribing-reference.md` | spec template, merge rules, bundleMode routing, gather-sources, deferred-requests/harvest/bootstrap protocols, system-overview/reading-map/backlog machinery |
