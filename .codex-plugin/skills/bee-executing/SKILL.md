---
name: bee-executing
description: >-
  Implement and cap exactly one parent-assigned cell as a worker. Use when running inside a swarming worker that received an assigned cell id.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Workers read and cap cells through the vendored .bee/bin helpers.
---

# Executing — Worker Bee

You are a short-lived worker subagent. Execute exactly one parent-assigned cell, cap it, release reservations, and return a structured result. Never wait silently — when you cannot safely finish, return `[BLOCKED]` or `[HANDOFF]`.

```text
Initialize -> Accept assigned cell -> Reserve -> Implement -> Commit -> Cap -> Release -> Return
```

Open `references/worker-details.md` for expanded commands, trace tiers, friction triggers, and result fields.

## 1. Initialize

- Read `AGENTS.md`.
- Run `node .bee/bin/bee.mjs status --brief --json`
- Read `docs/history/<feature>/CONTEXT.md`.
- Read the cell: `node .bee/bin/bee.mjs cells show --id <id>`
- Use the parent-provided agent nickname as your reservation identity.

## 2. Accept Assigned Cell

- Require exactly **one** assigned cell id from the parent. Never choose work yourself — do not browse `ready` or `list` for candidates.
- No assigned cell id, or the cell is missing/already capped → return `[NOOP]`.
- The cell is ambiguous, its deps are not capped, or it conflicts with locked decisions in CONTEXT.md → return `[BLOCKED]`. Never reinterpret a locked decision to make the cell fit.
- **Validate, never claim (D1):** the orchestrator already claimed this cell (`cells claim --id` or `claim-next`) before spawning you. Confirm it: `node .bee/bin/bee.mjs cells show --id <id>` must show `status: "claimed"` with `trace.worker` matching your nickname. A worker never runs `cells claim` itself — anything else (open, claimed by a different worker, missing, capped) is not yours to touch → return `[BLOCKED]` (or `[NOOP]` per the rule above), never claim it yourself to make the cell fit.

## 3. Reserve

- Reserve **every** file or glob before writing:
  `node .bee/bin/bee.mjs reservations reserve --agent "<name>" --cell "<id>" --path "<path>" --ttl 3600`
- Any conflict → stop and return `[BLOCKED]` with the paths and holder. Never edit through a conflict.
- Prefix write-heavy shell commands with `BEE_AGENT_NAME="<name>"`.

## 4. Implement

- Read every file before editing it. Start from the cell's `read_first` list.
- Match existing patterns and the cited locked decisions (D-IDs).
- No stubs, TODO-only placeholders, dead code, or pseudo-implementations.

**Deviation rules** — when reality disagrees with the cell:

1. Found a bug in touched code → **auto-fix**, record as a deviation.
2. Missing critical functionality the cell's outcome depends on → **auto-add**, record as a deviation.
3. Blocking issue (broken import, type error in the path) → **auto-fix**, record as a deviation.
4. Architectural change needed → **STOP**, return `[BLOCKED]` with the proposal. Never redesign inside a cell.

Package installs **always** checkpoint: stop and return `[BLOCKED]` with the package and reason — never install on your own authority.

## 5. Commit

- Implement, then commit — one commit per cell, cell id in the message. `[DONE]` carries the diff and commit, never verify output (main-verifies D4).
- The cell's `verify` field is MAIN's command, not yours: never run it, never cite its output as evidence.
- Bugfix cells: the repro red is already MAIN-produced pre-dispatch and cited in the cell — fix it, never re-prove it yourself.
- Classic path (still sanctioned for spot use, other repos, or transition): run the cell's targeted `verify` command and record it with output before cap. Full matrix, amendment history, test-shape rules, and the debug discipline: `references/worker-details.md` ("Verify in full").

## 6. Advisor Consult

High-risk/hard-gate cells require a recorded advisor consult before the execution gate — the CLI throws without a fresh `advisor_ref` (AO3/AO13; staleness is hash-and-decision-anchored, never a TTL). Resolve the advisor from config, run it read-only with the evidence bundle on stdin, record via `bee state advisor-ref record`. Advice never approves a gate and never overrides a locked decision. Full mechanics, digest shape, and the consult prompt: `references/worker-details.md` ("Advisor consult in full").

## 7. Cap

- Default: cap through the sanctioned pending path — no per-cell verify evidence required:
  `node .bee/bin/bee.mjs cells cap --id <id> --feature-verify-pending --outcome "<summary>" --files <a,b> [--deviations-file <f>] [--friction "<text>"]`
- Classic path: cap only after a verify pass is recorded (`cells verify`); `behavior_change: true` cells add `--behavior-change --evidence-stdin` and pipe `verification_evidence` (tests inspected, tests added/changed, red-failure/before-state evidence, verification run) — never a written evidence file (decision 0009).
- If any Advisor Consults happened on this claim, fold their count and advisor identity into the trace — no separate file, same decision 0009 rule.
- Trace depth follows the cell's lane (tiny = one line; high-risk = full trace). Record friction when a trigger fired.

## 8. Release

`node .bee/bin/bee.mjs reservations release --agent "<name>" --cell "<id>"`

## 9. Return

- Start your final message with exactly one of `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, `[NOOP]`, followed by the result fields.
- Write a **short** per-cell report to `docs/history/<feature>/reports/<cell-id>.md`: the status token, a one-line outcome, files touched, and a link to `.bee/cells/<cell-id>.json` for the full trace/evidence. Never re-embed the `verification_evidence` JSON or verify output (decision 0009 — the trace is the single source), and never a separate scratch file (docs/specs/doctrine-layer.md R17).
- If any Advisor Consults happened on this claim, add a **Consults** section to the report: the count, the advisor identity per consult, and a one-line ask/answer digest each — this is the field bee-swarming's goal-check reads (A2). No consults happened → omit the section entirely.

## Compaction

At roughly 65% context before a safe finish: write `.bee/HANDOFF.json` (cell, files, done, remaining, next_action), release reservations that are safe to release, and return `[HANDOFF]`. After compaction, reread `AGENTS.md`, `CONTEXT.md`, the cell, and your active reservations before continuing.

## Fresh-Session Handoff (downstream, not a worker action)

This `[HANDOFF]` is the pause kind — unrelated to the planned-next handoff (fresh-session-handoff D1). When this cell caps (pending or classic-evidenced) and further execution-approved work remains, the orchestrator continues in-session; only at real session exit does it run the finish → claim-next → planned-next handoff flow for the next fresh session to adopt silently (no-clear-stop D1) — never a stop or a `/clear` prompt to the user, and never something a worker claims or writes mid-swarm on its own initiative. A worker's job stays exactly Cap → Release → Return.

## Headless

Workers always run effectively headless: never ask the parent or user a blocking question. Unambiguous deviations are applied under the rules above; anything ambiguous becomes `[BLOCKED]` with an `Outstanding Questions` section in the report. Workers never approve gates — Gate decisions belong to the user via the orchestrator chain. This rule is unchanged by Advisor Consult (A4): consulting a configured advisor stays inside your own turn and is never "asking the parent or user."

## Red Flags

- editing outside reserved scope
- selecting your own cell, or handling more than one
- waiting silently instead of returning a status
- capping without a verify pass or `--feature-verify-pending`, or "verifying" via a substitute command
- recording `--passed true` with no output — small+ lanes refuse the cap; an assertion is not evidence
- `--files` left empty on a cell that touched files — the trace is the machine-readable record, not the outcome prose
- a `behavior_change` cell capped without verification evidence
- installing packages without a checkpoint
- leaving reservations active without reporting it
- reinterpreting a locked decision to make the cell fit
- consulting the advisor with no `Advisor` line in the dispatch, or consulting on an authority-type block instead of instant `[BLOCKED]`
- a model-shaped consult dispatched without the exact `advisor-consult <cell-id>: <advisor-model>` description prefix — it breaks the A2 attribution record
- treating advisor advice as a substitute for fresh verify output, or capping consults without a Consults section in the report

Violating the letter of the rules is violating the spirit of the rules.

One status token returned and the report written; the parent orchestrator collects it. Invoke bee-swarming skill to continue the wave.

## Reference Files

| File | When to Load |
|---|---|
| `references/worker-details.md` | Expanded commands, trace tiers, friction triggers, result fields, evidence example |
