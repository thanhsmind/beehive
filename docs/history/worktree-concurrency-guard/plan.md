---
artifact_contract: bee-plan/v1
mode: high-risk
approved_gate2: 2026-07-24T08:10:00Z
---

# Plan: Worktree Concurrency Guard

Mode: `high-risk` — hard-gate flag: data loss (prevents a concurrent session silently overwriting/racing another session's work in a shared companion checkout; two past incidents already lost commits this way). Also touches multi-domain (a PreToolUse hook + a CLI command handler + a shared lib).
Why this is the least workflow that protects the work: a hard-gate flag alone forces high-risk regardless of file count — self-discipline already failed here once, so this closes with a persona-panel feasibility proof before any code lands, not a lighter lane's inline reality check.

## Requirements (from CONTEXT.md)

- D1: Enforce at BOTH surfaces — `bee worktree new` (creation-time refusal) and `bee-write-guard.mjs` (write-time block).
- D2: Detection signal is structural and generic (never a hardcoded path name); targets a genuinely shared/aliased checkout, not ordinary submodules.
- D3: Hard fail-closed refusal, no override flag.
- D4: An already-blocked session is redirected to `bee worktree new --with-companion` (create-anew, never an in-place upgrade of the current worktree).
- D5: Hook-level guard, never silenced by `gate_bypass`.
- D6: Zero behavior change for hosts with no companion-eligible nested repo.

## Approach

See `docs/history/worktree-concurrency-guard/approach.md` (high-risk lane — fan-out per decision 0009). Its risk map flags one HIGH-risk open question — the detection signal's exact trigger condition (symlink-escape companion mounts vs. plain nested-`.git`-in-a-shared-checkout, STR65's actual incident shape) — that bee-validating must prove with concrete fixtures before Epic 1's cell is capped.

## Shape — epic map

**Feature outcome:** a live concurrent session can no longer silently write into, or create a new worktree that unguarded-ly touches, a shared/companion-eligible nested checkout — every such write is either already covered by a verified `--with-companion` mount, or refused with a fix pointing at that paved road.

**Repo-reality basis:** confirmed by direct code read this session — `isConcurrentMode()` (`claims.mjs:283`) has zero callers in `worktree-store.mjs` or `bee.mjs`; `bee-write-guard.mjs` already recognizes a *verified* companion mount (`resolveCompanionMountedRelPath`, `:384-414`) and known sibling/main-checkout targets (`describeCrossWorktreeTarget`, `:263-297`) but never consults concurrency; `guards.checkWrite()` (`guards.mjs:586`) has no check unconditional on "is this a shared nested checkout" today.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | Shared detection primitive | Both surfaces need the same answer to "is this target a shared/companion-eligible nested checkout reachable by another live session" — building it once, proven against real fixtures, avoids two divergent implementations. | 1 slice | Fixture tests for: symlinked companion mount (verified + unverified), plain nested `.git` inside a shared main checkout, plain git submodule (negative control) — resolves the approach.md HIGH-risk question. |
| E2 | Write-guard wiring | `bee-write-guard.mjs` blocks a live write per D1(b)/D3/D4. | 1 slice, depends on E1 | Test: a write into a detected shared target while `isConcurrentMode()` is true is refused with a typed, actionable message; an already-verified companion mount is unaffected; a no-concurrency session is unaffected (D6). |
| E3 | `worktree new` wiring | `bee.mjs handleWorktreeNew` refuses at creation per D1(a)/D3/D4. | 1 slice, depends on E1, parallelizable with E2 (different file) | Test: `worktree new` without `--with-companion` while concurrent and the source checkout has a shared target is refused typed; `--with-companion` present, or no shared target, or no concurrency — proceeds unchanged (D6). |
| E4 | Regression + backward-compat proof | Extends existing suites so the concurrency dimension is covered going forward, not just proven once by hand. | 1 slice, depends on E2 + E3 | `scripts/test_worktree_companion.mjs` and `hooks/test_write_guard.mjs` (or a new file, per approach.md's open convention question) pass green, including the D6 negative control. |

**Slice queue:** E1 → {E2, E3} → E4. Feasibility status: E1 not yet proven (the HIGH-risk detection-signal question is open); E2-E4 are ordinary implementation once E1's interface and trigger condition are settled.

**Current slice to prepare:** E1 only — the detection primitive and its proof. E2-E4 are next-slice work, not created as cells yet (their exact call sites depend on E1's resolved interface).

## Test matrix

High-risk lane — probes written out per edge dimension that applies:

- **Concurrency/ordering:** two live sessions, one holding a verified companion mount, one not — does the unverified one get blocked while the verified one proceeds? (Core of E2/E3.)
- **Boundary/structural:** symlink-escape vs. plain nested `.git` vs. ordinary submodule — the exact E1 fixture set.
- **Backward compatibility:** a host checkout with no nested/companion repo anywhere — zero new refusals (D6 negative control).
- **Failure mode of the check itself:** a corrupt/missing companion marker, or a broken symlink — must fail closed on the concurrency+shared-target question the same way existing checks fail closed on containment (`holdsStoreCorrupt` precedent, `guards.mjs:672-682`), never silently allow because detection itself errored.
- **Authorization/escape route:** the refusal message actually names `bee worktree new --with-companion` as the fix (D3/D4), matching the escape-route-teaching precedent (`docs/history/learnings/20260717-guard-membership-escape-routes.md`).
- **State/idempotency:** re-running `worktree new --with-companion` after a first refusal succeeds cleanly (no leftover partial state from the refused attempt).

## Out of scope

- The host-project-specific (fgOS/forgent) same-checkout multi-session lock already shipped for STR65 (`main-checkout-lock.mjs`, `session-identity.mjs`, pre-commit hook) — not bee-core's concern.
- STR85 (companion marker disappearing mid-session, root cause unknown) — explicitly separate per the brief.
- The `worktree new`/`register` stale-cell-copy bug discovered this session (git-tracked `.bee/cells/` checked out wholesale) — filed as its own PBI (`p-9c48a67c`, feature `worktree-scaffolding-cell-leak`), independent scaffolding bug, not a concurrency-guard gap.
