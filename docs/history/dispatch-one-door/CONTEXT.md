# Context: dispatch-one-door

## Problem

A host repo (`vnbptw-mapcompany`) runs with `models.claude.generation = {kind:"herding", agent:"agy-flash", fallback:"default"}`.
Every subagent dispatch there is refused by `bee-model-guard`:

- `Agent(subagent_type: "bee-gather")` → `herding-tier-denied`
- `Agent(subagent_type: "Explore")` (bare) → `bare-denied`

The refusals are correct — a PreToolUse hook can only allow or deny, it cannot rewrite an
Agent call into the Bash call a herding pane needs. The defect is upstream of the guard:
the agent was told, by prose, to name a `subagent_type` instead of asking the config.

Evidence (2026-08-21, this repo, same herding config):

```
bee dispatch prepare --runtime claude --kind gather --json
→ {"tool":"Bash","payload":{"command":".bee/bin/bee herding run --task-file - --json --agent \"agy-flash\""},
   "economics":{"channel":"herding-exec","enforcement":"herding-command"}}
```

`bee dispatch prepare` already resolves the slot correctly for every shape. Nothing else does.

## Drift found

| # | Where | Stale claim | Reality |
|---|---|---|---|
| 1 | `skills/bee-hive/references/gates-and-delegation.md:130` | "`subagent_type: bee-build\|bee-gather\|…` … prefer this shape" | Refused outright when the slot is `cli`/`herding` |
| 2 | `skills/bee-hive/references/gates-and-delegation.md:141-146` | "a gather never dispatches through a herding pane (herding-executor D7)" | `herding-tier D1-D6` widened it; `dispatch prepare --kind gather` returns a herding payload |
| 3 | `skills/bee-swarming/references/swarming-reference.md:110-113` | cli-shaped or unrendered slot → "spawn the runtime's default/general subagent type" | `general-purpose` at `generation` is denied (`generic-type-denied`) |
| 4 | `skills/bee-swarming/references/swarming-reference.md:300` | same, in table form | same |
| 5 | `skills/bee-swarming/references/swarming-reference.md:380-384` | "Scope B (a `{kind:\"herding\"}` tier kind in `models.*`) … does not exist yet" | It exists, is implemented, and is what the reporting host runs |
| 6 | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs` (bare-denied FIX) | "the generation tier is a cli executor or unconfigured" | The slot is `herding`; no FIX message names `bee dispatch prepare` |
| 7 | `packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:892-952` | `AGENT_FILE_TIER` covers gather/extract/review; drift text says "cli-shaped or unconfigured" | `bee-build.md` is rendered too and goes unchecked; a herding slot is mislabelled |

## Locked decisions

**D1 — one door.** Every subagent dispatch resolves its transport from `.bee/config.json`
through ONE verb, `bee dispatch prepare`, never from prose that names a `subagent_type`.
`prepare` reads the tier slot and returns the payload:

| Slot shape | Payload |
|---|---|
| model (`"sonnet"`, `{model,effort}`) | Agent/Task, naming the rendered bee agent |
| `{kind:"herding"}` | Bash — `bee herding run --task-file - --json` |
| `{kind:"cli"}` | Bash — the configured command verbatim, prompt on stdin |

Consequences:

- (a) the Transport bullet names `prepare` as the door; `subagent_type` survives only as
  something `prepare` RETURNS, never as the shape a reader is told to prefer;
- (b) the `herding-executor D7` gather boundary is retired;
- (c) "unrendered tier → `general-purpose`" is retired;
- (d) every model-guard FIX message names `bee dispatch prepare` as the remedy, and stops
  calling a herding slot "a cli executor or unconfigured".

Rationale: three documents gave three different answers for the same dispatch and all
three disagreed with the code. One verb that reads the config is the only shape that
cannot drift. Rejected alternative: patch the prose with an extra `cli`/`herding` branch
and keep `subagent_type` preferred — the agent still has to remember what the config says,
so the same refusal returns whenever an operator changes a slot.

Source: user, 2026-08-21. Logged via `bee decisions log` (tags: dispatch, model-guard,
herding, delegation).

**D2 — direct naming stays legal.** `prepare` is the door a reader is pointed at; the guard
keeps allowing a direct `subagent_type: bee-*` when the slot is model-shaped. D1 removes the
prose that *recommends* guessing, not the guard branch that accepts a correct guess.
