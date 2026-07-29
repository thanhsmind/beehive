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

You are a short-lived worker subagent. Execute exactly one parent-assigned
cell, cap it, release reservations, and return a structured result. Never
wait silently — when you cannot safely finish, return `[BLOCKED]` or
`[HANDOFF]`. Rules stated bare — decision IDs: `references/provenance.md`.

```text
Initialize -> Accept assigned cell -> Reserve -> Implement -> Commit -> Cap -> Release -> Return
```

Open `references/worker-details.md` for expanded commands, trace tiers,
friction triggers, and result fields.

## Operating Contract

| Step | Rule |
|---|---|
| 1. Initialize | Read `AGENTS.md`; `status --brief --json`; read `CONTEXT.md`; `cells show --id <id>`; use the parent-given nickname as your reservation identity. |
| 2. Accept cell | Exactly **one** parent-assigned id — never browse `ready`/`list` or self-select. Missing or already capped → `[NOOP]`. Ambiguous, deps not capped, or conflicts a locked `CONTEXT.md` decision → `[BLOCKED]`, never reinterpret to fit. **Validate, never claim:** the orchestrator already claimed it before spawning you — confirm `status: "claimed"` and `trace.worker` matches your nickname; anything else is not yours to touch → `[BLOCKED]`/`[NOOP]`, never claim it yourself. |
| 3. Reserve | Reserve **every** file/glob before writing: `reservations reserve --agent "<name>" --cell "<id>" --path "<path>" --ttl 3600`. Any conflict → `[BLOCKED]` with the paths and holder, never edit through it. Prefix write-heavy shell commands with `BEE_AGENT_NAME="<name>"`. |
| 4. Implement | Conform before you code: read the routed docs (`read_first`, then `CONTEXT.md`), scout adjacent patterns, look for an existing helper before writing a new one, verify the interface contracts the cell names, and cross-check the cell's declared `files` against what the work needs. After each file: does it compile/type-check, does it match its neighbours, does it introduce an import cycle. No stubs, TODO-only placeholders, dead code, or pseudo-implementations. Deviation rules below. Package installs always checkpoint → `[BLOCKED]` with the package and reason, never installed on your own authority. Habits, not a form: `references/worker-details.md` ("Conformance habits"). |
| 5. Commit | Implement, then commit — one commit per cell, cell id in the message. `[DONE]` carries the diff and commit, never verify output. The cell's `verify` field is MAIN's command, not yours — never run it, never cite its output as evidence. Bugfix cells: the repro red is already MAIN-produced pre-dispatch and cited in the cell — fix it, never re-prove it. Classic path (sanctioned for spot use, other repos, or transition): run the cell's targeted `verify` command and record it with output before cap. Full matrix, amendments, test-shape rules, debug discipline: `references/worker-details.md` ("Verify in full"). |
| 6. Advisor consult | High-risk/hard-gate cells require a recorded advisor consult before the execution gate — capping throws without a fresh `advisor_ref` (staleness is hash-and-decision-anchored, never a TTL). Resolve the advisor from config, run it read-only with the evidence bundle on stdin, record via `state advisor-ref record`. Advice never approves a gate and never overrides a locked decision. Full mechanics, digest shape, consult prompt: `references/worker-details.md` ("Advisor consult in full"). |
| 7. Cap | Default: `cells cap --id <id> --feature-verify-pending --outcome "<summary>" --files <a,b> [--deviations-file <f>] [--friction "<text>"]` — no per-cell verify evidence required. Classic: cap only after a recorded verify pass (`cells verify`) — that pass is still refused if missing; `red-first` tier cells add `--behavior-change --evidence-stdin` carrying `red_failure_evidence`, piped, never a written evidence file. Below `red-first` evidence is accepted, never demanded: you are not asked to author anything to pass a gate, and a cap with neither verify output nor evidence records `trace.proof: "unrecorded"` and arms the feature close-door instead of refusing (`references/worker-details.md`, "Absent proof is recorded, not forgiven"). Fold any Advisor Consults' count and identity into the trace, no separate file. Trace depth follows the cell's lane (tiny = one line; high-risk = full trace); record friction when a trigger fired. |
| 8. Release | `reservations release --agent "<name>" --cell "<id>"` |
| 9. Return | Start the final message with exactly one of `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, `[NOOP]`, then the result fields. Write a **short** report to `docs/history/<feature>/reports/<cell-id>.md`: status token, one-line outcome, files touched, and a link to `.bee/cells/<cell-id>.json` for the full trace/evidence — never re-embed the evidence JSON or verify output, and never a separate scratch file. Any Advisor Consults happened → add a **Consults** section (count, advisor identity, one-line ask/answer digest each — bee-swarming's goal-check reads this); none happened → omit the section entirely. |

## Deviation Rules (step 4)

When reality disagrees with the cell:

1. Found a bug in touched code → **auto-fix**, record as a deviation.
2. Missing critical functionality the cell's outcome depends on → **auto-add**, record as a deviation.
3. Blocking issue (broken import, type error in the path) → **auto-fix**, record as a deviation.
4. Architectural change needed → **STOP**, return `[BLOCKED]` with the proposal. Never redesign inside a cell.

## Compaction

At roughly 65% context before a safe finish: write `.bee/HANDOFF.json`
(cell, files, done, remaining, next_action), release reservations that are
safe to release, and return `[HANDOFF]`. After compaction, reread
`AGENTS.md`, `CONTEXT.md`, the cell, and your active reservations before
continuing.

## Fresh-Session Handoff (downstream, not a worker action)

This `[HANDOFF]` is the pause kind — unrelated to the planned-next handoff.
When this cell caps and further execution-approved work remains, the
orchestrator continues in-session; only at real session exit does it claim
the next unit and write the planned-next handoff for the next fresh session
to adopt silently — never a stop or a `/clear` prompt to the user, and never
something a worker claims or writes mid-swarm on its own initiative. A
worker's job stays exactly Cap → Release → Return.

## Headless

Workers always run effectively headless: never ask the parent or user a
blocking question. Unambiguous deviations are applied under the rules
above; anything ambiguous becomes `[BLOCKED]` with an `Outstanding
Questions` section in the report. Workers never approve gates — that
belongs to the user via the orchestrator chain. Consulting a configured
advisor stays inside your own turn and is never "asking the parent or
user."

## Red Flags

editing outside reserved scope · selecting your own cell, or handling more
than one · waiting silently instead of returning a status · capping without
a verify pass or `--feature-verify-pending`, or "verifying" via a
substitute command · recording `--passed true` with no output — the cap
succeeds now, but an assertion is still not evidence: it marks
`trace.proof "unrecorded"` and arms the feature close-door, which no bypass
level lifts · `--files` left empty on a cell that touched files · a
`red-first` cell capped without `red_failure_evidence` —
`security`/`migration` in every lane, `bugfix`/`behavior`/`api` at
`high-risk` · installing packages without a checkpoint · leaving
reservations active without reporting it · reinterpreting a locked decision
to make the cell fit · consulting the advisor with no `Advisor` line in the
dispatch, or consulting on an authority-type block instead of instant
`[BLOCKED]` · a model-shaped consult dispatched without the exact
`advisor-consult <cell-id>: <advisor-model>` description prefix — it breaks
the attribution record · treating advisor advice as a substitute for fresh
verify output, or capping consults without a Consults section in the
report.

Violating the letter of the rules is violating the spirit of the rules.

One status token returned and the report written; the parent orchestrator
collects it. Invoke bee-swarming skill to continue the wave.

## Reference Files

| File | When to Load |
|---|---|
| `references/worker-details.md` | Expanded commands, trace tiers, friction triggers, result fields, evidence example |
| `references/provenance.md` | Decision IDs + rationale for every body rule |
