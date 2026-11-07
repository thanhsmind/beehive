---
type: bee.delivery
title: knowledge-one-home — delivery and learnings
description: "What the twelve capped cells shipped, the proof each was capped against, the deviations they recorded, and the learnings the run left behind."
tags: [knowledge, delivery, learnings, standard]
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-delivery
  lifecycle: active
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  required_context: [work/knowledge-one-home/work-item.md, work/knowledge-one-home/plan.md]
  decisions: [D1, D2, D3, D4, D5]
  sources: [docs/history/knowledge-one-home/CONTEXT.md, docs/history/knowledge-one-home/plan.md]
  lane: standard
---

# knowledge-one-home — Delivery and Learnings

## What shipped

Twelve cells, twelve commits, one per cell.

| Cell | Commit | What it shipped | Decisions |
|---|---|---|---|
| koh-1 | `2baaf3ea` | The four flat keys parse, emit in a fixed position, and round-trip byte-identically; the profile-required table is untouched | D4 |
| koh-2 | `9a99c60a` | Six new check codes grade ownership maps, outbound lists, rule markers, and pointers as profile errors | D4 |
| koh-3 | `ab051ce8` | Ownership maps written for all fifteen areas, including three that had no overview concept to carry one | D3, D4 |
| koh-4 | `d3cc9c7a` | The first three inventoried rules homed with ids and outbound lists — the delegation threshold, the write-guard allowlist, the cap proof line | D4 |
| koh-5 | `b305b202` | Every cell must declare its predicted affected skills and specs, in every lane | D3 |
| koh-6 | `e8a07325` | The cap-time sync door: ownership, outbound-list, and prediction checks that refuse, with one recorded escape | D3, D4 |
| koh-7 | `712fc041` | The ownership loader, and the update-obligation list printed at the moment a decision is logged | D1, D4 |
| koh-8 | `f0bbf5b1` | Conflict candidates derived at plan time onto the workflow record, one verdict per candidate out of a closed three | D5 |
| koh-9 | `3dec8983` | The merged gate refuses a lane whose conflict review is absent, stale, or unverdicted; an acknowledged conflict is named on approval | D2, D5 |
| koh-10 | `1a4a1870` | Markers and pointers are read outside code regions only; the retired "the close door runs the test command" claim removed from five help strings | D4 |
| koh-11 | `b95a6f04` | A capture stub for a skill-owning area is refused without its skill answer | D4 |
| koh-12 | `d3eaa313` | The remaining nine inventoried rules homed; every other site reduced to one line plus a pointer; the named drifts resolved by citing the home | D4 |

## Where the area truth landed

Each cell wrote its behaviour into the concept that already homes it —
no new area concept was created for this work:

| Concept | Rules or content added | Cells |
|---|---|---|
| `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md` | The four new frontmatter field rows and their placement rule | koh-1 |
| `docs/knowledge/areas/okf-profile/conformance-check.md` | The six new profile-error codes, the exempt trees, and the code-region reading rule | koh-2, koh-10 |
| `docs/knowledge/areas/okf-profile/overview.md` | The area's own ownership map | koh-3 |
| `docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md` | The prediction fields required on every cell | koh-5 |
| `docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md` | The cap-time sync door and its escape | koh-4, koh-6 |
| `docs/knowledge/areas/workflow-state/gates.md` | Plan-time conflict derivation, the verdict set, and the merged-gate precondition | koh-8, koh-9 |
| `docs/knowledge/areas/workflow-state/capture-queue-and-the-blocker-threshold.md` | The skill answer on a skill-owning area's stub | koh-11 |
| `docs/knowledge/areas/decision-memory/overview.md` | The update-obligation list on a logged decision | koh-7 |
| `docs/knowledge/areas/doctrine-layer/delegation-threshold.md` | The delegation-threshold rule home | koh-4 |
| `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` | The duplication boundary, now enforced rather than declared, and the record of the operating-block homes | koh-4, koh-12 |
| `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md` | Two stacks of amendment notes collapsed to the live rule plus one superseded-notes line | koh-12 |
| `docs/knowledge/areas/doctrine-layer/helper-classes-and-transports.md` | Pointer reduced to one line | koh-12 |
| `docs/knowledge/areas/doctrine-layer/unenforced-obedience.md` | Pointer reduced to one line | koh-12 |
| `docs/knowledge/areas/doctrine-layer/the-communication-contract.md` | Pointer reduced to one line | koh-12 |
| `docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md` | The write-guard allowlist drift resolved by citing the crate as truth | koh-4 |
| `docs/knowledge/areas/onboarding/status-display-vendoring.md` | Pointer reduced to one line | koh-12 |
| `docs/knowledge/areas/workflow-state/review-sessions.md` | Pointer reduced to one line | koh-12 |
| `docs/knowledge/areas/workflow-state/worktree-isolation.md` | Pointer reduced to one line, with the per-site exemption lists resolved | koh-12 |
| `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md` | Pointer reduced to one line | koh-12 |
| Every area overview | The ownership map for that area | koh-3 |

## Verify

Each cell below was capped only against a recorded passing result — bee
refuses a cap without one.

- **koh-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee knowledge` — green, 99 passed 0 failed; the knowledge module is the touched scope.
- **koh-2** — `cargo test --release -p bee knowledge` — green; touched the checker, router, walker, bootstrap, and tests.
- **koh-3** — `bee knowledge check`, then `bee knowledge index --check` — green, zero missing maps and zero dangling paths; a docs-only cell, so the bundle check is the proof.
- **koh-4** — `bee knowledge check`, then `bee dev release-manifest --check` — green, zero profile errors including the four new rule codes; 270 files match the manifest.
- **koh-5** — `cargo test --release -p bee cells`, `--test registry_contracts`, `bee dev release-manifest --check` — green; cell validation is the touched scope, and the registry and manifest pin the help text and skill tree.
- **koh-6** — `cargo test --release -p bee cells` — green, 241 cells tests including 8 new door tests, plus the registry and catalog suites; the full crate run was 2361 passed with only a pre-existing unrelated failure also present on the clean tree.
- **koh-7** — `cargo test --release -p bee knowledge`, `decisions`, `--test registry_contracts` — green; the loader and the decision log are the touched scope.
- **koh-8** — `cargo test --release -p bee state_group` (162), `--test registry_contracts` (10), `--test registry_dispatch` (8), `bee knowledge check` (0 errors) — green.
- **koh-9** — `cargo test --release -p bee state_group` (172 passed, 10 new), `--test registry_contracts` (10), `bee dev release-manifest --check` (270 files) — green.
- **koh-10** — `cargo test --release -p bee knowledge` plus the four payload-consuming suites — green; the knowledge suite covers the two new extractor tests and the payload suites cover the help-text edit.
- **koh-11** — `cargo test --release -p bee capture`, `--test registry_contracts`, `bee dev release-manifest --check`, `bee knowledge check` — green.
- **koh-12** — `bee knowledge check`, then `bee dev release-manifest --check` — green; a docs-only cell, so marker and outbound-list grading plus manifest parity is the proof this change type needs.

## Deviations

Recorded on the cell traces:

- **koh-1** — the worker's commit lacked its cell trailer; the orchestrator amended it in place.
- **koh-3, koh-5, koh-7** — the worker did not run the finish step; the orchestrator capped from the worker's result form.
- **koh-4** — the worker's pane idled out after the edits and the regen chain; the orchestrator committed its tree and capped from inspection.
- **koh-6** — fixed a pre-existing red: the legacy-cell deviation line was unconditional and broke nine existing deviation tests; it now fires only when the touched set actually carries a skill path.
- **koh-8** — terms are normalized before scoring (see the learning below); the two new gate rules took the next free numbers, and the hand-edited help payload is recorded in the concept's pointers as its own decision requires.
- **koh-9** — the gate tests live in the gate file's own test module beside the advisor cases they copy, because the gate body is private to that file; the door reads the live workflow record rather than the lane record, because the lane projection never copies the review down; the pinned flag count is unchanged, since the cell adds a refusal and an output field but no flag name; the regenerated output trees were reserved and committed beside the cell files.
- **koh-10** — the payload fix was extended past the two verbs the cell named to three more that carried the same retired claim; two stale comments and one exempt-tree file were left alone; one pre-existing unknown pointer in a skill file was left for koh-12, because fixing a skill file would have pulled in the regen chain the cell forbids.
- **koh-11** — tests landed in the existing inline test module rather than a new file, because six capture tests already lived there; the new rule was homed with its own id and outbound list so the one-home discipline covers it.
- **koh-12** — the operating block's rendered file is generated from a source block, so the source was edited and the render re-run; the duplication-boundary concept carries no outbound list, because those ten rules are homed in the operating block and the key would fail the linkage check; a wrapped pointer left by koh-11 was rewritten onto one line; one marker covers a whole paragraph, because splitting its semicolon list would have reworded pinned prose.

## Learnings

**A text scanner with no idea what a code fence is cannot tell teaching
from claiming.** The first marker and pointer extractors read plain text,
so any document that quoted the literal marker spelling read as a second
home for that rule — the check would have refused the very concept that
explains the check. koh-10 closed it by stripping fenced blocks and
inline backtick spans before the scan. The general shape: a scanner that
grades a syntax must skip the regions where that syntax is quoted, or the
documentation of a rule becomes a violation of it (koh-2, koh-10).

**A pointer reference must sit on one line with balanced backticks.** The
extractor drops any id carrying a newline or a stray space, so prose that
wraps a pointer across a line break, or leaves an unbalanced backtick on
the line, silently breaks the outbound linkage — no error, just a rule
that quietly stops being pointed at. koh-11 left exactly this defect and
koh-12 repaired it (koh-11, koh-12).

**The pointer form is reserved for marker-homed ids, and nothing else.** A
worker wrote a pointer naming a spec rule number, which no marker homes,
and produced an unknown-pointer finding that survived two cells. A rule
number inside an area concept and a homed rule id are different
namespaces; the pointer form names only the second (koh-10, koh-12).

**A generator that emits concepts must emit them already conformant.** The
bootstrap path writes area overviews, so the new missing-map code would
have fired on bootstrap's own output the moment anyone ran it. The worker
saw it and added an ownership map to the generated overview — the right
call, but it was neither declared in the cell nor recorded on the trace,
so it shipped as an undeclared widening. A check added over a shape some
generator produces has to be tried against that generator in the same
cell (koh-2).

**Without term normalization, "zero conflicts" is unreachable.** The
conflict scorer counts term hits, and an unnormalized title contributes
its articles and prepositions, which hit nearly every decision in the
store. Lowercasing, trimming punctuation, dropping terms shorter than
four characters, and filtering a 24-word stop list is what makes an empty
candidate list a possible outcome — and D5 makes "0 conflicts" true only
when the derive genuinely returned nothing. A derived list that can never
be empty is not a check, it is noise (koh-8).

**A worker's claim about a bundle-wide check is not proof.** One worker
reported "knowledge check: 0 profile errors" while the tree it had just
written carried two. A test suite scoped to the worker's own files is
something the worker can honestly verify; a whole-bundle check is not,
because the worker cannot see what its neighbours left behind. The
orchestrator re-runs bundle-wide checks itself rather than trusting the
line (koh-11).

**A door placed on the cap path inherits every case the cap path already
had.** koh-6's first cut wrote its legacy-cell deviation line
unconditionally and broke nine existing deviation tests that had nothing
to do with ownership. Scoping the line to the case it actually describes —
a touched set that really carries a skill path — fixed it. A line that
says nothing about the change at hand is still a behaviour change to
everything downstream of it (koh-6).

**Cosmetic, and named rather than fixed:** on the finish path the
diff-versus-test advisory prints before the sync door can refuse, so a
refused finish shows an advisory about work that is not going to be
recorded. The ordering is harmless and was left alone (koh-6).

## Provenance

Recorded during the scribe step of `knowledge-one-home`, from the twelve
capped cell traces `koh-1` … `koh-12` and the eight capture stubs those
cells filed. Every line above is copied from a trace, a stub, or the work
item — nothing here is inferred.
