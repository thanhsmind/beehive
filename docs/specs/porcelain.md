# Porcelain surface — the flow verbs

The CLI has two surfaces. **Porcelain** is the small set of flow verbs an
agent needs to run the bee lifecycle; it is what `bee --help` shows.
**Plumbing** is everything else — still shipped, still invocable, listed
only under `bee --help --all`. Nothing is renamed or removed by this split;
it is a presentation contract that keeps the default surface small enough
to hold in a prompt.

Every porcelain verb obeys the **teach-at-point-of-contact contract**: its
text output (and its refusals) end with the next action in plain language —
what to run, or what decision the agent now owes. An agent should be able
to complete a tiny task from `orient` plus the outputs of the verbs it is
led through, with no skill preloaded.

## Porcelain set (v1, 16 verbs)

| Verb | Role in the flow |
|---|---|
| `bee status` | Full snapshot when routing work. |
| `bee orient` | Session-start context packet: where am I, what is locked, what is next. |
| `bee doctor` | Runtime health. |
| `bee state route` | Record the triage/lane classification. |
| `bee state gate` | Record a gate approval. |
| `bee cells add` | Persist shaped work (after the gate). |
| `bee cells ready` | What is claimable now. |
| `bee cells show` | One cell in full. |
| `bee dispatch prepare` | Build a worker dispatch payload. |
| `bee cells finish` | Worker completion: cap + release reservations in one verb. |
| `bee reservations reserve` | Claim write scope before editing. |
| `bee decisions log` | Record an agreement the moment it settles. |
| `bee decisions active` | The decisions currently in force. |
| `bee capture add` | Queue a learning/knowledge stub. |
| `bee backlog add` | Park future work. |
| `bee close` | Feature close driver: debts → verify → what remains. |

Everything not listed is plumbing. Registry entries carry
`surface: 'porcelain' | 'plumbing'`; a missing field reads as plumbing.

## Help behavior

- `bee --help` (text) and `bee --help --json`: porcelain entries only. Text
  ends with a footer naming the count of remaining commands and the `--all`
  flag; the JSON manifest carries `{schema_version, surface: "porcelain",
  total_commands, commands}` where `total_commands` counts the full
  registry.
- `bee --help --all` (text or `--json`): the full registry, each entry
  carrying its `surface` value.
- Group- and command-scoped help (`bee <group> --help`) is unchanged: it
  shows every match in the group regardless of surface.
- Manifest drift detection keeps hashing the FULL registry — the split is
  presentation only and must not mask a plumbing change.

## New verb: `bee orient`

Read-only. The one command a session (or worker) runs to know where it is.
Supersedes prose reading-orders in skills: the packet IS the context
assembly.

JSON shape:

```json
{
  "where":     { "phase", "feature", "mode", "gates", "gate_bypass_level" },
  "decisions": { "context_md", "active_count", "recent": ["<=3 one-liners"] },
  "work":      { "cells": { "open", "claimed", "capped" },
                 "ready": ["<=5 ids"],
                 "blockers": ["pending handoff, debts, stale reservations"] },
  "next":      { "action", "skill", "command" }
}
```

- `next.action` reuses status's recommended next step.
- `next.skill` maps the phase to the skill to load (exploring →
  bee-exploring, planning → bee-planning, swarming → bee-swarming,
  scribing → bee-scribing, compounding → bee-compounding, otherwise
  bee-hive).
- `next.command` is the runnable command when one applies, else null.
- Text output: at most six lines — one per section — ending with
  `next: <action>`.
- Implementation reuses the status builder; orient never computes state a
  second way.

## New verb: `bee cells finish`

The worker's single completion verb. Accepts the same parameters as
`cells cap` (same schema, same proof rules — nothing weakened or added),
and after a successful cap releases every reservation held by the cell's
claiming agent for that cell. Cap refusals pass through unchanged. The
result reports what was capped and which reservation paths were released,
and its text ends by telling the worker what to return
(`[DONE] …` with the expected fields).

`cells cap` and `reservations release` remain available as plumbing — a
failed finish can always be completed stepwise.

## Extended verb: `bee dispatch prepare --claim`

One verb from "cell chosen" to "worker prompt in hand". With `--claim`, the
verb first claims the cell (the same door as `cells claim` — refusals pass
through unchanged), then reserves every path in the cell's `files` for the
worker nickname (default TTL), then builds the payload exactly as today.
On a reservation conflict the claim is unwound and the refusal names the
conflicting paths and holder — state is left as it was found. The result
gains `{claimed: true, reserved: [paths]}`. Without `--claim`, behavior is
unchanged (cell must already be claimed). Workers keep `reservations
reserve` for extra paths discovered mid-work.

## New verb: `bee close`

The feature close driver — one verb that answers "what stands between this
feature and done, and can we pay it now".

- `bee close --feature <slug> --dry-run`: read-only report of the close
  doors — pending feature-verify cells, test-cell debt, scribing/capture
  debt — each with the exact command that settles it.
- `bee close --feature <slug>`: same checks; when the only outstanding door
  is the feature verify and a verify command is recorded for the feature,
  run it, record the pass/fail through the existing feature-verify
  recorder, and report the result. On green, the text names the capture
  checklist (what settles into decisions/knowledge) and the next skill; on
  red, the failing output is surfaced and nothing is recorded as passed.
- close never bypasses a door: debts it cannot pay are reported with their
  commands, never waived. Implementation reuses the exact door predicates
  the existing close path enforces — no second implementation of any debt
  rule.

## Compatibility

- All existing command names keep working; no removals, no renames.
- `AGENTS.md` documents `bee --help --json` as the porcelain manifest and
  `--all` as the full surface.
- Tests that pinned the old full-manifest default move to `--all`; example
  strings in the registry stay runnable as-is.
