# Evidence check: what piggery could improve in bee — Context

**Feature slug:** piggery-eval
**Date:** 2026-10-16
**Source:** spec drop sup-20261016-a506 (waggledance@e8b3751)
**Scope:** Small, docs only (research/xia brief plus proposed PBIs)

## Feature Boundary

Check six hypotheses about piggery v0.10.2 (`/mnt/data/Projects/refs/slp/piggery`,
read only) against this repo's own code and docs. Deliver one xia brief with a
verdict table, a ranked list of improvements worth doing, a do-not-do list, and
one `proposed` PBI per worthwhile item. No product code change and no piggery
integration.

## Locked Decisions

| ID | Decision |
|----|----------|
| D1 | Multi-harness support (pi, Codex, omp, dsh, opencode as piggery harnesses) is out of scope; only Claude Code and Paseo as bee runs them today count. |
| D2 | bee's "gate" (a human approval in `.bee/state.json`) and piggery's "gate" (a routing gate and a team's representative session) are different concepts and are never merged in the brief. |
| D3 | The brief is `docs/history/research/piggery-xia.md` (bee-research/v1, mode xia); the request is recorded as `docs/discovery/spec-drops/sup-20261016-a506.md`. |
| D4 | Each worthwhile improvement becomes its own `proposed` PBI; nothing is shaped or built in this feature. |
| D5 | The prior feature `piggery-borrow` (piggery v0.6.0, seven items) is prior art: the brief checks what it already shipped before it proposes anything new. |
