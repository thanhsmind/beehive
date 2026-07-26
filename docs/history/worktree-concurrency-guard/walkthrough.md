---
artifact_contract: bee-walkthrough/v1
feature: worktree-concurrency-guard
lane: high-risk
status: Shipped
updated: 2026-07-26
---

# Walkthrough: Worktree Concurrency Guard

## What Shipped

A live bee session can no longer silently write into, or create a new worktree that unguarded-ly touches, a shared/companion-eligible nested checkout while another session is concurrently active.

- `bee-write-guard.mjs` refuses an `Edit`/`Write`/`Bash` write whose target resolves inside a genuinely shared nested checkout (a verified companion mount, or a plain nested repo physically inside the tree) when another session is live and no verified `--with-companion` mount covers it (`wcg-2`).
- `bee worktree new` refuses at creation time under the identical condition, before any mutation (`wcg-3`), built on a new directory-scan primitive (`hasAnySharedNestedCheckout`) that complements the write-guard's point-check (`isSharedNestedCheckoutTarget`, `wcg-1`).
- A real git submodule is never flagged as shared, distinguished from an accidental shared nested repo via `.gitmodules` registration, not bare structural shape.
- Both surfaces are hard fail-closed, no override flag, never consult `gate_bypass`, and always redirect to opening a **fresh** `bee worktree new --with-companion` — never an in-place conversion of the current worktree.
- A checkout with nothing shared/companion-eligible present, or with no other session live, sees zero behavior change.

Two P1 defects found by an independent review session (below) are also fixed and shipped in this same close:
- `bee worktree new`'s concurrency check now excludes the acting session's own heartbeat, so a solo agent is never falsely refused (`wcg-fix-1`).
- The write-guard's shared-checkout detection now fails closed (denies) if the detection call itself throws, instead of silently allowing the write (`wcg-fix-2`).

## How It Was Verified

**Automated, all independently re-run by the orchestrator (not accepted on worker word alone):**
- `node --test hooks/test_write_guard.mjs` — 85 rows, ALL PASS (rows 65-77 companion/baseline recognition; 78-82 write-guard wiring; 83-85 fail-closed-on-error).
- `node --test scripts/test_worktree_companion.mjs` — 17/17 PASS (cases 6-9 worktree-new wiring; 10-11 self-exclusion fix).
- `node scripts/ledger_parity.mjs --check` — clean on every capped cell.
- Wave-close impacted verify (`run_verify.mjs --impacted-from-git`) — 32/32 suites green, run twice (once after `wcg-3`, once after both fixes).
- Every `behavior_change` cell (`wcg-2`, `wcg-3`, `wcg-fix-1`, `wcg-fix-2`) passed a semantic checklist judge dispatched on a different model (opus) than the builder (sonnet) — `model_independence: "confirmed"` on all four.

**Independent review (session `worktree-concurrency-guard-review-20260724`, 5 reviewers — code-quality, architecture, security, test-coverage, api-contract):**
- Found 2 P1s (both fixed and delta-re-reviewed, see below), 2 P2s and 6 P3s filed to backlog as non-blocking.
- Confirmed: D3 (no override), D5 (never consults `gate_bypass`), and existing symlink-containment all hold as designed.

**Human UAT (both items confirmed pass):**
- Live CLI demonstration against a real, disposable throwaway git repo (not this project) — proved the refusal fires with zero mutation when genuinely concurrent + shared, proceeds normally when solo, and proceeds normally even when the *acting* session's own heartbeat is the only live record (the exact P1 the review caught).

## How To Test It Yourself

1. In any git checkout with a nested `.git` directory somewhere inside it, run `bee worktree new --feature <slug>` while another bee session has a live heartbeat in that checkout (or simulate one via `claims.createSession`) — expect a typed `[WORKTREE_CONCURRENT_SHARED_NESTED]` refusal, zero mutation.
2. Re-run with `--with-companion` — it always proceeds regardless of concurrency.
3. With no other live session, or no nested checkout present, `worktree new` behaves exactly as before this feature.
4. The same three shapes apply to a direct `Edit`/`Write`/`Bash` write into the nested checkout, enforced by the write-guard hook instead.

## Deviations From Plan

- Epic 4 (regression-suite extension) was planned as its own slice but turned out already satisfied: `wcg-2` and `wcg-3` each added red-first + D6 negative-control tests as part of normal cap discipline, so no separate cell was needed — confirmed by reading the actual test code, not assumed.
- `wcg-3` needed a genuinely new `guards.mjs` export (`hasAnySharedNestedCheckout`, a directory-scan) beyond what was originally scoped for it — the original point-check (`isSharedNestedCheckoutTarget`) structurally cannot answer "does anything shared exist in this tree," only "is this one target shared." Caught by the batch-2 plan-checker before code was written.
- D2 (the detection signal's exact scope) was widened mid-validating after a spike proved the originally-locked text (symlink-escape only) missed the actual unguarded shape (a plain nested checkout inside the tree, STR65's real incident). Superseded via decision `0ccc1cf3`, with the false-positive cost stated explicitly and accepted by the user.
- A scheduling race between `wcg-2` and `wcg-3` (both silently rewrite the same manifest/ledger via their regen steps, only one declared it) was caught by an adversarial plan-checker before either cell ran, and fixed by declaring the shared files on both so the scheduler auto-serializes them.

## Known Limitations / Follow-Ups

Filed to backlog, non-blocking:
- P2: the write-guard's point-check doesn't skip `node_modules`/build dirs the way the down-scan does — a git-bearing dependency could be falsely flagged.
- P2: companion-marker verification logic is duplicated between `guards.mjs` and `bee-write-guard.mjs`.
- P3 (×6): unverified `.gitmodules` trust (spoofable, within D2's allowance), a stale comment, an unreachable branch with misleading test coverage, an untested idempotency sequence, a weak zero-mutation assertion, and an untested error-code regression pin.

Separately tracked, out of this feature's scope:
- `p-9c48a67c` — a fresh worktree inherits other features' stale claimed cells (`.bee/cells/` is git-tracked). Hit twice this session.
- `p-50f3af4d` — `bee cells schedule` should detect shared regen-obligation side-effects mechanically, not rely on the cell author declaring them (the exact class of bug this session's plan-checker caught by hand).
- `p-3d56a5c8` — this feature's detection primitives need a `controlRoot`-aware adaptation pass before merging onto bee 1.18.2's new workspace-ownership architecture (investigated mid-session when main's bee upgrade was discovered; complementary, not redundant, but not yet ported).

## Merge Status

PR #64 was opened, then closed (unmerged) when the independent review found 2 P1s. Both are now fixed, delta-re-reviewed, and the review session is closed `approved`. A fresh PR (or reopening #64) is the next step, not yet done as of this walkthrough.
