# exec-speed — CONTEXT

Feature: `exec-speed` · Lane: standard · Flags: public-contracts, multi-domain
Origin: user-requested execution-speed overhaul after a measured harness review
(timings.jsonl aggregates + direct hook benchmarks, 2026-07-30). User approved
both work groups explicitly: "Làm cả 2 nhóm A, B cho tôi."

## Problem

1. Every bee CLI invocation pays ~100ms node startup + full 1.3MB static import
   graph; small verbs are 4–10ms in-process, so startup dominates ~90% of wall
   time. A cell lifecycle makes 10–15 invocations → 2–3s pure overhead per cell.
2. The verify chain runs `knowledge check` + `knowledge index --check` per
   impacted target (3,212 logged invocations) though both are repo-global.
3. Hooks on every tool call import `state.mjs` (195KB) just to read
   `config.hooks[name] !== false`; read-only tools (Glob/Grep/Read) pay the full
   write-guard.
4. Reservations are one CLI call per path.
5. `bee-state-sync` rescans the whole cells store + rebuilds projections on
   every TodoWrite/SubagentStop/Stop.
6. Doctrine forces a dispatched execution worker, 4-command worker startup,
   per-behavior-cell judge dispatch, ~10–14 tick lines, per-cell report file,
   draft-PR per cap, and an unconditional high-risk pre-cap advisor consult —
   even where a lighter form loses no verification quality.

## Locked decisions

- **D1 (A1):** Enable `module.enableCompileCache()` (Node ≥22, guarded) in
  `bee.mjs` and `hooks/adapter.mjs`. Lazy-load heavy command groups only where
  it is a mechanical, low-risk split; correctness over completeness.
- **D2 (A2):** `run_verify.mjs` dedupes repo-global verify commands: identical
  command strings run once per verify run, result reused per target.
- **D3 (A3):** `bee-tools-logger` (and any hook needing only the toggle) reads
  `.bee/config.json` directly via a tiny `adapter.mjs` helper — no `state.mjs`
  import. `bee-write-guard` gets an early fast path: Glob/Grep and Read below
  the size threshold short-circuit through a static scout/secret path check
  without importing `state.mjs`/`guards.mjs`. Full guard behavior for
  Edit/Write/MultiEdit/Bash is unchanged.
- **D4 (A4):** `reservations reserve` accepts multiple paths in one call
  (repeatable/comma `--path`), atomic batch semantics: all-or-nothing conflict
  reporting. Single-path behavior unchanged.
- **D5 (A5):** `bee-state-sync` skips the `listCells()` scan + projection
  rebuild when the cells store is unchanged since the last sync (mtime/count
  stamp under `.bee/logs/`), same pattern as the inject cache. Heartbeat/hold
  renewal always still runs.
- **D6 (B1):** Tiny lane (≤2 product files, 0 flags) MAY execute inline in the
  orchestrator session; the merged gate, cap discipline, feature-verify law,
  ticks, and capture discipline are unchanged. small+ keeps dispatched workers.
- **D7 (B2):** Worker startup ceremony trimmed: the dispatch prompt inlines the
  full cell JSON and the state-at-dispatch line; workers no longer run
  `status --brief --json` or re-read the cell via `cells show` (ownership is
  enforced at cap by `guardClaimOwnership`). AGENTS.md + CONTEXT.md reads stay.
- **D8 (B3):** Semantic checklist judge runs once per slice at slice close over
  all capped `behavior_change` cells of that slice (one dispatch, one verdict
  per cell recorded via `cells judge-record`), replacing one dispatch per cell.
- **D9 (B4):** Tick diet: tiny/small emit composite ticks (one line per phase,
  `✗` never composited away); the concurrency-plan line is waived when exactly
  one cell is dispatched; tiny default ship visibility is `push-only`.
- **D10 (B5):** The per-cell report file becomes conditional: written only for
  `[BLOCKED]`/`[HANDOFF]` results or cells carrying consults; routine `[DONE]`
  cells rely on the cap trace + status token (extends decision 0009's
  single-source-trace rationale).
- **D11 (B6):** The unconditional high-risk pre-cap advisor consult keeps its
  code gate but the evidence bundle becomes a compact digest (diff summary +
  cell id + CONTEXT path, not full file excerpts). The on-failure consult loop
  (max 2/claim) is unchanged.

## Constraints

- No git in this checkout: one-commit-per-cell and push/PR duties are
  inapplicable here; caps record files + outcome only. Noted, not silent.
- Doctrine edits must never state something more permissive than the code
  enforces (worker-conformance rule). Where code enforces the old behavior
  (advisor_ref gate, judge guards), code and doctrine change together.
- Verify: `node scripts/run_verify.mjs` (full), impacted-from-git unavailable.

## Out of scope

- Full lazy-import refactor of all of bee.mjs beyond safe mechanical splits.
- Re-wiring .claude/settings.json (user removed it; a slim proposal file is
  delivered instead, adoption is the user's call).
