# Herding Review Slots — Context

**Feature slug:** herding-review-slots
**Date:** 2026-08-20
**Shaping session:** complete (user screenshot + relayed ask: review/advisor by herd name)
**Scope:** Quick

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `{kind:"herding", agent:"<name>"}` on ANY slot/purpose — cell, gather, reviewer, advisor, extraction — resolves to the herding-exec Bash payload. Full mapping; the operator owns the pane cost per slot. (Widens herding-executor D7 and ends the gather-default split; decisions logged touches:herding-executor, touches:herding-tier) | The user's target (omp Roles): every role mappable by herd name |
| D2 | Spelling stays `{kind:"herding", agent:"<name>"}` on every slot — never a separate kind, never promptVia (the brief travels by mailbox, not stdin) | One kind, one resolver, no duplicate grammar |
| D3 | Optional `"fallback": "default"` on the herding shape: a failed herding run (spawn failure, timeout, invalid result) re-dispatches through the runtime's default model path for that slot; absent the field, the failure stays loud with the pane kept. Prepare carries the fallback in the payload; the orchestrator doctrine names the re-dispatch move | User: nếu bị lỗi thì đổi về default — declared per slot, never silent |
