# Review Axes — Context

**Feature slug:** review-axes
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Quick
**Domain types:** READ

## Feature Boundary

bee-reviewing's report separates two finding axes without splitting the
report, and `expertise/review.md` gains the named smell vocabulary.
Prose-only change to review skill/reference/expertise files — no CLI,
no severity mechanics change.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every finding carries an axis label — `standards` (is the code well made: quality, architecture, security, tests) or `spec` (does it do what the locked decisions promised). The synthesis report stays ONE report but groups findings by axis, spec-axis group first. Severity rules (P1–P3, corroboration promotion, conservative disagreement) are unchanged and apply within and across axes; a P1 blocks regardless of axis. Axes are never merged into one undifferentiated ranked list again | Spec-conformance findings must not drown under style findings (pocock code-review's axis separation, light form) |
| D2 | `expertise/review.md` ("Style versus substance" territory) gains the 12 named Fowler smells, one line each (what it is → the usual fix), prefaced by: a documented repo standard always wins, and every smell is a labelled heuristic — a judgement call, never a hard violation | Shared vocabulary for reviewers; the caveat keeps it from becoming a lint list |
| D3 | Source provenance per porting-protocol D4: mattpocock/skills @ 84fdeff (code-review skill), decision at this lock + capture stub | — |

### Agent's Discretion

Exact wording, placement, and how the axis label rides the finding
format (a field line beside severity is the expected shape) — matching
each file's existing voice.

## Canonical References

- docs/history/research/pocock-skills-distill.md — nugget 4/4b origin
- /home/thanhsmind/projects/AI/mattpocock-skill/skills/engineering/code-review/SKILL.md
  @ 84fdeff — the 12 smell definitions (L45-56) and the axis rules

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
