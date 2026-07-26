# installer-verify-extra-drift-1

**Status:** DONE

**Outcome:** `scripts/install.sh`'s final verify step no longer hard-fails when the only
drift is an unmanaged `.mjs` file in `.bee/bin/lib/` (a `drift_detail` entry suffixed
`" (extra)"`) with versions already matching expected — it now prints the specific
extra file path(s) and continues to the `up_to_date` recheck. Any real hash-mismatch,
missing-file, or version-mismatch entry still hard-fails exactly as before. A red-first
regression test was added and the release manifest was regenerated.

**Files touched:**
- `scripts/install.sh` — verify node snippet now inspects `s.onboarding.drift_detail`
  before treating `drift === true` as fatal.
- `scripts/test_installers_e2e.mjs` — new `check()` planting an unmanaged extra `.mjs`
  in a clean target's `.bee/bin/lib/` and asserting the installer's verify step succeeds
  and names the planted file.
- `docs/history/codex-harness-hardening/release-manifest.json` — regenerated via
  `render_plugin_skill_trees.mjs` → `onboard_bee.mjs --apply` → `release_manifest.mjs --write`.

**Verify:** `node scripts/test_installers_e2e.mjs && node scripts/release_manifest.mjs --check`
→ 24 passed, 0 failed; manifest check passed (510 files match).

**Commit:** `7f58fb9` — fix(installer): extra-file-only drift warns instead of
hard-failing install.sh verify [installer-verify-extra-drift-1]

**Cell:** `.bee/cells/installer-verify-extra-drift-1.json`
