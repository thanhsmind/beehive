---
artifact_contract: bee-plan/v1
mode: standard
feature: packages-engine-move
approved_gate2: 2026-07-26 (auto — gate_bypass total)
---

# packages-engine-move — plan

## Mode gate (mechanical record)

Flags: **3** — public contracts (installer entrypoints `install.sh`/`install.ps1`, SKILL.md onboarding instruction), changes behavior existing tests assert (`test_onboard_bee` entrypoint location, identity anchors), multi-domain (engine + installers + verify discovery + plugin render + CLI strict-flags). → **standard**. Completes the packages-restructure vision: `skills/` truly instruction-only; plus the filed `capture add` silent no-op friction fixed.

## Decisions

- **D1** — The onboarding/distribution engine (`onboard_bee.mjs`, `plugin_distribution.mjs`, their 3 test suites) moves to `packages/bee/scripts/`. `packages/bee/` is then the complete standard code set: payload + engine.
- **D2** — Resolution contract preserved by construction: `packages/bee/scripts/onboard_bee.mjs` → 3×dirname = root (same arithmetic as today from `skills/bee-hive/scripts/`); engine's lib imports simplify to `../lib/*.mjs`. Both real source kinds (source_checkout, plugin_package) carry `packages/` — proven by packages-restructure (manifest roles + sparse-checkout already include `packages`).
- **D3** — Host projections (`.claude/skills/`, `.agents/skills/` synced copies) stop carrying the engine — they are compliance mirrors that deliberately refuse to be sources (`blocked_no_source`, source-identity D9). The identity anchor (`identityOk`: realpath check against `skills/bee-hive`) and `classifySource` re-derive from the engine's new location; the `project_projection` "no payload → blocked_no_source" contract is preserved (a projection still classifies as projection and still refuses).
- **D4** — `capture add --text` silent no-op class bug fixed centrally: the `bee.mjs` dispatcher validates flags against the command registry's declared `parameters.properties` for EVERY verb and throws on unknown flags (the strict-flag pattern `update`/`worker prune` already use, made universal). Registry schemas already exist — validation is a lookup, not new metadata.
- **D5** — Applied pattern `20260726-migration-tooling-is-a-consumer-of-the-migration`: `scripts/render_plugin_skill_trees.mjs` imports `renderSkillBytes` from the engine being moved — it is in cell 1's files from the start; manifest coverage of the engine moves in the same cell (engine files leave the `plugin_skill` walk and enter the `package_payload` walk automatically — cell 1 proves record counts).

## Discovery (L1 — gather digest 2026-07-26)

Live surface for the engine move (excluding rendered mirrors + history):
- Engine self: `onboard_bee.mjs:82-91` consts (SCRIPT/SCRIPTS/HIVE/PLUGIN_ROOT), `:65-67` imports `../../../packages/bee/lib/*`, `:1515,1537-1551` identityOk realpath anchor, `:490-534` readSourceReleaseIdentity / blocked_no_source branches; `packages/bee/lib/source-identity.mjs:50-108` classifySource path arithmetic.
- Callers: `scripts/install.sh:162-179` (BEE_SRC probe + ONBOARD + DIST_HELPER literals), `scripts/install.ps1:293,321,326-327`, `scripts/run_verify.mjs:94` DISCOVERY_ROOTS, `scripts/release_manifest.mjs:45` comment, `scripts/render_plugin_skill_trees.mjs` renderSkillBytes import, `packages/bee/bee.mjs:5618,5640,5788,5915` fix-hint strings, `packages/bee/lib/cells.mjs:192,201` regen instruction strings, `skills/bee-hive/SKILL.md:37-45` onboarding step, CI workflows (check), docs (`docs/specs/onboarding.md`, `reading-map.md`, `07-contracts.md`, `02-architecture.md`, README).
- Skill sync: `applySyncSkill` copies whole skill tree; after the move the synced bee-hive shrinks by 5 files — sync's fingerprint diff must remove them from projections (removal path check).
- Strict flags: `bee.mjs` handlers destructure flags ad hoc; strict pattern exists at `bee.mjs:1236-1239` (update) and `:3084` (worker prune); registry schemas in `packages/bee/lib/command-registry.mjs`.

## Approach

Two move-and-fix cells + one CLI-hardening cell, serial. Acceptance for the move is repo-wide rg-clean of the literal old path (pattern rule), never a file checklist. Regen obligation in every cell touching hashed roots.

Risks: identity anchor + classifySource are the behavior-bearing edits (MED — covered by test_onboard_bee/test_split_brain suites); universal strict flags could break a legitimate caller passing undeclared flags (MED — full suite + hook contract tests are the net; any hit is a schema gap to fix in the registry, not a reason to loosen).

## Slice (current)

- Cell 1: move engine to `packages/bee/scripts/` + full reference/resolution/regen sweep. Verify: full `run_verify` + manifest `--check`.
- Cell 2: SKILL.md + installers docs sweep + projection-shrink proof (self-onboard removes engine files from synced trees; onboard ends `up_to_date`). Verify: rg-clean + manifest `--check`.
- Cell 3: dispatcher-level unknown-flag rejection for all verbs + regression test (incl. the exact `capture add --text` repro red-first). Verify: targeted CLI suites + full impacted.
