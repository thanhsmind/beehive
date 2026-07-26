# Advisor consult — slice 5 execution (integration queue + invariant closure + release)

Advisor: claude/fable (ceiling), read-only, 2026-07-25. VERDICT: proceed-with-conditions.

Binding conditions:
- A (msn-22): queue wraps the `worktree.merge` CLI verb only — never dispatch-interlock.mjs; herding merge (human single-shot gesture, herding.mjs:11-13) untouched.
- B (msn-22): define the drainer — enqueuer becomes processor when the lease frees, bounded-wait its turn; a position-returned merge NEVER reads as success in CLI text.
- C (msn-22): processor lease ttl strictly positive + heartbeat-renewed through the multi-minute P2 verify (lease-store expires_at null on non-positive ttl = queue deadlock); fence-takeover epoch-bumped; P3 re-checks lease epoch after re-acquiring worktree-admin (existing staged-tree/HEAD fence = second line against zombie double-commit).
- D (msn-23): invariant 12 is unprovable universally — scope its test to enumerated long ops (merge-verify child, queue processing) + a grep-guard for withStoreLock wrapping spawnSync; note "not universal" explicitly.
- (msn-23): index suite must FAIL LOUD if an underlying imported suite is absent/skipped; each import asserts the SPECIFIC invariant; fresh tests for gaps 5/7/15 (7's advisory path guards.mjs:659-701).
- E (msn-24): original premise FALSE — writeHandoff is the live C1 no-workflow fallback (bee.mjs:3357) and rebuildHandoffProjection writes the file (state-projection.mjs:410). writeHandoff STAYS one more release; only reclassify projection writer + deprecation notes. (Contract patched at authoring.)
- F (msn-25): a red during release verify may NOT be waved as "known flake" — no test_store_lock file exists; the v1.16.1 red was never root-caused. Re-run to confirm determinism; persistent red blocks the tag and files verify-red. Wait for the in-flight CI run on main before tagging. Record the FULL-suite foreground output explicitly in cap evidence (cell verify field only gates impacted).
- Truth-chain consistency: 23's invariant-15 scoping exists BECAUSE 24 keeps writeHandoff, and 25's issue-56 closure must disclose exactly that deferral — three statements stay consistent.
- Template-vs-runtime trap: edit skills/bee-hive/templates/, vendor via onboard — never .bee/bin directly.
