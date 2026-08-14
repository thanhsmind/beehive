# Awaiting Human — Context

**Feature slug:** awaiting-human
**Date:** 2026-08-14
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

Whenever the agent is waiting on the human — a gate, an interview question, a
decision it cannot make alone — the run says so in one persisted place, and
stops saying so the moment the human answers. It ends at the state and the
CLI/JSON that exposes it; no dashboard is built here.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `awaiting-approval` covers EVERY moment the agent is waiting on the human, not only a pending gate. ONE state, plus a field naming what is being waited on — which gate, or the question that was asked. The agent marks the wait when it asks; the mark ends when the answer lands. | Origin: traceable-runs D8 (2026-08-14, user). Today `run_state` reads `awaiting-approval` only when a gate entry is `pending` and none later is approved, so an interview leaves it at `shaping` while the agent sits idle. A dashboard reads that as running. One state was chosen over a second `awaiting-answer` value because the boundary between "approve" and "answer" is arguable and would force a reader to watch two fields for one condition. |
| D2 | The waiting mark ends three ways, all live at once: the `UserPromptSubmit` hook clears it the moment the human sends anything; the agent may clear it explicitly when it acts on the answer; and a mark whose owning session heartbeat has gone stale expires on its own. | A mark only the agent can clear is a lie waiting to happen, and a stale "waiting on you" that nobody clears is exactly the untruth this work exists to remove — a dashboard that cries wolf is worse than none. The hook is the reliable layer because it fires on a real human action rather than on the agent remembering; expiry is the backstop for a session that dies mid-question. |
| D3 | A question asked while no feature is active still records the wait, on the default state record. The mark is session-scoped first, and feature-scoped only when a feature happens to be live. | The gap is not hypothetical: the four gray-area questions that opened `traceable-runs` were all asked before any feature record existed, so a feature-scoped-only mark would miss the exact moment the user pointed at when asking for this. A reader wants one answer to "is this session blocked on me", and that question does not wait for a feature to exist. |
| D4 | Stale expiry reuses the dual-condition rule cell claims already apply — expiry alone never clears a mark; the owning session's heartbeat must also be stale. | Reusing the proven rule avoids a second, subtly different staleness semantics in the same store. A live session that simply took a long time to answer must not have its wait silently erased. |
| D5 | No dashboard, UI, or web surface. This feature delivers the persisted state and the CLI/JSON that exposes it. | Same boundary traceable-runs D7 set; the consumer is still unbuilt, and coupling the shape to it now would be guesswork. |

### Agent's Discretion

Planning owns: whether the mark is a new field beside `run_state` or a richer
`run_state` payload; the exact field names; where the session-scoped mark lives
when no feature is active; the heartbeat/TTL numbers; and whether the agent-facing
verb is a new command or a flag on an existing one.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Waiting mark | The persisted record that the agent has asked the human something and has not yet received an answer. One state plus what is being waited on. |
| Wait subject | The field naming what the mark is about — a gate name, or the question the agent asked. |
| Clearing | Ending a waiting mark. Three paths per D2: the hook on the human's next message, the agent explicitly, or stale expiry. |
| Session-scoped | Recorded against the session/default record rather than a feature's workflow record, so a question asked before any feature exists is still visible (D3). |

## Specific Ideas And References

- The user's framing: "khi agent ask đợi tôi trả lời thì sẽ là phần này" — the
  state must fire when the agent asks and waits, not only at a formal gate.
- The motivating example is this repo's own history: the opening interview of
  `traceable-runs` left `run_state` at `shaping` while the agent was idle.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs` — `run_state`
  landed here in `traceable-runs` cell trun-7, with a closed vocabulary and typed
  refusal on an unknown value. This feature extends that field's meaning, not a
  new parallel one.
- `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs` — the dual-condition
  stale rule D4 reuses: a claim is reclaimable only when the lease is expired AND
  the owner's heartbeat is stale.
- `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` — the `UserPromptSubmit`
  hook, and `build_prompt_reminder` where the phase/gate/next-action lines are
  composed. This is D2's clearing point and its natural surfacing point.

### Established Patterns

- Record wins, projection is rebuilt from it (`docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`).
  Any new field belongs on the record, and must be added to the projection
  explicitly — `apply_workflow_d1_fields` copies a fixed list, so a field omitted
  there never reaches `.bee/state.json`. This exact trap cost `traceable-runs`
  a named plan warning.
- A shared rule is only as good as the census of its callers — the scribing-debt
  reconciliation in `traceable-runs` took three judge rounds because each pass
  found copies the previous had missed. Any read of the waiting mark needs the
  same census discipline.

### Integration Points

- `verbs/workflow_store/projections.rs` — `apply_workflow_d1_fields`, if the mark
  must reach `.bee/state.json`.
- `verbs/status_full/build.rs` — where `traceable-runs` exposed `run_state` and
  the gate records; the same surface a reader will look for this on.
- `hooks/prompt_context.rs` — the clearing hook (D2).
- `hooks/session_preamble/` — where a live wait would be surfaced at session start.

## Canonical References

- `.bee/decisions.jsonl` — traceable-runs D8 (the origin decision), plus this
  feature's D2 and D3, logged 2026-08-14.
- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md` — the
  record, its projection, and the gate/run_state fields this builds on.
- `docs/history/traceable-runs/CONTEXT.md` — D1-D7, whose scope this feature
  deliberately widens.

## Outstanding Questions

### Resolve Before Planning

None. The two gray areas were answered by the user on 2026-08-14.

### Deferred To Planning

- [ ] Does the mark ride `run_state` itself or a sibling field? — answered by
  auditing every reader of `run_state` added in trun-7 and counting what a value
  change would break.
- [ ] Where does a session-scoped mark live when no workflow record exists? —
  answered by checking what the default `.bee/state.json` record already carries
  and what the preamble reads.
- [ ] Can the hook clear reliably in every runtime (Claude, Codex, OpenCode)? —
  answered by checking the hook manifests for `UserPromptSubmit` coverage per runtime.

## Deferred Ideas

- The dashboard itself (D5) — out of scope; this ships its data source.
- Detecting an unmarked wait automatically (the agent asks in prose and forgets
  to mark) — would need the session-stop hook to infer intent; deferred as a
  separate problem from recording a wait the agent does declare.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
