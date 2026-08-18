# Pocock Nuggets — Context

**Feature slug:** pocock-nuggets
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Quick
**Domain types:** READ

## Feature Boundary

Merge six one-paragraph craft nuggets distilled from mattpocock/skills
(xia brief: docs/history/research/pocock-skills-distill.md) into bee's
expertise and reference prose. Additive prose only — no behavior, CLI,
or workflow change. The two-axis review idea (nugget 4) is explicitly
NOT in scope — separate decision after `.bee/expertise/review.md` is
checked.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `expertise/planning.md` (build-order territory) gains expand–contract sequencing for wide mechanical refactors — expand beside the old, migrate call sites in blast-radius-sized batches, contract when no caller remains, full green promised only at contract — plus the prefactor line "make the change easy, then make the easy change" (nuggets 1+9) | — |
| D2 | `expertise/debugging.md` gains: generate 3–5 ranked falsifiable hypotheses before testing any (single-hypothesis generation anchors on the first plausible idea), and compare candidate fixes only after diagnosis — never propose a fix menu from the symptom (nuggets 2+3) | Lands beside the existing "State the hypothesis before the fix" section |
| D3 | `expertise/tests.md` gains the pre-agreed-seams ritual: before TDD on new surface, confirm "what is the public interface, and which seams do we test?" — then test only at those seams (nugget 5) | Lands beside "Pick the cheapest level that can fail" |
| D4 | Pinned terms carry an `_Avoid_` list of the losing synonyms: one line in bee-shaping's pinned-terms rule and one in bee-capturing's area-spec Data Dictionary rules (nugget 7) | skills/ files — regen obligation applies |
| D5 | Source provenance recorded per porting-protocol D4: mattpocock/skills @ 84fdeff, decision at this lock + capture stub | — |

### Agent's Discretion

Exact wording and placement inside each file, matching each file's
existing voice and section structure.

## Canonical References

- docs/history/research/pocock-skills-distill.md — the xia brief these
  nuggets come from (quotes and anchors)
- /home/thanhsmind/projects/AI/mattpocock-skill @ 84fdeff — source

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
reads locked decisions and canonical references.
