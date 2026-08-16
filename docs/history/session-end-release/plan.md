# session-end-release — plan

## Problem

Session liveness is heartbeat-only (900s window). A cleanly closed
session keeps counting as a live worker until its heartbeat goes stale:
the concurrent-worker git guard blocks tree verbs, worktree merge
cleanup defers removal, and sibling sessions wait ~15 minutes for a
session that already left. bee wires `Stop`, `PreCompact`,
`SubagentStop` — but not Claude Code's `SessionEnd` event, so "the
session exited" is never recorded.

## Shape

Two cells, independent files, run in parallel.

### ser-1 — SessionEnd marks the session record closed

- `hooks/session_close/mod.rs`: on `ctx.event == "SessionEnd"`, mark
  the session record `status: "closed"` + `closed_at` (sessions store
  lock, fail-open like heartbeat_session), emit nothing, exit 0.
- `onboard/hooks_wiring.rs`: add `SessionEnd` → `bee-session-close.mjs`
  entry for the Claude wiring arms. Codex has no SessionEnd event —
  named gap, skipped there.
- Regen rendered manifests (`bee dev render-hook-manifests`), update
  wiring/installer tests.

### ser-2 — liveness readers treat closed as not-live

- A `closed` (or `dead`) status short-circuits liveness wherever a
  session record's heartbeat is judged for work-gating:
  `write_guard/store.rs` active_worker_session_ids' stale check,
  `cells/claims.rs` heartbeat_stale (feeds is_concurrent_mode),
  `worktree/prune.rs` live_session_holds (feeds merge cleanup too).
- `state_sync.rs` heartbeat_session: a fresh heartbeat clears
  `closed`/`closed_at` the same way it clears `dead` (revival) — a
  resumed session with the same id comes back alive.
- Tests per touched predicate.

## Cost if wrong

A guard that wrongly reads "closed" blocks nothing it should block —
the closed mark is only ever written by SessionEnd (the session is
gone) and cleared by the next heartbeat (the session is back). Failure
mode is the status quo: waiting out the 900s window.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
