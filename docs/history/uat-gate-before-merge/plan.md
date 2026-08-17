# uat-gate-before-merge — plan

Lane: standard (flags: data-model, covered-contract-change). Locked decision:
uat-gate-before-merge D1.

## Goal

A new `uat` gate sits between execution-complete and `bee worktree merge`.
Merge refuses (typed, zero-mutation) for standard/high-risk features until the
user approves the gate. Tiny/small/docs exempt. Escape hatches: `--skip-uat`
(one merge), config `uat_before_merge: false` (repo-wide). Gate bypass never
auto-approves uat.

## Evidence map (from code scan)

- Gate set is a fixed array: `state.rs:31` `GATE_NAMES = ["context","shape",
  "execution","review"]`; defaults `state.rs:22-29`; name validation
  `verbs/state_group/set_gate.rs:628-635`; per-gate record shape written at
  `set_gate.rs:771-789`.
- Bypass is agent-side: CLI only records `--actor auto --bypass-level
  --reason` (`set_gate.rs:650-675`); no Rust auto-approves.
- Merge preconditions live in `verbs/worktree/phases.rs` `merge_stage`;
  the last zero-mutation checkpoint is after the branch-mismatch check
  (~phases.rs:213), before companion teardown (~:215-228). New check slots
  there.
- Merge already resolves the feature: `merge.rs:413-426`
  `resolve_worktree_feature` (creation identity, fallback worktree state).
  It does NOT read the lane — map feature → lane via the workflow record /
  `.bee/lanes/<feature>.json` on main_root.
- Config pattern: `read_config_raw` + overlay (`state.rs:130-179`); model
  `uat_before_merge` on the fail-closed shape of
  `worktree_cleanup_on_merge_config` (`handlers.rs:415-421`): absent → ON
  (true), explicit false → off, non-bool → refuse.
- Flag plumbing for `--skip-uat`: `verbs/reservations/flags.rs:31-37`
  FLAG_ALONE_BOOLEANS; `handlers.rs:440` keys_known allowlist +
  bool validation pattern (:446-456); generated
  `registry_payload.json` (try `bee dev regen`; else hand-edit; tests
  `registry_contracts.rs` pin shape).
- Gate display: `hooks/session_preamble/render.rs:49-58` visible_gates and
  `verbs/status_full/orient.rs:498-508` iterate GATE_NAMES.
- `waiting-on --kind gate` already valid (`record.rs:356`).
- Herding role-merge doc branches on 3 outcomes only
  (`.claude/skills/bee-herding/references/role-merge.md` §5); new refusal
  needs its own branch.
- Doc drift found: `docs/handbook/register.md:116` still claims
  `worktree_cleanup_on_merge` absent = on; code says absent = keep. Fix in
  the docs cell.

## Shape — one slice, 3 cells (serial: one execution worker at a time — see
pattern 20260817-parallel-workers-in-one-worktree-share-one-git-index)

### ug-1 — the `uat` gate exists in state

- Add "uat" to GATE_NAMES + default_gates; set_gate accepts `--name uat` from
  `--actor user` (default) and REFUSES `--actor auto` for uat with a typed
  message ("uat is never bypass-approved" — D1).
- Visibility: preamble `visible_gates` shows "uat" only once the execution
  gate is approved (before that it is noise); orient's slash-line picks it up
  from the array.
- Tests: uat accepted/persisted; auto-actor refused; unknown-name test still
  red for garbage; visibility rule.

### ug-2 — merge refuses without uat approval

- New zero-mutation precondition in `merge_stage` after branch-mismatch:
  resolve feature (existing helper) → lane from main's lane/workflow record →
  when lane ∈ {standard, high-risk} (unknown/missing lane counts as standard,
  fail-closed) and the feature's uat gate is not approved and `--skip-uat`
  absent and config `uat_before_merge` ≠ false → refuse
  `WORKTREE_MERGE_UAT_PENDING`, remedy string naming all three exits.
- `--skip-uat` wired through FLAG_ALONE_BOOLEANS, keys_known, bool
  validation, registry payload (+ contract tests).
- Tests: refuse on standard+unapproved; pass on approved / --skip-uat /
  config false / tiny lane; non-bool config refused.

### ug-3 — docs + flow surfaces (docs-lane content, same worktree)

- `.claude/skills/bee-herding/references/role-merge.md`: new branch —
  `WORKTREE_MERGE_UAT_PENDING` = stop, report "awaiting user acceptance",
  never retry, never self-approve.
- `docs/handbook/register.md`: document the uat gate + `uat_before_merge` +
  `--skip-uat`; fix the stale `worktree_cleanup_on_merge` line.
- bee-hive skill references (gates wording "three gates") and bee-swarming
  close step: after last cell caps → stop in worktree, present "ready for
  user testing" + how to run, `bee state waiting-on set --kind gate
  --subject "uat: <feature>"`. AGENTS.md gate list line if it names gates.
- Tests: none beyond format checks; verify still runs the suite.

## Test scoping

`commands.test` (workspace cargo test) at every cap.

## Risks

- Self-hosting lag: the running binary enforces nothing until next release.
- Herding merge role will hit the refusal on standard features — ug-3's
  branch makes that a clean stop, not an anomaly.
- registry_payload generator location unconfirmed — worker tries `bee dev
  regen` first, records the fallback if hand-edited.
