# 2026-08-12 — A refusal must leave its own remedy runnable

Batch: lane-guard-deadlock (cells lgd-1, lgd-2; released as v2.4.8), plus the
flush of four queued capture stubs and three unapplied promote proposals
(harness-p1-fixes, judge-obligation, worktree-first-teeth).

- The user reported a deadlock, not a bug: a session bound to a lane with no
  record was refused on every shell command, and the refusal's own FIX line
  named `bee state session unbind` — a shell command. Only a human typing it
  outside the session could break out. The cause was ordering, not policy: the
  guard resolved the acting lane record before it looked at the command at all,
  so a resolution failure became a blanket deny over inputs the guard never
  meant to judge. **A guard resolves shared state only for the inputs it
  actually judges**, and **a refusal whose remedy names a command must leave
  that command runnable** — the second is checkable by reading any deny text and
  asking whether the guard that wrote it would allow the fix it prescribes.
- The entry door was the real defect. `session bind` accepted any well-formed
  lane id while every other lane-resolving seam refused a binding that named no
  record — a store the writer would not have written itself if it had asked the
  same question the readers ask. When one verb writes what all the readers
  reject, the writer is missing a check, not the readers.
- The red-first proof was worth more than the fix: disabling the new branch made
  the new test fail with the deadlock message verbatim — the deny naming the
  unbind while denying the unbind. A test whose failure output states the bug in
  the reporter's own words is a better regression than one that only goes green.
- Two environment traps cost more than the code did, both from the same root —
  a build artifact outliving the tree it was compiled from. A test crate carries
  `CARGO_MANIFEST_DIR` baked in at compile time, so test binaries built inside a
  feature worktree keep pointing at that path; after the worktree merged and was
  removed, the suite went red with "no such file" on 14 tests that had nothing
  to do with the change. The same stale artifact produced one earlier mystery
  red at cap time, which a rerun "fixed" and hid. **A red that a rerun clears is
  not a flake until its cause is named.**
- Worktree-first has a seam the tooling does not cover: the shared control plane
  refuses cell, route and bind commands from inside a granted worktree, while
  the write guard refuses source edits from main — so the two halves of one
  cell's work live in two directories, and the session must move between them.
  Dispatching a worker does not solve it (a worker inherits the orchestrator's
  directory), which is why both cells ran inline with a recorded reason. Filed
  as friction, not worked around.
- Of three unapplied promote proposals reviewed, none carried content the bundle
  did not already hold: harness-p1-fixes' three bullets and judge-obligation's
  two were already stated by the same-day scribe, and the one pattern candidate
  restated a rule cross-worktree-holds already carries. This is the second batch
  in a row where mining confirmed the scribe rather than adding to it.
