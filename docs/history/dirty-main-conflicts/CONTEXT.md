# dirty-main-conflicts — CONTEXT

## What was asked

Two agent sessions running at once kept deadlocking: a `tiny`/docs task ran in
the MAIN checkout, left tracked bee bookkeeping uncommitted, and the sibling
session's `bee worktree merge` refused with `WORKTREE_MERGE_MAIN_DIRTY`. The
user asked for an audit of every flow that can cause this, then for the five
findings to be fixed in order.

## What was found

An audit (three read-only passes, then two blast-radius checks) produced five
candidates. Verification demoted two of them and narrowed a third. What the
evidence actually supports:

### 1. Merge auto-commit scope is narrower than what bee itself writes — REAL

`bee worktree merge` auto-commits main's bookkeeping dirt before merging, but
`main_bookkeeping_roots` (`packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:170-176`)
returns only `.bee` plus `docs/history/<feature>`. bee's own `decisions log`
also writes `docs/decisions/taxonomy.json`
(`packages/bee-rs/crates/bee/src/verbs/decisions/mod.rs:120-121`), and the
capture chain writes `docs/knowledge/**`. Both are tracked and both sit outside
the auto-commit scope, so bee's own output refuses bee's own merge
(`verbs/worktree/phases.rs:122-136`).

This is the defect the user actually hit: at session start `git status` showed
exactly `.bee/decisions.jsonl` and `docs/decisions/taxonomy.json` dirty, and the
pending `hold-holder-attribution` worktree could not land until they were
committed by hand.

### 2. The unresolved-worker-count arm hides the working escape — REAL, NARROW

The concurrent-worker git guard has two deny arms
(`packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:653-681`). The
`count > 1` arm hands back `CONCURRENT_TREE_TEMP_INDEX_REMEDY`
(`hooks/write_guard/paths.rs:365-372`), which names the escape that works:
"A genuinely path-scoped `git commit -- <your paths>` is allowed too."

The `Unresolved` arm (`checks.rs:655-669`) replaces that with a one-line remedy
naming only `.bee/reservations.json`. A session that hits it is told to repair a
file, never told it could simply land its work path-scoped.

The original claim behind this finding — that a second session cannot clean main
at all — is wrong, and the check disproved it. `git commit -- <paths>` is
already exempt in BOTH arms (`paths.rs:317-326`), which is how main was cleaned
by hand at the start of this session. The remaining gap is only the missing
sentence in the unresolved arm's remedy.

`Unresolved` fires only when the reservation store is present but unparseable
(`paths.rs:397-399`), so this is a rare path — cheap to fix, low blast radius.

### 3a. The `tiny` exemption ignores its own live-session condition — REAL

`AGENTS.md:37-39` conditions the main-checkout exception on "a solo `tiny` fix
when no other session is live", and
`docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md:33` states
the same rule: "A `tiny` fix may stay in main only while no other live session
is present (heartbeat + non-idle phase, D9a); with one, it takes a worktree like
any feature."

The guard drops the condition (`hooks/write_guard/hook_local.rs:661`,
`if lane == "tiny" { return Ok(None); }`). Its own comment names the gap and
schedules the repair: "left unconditional rather than guessed at. Gap named here
per cell wtf-3; closing it is a separate cell." This feature is that cell.

The comment's stated blocker — that the live-session walk "has no equivalent on
this path without a new store dependency" — no longer holds. The write-guard
module already carries the pieces in-module: `hooks/write_guard/store.rs:383`
(`is_concurrent_mode`, with the same self-exclusion semantics),
`store.rs:374` (`heartbeat_stale`), `store.rs:364`
(`HEARTBEAT_STALE_SECONDS = 900.0`). The canonical predicate this must agree
with is `verbs/state_group/workflows.rs:587-591`
(`is_code_touching_lane`: `lane == "tiny" && !other_live_session`), already
pinned by `verbs/state_group/tests.rs:1536-1540`.

This is the exact collision the user hit: a `tiny` fix editing main while a
sibling session was live.

### 3b. The blanket `.md` exemption is wider than its own reason — USER'S CALL

`hook_local.rs:545-547` exempts every path ending `.md` from worktree-first,
at every lane and every phase. Its recorded reason is a single ported comment,
"docs-lane spelling outside docs/" (origin `7792b0a3`), and that reason is now
dead: the docs lane returns earlier at `hook_local.rs:604`, so it never reaches
this function.

Tightening it would NOT break the documented docs-lane exception, and would not
touch `docs/**`, `.bee/**`, `plans/**`, or `AGENTS.md` — all still exempt via
`GATE_ALLOWED_PREFIXES_INTAKE` (`guards.rs:42`). It bites only on `.md` outside
those prefixes — `README.md`, `CLAUDE.md`, `skills/**/*.md` (the handbook, ~80
files) — written from main by a NON-docs lane. Whether that traffic should be
forced into a worktree is a judgment call about how the handbook gets edited,
not a defect, so it is carried as the single open decision and not implemented
here.

### 4. An expired path reservation still blocks its own path — REAL

`list_reservations(.., active_only = true)` filters expired leases out of the
conflict pre-check (`verbs/reservations/leases.rs:330-338`), but the O_EXCL
`AlreadyExists` arm (`verbs/reservations/reserve.rs:309-330`) reads the stale
lease off disk and reports it as a conflict without ever consulting
`expires_at`. A dead session's reservation on the exact same path therefore
blocks a new session past its own TTL.

Unlike cell claims, reservations have no sweep door at all: `sweep_expired_leases`
(`packages/bee-rs/crates/bee/src/lease_store.rs:779`) is reachable from
`integration_queue.rs:376` and the manual `bee reservations sweep`, never from
`orient`, `recovery scan`, or any session-start path.

### 5. `cells claim <id>` does not sweep — DEMOTED, NOT A DEFECT

The claim sweep already runs at three doors: `bee orient`
(`verbs/status_full/orient.rs:274`), `bee recovery scan`
(`verbs/status_full/recovery_verb.rs:221`), and `bee cells claim-next`
(`verbs/cells/handlers_select.rs:662`). Every session runs `orient` when it
routes work, so a stale claim is cleared before a targeted `cells claim` can
trip on it. No change is warranted; the finding is recorded here so the next
audit does not re-raise it.

## Decisions

- D1 — The merge auto-commit widens to bee's own tracked output, and no
  further. `docs/decisions/` and `docs/knowledge/` join `.bee` and
  `docs/history/<feature>` in `main_bookkeeping_roots`, because bee writes all
  four itself. Arbitrary source dirt keeps refusing, named by path, exactly as
  the existing test `dirt_outside_bee_still_refuses_and_names_the_offending_path`
  (`verbs/worktree/tests.rs:1335-1353`) asserts.
- D2 — The unresolved arm keeps its own remedy (naming the reservation store is
  the right first move for a solo session) and gains the path-scoped-commit
  sentence the other arm already carries. The two arms stay distinct; only the
  missing escape is added.
- D3 — A stale, expired lease is taken over rather than reported as a conflict,
  and the sweep gains a door so the takeover is the rare path rather than the
  normal one.
- D4 — The `tiny` exemption gains the live-session condition its own comment,
  AGENTS.md, and the knowledge doc all already state, using the write-guard
  module's in-module `is_concurrent_mode` so no new store dependency is added.
  It stays fail-open on any read error, matching the guard's discipline
  (`hook_local.rs:630-639`).
- D5 — Findings 3b and 5 are not implemented in this feature. 5 is a non-defect.
  3b is a judgment call reserved for the user.

## Also observed

A `bee-review` subagent dispatched from this worktree session had every Bash
command refused: its cwd resolved to the shared checkout, not the session's
worktree, and the containment guard refused each one. Same failure family as the
findings above — a guard doing its job against a caller it cannot place — but a
dispatch-plumbing defect, not a merge or reservation one. Recorded here, not
fixed here.

## Out of scope

- The `gate_bypass: normal` setting in `.bee/config.json`. It is the user's
  switch, and turning it off is a separate, one-line decision.
- The 8 unapplied promote proposals and 8 queued capture stubs already reported
  by `bee orient`. Pre-existing debt, not caused by and not fixed by this work.
