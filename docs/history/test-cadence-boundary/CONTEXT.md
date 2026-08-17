# test-cadence-boundary — locked context

Locked from user directive (2026-08-17). Decision log id
`13ce1858-1d05-4ca3-9e31-23c26980a772` (test-cadence-boundary D1,
touches ci-owned-verify `08795382`).

## Locked decisions

- **D1 — boundary-only test cadence.** `bee cells finish` does not run
  `commands.test`. Per-cell test execution is removed entirely — NO
  config knob, no per-cell fallback. The one declared test command runs
  at the boundary only:
  - `bee close` when the work has no worktree;
  - `bee worktree merge` when it does.
  CI keeps running the same command on push/PR unchanged.
- **D1a — cap record honesty.** A cap must state that tests run at the
  boundary, not at the cap (exact trace marker shaped at planning). A
  cap never claims green it did not earn.
- **D1b — never-build-on-red survives at the boundary.** A red at
  close or merge refuses that boundary verb and becomes fix-first work,
  same as today's close behavior. The per-cell red-refuses-cap path is
  gone with the per-cell run.

## Motivation (user's words, condensed)

In real host projects `commands.test` is the full suite; running it at
every cell cap makes each cap slow. User explicitly rejected a config
knob — remove the path, do not gate it.

## Non-goals

- No change to CI.
- No change to `bee test` as a manual verb.
- No selective/impacted test mechanism in this feature.

## Sequencing

Started after uat-gate-before-merge merged (commit 7f9fed34) — its
merge-path edits land first; this feature edits merge/close on top.
