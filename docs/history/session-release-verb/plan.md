# session-release-verb — plan

## Problem

session-end-release made a CLOSED session release its holds instantly,
but a session that stays open between tasks still counts as a live
worker for up to 900s. One session does many things; between tasks it
holds nothing and should be able to say so.

## Shape

### srv-1 — the verb + revival semantics

- New `bee state session release` (state_group/sessions.rs, mirroring
  `session unbind` for registration/flags): resolves the session id
  (flag → BEE_SESSION_ID → CLAUDE_CODE_SESSION_ID), marks the record
  `status:"closed"`, `closed_at`, plus `released:true` for provenance.
  Liveness readers already treat closed as not-live — zero guard edits.
- Revival split: the PostToolUse heartbeat (state_sync) must NOT revive
  a `released:true` record — otherwise the release command's own
  trailing hook (or any same-turn tool call) would undo it. The
  UserPromptSubmit path (prompt_context) and session-init DO revive,
  clearing status/closed_at/released and stamping revived_at: the user
  speaking again is the re-engage signal.
- Tests: verb writes the mark; PostToolUse heartbeat leaves it; prompt
  revival clears it; guards (is_concurrent_mode) read released as
  not-live.

### srv-2 — doctrine line

- "Care for the session" list in the AGENTS block source
  (packages/bee/AGENTS.block.md): one line — end substantial work by
  running `bee state session release` after caps/reservations are
  clean. Regen the derived AGENTS.md via the repo's regen path; update
  any test pinning the block.

## Cost if wrong

Release is advisory-safe: worst case a released session's next
same-turn write meets guards as a "new" concurrent writer — but the
next user prompt revives before real work resumes. Failure mode of not
having it is the status quo (900s wait).

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
