# leader-check-door

## Problem

The leader completeness check has no teeth. `leader-completeness-check`
(2026-09-05, decision f5e3c084) shipped it as instruction only — its own
`docs/history/leader-completeness-check/CONTEXT.md:22-26` scopes itself
`Instructional completeness check only:` … `- No Rust or schema
changes`. A leader that never compares a capped cell's approved
requirements against actual artifacts is refused by nothing.

The two machine doors that exist do not close that gap:

- `judge-debt` (`verbs/drivers/close.rs:1565`) exists ONLY for a
  `standard`/`high-risk` closing route and counts only
  `behavior_change` cells. A `tiny`/`small` feature never grows the door
  at all.
- `bee close` and `bee worktree merge` read the worker's own recorded
  proof line. They check a string the worker wrote; they never ask
  whether the leader looked.

So at `tiny`/`small` — and, for non-`behavior_change` cells, at every
lane — "the leader checks again" is honour-system prose.

## Decision

Give the leader completeness check a recorded mark and two doors, on the
shape `dissent-debt` already established (one debt function reached by
both `bee close` and `bee worktree merge`).

- **D1 — The mark lives on the cell trace.** A new `trace.leader_check`
  array, append-only, one entry per recorded check. Same home as
  `trace.semantic_judge` and `trace.dissent`. The key is unused today
  (hat-facts-gaps swept every `trace.insert` site).
- **D2 — A new verb writes it**: `bee cells leader-check`. The leader
  runs it; no worker path writes this key. No existing verb can carry
  it: `cells update` refuses an unknown `trace` patch
  (`handlers_write.rs:366-378`), `cap`/`finish` are worker-run
  (`util.rs:73-74`), and `close` runs after merge under the default
  `uat_stop` (`src/uat.rs:44`).
- **D3 — The mark names artifacts, and the artifacts are checked.** The
  verb takes `--file <payload.json>`, the shape `judge-record` already
  uses. The payload answers every derived requirement with an artifact,
  and the verb VERIFIES the artifact rather than the syntax:
  - an artifact that looks like a repo path MUST exist on disk;
  - at least one answer per cell MUST name a path that appears in that
    cell's `trace.files_changed`.
  A box that can be ticked without looking is the defect this feature
  exists to remove, so a mark that names no real file is refused.
- **D3a — The requirement list is derived, and never empty.**
  `must_haves.truths` is optional below `standard`
  (`verbs/cells/validate.rs:319`), so the verb derives its list in
  order: `must_haves.truths` when non-empty; else one requirement per
  entry in `trace.files_changed`; else exactly one free requirement
  that must be answered non-blank. Never zero answers (the tick-without-
  looking hole at exactly the lanes this feature was asked for), never a
  permanent wall (a cell that can never be marked).
- **D4 — Every lane, every capped cell.** `AGENTS.md` and
  `bee-swarming` already say the check is "the routine step, every cap,
  every lane". The door matches that text — no `behavior_change`
  filter, no lane filter. It does NOT duplicate `judge-debt`: that door
  demands an independent judge dispatch for behaviour change; this one
  demands the leader's own artifact comparison. It applies to an inline
  `tiny` cell too, where the leader is also the worker: the value there
  is the forced re-read of one's own diff against the requirements, and
  the door is what makes that re-read non-optional.
- **D5 — Grandfather by a named stamp.** A module constant
  `LEADER_CHECK_DOOR_INTRODUCED_AT` lives beside the debt function in
  `verbs/cells/leader_check.rs`, copying `JUDGE_DOOR_INTRODUCED_AT`
  (`close.rs:443`) in shape and in fail-open posture: a missing or
  unparseable `trace.capped_at` reads as pre-door and is grandfathered
  (`close.rs:466-468`). Its value is the feature's merge date. Without
  it every already-closed feature in the repo would refuse to close.
  The `mistakes_debt` legacy skip on a cap with no `trace.report`
  (`close.rs:267`) is NOT copied — the `capped_at` cutoff subsumes it,
  and carrying two grandfather tests for one idea is the duplication
  this repo names as a defect.
- **D6 — Two doors, one debt function.** A `leader-check` door in
  `build_close_report_doors` plus a `WORKTREE_MERGE_LEADER_CHECK_DEBT`
  refusal in `worktree/phases.rs`, both reading one
  `feature_leader_check_debt` in `verbs/cells/leader_check.rs`. Exactly
  the `dissent-debt` shape (`close.rs:1654`, `phases.rs:252`). Both
  doors ship in the same slice: under the default `uat_stop: "close"`
  (`src/uat.rs:44`) merge runs BEFORE close, so a close-only door lets
  the branch land on main unchecked.
- **D6a — Ordering, named at both sites.** Push order in
  `build_close_report_doors`: after `dissent-debt` (`close.rs:1671`),
  before `advisor-nudge-debt` (`close.rs:1726`). Refusal-arm order:
  LAST among the cell-debt arms, after `mistakes` (`close.rs:2655`) and
  before the uat arm (`close.rs:2685`) — the specific debts surface
  first, and this broad one never masks them.
- **D7 — The named escape, the fifth of its shape**: a logged decision
  tagged `leader-check-deferral` naming the feature lifts the refusal
  at BOTH doors. Mirrors `has_dissent_deferral_decision`,
  `has_judge_deferral_decision`, `has_capture_deferral_decision`,
  `has_uat_deferral_decision`.
- **D8 (SUPERSEDES the first draft's D8) — a `gap` verdict records, and
  never mutates status.** The first draft had `--verdict gap` reopen the
  cell `capped` → `open`, copying `judge-record`. That is backwards
  here. Every debt reader filters `Some("capped")`
  (`close.rs:265,329,460`), and neither `bee close` nor
  `bee worktree merge` has any open-cell gate — so a reopened cell drops
  OUT of the debt scan and both doors go clear. Recording a gap would
  become the cheapest way past the door, which is the exact defect D3
  exists to remove. It also stranded an open cell on main when merge had
  already run, and it had no defined behaviour on an archived cell
  (`util.rs:283-294` refuses those outright).

  Instead: a `gap` entry is appended and the cell stays `capped`. The
  door counts a capped cell as debt unless its NEWEST `leader_check`
  entry reads `ok`. A gap therefore leaves the door RED — teeth without
  the reopen. The refusal names `bee cells reopen` as the road when the
  gap needs rework.

## Scope

In (7 product files):
- `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs` (new):
  the handler `run_leader_check`, `feature_leader_check_debt`,
  `has_leader_check_deferral_decision`, and the stamp constant. The
  handler lives HERE, not in `handlers_meta.rs` — `dissent.rs:83,309`
  is the cited precedent and it keeps its own handlers.
- `verbs/cells/mod.rs` — module declaration and exports.
- `verbs/cells/util.rs` — the dispatch match arm.
- `verbs/drivers/close.rs` — the door and its refusal arm, per D6a.
- `verbs/worktree/phases.rs` — the merge refusal, in the zero-mutation
  zone beside the dissent arm.
- `src/generated/registry_payload.json` — the command entry, hand-edited
  (decision 3358743e, `docs/decisions/index.md:2013`).
- Tests beside each.

Out:
- **`src/catalog.rs` is NOT touched.** The pinned count is over distinct
  flag NAMES (`catalog.rs:802-806`), and every flag this verb needs —
  `id`, `verdict`, `file`, `session-id`, `force-ownership`, `json` —
  is already in the 210-name vocabulary (verified by counting the
  payload). An unconditional bump would turn that test red.
- No change to `judge-debt`, `mistakes`, `scribing-debt`, or the proof
  line.
- No change to what a worker writes at `bee finish`. The worker caps its
  own cell before the leader ever sees it, so the door cannot sit there.
- No new report schema for workers.
- No batch form. One call per cell, matching `judge-record`. The typing
  cost on a many-cell feature is a known gap, recorded as backlog, not
  silently absorbed.

## Source

User decision, 2026-09-21: "A — Làm cửa có răng cho lane `small`".
Widened to every lane per D4 with the reason above; the user's ask is
the floor, not the ceiling.

Plan-step hat wave, 2026-09-21 (3 seats): D3a, D5's stamp, D6's
single-slice rule, D6a's ordering, D8's supersede, and the `catalog.rs`
removal all come from its findings. D3's artifact verification comes
from the user-impact seat's answer to "does a recorded line prove
verification, or only that someone typed".

Prior art cited, never reinterpreted:
- `docs/history/leader-completeness-check/CONTEXT.md` (decision
  f5e3c084-c91d-4dfa-825f-48175bc61fe5, 2026-09-05).
- decision 3358743e (worktree-reclaim D5) — `registry_payload.json` is
  hand-edited here and has no regen chain.
- decision a2affcba / 4b7aa303 (slp-dissent-stop-and-ask sd-4) — the
  two-door debt shape this copies.
- hardening-1-7-10 D7 (`handlers_meta.rs:277-283`) — why a reopen that
  skips `release_trace` is a defect, and part of why D8 no longer
  reopens at all.
