# Advisor consult — slice 2 execution (workflow-first state)

Advisor: claude/fable (ceiling), read-only, 2026-07-25. VERDICT: proceed-with-conditions.

Binding conditions:
- C1 (msn-7, biggest risk): idempotent seed — a non-idle legacy state.json is materialized into a workflow record BEFORE any rebuild treats state.json as derived; projection readers fall back to legacy state.json when zero workflow records exist. Land in msn-6 or pre-7, never slice 5.
- C2 (msn-9): projected gate boolean = plan-rev-EFFECTIVE approval (approved && approved_for_plan_rev === plan_rev), tested via a plan_rev bump flipping the projected claim-guard boolean (claim gate reads cells.mjs:1611).
- C3 (msn-8): startFeature worker precondition excludes the calling session (excludeSessionId pattern, claims.mjs isConcurrentMode); solo starter never blocked by own heartbeat (state.mjs:1810 revisited in msn-8, not just worker verbs).
- C4 (msn-6/msn-10): one global lock order — sessions and workflow:<id> never held together (binding is a separate transaction committed after the workflow lock releases). AB-BA today = spurious LOCK_BUSY flake.
- C5 (msn-7): default-path mutation still writes state.json via resolveMutationTarget (bee.mjs:2060-2065) until msn-10 — scope the no-write-through prohibition to lane projections until msn-10 lands (or land 7+10 atomically).

Also binding from findings:
- F5 (msn-6): scope the global-HANDOFF startFeature precondition per-workflow (mirror the lane path's handoff.feature check, state.mjs:1935) or document residual coupling until msn-15.
- F7: msn-8 and msn-9 execute SEQUENTIALLY (both touch templates/lib/state.mjs + templates/bee.mjs).
- F8 (msn-7): rebuild-on-read preferred; bee-state-sync becomes a full idempotent rebuild under its existing try-once discipline, never a partial RMW.
