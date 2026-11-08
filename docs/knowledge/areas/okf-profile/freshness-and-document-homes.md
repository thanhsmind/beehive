---
type: bee.area
title: "Bee OKF Profile — freshness, anchors and the document homes"
description: "Which kinds of truth live in docs/knowledge and which live in code, how an area concept names the files it depends on and is graded against git, the cap re-confirm step, the one-time host seed, and the pull-rate measure that tells whether any of it is read."
timestamp: 2026-11-07
bee:
  id: okf-profile-freshness-and-document-homes
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: ["knowledge-freshness-mechanic D1 (2b11f025): knowledge is pulled on demand, never pushed into briefs", "knowledge-freshness-mechanic D2 (93e1001d): the document homes rule", "knowledge-freshness-mechanic D3 (27ad96a5): freshness is derived from git through verified_at and verified_commit", "knowledge-freshness-mechanic D4 (e7e80d78): updates ride the cap sync door and promote", "knowledge-freshness-mechanic D5 (28fb074d): delete before write, work/ leaves the bundle", "knowledge-freshness-mechanic D6 (e7e133dc): pull rate is the measure", "knowledge-freshness-mechanic D7 (6a502ad0): the mechanic is portable and reaches hosts through onboarding", "knowledge-freshness-mechanic D8 (4148025d): bee.anchors is an explicit, seeded path list"]
  sources: [docs/history/knowledge-freshness-mechanic/CONTEXT.md, docs/history/knowledge-freshness-mechanic/plan.md]
  authoritative_for: "okf-profile: the document homes rule, the anchor and verified fields, the cap re-confirm step, the host seed step and the pull-rate measure"
  anchors: [packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs, packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs, packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs]
  verified_at: 2026-11-07
  verified_commit: edc0f9e6da7592e6b025189f86624c6b6cc88e3d
---

# Bee OKF Profile (freshness, anchors and the document homes)

## Purpose

A knowledge entry that describes code can go on saying the same thing long after the code changed, and nothing
tells the reader. This concept records the rule that decides what belongs in `docs/knowledge/` at all, and the
mechanic that keeps what does belong there honest beside the code it describes. It is portable: the binary
carries the mechanic, this area and `skills/bee-capturing/SKILL.md` document it, and a host project receives
all three through onboarding without configuring anything.

## The document homes rule

Five kinds of truth live in `docs/knowledge/`: the vision, the glossary, the area map (an area's boundaries
and invariants), decisions, and patterns. The glossary is named here as a home; it is not yet authored, and
neither is an event catalogue — both wait on the pull-rate measure below.

Three kinds live in code, where a change to the behaviour changes them in the same commit: verify features and
tests (what the product does), generated contracts (the shapes a verb or file exchanges), and runnable runbooks
(scripts that perform a procedure instead of describing it).

Every area entry anchors to at least one code file or verify feature. An entry that names nothing it depends on
cannot be graded and has no home in the bundle. Behaviour narrative that a test already states is the reader's
cue to delete the narrative, not to keep two copies.

## The fields and the grade

An area concept carries three fields under `bee`:

- `anchors` — a flat list of repo-relative paths the concept's truth depends on;
- `verified_at` — the ISO date the concept was last confirmed;
- `verified_commit` — the full sha of the commit it was confirmed against.

`bee knowledge check` grades each area concept `confirmed`, `suspect`, `orphan` or `unverified` from git. The
rule, including how a squashed or rebased sha and a commit that edits both an anchor and the concept are
treated, lives in `docs/history/knowledge-freshness-mechanic/plan.md` under Approach, "Grade rule"; it is
cited here and not restated. Search, show, context and the session preamble print the grade wherever a concept
is read.

Patterns carry a date and no grade: they are neither anchored, seeded nor graded.

## The cap step

The cap sync door adds a fourth check beside the three it already had. When a cell's touched files intersect a
concept's `anchors`, the cap refuses while that concept grades `suspect` or `orphan`. The worker either edits
the concept in the same cell or runs `bee knowledge confirm --id <bee.id>` (or `--path <file>` for every
concept anchoring that file), reserves the concept file and commits it with the cell. For an orphan the worker
fixes `bee.anchors` instead, because confirm never clears an orphan.

A concept that grades `unverified` passes: a host that never seeded, or a concept outside git, is never
blocked. `--sync-ack "<reason>"` remains the recorded escape and is stored on the cell's trace.

## Host steps

The binary and the skills arrive in a host through onboarding. The owner then runs `bee knowledge seed-anchors`
once: it fills `bee.anchors` from the repo paths that resolve in each area concept's body and stamps a baseline
`verified_commit` and `verified_at` from the concept file's own last commit, so the first grade is honest.
It writes only concepts that have no `bee.anchors` and never touches `patterns/`. Until the seed has run,
`bee knowledge check` prints a NOTE that no concept carries `bee.anchors`. New area entries carry `bee.anchors`
from the start; the seed is not re-run to author them.

## The pull-rate measure

Every timings row carries the feature bound to the session that ran the command. `bee knowledge report` prints,
per feature, how many `knowledge search`, `knowledge context` and `knowledge show` calls ran against all calls,
and the ratio. That ratio is the measure of whether the bundle is read. A flat rate means the lean layer
carries no weight with the agents it was written for, and the lean layer is deleted next, not extended.

## Pointers (implementation)

- Grade, bundle root, `seed-anchors` and `confirm`: `packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs`.
- The cap door's anchored-concept check: `packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs`.
- The pull-rate section of the report: `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs`.
- The locked decisions and the grade rule: `docs/history/knowledge-freshness-mechanic/CONTEXT.md` and
  `docs/history/knowledge-freshness-mechanic/plan.md`.
- The capture-side step that cites this concept: `skills/bee-capturing/SKILL.md`.
