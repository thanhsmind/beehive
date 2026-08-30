---
type: bee.area
title: Advisor Protocol — consult triggers
description: "The two consult triggers: the dispatcher's budgeted worker offer and the mandatory orchestrator consult before high-risk execution approval."
timestamp: 2026-08-06
bee:
  id: advisor-protocol-triggers
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [areas/advisor-protocol/overview.md]
  decisions: ["AO2(b)/AO3/AO13 (one orchestrator trigger; execution-gate precondition, folded from the old standalone execution gate into Gate 2 by validation-diet D2/D14; event-based staleness, never a TTL)", AO4 (call paths split by trigger class), AO14 (execution-worker class), "126412b9 (precondition keys on the selected record's mode)", "20969403 (gate-door-refusal: the high-risk execution refusal states its own cause instead of rendering as an argument-shape complaint; the honesty shipped, the unblock deliberately did not — cell gdr-1, 2026-08-04)", "b34fdea9 (proactive-leader-intake D4: the orchestrator consult IS the plan-step hat wave's synthesis, recorded post-plan with the unchanged advisor-ref verb)"]
  sources: ["advisor-and-orchestration Slices 2A-i..2A-iv, 2B, 3A, 3B, 4, 5 (cells ao-2ai-1..ao-5-1, traces in .bee/cells/, reports docs/history/advisor-and-orchestration/reports/, 2026-07-17)", first live orchestrator consult digest .bee/spikes/advisor-and-orchestration/slice5-advisor-digest.txt, "docs/specs/advisor-protocol.md#B1", "docs/specs/advisor-protocol.md#B3", "docs/specs/advisor-protocol.md#E3", "docs/specs/advisor-protocol.md#P2", "docs/specs/advisor-protocol.md#P6", "gate-door-refusal cell gdr-1 (both high-risk refusal arms return a stated refusal via one shared helper; trace .bee/cells/gdr-1.json, capped 2026-08-04 — state_group tests green)"]
  authoritative_for: "advisor-protocol: consult triggers"
---

# Advisor Protocol — Consult Triggers

## Entry Points & Triggers

- **Worker trigger (available, budgeted):** a worker that has just hit its
  first serious failed verification attempt may consult the adviser named in
  its dispatch — at most twice per claim, then it must return blocked.
- **Orchestrator trigger (mandatory, mechanical):** before the execution gate
  opens for work in the high-risk mode, the orchestrator must hold a live
  (non-stale) consult record. The approval verb itself refuses otherwise.
  This is machinery, not a human stop: every autopilot level still runs it.
  Since proactive-leader-intake (decision `b34fdea9`) the consult is no
  longer a separate dispatch: it IS the plan-step hat wave's synthesis
  (B3 below).
- No other trigger exists. Conflict-between-decisions and scope-creep triggers
  were considered and explicitly deferred/dropped (they lack a mechanical
  detector today).

## Behaviors & Operations

**B1 — The dispatcher offers the adviser; the worker never self-assesses.**
At dispatch the orchestrator resolves the configured adviser and applies the
one honest no-op; otherwise the dispatch names the adviser and exactly how to
reach it (its proven transport). Workers on the session's strongest tier are
offered advisers too — configuration outranks any strength intuition.

**B3 — The high-risk consult is the plan-step hat wave's synthesis, recorded
after the plan settles** (proactive-leader-intake D4, decision `b34fdea9`).
The consult surface no longer stands alone. At the plan step the leader opens
the hat wave — fixed perspectives dispatched `--kind advisor`, once per
feature — and synthesizes their returns itself; that synthesis is the evidence
bundle, and `bee state advisor-ref record --advisor <identity> --digest-file
<path>` records it. The verb is unchanged and no code moved: the approval verb
still verifies the record is live, and a missing or stale record still refuses
with a corrective message naming each failed condition and the consult flow.

**Timing law.** Record the ref AFTER plan.md holds its gate-ready bytes and
AFTER the last pre-gate `bee decisions log` write. The verb stamps its
staleness anchors from the active feature, the newest active decision id, and
that plan.md's sha256, so a ref recorded at wave time goes stale by
construction and refuses the very gate it was meant to open. The live ref
doubles as the once-per-feature mark: a resumed session never re-runs the wave
on it, and a stale ref after a material plan change permits exactly one re-run.

The wave's own law rides the same feature: firing point and threshold are
proactive-leader-intake D1/D2 (`a52c854d` — plan-step firing, once per
feature, clear/tiny asks skip), the kept discretionary pre-Lock critique
window is proactive-leader-intake D5 (`98ac20a1`), unattended runs recording
questions instead of blocking is proactive-leader-intake D6 (`f73d6c49`), and
the one-plain-voice communication rule is proactive-leader-intake D7. The
procedure home stays `skills/bee-hive/references/gates-and-delegation.md`
("Hat wave").

A workspace with no adviser configured records that fact and proceeds — the
rule adds one trigger, not a dependency on configuration. Procedure home for
the wave: `skills/bee-hive/references/gates-and-delegation.md` ("Hat wave").
Independent review (bee-reviewing, Gate 3) is untouched and stays
user-invoked.

**B3a — The refusal states its own cause, and it took a cell to make that true
(gate-door-refusal, cell gdr-1, 2026-08-04).** Until then B3's promise held only
on paper: both refusal arms returned a bare internal error that the command
surface rendered as *unsupported argument shape*, telling the caller their flags
were wrong when the flags were correct — a refusal that misdirects is worse than
a blunt one, because it sends the reader to fix the wrong thing. Both arms now
return a stated refusal that names the missing or stale consult, lists each
failed condition separately (no record at all, the feature changed since the
consult, a decision was logged since it, the plan changed, a revocation
postdates it), and names the recording flow. The verdict itself did not
change — only its honesty — and nothing is written on either path, exactly as
before. What each actor observes: the same approval is refused, and the reader
is now pointed at the real precondition instead of at their own command line.
That precondition and its recording verb remain unfinished; the gap is stated in
`areas/workflow-state/gates.md` rather than implied here.

## Edge Cases Settled

- **E3 —** Corrupt or hand-edited consult record → reads as missing; the verb
  never crashes; the approval refuses with the standard message.

## Open Gaps

- The conflict-between-decisions trigger waits on structured decision records
  (its prerequisite feature), and the scope-creep trigger has no source of
  truth; neither is built, neither is silently substituted.

## Pointers (implementation)

- **P2 —** Orchestrator consult + throw: `handleStateGate`'s
  `requireFreshAdvisorForHighRisk` (shared by the standalone `--name execution`
  path and the merged `--merge` path) + `state advisor-ref record/show` in
  `the bee binary`; helpers `advisorRefAnchors` /
  `advisorRefStale` in `packages/bee/lib/state.mjs`. There is no
  standalone validating skill (deleted, validation-diet D1) — the consult now
  has to happen during planning/briefing, before Gate 2's execution component
  is approved (validation-diet D14).
- **P6 —** Gate precondition spec detail: `docs/specs/workflow-state.md` B9/B9a.
- **P7 —** The stated refusal (B3a): `high_risk_advisor_refusal` in
  `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:535-558`, called
  from both the pre-lock peek (`set_gate.rs:611-613`) and the post-lock recheck
  (`set_gate.rs:621-623`) so the two arms cannot drift; the per-cause strings it
  joins are built by `advisor_ref_stale` in `advisor_ref.rs:133-199`. Test:
  `set_gate.rs:1048-1069` asserts the refusal is a stated one naming the
  high-risk cause, not the generic argument-shape error. Evidence: trace
  `.bee/cells/gdr-1.json`, capped 2026-08-04.
