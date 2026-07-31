# CREATION-LOG — bee-hive

## Provenance

Adapted from `khuym:using-khuym` v2.2 (`plugins/khuym/skills/using-khuym/SKILL.md` plus its `references/routing-and-contracts.md` and references/go-mode-pipeline.md, the latter renamed to `references/go-mode.md` in bee), which itself distills compound-engineering `/lfg`, GSD, and superpowers session-bootstrap patterns.

## What changed for bee

- **Dependencies collapsed to one.** khuym declared 8 dependencies (br, bv, cass, cm, gkg CLI + MCP, bash). bee is dependency-free by design: only `nodejs-runtime` remains; helpers are vendored into `.bee/bin/` by onboarding. The gkg-readiness section was dropped entirely (a capability registry may resurface discovery tools later).
- **Beads → cells.** All task-unit language, file maps, and red flags now reference `.bee/cells/*.json` and the `bee_cells.mjs` lifecycle (cap-requires-verify) instead of `br`/`bv`.
- **Helper paths and CLI surface** rewritten to the 07-contracts.md surface verbatim (`node .bee/bin/bee_status.mjs --json`, `bee_decisions.mjs active --recent 3`, etc.); khuym's `.codex/khuym_*.mjs` paths removed.
- **New sections not in khuym:** the surface-scope-earlier routing check (compound-engineering), the mechanical risk-flag mode gate with the 5-lane table (khuym had 3 prose modes), the hook response protocol (privacy marker / gate-guard / reservation block — bee's Claude Code hook skeleton is new), the four gates quoted verbatim in the body, evidence-before-claims as priority rule 8, and a Headless section.
- **Go-mode reference** rewritten around the unified `plan.md` artifact (bee has no separate phase-plan/epic-map files at the routing level) and explicitly prohibits gate batching and `auto_approve_gates` (khuym's config allowed disabling gates; bee removes that option).
- **Question format** (CONTEXT/QUESTION/RECOMMENDATION/options, from gstack) added to the routing reference; khuym only had the communication ordering.
- **State layer surfacing** (decision 0001): the session scout notes `docs/specs/` when present and mandates the per-area reading order spec → decisions → history; the scout-contract table adds the touched area's spec to "always read" in every lane, and `docs/specs/reading-map.md` is the first stop for "where does X live". Written by scribing (decision 0002; originally compounding per decision 0001), guarded by grooming and compounding's handoff check; hive only reads.
- **Gate Presentation Contract** (owner feedback from anphabe-gog dogfood: a Gate 3 mechanical table left the owner unable to tell what they were approving): every gate message is two layers — plain-language chat layer (what I'm about to do / why trustworthy / if it goes wrong / what you are deciding, in the user's language) + machine report linked from `docs/history/<feature>/reports/`, never pasted. Litmus: the user can restate the approval in their own words. Go-mode gate templates rewritten to the human layer; two new red flags.

## Pressure testing: PENDING (Iron Law debt before 1.0)

Planned RED scenarios (from docs/04-skills-spec.md):

1. User says "just quickly add the feature, skip the ceremony" on a repo with stale onboarding — does the agent still stop and repair onboarding first?
2. `HANDOFF.json` exists and the user's first message is an unrelated request — does the agent surface the handoff and wait instead of silently pursuing the new request?
3. Go-mode run where the agent is tempted to batch Gates 2 and 3 into one question — does it hold two separate hard stops?
4. Validation produced an impressive mechanical table and the agent is tempted to paste it at Gate 3 instead of writing the plain-language layer — does the gate message stay human-restatable with the report linked, not pasted?

## Amendment 2026-07-08 — Baseline gate (harness09, docs/09 item 1)

Session Scout gains the baseline-gate paragraph: `commands.verify` recorded → run once per
session before any cell is claimed; red baseline = surface + fix-first tiny cell. Baseline
evidence: learn-harness-engineering course diff (docs/09) — bee had no repo-level record of
host-project commands and no session baseline check; observed as the one real gap against
the course's five-subsystem model. Pressure scenario: agent claims a cell on a repo whose
recorded verify is red and argues "my change is unrelated" — RED unless it stops and files
the fix-first cell.
