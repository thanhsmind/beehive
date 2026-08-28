# Advisor consult — slp-dissent-stop-and-ask plan shape

**Advisor tier:** fable (configured advisor slot), 2026-08-28
**Scope:** the high-risk plan shape at `docs/history/slp-dissent-stop-and-ask/plan.md`,
read against CONTEXT.md's locked decisions and the named source anchors.

## P1 — the plan is wrong here

**P1.1 — The merge precondition as described drops the deferral escape.**
The proof-debt style has no escape: `verbs/worktree/phases.rs:154-178` refuses
unconditionally. The judge-debt shape a2affcba locks HAS one
(`has_judge_deferral_decision`, `verbs/drivers/close.rs:1471`). Copying
proof-debt literally means a logged `dissent-deferral` clears close and can
never clear merge — neither shape. The merge precondition must read the same
deferral decision the close door reads, through a shared helper in
`verbs/cells/` (the placement `feature_proof_check` already uses for both
doors), refusing with its own code and proceeding ungated when the identity
carries no feature. The plan's "merge has no judge-debt door" claim itself is
correct — verified.

**P1.2 — Lane gating is undecided, and the default copy makes dissent decorative below `standard`.**
The judge-debt arm exists ONLY for standard/high-risk routes
(`close.rs:1456-1461`). A verbatim copy inherits that gate, so a worker's
dissent in a `small` lane — a lane that does dispatch workers — would never be
enforced. Unlike judge debt, a dissent record exists only because a worker chose
to write one: its existence is the gate. The door must exist in every lane where
a dissent record exists, and the plan must say so.

## P2 — the plan is incomplete here

**P2.1 — The blocker tooth is in the wrong phase.** The test matrix asserts
`open -> dissent(blocker) -> blocked` as a transition of the Phase-1 record verb,
but the plan ships that behavior in Phase 3, so Phase 1's severity tests get
rewritten in Phase 3. The tooth is one call into the existing block mutation
(`handlers_close.rs:1102-1143`) inside the same verb — it belongs in Phase 1,
leaving Phase 3 purely the merge precondition.

**P2.2 — A herding worker cannot record a dissent at all.** The herding brief
orders "Never run any `bee` command" (`herding/mailbox.rs:310-315`), and
`bee herding run` does nothing with a Blocked result but label it and exit
(`herding/run.rs:2573-2593`). The CLI dissent verb is unreachable from the
herding lane; only swarming workers can record one. Either scope herding-lane
dissent out with a recorded reason, or carry dissent fields on the mailbox
result and have the control loop transcribe them.

**P2.3 — Post-verdict state per verdict kind is undefined.** Two questions with
teeth: does an `escalate-a-rung` verdict release the blocked cell and clear the
door (making escalate the cheap bypass), or not (in which case nothing releases
it)? And the verdict verb must do its own release like `run_judge_record` does
its own reopen (`handlers_meta.rs:262-289`), never shell into `run_reopen`,
which is claim-guarded.

**P2.4 — Claim ownership on the verdict verb is wrong-shaped.** `run_block`
does not release the claim, so a worker that dissents and exits leaves its claim
on the cell. The verdict is the orchestrator's verb by 4b7aa303; a worker-shaped
ownership guard would make `--force-ownership` the routine path. Decide: the
dissent record releases the exiting worker's claim, or the verdict verb is
exempt by design with the reason recorded.

**P2.5 — Archive handling is absent.** `judge_debt` counts archive-inclusive
(`close.rs:385`) and its remedy names `unarchive` FIRST because the record verb
refuses archived cells (`close.rs:1477-1490`). Without both, an auto-archived
cell with an unanswered dissent is either invisible debt or an unclearable door.

## P3 — worth knowing

- **The blocked-status reach is deps-only** (`schedule.rs:98-111`). Under-block:
  a sibling sharing files but carrying no dependency edge stays claimable, and an
  already-claimed in-flight dependent is untouched. Over-block: a dissent against
  one requirement of a five-requirement cell pauses all five. All acceptable under
  the locked decisions, but the plan should name them so the "not the rest of the
  slice" probe tests the real boundary.
- **`merge_ready` is free.** The doors vector is fed to `set_blocked_by` at every
  full-doors path; the new door name flows in with no code, and door tests find
  doors by name rather than index. Check whether the session preamble and the
  chain nudge, which both name `judge-debt`, should also name dissent debt.
- **A smaller honest shape exists at the verdict payload.** The `--file` plus
  versioned-schema pattern exists because judge verdicts are foreign-model output.
  A dissent verdict is session-authored, so
  `--verdict accept|reject|escalate --reason "..."` gives the same closed set and
  required reason with no schema module. That pattern is a form rule, not a locked
  decision; one deviation line covers it.
- **Phase order is otherwise sound**, and the mailbox parse change is backward
  compatible — the parser reads named fields and ignores extras.
- Nothing tells a swarming worker the dissent verb exists. One line in the worker
  contract and one in the orchestrator's per-result list are needed.
