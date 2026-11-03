---
artifact_contract: bee-context/v1
feature: pi-harness-workflow-parity
status: locked
source: user request and independent ipad-dashboard-mobile audit
locked_at: 2026-09-11
---

# Pi harness workflow parity

## Outcome

A Pi session gets the same Bee lifecycle checks as a Claude Code session. The checks cover source writes, gate review, proof records, intent attribution, dispatch briefs, and session identity.

## Decisions

### D1. Enforce one lifecycle contract in each supported runtime

The Rust CLI owns the shared rules. The Pi plugin and generated host helpers use those rules. Runtime-specific code can translate events, but it cannot weaken a rule.

This decision extends the parity gap recorded by decision `649602e4-ea83-4117-ace9-a2ee7749049b`.

### D2. Deny source writes from the main checkout before execution approval

A source write requires an active feature, the approved shape and execution gates, and the correct feature worktree. A stale feature or an approved gate for another feature does not grant access.

The guard remains a safety check. The approved workflow remains the authority.

### D3. Show the exact execution packet before gate approval

The shape gate preview shows each current-slice cell. Each preview includes the action, files, required reads, observable result, must-have behavior, and exact verification command.

The persisted cell must match the approved preview. A different packet requires a new plan revision and a new approval.

### D4. Store proof that another session can run again

Each new cap stores the exact command, the result class, and the evidence reason in structured trace fields. The cap rejects descriptive prose in place of a command.

The proof command must match the approved cell verification command. Historical caps remain readable.

### D5. Keep intent and dispatch context feature-scoped

Intent lookup never falls back to an anchor from another feature. If the active feature has no anchor, the command fails closed and names the missing anchor.

`dispatch prepare --purpose` copies the supplied purpose into the worker brief. The command never emits an empty task after it accepted a non-empty purpose.

### D6. Recognize Pi session identity wherever the CLI recognizes Claude Code

The shared environment order is `BEE_SESSION_ID`, `CLAUDE_CODE_SESSION_ID`, then `PI_SESSION_ID`. Explicit command flags still win.

All session-owned commands use one implementation of this order. This includes work records, claims, reservations, waiting marks, locks, recovery, and status.

### D7. Repair the two remaining Waggledance records

After the harness change passes its proof, run the original `idm-1` verification command and record fresh proof through the CLI. Supersede the stale broad responsive decision with the later dashboard-specific decision through the decision CLI.

Do not change the four audit findings that already have a verified repair.

### D8. Allocate collision-safe herding job ids

Concurrent `bee herding run` calls must allocate different job ids. The allocator owns uniqueness. Callers do not serialize work or add sleeps.

This decision records the reproduced collision from three runs that selected `job-1789132580732`.

## Acceptance

- A Pi-only session id resolves for each tested session-owned command.
- A pre-gate source write on main is denied in both Claude and Pi hook paths.
- A gate preview contains each exact cell verification command and execution field.
- A cap rejects a proof command that differs from the cell packet.
- A successful cap fills the structured proof fields.
- Intent lookup cannot return an anchor for another feature.
- `dispatch prepare --purpose` puts the purpose text in the worker prompt.
- Three concurrent herding runs allocate three different job ids and start without `agent_name_taken`.
- The Pi plugin contract tests show parity with the generated Claude hooks.
- The Waggledance `idm-1` record contains fresh replayable proof.
- The stale Waggledance responsive decision is no longer active.

## Constraints

- Keep gate bypass behavior. Full bypass records the same approvals without a human stop.
- Keep independent review user-invoked.
- Keep old cell records readable.
- Do not read a secret-shaped file without user approval.

## Out of scope

- Product changes to the iPad dashboard.
- A new runtime adapter.
- Automatic migration of all historical proof records.
