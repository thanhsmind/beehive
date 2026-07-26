# packages-restructure-1 — report

**Status:** [DONE]

**Outcome:** `git mv skills/bee-hive/templates packages/bee` (history-preserving), then fix-forward every
path reference across `scripts/`, `skills/`, `.github/`, `packages/bee/**` so the full suite stays green:
`onboard_bee.mjs` now resolves its vendored payload `PLUGIN_ROOT`-relative (D3) instead of self-relative;
`packages/bee/lib/guards.mjs`'s write-guard regex covers `packages/bee/lib` alongside `bin/lib`;
`release_manifest.mjs` gained a `package_payload` role enumerating `packages/bee/**` (D5); `plugin_distribution.mjs`
accepts the new role; `run_verify.mjs`, `install.ps1`, and `.github/workflows/windows.yml` all repointed at the
new tree; impact registry, plugin skill trees, and the release manifest were regenerated.

**Files touched:** 493 files — see `.bee/cells/packages-restructure-1.json` trace `files_changed` for the full
list (top-level: `packages/bee/**` new tree, `skills/bee-hive/scripts/*.mjs`, `scripts/**`, `.github/workflows/windows.yml`,
`.claude-plugin/`, `.codex-plugin/`, `.claude/`, `.agents/` skill mirrors, `docs/history/codex-harness-hardening/release-manifest.json`,
`scripts/impact-registry.json`, `docs/backlog.md`).

**Verification:** `BEE_VERIFY_CONCURRENCY=12 node scripts/run_verify.mjs && node scripts/release_manifest.mjs --check`
→ PASS, 105 suites green, 404-file manifest match. Full trace/evidence: `.bee/cells/packages-restructure-1.json`.

**Deviations (auto-fixed, in scope of touched code):**
1. `onboard_bee.mjs`'s three-version preflight read `installed_skills` from a marker no synced skill dir will ever
   carry again (skills are instruction-only post-move) — added a sync-owned `SKILLS_VERSION_STAMP`
   (`.bee-skills-version.json` at each managed skills root, written on every successful apply, gated so a
   skipped/blocked `bee-hive` sync never falsely reports parity) with a legacy-marker fallback so an
   already-onboarded host self-migrates on its next onboard instead of permanently bricking (`unknown`, never
   forceable). Consulted the advisor (fable) once on this design fork after two failed mechanical attempts;
   the shipped design is the advisor's recommendation, verified against the real suite.
2. Six test files under `packages/bee/tests/` used a hand-counted 4-up `path.resolve`/`path.join` to reach the
   repo root that was one level too many post-move (`test_herding.mjs`, `test_knowledge.mjs`,
   `test_bee_write_guard_hook.mjs`, `test_bundle_mode.mjs`, `test_herding_cli.mjs`) plus a 3-up case in
   `test_bee_cli.mjs` that was silently self-skipping instead of failing.
3. Two stale references to files moved by an earlier, unrelated repo refactor (`scripts/test_X.mjs` →
   `scripts/tests/test_X.mjs`, decision stm-1) that this cell's own verify run surfaced:
   `test_msn_invariants.mjs`'s `TEST_WORKTREE_MERGE_QUEUE` constant, and `docs/backlog.md` (regenerated via
   `bee backlog render --write`; confirmed present on baseline before this cell's changes — pre-existing,
   unrelated drift, fixed because it blocked a clean `run_verify.mjs` pass).

**Deliberate exceptions (not fixed):**
- `skills/bee-swarming/CREATION-LOG.md` and `skills/bee-evolving/CREATION-LOG.md` still name the pre-restructure
  path inside **verbatim historical command transcripts** (RED/GREEN proof blocks captured when those features
  closed). Rewriting a `$ node skills/bee-hive/templates/tests/test_lib.mjs` transcript to a path that did not
  exist when that command actually ran would falsify the record, not correct it. This is the only remaining hit
  against the cell's rg-clean must_have (`rg 'skills/bee-hive/templates' scripts/ skills/ .github/`); every
  live/executable reference is clean.
- `onboard_bee.mjs`'s legacy **global** skill root check (`~/.claude/skills`, opt-in `--global-skills`) keeps its
  old nested-marker resolution unchanged — no repo-root sibling exists for that root, matching D4's explicit
  scope limit (onboarding-engine changes deferred).

**Consults:** 1 — advisor `fable`, on the `SKILLS_VERSION_STAMP` design fork above. Ask: after two failed
mechanical fixes to the three-version preflight's `installed_skills` marker, what is the right minimal fix given
skills are now instruction-only. Answer: a sync-owned stamp file at each skills root (not inside a skill dir, so
the per-skill mirror never prunes it) with a legacy-marker fallback for pre-restructure targets — implemented
and verified green.

Full trace and evidence: `.bee/cells/packages-restructure-1.json`.
