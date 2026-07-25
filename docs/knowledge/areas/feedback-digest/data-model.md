---
type: bee.area
title: Feedback Digest — Data Model
description: "The digest's own shape: the six allowed fields an entry may carry, the closed kind vocabulary, how pain is computed once, the dropped list, and the scoped, gated auto-commit on filing — what a digest may hold and what it may never hold."
timestamp: 2026-07-23
bee:
  id: feedback-digest-data-model
  lifecycle: active
  areas: [feedback-digest]
  required_context: []
  decisions: ["D2 8cd4c84e, 9880542e, c45d0fb3", D 9157d074, D1 backlog-auto-commit, D2 backlog-auto-commit]
  sources: ["docs/history/evolving-loop/ (cells evolving-1 … evolving-11, capped)", docs/history/evolving-loop/reports/review-slice-a.md, docs/history/evolving-loop/reports/review-slice-b.md, "docs/history/cli-mutations/ (cell cli-mutations-2, capped; walkthrough.md)", "docs/specs/feedback-digest.md#R2", "docs/specs/feedback-digest.md#R7", "docs/specs/feedback-digest.md#R8", "docs/specs/feedback-digest.md#R9", "docs/specs/feedback-digest.md#E4", "docs/specs/feedback-digest.md#P8", "docs/history/backlog-auto-commit/ (cell backlog-auto-commit-1, capped)", "docs/history/backlog-auto-commit/ (cell backlog-auto-commit-2, capped)"]
  authoritative_for: "feedback-digest: data model"
---

# Feedback Digest — Data Model

This concept owns the digest's own shape: what a digest is built of when a repository writes
it, and what may not be written at all. Producing a digest from a repository's own records is
`generation-and-refresh.md`; safely reading someone else's already-written digest is
`cross-repo-trust-boundary.md`; scoring and acting on the collected view is
`ranking-and-self-improvement.md`.

## Entry Points & Triggers

- **An operator or agent files a friction/finding record** — Operator or any agent, at any
  time. The record is validated **at intake** — its type must belong to the closed vocabulary
  (either spelling the digest accepts), its severity to the three-level scale, its label capped
  in length — and only then appended to the repository's raw records. A record that fails
  validation is refused with a corrective message naming the allowed values, and nothing is
  written.

  Committing that append to the repository's history is a separate, explicitly requested step —
  never an automatic side effect of every successful filing. The filer states, at filing time,
  whether this record is a **new item being submitted into the processing queue** for someone
  else to pick up later, or the filer's **own observation** about work already in flight. Only a
  queue-submission is committed, scoped to just the raw-records file — no other in-progress
  change in the working copy is touched or swept in. A self-observation is appended to the raw
  records exactly the same as before, but is never committed on its own authority: committing an
  agent's own mid-session friction/debt/finding note would land it on the shared history before
  anyone reviewed it, so that class of record waits for whoever explicitly commits later. Outside
  a version-controlled working copy the append still succeeds and no commit is ever attempted,
  for either kind of record.

  When a queue-submission's commit cannot proceed because the working copy already has an
  unfinished merge in progress, the filer is told exactly that — the record is still appended,
  but the response names the reason the commit did not happen, distinct from the ordinary
  no-commit response every self-observation and every non-version-controlled filing already gets.

## Data Dictionary

A digest carries a schema version, the moment it was generated, the repository's label, counts, a
list of **dropped** records, and a list of **entries**.

### Entry — exactly six fields, and no others

| Field | Meaning |
|---|---|
| `kind` | What sort of record this is, from a closed vocabulary (below) |
| `layer` | Which layer of the workflow the record was attributed to, when the author bothered to say. Optional; frequently absent |
| `source` | Where the record came from: a work-item identifier, or a workflow-owned document path. Never project content |
| `title` | The short human-written label of the record |
| `first_seen` | When the record was first written |
| `pain` | How much this record hurt, as an integer (below) |

**There is no free-text field.** No description, no detail, no narrative, no reproduction steps.

This is the single most important rule in the area, and it was learned rather than designed. The
original intent was to carry the record's descriptive prose and strip code out of it. Measurement
of real repositories showed that such prose is ordinary sentences that happen to name functions,
files, and configuration keys — nothing that any code-stripping or secret-detection rule can find.
The surface was therefore **removed instead of filtered**, because a filter that cannot be trusted
is worse than no field at all: it advertises a guarantee it cannot keep.

### `kind` — the closed vocabulary

Repositories name their record types freely, and they diverge: one real repository invented eleven
type names and did not use the workflow's own canonical name for its most numerous class. So a raw
type is **translated** into a canonical kind. Canonical kinds and their meanings:

| Kind | Meaning |
|---|---|
| `friction` | Something repeatedly hurt while doing the work |
| `finding` | A defect or risk raised during review |
| `debt` | Known shortcut that will cost later |
| `audit` | A record produced by a housekeeping sweep |
| `deviation` | Execution departed from the agreed plan |
| `blocked` | A work item could not proceed |
| `learning` | A lesson written at the end of a feature |
| `proposal` | Something suggested but not yet decided |
| `outcome` | The recorded result of an earlier proposal |
| `approval` | A decision to proceed was granted |
| `correction` | Scope or direction was corrected mid-flight |
| `closed` | A record was retired |
| `harness-issue` | A defect in the workflow's own tooling |

A type that translates to none of these is **not silently discarded**. It is recorded as a drop with
the reason "unrecognized type" and counted, so an unknown vocabulary is visible rather than
invisible.

### `pain` — an integer, computed once, never judged later

| Source of the record | `pain` |
|---|---|
| A review finding marked highest severity | 3 |
| … marked middle severity | 2 |
| … marked lowest severity | 1 |
| A lessons record marked high / medium / low importance | 3 / 2 / 1 |
| Everything else, including all plain friction | **1** |

`pain` is computed when the digest is written, not when it is read. If a reader judged pain, two
readings of the same digest could rank differently, and the ranking would stop being reproducible.

In practice `pain` is 1 for the overwhelming majority of records, because plain friction carries no
severity anywhere. This is honest and known: the field exists so that ranking is *possible and
deterministic*, not because it currently discriminates well.

### `dropped` — a list, not a number

Each dropped record carries the same identifying fields as an entry, plus a **reason**, drawn from:
`secret`, `injection`, `oversize`, `unrecognized type`.

It carries the reason **category only** — never the text that matched. A bare count would not
distinguish one careless author from a repository systematically probing the reader every time it
closes a feature.

Records filed through the validating intake (Entry Points, above) cannot reach `unrecognized type`
— the intake refuses the record before it is stored. Drops of that reason therefore indicate
records written by some other hand (historical rows, or a writer bypassing the intake), which is
itself a signal worth reading.

## Business Rules

- **R2** (D2) — A digest carries the six allowed fields and nothing else. No free-text field exists.
  A field that is not allowed is removed rather than sanitized.
- **R7** — The reader's checks are never weaker than the writer's. Both construct records through one
  shared path, so a rule added on one side cannot be forgotten on the other.
- **R8** — **Every allowed field owns a declared validator.** The set of allowed fields is derived
  from the set of validators, so a field can never exist without one. This is the rule that closed a
  defect which had already survived two narrower fixes: each fix protected the fields someone
  remembered, and the next unremembered field was the next hole.
- **R9** — Translating a record's type is idempotent: translating an already-translated type returns
  it unchanged. Without this the reader rejects exactly the vocabulary the writer emits.
- **R10** (D 9157d074; updated D1 backlog-auto-commit) — A queue-submission's successful filing
  never leaves the raw-records file as an unrecorded change in a version-controlled working copy:
  the commit it produces touches that file alone, regardless of what other changes happen to be
  in progress in the same working copy at the time. A self-observation's successful filing is
  never committed on its own authority — only appended. A failure to record history is never
  surfaced as a filing failure — the record is already correctly filed either way.
- **R16** (D2 backlog-auto-commit) — When a queue-submission's commit cannot proceed because the
  working copy has an unfinished merge in progress, that specific reason is reported back to the
  filer instead of looking identical to an ordinary no-commit response. Every other reason a
  commit does not happen (no version control, an unexpected underlying failure) stays unreported
  by design — only the merge-in-progress case is distinguished today.

## Edge Cases Settled

- **E4** — A record whose text names an internal function, file, or configuration key is not a
  problem, because no such text is ever collected.

## Open Gaps

- **`oversize` is a declared drop reason that nothing can currently produce**: over-long titles are
  shortened rather than dropped.

## Pointers (implementation)

- **P8** — Tests: `packages/bee/tests/test_lib.mjs` (124 assertions, incl. a
  table-driven payload sweep over every allowed field, the ranking matrix, and a control-byte sweep
  over vendored sources)
- **P9** — Filing + scoped, gated commit: `packages/bee/bee.mjs` `handleBacklogAdd` /
  `commitBacklogRow` (mirrored to `.bee/bin/bee.mjs` and the plugin/skill trees via
  `skills/bee-hive/scripts/onboard_bee.mjs --apply`); the queue-submission/self-observation
  distinction is the `--queue-submit` flag (default false); the merge-in-progress check resolves
  the real git-dir (`git rev-parse --git-dir`) and looks for `MERGE_HEAD` there before attempting
  the commit. `lib/command-registry.mjs`'s `backlog.add` entry documents the flag. Example
  coverage in `packages/bee/tests/test_bee_cli.mjs` (`backlog.add` example, run
  against a directory with no version control to prove the silent-skip path) and the real-git
  fixture tests in `packages/bee/tests/test_cli_cells.mjs` (queue-submit
  omitted/true, and the merge-in-progress skip).
