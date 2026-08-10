---
type: bee.pattern
title: "A cell dependency in the wrong field name is silently ignored — verify the wave, not the write"
description: "Verify the wave, not the write"
tags: [failure, cells, deps, scheduler, silent-accept]
timestamp: 2026-07-16
bee:
  id: pattern-20260716-a-cell-dependency-in-the-wrong-field-name
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT41", "original feature: perf-log"]
  polarity: pitfall
  critical: false
---

# A cell dependency in the wrong field name is silently ignored — verify the wave, not the write

`cells add` accepted `"depends_on": [...]` without error (unknown keys are preserved), but the
scheduler and the claim gate read `cell.deps` — so a 1→2→3 chain collapsed into ONE wave with
`cycles: []`, looking healthy while enforcing no ordering. The field is `deps`. **Rule:** after
any `cells add` that declares dependencies, run `bee cells schedule --feature <f> --json` and
confirm the wave shape matches the intended order — a clean `cycles: []` is not proof the deps
were honored, only that nothing cycled. Generalizes: an optional-field writer that silently
keeps unknown keys turns every field-name typo into a silent no-op; confirm the *effect*
(the computed schedule), never the write.
