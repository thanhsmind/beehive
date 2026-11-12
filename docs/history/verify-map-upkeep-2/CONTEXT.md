<!-- bee:no-read-set: verify-upkeep pass; the read set is the live run 20261112-123209-529324 and the source wave reports, cited in plan.md -->

# verify-map-upkeep-2 — context

The second `bee-verify-upkeep` pass over `.bee/verify/verify-app/`. The first
ran on 2026-09-02. The owner asked on 2026-11-12 whether the verify map is
actually in the flow and kept up; this pass answers it and ships the proven
corrections.

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | Edit only `.bee/verify/verify-app/` source, then re-render with `bee onboard --apply`. | bee-verify-upkeep edit scope. |
| D2 | Product defects found by the pass go to the backlog, not into this change. | bee-verify-upkeep triage rule; six rows filed 2026-11-12. |
| D3 | Every correction in plan.md is proven live in run 20261112-123209-529324 or cited from source with high confidence; nothing else ships. | A wrong correction is worse than a stale line. |
