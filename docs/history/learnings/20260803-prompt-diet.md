# Learnings — prompt-diet (2026-08-03)

Feature: research-backed diet of the 9 SKILL.md bodies plus the
prompt-writing standard (docs/knowledge/areas/doctrine-layer/
prompt-writing-standard.md). Merged to main; suite 989 passed / 0 failed.

## What settled

1. **The duplication boundary generalizes.** router-cost proved it for
   router-vs-operating-block; prompt-diet applied the same law across
   every SKILL.md: one full statement in the AGENTS block, a one-line
   cite plus local delta everywhere else. Gate-approval went from 6
   copies to 1+cites, worktree-first 4→1, cite-never-reinterpret 4→1,
   65%-handoff 3→1, headless boilerplate 5→1 (canonical home:
   bee-hive "Headless").
2. **External evidence can drive a diet without becoming a gate.** The
   four 2025-26 context-file studies justified the one-off event; per
   decision 8f63adb4 no size ceiling was (or may be) introduced —
   density stays a per-edit judgment.

## Process learnings

- **Route/gate ordering vs worktrees (defect-shaped).** `bee route --set`
  for a code-touching lane refuses from main when any granted worktree
  exists (workflows.rs:622 → generic unsupported-argument-shape error),
  and the same verb is control-plane-refused inside the worktree. A
  feature that creates its worktree before recording its route can then
  record the route nowhere; this run logged the route in the lane's
  gate decision as a named deviation. Either the refusal should name the
  real reason, or lane-record routes should be exempt from the
  any-granted-worktree check.
- **The wave-barrier regen ack worked as designed.** pd-1..pd-3
  acknowledged `wave-barrier`; pd-5 ran render-skill-trees → onboard
  --apply → release-manifest --write once at wave close, and the
  validator's refusal message was sufficient documentation to wire it.
- **Worktree test-cwd defect surfaced.**
  `renew_cross_worktree_holds_renews_active_session_rows_only` resolves
  main_root from process cwd and fails under any worktree cwd
  (state_sync.rs:828-833); proven environmental, filed as follow-up.
- **Onboard-managed installed trees (.claude/skills, .agents/skills) do
  not refresh inside a worktree** when versions match — they sync from
  the main checkout's source root post-merge. Renders inside the
  worktree cover the plugin trees only; plan the post-merge onboard
  as an explicit step.
