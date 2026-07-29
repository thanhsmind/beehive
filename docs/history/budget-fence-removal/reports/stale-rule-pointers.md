# Stale numbered-rule pointers — verified inventory

**Date:** 2026-07-29
**Feature:** budget-fence-removal (supports locked decision D9)
**Status:** every row below was read directly and confirmed against the current numbering. This is
an inventory of *known* drift, not a proof of completeness — see "Completeness" at the end.

## Why this exists

The `## Critical rules` list in `packages/bee/AGENTS.block.md` was renumbered during byte-budget
diets. Cross-references written before a renumber kept their old number. Decisions `0006` and
`0007` were written the same day (2026-07-08) and both drifted by exactly one position, which is
the signature of a list that lost an entry above them.

Two numbered lists exist and are easy to confuse:

- **Critical rules** — `packages/bee/AGENTS.block.md:35-52`, mirrored to `AGENTS.md:42-59`. 17 rules.
- **Priority Rules (hive law)** — `skills/bee-hive/SKILL.md:106-123`. A separate 1-14 list.

A pointer must say which list it means, or be unambiguous from context.

## Current numbering (verified 2026-07-29)

**Critical rules** (`packages/bee/AGENTS.block.md`):

| # | Gist |
|---|------|
| 1 | Never execute before the merged gate approves |
| 2 | Capping proves at the feature boundary (R82) |
| 3 | Cells assigned by the orchestrator; workers never self-select |
| 4 | Reserve files; prefix write-heavy commands with `BEE_AGENT_NAME=<name>` |
| 5 | Write `.bee/HANDOFF.json` and pause before context runs out |
| 6 | `CONTEXT.md` is truth; log decisions through the CLI |
| 7 | One commit per cell, cell id in the message |
| 8 | Lanes scale ceremony, never memory — capture on settle |
| 9 | The agent runs the machinery, not the user |
| 10 | Work language only, purpose first |
| 11 | The hook is a safety net, not the authority |
| 12 | Fan out the gathering; keep the deciding — mandatory transport marker |
| 13 | Multi-session etiquette: lanes, claims, holds; worktrees for occupied checkouts |
| 14 | CI status gate before the first `cells claim`; never build on red |
| 15 | Concurrency is the default; serial is the named exception |
| 16 | Never author an artifact whose only purpose is to be deleted as evidence |
| 17 | Progress ticks: one line per step, on by default |

**Priority Rules (hive law)** (`skills/bee-hive/SKILL.md`): 8 = lanes/capture · 9 = agent-runs-the-machinery
· 10 = work language + ticks · 11 = no hand-edits to `.bee/*.json(l)` · 12 = hooks are a safety net
· 13 = headless.

## Mismatches

Source of truth is `packages/bee/`. `.bee/bin/` is a byte-identical synced copy (`cmp` clean as of
2026-07-29) — fix the source, then re-sync; never hand-edit `.bee/bin/`.

### Shipped package

| # | Anchor | Says | Should be | Evidence |
|---|--------|------|-----------|----------|
| 1 | `packages/bee/lib/recovery.mjs:473` | `critical rule 13:` for the `[bee-tier: …]` transport marker | critical rule **12** | Rule 13 is multi-session etiquette. The "marker as the first thing" text is rule 12. |
| 2 | `packages/bee/bee.mjs:1743` | `rule 13's mandatory transport` | critical rule **12** | Same concept, same drift. |
| 3 | `packages/bee/bee.mjs:4563` | `rule 12's forbidden escape hatch` for hand-editing bee state | hive law **11** | Critical rule 12 is fan-out; hive law 12 is hooks-as-safety-net. The hand-edit ban is hive law 11. `packages/bee/lib/cells.mjs:1500` cites the same concept correctly as "rule 11's hand-edit fallback". |
| 4 | `packages/bee/bee.mjs:6089` | `AGENTS.md rule 14` for `bee worktree new --with-companion` | critical rule **13** | Rule 14 is the CI status gate. Worktrees for occupied checkouts are rule 13. Not reported by the fresh-eyes pass; found while verifying rows 1-3. |

### Scripts

| # | Anchor | Says | Should be | Evidence |
|---|--------|------|-----------|----------|
| 5 | `scripts/run_verify.mjs:492` | `AGENTS.md critical rule 5 mandates prefixing … BEE_AGENT_NAME` | critical rule **4** | Rule 5 is the HANDOFF pause. The `BEE_AGENT_NAME` prefix is rule 4. `packages/bee/lib/reservations.mjs:62` cites rule 4 correctly for the same concept. |
| 6 | `scripts/run_verify.mjs:500` | `survive rule 5's own prefix` | critical rule **4** | Carried over from row 5. |
| 7 | `scripts/tests/test_agents_budget.mjs:46-47` | quotes hive law as `"Rules 2-4, 13 appear in full in AGENTS.md"` and "its rule 13 pointing at `AGENTS.md Guardrails`" | `"Rules 2-4, 12 …"` / hive law **12** | `skills/bee-hive/SKILL.md:108` actually reads "Rules 2-4, 12 are in `AGENTS.md` (auto-loaded)"; hive law 12 is the Guardrails pointer. Hive law 13 is now headless. |

### Knowledge patterns

| # | Anchor | Says | Should be | Evidence |
|---|--------|------|-----------|----------|
| 8 | `docs/knowledge/patterns/20260713-promote-an-order-to-the-always-loaded-layer.md:18` | `Critical rule 13 (fan out the gathering)` | critical rule **12** | Fan-out is rule 12. |
| 9 | `docs/knowledge/patterns/20260715-the-bill-is-turns-prefix-keep-the-prefix.md:23` | `rule 13 fan-out` | critical rule **12** | Same. |

### Decision records

Both files additionally cite `skills/bee-hive/templates/AGENTS.block.md`, a path that no longer
exists — the template moved to `packages/bee/AGENTS.block.md`. `0006:23` also cites
`skills/bee-hive/templates/lib/inject.mjs`, which resolves today to `packages/bee/lib/inject.mjs`
(`fd inject.mjs skills packages` → one hit).

| # | Anchor | Says | Should be | Evidence |
|---|--------|------|-----------|----------|
| 10 | `docs/decisions/0006-agent-runs-the-machinery.md:21` | `Critical rule 10` for "agent runs the machinery" | critical rule **9** | That doctrine is rule 9; rule 10 is work-language. |
| 11 | `docs/decisions/0006-agent-runs-the-machinery.md:22` | `Priority rule 10` | hive law **9** | Hive law 9 is "The agent runs the machinery, never the user". |
| 12 | `docs/decisions/0007-unprompted-capture.md:21` | `rule 9 extended` for capture/detection duty | critical rule **8** | Capture is rule 8; rule 9 is the unrelated machinery rule. |
| 13 | `docs/decisions/0007-unprompted-capture.md:22` | `priority rule 9 extended` | hive law **8** | Hive law 8 is lanes/capture. |

## Deliberately excluded

- `docs/history/<feature>/**`, `.bee/cells/**`, `.bee/decisions.jsonl`, `.bee/backlog.jsonl`,
  `.bee/reviews/**` — frozen point-in-time work logs. They are expected to record the numbering as
  it stood when written. Several carry the same drift; none are corrected.
- `.bee/bin/**` — synced copy of `packages/bee/`. Rows 1-4 reproduce there and clear on re-sync.
- Generated skill mirrors under `.claude/`, `.agents/`, `.codex-plugin/`, `.claude-plugin/` — cleared
  by re-rendering, never hand-edited.

## Correct references (spot-checked, no action)

Roughly 25 live cross-references were checked and match current content, including
`packages/bee/lib/reservations.mjs:62` (rule 4), `packages/bee/lib/cells.mjs:1500` (hive law 11),
`docs/specs/reading-map.md:156`, `docs/knowledge/areas/doctrine-layer/delegation-threshold.md:44,68`,
`docs/knowledge/areas/doctrine-layer/the-communication-contract.md:13,180`,
`docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md:206,215`,
`scripts/tests/test_gate_bypass_doctrine.mjs:413,418,420`, `packages/bee/tests/test_misc.mjs` (5 hits,
rule 12), `packages/bee/tests/test_guards.mjs:54`, `packages/bee/tests/test_cli_state.mjs:3294,3442`,
`skills/bee-reviewing/SKILL.md:109`.

Drift is scattered, not systematic — correct and incorrect citations of the *same* concept sit side
by side. That is the fingerprint of hurried renumbering, not of a wrong convention.

## Completeness

This inventory is not proven exhaustive. Rows 1-13 are confirmed; row 4 was found only while
verifying its neighbours, which means the discovery pass that produced rows 1-3 was itself
incomplete. The implementing cell re-runs discovery across `packages/`, `scripts/`, `skills/`,
`docs/knowledge/`, and `docs/decisions/` and reports the final count, rather than trusting 13.
