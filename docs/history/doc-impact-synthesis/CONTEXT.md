# Doc Impact + Synthesis — Context

**Feature slug:** doc-impact-synthesis
**Date:** 2026-08-16
**Shaping session:** complete (locked directly from the user-approved phase-2 proposal; no interview — the approved message was the reading)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Phase 2 of the doc-rot work: the knowledge layer gains impact-driven
maintenance — a decision or feature close finds the affected docs through
declared citations and forces the fix; locked decisions synthesize into
area specs (the final picture); deferral conditions written in docs join
the trigger registry. Ends at the bee CLI + docs layer; no change to
cells, gates, claims, or worktree mechanics. Phase 1
(knowledge-distill-trigger) is the substrate: freshness door, trigger
registry, relation declarations.

## Why (user critique, 2026-08-16)

Phase 1 fixed bookkeeping; the user's core ask is doc QUALITY: docs are
"mere notes", compound output included; an architecture change must FIND
and FIX old docs, not add new ones; scattered CONTEXT/work-item decisions
must synthesize into one complete final picture; conditions in documents
need triggers — and scanning is never the mechanism, synthesis through
declared structure is.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every decision write carrying `touches:`/`supersedes:` relations, and every feature close, walks DECLARED citations backward: docs/** files citing the touched decision short8s (or the closing feature's new decision ids) become must-fix capture stubs; unfixed items block close through an impact door. Citations only — blind text scanning never. | Extends the proven supersede citation-sweep (found 3 stale citations same-day) to `touches` + close. |
| D2 | At close, every locked D-ID in the feature's CONTEXT.md decision table must be routed: merged into exactly one area spec (bundle citation present) or explicitly recorded feature-local. Close counts unrouted D-IDs and blocks. Specs are the final picture; CONTEXT.md is frozen history. | — |
| D3 | Deferral-shaped prose written into docs (area specs, CONTEXT.md, delivery records) must name a registered trigger id. Checked at close over the closing feature's TOUCHED doc files (bounded by the diff, never a repo scan); blocks with a create-the-trigger teach line. | Same registry and law as phase-1's decisions-log guard, extended to the doc layer. |
| D4 | Mechanism slices first, then ONE bounded backfill slice over live areas (the 2026-08-16 audit's semantic-staleness list). The full historical 110-CONTEXT routing sweep is a standalone backlog campaign row, outside this feature. | Keeps a mechanism feature from becoming archaeology. |

Decision log ids: D1 `doc-impact-synthesis D1` (touches c48a9b0d), D2
(touches 57834418), D3 (touches 35a14961), D4 (touches 57834418) — logged
2026-08-16.

### Agent's Discretion

Door naming and placement in the close chain, stub shapes, the
feature-local recording mechanism (decision tag vs CONTEXT marker), and
citation-detection regex (short8 forms) — planning's call within D1-D3
behavior. Reuse `sweep_decision_citations` and the phase-1 door/stub
machinery wherever it fits.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| impact walk | Following recorded citations from a touched decision id backward to the docs citing it — never a content scan |
| routed decision | A CONTEXT D-ID whose substance lives in exactly one area spec, or is recorded feature-local |
| final picture | The area spec as the single current-truth surface; CONTEXT.md rows are history |

## Existing Code Context

### Reusable Assets

- `sweep_decision_citations` (verbs/decisions/supersede.rs, referenced ~:195) — the D1 walk's existing core; today fires only inside `do_supersede`.
- `add_capture_stub` (supersede.rs:56-86) — the must-fix stub writer D1 reuses.
- Close door frame + knowledge-freshness door (drivers/close.rs, phase 1 kdt-1) — D1/D2/D3 doors slot the same way.
- Trigger registry + `trigger_registered()` (verbs/triggers/mod.rs, phase 1) — D3's id resolution.
- Deferral-prose matcher `matches_deferral_prose` (verbs/decisions/verbs_read.rs, phase 1 kdt-3) — D3 reuses the same word list.
- `feature_touched_files` (close.rs:873-892) — D3's bounded file set.

### Integration Points

- `.bee/capture-queue.jsonl` — D1's must-fix queue.
- CONTEXT.md locked-decision table format (docs/history/*/CONTEXT.md) — D2's parse source.
- `docs/knowledge/areas/*/` frontmatter `decisions` lists + body short8 citations — D2's routing evidence.

## Canonical References

- `docs/history/knowledge-distill-trigger/CONTEXT.md` + learnings.md — phase 1 substrate and its lessons (arming order, plan-check value).
- `skills/bee-capturing/references/citations.md` — the short8 citation discipline D1/D2 walk on.

## Outstanding Questions

### Sent To Planning

- [ ] CONTEXT.md table parsing: exact grammar bee can rely on (header row, D-ID column) — check bee-briefing/render precedents before writing a parser.
- [ ] D1 close-walk scope: new decision ids of the closing feature — where recorded (decisions.jsonl has no feature field; ids may need collecting via the feature's decision-log calls or scribing stamp window).
- [ ] D2 "exactly one spec" enforcement: is one citation anywhere enough, or must the spec's frontmatter decisions list carry it.
- [ ] D3 word list false positives in historical quotes — the door reads the diff's ADDED lines only?

## Ideas Filed Out Of Scope

- Full historical routing sweep of 110+ CONTEXT files — filed as a backlog
  campaign row (D4) by this feature's kds-4 close backfill.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and the questions
sent to planning.
