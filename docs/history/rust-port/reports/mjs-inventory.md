# mjs Mechanical Surface Inventory (rust-port scout)

Delegated inventory digest, 2026-07-26. Input to planning: this is the surface queen-bee must reproduce.

> **Corrections from fresh-eyes review (measured):** `bee.mjs` = 7,259 lines (not ~3200); `scripts/` holds 60 mjs files (not 14); `packages/bee/tests` 36 (not 27); `packages/bee/hooks` 16 (not 12); repo total 158 files / ~119,900 lines (not ~150 / 70–80k). Dedup **runtime** surface (`bee.mjs` + `.bee/bin/lib/` + `.bee/bin/hooks/`) = ~38,300 lines. Command registry = **116 command defs across 18 group prefixes** (state 23, cells 20, backlog 11, decisions 8, reviews 7, perf 7, worktree 5, knowledge 5, reservations 4, intent 4, feedback 4, config 4, capture 4, herding 3, recovery 2, tmp 1, doctor 1, dispatch 1, + status) — the tables below understate. Hook wiring lives in BOTH `.codex/hooks.json` and `.claude/settings.json`. Third node entry point: `.claude/statusline-command.sh:60`.
>
> **Loop-2 corrections (this note wins over ALL body tables below):** 19 group prefixes not 18 (`status` counts; `doctor` has 2 defs); `.bee/bin/lib` 35 files, `.bee/bin/hooks` 11 (9 `bee-*.mjs` impls + adapter.mjs + tokenize-command.mjs), `packages/bee/lib` 35, `packages/bee/scripts` 5; `bee.mjs` 7,259 lines; verify estate = **105 suites** (82 `test_*.mjs` across 4 roots + extras), not ~35. Hot-path profile of `status` (463 ms total): git spawnSync ~97 ms (reviews.mjs:401, worktree-store.mjs:436), transcript tail ~37 ms (recovery.mjs:69), JSON reads ~65 ms, GC ~28 ms, module compile ~15 ms — cold start is NOT the dominant cost for `status`. Recognizer coupling: `bee-write-guard.mjs:499` `DISPATCHER_RE = /^bee\.mjs$/i` must learn the new entry-point name at flip. Cache stores live at `.bee/cache/inject-cache.json` + `.bee/cache/manifest-hash.json` (top-level `.inject-cache.json` is legacy fallback); also live: `.bee/claims/`, `.bee/intent/`, `.bee/tmp/`, `.bee/spikes/`. 11 code-embedded `node .bee/bin/bee.mjs` strings emitted at runtime (bee-session-close ×4, inject.mjs ×2, dispatch-prepare ×2, bee-chain-nudge ×2, compaction ×1), mirrored in packages/bee/.

## 1. Entry points & counts

| Directory | Count | Ownership |
|-----------|-------|-----------|
| `.bee/bin/bee.mjs` | 1 | Unified CLI dispatcher; imports from lib/*.mjs |
| `.bee/bin/lib/` | 34 files | Core state, storage, command handlers, guards, hooks support |
| `.bee/bin/hooks/` | 10 files | Hook implementations (Session, Write, Model, State, Chain, Codex audit) |
| `packages/bee/lib/` | 34 files | **Source** — byte-identical vendored copies of `.bee/bin/lib/` |
| `packages/bee/hooks/` | 12 files | Source hooks + test_*.mjs suites |
| `packages/bee/tests/` | 27 files | Test suites for core lib modules |
| `packages/bee/scripts/` | 4 files | Plugin dist, onboarding, split-brain regression test |
| `scripts/` | 14 files | verify, bump, impact registry, release manifest, okf_* |
| **Total** | ~150 .mjs files | ~70,000–80,000 lines |

## 2. Hook entry points (`.codex/hooks.json`, 8 events, 12 hook commands)

| Event | Hook files |
|-------|-----------|
| SessionStart (startup/resume/clear/compact) | `bee-session-init.mjs` |
| UserPromptSubmit | `bee-prompt-context.mjs` |
| PreToolUse (Edit/Write/Bash/Read/Glob/Grep) | `bee-write-guard.mjs` (gate, reservation, privacy/scout, CLI-shape checks) |
| PreToolUse (Agent/Task spawn) | `bee-model-guard.mjs` (model tier validation) |
| PostToolUse (update_plan/TaskCreate/TaskUpdate) | `bee-state-sync.mjs` |
| PostToolUse (all) | `bee-tools-logger.mjs` |
| SubagentStart | `bee-codex-subagent-audit.mjs` |
| SubagentStop | `bee-state-sync.mjs`, `bee-chain-nudge.mjs` |
| PreCompact | `bee-session-close.mjs` |
| Stop | `bee-state-sync.mjs`, `bee-session-close.mjs` |

Transport: `.bee/bin/hooks/adapter.mjs` (stdin normalization, root discovery, fail-open crash logging).
Fail-open policy: hooks exit 0 on internal error (crash logged to `.bee/logs/hooks.jsonl`); exit 2 only for deliberate denials (write-guard gate/reservation/privacy).

## 3. CLI command groups (dispatcher `bee.mjs` ~3200 lines; registry `command-registry.mjs` schema v1.0, 39 command defs)

status · cells (list/ready/show/add/claim/claim-next/verify/cap/block/drop/tier/judge/archive/schedule/reset-budget) · reservations (reserve/release/list/sweep) · decisions (log/supersede/redact/active/search/archive/tag/render) · state (set/gate/plan-rev/worker/scribing-run/start-feature/lanes/session) · backlog · capture · reviews · feedback · knowledge · intent · tmp — 12 groups in practice (9 canonical + knowledge/intent/tmp).

## 4. Storage ownership (all formats frozen per D3)

| File | Owner | Write path |
|------|-------|-----------|
| `.bee/state.json` | state.mjs | writeState → writeJsonAtomic |
| `.bee/config.json` | state.mjs (read-only from CLI) | hand/managed |
| `.bee/decisions.jsonl` | decisions.mjs | appendJsonl |
| `.bee/backlog.jsonl` | backlog.mjs | appendJsonl (append-only) |
| `.bee/cells/<id>.json` | cells.mjs | withStoreLock + atomic write |
| `.bee/reservations.json` | reservations.mjs | writeJsonAtomic |
| `.bee/HANDOFF.json` (legacy projection) | state.mjs ← state-projection.mjs | via mailbox transition |
| `sessions/*/workflow.json` | workflow-store.mjs | atomic via fsutil |
| `.bee/logs/hooks.jsonl`, `timings.jsonl` | adapter.mjs / perf | fail-open append |

Atomicity: `writeJsonAtomic()` + `appendJsonl()` (fsutil.mjs), `withStoreLock()` for read-modify-write (GH #27.2).

## 5. Test system today (doctrine carries into D6)

- Entry `scripts/run_verify.mjs` (~450 lines): glob discovery (`test_*.mjs` in 4 roots) + 7 EXTRA_SUITES ≈ 35 suites, parallel pool with capped concurrency.
- Impacted runs: `scripts/impact-registry.mjs` + committed `impact-registry.json`; transitive closure over 4 edge types (static ESM import, dynamic import, spawn argv, runModuleWorker) — regex scan, no AST. `--impacted-from-git`; cap `verify_impacted_cap: 0.30`.
- Contention split: lock-sensitive suites serial, rest parallel.
- Full suite CI-owned.

## 6. Structural patterns the port must honor

- **Vendored dual-location**: `.bee/bin/lib` ≡ `packages/bee/lib`, enforced by `release_manifest.mjs --check` + ledger parity. Rust port collapses this: one compiled binary replaces both runtime copies (D2/D4).
- **Flat import discipline, no cycles**: leaves (fsutil, lock, claims, reservations) → state → cells → handlers. Natural crate/module boundary map.
- **Multi-session coordination**: workflow/workspace/claims stores under `sessions/` (controlRootFor); heartbeat leases; cross-session cell claims + reservations.
- **Dispatch guard**: model-tier validation at spawn (bee-model-guard) — hive rule 12 enforcement lives in a hook.
