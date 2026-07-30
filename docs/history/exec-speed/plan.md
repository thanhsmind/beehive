# exec-speed — plan

Lane: standard · Decisions: D1–D11 in CONTEXT.md · No git in this checkout
(commit/push duties inapplicable; caps record files+outcome; full verify =
`node scripts/run_verify.mjs`).

## Baseline facts the plan builds on

- ~59ms node boot + ~45ms parse of the 1.3MB static graph per bee.mjs call;
  small verbs 4–10ms in-process (timings.jsonl, 6,644 rows).
- Hooks pay 2 processes (~220–230ms) per matched tool call; `state.mjs`
  (195KB) imported for a 3-line `hookEnabled` read.
- ESM hoisting fact: `enableCompileCache()` inside a module cannot cache that
  module's own static import graph — only modules loaded AFTER the call
  (dynamic imports, spawned children via `NODE_COMPILE_CACHE`). So:
  - hooks: call it at `adapter.mjs` module top — every wrapper's later
    dynamic import (`state.mjs`, `guards.mjs`, `cells.mjs`) gets cached.
  - verify chain: `childEnv()` in run_verify.mjs sets `NODE_COMPILE_CACHE`
    so every suite child (incl. the two `bee.mjs knowledge` entries) caches.
  - bee.mjs itself: top-level call helps only later dynamic imports → the
    real lever is the lazy-import split (es-1) plus cache benefits when
    invoked under run_verify.
- `.bee/bin/**` is hash-ledgered in `.bee/onboarding.json` (managed.lib /
  managed.helpers / hooks) — every edited file's sha256 must be refreshed
  there or `status` reports drift (es-12). ledger_parity.mjs --check is
  ALREADY red on Windows at baseline (ESM `c:` URL bug — pre-existing, not
  ours); release_manifest --check is already red (no stored manifest in this
  host checkout). Feature-verify compares against the baseline run's failure
  set, not absolute green, if the baseline shows pre-existing reds.

## Slices

### Slice 1 — code (group A), wave-parallel where disjoint

- **es-1** `.bee/bin/bee.mjs`: guarded top-level `enableCompileCache()` to
  `.bee/tmp/compile-cache`; convert heavy single-consumer static imports
  (worktree-store, integration-queue, feedback, herding, knowledge, perf-type
  libs — only where every use site is inside handler bodies) to lazy
  `await import()`. Exports used by tests stay static. Timing target: small
  verbs (`cells show`) shed most of the ~45ms parse.
- **es-2** `scripts/run_verify.mjs` + `scripts/verify-cache-inputs.json`:
  `childEnv()` sets `NODE_COMPILE_CACHE` (repo-local cache dir); declare the
  two `bee.mjs knowledge` EXTRA_SUITES cacheable with inputs
  `docs/knowledge/**` + `.bee/bin/**` + `.bee/config.json`.
- **es-3** hooks: `adapter.mjs` gains module-top guarded
  `enableCompileCache()` + `hookEnabledLite(root, name)` (reads
  `.bee/config.json` with `.bee/config.local.json` overlay precedence,
  fail-open enabled); `bee-tools-logger.mjs` drops its `state.mjs` import for
  the lite reader; `bee-write-guard.mjs` gains an allow-only fast path for
  Read/Glob/Grep — static secret/scout/size checks via constants extracted to
  a small shared `lib/guard-lite.mjs` (guards.mjs re-imports the SAME
  constants — no duplicated policy); anything suspicious falls through to the
  full guard for byte-identical denies.
- **es-4** (after es-1, same file): `reservations reserve` accepts
  comma-separated multi-path `--path`; conflict check for ALL paths first,
  all-or-nothing reserve inside the same lock section; registry description +
  example updated.
- **es-5** `.bee/bin/hooks/bee-state-sync.mjs`: skip `listCells()` +
  projection rebuild when `.bee/cells` is unchanged (count + newest mtime
  stamp at `.bee/logs/state-sync.stamp.json`); heartbeat/hold renewal always
  runs.
- **es-12** (after es-1/3/4/5): refresh sha256 entries in
  `.bee/onboarding.json` for every `.bee/bin/**` file this feature edited.

### Slice 2 — doctrine (group B), serial (shared files)

Chain A (routing docs): **es-6** (inline tiny execution) → **es-8** (judge
per slice) → **es-9** (tick diet / concurrency-plan waiver / tiny push-only).
Chain B (worker docs): **es-7** (startup trim) → **es-10** (conditional
report file) → **es-11** (advisor digest).

Doctrine edits must stay no-more-permissive than code: verified — no code
guard forces per-cell judges (only NEEDS_REVISION blocks a cap) and the
advisor gate is kept (only the evidence bundle slims), so all six are
instruction-layer edits.

## Verify

Dev loop: targeted suites per cell (`run_verify.mjs --only <token>`). Close:
one full `node scripts/run_verify.mjs` compared against the baseline log
(`scratchpad/verify-baseline.log`) — no new reds; pre-existing Windows reds
recorded as-is. Then hook micro-benchmarks re-run (tools-logger, write-guard
on a Grep payload) and 3 timed bee verbs, before/after, into the done-report.
