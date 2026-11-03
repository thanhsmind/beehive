# Approach: Pi full workflow parity

## Recommended path

Add one explicit `state handoff dismiss` command for pause handoffs. Keep planned-next handoffs on the existing guarded adoption path (D4 and D6; acceptance `0510d81d-24e9-46f7-bf53-701cfaccf978`). Make workflow close preflight all targets, clear open pause records, and rebuild the legacy projection. A planned-next record refuses the full close before any target changes. Also make projection rebuild ignore closed workflows as a recovery defense. Extend the real Pi sandbox after the core behavior exists.

## Rejected alternatives

- Delete `.bee/HANDOFF.json` only — rejected because the mailbox is authoritative and would recreate the stale projection.
- Let `state handoff adopt` clear pause records — rejected because adoption has claim-transfer semantics and must stay planned-next-only.
- Ignore closed workflows only during projection rebuild — rejected as the only fix because it leaves contradictory open records in a closed workflow.
- Clear all handoff kinds during close — rejected because this can discard a planned-next claim without guarded adoption.

## Risk map

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| Handoff mailbox transition | HIGH | A wrong transition can lose an audit record or claim. | pfp-1 | Pause becomes cleared; planned-next refuses unchanged. |
| Workflow close and projection | HIGH | Several close selectors and two state files must remain consistent. | pfp-1 | All selectors use one close helper; multi-close preflight prevents partial mutation; closed workflows cannot project open handoffs. |
| Public CLI registry | MEDIUM | The registry is hand-maintained and controls help and dispatch. | pfp-1 | Registry contract and dispatch examples pass. |
| Pi lifecycle sandbox | HIGH | Unit tests alone do not prove the installed Pi path. | pfp-2 | The onboarded sandbox dismisses, closes, and finds no `HANDOFF.json`. |
| Knowledge state | MEDIUM | The current documents say records persist and list only write/show/adopt. | pfp-1 | Existing workflow-state owners describe dismiss and close behavior. |

## Files and order

1. Add mailbox and C1 dismissal primitives.
2. Wire the command, registry entry, and close helper.
3. Add behavior tests before the implementation turns them green.
4. Update the two existing workflow-state knowledge owners.
5. Extend the installed Pi sandbox and run its exact lifecycle test.

## Questions still open

- None. The public command name is `bee state handoff dismiss`; its narrow pause-only meaning prevents claim loss.
