# Prompt Work Record — Context

**Feature slug:** prompt-work-record
**Date:** 2026-08-25
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | SEE

## Feature Boundary

Every user prompt lands in bee's session record as a work item the moment it
arrives — carrying the user's own words, in status `open` — so a dashboard can
say what a session is working on even when the session never calls a bee flow
verb. It ends at the record and its status transitions; it does not change how
cells, lanes, or gates work.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The UserPromptSubmit hook writes a stub carrying the user's VERBATIM prompt in status `open` before work starts; the agent then upgrades that stub with an acceptance and moves its status as the work advances. (decision `bd78f64a`) | A hook does not depend on the agent remembering; the agent supplies the acceptance a hook must never invent. |
| D2 | The stub rides on the existing session record `.bee/sessions/<id>.json`, not a new store, and is promoted to a board card only once the agent has upgraded it with an acceptance. (decision `856789db`) | waggledance already reads that directory, so the text reaches the screen as a field addition; promoting only on upgrade keeps trivia off the board. |
| D3 | A follow-up prompt appends to the record that is still `open` instead of opening a second one; the first prompt's text stays the record's title. (decision `5944ebbb`) | One job spans many turns — a record per turn would chop it into ten entries. |
| D4 | Only the agent moves a record out of `open`. A record whose session heartbeat has gone stale expires with that session. (decision `34944b7a`) | A turn boundary is the wrong place to call unfinished work done; heartbeat expiry reuses the liveness rule the cockpit already applies. |
| D5 | The stored prompt keeps the user's wording, with absolute paths scrubbed the way waggledance already scrubs `next_action`, and a secret-shaped prompt stored as `[redacted]`. (decision `f9bf6456`) | The board is reachable over a tunnel, so prompt text is a real exposure surface. |
| D6 | The mechanism ships from this repo; waggledance's rendering of the new field is a separate dependent feature in that repo. | The two repos version and release independently; bee's field must exist before a reader can show it. |

### Agent's Discretion

The field name and shape on the session record, the status vocabulary beyond
`open`, the CLI surface the agent uses to upgrade and move a record, and the
secret-shape detection reused from bee's existing guard — all planning's call,
within D1–D6.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| stub | The hook-written record: verbatim prompt, status `open`, no acceptance yet. |
| upgrade | The agent's write that adds an acceptance to a stub, which is what makes it board-worthy (D2). |
| record | The work item on `.bee/sessions/<id>.json`, stub or upgraded. |

## Specific Ideas And References

- The symptom the user reported: a `bee-researching` (`xia`) run showed a live,
  busy session with nothing on the board saying what it was researching.
- The user's framing is that the record is created on intake — "the content is
  what the user typed, as `open`" — before any routing decision is made.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/hooks/activity.rs` — the only writer of the
  session record's `activity` object, with the store locking, the capped log,
  the herded-pane mailbox sink and the always-exit-0 posture the new write needs.
- `packages/bee-rs/crates/bee/src/hooks/adapter.rs:265` — `HookContext.payload`
  is the raw hook stdin, so the UserPromptSubmit `prompt` field is already in
  reach; no bee hook reads it today.
- `bee intent set` — the existing verbatim-request anchor. It is not the store
  here (D2), but its immutability rule ("verbatim IS the mechanism", re-setting a
  changed request refuses without `--force`) is the precedent D3 should match.

### Established Patterns

- Passive-measurement hooks (`activity`, `tools_logger`): never deny, never
  print on stdout, always exit 0, every failure swallowed into
  `.bee/logs/hooks.jsonl`. The new write inherits this posture.
- Opt-out hook gating: `state.rs:193` `hook_enabled` treats an absent
  `hooks.<name>` key as enabled.

### Integration Points

- `.bee/sessions/<id>.json` — the record the field lands on.
- `packages/bee-rs/hooks/*.json` — the rendered per-runtime hook manifests, if
  the write needs a hook matcher that is not already wired.

## Canonical References

- `crates/waggledance-core/src/bee.rs:582` (waggledance) — `BeeActivity`, the
  reader's view of the record today: `state/event/tool_name/tool_use_id/at/
  age_seconds/pane/cwd/feature/cell`, and no field for what the work is.
- `crates/waggledance-core/src/bee.rs:2445` (waggledance) — the `.bee/sessions/*.json`
  read, including the rule that a malformed activity object is dropped rather
  than failing the session parse.

## Outstanding Questions

### Deferred To Planning

- [ ] Whether the write belongs in `hook activity` (already UserPromptSubmit-wired
      and already the session record's only writer) or in a sibling hook — an
      inspection of the wired matchers answers it.
- [ ] What "secret-shaped" reuses — bee already refuses secret-shaped decision
      text and guards secret-shaped files; planning names which guard applies.
- [ ] How a herded pane behaves: that pane gets no `.bee/sessions/<id>.json` at
      all, only the job mailbox, so the record needs a stated destination there.

## Deferred Ideas

- Rendering the record on the waggledance board and promoting an upgraded record
  to a card — the dependent feature in that repo (D6).
- Backfilling records for sessions that predate the field — no reader depends on
  history, so the field starts empty.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
