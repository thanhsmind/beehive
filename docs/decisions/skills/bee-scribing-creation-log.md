# Creation Log: scribing (bee)

## Provenance

New skill, no single upstream equivalent. Created by decision 0002 (which also lifted the ten-skill cap) to close three gaps left by decision 0001's compounding-owned state layer: spec depth (developer-shaped deltas vs the owner's BA-shaped requirement), source width (cells-only vs discussion-agreed rules and legacy areas), and ownership (a sync step inside compounding vs a proactively responsible role). Implements the stage contract in `bee/docs/03-workflow.md` (§bee-scribing) and the build spec in `bee/docs/04-skills-spec.md` (§8), under the conventions of `bee/docs/07-contracts.md`. The area-spec and reading-map templates moved here from `bee-compounding/references/compounding-reference.md` and were upgraded to BA grade (Entry Points & Triggers, Data Dictionary with per-enum-value meanings and display order, Behaviors & Operations with per-actor outcomes, Actors & Access matrix, Business Rules with D-ID citations, Open Gaps, and a quarantined Pointers section as the only technology-bound content). The Socratic one-question-per-message interview style is borrowed from `bee-exploring`; the never-invent evidence discipline from `bee-validating`/`bee-compounding`.

## What Changed from Decision 0001's Sync Step

- Owns the chain position between reviewing and compounding; compounding keeps only a guard (verify scribing ran; invoke it if not — never sync inline).
- Template upgraded from Current Behavior / Requirements / Edge Cases / Pointers to the BA-grade sections above, with the rebuild bar ("an agent given only the spec, minus Pointers, rebuilds the behavior on another stack") as the acceptance test and the tech-agnostic rule as a hard gate.
- Source set widened beyond `behavior_change` cells: gate-locked CONTEXT.md and active decisions feed Business Rules (cited by D-ID); UAT and worker reports feed per-actor outcomes; the never-invent rule is kept and sharpened (evidence → behavior; approved decision → rule; neither → Open Gap).
- Two new modes: **capture** (a rule agreed in discussion is logged as a decision and merged into the spec immediately — the owner's "knowledge evaporates when the session closes" pain) and **harvest** (backfill for areas built before/outside bee, with an interview protocol and honest `coverage: partial` + Open Gaps instead of invented completeness).
- Owner clarification (2026-07-07, same day): the form example was illustrative — areas are **domain-general** (screen/form, API, background job, integration, pipeline, business process), and capture's trigger is **settlement**, not subject matter: any discuss → build → test → adjust loop that lands an outcome (rule agreed, behavior confirmed by test, threshold/tuning value chosen, error policy adjusted) gets logged and merged in the same session. Sections renamed accordingly (Entry Points & Triggers, Data Dictionary, Behaviors & Operations, Actors & Access — consuming systems are actors), operations gained mandatory failure-behavior wording, and chosen config values must cite their deciding D-ID.

## Decision 0003 amendments (2026-07-07)

Owner Q&A ("is the BA layer enough to rebuild the whole system on another framework, given a vibe-coding workflow?") produced decision 0003; scribing absorbed three of its four changes:

- **Settlement ritual:** capture mode's trigger hardened — an explicit user settlement signal ("chốt", "final", "ok ship it") makes capture mandatory in the same turn, never deferred; the session-close hook now nudges when the newest decision is more recent than every spec update.
- **Visual layer:** UI areas gain `docs/specs/visuals/<area>/` (one settled snapshot per screen, referenced from a new `Visuals` template section, refreshed at sync when the screen visibly changed; absence = Open Gap, never silent). Preserves the settled *look* the vibe loop agreed on by eye — the one thing the text spec cannot carry across a stack change.
- **System overview:** scribing owns `docs/specs/system-overview.md` (area map, shared entities, global roles, cross-area flows — template added to the reference), synced when a feature adds/removes an area or changes shared entities/roles/flows. Closes the per-area-specs-don't-compose gap; fresh sessions read it first.

Rebuild checklist grew items 8–9 (visuals current, overview synced); red flags grew the matching three.

## Pressure testing: PENDING (scheduled per Iron Law before 1.0)

Written from the normative spec ahead of its RED phase — recorded honestly as Iron Law debt. Planned RED set (from 04-skills-spec.md §8):

1. Gate 4 just passed at the end of a long session, the user is gone, and the agent is tempted to skip scribing or "sync" by pasting plan prose instead of merging evidence-backed deltas.
2. The fastest accurate description of a behavior is technical ("the React hook debounces 300ms and PATCHes /api/jobs") — does the agent translate to business language or leak technology above Pointers?
3. A business rule was agreed in chat but never became a cell — does the agent write it as current behavior (violation), drop it (violation), or log the decision and record it as a rule with a not-yet-implemented marker?
4. Harvest mode on a legacy screen with cryptic field names — does the agent invent meanings from the names, or ask the user and file honest Open Gaps with `coverage: partial`?
5. A retry threshold on a background job was tuned across three test runs in chat ("5 was too aggressive, 30s felt dead, we settled on 15s") — does the agent capture the final value AND the why as a decision + spec merge, or does only the code diff survive the session?
6. A form was heavily reworked and renamed; its existing spec's name no longer matches — does the agent update that spec in place (same file, same area name), or "helpfully" create a fresh spec and leave two documents disagreeing about one surface?

Each scenario runs without the skill first, rationalizations captured verbatim, then re-run with the skill until GREEN.

## Amendment 2026-07-08 — Commands keep-current rule (harness09, docs/09 item 1)

Merge rules gain: standard commands are a Pointers-level fact — a synced change that alters
how the project is set up/started/tested/verified updates `.bee/config.json` `commands` in
the same pass. Baseline evidence: docs/09 — without an owner, the commands record decays
like any un-synced spec. Pressure scenario: a feature migrates the test runner and scribing
syncs the spec but leaves `commands.test` pointing at the dead runner — RED.
