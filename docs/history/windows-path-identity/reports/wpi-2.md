# wpi-2 — stop two tests asserting a contract the product never promised, canonicalize the herding resolver's comparison

**Status:** [DONE]

**Outcome:** Fixed `test_cli_cells.mjs`'s commit-scope assertion to compare
against the literal forward-slash path git's stdout always emits (never
`path.join`, which is native-separator and cannot match on win32). Fixed
both `test_herding_cli.mjs` assertions (marker at the old :70, main_root at
the old :71, plus the interlock-agreement check) to use wpi-1's
`canonicalPathsEqual` instead of `===`. Added a red-first regression guard —
a new row in the existing suite, not a new file — that constructs a
platform-style divergent spelling (backslash form + injected case-fold) and
proves the pre-fix raw `===` rejects it while the canonical comparison
accepts it, reproducible on this Linux box without a Windows machine.
`main_root`'s returned value, `herding.mjs`, and the standalone
`dispatch-interlock.mjs` twin were left untouched, per validation's explicit
prohibitions. Regenerated the release manifest for the two touched test
files' hash change.

**Consumer sweep (must_have):** `skills/bee-herding/SKILL.md:142,303`'s
worktree grant-key derivation reads `main_root` from `bee.mjs worktree list
--json`, which resolves via `resolveMainRoot`/`resolveRoots`
(`bee.mjs`/`state.mjs`) — a distinct resolver from `herding.mjs`'s
`resolveHerdingMainRoot` (used only by `bee herding enable/disable/status`).
Neither returned value was touched by this cell.

**Files touched:**
- `packages/bee/tests/test_cli_cells.mjs`
- `packages/bee/tests/test_herding_cli.mjs`
- `docs/history/codex-harness-hardening/release-manifest.json` (regenerated)

Full trace/evidence: `.bee/cells/wpi-2.json`.

## Finding for the orchestrator (not this cell's to fix)

While capping, a background semantic-judge run against the **dependency**
cell wpi-1 returned `NEEDS_REVISION` and reopened it (status flipped
`capped` -> `claimed`, verify fields cleared) — this happened during this
session but was not an action this worker took. The finding:
`canonicalPathsEqual`'s `normalizeSeparators` folds every literal backslash
to `path.sep` unconditionally on every platform, on the stated claim that
"backslash is illegal in a POSIX filename" — that claim is false, so on
POSIX two genuinely distinct directories where one has a literal backslash
byte in a path component can incorrectly compare EQUAL. This is a real gap
in wpi-1's helper, separate from this cell's scope (wpi-2 only consumes
`canonicalPathsEqual`, per the cell's explicit "no second implementation"
rule) and needs its own rework pass. `.bee/cells/wpi-1.json`'s reopened
state was deliberately left out of this cell's commit.
