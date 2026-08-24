---
type: grilling
status: closed
claimed-by: wayfinder (resolved)
blocked-by: (none)
---

## Question

`extraction` is a configurable slot (`models.rs:37`) that no dispatch
kind selects: `DISPATCH_KINDS` has no `extract`, and `slot_for_kind`
sends both `cell` and `gather` to `generation`
(`prepare.rs:31`, `prepare.rs:34-40`).

**Premise corrected 2026-08-24 (verified read).** The slot is not
inert — it is reachable by the *other* path. A cell may record
`tier: extraction` (`verbs/cells/validate.rs:29`), and `--kind cell`
prefers that recorded tier over the slot default
(`prepare.rs:731-745`), so the extraction model does resolve. What is
missing is the *role* path: no `--kind` selects extraction, and
`bee-extract` — rendered, onboarded, tier `extraction`
(`onboard/templates.rs:222-230`) — is never a value `prepare` can
return, because `pinned_agent_type` is consulted only when
`kind != "cell"` (`prepare.rs:810-811`). So the defect is a read-only
extraction *worker* that the one door cannot dispatch, while the
shipped swarming reference still instructs agents to name it
(`swarming-reference.md:104-114`, `:294`) and AGENTS.md forbids
hand-picking `subagent_type`.

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
- `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:32-39` — `"extraction" => "bee-extract"`, the only mapping, unreachable from `prepare`
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:653-659` — source comment: "there is no `--kind` value that resolves the extraction slot today"
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:660-666`, `:768` — refusal text "dispatch prepare has no --kind for the {t} tier yet"
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1614-1634` — test pinning that refusal
- decision `a2f85972` — the guard's herding-fallback widening is scoped to generation+review *because* extraction is unreachable; option 1 or 2 touches it
- decision `de967733` + `3ff7cd72` — down-tier I/O dispatch is bee's one cost pattern, and `tier_mix extraction 1` proves the tier ran live

## Answer

**Option 1 — add a kind.** Owner answer 2026-08-24. `bee dispatch
prepare` gains `--kind extract`, resolving the `extraction` slot and
returning the rendered `bee-extract` worker; `--kind gather` keeps
`generation`, so no existing caller changes model.

Full text, rationale and the rejected alternatives: decision
`8dad7c2e`. The consequences it records — the guard's
`dispatch_kind_for_tier` extraction arm, the widened herding-fallback
member set (touches `a2f85972`), and the now-reachable swarming
reference instruction — are that decision's, not this ticket's.

Frame this answer established, and the map's spine from here: **a model
role is reachable by two independent paths — a cell's recorded `tier`,
or a dispatch `--kind` — and both are legitimate.** `extraction` had
the first and lacked the second. Ticket 002 is re-framed on that split;
ticket 005 asks whether the pairing is enforced.
