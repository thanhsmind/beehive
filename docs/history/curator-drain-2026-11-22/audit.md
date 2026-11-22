# Capture-queue drain, 2026-11-22

The capture queue held 51 stubs: 49 machine-made promote proposals (32 features, 15 releases, 2 duplicates) and 2 hand stubs. Four read-only gather workers checked the 32 feature proposals against `docs/knowledge/` (reports: `gather-1.md` to `gather-4.md`). The leader read the 15 release proposals and the 2 hand stubs directly. Section (a) delivery drafts were not applied, because the bundle no longer keeps delivery records.

## What changed in the bundle

| Change | File | Evidence |
|---|---|---|
| Recurrence of the stale deploy permit (2.78.0, 2.86.0, 2.90.0, 2.91.1) past the pattern's stop, plus the rule "commit, then prepare the permit last" | `docs/knowledge/patterns/20261027-a-dispatched-stage-that-fails-preflight-gets-run-inline-by-the-leader-and-the-preflight-never-gets-fixed.md` | release proposals 2.78.0, 2.79.0, 2.86.0; `.bee/cells/archive/release-2-90-0/rel900-1.json`, `.bee/cells/archive/release-2-91-1/rel911-1.json` |
| The same inline-after-refusal shape in eight feature cells (concurrency refusal, idle worker, missing cap) | same pattern | gather-2, gather-3, gather-4 |
| The spoken line of `agents-context-handoff-65` still said "a context mark never stops work"; it now matches the playbook (120000 mark, hand off at the next clean stop, after every merge) | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` | `packages/bee/playbooks/handoff.md` rule block; `hook-runtime/token-economy-of-injected-context.md` |

## What needed no change

- Area bullets reported PRESENT: already stated in their owning concept.
- Area bullets reported ABSENT in a second area while PRESENT in the owning one: not copied (one fact, one home).
- `durable-state-writes` dsw-1 (fsync) was reported ABSENT; it is stated at `workflow-state/workflow-records-and-projections.md` (the "Durable store writes" pointers) and matches `fsutil.rs` `write_text_atomic` / `append_jsonl`.
- CONFLICT rows `lean-context` lnc-1 (150000), `context-mark-advisory` cma-1, `handoff-self-relaunch` hsrl-1/2: the proposals are older than the bundle; later decisions (token-spend-cuts D1, leader-context-thrift D1) set the current rule, which the bundle states.
- CONFLICT rows `agent-efficiency` aef-2 (auto-cap) and aef-3 (prompt skew): the bundle carries the later herding-leader-toil and lane-discipline statements. Not re-verified against code in this drain.
- Release pattern candidates: routine deviations, or the inline deploy path already covered by the pattern above.

## Lessons seen once, not promoted

Single source, no waste figure, so below the promotion bar: claude session env propagation (cssd-1), plan fact made false by its own D5 (pw-4), registry generator missing (cfg-1), tests written after code (cqp-2, rfl-1), worktree checkout lacking CONTEXT.md (dsw-1), three git merge-graph traps in freshness grading (kfm-1), read-set paths checked against the caller root (kdf-1), a config guard that reserves role models for the user (lnc-2).

## Follow-ups

- Backlog friction row: the session preamble names `bee deferred-queue complete`, a verb the binary does not have.
- `agent-efficiency` aef-4, aef-5, aef-6 (goal-check gap verdict, full prior-round reasons, trimmed worker prompt) are not stated in the bundle and were not verified against code here.
