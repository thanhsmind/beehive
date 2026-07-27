---
type: bee.area
title: Deciding whether two paths name the same location
description: "Separator meaning is a platform property, case behaviour is a per-volume one, and a zero device or index is absent rather than a value. Every ambiguity resolves to 'different', because a refused legitimate operation is a retry while an accepted wrong location is not recoverable."
tags: [worktree-parallelism, path-identity, cross-platform, merge-safety]
timestamp: 2026-07-27
bee:
  id: area-worktree-parallelism-path-identity
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: []
  decisions: []
  sources: ["windows-path-identity (cells wpi-1, wpi-2, wpi-3)", docs/history/windows-path-identity/plan.md, docs/history/windows-path-identity/reports/wpi-1.md]
  authoritative_for: "worktree-parallelism: deciding whether two path strings name the same location"
---

## Purpose

Two strings can name the same directory without being the same string. Every place bee decides "are these the same location?" — most consequentially when resolving a linked worktree before a merge — needs an answer that survives the differences between how a version-control tool spells a path and how the runtime spells it, without ever calling two genuinely different locations the same.

## Entry Points & Triggers

- Resolving a worktree from its recorded id, before a merge is staged. A wrong answer here either refuses a legitimate merge or, far worse, accepts the wrong location.
- Any comparison between a path obtained from a tool's output and a path built by the runtime. These are byte-identical on many systems and need not be on all of them.

## Data Dictionary

- **Separator form** — which character divides path segments. This is decided by the platform: some platforms accept both forms and treat them alike; others treat one of them as an ordinary character inside a name.
- **Case behaviour** — whether two spellings differing only in letter case name the same file. This is decided by the **volume**, not the platform: a single machine can carry both kinds at once, and a directory can be configured against its filesystem's default.
- **Filesystem identity** — the device and index numbers a filesystem reports for an entry. Where both are meaningful, two entries sharing them are the same entry.
- **Unusable identity** — a reported device or index of zero, which some filesystems return because they keep no such index. Treating zero as a value makes every entry on such a volume identical to every other.

## Behaviors & Operations

**Deciding sameness.** Both paths are resolved to absolute form using the rules of the platform in play — which is what settles separator differences, because that platform's own resolver knows whether a given character separates or belongs to a name. Where both paths exist and the filesystem reports meaningful identity, that identity decides. Otherwise the resolved forms are compared as text, folding case only when the volume in play has been shown to be case-insensitive.

**Deciding case behaviour.** The volume is asked, not assumed. The preferred question is read-only: take an existing entry, ask for it under a different case, and see whether the filesystem answers with the same entry. Only when no such question can be posed — no letters in the name to flip, or nothing readable to sample — does the check fall back to writing a temporary probe and looking for it under the other case. Both sides of a comparison must independently report case-insensitivity before case is folded; one side's answer is never applied to the other.

**Refusing.** Every failure to establish a fact resolves toward "not the same": an unreadable path, an unwritable directory during probing, a probe interrupted by a concurrent change. A refusal costs a legitimate operation a retry; a wrong acceptance costs correctness.

## Actors & Access

- **The merge path** is the consequential consumer: it resolves a worktree by id and refuses the operation when identity cannot be established.
- **Tests** may inject both the platform rules and the case answer, which is the only way to exercise the other platform's behaviour from this one.

## Business Rules

- **R1** — Separator handling is delegated to the platform's own resolver and never hand-rolled. A hand-rolled fold applied everywhere turns a legal filename character into a separator on the platforms where it is legal, which makes two different directories compare equal.
- **R2** — Case folding is decided per volume, never from the platform name, and requires both sides to agree.
- **R3** — A zero device or index is treated as absent, not as a value, and the comparison falls back to text.
- **R4** — A comparison prefers a read-only question over one that writes. Where writing is unavoidable it is a last resort, because a comparison that leaves a file behind is a comparison with a side effect.
- **R5** — Every ambiguity resolves to "different". Refusing a legitimate operation is recoverable; accepting the wrong location is not.

## Edge Cases Settled

- **The fix's own failure mode is the one it prevents.** The first implementation folded one separator form into the other on every platform, on the incorrect belief that the character could not appear inside a name. On platforms where it can, a directory whose name contains it compared equal to a genuinely different nested path — and the identity check then examined the wrong location entirely. The correction was to stop hand-rolling the fold at all.
- **A cache keyed by volume can still collide.** Where a filesystem reports a zero device, every such volume shares one cache slot, so an answer probed on one can be handed to another. Recorded as a known limit rather than fixed, because it needs a filesystem this project cannot currently exercise.

## Open Gaps

- The case-behaviour answer is cached for the life of the process and is not invalidated if a volume is remounted or replaced.
- Comparisons injected with another platform's rules run their volume question against the ambient platform's paths, so under injection the question is answered but means little — harmless for correctness, since both sides get the same answer, but worth knowing when reading an injected test.
- Nothing here is verified on the other platform from this one. The proofs establish the semantics; a run on that platform is what confirms the behaviour.

## Pointers (implementation)

- `packages/bee/lib/path-identity.mjs` (`canonicalPathsEqual`, its `platformPath` and case-probe injection seams).
- Consumed at `packages/bee/lib/worktree-store.mjs`'s `resolveWorktreeById` bidirectional gitdir check, threaded through the merge entry points' options.
- Tests: `scripts/tests/test_path_identity.mjs`.
