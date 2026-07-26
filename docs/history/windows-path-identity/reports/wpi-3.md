# wpi-3 — repair two broken workflow step paths, guard them with a test, file what the exclusions really hide

**Status:** [DONE]

**Outcome:** Fixed windows.yml's two broken run-step paths (`:114` portable
paths guard, `:117` config validate — both pointed at `scripts/<file>.mjs`,
the files live under `scripts/tests/`). Added
`scripts/tests/test_workflow_step_paths.mjs`, a new suite (auto-discovered by
`run_verify.mjs`'s `scripts/tests` convention) that parses every
`.github/workflows/*.yml` file, extracts every `node <path>` invocation from
inline and block-scalar `run:` steps, and asserts each resolves to a real
file. Proven to discriminate three ways: (1) a fixture step pointed at a
nonexistent path is reported missing, (2) a fixture step whose path exists is
not, (3) run against a copy of the actual pre-fix committed windows.yml (via
`git show HEAD:...`), it independently reports exactly the two real paths
this cell fixed as missing. Updated windows.yml's exclusion comment block to
state what is true now instead of what the original draft assumed: the four
exclusions stand, one contributing cause (test_bee_cli's gitdir pointer
mismatch) was fixed by wpi-1, each remaining cause is filed below, and
removal of any exclusion is gated on a real windows-latest run showing that
suite green.

**No exclusion was removed, extended, or reinterpreted.** All four stay in
`BEE_VERIFY_EXCLUDE`.

**Files touched:**
- `.github/workflows/windows.yml` (two step paths fixed, exclusion comment block rewritten)
- `scripts/tests/test_workflow_step_paths.mjs` (new)

**Backlog rows filed (`.bee/backlog.jsonl`, type friction, severity P2, layer ci-windows, feature windows-path-identity)** — one per remaining Windows cause, each with exactly what is known:
1. `test_bee_cli.mjs` — 3 undiagnosed non-path failure classes named individually (doctor-wording, recovery-window/recovery-scan, registry-example); the gitdir-pointer class is the one wpi-1 already fixed.
2. `test_cells.mjs` — the independent permission-model cause, anchored at `packages/bee/tests/test_cells.mjs:1634-1639` (its write-failure simulation's `euid===0` skip never fires on win32; win32 `chmod` does not block directory-entry creation, so the forced write succeeds where the assertion expects it to fail).
3. `test_bee_write_guard_hook.mjs` — recorded as having NO diagnosed cause; the original "path normalization is the likely shared root cause" framing was an unverified hypothesis applied uniformly to all four, and it does not hold here.
4. `test_misc.mjs` — same as above, recorded as NO diagnosed cause.

**Honest verification limit:** this machine is not Windows. The local run
proves no POSIX regression in the fixed test paths and the corrected
workflow step invocations, and proves the extraction logic correctly
discriminates real from missing paths (including against the actual pre-fix
committed file). It proves **nothing** about Windows behaviour for any of
the four still-excluded suites — none of them were re-enabled, and no claim
is made here that any of them would pass on Windows.

**Verify:** `node scripts/run_verify.mjs --only scripts/tests/test_workflow_step_paths.mjs && node scripts/run_verify.mjs --only scripts/tests/test_portable_paths.mjs` — both green.

Full trace/evidence: `.bee/cells/wpi-3.json`.
