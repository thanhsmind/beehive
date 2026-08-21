# Herding Receipt Source — Context

**Feature slug:** herding-receipt-source
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The delivery receipt reads `herdr pane read <pane_id>` (plain text stdout, no JSON envelope) instead of `herdr agent read <job_id>` — live smoke 7: agent read returns empty for an agy pane while pane read shows the text; the receipt source must be the one that sees | Wrong receipt source made every delivery look failed |
| D2 | The delivery window widens to 30 attempts spaced ~1s (~30s) — smoke 7 proved agy's input stays deaf well past 5 quick sends; the pointer is idempotent so repeats stay harmless | 5 sends in ~3s is inside the deaf window |
