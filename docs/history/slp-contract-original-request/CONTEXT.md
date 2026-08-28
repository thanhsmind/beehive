# SLP Contract Status + Original Request — Context

**Feature slug:** slp-contract-original-request
**Date:** 2026-08-28
**Shaping session:** complete (Lock from a closed map; no interview was owed)
**Scope:** Deep
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Two halves of one anti-drift guarantee. A worker can tell whether the contract
it is about to write tests against is SETTLED, and is refused when it is not;
and the user's own words ride every dispatch untouched, so no layer between
the ask and the work can quietly replace them. It ends there — no new store,
no interface registry, and no change to who may settle a contract.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

Source: `docs/discovery/slp-supervisor-lead-peer/MAP.md` (cluster 4 of the
build order) and `docs/discovery/slp-supervisor-lead-peer/tickets/007-contract-status-original-request.md`.
D-IDs below are local; the store ids they render are cited in each row.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A contract's settled/unsettled status is a DERIVED view over the decision log. A contract is settled if and only if a locked decision says so, and bee keeps no hand-maintained contract registry. (store `ca9960f5`) | One source of truth, and nothing new to forget to update. A second registry drifts from the log the first week nobody updates it. |
| D2 | The label is the tag convention `contract:<name>` over the ACTIVE decision set: settled is an active decision with no waiting trigger; unsettled is a decision whose trigger is waiting or due. (store `9c0104e0`) | The log already owns the three hard parts — supersession keeps the label current for free, the deferral guard already forces an unsettled decision to name a trigger, and the query surface exists. |
| D3 | Cells cite contract decisions in the EXISTING `cell.decisions` field, and a prepare/claim-time tripwire refuses the dispatch when a cited decision is retired or trigger-waiting. (store `9c0104e0`) | The cell field is already the slot for this and already rides the worker prompt verbatim. The tripwire is what makes a citation mean something rather than decorate the record. |
| D4 | A test-writing cell that cites NO contract decision is refused — the mint trap. (store `9c0104e0`) | The absence problem: a never-logged contract otherwise reads exactly like "there is no contract", and the worker mints one by writing tests against it. A refusal rule answers this without pre-enumerating interfaces. |
| D5 | The user's verbatim original request rides every cell and dispatch as an immutable field. Intermediate layers may only ADD guidance — never replace, never paraphrase. (store `3899fa60`) | Cheap insurance against meaning drift across delegation layers. |
| D6 | `bee intent`'s existing verbatim anchor serves D5: its request field, under its DO-NOT-PARAPHRASE framing, is read at dispatch prepare and rendered into every worker, gather, reviewer and advisor prompt template. (store `9c0104e0`) | The anchor already holds the verbatim string under the exact framing this asks for; it survives compaction today but never reaches a dispatch. The dispatch door is the one place every worker passes, so one read there covers all of them without per-cell copies of the same string. |

### Agent's Discretion

D2 fixes the tag convention but not its spelling in prose, and D3 fixes that a
tripwire exists at prepare or claim time but not which of the two doors carries
it — planning picks the door on evidence and records the reason.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Contract | An interface or agreement a caller depends on, named by the `contract:<name>` tag on the decision that settles it |
| Settled | An active decision tagged for the contract, with no waiting or due trigger |
| Unsettled | A decision for the contract whose trigger is waiting or due — the revisit condition is attached and has not fired |
| The mint trap | A worker writing tests against a contract nobody settled, so the tests themselves become the de-facto contract |
| Original request | The user's own words for what they asked for, stored once under the intent anchor and never rewritten by any layer |

## Specific Ideas And References

- The user picked the derived view over a registry in the 2026-08-26 grilling
  round, on the ground that one source of truth beats two that disagree.
- The spec's open question "who edits contract status" resolves by
  construction: decisions are user-gated, so the label's authority is already
  the human's.

## Existing Code Context

From the advisor-tier research digest only. Downstream agents read these before
planning, and re-verify the line anchors — the digest predates two features
that have since edited the same files.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs` — the active
  decision set already excludes superseded ids, the query surface already
  filters by tag, feature and cell, and the deferral guard already refuses an
  unsettled decision that names no trigger
- `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs` — the trigger record
  carries the waiting/due/resolved status keyed to a decision id, which is the
  machine-readable half of D2
- `packages/bee-rs/crates/bee/src/verbs/intent_group.rs` — the intent anchor
  and its verbatim, do-not-paraphrase render header
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — `prompt_body_for`
  and its vars slice, the single door every dispatch passes

### Established Patterns

- A conditional prompt block that renders byte-identically when its value is
  absent — established twice already, by the lane brief and by the dispatch
  reading list, with the absent case pinned for every runtime and kind
- A typed, zero-mutation refusal at the dispatch door that names its remedy —
  the shape every brief refusal takes

### Integration Points

- `packages/bee/prompts/worker-cell.md`, `advisor.md`, `gather.md`,
  `reviewer.md` and their vendored twins — the four templates D6 names, plus
  the release manifest and a rebuild, because a prompt edit and a rebuilt
  executable are one unit of work
- The cell record's `decisions` field, already inlined into the worker prompt

## Canonical References

- `docs/discovery/slp-supervisor-lead-peer/MAP.md` — the closed map; cluster 4
  of its build order is this feature
- `docs/discovery/slp-supervisor-lead-peer/tickets/007-contract-status-original-request.md`
  — the grilling ticket that produced D1, D5 and the mechanism half
- `docs/history/research/slp-contract-request-surfaces.md` — the advisor-tier
  surface digest every code anchor above comes from
- `docs/knowledge/areas/decision-memory/overview.md` — how the decision log,
  its relations and its triggers already behave

## Outstanding Questions

### Resolve Before Planning

None. The map records no unresolved item for this cluster.

### Deferred To Planning

- [ ] What makes a cell "test-writing" for D4's refusal? — the cell record has
      no test flag today, so planning must find the honest signal and say
      plainly what it does NOT catch
- [ ] Does the tripwire live at dispatch prepare or at claim time (D3)? — both
      are named; reading the two doors answers which one every dispatch really
      passes
- [ ] How does a dispatch resolve WHICH intent anchor to read (D6)? — the
      anchor is stored per feature key, and a gather or advisor dispatch may
      carry no feature
- [ ] Does an absent anchor render byte-identically, per template? — the
      pattern exists twice; the proof obligation is the same one both times

## Deferred Ideas

- A reverse index from contract name to trigger — the digest names its absence
  as a gap; nothing in D1 to D6 needs it, and building one would be the second
  registry D1 refuses.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and deferred-to-planning
questions. Planning's Gate 2 shape stage and reviewing use locked decisions for
coverage and UAT.
