---
area: workflow-state
updated: 2026-07-22
migrated_to: docs/knowledge/areas/workflow-state/
---

# Workflow State (phases, gates, feature lifecycle) (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/workflow-state/`](../knowledge/areas/workflow-state/index.md)
(okf-foundation D20/D29/D30/D37). It was the last area of the migration and by far the
largest — 1464 lines, 140 anchors — so it is also the only one migrated across SEVERAL
cells (okf-migration-f2 F10) rather than one commit.

Fifteen concepts, split by TOPIC rather than the old spec's headings. `overview.md` owns the
frame every other concept sits inside — purpose, the entry points, the complete Data Dictionary,
and who may write which part of the record (it claims no numbered anchor; those four sections
carry none); `gates.md` owns the guarded doors of a feature's life — the all-or-nothing start,
the closed phase vocabulary, the high-risk execution consult precondition, phase-owned routing
mutation, the gate-bypass ladder, and the three-step closing tail; `review-sessions.md` owns the
user-invoked review records, the append-only candidates ledger and derived review status;
`dispatch.md` owns the unified command entry point and its published catalog;
`advisor-consult.md` owns the stuck worker's adviser consult, its budget, the cost pattern behind
it, and adviser configuration tolerance; `cells-authoring-and-revision.md` owns how a unit of
work is authored, which plan fields may be revised afterwards, and the plan document frozen at
its gate; `cells-scheduling.md` owns the computed dispatch schedule and the dependency cycle
refused at the write; `cells-attempt-budgets.md` owns the append-only attempt history, the
lifetime budgets enforced at the claim door, and the audited reset; `cells-completion-judge-and-archive.md`
owns the completion teeth, the structured judge verdict and the journaled archive transaction;
`handoff.md` owns the two-kind handoff and the cross-lane work puller; `recovery.md` owns
crash-candidate detection and transcript mining; `claims-and-ownership.md` owns the single-winner
claim primitive, typed contention refusals and the claimed-unit ownership guard;
`sessions-lanes-and-identity.md` owns session identity, per-feature lanes and the renewing
heartbeat; `holds-and-the-coordination-lock.md` owns cross-session file holds and the shared-store
lock; and `worktree-isolation.md` owns the opt-in isolated-worktree dispatch and its transactional
merge-back.

This path stays alive as a pointer stub — it is never deleted in this feature (D20) — and the
anchor map below sends every numbered anchor the old spec exposed to the concept that now owns it,
so existing citations keep resolving. Coverage is machine-checked by
`scripts/okf_migrate.mjs --check workflow-state` in the verify chain (D35), against the pinned
pre-migration blob `506fef9` (140 anchors — 37 B / 58 R / 25 E / 20 P — 7 unparsed blocks —
okf-migration-f2 F8/F9), which was itself REPAIRED from the provenance blob `ed1644c` at
`df3072d:docs/specs/workflow-state.md` before pinning; see below.

## The three duplicate ids, and why one triple now reads `R19a`/`R20a`/`R21a`

The pre-migration source carried the rule ids **`R19`, `R20` AND `R21` twice each**, inside one
`## Business Rules` section. The first family (`:891-902`) is the fresh-session-handoff triple —
planned-next preconditions live in the verb, auto-resume authority exists only at the
fresh-session boundary, the work puller never widens authority (fresh-session-handoff D1/D2,
validation-s4 C10/C11). The second family (`:916-930`) is the chain-integrity triple — the
learning-capture phase is never settable, recording a knowledge sync demands executed work, the
terminal state demands learning capture plus zero spec debt (chain-integrity
D1-REVISED/D2/D3/D4). Six genuinely distinct rules, three collisions, none of them one rule
stated twice.

Because anchors are keyed by id, each FIRST member's text was silently overwritten by its second:
unmeasurable by the coverage gate's fidelity floor, permanently, and invisible to a set-equality
check as the pair's second member — 140 anchors carrying only 137 distinct ids, while every
declared count still added up. Neither rule of any pair may be dropped or merged, so the source
was **repaired before the migration pin was captured**: the second occurrence of each id in
document order — the chain-integrity family — was renumbered **`R19a`/`R20a`/`R21a`**, three
tokens on three lines, no other byte changed. That is the same repair `hook-runtime`'s `R14`
received, and the same reason.

**Reading an old citation.** Both families carried zero live citations in `skills/`, `scripts/`,
`hooks/`, `.bee/bin/`, `AGENTS.md` or `docs/specs/` when the repair was made, so no reference was
churned whichever side moved; the tie broke on document order. The only surviving external
mention — `docs/history/fresh-session-handoff/reports/validation-s5.md`, citing
`workflow-state.md B15/B16/R19-R21` — means the FIRST family, which kept its ids. So:

- a citation of `R19` / `R20` / `R21` that means **planned-next preconditions / fresh-session
  auto-resume authority / the work puller** resolves unchanged, on its own row below → `handoff.md`;
- a citation of `R19` / `R20` / `R21` that means **the learning-capture phase / the knowledge-sync
  precondition / the terminal state and spec debt** is the pre-repair id of the chain-integrity
  family and now reads `R19a` / `R20a` / `R21a`, on its own row below → `gates.md`.

The suffix here is a **disambiguation**, not a refinement of `R19` the way `R8a`/`R8b` refine `R8`
elsewhere. All six rows appear in the map.

The 7 unparsed blocks are all in "Behaviors & Operations" and none was invented into an anchor
(D10): `B9a`'s wrapped continuation line that happens to open with a bold run (`:214`), `B16`'s
`**actively owned by another live session**` continuation (`:356`), and the five un-ided bold-lead
paragraphs of the `### Closing a feature` subsection (`:547`, `:553`, `:558`, `:564`, `:577`).
Each travels, verbatim, with the anchor whose block it sits in — and because a `###` heading does
not close an anchor's block, the whole `### Closing a feature` subsection sits inside **`B24`**'s
extracted text. `B24`'s owning concept, `sessions-lanes-and-identity.md`, therefore carries that
prose verbatim, and `gates.md` carries the same prose because the closing tail is topically its.
Only the anchor CLAIM is unique; the prose is deliberately in both.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| B1 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Guarded feature start |
| B2 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Closed phase vocabulary |
| B3 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Feature close adds a review candidate |
| B4 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Review session lifecycle |
| B5 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Coverage and staleness are derived, never stored |
| B6 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Status surfaces tell the review truth |
| B7 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | Cell plans are revisable in place, execution records never |
| B8 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | Unified command discovery and dispatch |
| B9 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A stuck worker may consult a configured adviser, inside its own turn |
| B9a | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | High-risk execution approval requires a live adviser consult |
| B10 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | A whole slice of work units is created in one all-or-nothing call |
| B11 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Concurrent sessions coordinate through atomic claims, on every claim path, not only the cross-session picker |
| B12 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | A feature can start as its own lane, and every lane mutation is commandable |
| B13 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | Readers resolve through the acting session's lane |
| B14 | [docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md](../knowledge/areas/workflow-state/holds-and-the-coordination-lock.md) | A write into another live session's held path is refused at write time |
| B15 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | A finished task hands itself to a fresh session through the two-kind handoff |
| B16 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | A session out of work pulls the next approved unit itself |
| B17 | [docs/knowledge/areas/workflow-state/cells-scheduling.md](../knowledge/areas/workflow-state/cells-scheduling.md) | The schedule is computed, not guessed |
| B18 | [docs/knowledge/areas/workflow-state/cells-scheduling.md](../knowledge/areas/workflow-state/cells-scheduling.md) | A dependency cycle is refused at the door |
| B19 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | A generic routing mutation is phase-owned |
| B20 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Eligible workers may execute in isolated linked worktrees |
| B21 | [docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md](../knowledge/areas/workflow-state/holds-and-the-coordination-lock.md) | A shared coordination store serializes its own concurrent writes |
| B22 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | A session's identity is derived automatically, never handed to it by another party |
| B23 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Mutating a claimed unit of work requires the caller to own it, with an audited rescue door |
| B24 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | A live session's heartbeat renews itself, throttled, and carries its claims and holds forward with it |
| B25 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | The approved plan document is frozen; the current slice lives only in work units |
| B26 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | Every verification or block appends one entry to a unit's attempt history |
| B27 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | A unit's lifetime budget is enforced at the moment of claiming, inside the same exclusive operation that decides… |
| B28 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | A budget reset is the sole door back into an exhausted unit, and it is never silent |
| B29 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | Authoring a unit of work classifies its change, and an insufficient verification plan is a warning, never a block |
| B30 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | Completing a behavior-changing unit requires substantial, non-duplicated proof that the bug was real |
| B31 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A judge verdict is a structured, append-only record with an honest independence stamp |
| B32 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | The goal-check's semantic judge scales with the lane's risk, and stays inside the loop that finishes a unit, never the… |
| B33 | [docs/knowledge/areas/workflow-state/recovery.md](../knowledge/areas/workflow-state/recovery.md) | A session that died without pausing can be detected and its unsettled work recovered from the harness's own conversation… |
| B34 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | Archiving a cell is a guarded transaction, not a plain file move |
| B35 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | Archiving is serialized against every other cell mutator at the one place all of them write |
| B36 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A needs-revision verdict on an already-capped unit reopens it, and re-capping demands fresh proof, never stale evidence |
| R1 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | A new feature can never inherit gate approvals |
| R2 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Feature start never destroys evidence of unfinished work |
| R3 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Phase values outside the closed vocabulary are rejected at the record layer, whatever a skill's prose says |
| R4 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Full independent review starts only after an explicit user request |
| R5 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Verification and review are separate |
| R6 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Review approval covers only the immutable change set inspected by that review session |
| R9 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | A review session's scope is frozen at creation |
| R10 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | Review status is always derived from records plus actual change history, never stored |
| R11 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | The final human approval of a review (its Gate 3) exists only inside a review session |
| R12 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | The unified entry point serves all nine command groups from one implementation |
| R13 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | The published command catalog and executable dispatch surface describe the same command set |
| R7 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | The workflow runs one cost pattern |
| R14 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | The adviser is a per-runtime configured role beside the reviewer role |
| R15 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | Consult triggers are objective, never self-assessed |
| R16 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | Advice is advice: it never approves gates or installations, never widens a worker's file scope, and never substitutes… |
| R19 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | a planned-next handoff's preconditions live in its verb, never in prose — **kept this id**: the fresh-session-handoff family kept R19/R20/R21, so every surviving citation of `workflow-state.md R19-R21` still lands on the rule it meant |
| R20 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | auto-resume authority exists only at the fresh-session boundary — **kept this id** (fresh-session-handoff family) |
| R21 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | the work puller never widens authority — **kept this id** (fresh-session-handoff family) |
| R17 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Concurrent ownership is decided by atomic exclusive creation, never by check-then-write |
| R18 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Contention is answered with a typed refusal carrying a code and reason, never an exception |
| R8 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A workflow configuration file that still carries the retired advisor setting loads successfully |
| R19a | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | the learning-capture phase is never settable; it is produced only by recording a knowledge sync — **shipped as a second `R19`**; renumbered here so both rules are individually measurable |
| R20a | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | recording a knowledge sync is refused unless execution actually happened — **shipped as a second `R20`**; renumbered (chain-integrity family) |
| R21a | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | the terminal state may be entered only from learning capture, and only while spec debt is zero — **shipped as a second `R21`**; renumbered (chain-integrity family) |
| R22 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Spec debt is advisory everywhere it is displayed and binding only at the close |
| R24 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A configured external assistant whose reply is free prose is proven live only by a **known-answer probe** — a question… |
| R23 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | No instruction anywhere in the workflow may name a phase outside the closed vocabulary |
| R25 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | The gate bypass level is a strict ladder of floors, each honored literally |
| R26 | [docs/knowledge/areas/workflow-state/cells-scheduling.md](../knowledge/areas/workflow-state/cells-scheduling.md) | No dependency cycle can ever be recorded |
| R27 | [docs/knowledge/areas/workflow-state/cells-scheduling.md](../knowledge/areas/workflow-state/cells-scheduling.md) | One overlap semantics, two consumers |
| R28 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | When review status is derived from change history, a conclusive repository answer remains authoritative even if the… |
| R29 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Every generic routing mutation is authorized by the selected record's valid pre-change phase |
| R30 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Routing ownership is derived, never persisted |
| R31 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | gate mutation is a dedicated operation; review owns no active pipeline state, and validation alone decides execution readiness |
| R32 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Worktree isolation removes Git index contention only |
| R33 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | All linked worktrees share exactly one validated main coordination store |
| R34 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Same-user workers are cooperative and fallible, not security principals |
| R35 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Canonical physical containment always precedes logical path normalization and authorization |
| R36 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Every claim path — a direct claim by identity as well as the cross-session picker — acquires the same exclusive token… |
| R37 | [docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md](../knowledge/areas/workflow-state/holds-and-the-coordination-lock.md) | A shared coordination store's read-modify-write body always serializes through its coordination lock |
| R38 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | A session's identity is always self-resolved at the moment of the operation from its own runtime environment, never… |
| R39 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Mutating a claimed unit of work is refused when a live claim names a different session, naming the owner and expiry |
| R40 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | A worker never establishes its own ownership of a unit of work |
| R41 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | A unit's lifetime budget is checked inside the same exclusive operation that grants its claim, not before or after it |
| R42 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | Two failed attempts sharing an identical failure signature exhaust a unit immediately |
| R43 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | The automatic work picker skips a budget-exhausted or repeated-failure unit rather than surfacing the refusal, so one… |
| R44 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | No autopilot level ever overrides a budget or repeated-failure refusal |
| R45 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | A budget reset is the only door back into an exhausted unit |
| R46 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | A unit's change classification is set explicitly or derived only from the behavior-change flag — never any richer… |
| R47 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A behavior-changing unit's completion requires substantial (not placeholder), non-duplicated proof-of-red evidence |
| R48 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A judge verdict is accepted only in its one structured shape |
| R49 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A verdict's model-independence status is derived, never asserted |
| R50 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | The goal-check's semantic judge is verification inside the loop that finishes a unit, scaled by lane risk — it is never… |
| R51 | [docs/knowledge/areas/workflow-state/recovery.md](../knowledge/areas/workflow-state/recovery.md) | A crashed session's unsettled work is recovered only through its harness transcript as a secondary source, never by… |
| R52 | [docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md](../knowledge/areas/workflow-state/holds-and-the-coordination-lock.md) | A stale coordination-lock holder is a takeover candidate only |
| R53 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | Archiving or unarchiving a cell is refused before any move when the destination already holds a cell of the same… |
| R54 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A needs-revision judge verdict recorded against an already-capped unit reopens it with its verify evidence cleared |
| R55 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | Session identity resolution carries one durable fallback below the environment lookup and above the ownerless floor |
| E1 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | A capped prior-feature cell never blocks a new start |
| E2 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Refused starts are proven side-effect-free: the record is byte-identical after a refusal |
| E3 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A configuration file carrying the retired advisor setting → parses normally with the setting stripped from the parsed… |
| E4 | [docs/knowledge/areas/workflow-state/recovery.md](../knowledge/areas/workflow-state/recovery.md) | A stale-heartbeat session whose transcript tail carries the clean-stop sequence is not a crash candidate — it stopped… |
| E5 | [docs/knowledge/areas/workflow-state/recovery.md](../knowledge/areas/workflow-state/recovery.md) | A stale session whose transcript is missing entirely is reported as a session with no transcript, never as a recoverable… |
| E6 | [docs/knowledge/areas/workflow-state/recovery.md](../knowledge/areas/workflow-state/recovery.md) | When the dead session had no bound feature, its mining window keys on the last global settled outcome and its recovery… |
| E7 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | review coverage edge cases — exact-anchor coverage, one newer change, rewritten history, absent change-history tooling |
| E8 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | a corrupt review record: read paths skip it with a warning, write verbs refuse loudly with the record untouched |
| E9 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | The old "past reviewing but the review gate still pending" staleness warning is retired: closing through scribing/compounding… |
| E10 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | A catalog fingerprint change never appears inside the requested command's ordinary result |
| E11 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | A missing required parameter, a value with the wrong shape, or an unknown command is rejected before any workflow record… |
| E12 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A consult attempt that fails at the transport level (the adviser is unreachable, errors, or hangs past the external-work… |
| E13 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | A workflow whose configuration names no adviser dispatches byte-identical worker instructions to before the adviser… |
| E14 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | One invalid unit in a batch slice-creation request → zero units written |
| E15 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Linked pointers may be absolute or relative and may use supported Windows path forms |
| E16 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Traversal, outside-main absolute paths, symlink escapes, and separator/case escapes are denied across every… |
| E17 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Detached/ref/identity/common-location mismatch, non-descendant revisions, and out-of-reservation diffs halt integration |
| E18 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Transaction behavior is proven in deterministic temporary Git repositories because the live checkout's Git metadata is… |
| E19 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Project directories on network file systems are declared unsupported for session coordination: exclusive creation is not… |
| E20 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | A same-session round trip on one unit of work — claim, block, reopen, claim again — never self-refuses: the exclusive… |
| E21 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | A forced ownership override always leaves a permanent audit trace naming the verb, who forced it, whose ownership was… |
| E22 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | A single-user workspace with no session identity anywhere in the environment behaves exactly as before: claims and holds… |
| E23 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | A unit with no attempt history yet (every legacy unit) is measured against the default lifetime budgets and behaves… |
| E24 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | An autopilot level set to `total` still does not waive a budget or repeated-failure refusal — proven by a dedicated test… |
| E25 | [docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md](../knowledge/areas/workflow-state/cells-completion-judge-and-archive.md) | A behavior-changing unit riding the existing deliberate-exceptions door for its proof-of-red keeps that door's original… |
| P1 | [docs/knowledge/areas/workflow-state/worktree-isolation.md](../knowledge/areas/workflow-state/worktree-isolation.md) | Worktree isolation (B20/R32-R35) |
| P2 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | the record itself — `.bee/state.json`, its verbs, and `startFeature()`/`isKnownPhase` |
| P3 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | Phase-owned routing |
| P4 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | the 15 start-feature test rows |
| P5 | [docs/knowledge/areas/workflow-state/gates.md](../knowledge/areas/workflow-state/gates.md) | feature-start evidence — commit `928abf1`, trace `.bee/cells/codex-parity-5.json` |
| P6 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | Unified dispatcher and catalog |
| P7 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | Cost pattern / tier resolution |
| P8 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | Adviser (worker consult) |
| P9 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | Batch slice creation |
| P10 | [docs/knowledge/areas/workflow-state/dispatch.md](../knowledge/areas/workflow-state/dispatch.md) | Unified dispatcher (all nine groups) |
| P11 | [docs/knowledge/areas/workflow-state/advisor-consult.md](../knowledge/areas/workflow-state/advisor-consult.md) | Advisor config tolerance |
| P12 | [docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md](../knowledge/areas/workflow-state/cells-authoring-and-revision.md) | Cell revision |
| P13 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Session coordination (B11/R17/R18) |
| P14 | [docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md](../knowledge/areas/workflow-state/sessions-lanes-and-identity.md) | Lanes (B12) |
| P15 | [docs/knowledge/areas/workflow-state/holds-and-the-coordination-lock.md](../knowledge/areas/workflow-state/holds-and-the-coordination-lock.md) | Hold enforcement (B14) |
| P16 | [docs/knowledge/areas/workflow-state/handoff.md](../knowledge/areas/workflow-state/handoff.md) | Fresh-session flow (B15/B16) |
| P17 | [docs/knowledge/areas/workflow-state/review-sessions.md](../knowledge/areas/workflow-state/review-sessions.md) | review records — `.bee/reviews/`, the candidates ledger, and the derivation library |
| P18 | [docs/knowledge/areas/workflow-state/claims-and-ownership.md](../knowledge/areas/workflow-state/claims-and-ownership.md) | Multi-session hardening (B11/B21-B24, R36-R40) |
| P19 | [docs/knowledge/areas/workflow-state/cells-scheduling.md](../knowledge/areas/workflow-state/cells-scheduling.md) | Computed schedule (B17/B18, R26/R27) |
| P20 | [docs/knowledge/areas/workflow-state/cells-attempt-budgets.md](../knowledge/areas/workflow-state/cells-attempt-budgets.md) | Self-correcting loop (B26-B32, R41-R50) |
