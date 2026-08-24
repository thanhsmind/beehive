---
type: grilling
status: closed
claimed-by: wayfinder (resolved)
blocked-by: (none)
---

## Question

Decision `97ce5225` leaves a cell with exactly one open-ended name — its
`role` — and decision `3c9d6262` makes whoever authors the cell
responsible for choosing it. Nothing yet says **how** they choose.

This was fog in the map until `97ce5225` settled that the judgment is about
the job only; cost left the field with `tier`. It is now sharp enough to
ask:

1. **Free text, no vocabulary.** The author writes any name. Config
   decides whether it resolves; an unconfigured name warns and falls
   through (`06e49368`). Cheapest, and matches the open set — but two
   authors will write `test` and `testing` for the same job, and both
   will silently fall through to the default.
2. **A published vocabulary, plus room to invent.** bee's planning
   surface names a short recommended set (the jobs bee itself asks for,
   plus common ones like `test`, `docs`, `design`), the author picks
   from it or writes their own. Guidance, not enforcement.
3. **Derived from the cell's own fields.** A cell already declares
   `affects_skills`, `affects_specs`, and `must_haves`. A rule could
   read a role out of them — a cell touching only tests is test work.
   No new judgment for the author at all, but a derivation that guesses
   wrong is invisible.

## What makes it non-obvious

The whole value of `3c9d6262` rests on the role being *right* often
enough to be worth configuring a model for. A vocabulary that drifts
(option 1) means a user's configured `test` model never fires, because
half the cells say `testing`. A derivation that guesses (option 3) means
the user cannot tell why their `test` model did or did not run. Option 2
costs a list somebody has to maintain.

Note the failure is quiet in every option: an unmatched role falls
through to the default and the work still completes, just on the wrong
model. There is no red output to notice.

## Evidence

- decision `97ce5225` — role is the sole selector; cost left the cell
- decision `3c9d6262` — the cell's author writes the role
- decision `06e49368` — unconfigured names warn, then fall through
- `.bee/cells` scan, 2026-08-24: authors skipped the existing optional
  `tier` field on 215 of 506 cells, which is the closest evidence we
  have of how much per-cell judgment authors actually supply

## Answer

**Option 2, enforced** — decision `4eaf1b71`. `role` is **required** on a
cell, exactly as `lane` is: `bee cells add` refuses without it. The
value is any non-empty name (the open set of `06e49368` holds, so
validation checks presence and shape, never membership). bee ships a
recommended vocabulary — `code`, `read`, `test`, `docs`, `review`,
`design` — as *authoring guidance* on the planning surface and in
`bee cells add --help`, never as a validation enum.

Decided on a natural experiment already in the store, not on
preference:

| Field | Rule | Present in 506 cells |
|---|---|---|
| `lane` | required (`validate.rs:133-140`) | **506 / 506** |
| `tier` | optional (`validate.rs:164-173`) | 291 / 506 |

Same authors, same cells, same file. The 100 percent is enforcement,
not diligence — a cell without a lane never enters the store — and that
is the argument: enforcement is the only mechanism shown to make a cell
field present. An optional role reproduces the `tier` outcome, where a
configured per-job model fires on about half the cells that wanted it
and the miss is **silent**, because fall-through completes the work on
the default with no red output.

Backfill for the 506 stored cells is recorded in `4eaf1b71` (d).

Owner-delegated: the owner handed the remaining tickets to the agent on
2026-08-25 ("cứ làm tới khi ra plan không cần hỏi thêm"). This answer is
the agent's call under that delegation and is overturnable.
