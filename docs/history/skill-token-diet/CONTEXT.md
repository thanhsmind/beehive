# Skill Token Diet — Context

**Feature slug:** skill-token-diet
**Date:** 2026-07-28
**Exploring session:** complete (gate bypass TOTAL — approval-class gray areas locked from evidence-backed recommendations, audit decisions logged)
**Scope:** Standard
**Domain types:** ORGANIZE (skill text layout, fence config), RUN (verify-chain fence script)

## Feature Boundary

Enforce a thin-body doctrine over `skills/*/SKILL.md` with a machine-checked byte
budget, migrate the six heaviest bodies to it (content moves to `references/`,
meaning never changes), and add the compounding law that stops regrowth. Source
spec: `/home/thanhsmind/projects/AI/ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md`
(P1–P6; P7 rejected). Ends at: fence green in the verify chain, six bodies ≤ budget,
regrowth law stated and enforced. No gate, proof rule, or law changes *meaning* —
only where its text lives and how tersely it is phrased.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Byte budget: `wc -c` ≤ **8,192 bytes** per migrated `skills/*/SKILL.md` (target 5–6K); no upfront carve-out for bee-hive. *Mechanism superseded by D6:* the frontmatter `body_budget_bytes` idea is dropped — budgets and exceptions live in `scripts/skill-body-budget.json` (see D6). | An over-8K skill declares its exception in the baseline file with a justification note — pressure stays visible. |
| D2 | Fence covers ALL skills from day 1. Unmigrated skills are grandfathered at their **exact current `wc -c` value**, re-seeded into `scripts/skill-body-budget.json` (ratchet semantics: may only shrink). This feature migrates the measured worst 6: bee-hive 30.1K, bee-swarming 24.7K, bee-scribing 24.5K, bee-planning 23.0K, bee-reviewing 22.3K, bee-validating 18.3K — each migration lowers its baseline entry to ≤8,192. Remaining skills = wave-2 PBI `p-3d6877c2`. | Measurement over spec claim: spec's "bee-herding 56K body" is false — body is 14.3K (56K counted `references/`, 92KB dir); herding joins wave 2. Spec's bee-executing 19.5K is stale — now 10.2K. Note: several current bodies already exceed their recorded baseline (e.g. bee-hive 30078 > 29808) — pre-existing drift the migration itself resolves. |
| D3 | P4 per-runtime split = **render-time specialization** via the existing `<!-- bee:only <runtime> -->` markers (`packages/bee/scripts/onboard_bee.mjs`, `MARKER_ONLY_RE` ~line 747). No `references/runtime-<name>.md` files are created. The fence measures **source bytes** (`skills/*/SKILL.md` in the repo), markers included — render-time stripping does not relieve the budget. | The install pipeline already strips per-runtime blocks at render; spec P4 prefers render-time where supported. Measuring source keeps the fence one command with no render dependency. |
| D4 | P7 stage-instruction offload **rejected this pass**; filed as PBI `p-c6e61dfb`. P1–P6 stand alone. | Dispatch-topology change deserves its own spec and feature; spec flags P7 optional. |
| D5 | P6 compounding law in scope: a new learning lands in the knowledge bundle or the owning skill's `references/` by default; a body edit is allowed only for a load-bearing invariant (P1 category 3) and must fit the budget — over budget means trim elsewhere in the same body first ("one in, one out"). Stated in `bee-writing-skills` and `bee-evolving`; enforced mechanically by the D6 fence. | Without the regrowth law the diet is temporary — bodies grew monotonically because every lesson became permanent per-invoke tax. |
| D6 | Reuse the existing machinery: `scripts/skill-body-budget.json` stays the **single source of budgets** (per-skill bytes, ratchet: only shrinks, `--update-baseline` seeds/lowers). The **body-budget check plus the D8 provenance grep are promoted to a blocking fence** registered in `scripts/run_verify.mjs` (okf-fence pattern); `skill_lint.mjs`'s anchor-integrity and ordered-list checks stay advisory, per the 2026-07-27 lint-not-suite law. The baseline JSON schema gains an optional per-skill `note` for exception justifications (D1). **Supersession, recorded:** this narrows the 2026-07-27 user law ("instruction text gets a lint, not a suite; nothing blocks") for the fence's blocking checks only (budget + D8 provenance grep) — the user approved spec P2's chain-fail enforcement on 2026-07-28, which is the newer instruction. | One baseline file, one ratchet, no parallel frontmatter mechanism; the only law change is the minimum the approved spec demands. |
| D7 | P1 thin-body doctrine locked as spec'd: a body contains only (1) trigger + lane scaling, (2) one flow diagram or step table, (3) load-bearing invariants (Red Flags qualify), (4) reference routing table `| file | when to load |`. Everything else moves to `references/`. | The four categories are the editorial rule every migration commit is checked against. |
| D8 | P3 provenance exile locked: migrated bodies carry no decision IDs/plan names/hardening labels inline; **every migrated skill gets (or extends) `references/provenance.md`** mapping rule → decision IDs + rationale; the provenance grep (`\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` — tunable) **ships inside the D6 fence** and applies to migrated skills. | Auditability one hop away instead of per-invoke tax; grep keeps it mechanical. |

### Agent's Discretion

- Fence packaging: extend `skill_lint.mjs` with a blocking mode vs a small separate fence script reading the same baseline — either, provided D6's split (budget blocks, rest advisory) holds.
- Per-skill editorial choices during migration (what is a category-3 invariant vs reference material), bounded by D7's four allowed body categories and the acceptance criteria.
- Provenance-citation grep pattern (D8) is tunable; the point is mechanical, not perfect.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| body | The `SKILL.md` file of a skill — injected whole on every invoke, never partial, never unloaded. |
| prepaid | Instruction bytes paid on every invoke (body), vs references paid only when a branch is hit. |
| thin-body doctrine | A body contains only: (1) trigger + lane scaling, (2) one flow diagram/table, (3) load-bearing invariants, (4) reference routing table. Everything else lives in `references/`. |
| provenance exile | Body states rules bare (no D-IDs/decision hashes/plan names inline); each migrated skill's `references/provenance.md` maps rule → decision IDs + rationale. |
| grandfather exception | An unmigrated skill's `skill-body-budget.json` entry at its exact current byte size (ratchet) — blocks growth now; the skill's migration commit lowers it to ≤8,192. |

## Specific Ideas And References

- Rollout order (spec's shape, restated over D2's six): fence first (locks the ceiling), then one skill per commit worst-first — bee-hive → bee-swarming → bee-scribing → bee-planning → bee-reviewing → bee-validating — P4 folded in where `bee:only` blocks exist, P6 (D5 law text) after the first two migrations prove the shape.
- Worked example (spec P5): bee-hive crash-recovery ~1,100-char line → ~150-char body pointer + `references/recovery.md`.

## Existing Code Context

### Reusable Assets

- `scripts/skill_lint.mjs` + `scripts/skill-body-budget.json` — existing advisory body-budget ratchet (per-skill bytes, `--update-baseline` only lowers); D6 promotes its budget check to blocking and keeps the rest advisory.
- `scripts/okf_specs_fence.mjs` — existing chain-fail fence pattern (registered in `run_verify.mjs` ~:332); the blocking budget check follows it.
- `packages/bee/scripts/onboard_bee.mjs` (~line 670–760) — `bee:only`/`bee:end` marker parsing + per-runtime render strip; D3 relies on it as-is.
- `skills/*/references/` — 15 of 18 skills already have the directory; routing tables already exist in several bodies.

### Established Patterns

- Fence-as-chain-check — `scripts/run_verify.mjs` runs fences; red fails the chain.
- Reference routing tables (`| file | when to load |`) already in bee-hive and others — P1 formalizes what they should have been guarding.

### Integration Points

- `scripts/run_verify.mjs` — the new fence registers here.
- `skills/bee-writing-skills/SKILL.md`, `skills/bee-evolving/SKILL.md` — D5 law text lands here.
- `packages/bee/scripts/onboard_bee.mjs` render path — must keep rendering migrated skills correctly (references ship alongside bodies).

## Canonical References

- `/home/thanhsmind/projects/AI/ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md` — source spec (P1–P7, acceptance criteria per part).
- `docs/knowledge/index.md` — critical patterns digest (mandatory pre-planning context).

## Outstanding Questions

### Deferred To Planning

- [ ] Does the plugin render/install pipeline copy `references/` for every target, and does `MARKER_ONLY_RE` stripping also apply inside reference files? — read `onboard_bee.mjs` render path; determines whether moved `bee:only` blocks keep working after migration.
- [ ] Fence packaging per D6 discretion (blocking mode inside `skill_lint.mjs` vs separate script) and how baselines already exceeded by current bodies (bee-hive +270B drift) are handled pre-migration. — fence implementation detail.
- [ ] Which recorded scenarios serve as the 2–3 side-by-side behavior checks per migrated skill (P5 acceptance)? — pick during validation from existing tests / skill walkthroughs.

## Deferred Ideas

- Wave-2 migration of all remaining skills, removing grandfather exceptions — PBI `p-3d6877c2`.
- P7 stage-instruction offload (worker-loaded stage skills, orchestrator keeps hive+swarming+gates) — PBI `p-c6e61dfb`.
- Companion specs already in backlog: validation-speedup, test-batching, ship-visibility (filed 2026-07-28).

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Validating and reviewing use locked decisions for coverage and UAT.
