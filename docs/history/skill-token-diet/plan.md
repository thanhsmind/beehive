---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Skill Token Diet — Plan

Source of truth: `docs/history/skill-token-diet/CONTEXT.md` (D1–D8). Spec:
`/home/thanhsmind/projects/AI/ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md`.

## Mode-Gate Record

Flags: 1 (multi-domain: instruction text + verify-chain code). Story-sized
behavior (fence + six skill migrations + regrowth law) → **standard**. Product
files ≈ 12 (6 SKILL.md bodies + new reference files + `scripts/skill_budget_fence.mjs`
+ `scripts/run_verify.mjs` + `scripts/skill-body-budget.json` + `scripts/skill_lint.mjs`
+ 2 law-carrying skills) — over the small cap, and slices are genuinely sequential.
Smaller modes insufficient: multi-slice, chain-touching, byte-budget law change.

## Discovery

L1 (precedent-rich, no candidate comparison needed):
- Fence pattern: `scripts/okf_specs_fence.mjs` / `okf_instructions_fence.mjs`,
  registered in `scripts/run_verify.mjs` (~:332, ~:354) as a `--selftest` + bare
  invocation pair — selftest proves the fence BITES.
- Budget machinery: `scripts/skill_lint.mjs` (advisory, always exit 0) +
  `scripts/skill-body-budget.json` (per-skill byte ratchet, `--update-baseline`
  seeds/lowers only). Per D6 the budget check is promoted to blocking; anchors +
  ordered-list checks stay advisory in the lint.
- Render pipeline: `packages/bee/scripts/onboard_bee.mjs` `MARKER_ONLY_RE` (~:747)
  strips `bee:only` blocks at install (D3 — no runtime reference files; fence
  measures source bytes).

## Approach

**Fence first, then worst-first migrations, law after the shape is proven.**

1. New `scripts/skill_budget_fence.mjs` owns the budget machinery end-to-end:
   - Reads `scripts/skill-body-budget.json`; any `skills/*/SKILL.md` whose
     `wc -c` exceeds its recorded budget → chain FAIL (message: "pay for new
     text by removing text").
   - A skill with no recorded budget → FAIL (new skills seed explicitly).
   - `--update-baseline` moves here from `skill_lint.mjs` (seed new / lower
     after a trim; refuses to raise). Baseline re-seeded once at S1 to exact
     current sizes (resolves the pre-existing drift, e.g. bee-hive 30078 > 29808).
   - Provenance grep (D8): applies to every skill whose recorded budget ≤ 8,192
     (mechanical definition of "migrated") — body matches of the tunable pattern
     `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` → FAIL. Grandfathered
     skills (budget > 8,192) are exempt until their migration lowers the entry.
   - `--selftest` proves both bites on fixtures (over-budget body; provenance
     citation in a ≤8K skill; grandfathered skill passes).
   - Registered in `run_verify.mjs` as the selftest + bare pair, beside the okf
     fences. `skill_lint.mjs` drops its budget check (fence owns it), keeps
     anchor-integrity + ordered-list checks advisory (2026-07-27 law, narrowed
     per D6 only for the fence's checks).
   - Exception mechanism (D1): baseline entry may exceed 8,192 only with a
     sibling `notes.<skill>` string in the JSON carrying the justification
     ("pending migration" for grandfathers); fence FAILs an >8,192 budget with
     no note.
2. Migrations (one skill per commit, worst-first per D2): body rewritten to
   D7's four categories; exiled text lands in that skill's `references/`
   (provenance → `references/provenance.md` per D8); each commit lowers the
   skill's baseline entry to ≤8,192 and removes its grandfather note.
   **Hard constraints during text moves:**
   - `okf_instructions_fence.mjs` is line-local over instruction surfaces —
     bundle-branch teaching lines must survive the move (keep the line, or move
     it intact where the fence still grades it green). Chain must stay green
     after every migration commit.
   - `skill_lint.mjs` anchor integrity: every `references/x.md ("Heading")`
     pointer must resolve — new reference files carry the exact headings bodies
     point to.
   - `bee:only` marker grammar stays valid (`MARKER_ONLY_RE`); markers stay in
     source (D3).
   - Critical pattern (20260722): a migration is not done until generated
     orientation matches — re-run any skill-render projections after each
     migration (`scripts/tests/test_skill_render.mjs` scope).
   - Meaning never changes: rules move or compress, never weaken (spec non-goal;
     CONTEXT boundary).
3. P6 law (D5) after S2+S3 prove the shape: law text in `bee-writing-skills`
   (pressure-test check) + `bee-evolving` (learning-placement rule); fence
   already enforces mechanically.

**Rejected:** frontmatter `body_budget_bytes` (superseded by D6 — one baseline
file); blocking mode folded into `skill_lint.mjs` (violates the advisory-lint
law's shape; separate fence follows the okf precedent and keeps the law's
narrowing surgical); measuring rendered bytes (D3 — source is the law).

**Risk map:**
| Component | Risk | Proof needed |
|---|---|---|
| Fence script + registration | LOW | selftest fixtures; chain green |
| Baseline re-seed | LOW | one command, diff reviewed |
| bee-hive migration (30.1K → ≤8K, router) | HIGH | chain green + side-by-side behavior checks (2–3 recorded scenarios: routing table intact, gate contract reachable, crash-recovery pointer resolves) |
| Other 5 migrations | MEDIUM | same checks, per skill |
| P6 law text | LOW | fence blocks regrowth mechanically |
| Instructions-fence interplay | MEDIUM | fence stays green after each move (line-local grading) |

## Slices

- **S1 — fence:** `skill_budget_fence.mjs` + baseline re-seed + `run_verify.mjs`
  registration + `skill_lint.mjs` trim. 1 impl cell + 1 trailing test cell
  (selftest fixtures proving both bites + grandfather pass).
- **S2 — bee-hive migration** (the worst body, proves the editorial shape).
- **S3 — bee-swarming + bee-scribing migrations + P6 law text** (law lands after
  two migrations prove the shape, per spec rollout).
- **S4 — bee-planning + bee-reviewing + bee-validating migrations + closing
  measurement** (report: per-skill before/after bytes, pipeline prepaid total).

Current-slice cells: **S1 only** (D2 of planning).

## Test Matrix (edge dimensions, standard depth)

- **Boundary:** body exactly 8,192 bytes (pass); 8,193 (fail). Budget entry
  exactly 8,192 → provenance grep applies.
- **Absence:** skill with no baseline entry → fail loud; baseline file missing →
  fail loud (not silent skip).
- **State drift:** body grown over recorded budget (the live bee-hive case) →
  fail with byte delta.
- **Format:** provenance pattern in code fences — grep is line-based over body;
  acceptable false-positive surface documented as tunable (D8).
- **Idempotence:** `--update-baseline` twice → second run "nothing to lower";
  never raises.
- **Compat:** never-migrated skill (grandfathered, note present) → green;
  exception without note → fail.

## Open Questions (for validating)

- Does `test_skill_render.mjs` cover all 6 in-scope skills' render projections,
  or does migration need an added render check?
- Exact set of recorded scenarios for bee-hive's side-by-side behavior check.
