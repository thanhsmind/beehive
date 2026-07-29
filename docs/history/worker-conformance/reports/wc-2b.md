# wc-2b — Prove the cap-writes / door-reads seam end to end

**[BLOCKED]** — two step-2 preconditions fail; no source was read for edit, nothing implemented, nothing committed.

## Outcome

Returned blocked before implementation. The dispatch asserts wc-2b was claimed for me
(Bob) and that deps wc-1/wc-2 are capped. The cell records say otherwise on both counts.

## Blockers

1. **Ownership mismatch.** `.bee/cells/wc-2b.json` is `status: "open"` with
   `trace.worker: null` and `trace.claim_session: null` — never claimed, by Bob or anyone.
   The worker contract validates the claim and never creates it (D1), so this is not mine
   to touch.
2. **Uncapped dep.** `deps: ["wc-1", "wc-2"]`. wc-1 is `capped` (worker Kevin). **wc-2 is
   `open`.** Its trace still carries `capped_at: 2026-07-29T09:29:21.546Z`, a populated
   `outcome`, `files_changed`, `friction`, and capCell `warnings` — i.e. it *was* capped and
   was subsequently reverted to open with `trace.worker` nulled. That is consistent with the
   semantic judge's NEEDS_REVISION being applied as a reopen of wc-2 itself.

Both are authority-type blocks (uncapped deps, ownership), which the executing contract
routes to instant `[BLOCKED]` with no advisor consult and no retry.

## What is true in git

The implementation both dep cells describe **is** on `main` — `f852f66a` (wc-1, capCell
stamps `trace.proof = "unrecorded"`) and `50bc9610` (wc-2, both door predicates arm on the
marker), plus `f0f5108e` (wave-close regen). So the code seam wc-2b is meant to prove exists;
only the *cell bookkeeping* disagrees. The divergence is bookkeeping, not code.

## Decision the parent owns

Which of these is the intended shape:

- **(a)** wc-2 stays reopened and absorbs the revision — then wc-2b is redundant and should be
  dropped, or wc-2's action rewritten to include the producer-consumer case; or
- **(b)** wc-2b carries the revision — then wc-2 must be re-capped (its work is already
  committed at `50bc9610`) and wc-2b claimed under a worker before dispatch.

Under (b) this cell is immediately executable as written: scope is clear, the two files are
reservable, no locked decision conflicts the action, and `verify`
(`node packages/bee/tests/test_bee_cli.mjs && node scripts/release_manifest.mjs --check`)
is concrete and runnable.

## Files + commit

None touched. No commit. Full cell record: `.bee/cells/wc-2b.json`.

## Reservations

Reserved `docs/history/worker-conformance/reports/wc-2b.md` to write this report; released
on return. No other reservation taken, none left active.

## Outstanding Questions

1. Is wc-2's reopen deliberate (judge NEEDS_REVISION applied to wc-2) or an artifact of the
   split into wc-2b? The answer picks (a) or (b) above.
2. If (b): should wc-2's re-cap re-verify, given `50bc9610` is already merged and the wave-close
   regen `f0f5108e` landed after it?
