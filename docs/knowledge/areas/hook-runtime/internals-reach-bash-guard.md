---
type: bee.area
title: Hook Runtime — the internals-reach bash guard
description: "Why the write guard denies an inline-eval Bash command that imports a bin/lib or templates/lib internal module, why file-based script runs that import the same modules are unaffected, and the open gap this leaves for bee-scribing's own documented internals-eval helper calls."
timestamp: 2026-07-24
bee:
  id: hook-runtime-internals-reach-bash-guard
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [3fbe2f79 state-query-surface A -- internals-reach guard blocks inline-eval only never file-based node <path>.mjs runs]
  sources: [state-query-surface feature close 2026-07-24, .bee/bin/hooks/bee-write-guard.mjs internals-reach guard branch]
  authoritative_for: "hook-runtime: internals-reach bash guard"
---

# Hook Runtime — The Internals-Reach Bash Guard

## Purpose

Bee's internal library modules carry no compatibility promise: they are
implementation detail the CLI sits in front of, free to change shape without
notice. An agent that reaches past the CLI and imports one of those modules
directly bypasses whatever validation the CLI verb would otherwise have
applied. This guard exists to catch that reach at the point it happens —
inside a Bash tool call — and redirect the agent to the public, validated
surface instead.

## Entry Points & Triggers

- Any Bash command the runtime announces as an inline Node evaluation —
  `node -e "..."`, `node --eval "..."`, or `node -p "..."` — whose script
  text imports or requires a module under `bin/lib/` or `templates/lib/`.

## Data Dictionary

- **inline eval** — a Bash invocation that hands Node a script as a command-
  line string (`-e`/`--eval`/`-p`) rather than a file path.
- **file-based script run** — a Bash invocation that hands Node a path to a
  `.mjs`/`.js` file to execute (`node <path>.mjs`).
- **internal library module** — a module living under `bin/lib/` or
  `templates/lib/`: implementation detail behind the CLI, not part of its
  published contract.

## Behaviors & Operations

The write-guard denies a Bash command that is an inline eval (`node -e`,
`--eval`, or `-p`) whose script text imports or requires an internal library
module (`bin/lib` or `templates/lib`). It **allows** file-based script runs
(`node <path>.mjs`) that import those same modules — this is how the
project's own tests legitimately exercise internals, and that path stays
open. The denial message names the paved read in its place: `bee status
--json` for current state, or `bee <group> --help --json` for a command
group's full schema.

## Actors & Access

Any agent issuing a Bash command in a session where the write guard is
active. The guard does not distinguish orchestrator from worker — the same
inline-eval pattern is denied either way.

## Business Rules

1. Internals carry no compatibility promise; reaching in via inline eval
   bypasses the CLI's own validation, so the guard denies the inline-eval
   form specifically — never the file-based script form tests depend on
   (D 3fbe2f79).
2. Rationale note: the state that first triggered this guard's authorship
   (global scribing debt) was already public via `bee status --json` — the
   agent that reached in via `node -e` was fetching data the paved read
   already exposed.

## Edge Cases Settled

- A file-based `node <path>.mjs` run that imports `bin/lib` or
  `templates/lib` modules is never denied by this guard — that is the
  legitimate test-file shape, and the guard's scope is deliberately narrowed
  to inline eval only so it does not trap those files.

## Open Gaps

The guard, once live in a fresh session, also blocks bee-scribing's own
documented `node -e "import('...knowledge.mjs')..."` helper calls —
`scribingTarget` and `emitFrontmatter` — because there is no CLI verb yet
that exposes them. A scribing worker hitting this guard mid-render has to
fall back to writing a temporary file-based script under a gitignored path
(e.g. `.bee/tmp/`) and running it with `node <path>.mjs` instead of the
documented inline-eval form. This gap is tracked as backlog PBI
`p-0530164c`; until it closes, the documented inline-eval invocation in the
bee-scribing skill and the guard's own denial are in direct tension for
exactly these two helpers.

## Pointers (implementation)

- Guard: `packages/bee/hooks/bee-write-guard.mjs`, internals-reach branch (inline-eval
  detection + `bin/lib`/`templates/lib` import scan).
- Helpers this gap affects: `scribingTarget` and `emitFrontmatter` in
  `bin/lib/knowledge.mjs`.
- Tracked gap: backlog PBI `p-0530164c`.
