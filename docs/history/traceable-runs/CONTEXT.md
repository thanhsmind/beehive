# Traceable Runs — Context

**Feature slug:** traceable-runs
**Date:** 2026-08-14
**Shaping session:** complete
**Scope:** Deep
**Domain types:** RUN | READ | ORGANIZE

## Feature Boundary

Every request that writes a file produces a durable, dashboard-readable run
record: a shaped brief written before any source edit, an explicit approval
moment with its own persisted state, named statuses on the cell / feature /
gate that a reader can display without re-deriving them, and one claimable
queue holding the work this run deferred. It ends at the data and the CLI that
reads it — the dashboard itself is not built here.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every file-touching request writes a shaped brief and enters an explicit `awaiting-approval` state on its feature/workflow record BEFORE any source edit — at every lane, `tiny` and `docs` included. | Today `tiny`/`small` route straight to `bee-planning` (`skills/bee-hive/references/routing-and-contracts.md:33`), so `bee-shaping`'s Lock never runs and no `CONTEXT.md` is written; the ceremony table (`routing-and-contracts.md:120-126`) never lists Gate 1 as a human stop for those lanes. The run leaves no feature context and no visible approval moment. |
| D2 | `gate_bypass` stops deciding whether the record EXISTS. It decides only whether the run halts at the approval or is auto-approved. An auto-approval writes the same record with `actor: "auto"` plus the bypass level and reason. | Keeps tiny work fast while making the approval traceable either way. A bypassed run must be as readable after the fact as a stopped one. |
| D3 | A gate stops being a bare boolean and becomes a record: `state` (`pending` \| `approved` \| `rejected`), `actor` (`user` \| `auto`), and a timestamp. The `pending` state is persisted, not derived. | Today gates are plain bools in `state.json.approved_gates` plus `{approved, approved_for_plan_rev}` on the workflow record; the only trace of a refusal is a `gate_revoked_at` timestamp (`set_gate.rs:690-698`). Nothing anywhere can express "awaiting approval", so it cannot survive a restart or be read by a dashboard. |
| D4 | Cell and feature/workflow each get a real, persisted status vocabulary covering the states their lifecycle already has but cannot name — including an explicit waiting state. Statuses are stored, not computed at read time. | User chose the full option over the cheap derived rollup, so a dashboard reads one field and "waiting" survives a restart. Cell status today is a plain string `open\|claimed\|capped\|blocked\|dropped`; workflow status is `active\|paused\|closed` (`workflow_store/mod.rs:97`). |
| D5 | Deferred capture, scribing, review, and promote-proposal work all become records in ONE claimable queue. Each record carries enough payload to be executed by an agent that was absent when it was queued — feature, cells, areas, files, reason — plus a claim/lease so two parallel agents never take the same item. | Only capture stubs and review candidates are real stores today; scribing debt and unapplied promote proposals are derived scans (`state_group/ledger.rs`, `status_full/mod.rs:229-272`). A derived scan has nothing to claim and no payload, so it cannot be handed to a parallel agent — which is the capability being asked for. |
| D6 | The mandatory flow is scoped to requests that write a file — code AND docs. A pure question that changes nothing on disk creates no record. | Mechanically checkable at the write-guard hook, which already observes every write, and it keeps the store from filling with rows for conversation. |
| D7 | The dashboard is out of scope. This feature delivers the persisted data and the CLI/JSON surface that a dashboard would read. | User said the dashboard comes later; building it here would couple the data shape to one unbuilt consumer. |

### Agent's Discretion

Planning owns: the concrete status value names and where each is stored; whether
the unified queue is a new store or an extension of `.bee/capture-queue.jsonl`;
the lease duration and reclaim rule; the migration path for existing boolean
gates and existing cell records; the slice order.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Run | One request that writes at least one file, from the moment it is received to the moment its work is capped. The unit a dashboard row represents. |
| Brief | The shaped understanding written before any source edit — what was asked, what was found, what will be done. `CONTEXT.md` at full lanes; a shorter form at `tiny`/`docs`. Not the same as `implement-plan.md`. |
| Awaiting approval | A persisted state on the feature/workflow record meaning the brief is written and the run has not been approved yet. Distinct from "gate not yet approved", which today is indistinguishable from "gate never asked". |
| Deferred item | One unit of postponed capture, scribing, review, or promote-proposal work, carrying enough payload to be executed later by an agent with no memory of the run that queued it. |
| Claim / lease | The mechanism by which exactly one agent takes a deferred item, with a bounded hold that returns the item if the agent dies. |

## Specific Ideas And References

- The user's stated goal: "Every run is traceable" — a dashboard must always be
  able to answer "where is this task right now", including the answer
  "waiting for the user to approve".
- Lanes stay as they are. A small task still earns no detailed plan (D1 adds a
  brief and an approval record, not plan ceremony).
- Deferred work must be executable "by a separate agent running in parallel" —
  this is the acceptance bar for D5's payload completeness.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `.bee/capture-queue.jsonl` + `packages/bee-rs/crates/bee/src/verbs/capture.rs` — an
  append-only stub/flush event log; the closest existing shape to D5's queue.
  Pending = a stub with no later `flush` event naming its id (`pending_stubs`, `capture.rs:57-91`).
- `.bee/backlog.jsonl` + `verbs/backlog.rs` — an event-sourced store with an
  explicit five-value status set (`PBI_STATUSES`, `backlog.rs:63`) and a named
  store lock (`backlog.rs:437-490`). The pattern D4/D5 should follow.
- `.bee/review-candidates.jsonl` + `verbs/reviews.rs:1177-1284` — a real record log
  whose status is re-derived per call (`derive_candidate_status`, `reviews.rs:686-753`)
  against git and the review-session store. Shows both the record shape to keep
  and the derivation to replace.

### Established Patterns

- Record wins, projection is rebuildable — `.bee/state.json` and `.bee/lanes/*.json`
  are mechanically rebuilt from the workflow record (R65,
  `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`). New
  status fields belong on the record, with the projection derived from it.
- Per-record locking under one global order `workflow:<id>` → `state` → `lane:<feature>`
  (R79). Any new store gets its own named lock, never a lock scoped to a code path
  (`docs/knowledge/patterns/20260727-a-lock-scoped-to-the-wrong-record-buys-nothing.md`).
- Both writers of a shared record must lock, or the race merely moves
  (`docs/knowledge/patterns/20260724-a-lost-update-race-closes-only-when-both-writers-lock.md`) —
  directly relevant to D5's claim/lease.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` — the gate write path
  D3 changes; `--merge` currently flips `shape` and `execution` together (line 613).
- `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs` — `STATUS_VALUES`
  (`workflow_store/mod.rs:97`), `default_gate_entry` (`record.rs:69-74`), the schema D3/D4 extend.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs`, `handlers_close.rs` —
  every cell status transition D4 must cover.
- `packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:229-272` and
  `state_group/ledger.rs:174-200` — the two derived scans D5 replaces with queue records.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/` — the enforcement point for D6's
  file-touching boundary.
- `skills/bee-hive/references/routing-and-contracts.md` (routing table line 33,
  ceremony table lines 120-126) and `skills/bee-shaping/SKILL.md` — the doctrine
  D1 changes.

## Canonical References

- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md` — the
  workflow record schema, its projections, and the gate model D3/D4 modify.
- `docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md` — the
  lock primitive D5's claim/lease must build on.
- `skills/bee-hive/references/gates-and-delegation.md` ("Gate bypass mode") — the
  bypass level table D2 changes the meaning of.
- `.bee/decisions.jsonl` — D1-D4 logged 2026-08-14 during this shaping session.

## Outstanding Questions

### Resolve Before Planning

None. All four gray areas were answered by the user on 2026-08-14.

### Deferred To Planning

- [ ] Do existing repos migrate, or do old boolean gates and status strings stay
  readable as a legacy shape? — answered by auditing every reader of
  `approved_gates` and `cell.status` and counting the call sites.
- [ ] Does the unified queue absorb `.bee/capture-queue.jsonl` and
  `.bee/review-candidates.jsonl`, or sit beside them? — answered by checking
  whether any consumer outside bee reads those two files.
- [ ] What does `tiny`/`docs` write as its brief when no `CONTEXT.md` is warranted,
  and where does it live? — answered by choosing between a short `CONTEXT.md` and
  a new brief record on the workflow.
- [ ] How is the run's approval state kept honest when a session dies mid-approval?
  — answered by reusing the existing stale-claim sweep or adding a lease.

## Deferred Ideas

- The dashboard itself (D7) — out of scope; this feature delivers its data source.
- Retro-filling run records for already-closed features — no value without the
  dashboard, and the evidence to fill them accurately no longer exists.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
