---
area: feedback-digest
updated: 2026-07-22
migrated_to: docs/knowledge/areas/feedback-digest/
---

# Feedback Digest (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/feedback-digest/`](../knowledge/areas/feedback-digest/index.md)
(okf-foundation D20/D29/D37). Four concepts, split by TOPIC rather than the old spec's headings:
`data-model.md` owns the digest's own shape — the six allowed fields, the closed `kind` vocabulary,
how `pain` is computed, and the `dropped` list; `generation-and-refresh.md` owns how a repository
produces its own digest as a side effect of closing a feature, on request, or as a count only, and
how the closing routine's automatic refresh behaves; `cross-repo-trust-boundary.md` owns how the
maintainers' repository reads another repository's already-written digest as hostile input; and
`ranking-and-self-improvement.md` owns grouping the collected view into pressing findings and the
gated process that turns a ranking into a shipped, human-approved change to the workflow itself.
This path stays alive as a pointer stub — it is never deleted in this feature (D20) — and the anchor
map below sends every numbered anchor the old spec exposed to the concept that now owns it, so
existing citations keep resolving. Coverage is machine-checked by
`scripts/okf_migrate.mjs --check feedback-digest` in the verify chain (D35), against the pinned
pre-migration blob `eeb447e` (`3d69a2d`, 29 anchors — 0 B / 15 R / 6 E / 8 P — 26 unparsed blocks —
okf-migration-f2 F8/F9).

The 26 unparsed blocks are the source's entire "Behaviors & Operations" section — five markdown
subheadings (B1-B5), none carrying an id the classifier recognizes, holding 18 unnumbered bold-lead
paragraphs (Triggers/What is read/What is emitted/What blocks it/What each actor observes/
Reproducibility/How the boundary is enforced/Why the reader distrusts the writer/What it does) plus
8 un-ided continuation bullets (B2's field-by-field re-examination list; B4's grouping/score/floor
bullets) — none of them carries a numbered id, so none is invented into an anchor (D10); their
content is still carried, verbatim, into the concept whose topic it matches. This is the highest
unparsed ratio of the five areas pinned so far: roughly half this area's substance lives in
unnumbered prose, so the coverage gate governs less of it than it does elsewhere — worth knowing,
and not a reason to invent structure that was never there.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| R1 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | producing a digest is a side effect of closing a feature |
| R2 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | a digest carries the six allowed fields and nothing else |
| R3 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | no location outside the two workflow-owned areas is ever opened |
| R4 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | a record that cannot be made safe is dropped and counted |
| R5 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | the reader re-validates every field of every entry it did not itself produce |
| R6 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | a date is accepted only against a strict calendar format |
| R7 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | the reader's checks are never weaker than the writer's |
| R8 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | every allowed field owns a declared validator |
| R9 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | translating a record's type is idempotent |
| R10 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | a digest is a snapshot, regenerated whole |
| R11 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | generation never fails on bad input |
| R12 | [docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md](../knowledge/areas/feedback-digest/ranking-and-self-improvement.md) | a neutralization-wrapped title and its bare twin are the same title for grouping |
| R13 | [docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md](../knowledge/areas/feedback-digest/ranking-and-self-improvement.md) | the internal comparison form is never displayed |
| R14 | [docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md](../knowledge/areas/feedback-digest/ranking-and-self-improvement.md) | ranking is deterministic |
| R15 | [docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md](../knowledge/areas/feedback-digest/ranking-and-self-improvement.md) | the self-improvement process runs only in the maintainers' repository |
| E1 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | a repository with no friction, no findings, and no lessons produces a valid, empty digest |
| E2 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | a record location that does not exist is absent, and absence is not a containment violation |
| E3 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | a work item with no execution trace is skipped and counted |
| E4 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | a record whose text names an internal function, file, or configuration key is not a problem |
| E5 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | a source repository that has never closed a feature has no digest |
| E6 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | two generations from unchanged records differ only in the recorded generation moment |
| P1 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | collector, boundary, merge, and ranking — `packages/bee/lib/feedback.mjs` |
| P2 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | command surface — `packages/bee/bee.mjs` (`feedback` group) |
| P3 | [docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md](../knowledge/areas/feedback-digest/ranking-and-self-improvement.md) | the self-improvement process — `skills/bee-evolving/SKILL.md` |
| P4 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | source-repository list — `.bee/config.json` → `dogfood_repos` |
| P5 | [docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md](../knowledge/areas/feedback-digest/cross-repo-trust-boundary.md) | credential / instruction patterns and the neutralizer |
| P6 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | close-time refresh — `skills/bee-compounding/SKILL.md` step 8 |
| P7 | [docs/knowledge/areas/feedback-digest/generation-and-refresh.md](../knowledge/areas/feedback-digest/generation-and-refresh.md) | written artifact — `.bee/feedback-digest.json` |
| P8 | [docs/knowledge/areas/feedback-digest/data-model.md](../knowledge/areas/feedback-digest/data-model.md) | tests — `packages/bee/tests/test_lib.mjs` |
