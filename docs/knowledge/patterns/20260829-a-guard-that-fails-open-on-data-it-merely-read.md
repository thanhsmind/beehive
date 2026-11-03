---
type: bee.pattern
title: A guard that fails open on data it merely read
description: "When a guard's readers can return an error, that error becomes a third verdict the guard never meant to have — and if the guard's undecidable answer is \"let it through\", one unreadable byte in a record the guard only consulted switches the whole guard off. Each reader looks correct on its own; the hole exists only at the seam between the reader's error type and the verdict channel."
tags: [guards, fail-open, error-handling, parsing, review, sweeps]
timestamp: 2026-08-29
bee:
  id: pattern-20260829-a-guard-that-fails-open-on-data-it-merely-read
  lifecycle: active
  areas: [hook-runtime]
  sources: ["slp-followup-gaps cell sfg-3 (commit 113093a1, 2026-08-29 — the claim readers)", "slp-followup-gaps cell sfg-4 (commit b98f03ab, 2026-08-29 — the heartbeat, the resolved context, the config read)", "slp-followup-gaps cell sfg-5 (commit 77fbdfd5, 2026-08-29 — leases, holds, the strict session read, the silent-lockout warning)", "slp-followup-gaps cell sfg-6 (commits d502e845 and 85ead065, 2026-08-29 — the companion marker, and the header's own delegate list)"]
  polarity: pitfall
  critical: true
  evidence: exercised
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs — the sfg3_*/sfg4_*/sfg5_*/sfg6_* cases; each one was red first with the same failure text, 'expected a native verdict, got Delegate', and each named a single malformed field in a record the guard only consulted"
---

# A guard that fails open on data it merely read

A guard answers YES or NO. Almost every real guard has a third answer as well
— *undecidable*, *unknown*, *not my case* — and that third answer is wired to
one of the first two. Where it is wired to "let it through", the guard has a
hidden off switch, and the switch is any error its own readers can raise.

The readers are the problem, because a guard reads far more than it decides.
It consults records to learn who else is live, which unit of work this actor
holds, where the store lives, whether a path is already held. None of that
data is the subject of the verdict; all of it is input. So when one of those
reads answers `Err`, the error is about a *byte*, and it arrives on the
channel that means *the guard cannot decide* — and the guard turns off. Not
for that field. Not for that check. For the entire call: every check, every
path, every target.

## The smell

**An error type produced by a READ reaches the guard's own verdict channel.**
In practice it looks like this, and it is three characters long:

    let stamp = parse_timestamp(record.get("claimed_at"))?;

The `?` is the whole defect. The function it sits in has a `Result` return
type because a dozen other things in it can genuinely fail, so the propagation
compiles, reads naturally, and matches every line around it. Nothing at that
site says "this error will be spelled *do not run the guard*, twelve frames
up".

Read the smell backwards to find it: start at whatever the guard does when it
cannot decide, and if that outcome is permissive, every `?`, `throw`,
`unwrap_or_else(bail)` on the way down to it is a live off switch waiting for
malformed data.

## Why it is so easy to miss

Each reader is locally correct. A parser that refuses a value it cannot parse
is doing its job; a store reader that reports an unreadable file is doing its
job. Reviewed alone, each one is right — and every one of them was reviewed
alone, at the time it was written, against the question "does this read the
record correctly?"

The defect does not live in any of those functions. It lives at the **seam**:
the point where an error meaning *I could not read this* is spent as an
answer meaning *I have no verdict*. No single file shows both ends of that
seam, so no single-file review can see it. This is why the fault survives
careful code and careful reviewers.

Two more things hide it:

- **The blast radius is invisible from the site.** One malformed field in one
  auxiliary record does not disable one check; it disables the whole guard.
  Nothing about `?` on a timestamp suggests that scale.
- **The failure is silent and green.** Fail-open exits zero. The write lands,
  the command runs, no one is refused, nothing is logged. There is no incident
  to investigate — which is exactly why this class is found by reading, not by
  operating.

## Why the tests do not catch it

Because fixtures are written by the same hand that writes the reader, and that
hand writes *readable* data. Every fixture holds a well-formed timestamp, a
parsable record, a resolvable root — the suite exercises the guard's YES and
its NO exhaustively and never once produces the input that reaches the third
answer.

The fix is a test axis, not a test: for every record the guard reads, one case
per unreadable SHAPE, asserting a real verdict came back. The shapes are not
one thing. For a timestamp field they are at least: a string in the wrong
format, a numeric epoch, an object, a boolean — a reader that answers `None`
for *absent* answers `Err` for all of those, and "absent" is the only one
anybody thinks to test. Assert on the verdict, never on the outcome: a
fail-open guard returns the same exit code as a guard that allowed the write
on purpose, so a test that checks only the exit code passes in both worlds. It
has to assert that a verdict was *reached*.

## How to check for it

1. Name the guard's undecidable answer, and name which of YES/NO it resolves
   to. If it resolves to the permissive one, every step below is mandatory.
2. Enumerate every read the guard performs — store records, config, the
   filesystem, the environment. Reads, not decisions: the ones nobody thinks
   of as part of the verdict are exactly the ones this defect lives in.
3. For each one, walk the error path by hand, frame by frame, to the verdict
   channel. Reachability is the finding. "This reader can error and that error
   arrives here" is the whole bug; no reproduction is needed.
4. For each reachable one, ask the only question that settles the answer:
   **what is an unreadable byte EVIDENCE of?** It is evidence about the byte.
   It is not evidence that a session died, that a hold lapsed, that a claim
   expired, that no marker exists. Any answer that reads "unreadable" as
   *absent* is inventing a fact.

## The two honest answers

Step 4 has exactly two outcomes, and which one applies is decided by what the
guard would DO with the fallback:

- **Where the fallback claims nothing the guard spends** — take it, and warn
  in your own words. Reading a claim the parser cannot date as still active,
  or a hold whose expiry will not parse as still held, only ever adds teeth:
  the restrictive reading of an unreadable record costs at worst one refusal,
  which a human clears by repairing the file the refusal names.
- **Where the fallback would be a positive claim the reader cannot back** —
  refuse, and name the file. "No live peer", "no verified mount", "no owner"
  are not neutral defaults; each is an assertion, and each is the assertion
  that opens the door. Granting it off bytes nobody could parse is the same
  fail-open one layer down, spelled with a different word.

Both answers have a cost the fail-open never had: a restrictive read of a
record that will never repair itself can lock someone out indefinitely. Pay
that cost, do not undo it — but pay it out loud. The refusal, or a warning
beside it, must name the file it could not read and the command that clears
it, or a human is left guessing at a door with no sign on it.

## The second-order lesson, which is the sharper half

This class was closed in four rounds, and rounds two, three and four each
began with the previous round's author having declared the sweep complete.

- Round one fixed the readers it found and said so.
- Round two found the identical defect one call frame away, in three more
  readers, and swept "the rest of the module" — leaving one escape it had
  reasoned was unreachable.
- Round three proved that escape reachable from an ordinary write, and found
  two whole classes of record round two had not enumerated.
- Round four found the last read, in a file rounds two and three had both
  walked past. Its own re-scan then caught the module header asserting its
  list of exceptions was exhaustive while two live exceptions were missing
  from it.

Three consecutive re-scans, three consecutive "this is now complete" claims,
three consecutive holes. Not one of them was found by the author of the sweep
it followed; every one was found by an independent re-enumeration.

So: **for a defect class, a careful author's own list is not the deliverable
— an exhaustive re-enumeration by someone who did not write the fix is.** The
author's list is bounded by the same mental model that produced the defect. If
you missed a reader while writing the code, you will miss it again while
listing the readers, and you will miss it with confidence, because the list
feels complete from inside.

The mechanical form of that re-enumeration is what makes it work. Not "look
for more of these": compile the FULL set — every reader, every error-returning
call — and check each member against the rule, including the ones the previous
round already cleared. And when a comment or a header claims a list is
exhaustive, that claim is itself an assertion to be re-verified, not a
shortcut past the check.
