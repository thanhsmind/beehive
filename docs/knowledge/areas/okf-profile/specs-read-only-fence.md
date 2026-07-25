---
type: bee.area
title: Bee OKF Profile — the read-only fence over the compatibility surface
description: "Once a repo has a knowledge bundle, the legacy spec tree is read-only for NEW content: every file is classified structurally as a pointer stub, a named navigation surface, or a pinned unwritten placeholder, and anything else fails the chain by name."
timestamp: 2026-07-22
bee:
  id: okf-profile-specs-read-only-fence
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md, areas/okf-profile/migration-and-coverage-gates.md]
  decisions: [G1, G2, G3, G4, G6, G13, D20, D37]
  sources: ["okf-switchover-f3 cell f3-4 (the fence, red-first: every selftest assertion written and run against a classifier that classified nothing; trace in `.bee/cells/`, 2026-07-22)", "okf-switchover-f3 cell f3-5 (G6 — the profile migrated, and its named exception removed rather than relabelled; trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-switchover-f3/CONTEXT.md`]
  authoritative_for: "okf-profile: the docs/specs read-only fence"
---

# Bee OKF Profile — The Read-Only Fence Over the Compatibility Surface

## Purpose

Migration leaves behind a compatibility surface: the legacy spec tree, kept alive so existing
citations keep resolving (D20 — a migrated source path is never deleted). A surface kept alive is
a surface people keep writing into, and every new page written there is truth authored outside the
bundle that the bundle's own gates cannot see. This concept owns the rule that stops it: once a
repo has a bundle, the legacy tree is **read-only for new content**, and the rule is enforced
mechanically rather than asked for politely.

## Entry Points & Triggers

- The verify chain runs the fence on every green run, in two forms: a self-test that proves the
  classifier actually bites, and a check of this repo's own compatibility surface.
- Any agent about to write an area document is routed by the authoring gate first
  (`concept-model-and-authoring.md`); the fence is the backstop for the write that got past it.

## Data Dictionary

Every file under the compatibility surface is classified as exactly one of these, and the verdict
is reported per file:

| Verdict | What it means |
|---|---|
| `stub` | a **pointer stub** — a migrated source path whose frontmatter carries the migrated-to marker the migration loop writes, plus the anchor map that redirects every numbered anchor to its owning concept (D37) |
| `navigation` | the hand-written reading map — a "where does X live" surface that points *at* the bundle and is never area truth itself (G4) |
| `placeholder` | a **named, reasoned, pinned** file that holds no content to migrate, allowlisted only while it stays empty of meaning |
| new content | anything else — a failure, named by file |
| `inert` | the whole check, in a repo with no bundle: it does not scan at all |

## Behaviors & Operations

**A stub is recognised STRUCTURALLY, never by name.** The classifier reads each file's frontmatter
and asks whether it carries the migrated-to marker. It never consults a list of filenames. A
filename list rots the first time an area is added or renamed — and a rotted allowlist stops
fencing **silently**, which is the exact failure class this whole programme exists to prevent. The
consequence is deliberate and load-bearing: a stub whose marker is dropped or whose frontmatter
breaks stops being a stub and fails as new content, loudly, the same day.

**Named exceptions are closed, reasoned, and printed.** The set of files that pass by name is
closed by decision, each member carries its reason in the classifier itself, and the reasons are
emitted in the machine-readable report — so a reader can always see *why* a file was let through
without reading the implementation. Today the set holds one member: the reading map.

**A placeholder is pinned to its emptiness.** A file with genuinely nothing to migrate has no stub
to point anywhere, so it is allowlisted by name — but only while a marker in its own text says it
is unwritten. The moment somebody actually writes it, the marker goes, the allowlist stops
applying, and the fence fires and says where the content belongs: a concept in the bundle. The
allowlist can therefore never become the hole through which the very tree it just made read-only
gets authored as prose.

**The fence is gated on bundle mode, and inert without one.** Bee ships to other repos. A host
that never migrated must keep writing its spec tree freely and must not be able to tell this
release happened — so with no bundle the check does not scan at all and reports itself inert. It
asks the same single bundle-mode predicate the scribe's authoring gate asks
(`concept-model-and-authoring.md`); it never re-implements "is this repo migrated", because two
answers to one question drift apart.

**The compatibility surface is product documentation and resolves off the product root** (G13),
exactly like the bundle. In a repo that separates its workshop from the product it documents, the
workshop's own empty spec tree is never the tree that gets graded.

**A named exception is removed when its interval ends, never relabelled.** The profile spec — the
document that defines the bundle — was itself a named exception while it awaited its own migration
(G6). The moment it was migrated and grew a marker of its own, the exception was **deleted**, not
re-pointed at the new verdict. Relabelling would have left a name-based pass that keeps saying yes
if that file's marker is ever dropped: precisely the silent rot the structural branch exists to
close. Both directions are asserted in the self-test — the real stub passes structurally, and the
same filename *without* a marker fails as new content.

## Actors & Access

- **The scribe** is routed to the bundle before it writes; the fence is not its normal path.
- **Any agent or human** editing an existing stub, the reading map, or a placeholder is unaffected
  — the fence governs NEW content, not maintenance of what is already classified.
- **A downstream host repo with no bundle** never sees the fence at all.

## Business Rules

- R1 — Once a repo has a bundle, the compatibility surface accepts no new area truth; new truth is
  written as a concept in the bundle (G2/G3).
- R2 — A pointer stub is recognised only by its structural marker; recognising stubs by filename is
  prohibited outright, because a rotted list fails silently (G2).
- R3 — Named exceptions are closed by decision and each carries its reason in the classifier, which
  the report prints (G2).
- R4 — A placeholder passes only while it is provably unwritten; writing it is a failure that names
  where the content belongs (G2).
- R5 — With no bundle the fence is inert and scans nothing (G1).
- R6 — A named exception whose interval has ended is removed, never relabelled (G6).
- R7 — A migrated source path is never deleted; it stays alive as a stub so existing citations keep
  resolving (D20/D37).

## Edge Cases Settled

- **A stub that loses its marker.** It fails as new content. This is the intended behaviour, not a
  false positive: an unmarked file in a read-only tree is indistinguishable from freshly authored
  truth, and treating it as a stub on the strength of its filename is the silent-rot failure.
- **A dangling stub** — one whose marker points at a bundle path that does not exist — is reported
  as its own verdict class, not lumped in with new content, so the report distinguishes "wrote
  something it should not have" from "the migration target moved".
- **The system overview** is a decision, not an oversight: it is an unwritten placeholder holding
  no content to migrate, so no stub exists to point anywhere. It is pinned to that state.

## Open Gaps

- The fence classifies; it does not migrate. A file that fails is named, and moving its content
  into the bundle is the migration loop's job (`migration-and-coverage-gates.md`).

## Pointers (implementation)

- The classifier, its verdicts, its named exceptions and its pinned placeholders:
  `scripts/okf_specs_fence.mjs` (`fenceFindings`, `NAMED_EXCEPTIONS`, `PLACEHOLDERS`).
- Red-first proof and both directions of the profile-stub assertion:
  `node scripts/okf_specs_fence.mjs --selftest`.
- Wired into the chain as two entries (`--selftest` and `--check`): `scripts/run_verify.mjs`
  `EXTRA_SUITES`, pinned in `scripts/tests/test_verify_manifest.mjs` `MANDATORY_SUITE_ARGS`.
- The shared bundle-mode predicate and product-root resolver:
  `packages/bee/lib/knowledge.mjs` (`bundleMode`) and
  `packages/bee/lib/state.mjs` (`resolveProductRoot`).
