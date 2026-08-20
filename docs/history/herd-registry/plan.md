# herd-registry — Plan

**Lane:** standard (public-contracts; ~7 product files)
**Source:** CONTEXT.md D1–D2
**Date:** 2026-08-20

## Slice 1 — 3 cells

| Cell | What | Files | Decisions |
|---|---|---|---|
| hr-1 | Registry + resolver: `herding.agents` reader (per-entry validation, same rules as agent_command); `resolve_agent_command` gains an optional agent-name argument — named lookup, unknown name typed error listing registry keys; `herding.agent_command` as a plain string resolves through the registry; `bee herding run --agent <name>` wired to it. Tests: named lookup, unknown-name error text lists keys, string agent_command alias, absent = default. | `herding/wave.rs`, `herding/run.rs`, `generated/registry_payload.json` (flag doc only, examples[0] unchanged) | D1, D2 |
| hr-2 | Tier seam: `normalize_tier_value` keeps a trimmed optional `agent` on kind:herding; `resolve_tier` carries it on `Resolved::Herding`; prepare's herding-exec arm appends `--agent "<name>"` when present. Tests both. | `verbs/drivers/models.rs`, `verbs/drivers/prepare.rs`, `verbs/drivers/tests.rs`, `hooks/model_guard.rs` (only if the Resolved variant shape forces it) | D2 |
| hr-3 | Docs: `herding.agents` in operational-invariants.md (canonical), models-section note in config-reference.md, registry example in both samples (commented, copy-paste-safe). wave-barrier regen ack. | `skills/bee-herding/references/operational-invariants.md`, `docs/config-reference.md`, `.bee/config-sample.json`, `.bee/config-sample-cli-executors.json` | D1, D2 |

Deps: hr-1 ∥ hr-2 (disjoint files; the `--agent` flag contract is fixed here in the plan); hr-3 after both.
Proof: scoped cargo test per cell; hr-1 adds `--test registry_dispatch`. Full suite before merge.

## Smaller-path check

Fold hr-2 into hr-1? Different files, parallel wins, contract fixed here. Stands: 3 cells.

## Rollback

Additive: configs without `herding.agents` or `agent:` hit no new path. Revert = merge revert.
