---
type: grilling
status: closed
claimed-by: wayfinder (resolved)
blocked-by: (none)
---

## Question

Before the role surface grows, do the duplicated definitions collapse
into one source?

Three duplications, all hand-maintained:

1. **Two parsers of the same config shape.** `resolve_tier` over
   `Map<String, Value>` — `models.rs:318-383`; a second `resolve_tier`
   over the guard's own structs — `model_guard.rs:442-467`. Plus
   `resolve_advisor` in both (`models.rs:387-437`,
   `model_guard.rs:470-483`).
2. **Two tier lists, already drifted.** `CLAUDE_TIERS` has 4 entries and
   omits `advisor`; `CODEX_TIERS` has 5 and includes it —
   `model_guard.rs:192-193`. Whether that asymmetry is intended or is
   itself a bug is part of this question.
3. **Slot list vs normalize list.** `CONFIGURABLE_SLOTS` (3) and
   `MODEL_NORMALIZE_SLOTS` (4) — `models.rs:37`, `models.rs:40`.

Every role added by ticket 002 multiplies across all of these. The
drift risk is not hypothetical: item 2 has already drifted.

## Narrowed 2026-08-24 by decision `06e49368`

The open-set clause retires two of the three duplications this ticket
lists outright: item 2 (`CLAUDE_TIERS` / `CODEX_TIERS`, already drifted
4-vs-5) disappears when the guard asks "is this role configured"
instead of checking membership, and item 3 (`CONFIGURABLE_SLOTS` vs
`MODEL_NORMALIZE_SLOTS`) loses its meaning when any configured name is
a legal role.

**What stays open:** item 1 — the two independent `resolve_tier`
implementations (`models.rs:318-383` over `Map<String, Value>`, and
`model_guard.rs:442-467` over the guard's own structs), plus
`resolve_advisor` in both. Two parsers of one config shape is still two
parsers, and an open role set makes them *more* load-bearing, not less.
Also newly in scope: the four private copies of `MODEL_TIERS`
(`verbs/cells/validate.rs:29`, `verbs/state_group/mod.rs:166`,
`verbs/status_full/mod.rs:60`, `hooks/session_preamble/mod.rs:106`) and
the closed 3-value enum on `bee cells tier` and
`bee state worker add --tier`.

## Answer

**Yes, and it lands first** — decision `cd72ec97`.

Most of what this ticket listed collapses on its own from decisions
already taken: `CLAUDE_TIERS`/`CODEX_TIERS` end with `06e49368`'s
is-this-configured check; `CONFIGURABLE_SLOTS` vs
`MODEL_NORMALIZE_SLOTS` ends when any configured name is legal; the
four private `MODEL_TIERS` copies end with `97ce5225` retiring `tier`
as a selector.

One real item stays: the two independent `resolve_tier` implementations
(`models.rs:318-383`, `model_guard.rs:442-467`) plus the paired
`resolve_advisor`. They become **one** `resolve_role` in the drivers
module, which the guard calls rather than reimplements.

The operative half is **ordering**: that collapse is the first step of
the implementation sequence, not the last. Every remaining step in this
map writes into the shape it parses, so a second parser would have to
be edited in lockstep four more times. The drift is not hypothetical —
the guard's two tier lists had already drifted 4 against 5 with nothing
intending it, and that asymmetry is recorded as a defect rather than a
design.

Owner-delegated, 2026-08-25; the agent's call, overturnable.
