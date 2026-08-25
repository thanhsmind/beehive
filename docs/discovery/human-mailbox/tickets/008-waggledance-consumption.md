---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

What does waggledance already have that an inbox should be built on
rather than beside? Specifically: the notification outbox shipped by
dispatch-blocked-notify (its schema, its drain path, its opt-in switch),
the markdown indexer/viewer's file-watching and frontmatter handling, and
whether an inbox is a new surface or a tab on an existing one. The answer
constrains what the handover spec must promise.

Resolved by direct reads in the waggledance repo; findings linked here.

## Answer

Findings (waggledance repo, read 2026-08-25):

1. **waggledance already reads bee's store directly.**
   `crates/waggledance-core/src/bee.rs` is an ~11k-line pure reader that
   turns `<root>/.bee/` into a typed `BeeSnapshot` — cells, state,
   lanes, sessions, decisions, handoff, config, worktree grants. It
   carries its own numbered decisions about what it may and may not
   open (e.g. only live `.bee/cells/*.json`; `.bee/logs/tools.jsonl`
   never read whole). So the handover is **one more typed reader on an
   existing, already-documented integration**, not a new bridge — and
   the spec's natural shape is the same numbered-contract style that
   module already uses.

2. **A notification outbox exists and is generic enough to reuse.**
   `crates/waggledance-core/src/notify_store.rs` holds a
   `notifications` table `(id, pane_id, kind, body, created_at,
   delivered_at, run_id, project_id)` with a drain path and an opt-in
   switch, plus a real delivery channel at
   `crates/waggledance/src/notify/telegram.rs`. "A letter was filed"
   fits the existing `kind` + `body` shape without schema change —
   which means push delivery of the subject line is nearly free once
   the letter exists, and does not need to be in scope for v1.

3. **Frontmatter parsing already exists** in
   `crates/waggledance-core/src/render.rs`, which is what makes the
   markdown-plus-typed-frontmatter option in
   `tickets/005-record-format.md` cheap on the consuming side too.

Consequences for the frontier: recommendation (a) in ticket 005 gets
stronger — waggledance can parse frontmatter today and reads `.bee/`
today. Whether the inbox is a new surface or a tab stays a waggledance
product question and does not block bee's side.
