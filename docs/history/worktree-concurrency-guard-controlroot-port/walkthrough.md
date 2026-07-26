---
status: Shipped
---

# Walkthrough — Worktree Concurrency Guard, Port to bee 1.18.2

## What shipped

`wt/worktree-concurrency-guard` now merges cleanly onto `origin/main` (bee 1.18.2, commit `07d97049`). The already-shipped shared-nested-checkout concurrency guard (originally `wcg-1`/`wcg-2`/`wcg-3` + the two P1 fixes `wcg-fix-1`/`wcg-fix-2`) is re-applied at its new canonical location:

- `packages/bee/lib/guards.mjs` — the 10 relocated detection helpers (`isSharedNestedCheckoutTarget`, `hasAnySharedNestedCheckout`, and their shared internals). Both entry points now accept `opts.controlRoot` — the coordination root, which can physically differ from the checkout `root` when several worktrees share one control point — and scope the concurrency-liveness check to it, while every filesystem-scan helper stays scoped to the physical `root`.
- `packages/bee/hooks/bee-write-guard.mjs` — the pre-`checkWrite` refusal, now passing `ctx.controlRoot`, with the same scoped fail-closed try/catch shipped in `wcg-fix-2`.
- `packages/bee/bee.mjs`'s `handleWorktreeNew` — the pre-creation refusal, now deriving its own `controlRoot` via `controlRootFor(mainRoot)` and resolving the acting session against that root (the `wcg-fix-1` self-exclusion pattern), and throwing a plain `Error` (no `[CODE]` prefix — the new codebase's convention, `Port-D7`).
- Both regression suites re-applied at their new paths (`packages/bee/hooks/test_write_guard.mjs`, `scripts/tests/test_worktree_companion.mjs`) with two new rows/cases proving the `controlRoot`-vs-`root` scoping **bidirectionally** (a session seeded under one root only flips the verdict, seeded under the other only it doesn't).

A follow-up fix, found during independent review, closes a real fail-open gap: `isConcurrentMode`/`listSessionRecords` (and, one layer deeper, the per-session-record read) previously swallowed **every** filesystem error — including a genuine hard error (`EACCES`/`EIO`/`EMFILE`) — identically to "nobody else is live," silently allowing the write. An opt-in `strict` mode (default `false`, every other caller in the codebase unaffected) now propagates a real hard error instead of masking it as "solo," and the five structural scan helpers (self-contained to this guard, no other callers) do the same for their own catches.

## How it was verified

- Fresh, independent re-run (not the worker's own): `test_write_guard.mjs` 1/1 pass (87 rows, including both original P1 regression guards and the new bidirectional rows); `test_worktree_companion.mjs` 19/19→20/20 pass (adding Case 14 for the fail-closed fix); `test_claims.mjs` 38→40 pass (adding the two strict-mode unit tests, both using **real** filesystem errors — `ENOTDIR`/`EISDIR` — not synthetic stubs).
- `ledger_parity --check`, `knowledge check`, `git diff --check` (zero leftover conflict markers), and `onboard_bee.mjs --json` (vendored `.bee/bin/*` confirmed `up_to_date` against the edited canonical `packages/bee/*`) all clean.
- `release_manifest --check` reports 394 mismatches; independently confirmed every one is a pre-existing file-mode-only drift (0 sha256/content mismatches, 0 missing, 0 added) — same documented, already-filed noise as three prior cells in this feature line, not excluded from evidence-gathering, just not gated into the pass/fail verify command (matching precedent).
- Cross-model semantic-judge goal-check (builder: sonnet, judge: opus) confirmed both original P1 fixes transcribed correctly, `controlRoot` scoping correct in code (not just documented), and no scope creep beyond relocation + adaptation.
- Independent review session `worktree-concurrency-guard-controlroot-port-review-20260726`: 6 specialist reviewers (code-quality, architecture, test-coverage, api-contract, reliability, plus a security slot that failed to deliver twice — see Known Limitations) + 1 delta re-review of the fail-open fix. **0 P1 across every reviewer and the delta pass.** 2 P2 (one fixed — the fail-open finding; one filed as tooling friction — the failed security dispatch). 16 P3 filed to backlog, none blocking.
- Human UAT: the one CALL/RUN-domain scenario (`bee worktree new` refuses without `--with-companion` while another session is concurrently live over a shared nested checkout; succeeds with it) was presented to the user, who confirmed pass based on the automated evidence above rather than a manual repro.

## How to test it yourself

1. In a checkout with a plain nested git repo (or an unverified companion-mount symlink) and a second live session's heartbeat present, run `bee worktree new --feature <slug>` — expect a refusal naming `--with-companion` as the fix, zero mutation.
2. Re-run with `--with-companion` — expect success.
3. To see the fail-closed fix specifically: `node --test packages/bee/tests/test_claims.mjs` — the two new rows plant a real `ENOTDIR` (a file where `.bee/sessions` should be a directory) and a real `EISDIR` (a session file that is actually a directory), and assert the error propagates under `strict: true` while staying silent (byte-unchanged) under the default.

## Deviations from plan

- The plan's own "CRUCIAL MECHANIC" premise — that `packages/bee/*` would arrive 100% conflict-free from upstream since this branch never touched that path — was wrong in practice: git's rename-detection heuristic paired our old-location modifications with upstream's new canonical paths, producing real 3-way conflicts directly in `packages/bee/{lib/guards.mjs,bee.mjs}`. The effect was favorable, not harmful: git auto-merged most of the already-shipped logic verbatim via the rename pairing, shrinking hand-transcription to the import blocks and the `controlRoot`/`Error`-style edits. `scripts/tests/test_worktree_companion.mjs` arrived as a clean rename with zero conflict.
- The `controlRoot`-vs-`root` differential could not be exercised through `bee worktree new`'s own CLI surface (`controlRootFor(mainRoot) === mainRoot` for every CLI-reachable call from an ordinary checkout) — the differential was instead proven directly against the shared primitive (`hasAnySharedNestedCheckout`) both call sites depend on.
- Two follow-up fixes (fail-open on a hard fs error, at both the directory-listing and per-record-read levels) were found and fixed during the review session itself, after the merge had already been capped and scribed — not part of the original cell's scope.

## Known limitations / follow-ups

- **Security reviewer slot failed to deliver output twice** (both `review-security` and a fresh redispatch went idle with no report text despite repeated direct pings) — filed as tooling friction (P2, `docs/backlog.md`). The security-relevant properties this reviewer would have centered on (self-exclusion, fail-closed-on-error, `controlRoot` as a bypass surface) were nonetheless independently covered by the reliability reviewer, the delta re-review, and a prior cross-model semantic-judge pass — no P1 surfaced anywhere. This is disclosed, not smoothed over: a dedicated security-persona pass never actually ran.
- 16 P3 findings filed to backlog (stale comment, duplicate test-row numbering, 3 real-but-narrow test-coverage gaps, message/doc drift in the old feature's walkthrough) — none blocking, all traceable to `worktree-concurrency-guard-controlroot-port` in `docs/backlog.md`.
- The branch has not yet been pushed to `origin`; PR #64's `mergeable` status has not yet been re-checked against this merge. That is the next step, pending the user's go-ahead.
