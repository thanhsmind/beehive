# diet-9 — bee-validating thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-validating/SKILL.md` rewritten to the D7 thin-body doctrine:
18,273 → **8,049 bytes** (≤ 8,192). Load-bearing invariants preserved intact in the
body: reality-gate five dimensions, accepted-evidence law (plausibility = NOT READY),
decision vocabulary block, advisor staleness anchors (the four conditions, compressed
from a numbered list to an inline OR-clause — same four facts), delta/cache rule, and
the skill_lint hard check (`/merged reviewer prompt/i` still matches the body;
`references/validation-reference.md` keeps its "Merged Reviewer Subagent Prompt"
section with BLOCKER/WARNING/CRITICAL/MINOR). Everything else exiled meaning-intact
into `skills/bee-validating/references/validation-reference.md` (new sections:
"Required Inputs in full", "Review Wave in full" incl. the `bee:only claude`/`codex`
branches, "Advisor Consult in full"; extended "Spike / Probe Rules" with debug
discipline + the docs/history/ code-extension ban; extended "Approval Gate Block"
with the cross-model second opinion, bee-briefing refresh timing, and the
validated-is-not-a-phase note). New `references/provenance.md` maps every body rule
to its decision IDs (D1, decision 0009, D2, Delegation D2/D3, decision 0017, decision
0021, AO11, AO12/B1, AO2b/AO3/AO4, AO13, decisions 0010/dcf01d7b, chain-integrity D6,
spec #77 P6) — provenance grep now active on bee-validating, 0 hits. Baseline lowered
to 8049, `bee-validating` appended to `migrated[]`, `notes["bee-validating"]` deleted.
Regen obligation ran in-cell: plugin mirror render, onboard `--apply`, release
manifest `--write`/`--check`. Full verify chain green.

**Files:** `skills/bee-validating/SKILL.md`,
`skills/bee-validating/references/validation-reference.md`,
`skills/bee-validating/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-9.json`.

## Side-by-side behavior checks (P5 acceptance)

### s1 — the skill_lint hard check routes identically

Before (body, `## Reference Files`):

> | `references/validation-reference.md` | Report formats, repair routing, the merged reviewer prompt, approval block |

After (body, `## Reference Files`):

> | `references/validation-reference.md` | Report formats, repair routing, required inputs, review wave + advisor consult mechanics, the merged reviewer prompt, approval block |

`references/validation-reference.md`'s `## Merged Reviewer Subagent Prompt` section
and its BLOCKER/WARNING/CRITICAL/MINOR vocabulary are untouched. `skill_lint.mjs`'s
dedicated check ("bee-validating routes to the MERGED reviewer prompt...") passes
unchanged — same regex match, same section, same four finding-class words.

### s2 — advisor staleness anchors (rare branch) survive the format compression

Before (body, numbered list):

> An `advisor_ref` is stale if **any** of (AO13, verbatim):
> 1. its feature differs from `state.feature`;
> 2. the newest active decision id changed since the consult;
> 3. `sha256(plan.md)` changed since the consult;
> 4. the ref predates the most recent revocation of the execution gate.
>
> Never a time-based TTL — AO13 already burned this feature on one invented number once.

After (body, inline law-paragraph per the cell's "law-paragraph -> table/≤2-line
bullet" compression rule):

> Stale = its feature differs from `state.feature`, OR the newest active decision id
> changed since the consult, OR `sha256(plan.md)` changed since the consult, OR the
> ref predates the most recent execution-gate revocation — never a time-based TTL.

All four conditions are still present, in the same order, same meaning; the AO13
citation moved to `references/provenance.md` (provenance exile, D8). A `state gate
--name execution --approved true` call for a stale ref still refuses identically —
no code changed, only the prose describing the same four-condition check the CLI
already enforces.

### s3 — Review Wave dispatch mechanics reachable via the new pointer

Before (~1,800-byte body section, `## Review Wave`): the full "wave, not a chain"
paragraph, "one dispatch, two mandates" paragraph (decision 0021, model resolution),
the `bee:only claude`/`codex` blocks, the cli-shaped gather note (AO12/B1), "one shot
then at most one blocker pass", and "small-diff standard: same mandates, no dispatch"
— all inline in the body.

After (body, Operating Contract row 6, ~60 words):

> one merged reviewer (structure + cold-pickup), dispatched at stage start beside the
> matrix — cost is max(reviewer, matrix), never the sum; sync point holds findings
> until Gate 3; one shot, then at most one blocker-scoped pass. Full mechanics +
> runtime dispatch differences: `references/validation-reference.md` ("Review Wave in
> full", "Merged Reviewer Subagent Prompt").

The pointer resolves to the unchanged `## Review Wave in full` section in
`references/validation-reference.md`, which carries every paragraph from the old body
verbatim (decision citations exiled to `provenance.md`), including both `bee:only`
runtime blocks and the small-diff-standard inline-self-review carve-out.

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)`
  matches in the body (verified with the fence's exact regex before capping); the
  rule → decision-ID map is `references/provenance.md`.
- `bee:only claude`/`codex` blocks moved from the body into
  `references/validation-reference.md`'s new "Review Wave in full" section rather
  than staying in-body (unlike `bee-swarming`'s precedent) — both markers are inside
  an already-existing shared reference file, not a new per-runtime file, which stays
  inside D3's boundary ("no `references/runtime-<name>.md` files are created").
  `test_skill_render.mjs`'s 27/27 pass covers marker grammar and render projections
  across the whole skill tree, `bee-validating` included.
- Deviation: none — no bugs found in touched code, no missing critical functionality,
  no architectural change needed.
