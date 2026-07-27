---
type: bee.area
title: Verify Pipeline — suite topology and discovery
description: "Keeping full-repo verification fast and contention-free by giving every module its own suite file, discovering suites by convention instead of a hand-registry, failing loudly the moment a curated suite goes missing, and capping the transitive impacted run so a hot-file fan-out never quietly balloons back into a full run."
timestamp: 2026-07-25
bee:
  id: verify-pipeline-suite-topology-and-discovery
  lifecycle: active
  areas: [verify-pipeline]
  decisions: [contention-split D1-D6 (decision 1ce777d9), verify-scoping D1/D2 (decisions e39d3f89, 20534ea9), impacted-level1 D1 (decision 4f8295fb), i54-closeout D2, "test-economy D6 (impacted cap over the transitive tail — level-1 direct/self-selected always run, exit reflects only the part actually run, best-effort CI delegation)"]
  sources: ["contention-split cells cs-1/cs-2a/cs-2b/cs-3/cs-4 (fixture extraction, monolith split 430-check conservation, monolith deletion, convention-based suite discovery; traces in .bee/cells/, 2026-07-20)", "hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — Windows CI runs the real split suites through the runner's own discovery rather than a hand-maintained list; write-guard-hook-fix wgf-1, 2026-07-21 — the fixture that vendors a module tree copies the tree, never a hand-maintained file list)", "verify-scoping cells vs-1/vs-2 (scoped --only include filter + two-tier verify doctrine; traces in .bee/cells/, 2026-07-23)", "impacted-level1 cells l1-1/l1-2 (registry per-edge depth split + run_verify --level 1 direct-only selection; traces in .bee/cells/, 2026-07-23)", "test-economy cell te-3 (impacted cap — transitive tail delegated to CI; docs/history/test-economy/CONTEXT.md, trace in .bee/cells/, 2026-07-25)", "docs/specs/verify-pipeline.md#R1", "docs/specs/verify-pipeline.md#R2", "docs/specs/verify-pipeline.md#R3", "docs/specs/verify-pipeline.md#E1", "docs/specs/verify-pipeline.md#P1", "docs/specs/verify-pipeline.md#P2", "docs/specs/verify-pipeline.md#P3", "docs/specs/verify-pipeline.md#P4", "i54-closeout cell i54-closeout-2 (run_verify per-suite wall-clock timeout + heartbeat; trace in .bee/cells/, 2026-07-24)"]
  authoritative_for: "verify-pipeline: suite topology and discovery"
---

# Verify Pipeline — Suite Topology and Discovery

Keep the full-repo verification fast, complete, and — above all —
contention-free: two features working in parallel must not need to edit the
same test artifact unless they genuinely change the same module. This concept
owns how suites are shaped and found; how the run itself stays
concurrency-safe and hermetic is `concurrency-and-hermetic-runs.md`.

## Behaviors & Operations

- **Per-module suites, no monolith.** Every lib module or CLI area owns its
  own standalone suite file. The former single-file monolith (9.6k lines,
  ~100 sections, 430 checks) was split by contiguous section ranges with an
  exact check-count conservation proof (430 = 204 + 226) and then deleted.
  A new module brings a new suite file beside it, never a section appended
  to a shared file.
- **Shared fixture helper.** Suites import their temp-repo bootstrap,
  cell factory, and check-runner from one helper
  (`scripts/lib/test-fixture.mjs`) instead of duplicating setup. Each suite
  file owns its own temp root; cross-suite state sharing is prohibited.
- **Convention-based discovery.** The runner discovers suites by glob
  (`test_*.mjs` under the four test roots) plus a small EXTRA list for
  legacy names and an EXCLUDE list for non-suite helpers. Adding a test
  file requires zero runner edits — the registry hotspot is gone.
  Serial-sensitive routing (lock/race/timing suites run in one serial
  chain) derives from filename convention (`*_race`, `*_lock`,
  `*_concurrency`) plus a named exception list.
- **Deletion still fails loudly.** The manifest guard asserts a floor
  count, on-disk existence of every curated mandatory suite, and their
  membership in the discovered set — a vanished suite fails verify even
  though nothing "registers" it anymore.
- **Windows CI proves the real suites, not a stand-in list.** The split
  per-module template suites run on Windows CI through the runner's own
  discovery mechanism — a root-filter environment variable narrows discovery
  to the relevant roots, and discovery refusing to run when that filter
  matches zero suites is itself a guarded failure — rather than through a
  second, hand-maintained enumeration of which suites "should" run on
  Windows. There is exactly one place a suite is ever listed: the discovery
  convention itself (hardening-1-7-10).
- **Scoped runs for development iteration.** The runner accepts an include
  filter — a repeatable/comma `--only <token>` CLI flag or `BEE_VERIFY_ONLY`
  env (CLI wins) — matching runnables by case-insensitive substring on
  repo-relative path and display name, EXTRA list included, applied before
  the exclude filter. Zero matches is a typed refusal (exit 1), never a
  silent trivial green, and every scoped run prints a loud
  `SCOPED RUN (--only)` banner twice so a scoped green can never be mistaken
  for a full one (verify-scoping D1).
- **Impact-registry-scoped dev loop (ci-owned-verify D3/D4/D5).** The dev
  loop's own broader command, `commands.test`, runs `run_verify.mjs
  --impacted <files>` / `--impacted-from-git`, mapping changed files through
  the committed `scripts/impact-registry.json` (derived, never
  hand-authored — `impact_registry.mjs --write`/`--check` regenerates and
  byte-compares it) to the exact suites they can affect; a changed suite
  selects itself, unmapped changed files are listed loudly rather than
  silently dropped, and zero impacted suites is a loud pass ("full verify
  delegated to CI") rather than a silent trivial green. Cell `verify`
  commands are authored from the same registry's `--query <file...>`
  answers. The registry records each file→suite edge's depth (direct vs
  transitive); mid-iteration, `run_verify --impacted-from-git --level 1`
  selects direct edges only (seconds), while the transitive impacted run
  (`commands.test`) stays the wave-close/merge gate (impacted-level1 D1).
- **The transitive impacted run has a ceiling, and level-1 is exempt from
  it.** A single hot file (imported almost everywhere) can make the
  transitive impacted set balloon to nearly the whole suite pool, erasing
  the point of running "impacted" at all. `run_verify.mjs` reads a `0..1`
  ratio from `.bee/config.json`'s `verify_impacted_cap` key (a missing file,
  a parse failure, or a missing key all fail safe to the default `0.30`; a
  configured `1` is the documented off-switch, since a ratio can never
  exceed 1). The cap applies ONLY to a transitive selection (bare
  `--impacted`/`--impacted-from-git`, no `--level` flag) — a caller that
  already asked for `--level 1` is unaffected: no banner, no cut, today's
  behavior unchanged. When the transitive selection's share of the full
  suite pool exceeds the cap, the run re-queries the SAME changed-file set
  at level 1 instead: every self-selected suite (a changed test file maps
  directly to itself) and every suite that imports a changed file directly
  still runs — only the pure-transitive tail is dropped. The run then prints
  `impacted N/<total> exceeds cap (<cap>) — transitive tail delegated to CI`,
  and its exit code is the result of the part that actually ran (a red suite
  inside the capped selection still fails the run) — never the unconditional
  exit-0 the unrelated zero-impacted/unmapped branch prints for a
  structurally different reason (that branch ran zero suites; this one runs
  a real, just-narrowed selection). The delegation itself is a best-effort,
  client-side `gh workflow run CI` nudge — it never touches a workflow file,
  and a missing or erroring `gh` just logs a note, since CI already runs on
  its own cadence and would pick up the deferred tail regardless
  (test-economy D6).

- **A hung suite is killed and named, never left to hang the whole run.** Every
  suite the runner launches carries a per-suite wall-clock timeout (default
  300s, overridable via `BEE_VERIFY_SUITE_TIMEOUT_MS`; `0`/`none` disables it).
  On expiry the runner kills the suite's whole child process group — not just
  the direct child, so a grandchild the suite itself spawned cannot outlive it
  either — and reports a distinct `TIMEOUT` status, never conflated with an
  ordinary `FAIL`. A stderr heartbeat (default every 30s, overridable via
  `BEE_VERIFY_HEARTBEAT_MS`) names whichever suites are still in flight, so a
  long-running full verify never reads as frozen. Hermetic env scrub and
  non-timeout pass/fail semantics are unaffected (i54-closeout D2). One
  platform override exists: the Windows portable-suites CI job sets
  `BEE_VERIFY_SUITE_TIMEOUT_MS=600000` (600s) because Node-20-on-windows-latest
  runs the spawn-heavy git suites 2-4x slower than Linux (test_worktree_store
  was killed at exactly 300000ms while the Node-22 job on the same commit ran
  green — slowness, not logic); the 300s default stays everywhere else, and the
  job-level `timeout-minutes: 30` still bounds the whole run (win-ci-timeout
  D1, decision e2373374).

## Business Rules

- **R1** — A test suite is one file, one module/area, one temp root; adding
  tests for module X touches only X's suite file.
- **R2** — Suite membership is discovered, never hand-registered; exclusions
  carry a dated comment naming the reason.
- **R3** — Check-count conservation is the required evidence for any test-file
  migration (counts recorded before/after; additive setup fixes allowed,
  weakened or dropped checks are not).
- **R4** — Verify is two-tier, but the full tier moved off the machine
  (verify-scoping D2, superseded by ci-owned-verify D1/D5/D6): a cell's
  `verify` command is the narrowest honest scoped check covering its change
  (a direct test file or `--only` selection); the dev loop's own broader
  check is `commands.test` (the impacted run, `run_verify.mjs --impacted` /
  `--impacted-from-git`), resolved through the impact registry — never the
  full configured verify. The FULL configured verify (`commands.verify`) is
  CI-owned: it runs on the project's own CI cadence (push, nightly, or
  scheduled — the host workflow decides), never locally, and auto-files a
  deduped `verify-red` issue when red — no session baseline, feature close, or
  worktree-merge moment runs it locally by default; worktree merge gates on
  `commands.test` instead. Cap evidence is the cell's scoped verify; the full
  run belongs to CI. Mid-iteration, the level-1 impacted run (direct edges
  only) is the fast local check; the transitive impacted run (`commands.test`)
  remains what gates wave-close and merge (impacted-level1 D1). A repo that
  declares itself no-test (`commands.verify`/`commands.test` set to the exact
  sentinel `"none"`, decision 55b951e1) skips both tiers loudly instead: the
  session preamble, wave-close, session-finish, and worktree merge each print
  one disabled-gate line rather than running anything, and a cell's own
  `verify` may itself be `"none"` there, capped on the diff-backed outcome
  plus a recorded waiver note.
- **R5** — A transitive impacted run (no `--level` flag) is capped at
  `verify_impacted_cap` (`.bee/config.json`, default `0.30`, `1` disables the
  cap) of the full suite pool; level-1 direct edges and self-selected suites
  always run regardless of the cap, only the pure-transitive tail is ever
  dropped, and the run's exit code always reflects the part that actually
  ran — a capped run is never a silent green. An explicit `--level 1` run is
  never subject to this cap (test-economy D6).

## Edge Cases Settled

- **E1** — Discovery flip surfaced 4 orphan test files the hand registry never ran;
  3 were adopted (suite count 47→50), 1 (`test_bee_write_guard_hook`)
  fails standalone against the live hook and is excluded with a dated
  comment — filed as a P1 fix-first item, not silently adopted or trusted.

## Open Gaps

- The CLI dispatcher (4.4k lines, all handlers inline) remains the last
  structural hotspot; handler extraction is deferred to its own feature
  (contention-split D5).

## Resolved Gaps

- The guard-hook suite's 9/21 failures (P1, 2026-07-20) were TEST-WRONG with
  one shared cause: the fixture's hand-maintained lib-module list under-
  vendored the hook's import closure (10 listed vs 17 needed), so the hook's
  first dynamic import threw and its documented fail-open caught every deny
  path. The hook had no hole. Fixed 2026-07-21 (write-guard-hook-fix wgf-1):
  the fixture now vendors the lib directory wholesale — killing the stale-
  hand-list class, which had bitten at least twice — and the suite runs in
  CI for the first time. Corollary rule: R6 — a fixture that mirrors a
  module tree copies the tree, never a hand-maintained file list. (R6 is a
  post-migration corollary discovered after this area's pin was cut; it
  carries no numbered anchor of its own in the pinned source and is recorded
  here as prose, never invented into a claimed anchor — D10.)

## Pointers (implementation)

- **P1** — `scripts/run_verify.mjs` — discovery roots, EXTRA/EXCLUDE, serial
  convention, pool.
- **P2** — `scripts/tests/test_verify_manifest.mjs` — floor + existence + membership guard.
- **P3** — `scripts/lib/test-fixture.mjs` — shared fixture/check-runner.
- **P4** — `packages/bee/tests/` — per-module suites (11 files).
