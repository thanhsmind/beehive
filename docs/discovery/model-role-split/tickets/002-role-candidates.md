---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: 001-dead-extraction-slot
---

## Question

Which new roles earn their place — meaning each has a real dispatch site
that would select it, not just a name on a config screen?

Candidates, with the site that would use each:

- **`tiny`** — a `tiny` cell may run inline on the session model
  (AGENTS.md, "From small up, cells run through dispatched workers … a
  tiny cell may run inline"). Inline means the ceiling, the priciest
  model, for the cheapest work. Strongest candidate.
- **`judge`** — the goal-check judge tier (`bee-hive`, "§ Goal-check
  judge tier") currently has no slot of its own.
- **`plan`** — planning research dispatches; today they take the
  generation slot by default (decision 0023's aux-dispatch line).
- **`commit`** — commit-message and scribe writing; mechanical, and the
  cheapest tier would do.

For each: is there a dispatch site, and does routing it separately
change the model actually chosen? A role whose answer is "it would
resolve to the same model as `generation`" is a config knob with no
effect and should not ship.

## Constraint from the map

Whatever the count, each role costs: an entry in `CONFIGURABLE_SLOTS`
(`models.rs:37`), a branch in two independent parsers (`models.rs:318`,
`model_guard.rs:442`), and an entry in two hand-maintained guard tier
lists (`model_guard.rs:192-193`). See ticket 004.

## Answer

(open)
