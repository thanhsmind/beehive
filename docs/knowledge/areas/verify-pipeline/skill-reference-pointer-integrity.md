---
type: bee.area
title: Verify Pipeline — skill reference pointer integrity
description: "The gate that proves every pointer an instruction document makes to a reference document still resolves, why it checks named sections and not just files, why a citation naming several headings points at every one of them, and why its negative controls are the part that matters."
tags: [verify-pipeline, guards, instruction-surfaces]
timestamp: 2026-07-29
bee:
  id: verify-pipeline-skill-pointer-integrity
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/suite-topology-and-discovery.md, areas/doctrine-layer/overview.md]
  decisions: [router-cost D5, router-cost D8, tick-contract-inline T4 (a parenthetical carrying more than one quoted heading is a live pointer to every heading it names)]
  sources: [docs/history/router-cost/CONTEXT.md, "docs/history/router-cost/ (cell rc-2, capped)", "tick-contract-inline (cells tci-1/tci-2/tci-3, decisions T1-T7, traces .bee/cells/tci-{1,2,3}.json, reports docs/history/tick-contract-inline/reports/, 2026-07-29)"]
  authoritative_for: "verify-pipeline: skill reference pointer integrity"
---

## Purpose

Instruction documents defer. Rather than restate a long contract, one says "the full rule lives in
this reference, under that heading" — and a reader follows the trail. Nothing tests prose, so when a
reference is renamed, moved, or restructured, the pointer keeps reading as though it works. The
reader is sent somewhere that does not exist, and the rule effectively vanishes while every
automated check stays green.

This gate makes that class of rot mechanically detectable. It exists specifically so that moving
content out of an instruction document into a reference is a safe operation rather than a hopeful
one.

## Entry Points & Triggers

Runs as part of the standard verification chain. It is picked up by the chain's naming convention
alone — a check placed in the conventional location with the conventional name is discovered
automatically, with no registration step and no hand-maintained list of checks to update.

Two modes:

| Mode | Runs against | Used for |
|---|---|---|
| **live** (default) | the repository's real instruction documents | the chain; proves the tree is currently honest |
| **self-test** | synthetic fixtures built in a scratch location | proves the check itself can still detect breakage |

## Data Dictionary

| Element | Meaning |
|---|---|
| **pointer** | A citation inside an instruction document naming a reference document, written in the repository's quoted-path convention. Two forms are recognised: bare (resolved relative to the citing document's own directory) and qualified (naming the owning instruction set explicitly). |
| **named section** | A pointer that also names a heading inside the target — "the reference, under *that* heading". Three phrasings are in real use and all three are recognised. One citation may name **more than one** heading at once; when it does, it is a live pointer to every heading it names, not only to the first. |
| **source document** | An instruction document that authors write. Only these are scanned. |
| **rendered projection** | A byte-copy of a source document, produced mechanically for a delivery target. Never scanned. |
| **finding** | One broken pointer, reported with its file, its line, and the offending line quoted. |

## Behaviors & Operations

**Scanning.** Every source instruction document is read and its pointers extracted. Bare pointers
resolve against the citing document's own directory; qualified pointers resolve against the named
instruction set. Rendered projections are excluded, so one broken pointer in one source yields
exactly one finding rather than one per delivery target.

**Asserting existence.** Each pointer's target must exist. A pointer to a file that is not there is
a finding.

**Asserting the named section.** When a pointer names a heading, that heading must exist in the
target. A pointer to a real file that names a section which was renamed away is still a broken
promise to the reader, and is reported as one.

**Recognising a citation that names several headings.** A citation naming more than one heading in
one breath is one pointer per heading named. Which position a heading occupies does not matter,
and neither does a line break falling inside the citation — a reader following any one of those
headings is following a promise the document made, so each of them is checked. What does not count
is a heading quoted outside any citation: a mention is not a pointer, and reachability is the bar.

**Reporting.** Each finding names the file, the line, and quotes the offending line, so the fix
needs no search. The check reports how many pointers it examined, not only how many broke — a count
of zero broken is only meaningful next to a count of how many were looked at.

**Self-testing.** In self-test mode the check runs against synthetic fixtures, including fixtures
that are deliberately broken, and asserts that the breakage is **detected**. These negative controls
run on every verification, not once at authoring time.

## Actors & Access

| Actor | Observes |
|---|---|
| an author moving content into a reference | a green check if every new pointer resolves; otherwise a finding naming the exact line to fix |
| a reader following a pointer | a trail that leads somewhere, because a broken one cannot reach them |
| the verification chain | one more check, discovered by convention, requiring no wiring |

## Business Rules

- **R1.** Every pointer from an instruction document to a reference document must resolve
  (router-cost D5).
- **R2.** A pointer that names a section is checked against that section, not merely against the
  file. A correct file with a vanished heading is still a broken promise.
- **R3.** Only source documents are scanned. Rendered projections are excluded by construction, so
  findings are not multiplied by the number of delivery targets.
- **R4.** **The gate ships with negative controls, and they are not optional.** A check that has only
  ever been observed passing is not known to work. Both failure modes — a missing target file and a
  missing named section — are proven detectable on every run.
- **R5.** A real finding is fixed, never tuned away. Widening the scan, loosening the pattern, or
  adding an exclusion to turn a red green defeats the gate's whole purpose.
- **R6.** This gate is a prerequisite for any work that moves prose out of an instruction document
  into a reference. Ordering is the point: cutting first and guarding afterwards is cutting without a
  net (router-cost D5).
- **R7.** A citation naming more than one heading points at every heading it names. Reading only
  the first — or declining to read the citation at all because it names more than one — makes a
  reachable rule look unreachable, and leaves the headings it skipped unguarded
  (tick-contract-inline T4).

## Edge Cases Settled

- **A citation inside a fenced code block or a historical note** is prose about a path, not a live
  promise to the reader. The repository's convention is that a live pointer is quoted; a retired name
  is written unquoted so it reads as history.
- **A bare pointer in a document whose own directory has no such reference** is a finding, not a
  cue to search elsewhere. Resolution is deterministic; guessing would hide the error.
- **A pointer whose target exists in a rendered projection but not in the source** is a finding: the
  source is the truth, and the projection is downstream of it.

- **A finding that cannot fail a build is a finding that gets stepped over.** The advisory
  sibling's *reachability* check misread a two-heading citation for as long as that citation had
  carried two headings, and
  reported a rule as unreachable the entire time. Because that check cannot turn a build red, its
  warning was read twice in one session by two different workers, recorded both times as
  pre-existing and unrelated, and stepped past. The misreport was fixed at its cause; the class of
  failure it had failed to stop was closed separately, by a check that does turn the build red. An
  advisory finding is worth roughly what it can cost the person who ignores it
  (tick-contract-inline T4).

## Edge Cases Settled — what the gate found on first contact

The gate was run against the repository before anything was cut, and it was not clean. Three real
pointers were already broken, in documents that had passed every check for their whole existence:

- two citations in one instruction document used the bare form for a reference belonging to a
  different instruction set, so they resolved against a directory with no such file
- one citation named a reference under a name it had carried before an early rename

All three were sending readers to files that did not exist. Nothing detected this before, because
nothing had ever looked.

## Open Gaps

- **Only reference-document pointers are checked.** Citations to source files, history documents, or
  external URLs are out of scope. Whether they deserve the same treatment is undecided.
- **Heading matching covers the three phrasings in current use.** A fourth phrasing introduced later
  would pass unchecked rather than fail loudly, which is the quieter of the two possible wrong
  behaviours but still wrong.
- **A pre-existing set of stale delivery copies sits outside every check.** Two of the install-target
  trees are not covered by the manifest or by any chain check, so drift there is invisible. Unrelated
  to this gate, but adjacent to it and worth naming.

- **The advisory sibling's *file-and-heading* check silently skips any citation it cannot parse
  whole.** It recognises a citation only when the parenthetical holds exactly one quoted heading
  and nothing else — a separate check from the reachability one fixed above. A
  citation naming a second heading — or carrying any other trailing item at all — is not partially
  read, it matches nothing: the whole citation is skipped, so neither the target's existence nor
  any of its headings is checked. This is strictly worse than the partial coverage it was first
  reported as, and it is confirmed by running the pattern against fixtures rather than by reading
  it. The blocking gate has a milder form of the same limit — it binds only the first heading a
  citation names, leaving later ones unchecked. Both remain open; neither was in this feature's
  scope.

## Pointers (implementation)

- `scripts/tests/test_skill_pointers.mjs` — the check, with `--selftest` carrying the negative controls.
- `scripts/run_verify.mjs` — the discovery roots that make registration unnecessary.
- `scripts/okf_instructions_fence.mjs` — the sibling instruction-surface check whose reporting style
  this one follows.
- Advisory sibling: `scripts/skill_lint.mjs`. Its reachability check carries the multi-heading
  matcher (R7) in `pointsTo()` / `parentheticals()` — paragraph-scoped so an unbalanced `(` cannot
  swallow the file, depth-tracked so a nested parenthetical does not truncate its parent, and
  whitespace-normalised so a heading wrapped across lines still matches. Its `ANCHOR_RE` (check 1)
  carries the skip-whole-citation limit recorded above. The lint always exits 0 and is not a member
  of the verify chain.
- The reachability roster inside that lint is a literal three-entry list, which is the opposite bet
  from the derived every-turn check in `areas/doctrine-layer/placement-and-anchoring.md` (B6).
- Landed by `tick-contract-inline` decision T4, cell tci-2 (trace `.bee/cells/tci-2.json`).
