---
type: bee.area
title: "Workflow State — review sessions, the candidates ledger, and derived review status"
description: "The records behind user-invoked independent review: the candidate appended at every feature close, the session whose scope freezes at creation, and the coverage/staleness answer that is always derived from records plus real change history and never stored."
timestamp: 2026-07-22
bee:
  id: workflow-state-review-sessions
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [565e68d0-327f-404e-b49e-d1c61ba81bfd (independent review is user-invoked; a feature closes truthfully as unreviewed), a83a3613 (a conclusive repository answer outranks an auxiliary launch warning when review coverage is derived), c18ac30a (an approved review decision is refused while any P1 finding stands unresolved)]
  sources: ["review-on-demand cells review-od-1..3 (traces in .bee/cells/, reports docs/history/review-on-demand/reports/, 2026-07-12)", "codex-sandbox-baseline cell codex-sandbox-baseline-6 (status-first review history derivation, 2026-07-16)", "review-p1-teeth cell rp1-1 (approval refuses on an unresolved P1; trace .bee/cells/rp1-1.json, decision c18ac30a, 2026-08-04)", "docs/specs/workflow-state.md#B3", "docs/specs/workflow-state.md#B4", "docs/specs/workflow-state.md#B5", "docs/specs/workflow-state.md#B6", "docs/specs/workflow-state.md#R4", "docs/specs/workflow-state.md#R5", "docs/specs/workflow-state.md#R6", "docs/specs/workflow-state.md#R9", "docs/specs/workflow-state.md#R10", "docs/specs/workflow-state.md#R11", "docs/specs/workflow-state.md#R28", "docs/specs/workflow-state.md#E7", "docs/specs/workflow-state.md#E8", "docs/specs/workflow-state.md#E9", "docs/specs/workflow-state.md#P17"]
  authoritative_for: "workflow-state: review sessions, review candidates, and derived review status"
---

# Workflow State — review sessions, the candidates ledger, and derived review status

Independent review is never a stage the pipeline walks into by itself. It is a
user-invoked session over an immutable range, and everything this concept owns
follows from that: a candidate is *recorded* at close so nothing is forgotten, a
session's scope *freezes* at creation so what was approved can never drift, and
the answer to "is this reviewed?" is *derived* at read time so a stale approval
can never masquerade as a live one.

## Behaviors & Operations

**B3 — Feature close adds a review candidate.** When a feature finishes its
closing pass, one candidate entry is appended to the append-only ledger:
feature, range anchor at close, and the feature's lane. The lane is required —
the status surface uses it to warn prominently about high-risk work that has
not passed independent review. Observers see the candidate counted as
`unreviewed` immediately.

**B4 — Review session lifecycle.** A session is created only from an explicit
user request. Creation freezes the scope: included work in progress (open or
claimed) is automatically moved to the exclusions with the reason "in
progress" — never silently included; a pre-dispatch evidence check inspects
every included behavior-changing change for recorded completion evidence and,
on any gap, the creation fails with zero records written — review never
substitutes for missing verification. After creation, the baseline, head,
included, and excluded sets can never change; an attempted change is refused
and the record is left byte-identical. Reviewer manifest, findings,
user-acceptance items, and the decision (pending → blocked | approved) are
recorded onto the session as the review proceeds. A session id that already
exists cannot be created again.

**B5 — Coverage and staleness are derived, never stored.** Each candidate's
review status is computed at read time from the session records plus the
repository's actual change history. A candidate covered by an approved session
at its exact anchor reports `reviewed`; one newer change after that session's
head flips it to `review stale` while the session record itself stays
unchanged (the audit trail survives). When the change history cannot be
resolved (rewritten history, missing tooling), the answer degrades toward
honesty: `review stale` with a "range unresolvable" note when a covering
session exists, plain `unreviewed` when none does — the read path never fails.

**B6 — Status surfaces tell the review truth.** The session status summary
carries the candidate counts by derived status and any open sessions. A
feature that closed without review produces an informational completion line
("completed and verified; independent review not requested; N candidates
awaiting review") — not a warning, because closing unreviewed is the normal
truthful state. An unreviewed or stale high-risk candidate produces a
prominent warning that it has not passed independent review and that review
runs only on user request. The recommended-next-step line never proposes
starting a review by itself. A range already covered by an approved, unchanged
session is answered "reviewed (covered by that session)" so no second review
is dispatched for unchanged content.

**B7 — The review approval door: `approved` is refused while a P1 finding
stands unresolved.** Recording an `approved` decision first checks that
session's findings for any at P1 severity. If none stand open, the decision
is recorded as usual. If one or more stand open, the decision is refused —
the record stays byte-unchanged — and the refusal names every open P1, by
its id when the finding carries one, else by its position in the findings
list. A P1 counts as resolved only when the decision names it together with
the unit of work that fixed it; naming the P1 alone, with no fixing unit
attached, does not count, because that asserts the fix happened without
saying where. `blocked` and `pending` decisions pass through this check
untouched — recording where a review currently stands never needed a door.
The gate-bypass ladder's highest level is the one sanctioned way to lift
this door (see the bypass ladder in `gates.md`, R25).

## Business Rules

- R4 — Full independent review starts only after an explicit user request
  (rule: agents-review-user-invoked);
  completing a cell, slice, or feature never spends reviewer tokens by itself,
  and a merge/ship/release request is answered with the review status plus one
  explicit question, never a silent review dispatch (decision
  565e68d0-327f-404e-b49e-d1c61ba81bfd).
- R5 — Verification and review are separate: verification evidence remains
  mandatory for completion, while a completed feature closes truthfully as
  unreviewed and joins a later user-selected review batch (decision
  565e68d0-327f-404e-b49e-d1c61ba81bfd).
- R6 — Review approval covers only the immutable change set inspected by that
  review session; later changes never inherit the earlier approval — they
  surface as an unreviewed delta and the overall status reads `review stale`
  (decision 565e68d0-327f-404e-b49e-d1c61ba81bfd).
- R9 — A review session's scope is frozen at creation; the pre-dispatch
  evidence check fails closed with zero records written, and in-progress work
  is excluded with a recorded reason, never silently included (decision
  565e68d0-327f-404e-b49e-d1c61ba81bfd; SPEC A6/A10).
- R10 — Review status is always derived from records plus actual change
  history, never stored; legacy features with no review record derive
  `unreviewed` — no session records are ever fabricated for history (decision
  565e68d0-327f-404e-b49e-d1c61ba81bfd; SPEC §11.3).
- R11 — The final human approval of a review (its Gate 3) exists only inside a
  review session; gate bypass never creates or approves one (decision
  565e68d0-327f-404e-b49e-d1c61ba81bfd; SPEC R8).
- R28 — When review status is derived from change history, a conclusive
  repository answer remains authoritative even if the execution environment
  also attaches an auxiliary launch warning. Only an inconclusive answer
  degrades the result to `review stale` with an unresolvable-range note
  (codex-sandbox-baseline-6; decision a83a3613).
- R29 — Recording an `approved` decision is refused while any P1 finding on
  that session stands unresolved; the refusal names every open P1 (by id, or
  by its position in the findings list) and leaves the record untouched. A
  P1 is resolved only when the decision names it together with the unit of
  work that fixed it — naming with no fixing unit does not count. `blocked`
  and `pending` decisions are unaffected. The gate-bypass ladder's highest
  level is the sole sanctioned door past this check (`gates.md` R25) (rp1-1;
  decision c18ac30a).

## Edge Cases Settled

- Review coverage edge cases: exact-anchor coverage → `reviewed`; one newer
  change → `review stale` with the session record byte-unchanged; rewritten
  history / unknown anchor → `review stale` + "range unresolvable" (with a
  covering session) or `unreviewed` (without); change-history tooling absent →
  the status surface still renders, degraded, exit-clean.
- A corrupt review record: read paths skip it with a warning; write verbs
  refuse loudly with the record untouched (same strict-read discipline as the
  workflow record itself).
- The old "past reviewing but the review gate still pending" staleness warning is
  retired: closing through scribing/compounding without a review session is
  the normal state, reported informationally, never as drift. The
  unknown-phase warning is unchanged.

## Pointers (implementation)

- Review records: `.bee/reviews/<id>.json` (sessions) + `.bee/review-candidates.jsonl`
  (ledger), CLI `bee reviews` (create/list/show/record/candidate add/
  candidates/status), lib `packages/bee/lib/reviews.mjs`
  (`deriveCandidateStatus`, `readReviewStrict`; byte-mirrored to `.bee/bin/`).
  Status surface: `review` block in `the bee binary` (`status` group).
  Coverage derivation uses status-first `git merge-base --is-ancestor` +
  `git rev-list --count`: a concrete Git answer wins over an attached auxiliary
  launch warning, while missing/inconclusive output yields `review stale`.
  Tests: review-od checks in `packages/bee/tests/test_lib.mjs`
  (including codex-sandbox-baseline-6 coverage). Evidence: commits cc1c34d,
  e4f51a2, da2e165; traces
  `.bee/cells/review-od-{1,2,3}.json`; acceptance map
  `docs/history/review-on-demand/reports/uat-scenarios.md`.
- P1 approval door (B7/R29): `unresolved_p1_refusal` in
  `packages/bee-rs/crates/bee/src/verbs/reviews.rs`, checked only on the
  `decision`-kind record write when `status == "approved"`. A resolution is
  carried on `decision.p1_resolutions`, an array of `{finding, cell}` pairs;
  a pair only counts when `cell` is a non-empty string. Evidence: trace
  `.bee/cells/rp1-1.json`, decision c18ac30a.
