# Gate 1 report — pi-worktree-session-relocation

## Recommendation

Approve planning. The four stored decisions define one bounded outcome: move the
current Pi conversation between main and a registered feature worktree through
session replacement.

## Decision coverage

- D1 selects active-session replacement and rejects process cwd mutation.
- D2 requires `SessionManager.forkFrom` plus `ctx.switchSession`, with history
  preserved.
- D3 assigns worktree validation to bee and runtime replacement to the Pi belt.
- D4 fixes lifecycle order: enter after creation; exit before merge; no deletion.

## Boundary

Planning can choose command and transport mechanics. It cannot change the four
requirements above. Non-Pi relocation, cleanup changes, and dispatch changes remain
outside this feature.

## Main risk

A partial session switch could bind conversation history to one cwd while tools use
another. Planning must make replacement atomic or leave the original session active.