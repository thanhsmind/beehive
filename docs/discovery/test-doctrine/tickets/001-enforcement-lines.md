---
type: grilling
status: closed
claimed-by: session-wayfinding-1
blocked-by: 002-mechanism-inventory
---

## Question

With the agent owning test scope (D-58ec9664), which mechanical
enforcements does the CLI keep, exactly: does any door (release-mode
`bee close`, merge to main) still auto-run anything, and in which
structured field does the agent's scope + reason live (cap report key,
decision log, or a new trace field)?

## Answer

D-1f534837: no auto-run anywhere — close/merge require a recorded
proof line instead of running the suite; scope + reason live in the
cap report's `tests` field as a proof string
`<command> — <result> — <scope reason>` (replaces the
boundary/undeclared enum). CI full remains the deterministic net.
