# vd-4 — Prove gating from the real state machine and retire every hand-written phase fixture

**Outcome:** [DONE] — Per D4, migrated the 4 remaining slice-1 test sites off
the retired `'validating'` phase literal. `test_conformance.mjs`'s scenario3
(bee's canonical pre-execution-gate-deny proof) is rewritten to drive the
REAL state machine (`state start-feature` + `state set --phase planning`),
never a hand-built fixture — the D4 anchor. Also fixed 2 live regressions the
derived sweep's cousin-work turned up: `test_bypass_stop_net.mjs` had 2
pre-existing failures (vd-1's `PHASE_GATE={planning:'execution'}` made its
"Gate 2/shape" wording stale) and `test_msn_invariants.mjs` had 1 (a hand-built
`phase:'validating'` fed straight into `checkWrite` in-memory now hits vd-2's
new unknown-phase deny tail). `test_hook_contracts.mjs`'s unrelated nickname
fixture also swapped `'validating'` → `'planning'`.

Added 3 net-behavior scenarios to `test_conformance.mjs` (slice-tail
coverage): the merged-gate happy path (`state gate --merge` unblocks the
write-guard binary), the legacy-lane migration (a pre-cut lane record's
`'validating'` phase coerces to `'planning'` via the real `state lanes`
entrypoint, and the live machine now refuses to reconstruct `'validating'`
via `state start-feature --phase`), and the four still-allowed phases
(reviewing/scribing/compounding/grooming).

**Derived sweep (completeness evidence, D4):**
`rg -lw validating packages/bee scripts .bee/bin --glob '!node_modules'`
re-run clean after all edits: every remaining hit is either owned by an
earlier slice-1 cell (`test_guards.mjs`/`test_bee_write_guard_hook.mjs`,
vd-2 — not touched), a doc/source file naming the separately-deleted
`bee-validating` SKILL (D1, out of scope), or prose in my own files
explaining the migration — except `packages/bee/hooks/test_write_guard.mjs:664`,
an out-of-scope file (not declared, not owned by vd-1/2/3) that still
hand-builds `phase:'validating'` for its apply_patch gate-policy rows;
verified currently GREEN (the write-guard hook's own `readState` call
transparently coerces it to `'planning'` before `checkWrite` runs, same
shape as vd-2's own left-untouched fixtures) — reported per this cell's
scope boundary, not edited.

**Verify:** `node scripts/tests/test_conformance.mjs && node packages/bee/tests/test_guards.mjs && node packages/bee/hooks/test_bypass_stop_net.mjs`
```
test_conformance.mjs: ALL PASS (13 scenarios)
test_guards.mjs: 62 passed, 0 failed
test_bypass_stop_net.mjs: ALL PASS (15 rows)
```
Also run separately (declared files outside the recorded chain):
`test_hook_contracts.mjs` → ALL PASS (201 rows); `test_msn_invariants.mjs` →
17 passed, 0 failed (was 16/1 before the fix). Full before/after text:
`.bee/cells/vd-4.json` trace.verification_evidence.

**Files + commit:** `scripts/tests/test_conformance.mjs`,
`packages/bee/hooks/test_bypass_stop_net.mjs`,
`packages/bee/hooks/test_hook_contracts.mjs`,
`packages/bee/tests/test_msn_invariants.mjs`. Commit carries `vd-4`. Full
trace/evidence: `.bee/cells/vd-4.json`.

**Deviations:** none (no cell-text redirect; fixed 2 stale assertions found
red in one of my own declared files, folded a now-redundant scenario into
its sibling — recorded in the cap outcome, not a scope change).

**Outstanding finding (not mine to fix):** `packages/bee/hooks/test_write_guard.mjs:664`
still hand-builds `phase: "validating"` (out of this cell's declared files
and not owned by vd-1/2/3) — currently green only via transparent read-side
coercion; worth a follow-up cell if the estate is ever swept again.
