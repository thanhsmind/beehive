---
type: bee.area
title: Feedback Digest — Cross-Repository Collection and the Trust Boundary
description: "How the workflow's maintainers' repository reads other repositories' already-written digests as hostile input — re-validating and neutralizing every field before any of it can influence the workflow's own instructions."
timestamp: 2026-07-23
bee:
  id: feedback-digest-cross-repo-trust-boundary
  lifecycle: active
  areas: [feedback-digest]
  required_context: [areas/feedback-digest/data-model.md, areas/feedback-digest/generation-and-refresh.md]
  decisions: [D2 8cd4c84e, D2b 8cd4c84e, b8fe5c81, "ask-guard-autofix D3 (dead-source skip silent inside hook runs, 2026-07-23)"]
  sources: ["docs/history/evolving-loop/ (cells evolving-1 … evolving-11, capped)", docs/history/evolving-loop/reports/review-slice-a.md, docs/history/evolving-loop/reports/review-slice-b.md, "docs/history/cli-mutations/ (cell cli-mutations-2, capped; walkthrough.md)", ask-guard-autofix cell ag-2 (2026-07-23), "docs/specs/feedback-digest.md#R3", "docs/specs/feedback-digest.md#R4", "docs/specs/feedback-digest.md#R5", "docs/specs/feedback-digest.md#R6", "docs/specs/feedback-digest.md#E5", "docs/specs/feedback-digest.md#P1", "docs/specs/feedback-digest.md#P4", "docs/specs/feedback-digest.md#P5"]
  authoritative_for: "feedback-digest: cross-repository collection and trust boundary"
---

# Feedback Digest — Cross-Repository Collection and the Trust Boundary

Where `generation-and-refresh.md` is how a repository produces its own digest, this concept owns
what happens when a different repository — the workflow's maintainers' repository — reads it. A
digest is the only thing that ever crosses a repository boundary, and the reader trusts none of it:
it is scanned as hostile input, never as a friendly export.

## Entry Points & Triggers

| Trigger | Who fires it | Result |
|---|---|---|
| An operator asks the maintainers' repository to collect | Operator | Every configured source repository's already-written digest is read and merged into one view |

## Behaviors & Operations

### B2 — Collecting digests across repositories

**Triggers:** an operator asks the maintainers' repository to collect.

**What is read:** for each configured source repository, **only that repository's already-written
digest**. Never its raw records, never its code. The digest is the boundary; if the reader could
reach behind it, the boundary would exist nowhere.

**How the boundary is enforced.** The digest's location is followed to its true destination and must
still be an ordinary file inside the source repository's workflow-owned area. Then **every field of
every entry** is re-examined as hostile input:

- Values of the wrong shape become empty.
- The record type is re-translated through the closed vocabulary; an unknown type is dropped.
- Every text value is scanned for credentials and for text that tries to issue instructions. Any hit
  drops the whole entry, and the drop record's own fields are emptied so that the offending text is
  never stored either.
- The date value is accepted **only** if it matches a strict calendar format. It is not passed to a
  lenient interpreter, because lenient date interpreters ignore parenthesised text — which makes a
  date field a perfectly good smuggling channel.
- Every surviving text value is neutralized before it can ever be shown to a reader that acts on
  instructions.

**Why the reader distrusts the writer.** The producing repository scans its own records when it
writes them. That protects *that* repository from its own authors. It does not protect the reader,
who is a different party, reads a file the producer controls entirely, and uses what it reads to
change the workflow's own instructions. A digest edited by hand, or gone stale, or written with
intent, is just a file on disk.

**What blocks it:** nothing. A source repository that does not exist, cannot be read, has no digest
yet, or has a corrupt one, is skipped. One dead source never stops the reader. The skip is warned
about in interactive runs (status, onboarding); inside a lifecycle-checkpoint run the same skip is
silent, because the warning was surfacing on every checkpoint fire and reading as part of whatever
unrelated error appeared next to it (ask-guard-autofix D3, 2026-07-23).

**What each actor observes:** the operator sees, per source repository, how many entries survived and
how many were dropped with which reasons.

## Actors & Access

| Actor | May |
|---|---|
| A repository using the workflow | Produce its own digest. It never reads anyone else's |
| The workflow's maintainers' repository | Read its own digest, and the written digests of repositories it explicitly lists |
| An operator | Trigger generation, ask for counts, trigger collection |

A repository is a source only if the maintainers list it. Listing accepts either a bare location or a
location with a friendly label; a location with no label is labelled by its own final path segment.

## Business Rules

- **R3** (D2) — No location outside the two workflow-owned areas is ever opened, and containment is
  decided after following links to their true destination, not by comparing location names.
- **R4** (D2) — A record that cannot be made safe is dropped and counted, with its reason category
  recorded and its offending text never recorded.
- **R5** (D2b) — The reader re-validates every field of every entry it did not itself produce, and
  neutralizes every surviving text value before it can reach anything that acts on instructions.
  **Never trust a boundary artifact you did not produce.**
- **R6** (D2b) — A date is accepted only against a strict calendar format. Lenient interpretation is
  forbidden, because it silently ignores embedded text.

## Edge Cases Settled

- **E5** — A source repository that has never closed a feature has no digest; it is skipped, not an
  error.

## Open Gaps

- **Neutralization is escapable.** The neutralizing wrapper marks text with delimiters but does not
  remove those delimiters from the text itself, so a value containing the closing delimiter escapes
  the wrapper. It also leaves text-direction override characters intact.
- **Credential and instruction detection is heuristic.** Several widely used credential formats and
  several ordinary phrasings of an instruction are not recognized. Detection reduces risk; it does
  not establish a boundary.
- **A foreign digest is read without a size limit**, so an enormous one exhausts the reader's memory —
  which contradicts the rule that one dead source never stops the reader.

## Pointers (implementation)

- **P1** — Collector, boundary, merge, and ranking: `packages/bee/lib/feedback.mjs`
  (`ENTRY_FIELD_SPEC`, `resolveInScope`, `listInScope`, `buildDigest`, `mergeDigests`,
  `normalizeTitle`, `clusterEntries`, `rankClusters`)
- **P4** — Source-repository list: `.bee/config.json` → `dogfood_repos`, normalized in
  `packages/bee/lib/state.mjs`
- **P5** — Credential / instruction patterns and the neutralizer:
  `packages/bee/lib/decisions.mjs`
