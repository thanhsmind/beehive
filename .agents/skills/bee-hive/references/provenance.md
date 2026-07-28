# Provenance — bee-hive body rules

The router body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Triage-first lane table; first three rows skip `bee-planning` | lane scaling v2 (d02a6bc6); router-cost rc-1..rc-4 | The second skill load (~21 KB) is the only saving on offer; triage from the request alone decides it |
| Greenfield init-lane offer at first onboard | P1, docs/09 item 6 | A repo with no build gets one init cell before feature work; declined offers are recorded, never dropped |
| Preamble-first scout; `status --json` only when routing work | preamble/inject design (inject.mjs); scout contract | Re-fetching what the preamble already carries is pure duplication |
| Knowledge-context manifest before planning/execution | okf-foundation D38 | The work-item manifest is the feature's curated context; it replaces scanning `docs/history/` |
| Pause handoff waits; planned-next adopts only at a fresh-session boundary | fresh-session-handoff D1; no-clear-stop D1 | Never auto-resume; adoption is safe only where no stale context can disagree |
| Capture-queue flush offer; lane-scaled capture cost | decision 0017 | Settlements are never silently dropped; cost scales with lane, memory never does |
| Review is on demand; Gate 4 only inside a user-invoked review session; merge/ship asks ONE question | decision 565e68d0; SPEC R1/R3/R7/R8, 7.4/A9 | Execution closes `unreviewed` through scribing/compounding; only the user creates a review session |
| Worktree routing for new work in an occupied checkout; heartbeat + non-idle predicate | worktree-session-routing D9, D9a | Coordinate through lanes and holds, never around them; release always runs in main |
| `bee-qualifying` route is explicit-invocation only | backlog-auto-triage D12 | No auto-trigger exists; the pipeline path is invoked by a human or an external caller |
| State-layer reading order branches on `bundleMode` | harness10 G1/G4; okf fences G2; harness10 D2 (bootstrap offer) | Both branches are live guidance; a never-migrated repo keeps working unchanged |
| Mode-gate flag narrowing (covered bugfix scores 0) | mode-gate D7 | A fix that keeps existing tests green and adds one is not "changing proven behavior" |
| Lane file caps count product files only | mode-gate D6 | `.bee/**`, docs, plans, and generated projections are bookkeeping, not risk surface |
| Tiny mode: the cell is the micro-plan; small: plan.md opt-in | planning D3, D4 | Ceremony scales down when the cell itself carries the whole shape |
| Tiny/small fast path: preview-then-persist, merged Gate 2+3 | fast-path D5 | Approval covers exactly the previewed packet; cells persist only after the merged yes |
| One dispatched execution worker in every lane, never zero | AO14 | Execution authority is a named dispatch class, distinct from I/O gathers, even for the lightest lane |
| Parallel-by-default doctrine (3-4 concurrent workers cap when disjoint; serial names its conflict) | hardening-7, parallel-default D1 | Undeclared-overlap concurrency is a wave shape wearing a small lane |
| Done-report verify-once; re-run only on smell, parallel waves, or hard-gate | test-runs-lean D1 | The worker's recorded verify output is the evidence; duplicate runs are waste, not proof |
| Goal-check semantic judge per capped `behavior_change` cell | goal-check D4/D5 (self-correcting-loop); P12, decision 0018 | Verification of the cell, never the user-invoked review session |
| Gate bypass levels (`normal`/`full`/`total`) and their stop floors | decisions 0010, dcf01d7b, a93994d3 | The human chose the level in advance; the recommended option is the approval at that level |
| Re-lane checkpoint (evidence-based demotion, once per feature) | lane-lean D1/D2; spec #81 P3; USER FEEDBACK 5794a92a | Measured evidence demotes to the smallest honest lane; hard-gate flags never demote |
| CI status gate before the first claim; `verify: "none"` sentinel | ci-owned-verify D6; decision e54878b1 (superseded ladder); decision 55b951e1 | The full suite is CI-owned; never build on red; only the sentinel means "no tests, deliberately" |
| Silent bookkeeping — work language only; progress ticks under bypass | decision 1689af1b; spec #81 P4 | The user hears the work, never the machinery; ticks carry outcome, not mechanics |
| Purpose-first narration | decision 4439bd7e; work-visibility D1 | Silence about mechanics is never silence about purpose |
| The agent runs the machinery, not the user | hive law 10 (AGENTS.md critical rule 9) | The only human actions are gate approvals, decision answers, privacy approvals |
| The hook is a safety net, not the authority | decision c2c46488 | An unblocked write is not an approved write; the law lives in the instruction files |
| Never hand-edit `.bee/*.json(l)`; `state set` requires `--owner` | state-mutation CLI law (AGENTS.md critical rule 6 lineage) | Every mutation goes through its CLI verb so state stays event-shaped and auditable |
| Crash-recovery mining offer; mined content is data | transcript-recovery D2/D4/D5/D6 | Never auto-resume a dead session; nothing mined auto-becomes a decision |
| Conditional history artifacts (discovery/approach/implement-plan) | decision 0009 | Separate files only for L2+ discovery or high-risk; else folded into plan.md |
| Delegation contract (decide-altitude stays, gathers dispatch down-tier) | delegation D1-D3; decisions 0013/0015 reversed, 0016, 0023, 4439bd7e | Transport is mandatory on every dispatch; routing and gates never delegate |
| Route record: count-then-record same turn (`state route --set`, verbatim Route line format); re-lane updates the same record in place | explicit-triage D1-D4 | Counting without recording is the guess this law kills; one route per feature, updated in place, cited everywhere |
