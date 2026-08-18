# 002 — mechanism inventory findings

Digest of the read-only sweep (gather-tier, 2026-08-18). Anchors are
file:line in this repo.

## CLI enforcement

- Cap vocabulary: `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:54`
  `REPORT_KEYS = ["outcome","commit","files","tests","deviations"]`;
  `tests` must be `"boundary"`/`"undeclared"` (refusal at :125-139). No
  scope/reason sub-field exists — carrying one is a shape change to
  this validator plus `packages/bee/prompts/worker-cell.md`.
- Boundary auto-run copies: `verbs/drivers/close.rs:134`
  (`run_declared_tests`; defers to merge when a worktree exists,
  :1657-1706) and `verbs/worktree/phases.rs:681-732` (merge verify
  child). Only skip mechanism: `commands.test: "none"` sentinel
  (`verbs/worktree/handlers.rs:530`). No `--skip-tests` flag anywhere.
- Standalone runner: `verbs/test_runner.rs` (backs `bee test`); records
  `.bee/logs/test-results.json`.

## Preamble text

- "Never build on red: run the test command above before your first
  `cells claim`…" — `hooks/session_preamble/budget.rs:615`, rendered
  when `commands.test` declared; byte-pinned by
  `hooks/session_preamble/tests.rs:145,157`.

## Skill text refrains (canonical skills/ tree)

~16 passages carry the boundary refrain or "never build on red":
bee-swarming SKILL.md:123-126,140; swarming-reference.md:175-181,
261-263; worker-details.md:11-25; bee-planning SKILL.md:99,103;
planning-reference.md:225-234,345; bee-hive SKILL.md:94;
gates-and-delegation.md:115,177-181; routing-and-contracts.md:169;
bee-shaping mini-brief-template.md:28; implement-plan-template.md:118.
Generated trees mirror all of these (regen chain).

## Stale text

- AGENTS.md:86-89 and packages/bee/AGENTS.block.md:79-82 (identical):
  "`bee cells finish` runs them. Green caps the cell; red refuses the
  cap…" — contradicts decision 13ce1858; cap never runs tests today.
  AGENTS.block.md is hash-pinned via onboard managed block — edits need
  the regen/hash path.

## Onboard templates

- `onboard/templates.rs` carries no cadence prose; hash-pinned header
  confirmed (:10-23).
