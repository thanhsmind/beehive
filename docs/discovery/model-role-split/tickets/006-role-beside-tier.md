---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none)
---

## Question

Decision `3c9d6262` gives a cell a **role** (its job). A cell already has a
**tier** (its cost: `extraction` | `generation` | `ceiling`). Do both
survive, or does one absorb the other?

1. **Both, and the dispatch list carries both.** A cell's request
   becomes roughly `[<role>, <tier>, <default>]` — job first, cost as
   the fallback, the always-configured backstop last. Keeps every
   existing tier mechanism working untouched.
2. **`role` replaces `tier`.** One field. `ceiling` and the cost words
   become role names like any other, and a cell that wants the cheap
   model just writes `role: read`.

## What hangs on it

`tier` is not only a model selector — real machinery keys off it, and
option 2 has to rehome all of it:

- **The ceiling budget guard.** `CEILING_SHARE_REFUSAL_MAX = 0.4`
  (`verbs/cells/handlers_close.rs:1063`) refuses a `--tier ceiling`
  past a 40 percent share unless `--reason` is given (`:1126-1133`),
  persisting `tier_reason` on the trace. A role-only world needs a rule
  for which role names are rationed.
- **`ceiling` is deliberately not configurable** (decision `0015`) and
  resolves to `Resolved::Inherit`, the session model
  (`prepare.rs:761-762`). An open role set where any name is legal has
  to keep exactly one name that means "the session model" and refuses
  configuration — an exception to `06e49368`'s open rule.
- **Tier-mix accounting at close** (`handlers_close.rs:1054-1102`) and
  the preamble's ceiling-erosion advice
  (`hooks/session_preamble/store.rs:309-320`) both count tiers. Under
  option 2 they count roles, and "how much did we spend at the top" is
  no longer a single field.
- **The worker registry** carries `--tier` with the same closed enum
  (`verbs/state_group/workers.rs:89`).

## Why it is not obvious

Option 1 is safer and keeps cost visible as its own number, but it asks
the cell author for two judgments instead of one, and the two can
disagree — `role: test` with `tier: ceiling` says "cheap job, priciest
model" and nothing rejects it. Option 2 is simpler to configure and
matches the source read, where cost words are just role names among
others — but it dissolves the one number bee currently uses to prove it
is not overspending.

## Evidence

- decision `3c9d6262` — the cell declares its job role
- decision `06e49368` — open, fall-through role set
- decision `0015` — `ceiling` means the session model and is never
  configurable
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1063`,
  `:1126-1133`, `:1054-1102`
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:731-745`,
  `:761-762`

## Answer

(open)
