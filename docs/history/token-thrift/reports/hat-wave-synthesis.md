# Hat wave synthesis — token-thrift

Three seats on standard work: hat-facts-gaps, hat-alternatives,
hat-user-impact (opus each). The leader made the synthesis.

## Findings taken into the plan

| # | Seat | Finding | Change to plan |
|---|------|---------|----------------|
| 1 | alternatives | The host counts the whole session-start stdout (belt line + intent anchor + preamble). On `resume` the anchor has no cap (`hooks/session_init.rs:118-131`, `:186-191`). A 9,500-byte preamble alone does not keep the output under the limit. | tt-1 adds `build_session_preamble_within(…, budget_bytes)`. `session_init.rs` subtracts belt and anchor bytes. A test checks the whole stdout on a resume with a long anchor. |
| 2 | alternatives, user-impact | `bee status --json` carries `recent_decisions` but not the pattern digest or the project map (`verbs/status_full/build.rs:475`, `:506`). | Each Omitted entry names its own fetch path: decisions → `bee status --json`; patterns and project map → `docs/knowledge/index.md`; command surface → `bee --help --names`. |
| 3 | facts-gaps | tt-2 missed door pins: `model_guard.rs:4122` (60-char test), `:3692-3695`, `:3745`, `session_preamble/tests.rs:1460` and others, `compaction.rs:1762-1764`. | tt-2 action lists every pin and sweeps `hooks/` for ` → `. |
| 4 | facts-gaps | `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md:161` states the 60-char door line. | tt-4 updates that file. |
| 5 | facts-gaps | Nobody measured the pathological fixture with the four sections removed. | tt-1 step 0 measures it and returns BLOCKED if over 9,500. |
| 6 | user-impact | The budget binds every runtime, not only Claude Code. | Named in plan Out of scope and in the tt-4 concept. |
| 7 | user-impact | No test asserts the drop order or that a dropped section is absent. | tt-1 adds both tests. |

## Findings not taken

- The compact capsule has no budget (alternatives, fact 2). It is a separate
  path and it was not measured over the limit. The plan names it out of scope.
- The drop order puts the pattern digest before the command surface
  (user-impact). Kept: the digest points to one file the agent can read, and
  the command surface is the single largest lever. The order is D2 as locked.

## Verdict

All three seats recommend approval after the changes above. The plan now
carries them.
