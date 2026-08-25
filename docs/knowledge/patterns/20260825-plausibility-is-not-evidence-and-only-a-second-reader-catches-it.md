---
type: bee.pattern
title: Plausibility is not evidence, and the author is never the one who catches it
description: Plausibility is not evidence, and the author is never the one who catches it
tags: [failure, diagnosis, attribution, review, concurrency]
timestamp: 2026-08-25
bee:
  id: pattern-20260825-plausibility-is-not-evidence
  lifecycle: active
  areas: [doctrine-layer, workflow-state]
  sources: ["claim-reserves-files / exclusive-create-atomic / state-lock-lost-update, 2026-08-25 — four dead diagnoses across two sessions in one day", "peer session on wt/model-role-split, which killed two of them and had one of its own killed"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "count the verification step: an attribution naming a specific commit, session, or mechanism should have ONE check between the conclusion and the sentence — if the check happened only after someone objected, the claim was plausibility"
---

# Plausibility is not evidence, and the author is never the one who catches it

Four diagnoses died in one day across two sessions working the same repository.
Every one of them was reasonable when written. Every one was killed by **the
other party re-running the check** — none by the person who made it.

| the claim | why it was believed | what killed it |
|---|---|---|
| "commit X broke the claim race" | one green run at `X~1`, one red at `X` | re-running the parent **five** times: 4/5 red. The race predated X. |
| "load hides this race" | three green full-suite runs | a full-suite run that failed — already sitting in the author's own logs when the claim was filed |
| "those untracked files are that session's" | the file count grew between two merge attempts, and that session was the one being talked to | `bee discovery list` — a different open map entirely, owned by neither |
| "take the lock before the authoritative read" | it was the exact fix that had worked on a sibling bug an hour earlier | reading the function: it **already** locks before reading |

Note the third and fourth rows. One session misattributed the other's files;
the other session pattern-matched its own fresh fix onto a symptom that rhymed.
Neither was careless. Both were *pattern-completing from a strong prior* — and
a strong prior is exactly what makes the check feel unnecessary.

**The shape is always the same:** the conclusion arrives first, and the
supporting reasoning is assembled behind it. That assembly is genuinely good
reasoning, which is why it convinces the author. What it cannot do is
distinguish a true conclusion from a plausible one, because it was built to
support the conclusion either way.

**Why the author never catches it.** By the time the sentence is written, the
author has the conclusion, the reasoning, and no remaining doubt to spend. A
second reader has the same data and *no attachment to the answer*. In four for
four, that asymmetry was the entire detector. Not seniority, not care, not
slowing down — someone else with the same evidence and nothing invested.

**The rule:** an attribution that names a specific commit, a specific session,
or a specific mechanism is a claim about someone else's work. It earns exactly
one verification step **between the conclusion and the sentence** — never after
the objection. If the check happened only because someone pushed back, what was
filed was plausibility.

## The limit of a second reader: a shared error is invisible to review

A fifth diagnosis died the same day, and it broke this pattern's own remedy.

One session reproduced the state-lock lost update and observed that the entry
lost was **`w0`, the first racer**. The data was sound — that run really did
lose the first entry, and the file really was well-formed. What happened next
was that `n=1` became a law: "the FIRST entry dies". It was then *refined*
("`w0` dies because it is the only entry already complete when a second holder
enters"), which made it more persuasive without making it better supported. The
other session wrote it into a feature's locked context as an established fact.

Instrumentation later caught a real failing run. **The lost racer was `w5`.**
The victim is whoever won the takeover and was clobbered by the plain acquirer
that walked into its rename vacancy; position was a coincidence of one sample.

The four earlier errors were each caught by the other party re-running the
check. **This one could not be** — both parties had made the *same*
generalisation from the *same* single observation, and cross-checking cannot
detect an error that both readers share. What killed it was a **second
sample**.

So the remedy has two halves, and only one of them is review:

- Errors made **separately** are caught by a second reader with the same data
  and no attachment to the conclusion.
- Errors made **together** — a shared prior, an agreed framing, one observation
  promoted to a rule — are caught only by **new data**. No amount of careful
  re-reading reaches them, because the reading is the thing that agrees.

The tell for the second kind: a claim everyone accepts that rests on **one
run**. Ask how many samples are under it. "We both think so" is not a second
sample.

**A corollary about elimination.** "Eliminated" and "eliminated on the route I
checked" are different claims, and only the second is ever earned. In the same
investigation, an empty-lock hypothesis was correctly eliminated via the
staleness path — `mtime` age is checked before the holder is read — and then
turned out to be reachable through a door nobody had enumerated, a post-rename
read. Record which route was closed, not that the question is.

**The cheap version, for intermittent symptoms:** one run per side is not a
bisect. At a 1-in-10 rate a clean 20 is roughly a coin flip. Repetition counts
are not pedantry; they are the difference between measuring and guessing.

**And when a good framing arrives from a reviewer mid-investigation, hold it
back.** A plausible hypothesis handed to an investigator becomes the place they
look first. If the investigation lands there independently, that is
corroboration worth far more than the hint. If it lands elsewhere, the hint was
a thumb on the scale. Record it to evaluate the finding *against*, not to seed
it with.
