---
artifact_contract: bee-implement-plan/v1
feature: worktree-concurrency-guard
lane: high-risk
status: Shipped
updated: 2026-07-26
sources: [CONTEXT.md, approach.md, plan.md, cells/wcg-1.json, cells/wcg-2.json, cells/wcg-3.json, cells/wcg-fix-1.json, cells/wcg-fix-2.json, reports/review-20260724.md, walkthrough.md]
decisions: [D1, D2, D3, D4, D5, D6, 0ccc1cf3]
---

# Implementation Plan: Worktree Concurrency Guard

> Human-layer projection of the truth artifacts. Truth lives in CONTEXT.md
> (decisions), plan.md + cells (work), and the validating report (evidence).
> Feedback on this document flows back to those artifacts, then this re-renders.

## 1. Goal

Stop a live concurrent bee session from silently writing into, or creating a new worktree that unguarded-ly touches, a shared/companion-eligible nested checkout — the exact class of race that already ate two commits' worth of another session's work. Today `bee worktree new` and the write-guard hook never consult whether anyone else is concurrently active; this closes that gap on the bee-core side.

**Success looks like**
- `bee worktree new` without `--with-companion` refuses when another session is concurrently live and the source checkout has a companion-eligible shared nested repo (D1a).
- The write-guard hook blocks an unverified write into a genuinely shared nested checkout under the same condition (D1b).
- A host with no companion-eligible nested repo anywhere sees zero behavior change (D6).

## 2. Current State

`isConcurrentMode()` (`claims.mjs:283`) already exists and is proven — it has exactly two callers today (`claims.mjs:521`, `reservations.mjs:154`), neither in `worktree-store.mjs` nor `bee.mjs`. The write-guard hook (`bee-write-guard.mjs`) already recognizes a *verified* companion mount (`resolveCompanionMountedRelPath`, lines 369-424) and known sibling/main-checkout targets (`describeCrossWorktreeTarget`, lines 263-297), and resolves every Write/Edit/Bash target through `guards.checkWrite()` (`guards.mjs:586`) — but none of these existing checks are conditioned on concurrency. `bee worktree new`'s `--with-companion` flag (`bee.mjs:3845`) is a pure opt-in with no enforcement.

## 3. Scope

**In scope**
- A shared detection primitive answering "is this target a shared/companion-eligible nested checkout reachable by another live session" (D2).
- Wiring that primitive + `isConcurrentMode()` into the write-guard hook (D1b) and `bee worktree new` (D1a).
- A hard, fail-closed refusal with no override (D3), always redirecting to `bee worktree new --with-companion` (D4).

**Out of scope**
- The host-project-specific (fgOS/forgent) same-checkout multi-session lock already shipped for STR65.
- STR85 (companion marker disappearing mid-session) — separate, root cause unknown.
- The `worktree new`/`register` stale-cell-copy bug found this session — filed as its own PBI (`p-9c48a67c`).

## 4. Proposed Approach

Build one shared structural-detection primitive first, proven against real fixtures, then wire it into both surfaces (approach.md). Epic 1 (this slice) builds and proves the primitive in isolation; Epics 2-4 (next slices) wire it into the write-guard hook, `worktree new`, and extend the regression suites.

**Why this approach** — both surfaces need the identical answer to the same question; building it once against proven fixtures avoids two divergent, unproven implementations of the same safety check.
**Alternatives considered** — a new mid-session "declare this mount as trusted" verb (rejected, D4: reopens the self-discipline gap this feature closes); an override/`--force` flag (rejected, D3: no existing hook-level guard in this codebase offers one); gating the check on `gate_bypass` (rejected, D5: hook-level guards are outside the four Gates' bypass scope).

## 5. Technical Design

```text
Write/Edit/Bash tool call -> bee-write-guard.mjs -> resolveCompanionMountedRelPath / new shared-checkout detection
                                                   -> isConcurrentMode(root) -> guards.checkWrite() -> allow/refuse

bee worktree new --feature <slug> [--with-companion] -> handleWorktreeNew
                                                       -> new shared-checkout detection (against source checkout)
                                                       -> isConcurrentMode(root) -> allow/refuse
```

This slice (Epic 1) only builds and proves the left-hand detection primitive in isolation — no wiring into either dispatch path yet. **Security/Permissions:** the primitive must fail closed (refuse, never silently allow) on any error resolving symlinks or reading fixtures, matching the existing `holdsStoreCorrupt` precedent (`guards.mjs:672-682`) of failing closed on a broken store rather than treating it as empty.

## 6. Affected Files

| Action | File / Component | Purpose |
|--------|------------------|---------|
| Modify | `.bee/bin/lib/guards.mjs` | New exported detection helper (Epic 1, cell `wcg-1`) |
| Modify | `hooks/test_write_guard.mjs` | Fixture tests proving the helper's 3 boundary cases (Epic 1, cell `wcg-1`) |
| Modify | `docs/history/codex-harness-hardening/release-manifest.json` | Regen obligation — `.bee/bin/lib` is a manifest-hashed root (cell `wcg-1`) |
| *(next slice)* | `.bee/bin/hooks/bee-write-guard.mjs` | Wire the helper + `isConcurrentMode()` into the write dispatch (Epic 2, not yet celled) |
| *(next slice)* | `.bee/bin/bee.mjs` (`handleWorktreeNew`) | Pre-creation refusal reusing the helper (Epic 3, not yet celled) |
| *(next slice)* | `scripts/test_worktree_companion.mjs` | End-to-end regression + D6 negative control (Epic 4, not yet celled) |

## 7. Implementation Steps

- [ ] Build and prove the shared detection primitive against 3 fixtures (verified companion mount, plain nested-`.git`-in-a-shared-checkout, submodule negative control) (`wcg-1`)
- [ ] *(next slice, not yet celled)* Wire the primitive into the write-guard hook (Epic 2)
- [ ] *(next slice, not yet celled)* Wire the primitive into `bee worktree new` (Epic 3)
- [ ] *(next slice, not yet celled)* Extend regression suites with the concurrency dimension + D6 negative control (Epic 4)

## 8. Validation Plan

**Automated** — `node --test hooks/test_write_guard.mjs && node scripts/release_manifest.mjs --check && node scripts/ledger_parity.mjs --check` (cell `wcg-1`) → expected: all 3 fixture cases (companion mount, plain nested shared checkout, submodule negative control) pass, manifest/ledger regen clean.
**Manual** — none for this slice (no user-facing surface changes yet — Epic 1 is a pure library addition, `behavior_change: false`).
**Evidence** — pending; bee-validating's feasibility proof runs next and will produce `docs/history/worktree-concurrency-guard/reports/`.

## 9. Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Detection signal's exact trigger condition is genuinely open (symlink-escape companion mounts vs. plain nested-`.git`-in-a-shared-checkout, STR65's actual incident shape) | High | Cell `wcg-1` requires fixture tests proving each of the 3 boundary cases BEFORE the helper is written — no algorithm guessed from prose. bee-validating proves this concretely before Epic 2/3 are even celled. |
| Where to add the shared helper | Low | `guards.mjs` chosen as the existing shared home for write-checking logic; confirmed during Epic 1, not a design question. |
| Refusal message could become a dead-end lockout | Low | Precedent already settled (`docs/history/learnings/20260717-guard-membership-escape-routes.md`): every refusal must teach the `--with-companion` fix, per D3/D4. |
| Backward compatibility for hosts with no companion repo | Low | Additive-only check (D6); Epic 4's negative control proves zero new refusals when nothing shared/aliased exists. |

## 10. Rollback Plan

This slice (`wcg-1`) is purely additive — a new, unwired library helper plus its own tests, `behavior_change: false`, nothing else calls it yet. Rollback is a plain revert of the cell's commit; nothing downstream depends on it. For the full feature once Epics 2-4 land: each wiring point (write-guard hook, `handleWorktreeNew`) is a small, additive conditional branch around existing logic — rollback is reverting those specific commits, with no data migration, no schema change, and no stored state to unwind (the check is computed fresh on every call, nothing is persisted). No feature flag is planned; per D3/D5 this is a permanent hook-level guard, not something meant to be toggled off in production once shipped — the "off switch" is reverting the commit.

## 11. Open Questions

- Does today's containment check already block an *unrecognized* symlink escape regardless of concurrency, narrowing the write-guard's real gap to verified companion mounts only? (approach.md; resolved by `wcg-1`'s fixture tests.)
- For the plain-nested-`.git`-in-a-shared-checkout shape: does the check need a companion marker/config signal, or does `isConcurrentMode()` + root being the ordinary main checkout suffice? (approach.md; resolved by `wcg-1`.)
- Which existing test file should host the Epic 4 concurrency-dimension tests — `hooks/test_write_guard.mjs`'s existing companion rows vs. a new file? (Repo-convention question, not a product one — deferred to Epic 4.)
