# Issue #56 claims re-verified against v1.16.1 (2026-07-24)

| # | Claim (v1.11.2) | v1.16.1 status | Evidence |
|---|-----------------|----------------|----------|
| 3.1 | Global `state.json` pipeline; `startFeature` demands repo-wide idle under `state` lock | STILL TRUE | `state.mjs:1768-1866` (preconditions), `:1782` (lock) |
| 3.2 | Lanes are a compat layer; `startLane()` contends on the same `state` lock | STILL TRUE | `state.mjs:1782-1791`; `lock.mjs:88-97` (lock file from name only) |
| 3.3 | `reservations.json` one global file+array, one lock, whole-file rewrites | STILL TRUE | `reservations.mjs:16,162,190,207` |
| 3.4 | `pathsOverlap` prefix/glob pessimism; no intent vs lease split | STILL TRUE mechanism; issue's example wrong (`src/api/users/*` vs `src/api/orders/x.ts` does NOT conflict — bases don't prefix) | `reservations.mjs:55-71`; reused by `worktree-holds.mjs:52-56,205` |
| 3.5 | Cross-worktree holds hard-block all paths, no advisory mode | STILL TRUE | `guards.mjs:659-701` ("a cross-worktree hold is a hard block") |
| 3.6 | `bee-state-sync` writes global aggregate into default `state.json` under `state` lock | STILL TRUE | `bee-state-sync.mjs:80-109` |
| 3.7 | Single global `HANDOFF.json` | STILL TRUE (lane-name filtering added: `state.mjs:1934-1939`; path fixed at `state.mjs:963`) | |
| 3.8 | Heartbeat vs `bindSessionLane` lost-update race | PARTIALLY FIXED — heartbeat now locked (`claims.mjs:241-260`, sessions lock `:401,432-438`); `bindSessionLane`/`unbindSessionLane` still lock-free (`claims.mjs:297-319`) | |
| 3.9 | ~5s lock retry; long op holds lock across multi-minute verify | PARTIALLY FIXED — retry unchanged (`lock.mjs:32-34`); pid-aware stale takeover closed false-steal (`lock.mjs:10-22,193-208`); `mergeFeatureWorktree` still holds `worktree-admin` across `spawnSync(verify)` (`worktree-store.mjs:1300,1525`) | |

Mitigations since v1.11.2 that reduce symptoms without changing primitives: lane overlay (B12/R56), lane-write session auto-resolve (i54-closeout D7), pid-liveness lock takeover (rel180-4). None change the single-`state`-lock, global-store architecture the issue targets.

Issue #56's 15 invariants (mục 15) adopted verbatim as the feature's acceptance suite — see CONTEXT.md D9.
