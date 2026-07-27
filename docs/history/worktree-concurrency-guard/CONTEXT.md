# Worktree Concurrency Guard — Context

**Feature slug:** worktree-concurrency-guard
**Date:** 2026-07-24
**Exploring session:** complete
**Scope:** Standard
**Domain types:** CALL, RUN

## Feature Boundary

Close the gap where `bee worktree new` and the `bee-write-guard.mjs` PreToolUse hook let a session write into a shared/companion-mounted nested checkout while another session is concurrently live (`isConcurrentMode()` true) without ever having declared `--with-companion` — today this is 100% self-discipline, and self-discipline already failed once (two commits silently ate another session's work). This feature ends at: `bee worktree new` refuses without `--with-companion` when concurrent and a companion-eligible target exists; the write-guard hook blocks an unverified write into a genuinely shared nested checkout under the same condition. It does not touch the host-project-specific (fgOS/forgent) same-checkout lock already shipped for STR65, and does not touch the separate companion-marker-disappearance bug tracked as STR85.

**External tracking note:** the reporting project labeled this "STR84" in its bug report/prompt to us — that is their tracking ID, not this repo's feature or branch name (user decision, this session: pick a beegog-native slug instead of reusing an external project's ID).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Enforce at BOTH surfaces: (a) `bee worktree new` refuses at creation time when `isConcurrentMode()` is true, `--with-companion` is absent, and the source checkout has a companion-eligible shared nested repo; (b) `bee-write-guard.mjs` blocks a live Write/Edit/Bash write whose target resolves inside a genuinely shared nested checkout when `isConcurrentMode()` is true and no verified companion marker covers it. | Backlog CoS names both surfaces explicitly ("bee worktree new (or the write-guard hook) blocks/prompts…"). |
| D2 | **(Superseded 2026-07-24, decision `0ccc1cf3`, supersedes `4d0b6f9d`)** The detection signal is structural and generic — never a hardcoded path name like `repo/` — and covers TWO shapes: (a) git data resolving via symlink/indirection to a location OUTSIDE the checkout's own directory tree (the original companion-mount shape); AND (b) a plain, non-symlinked nested git repo physically INSIDE the checkout's own tree, when `isConcurrentMode()` is true for that checkout — the actual STR65 incident shape, confirmed by a validating-stage spike to be completely unguarded today. Ordinary git submodules are still excluded, but the exclusion signal is registration-based (a real `.gitmodules` entry / `git submodule status`), NOT bare has-own-`.git` structural detection — the same spike proved a plain nested repo and a registered submodule are structurally identical and both currently unguarded, so has-own-`.git` alone cannot tell them apart. | Originally locked narrower (symlink-escape only); widened after a validating-stage spike (`.bee/spikes/worktree-concurrency-guard/probe-nested-checkout-baseline.mjs`, see `reports/validation-e1.md`) proved the narrower reading leaves STR65's actual incident shape unguarded. The configured advisor's independent Gate-3 review flagged this exact conflict; the user chose to widen (accepting the false-positive cost: an unrelated nested repo in a shared checkout gets flagged whenever a second session is merely live, hard fail-closed per D3, no override) over narrowing the write-guard's coverage or scoping the widening to the main checkout only. |
| D3 | Enforcement is a hard fail-closed refusal with no override flag. "Blocked or prompted" in the CoS is one mechanism described from two angles (the guard refuses; the agent surfaces the refusal + fix action to the human), not two alternative behaviors. | Matches every existing hook-level guard in this codebase (SESSION_REQUIRED, CLAIMED, reservation conflict, intake gate) — all fail closed with an actionable FIX message, no bypass flag. |
| D4 | An existing session (not mid `worktree new`) blocked by the write-guard is always directed to re-enter via `bee worktree new --with-companion` — the existing, already-shipped paved road. No new mid-session "declare this mount as trusted" verb is introduced. This is a create-a-new-worktree remedy, not an in-place conversion: `bee worktree new` (`bee.mjs:3845`) always creates a fresh worktree, it cannot retrofit companion mounting onto the already-live worktree the session is blocked in. | AGENTS.md rule 14 already names this as the paved road. A new self-declare verb would reopen the exact self-discipline surface this feature exists to close. Planning must word the write-guard's refusal message so it directs to opening a *new* companion worktree, never implies upgrading the current one in place (fresh-eyes review finding). |
| D5 | This guard is hook-level, architecturally separate from the four approval Gates — never silenced by `gate_bypass` (`normal`/`full`/`total`), same as existing hook-level guards (privacy marker, reservation conflict, intake gate). | `gate_bypass`'s documented scope is Gates 1-4 only; it has never covered PreToolUse hook checks. |
| D6 | Hosts with no companion-eligible nested/shared repo anywhere in the checkout see zero behavior change — the new check is a pure no-op when nothing shared/aliased exists to protect. | Explicit brief constraint: "Backward compatible… zero behavior change." |

### Agent's Discretion

All six decisions above were locked by the agent from brief text, code evidence, and existing architectural precedent rather than asked as questions — `gate_bypass_level` was `total` for this session, and the gate-bypass refinement rule (bee-exploring step 4) requires an "approval-type" question (one the agent already has a confident, evidence-grounded answer for) to be locked directly rather than asked, reserving real questions for genuine information gaps only the user can fill. None of the candidate gray areas identified during scouting survived as a genuine information gap — each resolved cleanly to one dominant answer grounded in the brief's own Constraints/Acceptance-Criteria text or this repo's established hook philosophy. The exact mechanical detection logic (how to walk a path to determine if it resolves outside the checkout's own tree, performance/depth limits, which existing helper in `bee-write-guard.mjs`/`guards.mjs` to extend vs. add) is left to planning/validating, per D2's framing (structural signal is locked; the walking algorithm is implementation).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Concurrent session | `isConcurrentMode(root)` returns true: at least one other session's heartbeat is live (not stale) for this checkout's `.bee` store. |
| Companion checkout / companion mount | An existing `.bee/companion-session.json` marker whose declared `worktreePath` realpath matches the live realpath of the mount symlink at `mountPath` — today's recognition path, built by PR #61 / `resolveCompanionMountedRelPath()`. |
| Shared/aliased nested checkout (D2, widened 2026-07-24) | A subdirectory within the checkout that is itself a distinct git repository (whether via a symlink resolving outside the checkout's own tree, OR a plain nested repo physically inside the tree) that another concurrently-live session could also reach — the structural signal this feature adds, independent of whether a companion marker exists yet. Excludes a real, registration-verified git submodule. |

## Existing Code Context

From the quick scout only (dispatched as a bee-gather worker). Downstream agents read these before planning.

### Reusable Assets

- `.bee/bin/lib/claims.mjs:283` — `isConcurrentMode(root, { excludeSessionId, now, staleSeconds })`: returns true iff another session's heartbeat is live and not stale. Zero callers today in `worktree-store.mjs` or `bee.mjs` — worktree creation never consults concurrency yet.
- `.bee/bin/hooks/bee-write-guard.mjs:369-424` — `resolveCompanionMountedRelPath(root, cwd, rawTarget)`: recognizes an already-marker-verified companion mount; returns `null` on any mismatch (fail-open to the caller's fallback).
- `.bee/bin/hooks/bee-write-guard.mjs:263-297` — `describeCrossWorktreeTarget()`: produces a human-readable denial naming the sibling/main checkout when containment fails.
- `.bee/bin/hooks/bee-write-guard.mjs:791,804,818` — the hook's `main()` resolves `rel` via `canonicalRelPath(...) || resolveCompanionMountedRelPath(...)` then calls `guards.checkWrite(storeRoot, state, rel, agentName, { sessionId })` per resolved path — the natural plug-in point for the new check is right where companion-mount resolution succeeds/fails, before or inside `checkWrite`.
- `.bee/bin/lib/guards.mjs:586` — `checkWrite(root, state, relPath, agentName, { sessionId })`: first-hit-wins ordered checks (direct-edit deny, docs/history deny, scratch-shape deny, lane-record resolution, cross-session hold conflict, cross-worktree foreign-hold, phase gating). No existing check is unconditional on reservations being taken — STR65's incident sessions never took a reservation, so the new check must not depend on one either.
- `.bee/bin/bee.mjs` `handleWorktreeNew` (~3845-3899) and `.bee/bin/lib/worktree-store.mjs:682` (`runCompanionStart`) / `:1334` (`teardownCompanionIfPresent`) — today's `--with-companion` parsing and lifecycle. Confirmed zero `isConcurrentMode` references anywhere in this file.

### Established Patterns

- Fail-closed hook refusal with a typed FIX message (SESSION_REQUIRED, CLAIMED, reservation conflict, intake gate) — the new check should emit the same shape of typed, actionable refusal.
- `bee worktree new --with-companion` as the paved road (AGENTS.md rule 14) for any session about to touch a shared companion checkout.

### Integration Points

- `.bee/bin/hooks/bee-write-guard.mjs` — new concurrency+shared-checkout branch alongside the existing companion-mount resolution.
- `.bee/bin/bee.mjs` `handleWorktreeNew` — new pre-creation refusal check.
- `.bee/bin/lib/guards.mjs` `checkWrite()` — likely plug-in point if the check is expressed as a new ordered check rather than a hook-level pre-check.

## Canonical References

- `.bee/bin/lib/claims.mjs:283` — `isConcurrentMode()` definition.
- `.bee/bin/hooks/bee-write-guard.mjs:263-297,369-424,649-830` — companion-mount recognition and hook dispatch order.
- `.bee/bin/lib/guards.mjs:586-731` — `checkWrite()` ordered checks.
- `scripts/test_worktree_companion.mjs` — existing companion lifecycle E2E test (start/mount/teardown), no concurrency dimension yet.
- `hooks/test_write_guard.mjs:1104-1161` — existing companion-mount-recognition tests (rows 65-69), no concurrency dimension yet.
- AGENTS.md rule 14 — "New feature work in an occupied checkout uses `bee worktree new`/`bee worktree merge`."
- Background reference only (do not port verbatim, different repo): forgent workshop's `docs/history/str65-worktree-isolation-enforcement/CONTEXT.md`.

## Outstanding Questions

### Deferred To Planning

- [ ] Exact mechanical detection for "resolves outside this checkout's own directory tree" (symlink resolution depth, performance bound on the walk, whether to reuse/extend `resolveCompanionMountedRelPath` or add a sibling helper) — planning/validating should prove this against the real `bee-write-guard.mjs`/`guards.mjs` code.
- [ ] Which existing tests (`scripts/test_worktree_companion.mjs`, `hooks/test_write_guard.mjs`) get extended vs. which need a new file for the concurrency dimension.
- [ ] Exact refusal message text/shape for both surfaces (worktree-new-time vs. write-guard-time), consistent with existing typed-refusal conventions.

## Deferred Ideas

- A `bee worktree new`/`register` bug where a freshly created worktree's `.bee/cells/` contains ALL cell files from the source checkout, including other features' stale, uncapped, claimed cells (not just its own feature's cells) — hit twice in this session (`rel1150-1` from `release-1-15-0` blocked `state start-feature` in two separately created worktrees before this feature could even start). Root cause per fresh-eyes review spot-check: `.bee/cells/` is git-tracked, so `git worktree add` checks all of it out — `bootstrapWorktreeStore` (`worktree-store.mjs:334-368`) only copy-if-absents `onboarding.json`/`config.json`/`state.json`, it never touches `cells/`. The fix direction is git-tracking/checkout scoping, not a copy-call change. Filed as its own PBI (`p-9c48a67c`, feature `worktree-scaffolding-cell-leak`) rather than folded into this feature's scope, since it's an independent scaffolding bug, not a concurrency-guard gap.
- STR85 (companion mount marker disappearing mid-session) — explicitly out of scope per the brief's "Do not conflate with" section; tracked separately, root cause unknown, not this feature's problem.
- The host-project-specific (fgOS/forgent) same-checkout multi-session lock (STR65's product-side fix: `main-checkout-lock.mjs`, `session-identity.mjs`, pre-commit hook) is out of scope for bee-core; not re-implemented here.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs D1-D6 are stable. Planning reads locked decisions, existing code context, canonical references, and the two deferred-to-planning technical questions above. Validating and reviewing use locked decisions for coverage and UAT.
