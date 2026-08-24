---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none)
---

## Question

`extraction` is a configurable slot (`models.rs:37`) that no dispatch
kind selects: `DISPATCH_KINDS` has no `extract`, and `slot_for_kind`
sends both `cell` and `gather` to `generation`
(`prepare.rs:31`, `prepare.rs:34-40`). So `models.claude.extraction`
is set in this repo's own `.bee/config.json` and never used.

Which is the intended fix:

1. **Add a kind.** A new `--kind extract` resolving to the `extraction`
   slot, leaving `gather` on `generation`. Callers choose. Matches the
   two agents that already exist (`bee-extract` vs `bee-gather`) and
   their stated split — a narrow scoped lookup versus an open-ended
   multi-file hunt.
2. **Remap `gather`.** Point `gather` at `extraction` and let `cell`
   keep `generation`. No new kind, but it silently moves every existing
   gather onto a cheaper model — a behavior change for every caller,
   including ones that chose `gather` for a heavy sweep.
3. **Delete the slot.** Concede that `extraction` was never wired and
   drop it from `CONFIGURABLE_SLOTS`, folding its agents onto
   `generation`.

## Why it comes first

Every candidate role in ticket 002 needs a dispatch kind to be
reachable. Whatever answer 001 takes is the pattern the new roles copy;
deciding role count before this is decided means guessing at that
pattern.

## Evidence

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:37`
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:31`
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:34-40`
- `.bee/config.json` — `models.claude.extraction: "sonnet"`, live and unused
- `rg -n extraction packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — no match

## Answer

(open)
