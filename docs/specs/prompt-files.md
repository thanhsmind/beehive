# Prompt files and learned-context injection

Status: owner-approved direction (2026-07-31). Two coupled changes to the
dispatch layer.

## 1. Prompts live in files, not code

`packages/bee/prompts/*.md` holds every machine-assembled prompt body:
`worker-cell.md`, `gather.md`, `reviewer.md`, `advisor.md` (and any other
prompt string currently embedded in dispatch/judge code). A minimal
renderer (~30 lines, no dependency) substitutes `{{name}}` placeholders
and includes/omits blocks marked `{{#if name}}…{{/if}}`.

- The SPLIT: templates hold wording; code holds the logic that computes
  which blocks appear and what fills the placeholders. Editing a
  prompt's wording never touches code.
- Rendering is byte-identical to today's output for every existing case —
  pinned by the existing dispatch tests before and after the move.
- Onboarding vendors `prompts/` beside the engine (`.bee/bin/prompts/`),
  version-managed like `lib/` — hosts always run the prompts matching
  their vendored engine.

## 2. Learned context is injected, never re-derived

The worker prompt gains one conditional block, machine-assembled at
dispatch time — closing the learn→use loop (the capture layer writes the
project's memory; dispatch now reads it back in):

```
Learned context (machine-assembled — read before implementing; prefer it
over re-deriving):
- <path> — <one-line title>
…
```

Source resolution, first hit wins, all failures silent (the block is an
enrichment, never a refusal path):

1. Bundle repo with a `bee.work-item` concept matching the cell's
   feature → `knowledge context --work <feature> --lane <lane>` manifest;
   render its selected paths + titles (paths only, never file contents —
   the budget belongs to the worker's own reading).
2. Bundle repo, no work-item concept → the bundle index's critical
   patterns pointer (`docs/knowledge/index.md`) plus the area concepts
   whose `areas` overlap the feature's touched areas when that mapping
   is cheaply available; otherwise just the index pointer.
3. No bundle → `docs/history/learnings/critical-patterns.md` when it
   exists on disk.
4. Nothing found → block omitted; prompt byte-identical to today.

Cap the block at 8 lines (8 pointer lines; the header line is not
counted — the same convention as the Prior-rounds event-line cap). The
cell's own `read_first` stays authoritative and is never duplicated into
this block.
