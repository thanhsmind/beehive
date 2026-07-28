# Provenance — bee-swarming body rules

The orchestrator body states its rules bare (provenance exile, skill-token-diet D8). This table maps
each body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Tiny/small implementation runs through one dispatched execution worker, never a wave | AO14 | Execution authority is a named dispatch class, distinct from I/O gathers, even for the lightest lane |
| Small-lane cells (1-3) run PARALLEL when product file sets are disjoint, 3-4 live workers cap; serial names its conflict | hardening-7, parallel-default D1 | Reservations are the proof and the police; undeclared-overlap concurrency is a wave shape wearing a small lane |
| `regen_obligation_ack: "wave-barrier"` drops shared generated artifacts from the disjointness check; orchestrator owes the regen chain once at wave close | parallel-default D2 | Shared generated artifacts were the near-universal overlap forcing serial dispatch |
| Orchestrator claims the cell before spawning; workers only validate | D1 | A spawned worker never self-selects or claims its own work |
| `--session-id` optional, self-derives from `CLAUDE_CODE_SESSION_ID` | D3 | Session id is environment-derived, never pasted transcript |
| Worker prompt carries only the contract fields, never session history or a literal session id | D3 | Isolation guarantee: fresh context per dispatch |
| Spawn the tier-matched pinned agent type (`bee-gather`/`bee-extract`/`bee-review`), never another plugin's type | W3, AO5/AO10/AO11 | A same-named agent from another plugin carries a different contract |
| NEVER pair a `[bee-tier: ...]` marker with `subagent_type: "general-purpose"` | decision 0023/AO5 (`generic-type-denied`) | `bee-model-guard` enforces this so the rule can't be skipped by habit |
| Codex has no per-agent `subagent_type`; tier stays a read budget + output cap | AO11 asymmetry | Documented asymmetry between runtimes, not parity |
| A planning-recorded `tier` is a hint the orchestrator may override, never fixed | decision 0016 | The orchestrator judges the task in front of it at dispatch time |
| Tier resolution semantics (`resolveTier`, marker anchoring, dispatch economics) | decisions 0012/0015/0019, 0023 (hardened per P1-1), AO12/B1 (plan 2A-ii) | One typed resolver keeps the ceiling scarce and the marker transport mandatory |
| A cli-shaped tier serves gathers only, never cell dispatch, until W9 lands | Discovery-2, AO12/B1 | An external CLI's cwd is not the repo root; unsafe for the reserve/verify/cap contract today |
| Advisor slot resolved and added unless the same-model no-op | AO4/AO5 | Config is the authority; no self-judged strength ladder |
| Goal-check every `[DONE]` yourself — miss reruns, hit ships | P12, decision 0018 | A worker's word is never the evidence; the orchestrator measures before the cell counts |
| Verify re-run is the cell's targeted suite; impacted run once at wave close; full chain is CI-owned | D4, decision `e54878b1` (superseded by ci-owned-verify D1/D6) | Per-cell full-chain re-runs are retired; CI owns the full suite |
| Semantic judge per capped `behavior_change` cell, `standard`/`high-risk` only | D4 (goal-check D4/D5, self-correcting-loop) | Verification of the cell, distinct from any user-invoked review session |
| No auto reviewer; independent review runs only on user request | decision 565e68d0 (R1) | Execution closes `unreviewed`; only the user creates a review session |
| Test consolidation: one done-report line naming the trailing test cell | slice-tail-test-batching P5, spec #80/#85 | Authoring is batched at the slice tail; this is the only place coverage is visible at a glance |
| Fresh-session handoff: continue in-session; `planned-next` only at real session exit | fresh-session-handoff D1/D2, no-clear-stop D1 | Finishing a unit is never a reason to stop; adoption is safe only where no stale context can disagree |
| Rescue ladder rung 2 (stronger tier) tops out at ceiling = the session model | decision 0015 | Ceiling has no config entry; it IS the orchestrator |
| `[BLOCKED]` here already spent its claim consult budget; a rung-1 re-dispatch grants a fresh one | AO3/AO13 (advisor consult budget) | The 2-consult cap is per claim, not per cell lifetime |
