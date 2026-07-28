# vo-1

[DONE] Labeled the cell `verify` field as MAIN's command at the exact moment a worker reads it.

Files touched:
- `packages/bee/bee.mjs` — `handleCellsShow` now returns the cell with a `verify_owner` field inserted right after `verify` (value: `main (feature close) — the worker never runs this`), covering both `--json` and text output since both render the same annotated object.
- `packages/bee/lib/command-registry.mjs` — `cells.show` description states the `verify_owner` field and its meaning; `cells.update`'s plan-fields description notes `verify` is edited but never run by the worker.
- `skills/bee-executing/SKILL.md` — added an unmissable one-line rule under Commit ("never run it, never cite its output as evidence"); trimmed stale "worker verifies" phrasing elsewhere in the same file to net-zero body growth (9225B, ceiling 9227B).

Full trace/evidence: `.bee/cells/vo-1.json`.
