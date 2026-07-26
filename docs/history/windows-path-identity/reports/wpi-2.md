# wpi-2 — stop two tests asserting a contract the product never promised, canonicalize the herding resolver's comparison

**Status:** [DONE]

**Outcome:** Fixed `test_cli_cells.mjs`'s commit-scope assertion to compare
against the literal forward-slash path git's stdout always emits (never
`path.join`, which is native-separator and cannot match on win32). Fixed
both `test_herding_cli.mjs` assertions (marker at the old :70, main_root at
the old :71, plus the interlock-agreement check) to use wpi-1's
`canonicalPathsEqual` instead of `===`. `main_root`'s returned value,
`herding.mjs`, and the standalone `dispatch-interlock.mjs` twin were left
untouched, per validation's explicit prohibitions.

**Consumer sweep (must_have):** `skills/bee-herding/SKILL.md:142,303`'s
worktree grant-key derivation reads `main_root` from `bee.mjs worktree list
--json`, which resolves via `resolveMainRoot`/`resolveRoots`
(`bee.mjs`/`state.mjs`) — a distinct resolver from `herding.mjs`'s
`resolveHerdingMainRoot` (used only by `bee herding enable/disable/status`).
Neither returned value was touched by this cell.

## Rework round (goal-check judge, NEEDS_REVISION)

The first cap's red-first regression guards did not discriminate: reverting
all three corrected assertions to their pre-fix `===` form left the suite
fully green, because the guards compared static/synthetic values disconnected
from the code under test rather than the real resolver output. Compounding
it, the herding guard's separator sub-test asserted
`canonicalPathsEqual(backslashSpelling, root)` was `true` on POSIX — true
only because of wpi-1's then-unconditional backslash-to-separator fold,
which wpi-1's own rework (commit `ddcb6733`) subsequently removed as a real
bug, leaving this cell's verify red in the working tree.

Fixed both guards to route the REAL resolver output (`changedFiles[0]` from
git; `out.main_root` from the herding CLI) through wpi-1's injectable
`platformPath` seam (`path.win32.resolve` / `path.win32.join` vs the ambient
default) instead of a disconnected synthetic value, and added an explicit
POSIX-ambient control proving `canonicalPathsEqual` with no `platformPath`
injected correctly rejects a win32-shaped rendering — matching the reworked
module, not the removed bug. Manually reverted each fixed assertion in place
and re-ran: both guards flipped from green to exactly one expected failure
and back to green, confirming genuine discrimination (quoted in the cap's
`verification_evidence`). Capped with an audited `--override-judge` against
this cell's own recorded `NEEDS_REVISION`, since a worker cannot dispatch a
fresh judge pass itself.

**Files touched:**
- `packages/bee/tests/test_cli_cells.mjs`
- `packages/bee/tests/test_herding_cli.mjs`
- `docs/history/codex-harness-hardening/release-manifest.json` (regenerated, both rounds)

Full trace/evidence: `.bee/cells/wpi-2.json` (two cap events, two judge_overrides entries).

## Finding for the orchestrator (from the first round, since resolved)

While capping the first round, a background semantic-judge run against the
**dependency** cell wpi-1 returned `NEEDS_REVISION` for the same unconditional
backslash-fold defect this cell's own rework round above addresses on the
consuming side. wpi-1's rework (`ddcb6733`) has since landed and was
independently judged `PASS`; this cell's rework is compatible with the
reworked module (verified: `scripts/tests/test_path_identity.mjs` PASS,
unaffected).
