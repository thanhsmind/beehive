---
type: bee.area
title: "Bee OKF Profile — purpose, entry points, and actors"
description: "Why bee closes OKF v0.1's deliberate permissiveness with a profile of its own, which verbs enter it, who is bound by it, and what it does not yet answer."
timestamp: 2026-07-22
bee:
  id: okf-profile-overview
  lifecycle: active
  areas: [okf-profile]
  decisions: [D2, D10, D13, D15, D20, D21, D23, D24, D27, D29, D30, D34, D37, D38]
  sources: ["okf-foundation cell okf-1 (knowledge.mjs core — emitter-first frontmatter codec, concept model, two-level check verb; trace in `.bee/cells/`, report `docs/history/okf-foundation/reports/`, 2026-07-22)", "okf-foundation cell okf-2 (bundle skeleton + this spec, 2026-07-22)", CONTEXT.md `docs/history/okf-foundation/CONTEXT.md`, "okf-switchover-f3 cell f3-5 (G6 — this spec migrated into the bundle it describes; trace in `.bee/cells/`, 2026-07-22)", "docs/specs/okf-profile.md#P4", "docs/specs/okf-profile.md#P6", "docs/specs/okf-profile.md#P7"]
  authoritative_for: "okf-profile: purpose, entry points, and actors"
---

# Bee OKF Profile (purpose, entry points, and actors)

## Purpose

The Open Knowledge Format (OKF) v0.1 is deliberately permissive: `type` is its only required
field, consumers MUST tolerate unknown types, MUST NOT reject unknown fields, and MUST tolerate
broken links (OKF §5, §8). That permissiveness is right for a spec meant to fit many organizations,
but it gives an agent nothing to check. The **Bee OKF Profile** is the closed layer bee adds on top:
a nine-type vocabulary, a fixed field set with an identity/path direction, authority-uniqueness
rules, and a two-level validator (`bee knowledge check`) that turns "is this bundle in good shape"
into a machine-checkable question instead of a matter of taste.

**The profile is bee's own contract, not a standard.** OKF profiles are an open, unratified
proposal — `https://github.com/GoogleCloudPlatform/knowledge-catalog/issues/212` — not yet
standardized by the spec's maintainers. Nothing in this document binds any bundle outside this
repository; a different OKF consumer is free to define its own profile, or none.

## How this area is split

The split follows what each part of the profile *governs*, not the section heading it was written
under:

- what a concept **is** — the nine types, the frontmatter field rules, the identity/path direction,
  the authority rules and the legacy carry-over map — and how one is authored, with the four
  canonical worked examples and the rebuild bar: `concept-model-and-authoring.md`;
- how a bundle is **graded** — the two-level check, its exact finding codes, the emitter-first
  codec, and the checker's read-only boundary: `conformance-check.md`;
- how curated knowledge is **consumed and proposed** — the budget-aware context manifest, its
  relevance ranking, the promote proposer, and the session preamble that makes the bundle
  load-bearing rather than optional: `context-and-promote.md`;
- how a legacy source is **migrated** into the bundle and what grades that migration — the coverage
  gate, the content-addressed pin, the unparsed report, the fidelity floor and the drift telemetry:
  `migration-and-coverage-gates.md`.

## Entry Points & Triggers

- `bee knowledge check [--strict] [--json]` — the profile validator; every `knowledge` verb
  supports `--json` (D13). Walks **only** `docs/knowledge/` and never touches a file outside it
  (D23); a missing or empty bundle passes.
- `bee knowledge index [--check] [--json]` — regenerates every `index.md` inside the bundle from
  concept frontmatter (D21); `--check` re-renders in memory and fails naming any stale file.
- `bee knowledge list [--type T] [--lifecycle L] [--area A] [--json]` — one row per concept (path,
  id, type, lifecycle, title), never file content (D15).
- `bee knowledge context --work <id> --budget <tokens> [--json]` — the budget-aware consumer (D27):
  the curated context for a work item, as an ordered **manifest**, never content (B6, in
  `context-and-promote.md`).
- **The session preamble** (`inject.mjs`) — when the active feature has a matching `bee.work-item`
  concept, the preamble names the `context` command and instructs the session to load its manifest
  before touching code (B7, in `context-and-promote.md`). This is the trigger that makes the bundle
  load-bearing rather than optional.
- `node scripts/run_verify.mjs` — the verify chain `knowledge check` and `knowledge index --check`
  both join (D34); a profile violation fails the chain the same way any other suite does. The
  per-migration coverage gates (`scripts/okf_migrate.mjs --check <area>` and `--check-patterns`,
  D35) join the same chain, one entry per migrated source.
- `bee knowledge search --text "<symptom>" [--limit N] [--json]` — the mid-flow symptom pull
  (knowledge-search D1-D7): ranked term-hit search over the bundle's patterns and area concepts,
  read-only at any phase from any checkout (B10, in `context-and-promote.md`).
- `bee knowledge bootstrap [--json]` — stands up an empty, well-formed bundle skeleton in a host
  repo that has none, so a fresh project can start capturing into `docs/knowledge/` without
  hand-typing frontmatter (knowledge-usable U9, cell ku-9, 2026-08-10).
- `bee knowledge report [--json]` — recurrence counts for critical patterns grouped by failure
  signature, the measurement feeding the selective `bee.critical` bar (knowledge-usable U8, cell
  ku-8; the bar itself: `critical-bar.md`).
- `bee knowledge promote --work <id> [--json]` — the loop closer (D38): finished work **proposes**
  the knowledge it earned. It reads the work item's concept and the **capped** cell traces of that
  feature from `.bee/cells/*.json` (a read of the runtime store — D2 permits reads and forbids
  writes) and prints three proposals: a delivery draft, candidate area spec-sync bullets, and
  candidate pitfall patterns. It writes nothing (B5, in `context-and-promote.md`).
- Migration authoring (the D20 loop: read a legacy source, split it into concepts, carry its
  frontmatter across per D33, replace it with a pointer stub per D37) is the human/agent trigger
  that populates the bundle the checker then grades. The checker itself performs no writes (D2).

## Actors & Access

- **The migrator** (a scripted, non-shipped tool in `scripts/`, D24) — authors new concepts under
  `areas/<area>/`, carries frontmatter values across per D33, writes the coverage report (D35),
  and replaces the legacy source with a pointer stub carrying the anchor map (D37, D20).
- **`bee knowledge check`** — grades the bundle; never writes; the sole enforcement surface for
  this profile today.
- **`bee knowledge promote`** — reads the bundle and the capped cell traces; proposes the delivery
  draft, the area bullets and the pitfall candidates a finished work item earned; writes nothing.
  The **human or agent who accepts a proposal** is the actor that turns it into a concept — the
  profile deliberately keeps that step outside the tool (D38).
- **`node scripts/run_verify.mjs`** — the CI-equivalent chain `check` (and later `index --check`)
  joins; a profile violation fails the same way any other suite failure does.
- **The human owner** — configures nothing here; the profile has no runtime knobs. Owner-level
  input landed once, at exploring, as the locked decisions this document cites.

## Open Gaps

- `bee knowledge index` (the generator that replaces this hand-seeded `index.md`, D21) and
  `index --check`'s freshness guard are not built — targeted at **S3**.
- Only one legacy area (`advisor-protocol.md`, F1's proof, D29) is migrated end-to-end; the
  remaining ten `docs/specs/*.md` areas, and `workflow-state.md`'s locked nine-concept
  decomposition (D30), are deferred to **F2**.
- `bee knowledge stale`, work-item back-migration, pointer-stub deletion, and skill rewiring
  (scribing writes concepts, compounding promotes them, hive assembles its context packet, grooming
  hunts stale knowledge) are all deferred (F2/F3 — see CONTEXT.md "Deferred Ideas", filed P66-P69).
  `bee knowledge promote` shipped in S7 (B5); what stays deferred there is the *accepting* half —
  no skill yet routes a promote proposal into an authored concept, so today a human or agent reads
  the proposal and authors the file.
- A work item that declares no `bee.areas` gets an **empty** area-updates section from `promote` —
  correct behaviour (the profile never guesses which area a cell touched, D10), but it means the
  section is only as useful as the work item's own `bee.areas` list. `work/okf-foundation/`'s work
  item declares none, so this feature's own promote run proposes area bullets for nothing.
- Host-repo adoption of the migrator stays out of scope by design (D24: the migrator is bee-only,
  never shipped to hosts).

## Pointers (implementation)

- The bundle itself: `docs/knowledge/index.md`, `docs/knowledge/log.md`.
- Locked decisions this profile implements exactly, cited never reinterpreted:
  `docs/history/okf-foundation/CONTEXT.md`.
- Normative OKF v0.1 spec: `https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md`.
  Profile-as-open-proposal: `https://github.com/GoogleCloudPlatform/knowledge-catalog/issues/212`.
