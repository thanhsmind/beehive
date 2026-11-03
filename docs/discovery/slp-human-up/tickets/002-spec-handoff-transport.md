---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

What is the concrete transport for a cross-repo spec drop (6f039742,
5bed1c01)? Facts needed from the waggledance and bee codebases: (a) exact
semantics of `waggledance_dispatch`/`waggledance_await`/`waggledance_runs` —
opt-in surface, busy-pane behavior, delivery guarantee, payload shape;
(b) how a dropped file + backlog item becomes work bee's route/claim actually
picks up in the receiving repo (backlog add? PBI? mailbox letter?); (c) where
the correlation id and provenance line live on each side without cross-repo
storage; (d) what herdr's dispatch role needs so a fresh spec item qualifies
as "safe backlog work" it may ignite.

## Answer

Advisor-tier research (dispatch c938e6d0, 2026-08-29): the transport exists
end to end with ZERO new plumbing.

- (a) `waggledance_dispatch` is double-gated (global terminal switch + per-
  project `orchestration_enabled`, default OFF; refusal names the remedy).
  Payload = `project` + free-text `task` + exactly one of `preset` (spawns a
  fresh pane — preferred, dodges busy panes) or `pane_id` (same-project only).
  Busy panes refuse fail-closed (`Working`/`Blocked`; snapshot failure =
  unverifiable = refuse). Delivery is at-most-once; a successful send persists
  a durable Run row (id, task, project, pane, timestamps) in waggledance's
  SQLite; `waggledance_await` clamps to 60s and returns
  working/done/blocked/timeout + transcript delta; `waggledance_runs` lists
  run rows durably.
- (b) Best bee intake: `bee backlog pbi add --id <corr-id> --title <story>
  --cos "from <repo>@<commit>: <CoS>" --status proposed`. `backlog add` rows
  are findings excluded from the PBI fold; `backlog propose` takes no --id;
  the mailbox is an outbox (bee→human), not an intake. First-add-wins on the
  requested id makes redelivery idempotent: duplicate `pbi add --id` refuses
  deterministically, so the sender may re-dispatch on timeout safely.
- (c) Correlation id + provenance, no new storage: sender side = first task
  line `spec-drop <corr-id> from <repo>@<commit>` (durably queryable via the
  run row); receiver side = the corr-id IS the PBI id, provenance rides
  `--cos` verbatim in `.bee/backlog.jsonl`.
- (d) Herding dispatch requires: enable interlock, feature slug +
  `docs/history/<slug>/CONTEXT.md`, status exactly `in-flight`, no worktree
  grant, zero cells, two-key lane safety, overlap ranking. A fresh spec-drop
  PBI (proposed, no CONTEXT.md) is NOT ignitable today — by design. The glue
  is workflow, not code: bee-shaping's Qualify pass triages new proposed
  PBIs unattended, locks CONTEXT.md, flips to in-flight; then herding's
  conditions hold. High-risk spec content parks at any confidence — the
  right posture for foreign-origin work.

Decision logged: see MAP.md (spec-drop transport). Ticket 003 unblocked.
