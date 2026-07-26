---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-24 (auto-approved, gate_bypass=total; audit decision 59b96255)
---

# plan — i54-closeout

Close the remaining open items from the issue #54 cross-check (v1.16.0).
Decisions locked in `CONTEXT.md` (D1–D11).

## Mode gate record

Risk flags counted: **multi-domain** (dispatch guard, verify runner, knowledge
budget, herding, doctor, state CLI), **changes behavior an existing test
asserts** (guard round-trip tests, `test_model_guard` row semantics,
`test_cli_state` mutation-target rows), **external systems** (live codex CLI
probe). Auth/authorization/data-loss/secrets/validation-removal: none — the
guard change widens observation, never removes a deny (D1). **3 flags →
standard.** Product files ≈ 14 across 8 cells, each cell ≤ 5. Smaller modes
are insufficient: multi-domain and covered-contract changes exceed small's
0–1 flag ceiling; high-risk is not honest because no hard-gate flag trips.

## Discovery

L1 (quick verify), already performed pre-feature with anchors: four parallel
codebase gathers (dispatch schema, hook sources, lane writes, 1.12 items) and
inline checks (`plugin.json` target parses; `hooks/hooks.json` is a
conformance-asserted render; codex CLI 0.145.0 on PATH; CI green on main; no
verify-red issue). Repo precedent: `codex-loop-p0`, `fresh-session-handoff`,
`codex-native-transport` (canary P6/P7 probe legs already exist).

## Approach

One slice, eight independent cells (only the capability-pin cell consumes the
canary evidence produced during validating). All edits in canonical sources;
self-onboard `--apply` syncs vendored trees before verify (D10). Each cell's
verify is registry-scoped (`run_verify.mjs --only <suite>` or direct test
file); the transitive impacted run (`commands.test`) closes the wave.

Risk map:
- dispatch-guard/helper alignment — MEDIUM — proof: live canary schema
  observation during validating + doc-canonical round-trip test (D1).
- lane-write auto-resolve — MEDIUM — proof: existing fsh test fixtures extend
  to the omitted-`--lane` bound-session row (D7).
- verify timeout/heartbeat — LOW — sleeping-fixture test.
- knowledge lane budgets — LOW — default unchanged, preset table test.
- herding adapter — LOW — default byte-equivalent command, template test.
- doctor dual-source row — LOW — both-present fixture test.
- bypass consistency suite — LOW — parses two markdown tables.
- capability pin bump — LOW — canary record is the evidence (D8).

## Test matrix (edge dimensions, scaled)

- **Boundary:** timeout exactly at limit (suite killed, TIMEOUT reported);
  budget preset vs explicit `--budget` precedence; `--lane` explicit vs
  session-bound omitted vs unbound omitted.
- **Absence:** no codex on PATH (canary skip guard stays green); no
  `herding.agent_command` config (default claude command byte-equal); missing
  lane record (refuse loud, unchanged).
- **Duplication/conflict:** both hook sources present (doctor names it);
  README vs SKILL.md matrix drift (suite fails).
- **Contract:** doc-canonical `{task_name, message, fork_turns}` payload
  through the guard — must be an OBSERVED shape after D1, not a no-opinion.

## Slice 1 (current, complete)

Cells i54-closeout-1 … i54-closeout-8, one commit per cell, cell id in the
commit message.
