---
type: bee.area
title: Bee OKF Profile — the migration loop and the gates that grade it
description: "What a legacy source must satisfy to become bundle truth: the coverage report, the content-addressed pin, the unparsed report, the fidelity floor, the drift telemetry, and the pointer stub that keeps every citation resolving."
timestamp: 2026-07-23
bee:
  id: okf-profile-migration-and-coverage-gates
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: [D10, D20, D29, D30, D35, D37, "F8/F11/F12 (okf-migration-f2 — the derived pin, the fidelity floor, the scheme-keyed drift band)", F4-D10]
  sources: ["okf-foundation cell okf-6 (critical-patterns.md -> patterns/ migration, work/okf-foundation/ work item + plan concepts, Templates section; trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-foundation/CONTEXT.md`, "okf-switchover-f3 cell f3-5 (G6 — this spec migrated into the bundle it describes; trace in `.bee/cells/`, 2026-07-22)", "docs/specs/okf-profile.md#B8", "docs/specs/okf-profile.md#B9", "docs/specs/okf-profile.md#B10", "docs/specs/okf-profile.md#B11", "docs/specs/okf-profile.md#B12", "docs/specs/okf-profile.md#E3", "docs/specs/okf-profile.md#E4", "docs/specs/okf-profile.md#P5", "okf-integration-close-f4 cell f4-7 (the shape ratios count anchor-owning concepts, so a migrated area is not punished for growing; denominator corrected, band untouched; trace in `.bee/cells/`, 2026-07-23)"]
  authoritative_for: "okf-profile: the migration loop, its coverage gate and its fidelity floor"
---

# Bee OKF Profile — The Migration Loop and the Gates That Grade It

This concept owns what happens when a legacy source moves into the bundle: the coverage report that
proves nothing was lost, the content-addressed pin that makes that proof honest, the unparsed report
that keeps format-blindness visible, the fidelity floor that separates migration from summary, and
the drift telemetry that compares only comparable shapes.

## Data Dictionary

**Coverage report (D35):** every migration emits a report proving each numbered behavior, rule,
edge case, and pointer bullet in the source (e.g. `B1`…`B36`, `R1`…`R55`) lands in **exactly one**
concept — no loss, no duplication. The verify chain asserts the report. This is what lets a
1464-line re-authoring (`workflow-state.md`, F2 — D30) proceed on evidence instead of care.

**Pointer-stub anchor map (D37):** a migrated legacy file is never deleted in this feature (D20);
it becomes a pointer stub carrying an anchor map — every numbered anchor the source exposed
(e.g. `B17`, `R26`) mapped to the concept path that now owns it. Any citation into the migrated
file (e.g. `docs/specs/reading-map.md`'s citations of `workflow-state.md` `B17`/`B18`, `R26`/`R27`,
`B33`, `R51`) is rewired in the **same cell** as the stub — a path-only stub would preserve the
path and silently destroy the anchors, which is the exact failure D37 exists to eliminate.

## Behaviors & Operations

**B8 — Migration is gated by anchor coverage (D35).** Every migration of a legacy source into the
bundle is guarded by a chain suite that asserts set-equality: every numbered anchor in the frozen
source inventory (`B*`/`R*`/`E*`/`P*` for a nine-section BA spec, `PAT*` for the flat pattern list)
is claimed by **exactly one** concept's `bee.sources`, and the pointer stub's anchor map agrees.
No loss, no duplication. Shipped coverage: `advisor-protocol` 26/26, `critical-patterns` 47/47.
*Known limit:* the frozen inventory is a hand-editable constant with no cryptographic tie to git
history — an editor who shrinks the inventory and the concepts together keeps the gate green.
Binding it to the pre-migration blob is open work.

**B9 — A pin is content-addressed, and no unverified extraction may read as a pass (F8).** Every
migrated source is pinned as `{commit, path, blob_sha, scheme, expected_counts}` and **all five are
asserted at check time**. The pinned bytes are also committed verbatim under
`docs/history/okf-migration-f2/sources/<area>.md` and verified with `git hash-object`, so a
`--depth 1` clone — where `git show <sha>:<path>` fails outright — still verifies. Every failure
mode is a typed refusal with exit 1, never a silent skip: `PIN_NO_SCHEME`, `PIN_UNKNOWN_SCHEME`,
`PIN_INCOMPLETE` (including a missing `unparsed_blocks`), `PIN_SHA_MISMATCH`, `PIN_COPY_MISSING`,
`PIN_UNRESOLVED`, `PIN_DUPLICATE_ANCHOR`, and `PIN_EMPTY_EXTRACTION` — which is raised **before**
the count comparison, so a pin declaring `total: 0` cannot launder itself green. Where a source had
to be repaired before pinning (duplicate ids), the pin declares `repaired_from` + `repair_reason`
and the git leg asserts the **provenance** blob; an undeclared or unexplained disagreement stays a
`PIN_SHA_MISMATCH`.

**B10 — The extractor reports what it could not read (F8).** Every inventory returns unparsed block
and line counts per section, and `unparsed_blocks` is a **mandatory** pin field. This exists because
the original classifier required bare anchor ids and was blind to the `- **R1** — …` form, hiding
**86 anchors across five areas** and making two areas look "shapeless" when they were merely
unreadable — a blindness that converts *lost content* into *content that never existed*. Three
schemes ship: `ba-nine-section` (numbered `B*/R*/E*/P*`, letter suffixes and bold wrapping
accepted), `flat-pattern-list` (`## [YYYYMMDD] title` headings), and `narrative-sections` (the
source's own `## ` headings ARE the anchors, for a source with no numbering at all — a `###`
subheading is not an anchor, and a source with zero `## ` headings is refused, never passed 0/0).
Anchor ids are **read, never invented**: an unnumbered block stays unparsed and is counted.

**B11 — The fidelity floor measures whether content was migrated or summarised away (F11).** For
each anchor, the owning concept's **body** (never its frontmatter) must retain **≥ 0.60** normalized
token overlap with that anchor's text in the pinned blob; below the floor fails the gate naming the
anchor, its owner, and the ratio. Normalization lowercases, collapses non-alphanumerics, and drops
only articles, prepositions, pronouns and auxiliaries — **modal and negation words (`never`,
`always`, `must`, `only`, `refuses`) are content here and are never stopwords**. The metric
discriminates rather than merely detecting absence: a faithful re-wording scores 0.815, a markdown
re-format 0.963, a plausible paraphrase 0.296, a gutted concept 0.000. *Known limit:* the metric is
lexical, so a faithful rewrite that **renames terminology** scores near zero — the resolution is to
keep the source's terms (migration re-homes content, it does not improve its wording), never to
lower the floor. A suite assertion requires each area's **median ≥ 0.75**, so a future
over-strict normalization that hugs the floor goes red instead of passing quietly.

**B12 — Drift telemetry compares only comparable shapes (F12).** Each pinned source reports
`anchors_per_concept` and `concepts_per_100_source_lines`, failing when it is an outlier against
the running median of already-pinned sources **of the same scheme**; with fewer than three
comparable samples there is no median and it reports only. Comparability is keyed by scheme
because a `flat-pattern-list` migration is one anchor per concept *by construction* and can never
sit in a band drawn around nine-section areas — pooling them turned an already-shipped area red on
work it never touched. The gate additionally runs the whole-bundle invariants every check
(authority uniqueness, zero `not_canonical`, index freshness) and treats those three as hard
failures **for itself**, leaving `knowledge check`'s own non-strict exit contract (D13) untouched.

**B12a — The shape ratios count anchor-OWNING concepts, because a migrated area is allowed to
grow.** Both ratios measure how a *pinned source* was decomposed, so their denominator is the
number of concepts the coverage walk actually attributes an anchor to — never every file sitting in
the area's directory. Counting files conflates two different things: concepts that carry migrated
truth, and concepts of genuinely NEW truth authored later. Only the first kind can ever raise the
numerator, so a file-counting denominator drifts a healthy area out of band permanently the moment
it gains new truth — punishing exactly the behaviour a live area is supposed to show. It stayed
invisible while every area's two counts happened to coincide, and surfaced the first time a
migrated area grew.

The correction is to the denominator, never to the band: the outlier band and the
minimum-sample count are unchanged, no pin is altered, and no area is excluded from its population.
That distinction is the whole point — widening a band to clear a red removes the signal, while
correcting what is counted restores it. The evidence that it restores rather than removes: the
file-counting denominator scored an area with **every** anchor dumped into a single concept as
comfortably in-band, because the empty sibling files diluted it. Both failure directions —
one dumping-ground concept, and anchors shredded one per concept — are held by a negative control,
alongside a healthy fixture that must pass, so the control discriminates instead of always firing.
Each row also reports its directory file count beside its owning-concept count, leaving the growth
gap visible rather than silent.

## Business Rules

- A migration is not "done" until its coverage report accounts for every numbered source anchor
  exactly once (D35) and its pointer stub carries the anchor map those citations depend on (D37).

## Edge Cases Settled

- A pointer stub is authored in the **same cell** as the citations that point into the file it
  replaces, closing the gap where a path survives migration but a numbered anchor a citation
  depends on does not (D37; the verified gap was `docs/specs/reading-map.md:101-102` citing
  `workflow-state.md` `B17`/`B18`/`R26`/`R27` and `B33`/`R51`).
- Migrated legacy files are **not deleted** in this feature (D20) — stubs keep existing consumers
  (`.bee/bin/lib/inject.mjs:70-95`'s filename-only spec count; `packages/bee/hooks/bee-session-close.mjs:100-140`'s
  mtime-based staleness nudge) working through the migration without a flag day.

## Pointers (implementation)

- F1's proof area (D29), still living in its legacy location pending its own migration cell:
  `docs/specs/advisor-protocol.md`.
