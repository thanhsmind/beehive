# Proof Honesty — Context

**Feature slug:** proof-honesty
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

bee's proof discipline gains two honesty guards distilled from openJev-verdict-2.0: a
confirm-checklist gate on the one release flag that skips the suite, and an optional
baseline field on the cap proof line. It ends at those two surfaces — no new verb, no
new artifact, no change to what a gate asks.

The distill named four guards. The other two (the anti-cherry-pick rule and the
smoke-first rule) were delivered by the sibling feature `proof-completeness-rules`
while this lane was still shaping; this feature does not re-file them. Scope was
split, never shrunk — all four are being delivered, two of them elsewhere.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | ~~Anti-cherry-pick rule in AGENTS.md~~ — **delivered by the sibling feature `proof-completeness-rules`**, not by this one. See Prior Art. Kept as a D-ID so nothing downstream renumbers. | Another session shaped the same distill from spec drop `3d41a292` and landed it in `packages/bee/AGENTS.block.md` before this lane reached its gate. Re-filing it here would be a near-duplicate. |
| D2 | ~~Smoke-first rule in AGENTS.md~~ — **delivered by `proof-completeness-rules`**. Same note as D1. | Same. |
| D3 | `scripts/release.sh --no-test` prints a four-item release checklist and exits 1 unless `--confirm` is also passed. With no TTY (headless, or the unattended herding loop) it proceeds and prints every checklist line into the log instead of refusing. Decision `b4b806c4`. | A retyped command is a deliberate act; a scrolling warning line is not. An absolute refusal would break herding's release path until its call site is taught both flags, buying no guard the logged skip does not already give. |
| D4 | The baseline rides the cap REPORT, not the proof line. `bee cells finish --report` gains `"baseline"` as a second OPTIONAL key beside `mistakes`: a string naming what the same proof command produced on the base commit, validated as a string when present and stored on the cell trace. `REPORT_KEYS` keeps its five required keys; the proof line keeps its three segments byte for byte. Decision `f4261145`, superseding `f873d3f5`. | The first draft put the floor inside the proof line as a fourth segment. Measured, that costs 26 source files, 11 crate tests and four doc homes, and walks through `proof-strength-and-expiry` D6. The report is already the structured half of a cap; a baseline is structured data, not prose. Same honesty, about six files, and D6 is never touched. |
| D5 | The four adoptions ship as two slices: slice 1 = D1, D2, D3 (docs lines plus one shell script, no parsed contract touched); slice 2 = D4 (the parser, its callers, and the skills that state the proof-line shape). | Slice 1 has no contract surface and can land on its own. Binding it to the parser change would hold two one-line rules behind a covered-contract change. |

### Agent's Discretion

Exact checklist wording for D3, the exact sentence wording for D1 and D2, the TTY
test used in `release.sh`, and where in `finish_support.rs` the optional field is
parsed. The user fixed the *behavior*, not the prose.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| baseline | What the same proof command produced on the base commit, before this change — the floor a result is read against. Not a target, not a threshold. |
| the whole block | Every check a change's proof depends on, including the ones that were skipped, narrowed, or went red. |
| smoke run | A deliberately narrowed run whose only job is to catch an environment fault before an expensive run starts. Never a reportable result. |

## Specific Ideas And References

- `RUNBOOK.md` §7 of the source: "Publish these together, never a subset … Quoting
  only the second reads as metric gaming." This is the sentence D1 translates.
- `verdict2/evaluate.py:99-108` of the source: the checklist is not advice, it is the
  only path to the command. D3 copies that shape, not the text.
- `reports/reference_floors.json` of the source: five floors in one committed file.
  bee's equivalent is per-proof-line, not a repo-wide file — bee has no single
  benchmark to floor.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs` — `parse_proof(s) -> Option<(String, String, String)>`, the one parser of the proof line. The single place D4 changes shape.
- `scripts/release.sh:68-84` — the option-parse loop where `--confirm` is added; `scripts/release.sh:269` is the warning line D3 replaces.

### Established Patterns

- Refuse-with-a-named-remedy — `release.sh` `fail()` already ends every refusal with the fix to run. D3's checklist follows it.
- The write/read split — `proof-strength-and-expiry` D2 (decision `cb7b14b7`) already proved it on this exact line: the WRITE path closes over a vocabulary while the READ path stays tolerant of historical records. D4 reuses it rather than inventing an optional-field scheme.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs` and `handlers_close.rs` — both call `parse_proof`; both must accept the fourth field without requiring it.
- `skills/bee-swarming/references/worker-details.md`, `skills/bee-planning/references/planning-reference.md` — these state the proof-line shape to workers. D4 is not done until they say the same thing as the parser.
- `packages/bee/AGENTS.block.md:109` § Prove, then say so — the REAL home for D1 and D2. `AGENTS.md` is generated: its `<!-- BEE:START -->` block is rendered from this template by onboard, so an edit to `AGENTS.md` itself is overwritten by the next `bee dev regen`.
- `.bee/expertise/tests.md`, `packages/bee/prompts/worker-cell.md`, `packages/bee/agents/bee-build.md.tmpl` — the three places a worker reads the proof-line contract at the moment it writes one. D4 is not done until they and the parser agree.
- `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md` R16c-a — the knowledge home of the proof line's shape and vocabulary. D4 updates it or it is stale.

## Canonical References

- `docs/history/research/openjev-verdict-2-xia.md` — the distill this feature implements, with the source anchors for all four adoptions.
- Decision `fda409ec` — the source manifest and resolved commit SHA.

## Prior Art On This Exact Line

### The sibling that already shipped D1 and D2

`docs/history/proof-completeness-rules/CONTEXT.md` (lane `docs`, phase
`compounding`). It shaped the SAME distill — arriving as spec drop `3d41a292`
from waggledance@aicoworker — and landed both rule lines as unmarked bullets in
`packages/bee/AGENTS.block.md:131-140`, regenerating `AGENTS.md:138-147`. Its
reading of the surface is better than this feature's first draft and is adopted
here wholesale:

- `AGENTS.md` is rendered, not authored; `packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs`
  pins the bytes between `<!-- BEE:START -->` and `<!-- BEE:END -->`.
- A `<!-- rule: … -->` marker costs three synchronized copies, pinned by
  `tests/rule_index_parity.rs`. Unmarked bullets carry no parity obligation, which
  is why both rules landed unmarked.

That work is uncommitted in the main checkout at shaping time. This feature does
not touch those two bullets, and treats their tree as its base.

### The fence on the proof line

Read before touching D4 — this line has been changed once already, deliberately and narrowly.

- `proof-strength-and-expiry` D1/D2/D3/D5/D6, decision `cb7b14b7`, knowledge home
  `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md` R16c-a. That
  feature closed the RESULT segment to `green:live` / `green:unit` / `green:static`
  and left the three-segment shape alone on purpose.
- Its **D6** reads: "No change to the proof line's three-segment shape." Its own
  rationale states why — "the blast radius is already 20+ files; widening it to the
  proof line's structure would put a second contract change in the same feature." So
  D6 is a scope boundary of that feature, not a property of the line. D4 no longer
  goes near it: the revised D4 (`f4261145`) leaves the three-segment shape byte for
  byte and puts the baseline in the cap report instead. The fence is respected, not
  argued with — which is also why no supersession of another feature's decision is
  needed.
- Measured blast radius of the shape: 26 files under
  `packages/bee-rs/crates/bee/src/verbs/cells`, 11 crate tests, plus the three worker
  contract files and the knowledge entry above. That measurement is what killed the
  first D4: it is the cost of touching the shape, and the revised D4 pays none of it.

## Outstanding Questions

### Deferred To Planning

- [ ] Does any other reader parse the proof line besides `parse_proof` — a hook, a herding pre-flight, the dashboard? The cross-cutting sweep in the distill left this unchecked, and an unchecked reader is not a clean one.
- [ ] Is `bee dev regen` worth a `--check` mode in this feature, or is it separate work? The distill found the gap; nothing in D1-D5 covers it.

## Deferred Ideas

- A failure gallery — a committed list of what a feature still gets wrong, beside whether its guard caught it. Real, but it is a new artifact type, not a guard on an existing one.
- Numbered, independently re-runnable controls with one receipt each. bee's cells already approximate it; the gap is not proven.
- A bee principle from the source's Phase-0 reasoning: a result systematically worse than random is inverted wiring, not weak capability. Belongs to bee-capturing, not to this feature.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
