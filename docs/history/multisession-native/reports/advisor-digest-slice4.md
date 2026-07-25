# Advisor consult — slice 4 execution (workspace isolation by default)

Advisor: claude/fable (ceiling), read-only, 2026-07-25. VERDICT: proceed-with-conditions.
(Recovered 2026-07-25 after a worker git-stash incident dropped this untracked file; content restored verbatim from the orchestrator's session context.)

Binding conditions:
1. File anchors FIXED at authoring (F1/F2): msn-17 edits lib/state.mjs (resolveRoots lives at state.mjs:748, NOT fsutil.mjs); msn-21 edits hooks/bee-write-guard.mjs at repo root (templates/hooks/ does not exist — confirm vendoring reality before editing).
2. (F3) msn-18 re-roots the guard's lane/workflow read IN-CELL or ships a compat shim + a "worktree write passes the lane guard" truth — no broken window across cells 18→20 (checkWrite → resolveWriteRecord → resolvePipeline reads lanesDir; unresolvable lane = typed hard deny, guards.mjs:44).
3. (F4) msn-18 declares which plane cells/backlog/decisions live in, proves claim→cell resolution stays same-plane, and MIGRATES-OR-FAILS-LOUD on pre-existing worktree-local sessions/claims/leases. Never silently orphan in-flight data.
4. (F5) msn-19: write_owner_session COEXISTS with writeGrant (grant = store topology, ownership = live-session lock); never subsumes; compose test required.
5. (F6) msn-17 makes resolveContext the SINGLE git-common-dir resolver; herding.mjs:24-30 and command-registry.mjs:1947 reconcile or carry a tracking note.
6. (F7) msn-20 auto-isolation: explicit consent/--isolate acknowledgement or config opt-out surfaced once; loud one-line cost disclosure; register/create writes proven CLI-owned-allowlisted (guards.mjs:168-175 pattern) so isolation cannot self-deadlock.
7. Every cell keeps a worktree-topology test in --impacted scope, including the guard in msn-21.

Re-slice addendum (advisor re-review after msn-18 honest block): 18a (controlRoot=mainRoot accepted as least-churn D2 reading; localRuntimeRoot-distinct test BINDING) → 18b (cells/reservations/recovery/compaction/state-projection sweep) → 18c (bee.mjs sweep — standalone, NEVER folded into msn-21, lands before msn-19) → 18d (onboard migrate-or-fail-loud, gates before 19/20).
Biggest risk: msn-18 family — stores the guard and grant system read; granted worktrees with live in-flight data are the population that silently breaks.
