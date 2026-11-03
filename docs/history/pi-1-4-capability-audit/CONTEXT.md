# pi-1-4-capability-audit — Context

## Request

Upgrade pi to 1.4.0 (done via mise, `pi --version` prints 1.4.0) and make the bee Pi belt support it.

## Locked decisions

- D1 — The belt's `before_agent_start` handler runs the `prompt-context` and `activity` hooks only when `event.messageType` is `"user"` or absent (pre-1.2 hosts). A `"custom"` run (`pi.sendMessage(..., {triggerTurn: true})`) still sets the busy latch (`selfBusy`, `turnStartPending`, `activeDrainCtx`) but injects nothing and logs no UserPromptSubmit activity.
- D2 — The proven ceiling moves from 1.0.0 to 1.4.0 on live evidence: a real `pi` 1.4.0 session in this repo loads the belt and runs one turn without an extension error. The belt header in `.pi/extensions/bee-guard/index.ts` and `docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md` record it.
- D3 — The 1.0.0 to 1.4.0 delta is dispositioned in the audit page: 1.2.0 `before_agent_start` messageType (adapt, D1); 1.2.0 run-start busy window for `sendUserMessage` without `deliverAs` (keep: `sendLeaderMessageOutcome` already catches the throw and resets `turnStartPending`; recorded as a re-probe row); 1.3.0 resumed sessions keep active tools (keep: `turn_start` captures `fullToolSet` from `getAllTools` first); codemode worker queueing, compaction output limit, schemas, model defaults (no belt surface).

Store id: 5345c42d-50e7-426d-a734-568dff58008a.

## Out of scope

- Any use of codemode, `reachesTools`, or `$schema` validation in bee.
- Changing the floor (stays 0.84.4).
