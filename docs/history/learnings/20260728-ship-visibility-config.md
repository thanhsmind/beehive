---
date: 2026-07-28
feature: ship-visibility-config
categories: [workflow-state, cli]
severity: low
tags: [config, visibility, preamble]
---

# ship-visibility-config — feature close learnings

## What Happened

Closed the last gap of the four ak performance specs: spec #81 P1's
`ship_visibility` config (off | draft-pr, default off) now reads from
`.bee/config.json`, surfaces in `status --json`, and adds one preamble line
only when active. PR/push mechanics stay prose-driven (routing-and-contracts
"Ship visibility") — the runtime carries the switch, the orchestrator carries
the act. P2/P3/P4 were confirmed prose-complete by audit; no code owed.
Lean close (no analyst wave): 2-cell feature, first-hand evidence.

## Findings

1. **Export-allowlist tests make every new export a declared act.** sv-1's two
   new `state.mjs` exports tripped `EXPECTED_STATE_EXPORTS` in `test_misc.mjs`
   — an undeclared-file judge hit that was the allowlist doing its job.
   *Rule: a cell adding exports to an allowlisted module should declare the
   allowlist test in its files up front; the judge hit is otherwise guaranteed.*
2. **Session state got clobbered twice** (phase/feature reverted to a prior
   feature mid-swarm; once after a session resume, once during a worker run).
   Filed as P3 friction with repro notes — suspect worker-side or
   session-close-hook state writes racing the orchestrator's lane.

## Recommendation

- When adding a config key, mirror an existing reader (`bypassLevel` pattern:
  normalize + warn + default) — sv-1 did, and the surface stayed one-line
  cheap in the preamble (zero cost when off).
- Track the state-clobber friction before the next multi-worker feature; a
  lane-bound orchestrator session should never lose its phase to a worker.
