---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none) — unblocked by 001
---

## Question

Which new roles earn their place — meaning each has a real dispatch site
that would select it, not just a name on a config screen?

**Re-framed by ticket 001's answer (decision `8dad7c2e`).** A role is
reachable by two independent paths, and the candidates below do not all
want the same one:

- **Cell tier** — the work item carries `tier:` and `--kind cell`
  honours it (`validate.rs:29`, `prepare.rs:731-745`). Fits a role that
  describes *how big the work is*.
- **Dispatch kind** — the caller passes `--kind` and the door resolves
  the slot (`prepare.rs:31-40`). Fits a role that describes *what job
  the worker does*.

So each candidate now needs two answers, not one: does it earn a slot,
and which path reaches it? A role that wants a cell tier costs nothing
at the door; a role that wants a kind costs a `DISPATCH_KINDS` entry, a
`slot_for_kind` arm, and a `dispatch_kind_for_tier` arm.

Candidates, with the site that would use each:

- **`tiny`** — reads as a **cell tier**, not a kind: it names work
  size. A `tiny` cell may run inline on the session model
  (AGENTS.md, "From small up, cells run through dispatched workers … a
  tiny cell may run inline"). Inline means the ceiling, the priciest
  model, for the cheapest work. Strongest candidate.
- **`judge`** — reads as a **dispatch kind**: it names a job. The
  goal-check judge tier (`bee-hive`, "§ Goal-check
  judge tier") currently has no slot of its own.
- **`plan`** — reads as a **dispatch kind**. Planning research
  dispatches; today they take the
  generation slot by default (decision 0023's aux-dispatch line).
- **`commit`** — path unclear; ask which it is. Commit-message and
  scribe writing; mechanical, and the
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
