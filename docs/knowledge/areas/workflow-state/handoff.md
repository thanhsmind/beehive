---
type: bee.area
title: "Workflow State — the two-kind handoff, its per-workflow mailbox, and pulling the next approved unit"
description: "How a finished task hands itself to a fresh session through a guarded planned-next record adopted only at a real fresh-session boundary, how that record now lives in a per-workflow mailbox scoped by target role so one workflow pausing never blocks another, how the legacy single HANDOFF.json file survives only as a projection with exactly one sanctioned writer set enforced by a grep audit, and how a session out of work pulls its next unit without ever widening the authority a human granted."
timestamp: 2026-07-25
bee:
  id: workflow-state-handoff
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [fresh-session-handoff D1-D4 (docs/history/fresh-session-handoff/CONTEXT.md — auto-resume authority exists only at the fresh-session boundary; the puller never widens authority), no-clear-stop D1 (planned-next is written only at real session exit; the orchestrator never stops mid-flow to offer /clear), "multisession-native D5 (slice 3, issue #56 3.7/mục 10: per-workflow handoff mailboxes replace the single-file HANDOFF.json as the durable record; the legacy file becomes a projection of the newest open mailbox record — docs/history/multisession-native/CONTEXT.md)", "multisession-native D5 amendment (msn-24, advisor-digest-slice5 condition E: rebuildHandoffProjection is reclassified as the sole sanctioned writer of the legacy .bee/HANDOFF.json for any repo with a live workflow record; writeHandoff/adoptHandoff are retained one more release as the C1 no-workflow-records fallback with dated deprecation notes naming the removal condition; a grep-audit test enforces the exact production writer set)"]
  sources: ["fresh-session-handoff S1 cells fsh-1/fsh-2 (traces in .bee/cells/, reports docs/history/fresh-session-handoff/reports/, 2026-07-13)", fresh-session-handoff validation-s4 C10/C11 (docs/history/fresh-session-handoff/reports/validation-s4.md), "GH #20 live-owner lane guard (trace .bee/cells/cnlg-1.json)", "docs/specs/workflow-state.md#B15", "docs/specs/workflow-state.md#B16", "docs/specs/workflow-state.md#R19", "docs/specs/workflow-state.md#R20", "docs/specs/workflow-state.md#R21", "docs/specs/workflow-state.md#P16", "multisession-native cell multisession-native-15 (per-workflow handoff mailboxes; trace .bee/cells/multisession-native-15.json, commit 60d0419, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice3.md condition G)", "multisession-native cell multisession-native-24 (rebuildHandoffProjection reclassified as sole sanctioned writer; writeHandoff/adoptHandoff dated deprecation notes; grep-audit test in test_state.mjs enforcing the exact writer set; trace .bee/cells/multisession-native-24.json, commit cee2d5f, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice5.md condition E)"]
  authoritative_for: "workflow-state: the two-kind session handoff, its per-workflow mailbox, the legacy projection's sanctioned writer set, and the cross-lane work puller"
---

# Workflow State — the two-kind handoff, its per-workflow mailbox, and pulling the next approved unit

A baton is only safe if the runner picking it up is genuinely a new runner. Both
halves of this concept turn on that: a handoff record is adopted automatically
ONLY at a fresh-session boundary and is otherwise surfaced and waited on, and the
work puller that feeds the next session selects only from lanes a human already
approved — never widening authority to keep itself busy.

## Behaviors & Operations

**B15 — A finished task hands itself to a fresh session through the two-kind
handoff.** Trigger: a session finishes a unit (finished = capped with green
verification) and a next unit has been claimed for it. What happens: the
planned-next handoff is written through its guarded verb — the verb itself
refuses (typed, zero mutation) when the previous unit is unfinished, its
verification did not pass, or the carried claim is absent or not owned by the
writer. The write happens only at **real session exit** (no-clear-stop D1):
while work remains, the orchestrator continues in-session and never stops
mid-flow to offer the clear command — the handoff is not a mid-session pause
gesture. When it is written, the owner then starts a fresh session (types the
clear command); adoption machinery below is unchanged by this. At
session start, only on the **fresh-session boundaries** (a cleared or newly
started session) does the runtime adopt the carried claim for the new session
— ownership transfers through the claim's gate, the record staying present
throughout, so a third session racing for the same unit loses with a typed
failure — and the new session's opening context replaces the wait-block with
a start-now instruction naming the adopted unit, its verification command,
and its lane. On a **resumed or memory-compacted** session the runtime never
adopts: the handoff stays on disk untouched and is shown as pending — those
events are not a fresh session, and auto-resume authority exists only at the
fresh-session boundary (D1). A pause handoff — and every legacy record
without a kind — renders exactly today's present-and-wait block. A failed
adoption never fabricates a start-now instruction; the reason is shown and
the wait presentation applies. Clearing the record happens after adoption and
recovers idempotently if interrupted (never claimed as atomic across files).
Since multisession-native D5 (slice 3), the durable record behind this
handoff lives in a per-workflow mailbox rather than a single shared file —
see below — with every guarantee in this paragraph unchanged: same kind
contract, same preconditions, same fresh-session-only adoption rule.

**B16 — A session out of work pulls the next approved unit itself.** Trigger:
a session asks for its next unit (typically right before writing a
planned-next handoff). What happens, in order: expired claims are swept first
— a unit claimed by a dead session (lifetime expired AND heartbeat stale) is
reclaimed in the same pass, so "no work" is never reported while such a unit
exists; then selection prefers the session's own lane's ready units (its
execution gate approved), then ready units of OTHER lanes **whose execution
gate a human approved** — an unapproved lane is never touched even when its
units are the only ready ones (the puller never widens authority, D2); a lane
**actively owned by another live session** — some other session is bound to it
and its heartbeat is still fresh (the same staleness rule that governs claim
reclaim) — is never pulled from either, even when its units are the only ready
ones anywhere: the planner keeps the work it just planned (GH #20), and only a
stale-heartbeat owner leaves its lane reclaimable again (steal-after-death is
preserved). The default pipeline record carries no session binding, so this
ownership guard applies to lanes only, deliberately; the acting session's own
binding never blocks its own pull, and explicitly claiming a named unit stays
possible (orchestrator assignment is intent, the guard governs only automatic
selection). Units whose files overlap another session's live holds are skipped;
cross-lane order follows the product backlog's ranking (by the lane's feature
row), falling back to lane age. Nothing qualifies → the typed answer "no
approved work left", and the session stops honestly. Claiming the chosen unit is
crash-safe: the cross-session claim file is taken first, the work record
second, and a failure of the second releases the first (no orphaned claim).

**The durable handoff record lives in a per-workflow mailbox, scoped by
target role (multisession-native D5, slice 3, msn-15).** Trigger: writing or
adopting a handoff in a repo that has a live workflow record for the target
feature. What happens: each workflow gets its own mailbox at
`.bee/runtime/handoffs/<workflow-id>/<seq>.json`, a sequence of append-only
records rather than one shared file. Writing a NEW record for a given
`(workflow, target_role)` pair marks the previous OPEN record for that SAME
pair `'cleared'` in the same write — the mailbox's answer to the single
legacy file's implicit overwrite-on-rewrite — so a caller who never names a
role keeps exactly the old "one active handoff" behavior. A DIFFERENT
`target_role` (a reviewer's handoff alongside an implementer's, say) gets its
own independent slot and is never touched by that auto-clear — this is what
lets one workflow's own handoff pause without ever blocking or clobbering a
different role's, or a different workflow's, handoff. Every record — cleared
or not — stays on disk under its own sequence number for audit history,
never deleted. `bee.mjs`'s `state handoff write/show/adopt` verbs resolve
which workflow they target — an explicit `--lane`, the calling session's own
bound lane, or the default record's live workflow — and route to that
workflow's mailbox; a repo with no workflow records anywhere keeps using the
untouched legacy single-file path (the same C1 no-workflow fallback
`workflow-records-and-projections.md` documents for state/lane projections).
Adopting a mailbox record transfers the carried claim through the SAME
`adoptClaim` this area's claim fencing (`claims-and-ownership.md`) already
bumps, then clears the mailbox record idempotently — a crash between
transfer and clear leaves the record `'adopted'` (not yet `'cleared'`), and
the next adopt call for that mailbox+role self-heals by finding that same
`'adopted'` record and skipping straight to the clear step, never re-running
`adoptClaim` and never double-bumping the claim's fence epoch on recovery.
The legacy single `HANDOFF.json` file still exists, but only as a projection
(`rebuildHandoffProjection`, registered in `state-projection.mjs`'s
`rebuildAllProjections` — the same aggregator `workflow-records-and-
projections.md` describes for state/lane) mirroring the newest open mailbox
handoff across every workflow, so every byte-identical legacy reader
(`bee-session-close.mjs`, `lib/compaction.mjs`, `lib/inject.mjs`, this repo's
own AGENTS.md startup step 5) keeps working unedited. **Documented compat
limitation:** two simultaneously paused workflows only ever show ONE of them
in that single legacy file at a time — never both — until a later slice
retires the file outright; a reader that needs the full picture across every
paused workflow must read the mailboxes directly, not the legacy projection.
What each actor observes: a reviewer's handoff and an implementer's handoff
on the same workflow never clobber each other; one workflow pausing never
blocks or leaks into a different workflow's own mailbox (proven directly,
not merely asserted); a repo that has never touched slice 3 sees no behavior
change at all.

**The legacy `.bee/HANDOFF.json` has exactly one sanctioned writer set, enforced
by a grep audit rather than trusted as documentation (multisession-native D5
amendment, msn-24, advisor-digest-slice5 condition E).** `rebuildHandoffProjection`
is documented in its own header as the ONLY writer of the legacy file for any repo
with at least one workflow record: every mailbox write/adopt call in `bee.mjs`
invokes it immediately afterward to keep the projection current, and no other
production path writes that file on the live-workflow side. `writeHandoff`/
`adoptHandoff` (`state.mjs`) — the pre-mailbox direct-file writers — are retained
for exactly ONE more release as the C1 no-workflow-records legacy fallback:
`handleStateHandoffWrite`/`handleStateHandoffAdopt` in `bee.mjs` call them ONLY on
the branch where resolving the target workflow id returns null (a repo with zero
workflow records anywhere — the same C1 fallback `workflow-records-and-
projections.md` documents for state/lane projections). Each carries a dated
deprecation note naming its removal condition: once no supported repo can lack
workflow records (every `startFeature` call already seeds one via
`seedLegacyWorkflows` — this fallback exists only for a repo that predates
multisession-native or was never onboarded through it), both retire together. A
grep-audit test (`test_state.mjs`) makes this a structural guarantee, not a
convention to remember: it scans every `.mjs` file under `lib/` plus `bee.mjs` for
the two mutation primitives every legal writer uses
(`writeJsonAtomic(handoffPath(...))` / `fs.rmSync(handoffPath(...))`), resolves each
hit's enclosing function name, and asserts the exact production writer set is
`{rebuildHandoffProjection, writeHandoff, adoptHandoff}` — a fourth writer added
anywhere in the codebase fails this test by name, and the audit itself asserts it
found at least one hit so a broken regex can never silently pass the sweep. What
each actor observes: the legacy file's contents are always traceable to one of
exactly three functions, never a stray direct write; a repo with live workflow
records sees the projection stay current automatically, and a repo with none keeps
using the untouched pre-mailbox path.

## Business Rules

- R19 — A planned-next handoff's preconditions live in its verb, never in
  prose: finished previous unit with passing verification and a carried,
  writer-owned claim, or the write refuses with zero mutation
  (fresh-session-handoff D1).
- R20 — Auto-resume authority exists only at the fresh-session boundary
  (cleared or newly started session); resumed and memory-compacted sessions
  never adopt a handoff, and a kindless record is pause everywhere
  (fresh-session-handoff D1; validation-s4 C11).
- R21 — The work puller never widens authority: cross-lane pull selects only
  from lanes whose execution gate a human approved, and the "no approved work
  left" stop may only be answered after the stale-claim sweep ran
  (fresh-session-handoff D2; validation-s4 C10).
- R73 — A handoff's durable record is a per-workflow mailbox scoped by
  `target_role`; writing a new record auto-clears only the prior OPEN record
  for the SAME `(workflow, target_role)` pair, never a different role's or a
  different workflow's; the legacy `HANDOFF.json` is a projection of the
  newest open mailbox record across every workflow, and shows at most one
  workflow's handoff at a time until a later slice retires the file
  (multisession-native D5, msn-15).
- R77 — The legacy `.bee/HANDOFF.json`'s production writer set is exactly
  `{rebuildHandoffProjection, writeHandoff C1 fallback, adoptHandoff C1
  fallback}`, enforced by a grep-audit test rather than merely documented; the
  two fallback writers fire only when no workflow record resolves and each
  carries a dated deprecation note naming its removal condition
  (multisession-native D5 amendment, msn-24, advisor-digest-slice5 condition
  E).

## Pointers (implementation)

- Fresh-session flow (B15/B16): two-kind handoff (`readHandoff` normalization,
  strict `writeHandoff`, `adoptHandoff`) in `packages/bee/lib/state.mjs`;
  CLI `state.handoff.write/adopt/show`; source-gated adoption + session
  registration in `packages/bee/hooks/bee-session-init.mjs`; pure kind-branch rendering in
  `lib/inject.mjs` (`handoffOutcome` param); `claimNextCell`/`claimCellCrossSession`
  in `lib/cells.mjs` + `featureBacklogRank` in `lib/backlog.mjs`; CLI
  `cells.claim-next` (the production `sweepExpiredClaims` trigger); live-owner
  lane guard in the cross-lane fallback pool (`listSessionRecords` in
  `lib/claims.mjs` + `heartbeatStale` membership check in `claimNextCell`,
  GH #20, trace `.bee/cells/cnlg-1.json`). Evidence:
  traces `.bee/cells/fsh-{9,10,11}.json`, commits 79e800e, d419e0e, 9931fc6;
  `docs/history/fresh-session-handoff/reports/validation-s4.md` (incl. the
  orchestrator's retro-RED probe for fsh-11).
- Per-workflow mailbox (multisession-native D5, msn-15): `writeMailboxHandoff`/
  `adoptMailboxHandoff`/`listHandoffMailbox`/`newestOpenHandoffMailboxRecord`
  in `packages/bee/lib/state.mjs`; `state handoff write/show/adopt`
  resolve their target workflow (`--lane`, the calling session's bound lane,
  or the default record's live workflow) in `bee.mjs`; `rebuildHandoffProjection`
  in `state-projection.mjs`, registered in `rebuildAllProjections` (see
  `workflow-records-and-projections.md` for that aggregator). Proves invariant
  9: one paused workflow never blocks or leaks into another's own mailbox.
  Evidence: trace `.bee/cells/multisession-native-15.json`, commit 60d0419;
  advisor digest
  `docs/history/multisession-native/reports/advisor-digest-slice3.md`
  condition G.
- Sanctioned-writer reclassification (R77, msn-24): dated deprecation notes on
  `writeHandoff`/`adoptHandoff` and on `rebuildHandoffProjection`'s own header
  in `state.mjs`/`state-projection.mjs`; the grep-audit test in
  `packages/bee/tests/test_state.mjs`. Evidence: trace
  `.bee/cells/multisession-native-24.json`, commit cee2d5f; advisor digest
  `docs/history/multisession-native/reports/advisor-digest-slice5.md`
  condition E.
