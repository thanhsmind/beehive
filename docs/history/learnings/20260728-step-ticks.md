---
date: 2026-07-28
feature: step-ticks
categories: [orchestration, communication]
severity: medium
tags: [visibility, ticks, bypass, ak-parity]
---

# step-ticks — feature close learnings

## What Happened

Bee's last invisible habit closed. The old Progress-ticks text was thin prose
and, under gate bypass, the pipeline went quiet from Gate 1 to feature close —
perceived latency was the whole run even when each unit finished in minutes.
Now the section is a contract (R84): every perceivable step emits exactly one
short line, on by default, in the user's work language; one fixed shape
(state glyph + event + key fact); one merged catalog of 20 events with worked
examples, absorbing the old cap-seam/slice/re-lane/PR rows so no second list
can drift.

Two silence rules were tightened, not loosened:
- **Bypass silences questions, never ticks** — the `⚡` auto-approval line is
  itself a tick, not an exception to one.
- **`ship_visibility: "off"` now silences only its own two PR lines**, where
  the old text silenced the entire stream — a real behavior change, recorded.
- A red or a refusal is never silenceable, even under an explicit quiet.

## Findings

1. **Visibility is an authoring contract, not a subsystem.** No emitter, no
   polling, nothing to build — the value came entirely from fixing the shape
   and enumerating the events so the agent cannot forget one. Cheapest
   user-visible win of the day.
2. **A silence switch scoped too wide reads as a bug.** `ship_visibility:
   "off"` meaning "no PR pushes" quietly meant "no progress lines at all";
   narrowing it to its own two lines is what the name always implied.
3. **The feature demonstrated itself while shipping** — this close was
   narrated tick-by-tick in the shape the cell was writing, which is the only
   honest acceptance test for a communication law.

## Ran under the new laws

R82 (worker ran no suites, capped pending; main verified once at the boundary
— green first pass) and R83 (one sync, one compounding, at close).
