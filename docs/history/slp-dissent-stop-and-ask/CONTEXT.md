# SLP Dissent / Stop-and-Ask — Context

**Feature slug:** slp-dissent-stop-and-ask
**Date:** 2026-08-28
**Shaping session:** complete (Lock consumed the closed map docs/discovery/slp-supervisor-lead-peer/MAP.md — no decision originated here)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

A dispatched worker gains a voice with teeth. It can record a **dissent**
against the cell it was handed — a claim, an alternative, and a severity — and
it can **stop and ask** with options instead of guessing. A blocker-severity
dissent pauses the related work and obligates the orchestrator to one of three
logged answers before that work resumes. The obligation is enforced at the
close and merge doors, exactly the way judge-debt already is. No live
mid-flight question-and-answer channel is built.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. IDs are bee decision-log ids (search with
`bee decisions search`).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| 787a9eb0 | SLP is distilled into bee's skeleton; bee's locked rules (R2 human merge, R3 owner interlock, R4 permission split, gates) win on any conflict | dissent adds an obligation on the orchestrator, never a relaxation of a gate |
| a020319d | This is the SECOND of the four slp clusters; the supervisor heartbeat is shipped, blind lanes and contract/original-request are separate features and out of this boundary | — |
| 4b7aa303 | Dissent has FULL TEETH: a worker's blocker-severity dissent pauses the RELATED part of its work while other parts continue, and the orchestrator is OBLIGATED to answer one of three — accept and log / reject with reasoning / escalate a rung — recorded in the decision log before the related work resumes | advice-only dissent breeds compliant workers; a full stop stalls unattended runs. Pausing only the related slice keeps throughput while the obligation stays real |
| a2affcba | Mechanism: a new cells-level dissent record `{target, claim, alternative, severity}` written through the CLI; `bee close` and `bee worktree merge` REFUSE while any dissent lacks a recorded accept/reject/escalate-a-rung verdict with reason (the same enforcement shape as judge-debt); blocker severity also rides the existing blocked-status machinery so the related work stays unclaimable; StopAndAsk takes the herding round-mailbox shape with `options[]` and `leaning` added to the `[BLOCKED]` form; NO live mid-flight Q&A channel is built; the SLP escalate verb NEVER reuses `bee cells escalate`, which means model tier | doors are the only place bee makes an obligation real. Native subagents exit when they speak, so a synchronous wait channel would need a transport bee does not have. Consider-grade dissent has no carrier today at all |

### Agent's Discretion

Everything the decisions above leave open is the implementer's choice at
planning: the dissent record's store location and exact schema beyond the four
named fields, the new verb's name and flag surface, how "the related part" is
resolved from a dissent to the work it pauses, the verdict payload's validation
shape, and the wording of the two door refusals. Constraint: reuse the existing
machinery — the judge-debt door pattern, the blocked-status transitions, the
herding round-mailbox, and the decision log — before inventing any new
subsystem.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Dissent | A worker's recorded disagreement with the cell it was handed: `{target, claim, alternative, severity}`. A record, never prose in a report. |
| Blocker dissent | A dissent at blocker severity. It pauses the related work and obligates a verdict. |
| Consider dissent | A dissent below blocker severity. It is recorded and answered, but it pauses nothing. |
| Verdict | The orchestrator's obligated one-of-three answer to a dissent: accept and log, reject with reasoning, or escalate a rung. Written to the decision log. |
| Escalate a rung | Raise the dissent to the next authority up. NOT `bee cells escalate`, which means model tier and keeps that meaning. |
| StopAndAsk | A worker ending its turn on the `[BLOCKED]` form carrying `options[]` and a `leaning`, instead of guessing. |
| Related part | The portion of the work a blocker dissent pauses. Other parts continue. |

## Specific Ideas And References

- The obligation is enforced the **judge-debt** way. That door already proves
  the shape: a capped cell without its record refuses the close by name, and
  the remedy is printed with it. Dissent copies that, it does not invent a
  second enforcement style.
- StopAndAsk reuses the **herding round-mailbox** shape rather than a new
  channel. A worker that speaks and exits is the transport bee actually has.
- Boundary signals worth naming in worker instructions, from the grilling
  round: contract or API changes, trading data quality or user experience for a
  technical target, and new dependencies.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1420` — `build_close_report_doors`, the single door builder for close. A dissent-debt door is one more entry here.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1461-1516` — the judge-debt door arm itself: lane-gated existence, a debt count, a named escape decision, and a `command` remedy. The exact arm to copy.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:382` — `judge_debt`, which counts a capped cell as debt when its trace carries no verdict. A dissent-debt counter mirrors it one for one.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2016-2046` — the three-line refusal emit (headline, `remedy:`, `next:`), reading the door's own verdict and never recomputing it.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs:169-289` — `run_judge_record`: a `--file` payload, validated, appended to the cell trace, and — on a negative verdict — the cell is REOPENED and a decision logged. The dissent verdict mirrors this.
- `packages/bee-rs/crates/bee/src/verbs/cells/judge.rs:244` — `validate_judge_verdict` with its versioned schema string and closed enum sets. The template for a `dissent-verdict/1`.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1102-1143` — `run_block`, which writes the blocked status and its reason plus an attempt record. The "blocker dissent parks the work" half.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1249-1292` — `run_reopen`, the only unblock path.
- `packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs:98-111` — a blocked dependency makes its dependents unschedulable. This is the REAL tooth behind "the related work stays unclaimable".
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:318-345` — the result contract rendered into every worker brief; its shape is `{status, summary, files_changed, proof}`. Where `options[]` and `leaning` graft on.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:367-397` — the parsed result struct, its two-value status, and its typed errors. A new field needs a change here too.
- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:226-238` — the closed departure-kind set, with the header comment stating why a fifth kind is a decision and never a worker's word choice. The precedent for a closed severity set.
- `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:56` — the five cap-report keys. There is no `concerns` key; consider-grade dissent genuinely has no carrier today.

### Established Patterns

- **Door then refusal, never recomputed.** A door builder produces the verdict; the driver refuses by reading it.
- **Every debt door has a named escape.** A logged decision tagged `<door>-deferral` naming the feature clears it. `dissent-deferral` follows that precedent.
- **A door exists only for the lanes that care.** Below `standard` the judge-debt door is absent, not merely non-blocking; an authoring-time obligation check fills the gap (`packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs:283,325`).
- **A verdict with teeth changes state.** A recorded "no" reopens the cell and logs a decision; an inert trace entry is the named anti-pattern.
- **A structured payload arrives as a `--file`,** validated against a versioned schema string; free prose is a failed run, not a soft verdict.
- **Serve and declare in one change.** A verb the dispatcher serves and the registry does not declare fails the contract tests, and so does the inverse.

### Integration Points

- Handler module plus its group `mod.rs`; the sub-verb dispatch table at `packages/bee-rs/crates/bee/src/verbs/cells/util.rs:75-100`.
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — HAND-EDITED; `bee dev regen` never touches it.
- `packages/bee-rs/crates/bee/src/catalog.rs:611` — `PINNED_FLAG_COUNT`, bumped on purpose with the reason recorded.
- `packages/bee-rs/crates/bee/src/router.rs:70-71` — the cells coverage lines a new sub-verb must join.
- Contract tests that walk the declared set: `tests/registry_dispatch.rs`, `tests/registry_contracts.rs`, `tests/front_door.rs`, `tests/installer_invocations.rs`.
- Worker prompt surfaces, two of them: `skills/bee-swarming/references/swarming-reference.md:539-544` (the `[BLOCKED]` form) and `packages/bee-rs/crates/bee/src/herding/mailbox.rs:318-345` (the herding brief).
- `packages/bee-rs/crates/bee/src/verbs/workflow_store/merge_ready.rs:240` — `set_blocked_by` already takes door names, so a `dissent-debt` door name flows into the projection for free. Note that nothing reads `merge_ready`; it is additive only.

### Two facts that correct the research digest

- **`bee worktree merge` has NO judge-debt door today.** Its only cell-debt precondition is proof debt (`packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:154-172`). Judge-debt is close-only. a2affcba names both doors, so the merge half is NEW code, not a copied arm — planning must budget for it.
- **A blocked cell does not refuse `bee close`.** It only suppresses cell retirement. The unclaimable tooth comes from the scheduler's dependency check, not from the close driver.

## Canonical References

- `docs/discovery/slp-supervisor-lead-peer/MAP.md` — the closed map and full decision gists.
- `docs/discovery/slp-supervisor-lead-peer/tickets/005-dissent-stop-and-ask.md` — the grilling ticket, question and answer.
- `docs/history/research/slp-dissent-surfaces.md` — the surface-by-surface findings this mechanism was chosen from.
- `docs/history/slp-supervisor-heartbeat/CONTEXT.md` — cluster 1, shipped; the sibling this feature sits beside.
- `docs/specs/slp-supervisor-lead-peer/` — the source SLP spec (idea source only, per 787a9eb0).

## Outstanding Questions

### Resolve Before Planning

(none — every product decision is a locked D-ID above)

### Deferred To Planning

- [ ] Where the dissent record lives and its full schema beyond `{target, claim, alternative, severity}` — read the cell trace store and the supervisor stores, pick one pattern.
- [ ] How "the related part" is resolved from one dissent to the work it pauses — by cell, by declared file overlap, or by an explicit field on the record.
- [ ] The new verb's name and flag surface, given that `bee cells escalate` is taken and means model tier.
- [ ] The verdict payload's validation shape — whether it mirrors `judge-verdict/1` or takes a smaller form.
- [ ] Where `options[]` and `leaning` attach to the `[BLOCKED]` worker form, and which prompt templates must carry the boundary signals.
- [ ] Which declaration surfaces a new verb group must be added to so no contract test passes on an absent row.

## Deferred Ideas

- A live mid-flight question-and-answer channel between orchestrator and worker — explicitly not built (a2affcba); it needs a transport bee does not have, and returns only as a fresh effort.
- Clusters 3 and 4 — blind lanes, and contract status with the verbatim original request — separate features by a020319d.

## Handoff Note

<!-- bee:not-a-deferral: Handoff Note describing which parts of this template planning reads. It names the deferred-to-planning section as machinery, it does not defer anything. -->
CONTEXT.md is the source of truth. Decision IDs are bee decision-log
ids, stable. Planning reads locked decisions, code context, canonical
references, and deferred-to-planning questions. Planning's Gate 2
shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
