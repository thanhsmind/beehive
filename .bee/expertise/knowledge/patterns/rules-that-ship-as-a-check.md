# Rules that ship as a check

## Context

When a knowledge entry states a rule that a machine could enforce — a
sanctioned way to compute a number, a shape a file must have, a command
that decides whether something is done. Prose asks every future reader to
remember and comply; a check decides. This pattern is how a rule stops
being advice and becomes something a run can be measured against.

## Mechanism

Give the rule its **own entry**, and put four things in it:

- **The runtime** — what kind of thing this is, because that is what makes
  the rest interpretable. The same parameter list means different things in
  a query, a build tool, and a script.
- **The rule itself** — the exact computation, command, or shape, with its
  named holes declared: what may be filled in, of what type, and which are
  required.
- **The receipt** — the fields a run must hand back as evidence. Not "it
  passed": the identifiers that let someone re-read what actually ran, and
  the output it actually produced.
- **The checker** — a deterministic thing that takes a receipt and returns
  a verdict. Deterministic matters: a checker that reasons is a second
  opinion, not a check.

A consumer then discovers the entry, binds the holes, runs it, gets a
receipt, and runs the checker over the receipt. **A failing verdict gates:
the result is refused, not annotated.** A check whose failure can be
displayed alongside the answer is a warning, and warnings are read as noise
within a week.

The receipt is the load-bearing piece and the one most often skipped. Its
job is to make two different lies detectable:

- **Provenance** — that what ran is the sanctioned rule bound with the
  claimed inputs, not something improvised that happens to return a number.
- **Fidelity** — that the value being reported is the value the run
  produced, re-read from the run's own record rather than copied out of
  whatever the caller wrote down.

Without a receipt neither is checkable, and "I ran the check" becomes an
assertion by the same party the check exists to constrain.

## Notes

**Confirming the rule and confirming a run are different acts, and both are
needed.** Confirming the rule asks "does this definition still match what
we intend?" — slow, occasional, recorded in the entry itself. Confirming a
run asks "did this particular execution follow it?" — per call, at runtime,
and not stored in the layer. A rule can be freshly confirmed and still be
applied wrongly on the next run; a run can execute a stale definition
perfectly. Neither substitutes for the other.

**One rule, one entry.** Trust state, expiry, and confirmation apply to a
single rule. Bundling three rules into one entry means their confirmations
and expiries collapse into one, and the first one to go stale drags the
other two with it.

**Prefer a check to prose wherever the rule can be mechanized**, but do not
force it. Judgment, taste, and intent do not have receipts, and dressing
them as checks produces a check that is either unfalsifiable or wrong. The
test is whether a specific, deterministic command decides the question — if
naming that command requires hedging, the rule is prose, and prose is the
honest form for it.
