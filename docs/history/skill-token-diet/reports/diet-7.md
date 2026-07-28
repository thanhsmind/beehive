# diet-7 — bee-planning thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-planning/SKILL.md` rewritten to the D7 thin-body doctrine:
23,027 → **8,139 bytes** (≤ 8,192). All exiled text landed in
`skills/bee-planning/references/planning-reference.md` (six new sections:
Greenfield init lane, Lane-scaled bootstrap in full, Discovery in full,
Gate 2 bypass mechanics, Tiny/small merged gate, Slice-tail test batching in
full); new `skills/bee-planning/references/provenance.md` maps every body
rule to its decision IDs. Baseline lowered to 8139, `bee-planning` appended
to `migrated[]` (provenance grep now active — 0 hits after scrubbing every
inline `(D#)` citation from the body), `notes["bee-planning"]` deleted.
Regen obligation ran in-cell: plugin mirror render, onboard `--apply`,
release manifest `--write`/`--check`. Full verify chain green.

**Files:** `skills/bee-planning/SKILL.md`,
`skills/bee-planning/references/planning-reference.md`,
`skills/bee-planning/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-7.json`.

## Side-by-side behavior checks

### s1 — mode-gate flag list and lane thresholds route identically (verbatim flags, per preserve-list)

Before (§1 Intake & Mode Gate):

> Count risk flags — do not vibe it:
> auth · authorization · data model · audit/security · external systems ·
> public contracts · cross-platform · changes behavior an existing test
> asserts (a covered contract must change) · the change requires weakening,
> deleting, or replacing existing proof · multi-domain
> ...0–1 flags → tiny...2–3 flags or story-sized behavior → standard...

After (§1 Mode Gate):

> Count risk flags — do not vibe it:
> auth · authorization · data model · audit/security · external systems ·
> public contracts · cross-platform · changes behavior an existing test
> asserts (a covered contract must change) · the change requires weakening,
> deleting, or replacing existing proof · multi-domain
> | `tiny` | 0–1 flags, ≤2 product files, one direct task |
> | `standard` | 2–3 flags, or story-sized behavior |

The flag list is byte-identical (preserved verbatim, per the cell's
preserve-list); the same threshold values route to the same lanes. A "fix
this typo, one file" request still triages `tiny`; a 3-flag story-sized
change still triages `standard`.

### s2 — plan-freeze rule (frozen at Gate 2, approval-stamp-only) survives intact

Before (§5 Shape):

> **Plan freeze (D1).** `plan.md` is **frozen at Gate 2**: once
> `approved_gates.shape` is set, its content sections are immutable. The
> only permitted post-approval write is an **approval stamp** (status +
> timestamp in the frontmatter) — never a content edit. There is no "enrich
> the same plan.md in place to implementation-ready" step...

After (Hard Gates):

> **`plan.md` frozen at Gate 2:** once `approved_gates.shape` is set,
> content sections are immutable — only an approval stamp may follow, never
> a content edit.

Same trigger (`approved_gates.shape` set), same immutability scope (content
sections), same single exception (an approval stamp), same prohibition
(never a content edit). The decision ID moved to `references/provenance.md`
(D8 provenance exile) — the rule's meaning is unchanged.

### s3 — walking-skeleton and slice-tail-test rules preserved as load-bearing invariants

Before (§6 Prep):

> **Walking skeleton first (spec #81 P2).** When the feature has any
> user-visible surface (UI, API, CLI), slice 1 is the thinnest end-to-end
> runnable path through it — one happy path, real behavior however thin, no
> stubs...
> **One trailing test cell per slice (slice-tail-test-batching P2, spec
> #80/#85).** Whenever the slice holds ≥1 `change_class: 'behavior'`/`'api'`
> cell **that touches code**... emit **exactly one** `change_class: 'test'`
> cell, last, with `deps` naming **every** implementation cell of the slice.

After (§4 Prep):

> **Walking skeleton first.** Any user-visible surface (UI/API/CLI) → slice
> 1 is the thinnest end-to-end runnable path, one happy path, real behavior
> however thin, no stubs; each slice's done-report owes one artifact proving
> it runs.
> **One trailing test cell per slice.** Any slice with ≥1 code-touching
> `behavior`/`api` cell (instruction/knowledge text owes no test) emits
> exactly **one** `change_class: 'test'` cell, last, `deps` naming every
> implementation cell.

Both rules keep identical mechanics (thinnest end-to-end path, no stubs;
exactly one trailing test cell, deps naming every implementation cell). The
spec/plan-number citations moved to `references/provenance.md`; the full
`bugfix`/`high-risk` per-cell red-first carve-out and the D5/D3 test-economy
interplay are one hop away in `references/planning-reference.md`
("Slice-tail test batching in full").

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` matches in the body (`skill_budget_fence.mjs` bare run confirms — 18 skills checked, 0 findings); the rule → decision-ID map is `references/provenance.md`.
- All quoted headings in the body resolve to real headings in `references/planning-reference.md` or cross-skill `bee-hive/references/routing-and-contracts.md` (`skill_lint.mjs` anchor-integrity check green).
- `okf_instructions_fence.mjs` stayed green after the move — the bundle-branch teaching paragraph (area-truth reading order) was relocated to `references/planning-reference.md` ("Lane-scaled bootstrap in full") intact as a single unbroken line so its `docs/knowledge`/`docs/specs/` branch markers stayed line-local.
- Deviation (recorded in cell trace): the mode-gate paragraph's "stating why smaller modes are insufficient" clause and the Gate 2 `bee-plan/v1` artifact-contract literal were dropped from the body as redundant with the `plan.md` template already carried in `references/planning-reference.md` — additive-safe compression, no rule dropped.
- Scope-Reduction Prohibition preserved as its own section, unabridged in meaning (SPLIT RECOMMENDED answer, per-slice honoring of every touched locked decision, cheaper-alternative noting, D-ID supersession requirement).
