# pi-worktree-session-relocation — CONTEXT (locked decisions)

**Feature slug:** pi-worktree-session-relocation  
**Scope:** High-risk  
**Domain types:** CALL | RUN | ORGANIZE

## Feature Boundary

Move the current Pi conversation between the main checkout and its bee-managed
feature worktree. Bee verifies the target and emits structured transition intent.
The Pi belt performs the session replacement through Pi's supported session APIs.

This feature extends the worktree lifecycle shipped by
`worktree-session-routing`. It does not replace worktree creation, merge, or
cleanup.

## Locked Decisions

These decisions are fixed. Planning must cite them and must not reinterpret them.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `c47fa930-a3ba-44ae-a6a3-aac3e383647a` | Pi worktree navigation uses active-session replacement and does not mutate process cwd. | Pi binds tools, resources, trust, and persistence to session cwd. Session replacement rebuilds them together. |
| D2 | `1f947417-5cf9-48e4-a0c8-e3b7fe3aa0a1` | The current conversation moves across cwd with `SessionManager.forkFrom` and `ctx.switchSession`. | Pi 0.85.1 exposes this supported cross-project session path and preserves history. |
| D3 | `85d85ede-7ad1-4dee-8931-f9484e1a5328` | Bee CLI emits structured session-transition intent. The Pi belt validates it and performs the harness action. | The CLI owns verified worktree identity. The harness owns runtime replacement. |
| D4 | `0b4ff55b-f4d1-4ffb-8018-91b5882c6aa3` | Enter follows successful worktree creation. Exit occurs before merge and never deletes the worktree. | `bee worktree merge` remains main-only. Cleanup remains a separate operation. |

### Agent's Discretion

- The exact CLI and Pi-extension integration shape that carries transition intent.
- The exact command names and output fields, if existing command contracts do not
  already decide them.
- The session-file location and fork sequence, provided the current conversation
  and history move as D2 requires.
- Test decomposition and documentation placement.

## In Scope

1. Bee CLI behavior that emits validated enter and exit transition intent.
2. Pi belt behavior that consumes the intent and replaces the active session.
3. Conversation-history preservation across both directions.
4. Contract, failure-path, onboarding-copy, and end-to-end tests.
5. User and agent guidance for the Pi worktree transition.

## Out of Scope

- Process-wide cwd mutation.
- Worktree deletion during exit.
- Replacing `bee worktree new`, `bee worktree merge`, or their cleanup behavior.
- Automatic session relocation for harnesses other than Pi.
- Changes to worker dispatch or result-mailbox transport.

## Existing Code Context

### Reusable Assets

- `.pi/extensions/bee-guard.ts` — the project-local Pi belt.
- `packages/bee-rs/crates/bee/src/verbs/` — the Rust CLI verb and driver surfaces.
- `docs/history/worktree-session-routing/CONTEXT.md` — parent worktree decisions.
- `docs/history/pi-support/CONTEXT.md` and
  `docs/history/pi-beehive/CONTEXT.md` — Pi belt constraints and parity rules.

### Canonical External Reference

- `/home/thanhsmind/.local/share/mise/installs/pi/0.85.1/pi/docs/extensions.md`
  — Pi 0.85.1 extension and session-replacement contracts.

## Outstanding Questions

### Resolve Before Planning

None. D1-D4 define the product boundary.

### Deferred To Planning

- Which existing worktree command surfaces should carry enter and exit intent?
- How does the Pi belt receive the structured intent without parsing prose output?
- What exact `SessionManager.forkFrom` and `ctx.switchSession` sequence preserves
  the current conversation across project cwd values?
- Which failure leaves the user in the original session, with zero partial state?

## Deferred Ideas

- Relocating non-Pi harness sessions.
- Deleting a worktree as part of exit.
- Mutating the Pi process cwd in place.

## Handoff Note

Planning must validate each Pi API and CLI seam against current code. It must carry
D1-D4 into the plan, cells, and user acceptance checks.