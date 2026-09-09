# Team Config Rename — Context

**Feature slug:** team-config-rename
**Date:** 2026-09-09
**Shaping session:** complete
**Scope:** Standard
**Domain types:** ORGANIZE | READ

## Feature Boundary

The `.bee/config.json` block `models.<runtime>.<role>` — the leader's roster of
which agent does which job — is renamed `team.<runtime>.<role>`, with the old
key kept as a read alias that warns; `bee models show` becomes `bee team show`
with the old verb kept as an alias. Values, slot shapes, resolution rules and
the `herding.agents` registry are untouched — this feature changes a name and
adds one accessor, nothing about what runs.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `models.<runtime>.<role>` is renamed **`team.<runtime>.<role>`**. Values and slot shapes are unchanged. `bee models show` becomes **`bee team show`**. (decision logged 2026-09-09, touches `06e49368`) | Most slot values are `{kind: herding, agent: <key>}` — a *who*, not a model — and the old name made agents read the block as a model-id list and hand-pick `model` params, the exact error the model-guard hook exists to catch. `team` was chosen over the owner's first spelling `peer-agents` because `4a6e38be` already fixes *peers* = the execution roles only, and `herding.agents` next door already means *how to launch*. |
| D2 | The old `models` key stays readable as an **alias**, resolved through **one accessor** — `team` first, `models` as fallback — never two spellings at a call site. A config still carrying `models` gets **one warning line from `bee doctor` and one from the session preamble** naming the new key. Removing the alias needs its own release and decision. (decision logged 2026-09-09) | 123 Rust reads across 12 files and 99 docs/skills files name the old key; every host repo's `.bee/config.json` carries it. One accessor is the single-source-of-truth shape. |

### Agent's Discretion

- The accessor's name and module (it belongs beside `resolve_role` in the drivers module, per B1 "one parser owns the shape").
- The exact wording of the two warning lines.
- Whether `bee models show` prints its deprecation line to stderr or as a trailing stdout line.
- How the 99-file docs sweep is sliced — but every file that names the old key must end by naming the new one, and the alias must be documented once in `docs/config-reference.md`.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| team | The roster: for each runtime, which agent (or native subagent + model) does which job role, and what it is good at (`description`). Answers *who*. |
| herding.agents | The registry: how to launch a named agent program (argv, env, trust). Answers *how to start*. Unchanged. |
| role | A job name inside the team block (`code`, `review`, `advisor`, …). Unchanged, still an open set. |
| alias | The old `models` key, read only when `team` is absent. When both are present, `team` wins and `models` is ignored with a warning. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` — `resolve_role` and the slot normalizer; the one parser B1 names. The accessor lives beside it.
- `packages/bee-rs/crates/bee/src/verbs/models_group.rs` — the `models show` verb; becomes `team show` with `models show` aliased.
- `bee doctor` and the session preamble (`hooks/session_preamble/`) — both already emit advisory lines; D2's warnings ride the existing shapes.

### Established Patterns

- **Read-time alias, never a rewrite** — the review store already normalizes legacy `gate4` → `review` on read (`reviews.rs`, "legacy gate4 still read and normalized"). Same posture here: no config file is rewritten by bee.
- **Warn, never refuse, on a legacy spelling** — B2's "a name nothing configures is warned on stderr, never a refusal".

### Integration Points — the raw readers that must all go through the accessor

*(Corrected 2026-09-09 after the plan-step hat wave; the original list counted a telemetry read as a config read and missed a writer.)*

Config enters the crate through **three loaders** — `state.rs:161`, `hooks/session_preamble/state.rs:50`, `hooks/compaction.rs:124` — two of which already normalize a key at load (`shift_remove("advisor")`). One reader bypasses them: `onboard/agents.rs:73`. Eleven production sites then read the loaded map by the old key: `doctor.rs:297`, `models_group.rs:149`, `status_full/build.rs:343`, `store.rs:448`, `:535`, `:853`, `model_guard.rs:80`, `compaction.rs:1525`, `herding/wave.rs:358`, `session_preamble/budget.rs:650` (the last two through one shared helper, `model_guard.rs:520`). A **writer** generates the old key for every fresh host: `onboard/templates.rs:175`, beside a compiled-in sample (`:264`). `hooks/session_close/perf.rs:576` reads a session usage record, **not** config, and must not be touched. A reader left on the raw key silently ignores a `team` block — that is the failure this feature must not ship.

## Canonical References

- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` — B1 (one parser), B2 (open set), B12 (`description`), B13 (`models show` is the one read door), B16 (`models.pi`).
- `docs/config-reference.md` — where the alias is documented once.
- `docs/history/research/bee-model-binding-stability-xia.md` — the research that surfaced the misreading.

## Outstanding Questions

### Resolve Before Planning

- None.

### Deferred To Planning

- [ ] Which of the fourteen sites are reachable from a host repo's hook path (model_guard, preamble, compaction) versus dev-only (perf) — the hook-path ones are the ones a wrong accessor breaks for every host.
- [ ] Whether `bee onboard` should write `team` into a fresh config and `.bee/config-sample.json`, so new hosts never see the old key.
- [ ] How the docs sweep proves itself — an `rg` count of `models.<runtime>` outside the alias paragraph must reach zero.

## Deferred Ideas

- Letting bee read the model out of a herding agent's argv (`bee-model-binding-stability-xia.md`, fix 3) — a separate feature.
- Injecting the resolved model into the worker brief, omp-style (fix 4) — separate.
- Removing the `models` alias — its own release, per D2.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
