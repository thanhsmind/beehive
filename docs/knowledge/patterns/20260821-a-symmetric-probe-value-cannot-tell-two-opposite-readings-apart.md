---
type: bee.pattern
title: A symmetric probe value cannot tell two opposite readings of a parameter apart
description: A share parameter was only ever exercised at 0.5, where "the share the parent keeps" and "the share the child gets" produce identical output, so the reading was assumed — and the assumption was backwards
tags: [failure, external-api, measurement, proof-discipline]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-a-symmetric-probe-value-cannot-tell-two-opposite-readings-apart
  lifecycle: active
  areas: [bee-herding]
  sources: [".bee/cells/archive/herding-split-serialize/hss-3.json", "original feature: herding-split-serialize"]
  polarity: pitfall
  critical: false
  evidence: prose
  evidence_ref: "packages/bee-rs/crates/bee/src/herding/run.rs, first_split_geometry doc comment — herdr's --ratio is the share the PARENT KEEPS, not the share the child gets"
---

# A symmetric probe value cannot tell two opposite readings of a parameter apart

A ratio, share, percentage or split point has two opposite readings — what the
caller keeps and what the callee gets. At the midpoint the two readings produce
the SAME output, so every run at that value confirms both and settles neither.

The instance: the terminal tool's `--ratio` had been used at `0.5` and only
`0.5` since the seam was written, and the reading was never documented. A
decision then asked for a worker column of one third. Under the assumed reading
the human's own pane would have been left the third and the worker given two —
the exact inversion of what was decided, shipping green.

## The rule

- Before depending on a directional parameter, probe it at a value where the
  two readings disagree, and record the measured pair.
- Treat a parameter whose only production value is its symmetric one as
  UNMEASURED, however long it has shipped.
- Record the reading beside the code that computes it, in the units the
  contract cares about — the width, not the wire encoding.

Measured live against herdr 0.8.0: `--ratio 0.25` on a 173-column pane left the
parent 43 columns and handed the child 130. The share is what the PARENT keeps.
