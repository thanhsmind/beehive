---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none)
---

## Question

Does a role entry gain an **ordered fallback chain** (model A, then B,
then C on failure), or do bee's two existing single-step mechanisms stay
as they are?

What exists today:

- Explicit-only composite `{primary, fallback_policy: "explicit-only",
  fallback: {kind: "cli", …}}` — `models.rs:134-166`, decision 3ceba8f5
  D2. One fallback, and by that decision it never fires silently.
- Herding slot `fallback: "default"` — `models.rs:112-133`, decision
  267192c1. A flag, not a model; absent, a failure stays loud.

Both were deliberately built to fail loudly rather than degrade
quietly. A chain is the opposite posture: keep going down the list. So
this is not an additive feature — it reopens a settled stance.

The real question is therefore: **which failures should a chain absorb?**
A quota refusal and a rate limit are transient and worth retrying
elsewhere. A tool-contract failure or a bad result is not — falling
through to a weaker model there hides the defect.

Related evidence that the loud posture has teeth: decision 4faf1de9 —
an advisor consult was recorded as NOT OBTAINED when the configured
advisor hit its quota, and no substitute was run, because the advisor
has no fallback by design.

## Answer

(open)
