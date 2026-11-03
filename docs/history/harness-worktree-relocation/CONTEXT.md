# Harness Worktree Relocation — Context

**Feature slug:** harness-worktree-relocation
**Date:** 2026-09-16
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

Every harness bee installs for (Claude Code, Codex CLI, OpenCode, Pi) moves its
live session into a bee worktree and back to main through bee verbs, the way
Claude Code's EnterWorktree/ExitWorktree do; the feature ends at the session
move and the merge that follows it, and does not change worktree paths, the
merge transaction, or cleanup.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Add a plain `bee worktree exit` verb: it moves the session from a worktree back to main with no merge and keeps the worktree (like Claude Code `ExitWorktree` keep). Exit-before-merge stays as the merge path. Decision `016ae1ef`, supersedes `0b4ff55b`. | The old "exit only on merge" rule left no way back to main. |
| D2 | When `bee worktree merge` runs from main while the calling harness session still sits in that worktree, bee emits `exit-worktree-before-merge` so the session moves to main first, and the merge runs after the move. Sessionless callers (herding cockpit) pass `--detached` to merge directly. Decision `86337de7`. | Incident 2026-09-16 in memorypad: a Pi agent ran `cd "$main" && bee worktree merge --id …`; the merge landed but the Pi session stayed in the worktree. |
| D3 | On Codex, which cannot move its own session, bee stops, marks a waiting-on question, and tells the user to type `/cd <target path>`; the same chat then continues. Decision `6c860e87`. | Codex `/cd` keeps history and reloads AGENTS.md; only the human can type it. |
| D4 | Bee worktree paths stay sibling paths (`<repo>--wt--<feature>`). On Claude Code the session moves through `EnterWorktree {path}` / `ExitWorktree {action: keep}`, and the user accepts the per-move approval prompt outside `.claude/worktrees/` (none in bypass mode). Decision `c6023a6e`. | Moving paths under `.claude/worktrees/` would change layout for every harness. |

### Agent's Discretion

- The per-harness adapter shape (how each belt or hook reads the transition
  intent) and the transition payload schema change, provided Pi's shipped
  behavior keeps working.
- OpenCode's mechanism (move-session route vs. fork with directory), chosen
  after a local probe; if no in-session move works, OpenCode takes the D3
  fallback shape (tell the user the command, mark waiting).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| session move | The live harness session's working directory changes to the target path while the chat history stays. Not a `cd` inside one shell call. |
| transition intent | The `sessionTransition` payload (and stderr marker) bee emits; bee itself never changes any process cwd. |
| adapter | The per-harness piece (Pi belt, OpenCode plugin, Claude/Codex hook or skill text) that turns a transition intent into a session move or a user instruction. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs` — builds `enter-worktree` (new, enter) and `exit-worktree-before-merge` + continuation (merge inside a worktree); emits the stderr marker only when `PI_SESSION_ID` is set.
- `.pi/extensions/bee-guard.ts` — the one working adapter: validates intent, `SessionManager.forkFrom` + `ctx.switchSession`, runs the merge continuation on main.
- `packages/bee-rs/crates/bee/src/session_identity.rs` — resolves caller session id from `BEE_SESSION_ID` → `CLAUDE_CODE_SESSION_ID` → `PI_SESSION_ID`.
- `.bee/sessions/<id>.json` records (`hooks/session_init.rs`, `hooks/activity.rs`) — `workspace_id`, `activity.cwd`, `last_heartbeat`; liveness window in `verbs/state_group/sessions.rs`.

### Integration Points

- `.claude/settings.json` hooks, `.codex/hooks.json`, `.opencode/plugins/bee-guard.ts` — today none reads the transition intent.
- `AGENTS.md`, `packages/bee/AGENTS.block.md`, `skills/bee-swarming/SKILL.md` — prose that names EnterWorktree; no exit line.
- `bee-herding` merge role runs `bee worktree merge --id … --cleanup` from main with no harness session — needs `--detached` (D2).

## Canonical References

- `docs/history/pi-worktree-session-relocation/CONTEXT.md` and `plan.md` — Pi relocation design this feature generalizes.
- `docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md`, `returning-and-the-merge-gate.md` — current enter/exit behavior.
- `.bee/verify/verify-app/features/worktree-and-close.md` — mapped feature this changes.
- Harness capability research (advisor digest, 2026-09-16): Claude Code 2.1.273 EnterWorktree/ExitWorktree; Codex 0.154.0 `/cd` user-typed only; OpenCode 1.18.30 `/move` and `POST /experimental/control-plane/move-session` (unprobed); Pi 0.85.1 done.

## Outstanding Questions

### Deferred To Planning

- [ ] How bee knows "the caller session sits in worktree X" for D2 across harnesses (session record `workspace_id` / `activity.cwd` vs. env) — read the session record code and real records.
- [ ] Codex and OpenCode session id in shell children (`CODEX_THREAD_ID`? OpenCode `shell.env` injecting `BEE_SESSION_ID`) — local probe.
- [ ] Whether OpenCode's move-session route moves a TUI session between sibling worktrees of one repo — local probe.
- [ ] How the Claude Code hook surfaces "call EnterWorktree/ExitWorktree" to the model (PostToolUse `additionalContext`) — read the hook wiring.

## Deferred Ideas

- None.
