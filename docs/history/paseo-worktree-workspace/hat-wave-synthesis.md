# Hat wave synthesis — paseo-worktree-workspace

Seats: hat-facts-gaps, hat-alternatives, hat-user-impact (3 of 3 returned; none dropped).

## Taken into plan.md

- facts-gaps BLOCKER: pw-2 did not list `verbs/worktree/tests.rs`, which builds `Created` by hand (tests.rs:699, 736). Added.
- facts-gaps: env tests hold `crate::hooks::herding_env_lock()`; pw-1 proves `paseo_env_pairs`; read_first lists widened for pw-1 and pw-3.
- facts-gaps: `paseo_steer.rs:108` also spawns the Paseo CLI package; scrubbed in pw-1. `herding fork` stays out of scope, named.
- facts-gaps: tests that seed `paseo_workspace_id` call `archive_with_cli`, never teardown, so no test reaches a real daemon.
- facts-gaps: live proof now includes one herding run checked with `paseo inspect`.
- alternatives: reuse `path_identity::canonical_paths_equal`; build `first_json_document` from the existing `parse_run_agent_id` scan (now also `[`); one `paseo_ws::ensure_and_record` for worktree new and herding run; all D5 scrubs in pw-1.
- user-impact: per-call timeout 8 s and a `▸ paseo: registering workspace…` line; archive failure prints `FIX: paseo workspace archive <wks>`; run refusal FIX names the real paseo command; JSON key `paseoWorkspace` (camelCase) and absent outside Paseo; docs name the Paseo-terminal case.

## Dismissed

- user-impact: skip archive when the workspace has live agents — dismissed: merge cleanup and prune already refuse a worktree that a live bee session holds (`live_session_holds`), and a non-bee agent in a bee worktree workspace is outside this feature's contract; named in Out of scope.
- user-impact: one 15 s budget for the whole ensure step — dismissed in favor of 8 s per call: the plain per-call timeout already exists on `RealPaseoCli`, and three calls max out near 24 s only when the daemon hangs.
