---
type: bee.pattern
title: A guard deleted with its runtime is a guard removed
description: A guard deleted with its runtime is a guard removed
tags: [failure, tests, verification, cutover, migrations]
timestamp: 2026-08-25
bee:
  id: pattern-20260825-a-guard-deleted-with-its-runtime-is-a-guard-removed
  lifecycle: active
  areas: [verify-pipeline, onboarding]
  sources: ["statusline-binary-lookup cells sbl-3/sbl-4, 2026-08-25: the statusline usage segment was silently dead on every host, and the byte-equality sweep that would have caught it died in the R6 Node cutover"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/tests/statusline_contract.rs — the restored byte-identity sweep plus three fixtures that EXECUTE the vendored script; the execution half is what the identity half could never prove"
---

# A guard deleted with its runtime is a guard removed

A cutover deleted the Node tree. Inside it, one file was a test suite, and inside
that suite was the sweep enforcing a spec rule: the canonical script and its
vendored copy must be byte-identical. The tree went; the rule stayed written; the
pointer in the spec kept naming the deleted path.

For weeks, every reader who followed that pointer believed a guard existed. It did
not run. In the same commit that removed it, someone fixed a bug in the vendored
copy and not in the canonical template the vendoring engine copies FROM — so the
next `onboard --apply` on any host overwrote the fix with the broken original, in
silence, fail-open, exit 0.

The bug was one wrong path in a shell resolver. Finding it cost a day.

## The rule

A commit that deletes a test tree owes, in that same commit, either the
replacement guard or a named debt entry. "The suite was unrunnable so it went"
describes the file, not the coverage. Grep the deleted tree for what it was the
only enforcement of, before deleting it.

## The corollary, which is the sharper half

**An identity guard is not a behaviour guard.** Byte-equality, sha fingerprints,
a release manifest's `--check` — every one of them compares copies to each other,
and not one of them ever runs the thing. A vendored script can be byte-perfect
across every projection and still not work, because "identical" and "correct" are
different claims and only the first was ever being tested.

Any mechanism whose whole value is that it *runs* owes at least one test that runs
it. For a vendored script that means a fixture host, a stub on the path it should
resolve, and an assertion on real stdout — not a diff.

## Two smaller edges from the same day

- **A vendored pair has exactly ONE edit site: the canonical source.** The engine
  overwrites the copy on every apply, so a fix to the copy alone is erased and a
  fix to the canonical alone is invisible until the next apply. Fixing the copy
  feels like it worked, and that feeling is the trap.
- **A repo that ships an opt-in must dogfood it.** bee's own checkout carried no
  `statusLine` key, so its onboarding read it as not opted in and never healed the
  pair here — the one repo most likely to notice the drift was the one place the
  healing never ran.
