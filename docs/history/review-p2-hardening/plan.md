---
artifact_contract: bee-implement-plan/v1
feature: review-p2-hardening
lane: standard
status: Approved
updated: 2026-08-11
sources: [review backlog-batch-20260811 findings B-P2-1..B-P2-8, D-P2-1, D-P3-1..3]
decisions: [see decisions log — review-p2-hardening scoping synthesis]
---

# Implementation Plan: Review P2 Hardening

**Goal** — Close the batch review's P2 set: hardening bound to mechanisms
instead of copies, loud validation at the surfaces that were silently
permissive, and honest reporting where refusals were invisible.

**In scope — 4 cells**
- rph-1 (git mechanism): one shared hardened unsigned-commit helper in
  `verbs/worktree/git.rs` used by close's bookkeeping commit and merge's
  commit; close's config refusal names the FILE actually carrying the bad
  value (merged local overlay) and treats `null` as unset; the
  defense-in-depth fallback arm regains its direct test. (B-P2-1, B-P2-4)
- rph-2 (registry cluster): `worktree register` validates `--feature` with
  `feature_slug_ok`; `worktree new` surfaces the cellsSync skip report in
  text and JSON; the ls-files tracked-set guard fails CLOSED on unexpected
  output shape; register-on-adopted-worktree prune is named in the report;
  the fail-safe test gets a ceiling guard; the dest archive dir joins the
  symlink-checked set; the close fixture seeds with --no-gpg-sign.
  (B-P2-2, B-P2-7, D-P2-1, D-P3-1, D-P3-2)
- rph-3 (locate boundary): `Engine::locate` stops at the first repository
  boundary (a directory containing `.git`) instead of walking to `/`;
  `--repo-root` becomes the first locate candidate; tests neutralize
  ambient `BEE_JS_ENTRY` and pin production `locate()`. (B-P2-3)
- rph-4 (worker/cells doors): `push_worker_record` upserts by
  (nickname, cell) so auto-registration cannot accumulate duplicates;
  payload keys `worker_registered`/`registration_error` gain direct
  assertions through `run_dispatch_prepare`; the swarming reference stops
  teaching manual `state worker add`; `cells update` applying
  `change_class: behavior` arms `trace.behavior_change` under the same
  explicit-false-wins rule as add. (B-P2-6, B-P2-8)

**Out of scope** — B-P2-5 (Windows pins for index-restore/reason): deferred
with reason — needs a Windows-portable failure driver best validated
against real Windows CI; stays as its filed review-finding row.

**Validation** — `commands.test` (cargo test --release) green at every cell
finish, at merge, at close.

**Risk** — wording drift around git failure texts is deliberately NOT
unified (R81 pins close's spelling); rph-1 keeps each caller's user-facing
text and only unifies the spawn/hardening mechanics.

**Rollback** — revert per-cell commits; all additive hardening.
