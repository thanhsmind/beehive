---
type: bee.pattern
title: Existence is not evidence — a plan that cannot be authored without reading is the only plan that was read
description: Existence is not evidence — checking that a thing exists is not checking what it contains or whether its path runs; the fix is structural, never exhortative
tags: [planning, gates, verification, failure, evidence]
timestamp: 2026-08-30
bee:
  id: pattern-20260830-existence-is-not-evidence
  lifecycle: active
  areas: [workflow-state, doctrine-layer]
  sources: ["waggledance field report, 2026-08-30 — 7 assertion errors across 2 features, one shape, all under green tests", "existence-is-not-evidence feature, 2026-08-30 — the fix caught its own author twice before shipping"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "R139: a shape/merged approval refuses while the plan's load-bearing claims table is missing, malformed, or carries a guessed row (verbs/state_group/plan_claims.rs)"
---

# Existence is not evidence

Seven planning assertions failed in one day across two features in a host
repo, all one shape: the agent checked that a thing EXISTS, never what it
CONTAINS or whether its path RUNS. A status enum was planned around a variant
that did not exist. A "Done" state hid a second, guessed source. A field was
displayed because the schema had it — its real value was an internal tool
name. Two "forgotten" fields had been deleted on purpose, with the reason
three lines from the read. A daemon was "fixed" from its CLI's help text; one
direct socket probe disproved it in 0.00 seconds. All seven had green tests.

**The three cheap proxies.** Reading the schema instead of the data; reading
the docs instead of the behavior; reading the structure instead of the run.
Each is cheap, each leaves a tool call in the transcript, and each therefore
FEELS like verification — the transcript pattern-matches "I checked".

**Why prose cannot fix it.** The host repo already had a
prove-the-whole-path pattern injected into every worker prompt. It prevented
none of the seven, for two stacked reasons: it addressed the wrong layer
(cells, while the errors were born at plan altitude), and — deeper — any
instruction of the form "verify when unsure" gates on the model's own
uncertainty estimate, which is miscalibrated exactly where the prior is
strong. `RunStatus::Failed` does not feel like a guess to the model that has
seen a thousand enums carry `Failed`. The moments the rule exists for are
precisely the moments it does not fire.

**The fix is structural.** Make the artifact impossible to author without
touching the tree, then verify with a different task than the one that
generated it:

1. The plan carries a load-bearing claims table — claim, label
   (`read`/`ran`/`guessed`), anchor, verbatim quote. A quote with a line
   number cannot be filled from a prior; filling the row IS the read.
2. The gate refuses mechanically while a load-bearing row is still
   `guessed` (R139) — "be diligent" (judgment) became "fill this field"
   (mechanical), and the net is a binary check, not a reminder.
3. A second reader audits the table — opens each anchor, compares bytes
   (existence-is-not-evidence D4). Generation cannot fail at generating;
   only a task whose JOB is comparison catches the author
   (pattern-20260825: the author is never the one who catches it).
4. One cheap reality touch per novel surface before the gate: open the
   real data or run the real path once, output recorded
   (existence-is-not-evidence D3). Each of the seven errors dies in
   seconds against its real object.

The carrier for all four is skill text, templates, and the binary — a
knowledge pattern like this one is additive, never the sole carrier
(existence-is-not-evidence D5): prose alone already failed in the field.

**Proof it works on its own author.** Building this very feature, the wave's
second reader caught the author's plan asserting "existing tests may need
fixture updates" (unverified — the real count, once swept: one fixture) and
"frozen plans are never re-gated" (unverified, load-bearing, no row) — and
caught the check itself being wired to a gate where it could never fire,
via a test that pins `--name shape` outside the exec guard. The claims table
the plan carried was audited row by row: 7 match, 1 partial mismatch (a
quote had silently dropped its `PATH=` prefix). Existence-is-not-evidence
failures appeared WHILE building the existence-is-not-evidence gate; the
structure caught them, prose would not have.

**The residuals, named.** A claim omitted from both table and prose is
caught by nothing mechanical — the membership sweep is the second reader's
job. A fabricated quote at a real path passes the binary — the audit's job.
The reality touch ships as prose — its output lands as `ran` rows, which
the table makes visible.
