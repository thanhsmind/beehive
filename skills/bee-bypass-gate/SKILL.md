---
name: bee-bypass-gate
description: >-
  Toggle opt-in gate-bypass autopilot, which now has levels. `normal` auto-approves Gates 1-2 for tiny/small/standard work (high-risk, secrets, and Gate 4 still stop); `full` also auto-approves high-risk/hard-gate work; `total` stops for nothing at all. Use when the user wants to run the pipeline without approving every gate, to widen how far bypass reaches, or to check or turn it off. Invocable as the command bee-bypass-gate with off / on / normal / full / total / status.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads and reports state/config via the vendored .bee/bin helpers.
---

# bee-bypass-gate (autopilot toggle)

This skill sets one persistent per-repo value: `.bee/config.json` `gate_bypass`, which is a **level** — `off` / `normal` / `full` / `total`. It does not run any pipeline work itself. The **behavior** of each level lives in the Gate Presentation Contract (`bee-hive/references/routing-and-contracts.md`, "Gate bypass mode") — this skill only sets the level and states exactly what that level means, so the user opts in with eyes open.

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

## The levels (say the chosen level's row when setting it)

Gate 2 approves `shape` and `execution` together (`--merge`) — the retired Gate 3 has no question of its own, so "Gates 1-2" is the whole set below review. At any on-level the agent stops asking at bypassed gates and instead takes the RECOMMENDATION option, records the approval with `node .bee/bin/bee.mjs state gate --name context|shape|execution --approved true`, logs a one-line audit decision, and continues — posting a short `⚡ auto-approved Gate N` line, not a question. How far that reaches is the level:

| Level | `gate_bypass` | Auto-approves | Still stops for the human |
|---|---|---|---|
| `off` | `false` | nothing | **every** gate (default) |
| `normal` | `true` | Gates 1-2 for `tiny`/`small`/`standard` non-hard-gate work | high-risk/hard-gate Gates 1-2 · secret reads · Gate 4 UAT/P1 |
| `full` | `"full"` | **all** Gates 1-2 at every lane, high-risk/hard-gate included | secret-file reads · a review P1 finding |
| `total` | `"total"` | **everything** — all Gates 1-2, secret reads, Gate 4 UAT, review P1 | **nothing — zero stops** |

- The high-risk/hard-gate flags are: auth · authorization · data loss · audit/security · external provider · validation removal · database migration/schema change. Under `normal` they stop; under `full`/`total` the user has chosen to lift that floor.
- **Secret-file reads** (`.env*`, `*.pem`, keys, `credentials*`, `secrets.*`) stop under `off`/`normal`/`full`; only `total` auto-proceeds on them — meaning credential contents may enter context/logs unprompted. Say this plainly when the user picks `total`.
- **Gate 4** exists only inside a review session the user explicitly started; no level creates one. Under `normal`/`full` its UAT items and P1 findings stop; under `total` they auto-proceed on the recommended resolution.

Bypass is **not** headless: headless defers within-stage questions but still stops at every gate. Bypass is the one mechanism that self-approves gates. Legacy `gate_bypass: true` is read as `normal`, so existing repos are unchanged.

## Operation

Parse the argument: `off` | `on` | `normal` | `full` | `total` | `status` (no argument → `status`, then ask which level the user wants). `on` is an alias for `normal`.

1. Read current state: `node .bee/bin/bee.mjs status --json` (the `gate_bypass_level` field) and `.bee/config.json`.
2. Apply:
   - **status** — report the current level in plain language, plus what that level does and does not stop for. No write.
   - **off** — set `gate_bypass: false`. Confirm every human gate is back. Log it: `node .bee/bin/bee.mjs decisions log --decision "gate-bypass set to off" --rationale "..."`.
   - **on** / **normal** — set `gate_bypass: true`. State the `normal` row. Log it.
   - **full** — set `gate_bypass: "full"`. State the `full` row, and name plainly that high-risk/hard-gate work (auth, data loss, security, external, DB migration) will now auto-approve without a human checkpoint. Log it.
   - **total** — set `gate_bypass: "total"`. State the `total` row, and name plainly the two things that stop being human-gated versus `full`: secret-file reads and a review's UAT/P1 findings. Confirm the user wants literally zero stops. Log it.
     (Preserve every other config field; create `gate_bypass` if absent.)
3. Config writes are `.bee/`-layer — allowed in any phase, no gate, no permission needed. Never touch `state.json` gates from this skill; it only sets the config value.

The change takes effect immediately for the current session and persists across sessions until changed. The session preamble and `bee_status` both print a loud level-specific `GATE BYPASS` banner (`NORMAL` / `FULL AUTOPILOT` / `TOTAL AUTOPILOT — ZERO STOPS`) while it is active, so the active level is never silently in effect.

## Hard Gates

- This skill only writes `.bee/config.json` `gate_bypass`. It never approves a pipeline gate, never edits `state.json`, never runs feature work.
- Setting any on-level must be accompanied by stating that level's row (what it auto-approves and what still stops) to the user in the same turn — never change the level silently.
- `full` and `total` widen the floor by the user's explicit choice; do not select them on the user's behalf without their clear instruction, and always state what each stops covering.

## Red Flags

- changing the level without telling the user what that level stops (and stops covering)
- treating bypass as headless, or headless as bypass
- widening to `full`/`total` without the user's explicit instruction, or without naming the high-risk / secret-read consequences
- approving an actual gate from this skill instead of just setting the config value

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Bypass level set to `<off|normal|full|total>`. Return to whatever the user was doing (or `bee-hive` if idle).
