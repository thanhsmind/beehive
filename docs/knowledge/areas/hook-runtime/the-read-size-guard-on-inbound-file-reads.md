---
type: bee.area
title: Hook Runtime — the read-size guard on inbound file reads
description: "Why an unbounded read of a large file is refused rather than warned about, which two escapes the refusal names, how the threshold is set and overridden, and why every measurement failure allows instead of denying."
tags: [hook-runtime, guards, context-budget]
timestamp: 2026-07-23
bee:
  id: hook-runtime-read-size-guard
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/write-guard-request-shapes.md]
  decisions: [router-cost D1, router-cost D2, router-cost D3, router-cost D4, router-cost D10]
  sources: [docs/history/router-cost/CONTEXT.md, "docs/history/router-cost/ (cell rc-1, capped)"]
  authoritative_for: "hook-runtime: the read-size guard on inbound file reads"
---

## Purpose

An assistant that reads a large file whole spends the scarce resource — its own working context — on
bytes a delegated helper could have read for it and returned as a digest. The guidance to delegate
such reads already exists as prose, and prose alone did not hold: an assistant delegated a search
correctly and then, in the same turn, read a 1601-line file inline anyway.

This concept is the mechanical backstop. It refuses an unbounded read of a large file at the moment
the read is attempted, and the refusal names how to get what the reader actually wanted.

## Entry Points & Triggers

The guard runs at the same pre-action checkpoint that already inspects file reads for secret-shaped
paths and excluded directories. It is not a separate checkpoint and adds no new one — the existing
checkpoint already sees every read request.

It engages only when **all four** hold:

1. the request is a file read
2. the target resolves to a regular file inside the current work tree
3. the file's line count is at or above the threshold
4. the request carries **neither** a starting offset **nor** a line limit

Any one of the four missing means the guard is silent.

## Data Dictionary

| Element | Meaning |
|---|---|
| **threshold** | The line count at or above which an unbounded read is refused. Configurable per repository; when the setting is absent the built-in default applies. An absent setting means "use the default", never "disabled" — disabling is a separate, explicit act. |
| **default threshold** | 800 lines. Derived by measurement over 2092 tracked files: 800 trips on 7.3% of them, against 5.4% at 1000 and 11.0% at 500. Chosen to cover the files that dominate a working context while leaving roughly thirteen reads in fourteen untouched. |
| **offset** | A starting line for the read. Its presence signals the reader wants a slice, and the guard stands down. |
| **limit** | A maximum number of lines to read. Same signal, same effect. |
| **unbounded read** | A read carrying neither offset nor limit — a request for the whole file, however large it turns out to be. |

## Behaviors & Operations

**Refusing an unbounded large read.** When all four trigger conditions hold, the request is refused
before it executes. The refusal states the file, its measured line count, the threshold in force, and
both escape routes. The reader observes a refusal carrying the fix, not a bare denial; the file is
not read and nothing is modified.

**Allowing a bounded read, at any size.** A read carrying an offset or a limit is always allowed,
however large the file. Reading a slice is the correct way to read a large file and must stay
frictionless — the guard exists to redirect unbounded reads, never to make large files hard to work
with.

**Allowing whenever measurement fails.** If the target cannot be measured — it does not exist, it is
a directory, it is too large to sample, it appears to be binary, or any read or inspection error
occurs — the request is **allowed**. A guard that refuses because it could not measure is worse than
no guard: it converts an unrelated failure into a blocked action. Every uncertain path resolves to
allow.

**Standing down with the surrounding guard.** The read-size check is part of the existing file-guard
family and is disabled together with it by the same per-repository toggle. There is no separate
switch, because there is no case where a repository wants the other file guards but not this one.

## Actors & Access

| Actor | Observes |
|---|---|
| the assistant issuing the read | either the file contents, or a refusal naming the file, its size, the threshold, and the two escapes |
| the repository owner | control over the threshold, and the ability to disable the whole file-guard family |
| a delegated helper | nothing — the guard never inspects who is reading, only how |

## Business Rules

- **R1.** An unbounded read of a file at or above the threshold is refused (router-cost D1).
- **R2.** The refusal is a refusal, not a warning. The checkpoint this guard runs at has exactly two
  outcomes that reach the reader: refuse, or allow. It has no "proceed with a caution attached" —
  that form is reserved for end-of-turn checkpoints, where refusing would wrongly extend the turn.
  A soft nudge was considered and rejected on evidence: the prose guidance this replaces *was* the
  soft nudge, and it failed (router-cost D1).
- **R3.** A read carrying an offset or a limit is never refused, at any size (router-cost D4).
- **R4.** Every measurement failure allows. No error path refuses (router-cost D1).
- **R5.** The threshold is a repository setting with a built-in default; an absent setting means the
  default, not disabled (router-cost D2).
- **R6.** The refusal names both escapes — read a slice, or delegate the whole file to a helper that
  returns a digest. A refusal that does not teach the fix is incomplete (router-cost D4).

## Edge Cases Settled

- **A file exactly at the threshold** is refused: the comparison is "at or above", not "above".
- **A binary file** is allowed. Line counting is meaningless there, and the guard declines to guess.
- **A very large file** is allowed rather than refused, because it is not sampled at all — measuring
  it would itself be the cost the guard exists to avoid.
- **A path outside the work tree** is not this guard's concern; containment is decided earlier by the
  surrounding guard family.

## Open Gaps

- **The threshold is calibrated on one repository.** 800 comes from this repository's file-size
  distribution. A repository whose distribution differs may want a different number, which is
  precisely why the value is a setting rather than a constant. No second distribution has been
  measured.
- **The value is bounded above by the host's own truncation.** The assistant harness already caps a
  very large read at roughly 25000 tokens and returns a partial result with an instruction to
  paginate. So this guard's contribution is not protection at the top end — the host already provides
  that. It is (a) the band between the threshold and the host's cap, where a file is large enough to
  dominate a turn but small enough to be returned whole, and (b) converting a *silent partial read*
  into an explicit refusal that names the delegated route. A truncated read leaves the reader holding
  an incomplete file that looks complete enough to reason from, which is a worse failure than a
  refusal (router-cost D10).
- **The guard has not been observed firing in live use.** The session that authored it could not
  exercise it: the executing copy of the guard is resolved from the main checkout, which does not
  carry a change made in a linked work tree until it merges. Every claim about live behaviour rests
  on the test suite. A later session must not read "I did not see it refuse me" as evidence that it
  does not work (router-cost D10).

## Pointers (implementation)

- `packages/bee/hooks/bee-write-guard.mjs` — the `READ_TOOLS` branch carries the size check; `packages/bee/hooks/adapter.mjs`
  defines the verdict encoding and the advisory-event set that R2 refers to.
- `hooks/test_write_guard.mjs` — the eight rows covering refusal, both escapes, the under-threshold
  case, every error path, the toggle, and a custom threshold.
- `.bee/config.json` — `guards.max_read_lines` (threshold) and `hooks.write-guard` (family toggle).
- The pre-action checkpoint's matcher already routes file reads into this guard, so no catalog entry
  and no regeneration of the rendered hook projections was required (router-cost D3).
