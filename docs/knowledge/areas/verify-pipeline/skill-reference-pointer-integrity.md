---
type: bee.area
title: Verify Pipeline — skill reference pointer integrity
description: "The gate that proves every pointer an instruction document makes to a reference document still resolves, why it checks named sections and not just files, and why its negative controls are the part that matters."
tags: [verify-pipeline, guards, instruction-surfaces]
timestamp: 2026-07-23
bee:
  id: verify-pipeline-skill-pointer-integrity
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/suite-topology-and-discovery.md, areas/doctrine-layer/overview.md]
  decisions: [router-cost D5, router-cost D8]
  sources: [docs/history/router-cost/CONTEXT.md, "docs/history/router-cost/ (cell rc-2, capped)"]
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
| **named section** | A pointer that also names a heading inside the target — "the reference, under *that* heading". Three phrasings are in real use and all three are recognised. |
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

## Edge Cases Settled

- **A citation inside a fenced code block or a historical note** is prose about a path, not a live
  promise to the reader. The repository's convention is that a live pointer is quoted; a retired name
  is written unquoted so it reads as history.
- **A bare pointer in a document whose own directory has no such reference** is a finding, not a
  cue to search elsewhere. Resolution is deterministic; guessing would hide the error.
- **A pointer whose target exists in a rendered projection but not in the source** is a finding: the
  source is the truth, and the projection is downstream of it.

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

## Pointers (implementation)

- `scripts/tests/test_skill_pointers.mjs` — the check, with `--selftest` carrying the negative controls.
- `scripts/run_verify.mjs` — the discovery roots that make registration unnecessary.
- `scripts/okf_instructions_fence.mjs` — the sibling instruction-surface check whose reporting style
  this one follows.
