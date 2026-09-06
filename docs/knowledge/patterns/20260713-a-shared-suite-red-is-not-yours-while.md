---
type: bee.pattern
title: A shared-suite red is not yours while a sibling cell is in flight
description: A shared-suite red is not yours while a sibling cell is in flight
tags: [failure, swarming, verify, parallel-waves]
timestamp: 2026-07-22
bee:
  id: pattern-20260713-a-shared-suite-red-is-not-yours-while
  lifecycle: active
  areas: [worktree-parallelism]
  sources: ["docs/history/learnings/critical-patterns.md#PAT29", "original feature: advisor", "okf-integration-close-f4 f4-4 (this heuristic misapplied by the orchestrator — the trap below; trace in `.bee/cells/`, 2026-07-22)"]
  polarity: pitfall
  critical: true
---

# A shared-suite red is not yours while a sibling cell is in flight

When a cell's verify runs the full shared suite, a red observed while another
cell is claimed-but-uncapped may be the sibling's mid-flight state, not your
defect. Check `.bee/cells/*.json` for in-flight siblings before diagnosing;
re-run after they cap. Never "fix" files outside your cell's scope to green it.

**The trap, learned by walking into it (okf-integration-close-f4, 2026-07-22).** This heuristic
supplies a *plausible* explanation on demand, and a plausible explanation is exactly what stops an
investigation. Two workers reported write-guard reds; they had run the chain concurrently; the
orchestrator applied this pattern, re-ran serially, saw green, concluded "sibling noise" and said so
to the user. The real cause was an environment variable — bee's own mandated `BEE_AGENT_NAME`
prefix — leaking into the spawned suites. The serial re-run was green only because it happened not
to carry the prefix.

**So the pattern gains a precondition: a green re-run is evidence about the conditions it ran
under, and re-running "the same thing" from a different shell silently changes those conditions.**
Before accepting ANY environmental explanation for a red — sibling in flight, concurrency, flake —
reproduce it by toggling exactly ONE variable while holding everything else fixed. If you cannot
make it appear and disappear on demand, you have not diagnosed it; you have found a story that fits.
Suspect this most when the failing assertions are in coordination or concurrency guards, because
there the plausible story is always available and always the same one.
