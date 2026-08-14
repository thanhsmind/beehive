# Handbook Currency — Brief

**Feature slug:** handbook-currency
**Date:** 2026-08-14
**Lane:** docs
**Shaping:** short brief (docs lane)

## What this is

The handbook drifted behind two features that shipped today (`traceable-runs`, `awaiting-human`). An audit against the merged code found wrong statements, missing rows, and stale counts. This corrects the handbook to match reality. No runtime code changes.

## What is wrong, from the audit

**Three places state a falsehood I created today.** `overview.md:230`, `index.md:88`, and `architecture-map.md:164` all say the docs lane has *no gates*. It now has Gate 1 — a short brief and a recorded approval — which is exactly the doctrine that made this very brief mandatory.

**`register.md` is the most stale file:**
- `:27` describes a gate entry as `{approved, approved_for_plan_rev}`. It now also carries `state` (`pending`/`approved`/`rejected`), `actor`, `at`, `reason`, `bypass_level`.
- The `.bee/state.json` key table has no row for `run_state` or for `waiting_on`.
- `.bee/deferred-queue.jsonl` has no register entry at all, though it sits beside `.bee/backlog.jsonl` and `.bee/decisions.jsonl` as an event-sourced store.
- The write-guard invariant does not name the gated-vs-intake prefix split, so it cannot explain why a `docs/` write outside `docs/history/` now refuses at a gated phase.
- The doors table has no row for `worktree merge`'s bookkeeping auto-commit.
- `bee gate`'s row does not mention `--actor` / `--bypass-level` / `--reason`.
- **Counts are wrong, measured fresh against `generated/registry_payload.json`:** the register says *"141 entries, 18 of them porcelain"* and *"20 registry entries"* declared-but-not-built. Real numbers are **150 entries**, **19 porcelain**, **18 unavailable**.

**Smaller gaps:** `architecture-map.md:22,31` and `overview.md:184` summarise `state.json` as `phase · gates · feature`, omitting the two new fields; `architecture-map.md:150`'s merge sequence omits the auto-commit step; `architecture-map.md:246-247` and `stages/hive.md:40-45` omit `waiting_on` from the preamble/state surfaces; `overview.md:95-99` and `index.md:82-83` describe gates without the new trace fields or the rule that bypass scopes the stop, never the record.

Files with no drift, checked and left alone: `using-as-planner.md`, `writing-skills.md`, `writing-skills-references/*`, `evolving.md`, and the `exploring`/`reviewing`/`executing`/`scribing`/`compounding` stage files. `planning.md` and `swarming.md` reference `approved_gates.shape`/`.execution`, which are still accurate — those are the boolean projection, unchanged.

## Boundary

Correct what is wrong and fill the named gaps in five files: `register.md`, `architecture-map.md`, `overview.md`, `index.md`, `stages/hive.md`. No restructuring, no new handbook file — every item has an existing heading. The one genuinely new element is a `.bee/deferred-queue.jsonl` register row.

Every count goes in measured, not estimated.

## Out of scope

- The knowledge bundle under `docs/knowledge/` — already synced by the two features' own capture passes.
- `docs/07-contracts.md` and `docs/02-architecture.md` — flagged by an earlier audit as carrying the same shapes, but not part of this request; note them for a follow-up rather than widening silently.
- Any runtime code.
