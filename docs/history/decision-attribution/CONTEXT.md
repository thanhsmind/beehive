# decision-attribution — CONTEXT

**Route.** class `bugfix` · lane `standard` · flags `data-model`,
`public-contracts`, `covered-contract-change` · 4 product files.

## The bug, proven

`bee decisions log` stamps a `feature` on every new decide event. It
resolves that name through `resolve_mutation_target(root, None, "decisions
log", false)` (`verbs_read.rs:613`), which returns, in order:

1. an explicitly named lane, else
2. **the calling session's bound lane**, else
3. **the shared default `.bee/state.json` record** (`ledger.rs:337`).

Step 3 is the defect. It is correct for the verbs `resolve_mutation_target`
was built for — an unbound session *mutating* state should act on the default
record. But `decisions log` uses it only to **read a name to stamp**, and
there the default record's `feature` is not "this decision's feature". It is
whatever feature some *other* session most recently made active.

**Measured on `.bee/decisions.jsonl`, 2026-08-25.** 32 decisions carry
`feature: model-role-split`. 18 of them open with `human-mailbox D1..D18`
and 5 more are `prompt-work-record`'s. Every one of those 23 was written by a
session that had no lane of its own at the time:

```
18 of 18 human-mailbox decisions were logged BEFORE the human-mailbox
lane existed — earliest 10:11:06, latest 10:55:26; lane created 10:57:48.
```

That is the whole mechanism. The bug bites the **Discovery flow**
specifically, because a discovery map locks decisions ticket by ticket, and
`bee-wayfinding` runs before any lane exists. The busiest decision-logging
phase in bee is exactly the phase with no lane to attribute to.

**Two consequences, both observed.**

- `model-role-split`'s close was blocked by an impact door naming 34 citing
  docs it had no relationship with — the citing docs belong to
  `human-mailbox`'s live discovery map. It closed on a recorded exemption.
- The reverse loss is larger and silent: `human-mailbox` and
  `prompt-work-record` will not see their own decisions at close, so their
  routing door will not ask for those D-IDs to be routed into
  `docs/knowledge/`.

**Why no test caught it.** `log_touches_sweep_own_history_stays_a_citation_when_no_feature_is_bound`
(`decisions/tests.rs:1727`) covers the unbound case with a fixture that has
**no `.bee/state.json` at all**. The real shape — a state file that exists and
names a foreign feature, with no bound lane — has no test.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | **An unresolved feature is stamped as absent, never borrowed.** When `decisions log` cannot resolve a feature from an explicit `--feature` or a bound lane, the event carries **no `feature` field**. The default-state-record fallback is removed from this one call site only. | Absent is already a supported, tolerated state — the code's own comment says "Legacy (pre-D1a) lines simply lack the field, and every reader tolerates that absence." A wrong name is strictly worse than no name: it is invisible, it misroutes two doors, and it cannot be told apart from a correct one by reading. |
| D2 | **`bee decisions log` gains `--feature <slug>`.** An explicit value always wins over the bound lane. | D1 alone would leave Discovery decisions permanently unattributed, which trades a wrong answer for no answer. `--feature` is how a wayfinding session names the effort it is charting before that effort has a lane. |
| D3 | **`resolve_mutation_target` is not changed.** The fix lives in `decisions log`'s use of it. | It is shared by many state-mutating verbs, for which fallback-to-default is the correct and intended behavior. Changing it to fix one caller's misuse would be a wide blast radius for a narrow bug. |
| D4 | **The regression test pins the real shape**: a `.bee/state.json` that exists and names a foreign feature, with no bound lane, must produce an event with no `feature`. | The existing unbound-case test uses a fixture with no state file, so it passes both before and after the fix. A test that cannot fail is not proof (recurring pattern `20260819`). |

## D5 — answered by the owner, 2026-08-25

**What happens to the 23 records already stamped wrong?** Answer: **(a),
narrowed** — corrected in place by a one-time, dry-runnable migration that
touches only the `feature` label, only on records whose own text opens with
`<other-feature> D<n>` naming a feature different from the stamp, and reports
every change. Idempotent. The reasoning and the append-only tension are
recorded in the store under `decision-attribution D5`.

<!-- bee:not-a-deferral: this sentence explains why the superseded framing is kept in the document; it records a decision already taken and promises no work. -->

The original framing is kept below, because the tension is real and a
subsequent reader should see what was traded.

<!-- /bee:not-a-deferral -->

**Shipped since:** `bee decisions reattribute` now exists and was applied on
2026-08-25 — 2358 scanned, 67 corrected, 0 on a second run. The paragraph
below describes the state at the time the question was asked, when there was
no such verb.

At the time of asking there was no verb to re-attribute a decision (`bee
decisions` had log,
supersede, redact, active, search, archive, tag, render), and hand-editing
`.bee/*.jsonl` is forbidden by AGENTS.md.

- **(a) Correct them in place**, via a one-time dry-runnable migration, the
  way `bee cells backfill-roles` corrected 540 cell records. Precedent
  exists and it is idempotent and testable. **Tension:** AGENTS.md says
  "Historical records are never rewritten: decisions are superseded,
  learnings and logs appended."
- **(b) Leave them and let the two features take a recorded exemption**, the
  same one `model-role-split` took. Honours the append-only rule exactly.
  **Cost:** `human-mailbox`'s 18 D-IDs and `prompt-work-record`'s 5 never get
  routed into the knowledge layer, and the loss is permanent and quiet.

**Recommendation: (a), narrowed.** The append-only rule protects a decision's
*content* — what was decided and why. `feature` is a filing label bee wrote
itself, not something a human decided, and it was written wrong by a bug. A
migration that touches only that one field, refuses to run on any decision
whose text does not name a different feature, and reports every change, is
correcting a mis-filed record rather than rewriting history. Scope it to
records whose `decision` text opens with `<other-feature> D<n>` — that is what
made the 23 provable in the first place.
