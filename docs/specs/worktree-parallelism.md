---
area: worktree-parallelism
updated: 2026-07-22
migrated_to: docs/knowledge/areas/worktree-parallelism/
---

# Spec — Worktree feature parallelism (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/worktree-parallelism/`](../knowledge/areas/worktree-parallelism/index.md)
(okf-foundation D20/D29/D37). Seven concepts, split by TOPIC: `overview.md` owns why this area
exists at all — the two different kinds of parallelism, and the surfaces deliberately left out
of scope; `the-trust-model.md` owns the load-bearing rule, that a worktree gets its own store
only when it is granted from a source it cannot forge;
`entering-creating-and-registering.md` owns the paved road in — `worktree new`, the `register`
adoption path, the bootstrap contract and every typed zero-mutation refusal;
`returning-and-the-merge-gate.md` owns the way back — the staged merge, the verify gate that
catches a semantic conflict, and post-commit cleanup; `routing-and-visibility.md` owns whether
to take a worktree at all, the lane-first refinement that defers the grant to Gate 2's execution component, and the
notices an occupied checkout prints so isolation is never silently absent;
`cross-worktree-holds.md` owns the shared holds ledger that makes an island's write intent
visible to its siblings; and `store-tiers-and-where-it-lives.md` owns which parts of the store
merge, which are rebuilt, which never travel, and where every piece of the mechanism is
implemented and proven. This path stays alive as a pointer stub — it is never deleted in this
feature (D20) — and the anchor map below sends every section the old spec exposed to the
concept that now owns it, so existing citations keep resolving. Coverage is machine-checked by
`scripts/okf_migrate.mjs --check worktree-parallelism` in the verify chain (D35), against the
pinned pre-migration blob `df2f441` (10 anchors, 2 unparsed blocks — okf-migration-f2 F8/F9).

## Why this area's anchors are section slugs, not numbered ids

This is the one area of the eleven that genuinely had **no numbered anchors**. Every earlier
"shapeless" verdict in this migration turned out to be a blind reader — `decision-memory`'s nine
rules were written `- **R1 — …**` and the classifier widening found them — but this source
really does carry no `B*`/`R*`/`E*`/`P*` id anywhere in its 225 lines, and none of the four
anchor-bearing nine-section headings. The `ba-nine-section` scheme derives 0 anchors AND 0
unparsed blocks from it, which is what real shapelessness looks like.

F9 forbids forcing an area into the nine-section shape, and D10 forbids inventing numbered ids
a source never had — the two together rule out both easy answers. So `okf_migrate` gained a
**third scheme**, `narrative-sections`, and this area is its first user: **the source's own
`## ` headings ARE the anchors**, slugified mechanically from the heading text (`S-` plus the
heading, lowercased, every non-alphanumeric run collapsed to a hyphen). Nothing is invented —
the structure the author actually wrote is the ground truth. This mirrors the `flat-pattern-list`
precedent, which already treats a `## [YYYYMMDD] …` heading as an anchor.

Three boundaries hold the scheme in place: a `## ` heading is an anchor and a `###` subheading
is **not** (its prose travels with the section that contains it, so the fidelity floor measures
it there); a source with **zero** `## ` headings is REFUSED rather than passed 0/0; and two
headings that slugify to the same id are refused outright, so the duplicate-id hazard that had
to be repaired in `hook-runtime.md` can never arise here.

The 2 unparsed blocks are the `**Area:**` and `**Status:**` lines of the document preamble,
which sit before the first `## ` heading and therefore belong to no section. Neither is invented
into an anchor (D10); both travel verbatim into `overview.md`.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| S-what-problem-this-solves | [docs/knowledge/areas/worktree-parallelism/overview.md](../knowledge/areas/worktree-parallelism/overview.md) | What problem this solves — swarm-worker worktrees (P40) vs independent-feature worktrees |
| S-the-trust-model-the-load-bearing-rule | [docs/knowledge/areas/worktree-parallelism/the-trust-model.md](../knowledge/areas/worktree-parallelism/the-trust-model.md) | The trust model (the load-bearing rule) — grants, git-verified ids, per-write resolution, why a worktree cannot grant itself |
| S-registering-a-worktree-the-cli | [docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md](../knowledge/areas/worktree-parallelism/entering-creating-and-registering.md) | Registering a worktree (the CLI) — register / list / unregister and the bootstrap contract |
| S-entering-worktree-new-feature-slug-d7-gh-21 | [docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md](../knowledge/areas/worktree-parallelism/entering-creating-and-registering.md) | Entering: `worktree new --feature <slug>` (D7, GH #21) — the paved road in, and its typed zero-mutation refusals |
| S-returning-worktree-merge-id-id-d8 | [docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md](../knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md) | Returning: `worktree merge --id <id>` (D8) — the staged transaction, the verify gate, and `--cleanup` |
| S-routing-rule-d9-prose-not-a-hook | [docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md](../knowledge/areas/worktree-parallelism/routing-and-visibility.md) | Routing rule (D9 — prose, not a hook), the worktree-ux visibility notices, and the lane-first refinement |
| S-cross-worktree-holds-the-shared-ledger-cross-worktree-holds-d1-d6-2026-07-20 | [docs/knowledge/areas/worktree-parallelism/cross-worktree-holds.md](../knowledge/areas/worktree-parallelism/cross-worktree-holds.md) | Cross-worktree holds (the shared ledger — cross-worktree-holds D1-D6, 2026-07-20) |
| S-the-three-tiers-what-merges-what-does-not | [docs/knowledge/areas/worktree-parallelism/store-tiers-and-where-it-lives.md](../knowledge/areas/worktree-parallelism/store-tiers-and-where-it-lives.md) | The three tiers (what merges, what does not) — log / cache / runtime |
| S-boundary-out-of-scope | [docs/knowledge/areas/worktree-parallelism/overview.md](../knowledge/areas/worktree-parallelism/overview.md) | Boundary (out of scope) |
| S-where-it-lives-reading-map | [docs/knowledge/areas/worktree-parallelism/store-tiers-and-where-it-lives.md](../knowledge/areas/worktree-parallelism/store-tiers-and-where-it-lives.md) | Where it lives (reading map) — modules, resolver, CLI, merge safety, ledger, tests |
