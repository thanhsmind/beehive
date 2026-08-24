---
type: grilling
status: open
claimed-by: (unclaimed)
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

## Answer

(open)
