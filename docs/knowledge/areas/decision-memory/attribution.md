---
type: bee.area
title: Decision Memory — which feature a decision belongs to
description: "How a decision event gets its feature stamp: a lane-resolved name or an explicitly named one, never the shared default record's; why an absent stamp beats a borrowed one; and the narrow correction verb for records whose stamp already contradicts their own text."
timestamp: 2026-08-25
bee:
  id: decision-memory-attribution
  lifecycle: active
  areas: [decision-memory]
  required_context: [areas/decision-memory/overview.md]
  decisions: [decision-attribution D1/D2/D3/D4/D5]
  sources: ["decision-attribution (docs/history/decision-attribution/CONTEXT.md, cells da-1/da-2/da-3, merged 2026-08-25)", "measured on .bee/decisions.jsonl 2026-08-25: 67 records corrected out of 2358 scanned", "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs (feature_for_stamp)", "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs (plan_reattribution, do_reattribute)"]
  authoritative_for: "decision-memory: which feature a decision event is attributed to"
---

# Decision Memory — Which Feature a Decision Belongs To

A decide event may carry a `feature`. Two close doors read it: the impact door
walks the citing docs of a closing feature's decisions, and the routing door
asks that every locked D-ID reach the knowledge layer. A wrong stamp therefore
does double damage — it blocks a feature on documents it has nothing to do
with, and it silently denies the true owner its own decisions at close.

## Behaviors & Operations

**B1 — Only a lane-resolved name may be stamped** (decision-attribution D1).
`decisions log` resolves the feature from the calling session's bound lane. It
does **not** consult the shared default `.bee/state.json` record. With no bound
lane and no explicit name, the event carries no `feature` key at all.

The rule exists because the resolution helper it calls,
`resolve_mutation_target`, falls back to that default record — correct for the
verbs it was built for, where an unbound session *mutating* state should act on
the default record. This call site is different in kind: it reads the target
only to borrow a **name**, and the default record's `feature` is whatever
another session most recently made active.

Measured before the fix, 2026-08-25: **67 of 2358 decisions carried a feature
that their own text contradicted**, spread across `human-mailbox`,
`prompt-work-record`, `herding-orchestration`, `test-cadence-boundary`,
`staging-lane`, `uat-after-merge` and more.

**B2 — Absent beats wrong.** A missing `feature` is a supported state that
every reader already tolerates; legacy pre-stamp lines simply lack the field. A
wrong one is invisible on inspection and cannot be told from a correct one by
reading. So the failure mode is chosen deliberately: say nothing rather than
say something untrue.

**B3 — An explicit `--feature` outranks the bound lane**
(decision-attribution D2). It is how a session names the effort it is charting
before that effort has a lane. This is not a convenience: the **Discovery flow
is where decisions are logged most**, and a wayfinding map locks them ticket by
ticket, before any lane exists. Without this door, B1 would trade a wrong
answer for no answer. A blank value is refused rather than ignored — passing
the flag is an act of naming, and silently dropping it would stamp the lane
while appearing to obey.

**B4 — The shared resolver is not changed** (decision-attribution D3).
`resolve_mutation_target` keeps its default-record fallback for all of its
other callers, every one of which either passes an explicit lane or uses the
result as a genuine mutation target. Widening a shared helper to fix one
caller's misuse puts a large blast radius behind a narrow bug.

**B5 — A stamp is corrected only where the record contradicts itself**
(decision-attribution D5). `bee decisions reattribute` acts on a record only
when its `decision` text opens with `<slug> D<n>` **and** that slug differs
from the stamped `feature`. Consequences of the narrowness, all deliberate:

- A record with **no** stamp stays unstamped. Post-B1 that is a normal state,
  and filling it in from a prose convention would be the inference B3's
  alternatives rejected — the `<slug> D<n>` habit is a convention, not a
  contract.
- A record whose text makes no claim is left alone whatever its stamp says.
- Only the `feature` field is ever written. Every other field, and every
  untouched line, is byte-identical afterwards.

The verb holds the decisions lock across the **whole** pass, the read included.
That is not incidental: `cells backfill-roles` shipped a scan outside its lock
and silently reversed an operator's concurrent write, and sibling sessions
append to this store continuously. It is idempotent, so an interrupted run is
finished by running it again, and `--dry-run` reports the same counts while
writing nothing.

**B6 — The regression is pinned on the shape that actually broke**
(decision-attribution D4): a `.bee/state.json` that EXISTS and names a foreign
feature, with no bound lane. The pre-existing unbound-case test used a fixture
with no state file at all, so it passed both before and after the fix and was
never evidence.

## Why this is append-only's one narrow exception

Historical records are never rewritten — decisions are superseded, logs
appended. B5 writes to records already on disk, so it needs its warrant stated:
the rule protects a decision's **content**, what was decided and why, and B5
never touches that. `feature` is a filing label bee wrote itself, by machine,
and wrote wrong. The text-proven predicate is what keeps it a correction rather
than a rewrite — the verb can only act where the record already contradicts
its own stamp, so it cannot express an opinion of its own.

**B7 — A human may name the correction for a record whose text makes no
claim** (reattribute-by-name, cell rbn-1, 2026-08-26). `bee decisions
reattribute --id <decision> --to <feature>` corrects exactly one record by
the operator's explicit word — the case B5's predicate correctly declines,
because a text-less record carries no contradiction to resolve. The pair
comes together or not at all; the first id match wins outright so a short
prefix can never rewrite a second record; a record whose own text claims a
**different** feature refuses toward the automatic pass — the manual door
never contradicts a record's text; and only the `feature` field is ever
written. Applied 2026-08-26: the five `prompt-work-record` records took
their own feature, and a re-run reports zero.

## Open Gaps

- Nothing prevents a session from binding a lane that is not what it is
  actually working on, which would reintroduce a wrong stamp by a different
  route. B1 removes the silent case, not the mistaken one.
- `.bee/decisions.jsonl` is git-tracked and merges as TEXT, so a worktree
  branch that diverged before a correction can resurrect the stale stamps at
  merge time — observed 2026-08-26, when 18 corrected records reverted after
  a merge and the automatic pass re-fixed them. Recoverable but silent;
  filed as a P2 (run the automatic pass after a merge touching the store, or
  merge the store as append-only data).
