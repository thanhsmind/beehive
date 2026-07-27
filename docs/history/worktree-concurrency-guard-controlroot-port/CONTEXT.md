# Worktree Concurrency Guard — Port to bee 1.18.2 — Context

**Feature slug:** worktree-concurrency-guard-controlroot-port
**Date:** 2026-07-26
**Exploring session:** complete
**Scope:** Deep
**Domain types:** CALL, RUN

## Feature Boundary

Re-apply the already-shipped `worktree-concurrency-guard` feature (cells `wcg-1`/`wcg-2`/`wcg-3`/`wcg-fix-1`/`wcg-fix-2`, PR #64) at bee 1.18.2's new canonical file locations and its new `controlRoot`/workspace-ownership architecture, so PR #64 merges cleanly against `origin/main` (currently `mergeable: CONFLICTING`). This is a **relocation + adaptation**, not a redesign — the locked product decisions D1-D6 from `docs/history/worktree-concurrency-guard/CONTEXT.md` are unchanged; only the file layout and the concurrency-root plumbing change. Ends at: `wt/worktree-concurrency-guard` merges onto `origin/main` with zero conflicts, all tests (ours + upstream's) green, PR #64 shows `mergeable: MERGEABLE`.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| Port-D1 | Sync onto upstream via a **merge** of `origin/main` into `wt/worktree-concurrency-guard`, not a rebase. | Preserves existing commit hashes and PR #64's review thread; avoids force-push. |
| Port-D2 | Mechanical/generated-file conflicts (manifest hashes, onboarding ledger, knowledge indexes, `docs/backlog.md`, `taxonomy.json`, JSONL logs) are resolved by accepting upstream then re-running the **owning regen command** — never hand-merged. | Matches the original feature's own `regen_obligation_ack` discipline. |
| Port-D3 | This port is a **minimal, behavior-preserving re-application** of the already-locked D1-D6 at the new canonical location (`packages/bee/lib/guards.mjs`, `packages/bee/hooks/bee-write-guard.mjs`, `packages/bee/bee.mjs`, `packages/bee/hooks/test_write_guard.mjs`, `scripts/tests/test_worktree_companion.mjs`) — no redesign, no deeper integration with the new workspace-ownership system beyond composing alongside it. | YAGNI/KISS — the goal is a mergeable PR, not a redesign. |
| Port-D4 | `isSharedNestedCheckoutTarget`/`hasAnySharedNestedCheckout`'s internal concurrency check must call `isConcurrentMode(controlRoot, opts)`, not `isConcurrentMode(root, opts)` — **scoped to that one call only**. The filesystem scan itself (`realpathOrNull`, `findNestedCheckoutDir`, `scanForNestedCheckout`) stays on the physical `root`; `controlRoot` can differ physically from `root` (confirmed: `resolveWriteTopology` resolves `controlRoot = override \|\| ctx.controlRoot \|\| root`, `guards.mjs:284-290`), so swapping `root` for `controlRoot` throughout would scan the wrong tree. | The new architecture scopes session/concurrency state to `controlRoot`, since multiple linked worktrees can share one; checking bare `root` would miss a concurrent session on a sibling worktree. Fresh-eyes review (2026-07-26) flagged the scan-vs-concurrency distinction as worth stating explicitly, since the two functions currently take a single `root` param serving both purposes. |
| Port-D5 | The write-guard wiring reuses `ctx.controlRoot` (already resolved by `readHookContext`) exactly as the existing `checkWrite` call does — no new topology resolution. | `ctx.controlRoot` is already available at the exact plug-in point. |
| Port-D6 | The worktree-new wiring derives its own `controlRoot` via the existing `controlRootFor(mainRoot)` helper (already used by sibling functions like `handleWorktreeMerge`) and resolves its own acting session id via `resolveSessionId({ root: controlRoot })`, mirroring the shipped `wcg-fix-1` self-exclusion fix. | `handleWorktreeNew` has no `controlRoot` today; `controlRootFor` is the established pattern to get one. |
| Port-D7 | The refusal drops the old `[WORKTREE_CONCURRENT_SHARED_NESTED]` code-prefix convention, throwing a plain, descriptive `Error` matching every other refusal already in the new `handleWorktreeNew`. | `WorktreeCreateError` no longer exists in the new `bee.mjs` (zero grep matches); every sibling refusal in this function is a plain `throw new Error(...)` with no code prefix. |
| Port-D8 | Test additions are re-applied at the new test file paths, preserving the exact same proven scenarios, adapted only for each file's existing harness idioms. | Both new test files confirmed to still use the same harness style (`check()` / `record()`) as before — mechanical re-application, not a rewrite. |

### Agent's Discretion

All 8 decisions were locked by the agent from direct code evidence (materialized `origin/main` files, read via `git show`) rather than asked as questions — `gate_bypass_level` was `total` for this session, and every candidate gray area resolved to a confident, evidence-grounded answer. Port-D7 (confidence 75, the lowest of the 8) is the one genuine judgment call among them — matching the new codebase's own convention over preserving the old one — flagged here in case a fresh-eyes reviewer or the user wants to revisit it.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| `controlRoot` | The new architecture's coordination-state root — may differ from the local physical checkout `root` when multiple linked worktrees share one control point. Resolved via `resolveWriteTopology(root, controlRootOverride)` in `guards.mjs`, or `controlRootFor(root)` in `bee.mjs`. |
| Workspace ownership | The new `checkWorkspaceOwnership` deny class in `guards.mjs` — locks write ownership per physical workspace when a second live session holds it. Confirmed complementary to, not overlapping with, this feature's nested-checkout detection (zero shared logic, verified by grep). |

## Existing Code Context

From the quick scout (dispatched as a `bee-gather` worker against `git show origin/main:<path>` content materialized to scratch). Downstream agents read these before planning.

### Reusable Assets

- `packages/bee/lib/guards.mjs:825` — `checkWrite(root, state, relPath, agentName, { sessionId, controlRoot })`; topology resolved at `:871` via `resolveWriteTopology(root, controlRootOverride)` → `{ ctx, controlRoot }`. Our new check's natural plug-in point.
- `packages/bee/lib/guards.mjs:732-749` — `checkWorkspaceOwnership(controlRoot, ctx, sessionId)`: the pattern to follow for how a check consumes `controlRoot`.
- `packages/bee/hooks/bee-write-guard.mjs:650-651` — `readHookContext(HOOK_NAME)` resolves both `ctx.root` and `ctx.controlRoot` once, at the top of dispatch.
- `packages/bee/hooks/bee-write-guard.mjs:818-843` — the existing dispatch loop calling `guards.checkWrite(storeRoot, state, rel, agentName, { sessionId, controlRoot: ctx.controlRoot })`. `canonicalRelPath` (`:65-114`), `describeCrossWorktreeTarget` (`:263-297`), `resolveCompanionMountedRelPath` (`:384-414`) all confirmed unchanged.
- `packages/bee/bee.mjs:4906-4981` — `handleWorktreeNew`'s current full body. `mainRoot` set at `:4923`; `createFeatureWorktree` mutation call at `:4945`. Plug-in point: between the two. No `controlRoot` variable exists in this function today.
- `packages/bee/bee.mjs:501, :1665, :2337, :2406, :871` — `controlRootFor(root)` usage sites in sibling functions (the pattern to copy).
- `packages/bee/bee.mjs:120` — `resolveSessionId` already imported; used elsewhere (e.g. `:5058` in `handleWorktreeMerge`) but not yet in `handleWorktreeNew`.
- `packages/bee/lib/claims.mjs:333` — `isConcurrentMode(root, { excludeSessionId, now, staleSeconds })` — confirmed byte-identical signature to the pre-1.18.2 version. No adaptation needed to this function itself.

### Established Patterns

- Generated/bookkeeping files (manifest hashes, onboarding ledger, knowledge indexes) are always resolved via their owning regen command, never hand-merged — established this session on the original feature.
- Fail-closed-on-detection-error, self-exclusion in concurrency checks, and "redirect to `--with-companion`, never in-place conversion" all carry over unchanged from the original feature's D3/D4/D5/D6 — this port does not revisit them, only relocates and re-plumbs them.

### Integration Points

- `packages/bee/lib/guards.mjs` — new exports (relocated + `controlRoot`-adapted) alongside `checkWrite`.
- `packages/bee/hooks/bee-write-guard.mjs` — new pre-`checkWrite` check (relocated).
- `packages/bee/bee.mjs` `handleWorktreeNew` — new pre-creation refusal (relocated + `controlRootFor`-adapted).
- `packages/bee/hooks/test_write_guard.mjs`, `scripts/tests/test_worktree_companion.mjs` — relocated test rows.

## Canonical References

- `docs/history/worktree-concurrency-guard/CONTEXT.md` — the original feature's locked D1-D6, unchanged by this port.
- `docs/history/worktree-concurrency-guard/reports/review-20260724.md` and `walkthrough.md` — what shipped and what the independent review found/fixed; the port must not regress any of it.
- PR #64: `https://github.com/thanhsmind/beehive/pull/64` (repo redirected from `beegog`) — currently `mergeable: CONFLICTING` against `baseRefOid: 07d97049` (v1.18.2).
- Backup safety tag `pre-1.18.2-port-backup` on `wt/worktree-concurrency-guard` (commit `56b437aa`) — the pre-port state to fall back to if the merge attempt needs to be aborted again.

## Outstanding Questions

### Deferred To Planning

- [ ] Exactly how `controlRoot` reaches `isSharedNestedCheckoutTarget`/`hasAnySharedNestedCheckout`'s new call signature — a new positional parameter, or an added field on the existing `opts` object (which already carries `excludeSessionId`) — is an implementation choice, not locked here. Naming it explicitly so the write-guard and worktree-new cells don't diverge on the choice (fresh-eyes review, 2026-07-26).
- [ ] Exact mechanical resolution order for the ~20 files git reports as conflicting (which to resolve first, whether any depend on another's resolution) — planning should sequence this into cells.
- [ ] Whether `.bee/cells/rel1150-1.json` (a stray unrelated cell record git also reports as conflicting) needs any resolution at all, or is simply dropped/ignored as pre-existing noise from the earlier worktree-scaffolding-cell-leak bug.
- [ ] Confirm `docs/knowledge/areas/workflow-state/worktree-isolation.md` auto-merged cleanly (git's `--no-commit` attempt showed "Auto-merging" with no conflict reported for this file specifically) — verify content once the real merge is re-attempted.

## Deferred Ideas

- Deeper integration between this feature's nested-checkout detection and the new workspace-ownership system (e.g., a unified "why was this write refused" surface covering both deny classes) — explicitly out of scope per Port-D3; would be its own feature if ever wanted.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs Port-D1 through Port-D8 are stable. Planning reads locked decisions, existing code context, canonical references, and the deferred-to-planning questions above.
