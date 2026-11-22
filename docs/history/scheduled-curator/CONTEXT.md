# Scheduled curator — Context

**Feature slug:** scheduled-curator
**Date:** 2026-11-22
**Scope:** Standard
**Domain types:** RUN, CONFIG

## What was asked

The owner (2026-11-22): "Thêm tính năng cho setup chạy curator theo từng dự
án và theo lịch" — set up the curator to run per project and on a schedule.

Source idea: Harrison Chase, "How to build a knowledge factory" (LangChain,
2026-11-21). Agents that use knowledge flag gaps well and fix them badly; a
separate curator runs in the background on a cadence and drains the flags.
bee agents flag well (capture stubs, promote proposals, scribing debt), and
nothing drains that debt on a schedule. The manual drain of 2026-11-22
(`docs/history/curator-drain-2026-11-22/audit.md`) is the method.

## What will be done

Each project turns its curator on and sets its cadence in `.bee/config.json`.
`bee upkeep due` gains a `curator` item that is due when the curator is on,
the cadence has elapsed, and a debt count reaches its threshold. A new skill,
`bee-curating`, runs one bounded drain pass and records it. The work-queue
verb group that settles a promote proposal joins the command registry. The
schedule tick itself is waggledance's (handed to its leader).

## Locked Decisions

<!-- bee:not-a-deferral: names the deferred-queue verb, a command name, not a promise to act -->

| ID | Decision | Store id |
|---|---|---|
| D1 | Per-project `curator` block in `.bee/config.json`: `enabled` (bool, default false), `cadence_hours` (integer 1..720, default 24), `kinds` (subset of `capture`, `promote`, `scribing`; default all three), `thresholds` {`capture`, `promote`, `scribing`} (integer >= 1, default 1 each). A missing or bad value falls back to its default, like `herding.ceiling_seconds`. Setup is `bee config set curator.enabled true` plus any cadence or threshold key; no new setup verb. | bca6b659 |
| D2 | `bee upkeep due` gains kind `curator`: skill `bee-curating`, invoke `/bee-curating`, audit kind `curator-run`. `present` = `curator.enabled`. Debt reasons, one per enabled kind whose count reaches its threshold: `capture_queue`, `promote_unsettled`, `scribing_orphaned`. `due` = present, at least one debt reason, and the cadence elapsed (no `curator-run` row, or the newest row older than `cadence_hours`). Debt inside the cadence window shows under `info` as `cadence_not_elapsed` and does not set due. Every item gains `cadence_hours` (null for kinds without one). `--kind` accepts `curator`; the router help lists all four kinds. | 9d7b5180 |
| D3 | New skill `bee-curating`, `disable-model-invocation: true`, one bounded pass: (1) flush the capture queue by the bee-capturing Compound step; (2) settle up to 10 unapplied promote proposals, oldest first: apply what belongs to `docs/knowledge/`, then `bee deferred-queue complete --kind promote --feature <f>`; (3) sync up to 5 orphaned-scribing features, heaviest first, then `bee state scribing-run --feature <f> --areas "<a,b>" --next-action "<n>"` (ledger-only for a feature that is not active). Read-only gathers may fan out the reading. It commits the bundle change and ends on every outcome with `bee audit log --kind curator-run --feature curator --note "<clean|changed|blocked> — <one line>"`. | dec37fc8 |
| D4 | The work-queue verb group of `verbs/deferred_queue.rs` (add, list, claim, release, complete) joins `registry_payload.json`, so help, the command surface and the CLI-shape guard know it. The preamble clear line stays; it is now a declared command. | 04671091 |
| D5 | The schedule tick lives in waggledance: its daemon runs `bee upkeep due --json` per registered project on a fixed tick and dispatches each due item's `invoke` to that project's leader, one dispatch per project, kind and window. bee owns no scheduler. The okf-profile overview "Scheduled upkeep" section and `docs/config-reference.md` describe the curator block and this setup. Handed to the waggledance leader as run `run-fe3c8669856e98a2`. | a9b01f87 |
| D6 | `docs/product-description/memory/knowledge.md` is corrected to match `skills/bee-capturing/SKILL.md`: `bee close` no longer writes `promote-proposals.md`; the existing files are legacy debt the curator settles. | b113f500 |

<!-- /bee:not-a-deferral -->

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/okf-profile/overview.md | suspect | holds the Scheduled upkeep how-to the curator joins (D5) |
| docs/knowledge/areas/decision-memory/audit-log.md | suspect | the closed audit kind set that gains `curator-run` (D2) |

The due signal this extends is `docs/history/upkeep-due-signal/CONTEXT.md`;
the method the skill encodes is `docs/history/curator-drain-2026-11-22/audit.md`.

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs` | the `Kind` enum, reasons and item JSON of `bee upkeep due` |
| `packages/bee-rs/crates/bee/src/audit.rs` | the closed audit kind set |
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/store.rs` | `capture_queue_count` and `global_scribing_debt` |
| `packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs` | `unapplied_promote_proposals` and the preamble clear command |
| `packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs` | the work-queue verbs D4 registers |
| `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | the verb registry |

## Out of scope

The waggledance tick (D5, its leader builds it). Fix-at debt as a curator
kind: it already has its own `fix-at` upkeep kind.
