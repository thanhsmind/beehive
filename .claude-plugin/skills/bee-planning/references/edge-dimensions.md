# The 12 Edge-Case Dimensions

**Scope: `high-risk` and hard-gate work only.** Never reach this file from a
standard lane: at `standard` and below the test shape is the triad — happy
path, edge cases, error paths — at its smallest demonstrating size. Read as a
checklist to fill, twelve dimensions generate volume — the opposite of what a
standard slice needs. See `planning-reference.md` ("Tests ride the cell").

At `high-risk`/hard-gate, use this during the planning test matrix (and again in
reviewing coverage checks): written probes per applicable dimension, each mapped
to a cell's `must_haves.truths`. Even here, the cell's writer judges existing
coverage first and authors only the dimensions not already pinned by an
existing test.

## 1. User types

- What does an anonymous / unauthenticated actor see or trigger here?
- Admin vs regular user vs service account — same path, different rights?
- First-time user with no data vs power user with years of history?

## 2. Input extremes

- Empty string, null, missing field, zero-length file?
- Maximum realistic size (10MB payload, 100K-row CSV, 4-hour audio)?
- Unicode, emoji, RTL text, control characters, `"; DROP TABLE`-shaped input?

## 3. Timing

- Two requests arrive within the same millisecond — race or last-write-wins?
- The operation is interrupted halfway (kill, deploy, network drop) — what state remains?
- Clock skew / timezone boundary / DST transition during the operation?

## 4. Scale

- 0 items, 1 item, 10K items — does the UI/query/loop survive all three?
- N+1 queries or unbounded fan-out when the collection grows?
- What is the first thing that falls over at 100× current volume?

## 5. State transitions

- Every state → every event: which combinations are impossible, and are they rejected?
- Re-entry: the same transition fired twice (double-click, retry, webhook redelivery)?
- Resuming from a stale state after a long pause (expired session, superseded record)?

## 6. Environment

- Works on the dev machine — what differs in CI and production (paths, env vars, permissions)?
- Missing optional dependency, older runtime version, case-sensitive filesystem?
- Offline / degraded network / proxy in the middle?

## 7. Error cascades

- The dependency this feature calls returns 500 — what does *our* caller see?
- Retry storms: does a failure retry itself into a bigger failure?
- Partial success: 3 of 5 items processed, then a crash — are the 3 reported, rolled back, or lost?

## 8. Authorization

- Horizontal escalation: user A requests user B's resource by id — 403 or leak?
- Object-level checks on every path (list, detail, export, webhook), not just the UI route?
- Revoked/expired credentials mid-session — cut off now or at next login?

## 9. Data integrity

- Can this write produce orphans, duplicates, or dangling references?
- Concurrent edit of the same record — merge, reject, or silently clobber?
- Is the migration reversible, and what happens to rows written between deploy and rollback?

## 10. Integration

- Contract drift: the external API adds/renames a field — do we break loudly or corrupt quietly?
- Idempotency keys / dedup on inbound webhooks and outbound retries?
- Version mismatch between our client and their server (or two of our own services)?

## 11. Compliance

- PII in logs, error messages, analytics events, or LLM prompts?
- Deletion requests: does "delete my account" actually reach this feature's data?
- Audit trail: can we answer "who changed this and when" for regulated actions?

## 12. Business logic

- Boundary values of every rule: exactly at the limit, one over, one under (quota 10 → test 9, 10, 11)?
- Money/rounding: fractions of cents, negative amounts, currency mismatch?
- Rule conflicts: two policies both apply and disagree — which wins, and is it deterministic?
