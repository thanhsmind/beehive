# workflow-lessons — plan

Source: review of claude-cookbooks `08_Dynamic_workflows.ipynb` (2026-08-11).
Theme: move four coordination rules from "instructed prose" to "enforced
structure". PBIs: p-e0213b88, p-c9e20303, p-e0234de4, p-7484c2ad.

## Shape (standard lane, 4 cells, 2 waves)

Wave 1 — disjoint files, parallel:

- **wl-1 — structured worker result form.** The rendered worker prompt
  (`packages/bee/prompts/worker-cell.md`, compiled via
  `devtools/prompts.rs`) gains a fixed Result-form JSON block the worker
  must emit beside its status token; `bee cells finish` gains optional
  `--report <json>` validated against that shape and stored on the trace.
  Tending reads the form, never parses prose.
- **wl-2 — `bee dev regen`.** One deterministic verb chaining
  render-skill-trees → onboard --apply → release-manifest --write, stop on
  first red, one summary line per step. The REGEN_OBLIGATION refusal text
  names the verb instead of dictating the three steps.
- **wl-3 — judge close door.** `bee close` gains a blocking door for
  standard/high-risk routes: every capped `behavior_change` cell must
  carry a judge record (`cells judge-record`), remedy names the command.
  Tiny/small keep judge-on-smell.

Wave 2 — depends on wl-1 (same dispatch surface):

- **wl-4 — `bee dispatch wave`.** One call prepares every ready cell of
  the current schedule wave (claim + reserve + payload array + economics),
  so the orchestrator spawns all workers in one message. Per-cell
  `dispatch prepare` stays as fallback and for singles.

## Named deviations

- Idea 2 of the source descoped: no generated JS Workflow script — that is
  Claude-Code-runtime-specific; wave-batch prepare captures the context
  win runtime-agnostically. Revisit only if a runtime-neutral script
  runner lands.

## Verify

`commands.test` (cargo test) at every finish; close re-runs; merge verify
re-runs.
