---
type: bee.area
title: "Compiled runtime: prompt files, and the learned-context block that closes the learn-to-use loop"
description: "Why every machine-assembled prompt body lives in a file rather than in code, what the split between template wording and computing logic buys, how a dispatched worker is handed the project's own learned context instead of re-deriving it, and why every source in that resolution chain fails silently."
tags: [rust-runtime, prompts, dispatch, knowledge]
timestamp: 2026-08-03
bee:
  id: rust-runtime-prompt-files-and-learned-context
  lifecycle: active
  areas: [rust-runtime]
  required_context: [areas/rust-runtime/overview.md]
  decisions: [prompt-files]
  sources: [docs/specs/prompt-files.md, packages/bee/prompts, packages/bee-rs/crates/bee/src/devtools/prompts.rs]
  authoritative_for: "rust-runtime: prompt file rendering and the worker learned-context block"
---

# Compiled runtime — prompt files and learned context

Two coupled changes to the dispatch layer, both aimed at the same thing: a
prompt is content, and content that lives inside code can only be changed by
someone willing to touch code.

## Purpose

Let a prompt's WORDING be edited without touching logic, and let a dispatched
worker start from what the project already learned rather than rediscovering it.
The second half is what closes the loop the capture layer opens: capture writes
the project's memory, and dispatch is where it gets read back in. Before this,
the memory was written and never spent.

## Entry Points & Triggers

- Any dispatch that assembles a prompt body — worker, gather, reviewer, advisor.
- `bee dev render-prompt` renders one by name, for inspection or diffing.
- Onboarding vendors `prompts/` beside the engine (`.bee/bin/prompts/`), version
  managed exactly like the binary, so a host always runs the prompts that match
  its vendored engine.

## Data Dictionary

| Term | Meaning |
|---|---|
| prompt file | `packages/bee/prompts/<name>.md` — the body of one machine-assembled prompt |
| placeholder | `{{name}}` — substituted by the renderer |
| conditional block | `{{#if name}}…{{/if}}` — included or omitted by the renderer |
| learned-context block | The worker-prompt section listing paths the worker should read before implementing |

## Behaviors & Operations

**Wording lives in the template; the decision about what appears lives in code.**
The renderer is minimal and dependency-free: substitute placeholders, include or
omit `{{#if}}` blocks. It computes nothing. What each actor observes: an editor
changing a sentence touches one `.md` file; a developer changing WHICH block
appears touches the dispatch code that fills it. Failure behavior: the move was
pinned byte-identical against existing dispatch output before and after, because
a prompt refactor that quietly changes what a worker is told is indistinguishable
from a behavior change until a worker acts on it.

**The worker prompt carries a machine-assembled learned-context block.** It
lists PATHS and one-line titles, never file contents — the reading budget belongs
to the worker, and handing it prose it did not ask for spends that budget for it.
Resolution is first-hit-wins:

1. A bundle repo with a `bee.work-item` concept matching the cell's feature →
   the `knowledge context --work <feature> --lane <lane>` manifest; render its
   selected paths and titles.
2. A bundle repo with no matching work-item concept → the bundle index's
   critical-patterns pointer, plus the area concepts whose `areas` overlap the
   feature's touched areas when that mapping is cheaply available; otherwise just
   the index pointer.
3. No bundle → `docs/history/learnings/critical-patterns.md` when it is on disk.
4. Nothing found → the block is omitted entirely, and the prompt is
   byte-identical to one assembled without this feature.

**Every failure in that chain is silent, deliberately.** The block is an
ENRICHMENT, not a gate. A worker that cannot be told what the project learned
must still be dispatchable; turning a missing manifest into a refusal would make
the memory layer a single point of failure for execution, which is a far worse
trade than a worker occasionally re-deriving something.

## Actors & Access

| Actor | May |
|---|---|
| The orchestrator | Assemble and dispatch; it owns which sources are consulted |
| A dispatched worker | Read the block; it never writes one |
| A host project | Receive vendored prompts through onboarding; never hand-edit `.bee/bin/prompts/` (it is version-managed and re-synced) |

## Business Rules

- R1 — Every machine-assembled prompt body lives in `packages/bee/prompts/*.md`.
  No prompt string is embedded in dispatch or judge code.
- R2 — The renderer substitutes and includes/omits. It never computes.
- R3 — Rendering is byte-identical to the pre-move output for every existing
  case.
- R4 — The learned-context block carries paths and titles only, never contents.
- R5 — The block is capped at 8 pointer lines; the header line is not counted
  (the same convention as the Prior-rounds event-line cap).
- R6 — Every source in the resolution chain fails silently; the block is omitted
  rather than the dispatch refused.
- R7 — The cell's own `read_first` stays authoritative and is never duplicated
  into the block.

## Edge Cases Settled

- **A cell whose `read_first` already names a learned path** keeps it there. The
  block does not deduplicate against `read_first`, and does not need to: one is
  the cell's own contract, the other is ambient context.
- **A host running a stale vendored prompt copy** is the same class of drift as a
  stale vendored tool, and onboarding alone does NOT heal it. The regeneration
  chain re-vendors prompts, libraries, helpers and hooks; it never re-vendors the
  tool's own executable, which is ignored by version control and protected from
  removal on purpose. The skew check compares both the source prompt and the
  vendored prompt against the copy compiled into the running executable, so
  editing a prompt without rebuilding and reinstalling that executable puts every
  dispatch preparation in the checkout into skew, where it returns nothing and has
  nothing behind it to fall back to. A feature worktree cannot repair it either:
  its copy of the tool is a link back to the main checkout, and the write guard
  refuses a write that travels through a link. A prompt edit and a rebuilt,
  reinstalled executable are therefore one unit of work, never two.

## Open Gaps

- Rule 2's "area concepts whose `areas` overlap the feature's touched areas when
  that mapping is cheaply available" is deliberately soft. What counts as cheap
  has never been pinned to a measurement.

## Pointers

- The templates: `packages/bee/prompts/`.
- The renderer and its verb: `packages/bee-rs/crates/bee/src/devtools/prompts.rs`.
- The manifest the first source reads: `bee knowledge context --help`.

## Sources and provenance

- `docs/specs/prompt-files.md` — the migrated source; it survives as a pointer
  stub and is superseded by this concept (2026-08-03).
- `packages/bee/prompts/` — the templates themselves.
- `packages/bee-rs/crates/bee/src/devtools/prompts.rs` — the renderer and
  `bee dev render-prompt`.
- Decision: prompt-files (owner-approved direction, 2026-07-31) — prompt bodies
  move to files behind a minimal renderer, and the worker prompt gains a
  machine-assembled learned-context block.
