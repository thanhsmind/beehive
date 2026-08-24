---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none)
---

## Question

Ticket 001 (decision `8dad7c2e`) found a configurable slot that no door
kind could reach, and it stayed that way long enough for the docs, the
guard's refusal text and a pinned test to all describe the gap as
normal. Does bee now **enforce** that this cannot recur?

The candidate invariant: *every entry in `CONFIGURABLE_SLOTS` is
reachable by at least one path — a `DISPATCH_KINDS` entry that
`slot_for_kind` maps to it, or a `MODEL_TIERS` value a cell may record.*

Three shapes it could take:

1. **A test.** One assertion over the three constants
   (`CONFIGURABLE_SLOTS` — `models.rs:37`, `DISPATCH_KINDS` /
   `slot_for_kind` — `prepare.rs:31-40`, `MODEL_TIERS` —
   `validate.rs:29`). Cheapest; catches the drift at CI time.
2. **A doctor check.** `bee doctor` reports an unreachable configured
   slot in the *host's* config, not only in bee's own constants.
   Catches a host that configures a slot bee cannot dispatch.
3. **Nothing — the pairing stays a convention.** Accept that a role and
   its door are added together by whoever adds them, as ticket 002 will
   do for each new role.

## Why it matters here

Ticket 002 adds roles. Whatever it adds multiplies over the
duplications ticket 004 lists. If the pairing is not enforced, the next
unreachable slot is one merge away — and 001 shows the failure mode is
silent for months, because every surface politely describes the gap
instead of failing.

## Relation to ticket 004

004 asks whether the duplicated *definitions* collapse into one source.
005 asks whether the *pairing* between a slot and its door is checked.
An answer to 004 that produces one shared table may make 1 trivial —
resolve 004 first if both are open.

## Evidence

- decision `8dad7c2e` — ticket 001's answer and the two-path frame
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:653-659` — the
  gap, written down in a source comment rather than caught
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1614-1634` — a
  test that pins the *symptom* of the gap as expected behavior

## Answer

(open)
