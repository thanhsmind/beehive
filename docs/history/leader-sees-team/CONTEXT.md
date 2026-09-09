# Leader Sees Team — Context

**Feature slug:** leader-sees-team
**Date:** 2026-09-09
**Shaping session:** complete
**Scope:** Standard
**Domain types:** SEE | READ

## Feature Boundary

Every surface the leader reads to pick a team member leads with the **job and
its description** and shows the transport (native model / herding pane / cli)
only as a trailing detail; bee **reads the model out of the herding or cli
argv it builds** and reports it as `declared`; and the always-loaded doctrine
bee renders into every host (`AGENTS.md` block, the delegation contract, the
worker brief) states the rule in one sentence. It ends there: no change to how
a slot resolves, how a worker is spawned, or what a role may run.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | **The leader's unit of choice is a team member by job, never a transport.** Whether a `team.<runtime>.<role>` slot is a native subagent, a herding pane or a cli executor is a config fact; the leader picks the role for the job and runs whatever the one dispatch door returns. Every surface the leader reads — the session preamble's dispatch-door line and `bee team show` — leads with job and description, and shows the transport only as a trailing detail. (logged 2026-09-09, touches `06e49368`, `3c9d6262`) | The door already behaves this way; the two read surfaces print `{kind:herding, agent:agy-flash}` / `generation=herding (agy-flash)` transport-first, which invites the leader to reason about transport. |
| D2 | For a herding or cli dispatch, bee **reads the model out of the argv it built** (the `--model` token of the resolved `herding.agents` entry, or the cli command) and records it as `requested_model` with a new `effective_model_status` value **`declared`** — "bee asked the external program for this model and can show the flag" — never `pinned`. An argv with no model token keeps `null` / `unverified`, so silence stays distinguishable. The preamble line and `bee team show` print the declared model beside the job. (logged 2026-09-09) | `derive_economics` hard-codes `unverified` and a null model for `herding-exec` (`guard.rs:381-391`) although the agent name is in hand one call earlier (`prepare.rs:1805`) and its argv is resolvable (`wave.rs:402-417`). |
| D3 | **The rule ships to every host.** The sentence "pick the team member by job; the transport is config" lands in the rendered `AGENTS.md` block, in the delegation contract the leader loads, and in the worker brief — one sentence each, one home per surface, never a paragraph. | A principle only in bee's own decision log does not reach a host that launches bee. |

### Agent's Discretion

- The exact rendered shape of the roster line and `bee team show` rows, within D1's ordering (job → description → declared model → transport).
- Where the argv model token is parsed (beside `resolve_agent_command_for_runtime` is the natural home).
- The exact one-sentence wording for each of D3's three homes.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| team member | One `team.<runtime>.<role>` slot: a job name, a description, and a way to run it. The leader chooses by the first two. |
| transport | How a member runs: native subagent with a model param, herding pane running a registry agent, or a cli command. Config, not choice. |
| declared | bee built an argv that names a model and can show the token. Weaker than `pinned` (a param bee controls), stronger than `unverified` (bee never looked). |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:374-391` — `derive_economics`, the one place status and `requested_model` are set.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:402-417` — `resolve_agent_command_for_runtime`, which already turns a registry key into its argv.
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:520-545` — `dispatch_door_lines`, the preamble roster line (`- Roles (claude): …`).
- `packages/bee-rs/crates/bee/src/verbs/models_group.rs` — `bee team show`'s row renderer and `TEACH` line.
- `packages/bee-rs/crates/bee/src/onboard/` — renders the host `AGENTS.md` block; the test `agents_md_block_is_the_rendered_source_byte_for_byte` pins it to source.

### Established Patterns

- **Status words are a closed, honest set** — `pinned` / `unverified` / `inherited-or-unknown` / `native-requested`; `declared` joins as one more, each with its exact condition (`guard.rs:374-389`).
- **One sentence, one home** — AGENTS.md is always-loaded context; every word costs every session.

### Integration Points

- `AGENTS.md` "Work in parallel, coordinate through the store" — the dispatch-door paragraph (the host block is rendered from it).
- `skills/bee-hive/references/routing-and-contracts.md` — the delegation contract.
- `skills/bee-swarming/references/worker-details.md` — the worker brief's model-shaped vs cli-shaped advisor text.
- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` — B12/B13 (display law, the read door).

## Canonical References

- `docs/history/research/bee-model-binding-stability-xia.md` — fix 3 and fix 4.
- `docs/history/team-config-rename/CONTEXT.md` — D1/D2 of the rename this builds on.

## Outstanding Questions

### Resolve Before Planning

- None.

### Deferred To Planning

- [ ] Whether the dispatch record (`.bee/logs/dispatch.jsonl`) gains `declared` only at prepare time or also from the model guard's audit line — read `guard.rs` and `model_guard.rs` for who else writes the status.
- [ ] Whether the cli executor's command string is parsed for `-m`/`--model` too, or only herding argv — the codex `review` slot is a cli command with `-m gpt-5.5`.

## Deferred Ideas

- Injecting the declared model into the worker brief itself, omp-style (fix 4) — a separate feature once `declared` exists to inject.
- Removing the `models` alias — its own release.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
