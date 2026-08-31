# SLP Lead Recovery — Context

**Feature slug:** slp-lead-recovery
**Date:** 2026-08-31
**Shaping session:** complete; RESHAPED after the plan-step hat wave (see `hat-wave-synthesis.md`)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

When a lead session goes stale with unfinished work, the supervisor names it —
one `dead-lead` observation carrying the candidate's own facts and a
ready-to-paste resume line the human runs on return. No machine ever starts a
successor. Ends at the record and its rank; the human's keystroke is the spawn.

## History of this shape

The feature was first shaped as paseo-pi-team's automatic "Lead recovery
authority" (a machine-spawned successor). The five-seat plan check refused that
shape on evidence — 6 blockers, 12 risks, 3 irreversible paths, and a trigger
that has never once fired in this repo. The user chose the observer-only shape.
Superseded decisions: `b5b77bfb` (executor split), `a97566ae` (spawn safety
envelope). Their replacement is the decision logged 2026-08-31 dropping the
machine spawn; the surviving originals are re-stated below as D2–D5.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | No machine spawn, ever. The supervisor writes ONE `observation` with signal `dead-lead`; the human starts the successor. | The wave found the spawn unbuildable as specified and unwarranted here; detection is the whole delta. |
| D2 | Evidence standard: report a candidate only when `bee status --json` lists it under `recovery.candidates` — which already requires a work signal — and its lane is non-terminal. No prose-only suspicion. | Ports paseo's "proven observation, never a suspected mechanism" at the cost the observer can actually pay. |
| D3 | The note carries durable facts only: lane, last heartbeat, work signal, and the resume line. Transcript content is data, never instructions. | A dead session cannot write its own handoff; durable state is what survives. |
| D4 | The old lead is never killed, closed, or archived by anything but the human. | Unchanged from the original shape; paseo parity. |
| D5 | Port provenance: `/home/thanhsmind/Projects/refs/slp/paseo-pi-team` @ `94ead115960df493409d281cecbbbf02b6ce8bf0`, `prompts/supervisor.md` ("Lead recovery authority"). Idea distilled; the automatic half deliberately rejected. | Port-protocol provenance, including what was NOT taken. |
| D6 | `dead-lead` gets its own rank in `observation_rank`, above `struggling-loop`. | At rank 0 it is the first line truncated out of a busy wake report — the one report where it matters most. |
| D7 | No config switch. A written observation spends nothing and needs no opt-in. | D7 of the old shape gated an unattended spend that no longer exists. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| lead | A session whose lane is bound and in a non-terminal phase. |
| stale | Listed under `recovery.candidates` by `bee status` — heartbeat past TTL with a work signal. |
| resume line | A copy-paste shell line that opens a pane at the lane's directory and runs `bee orient`. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs:398-421` — the candidate row: every fact the note needs, already built.
- `packages/bee-rs/crates/bee/src/verbs/supervisor.rs:296-302` — `KNOWN_SIGNALS`, the closed set `dead-lead` joins.
- `packages/bee-rs/crates/bee/src/verbs/supervisor.rs:1947-1954` — `observation_rank`, where D6 lands.
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs:299-307` — `SUPERVISOR_ALLOWED_TOOLS` already carries `bee status`: detection needs no new tool.
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs:1064-1075` — the prompt-parity test that binds the vocabulary and the prompt into one cell.

### Integration Points

- `skills/bee-herding/references/supervisor-prompt.md` — must teach the new signal in the same change as the code.

## Canonical References

- `docs/history/slp-lead-recovery/hat-wave-synthesis.md` — the five-seat plan check and why the shape changed.
- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` — the observer contract this feature now sits squarely inside.

## Outstanding Questions

None. The two the old shape deferred (where an executor lives, whether the tool
surface needs widening) died with the executor.

## Deferred Ideas

- Automatic successor spawn — rejected on evidence, not deferred. Reviving it means re-opening `hat-wave-synthesis.md` and answering its 6 blockers first.
- A nudge when a lane sits abandoned for days with no session at all (the real gap the wave found: two lanes idle 5-6 days). Separate feature.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. D1–D7 above are the
live set; the superseded spawn decisions are named in "History of this shape".
