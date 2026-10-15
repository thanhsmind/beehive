# Paseo worktree workspaces — Context

**Feature slug:** paseo-worktree-workspace
**Date:** 2026-10-15
**Shaping session:** complete (gate bypass full; spec-drop sup-20261014-6234 from waggledance@667307d)
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

When bee runs inside Paseo, the bee worktree lifecycle must show in Paseo.
`bee worktree new` registers the worktree as a Paseo workspace. Merge
cleanup and prune archive that workspace. A Paseo herded worker starts in
the workspace of its worktree, never in the workspace of the calling agent,
and never as a child of the calling agent.

Evidence (waggledance, Paseo 0.11.2, 2026-10-14): feature worktrees never
showed in Paseo, and `paseo run --cwd <dir>` from inside an agent attached
the new agent to the caller's workspace and made it a child of the caller.
Captures: `waggledance/crates/waggledance/testdata/fixtures/paseo-0.11.2/README.md`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | bee is "inside Paseo" when `PASEO_AGENT_ID` or `PASEO_CLI` is set and not empty. Outside Paseo, `bee worktree new` and the herding run do no new Paseo call. Unit tests see "outside Paseo" unless a test sets a seam. | The spec names these two variables. A test run inside a Paseo session must not create real workspaces. |
| D2 | After `bee worktree new` creates and registers a worktree, and bee is inside Paseo, bee makes sure a Paseo workspace exists for the worktree path. It first reads `paseo workspace ls --json` and reuses a workspace whose `cwd` is the same directory (paths compared after canonicalize). Else it reads `paseo project ls --json`, takes the `projectId` whose `path` is the main checkout (canonical compare), and runs `paseo workspace create --path <worktree> --isolation local --project <projectId> --json` (no `--project` when no project matches). bee records the id as `paseo_workspace_id` on the worktree's bee workspace record (`.bee/runtime/workspaces/<id>.json`) and prints it (`paseo: workspace <id>`; JSON field `paseo_workspace`). A Paseo failure prints one warning line and never fails or rolls back the worktree. | The `project` field of `workspace ls` is a name, not an id: `--project beehive` fails with `Unknown project: beehive`. The main checkout path in Paseo can be a symlink of bee's path (`/home/...` vs `/mnt/data/...`). |
| D3 | When a worktree's bee record is removed by `bee worktree merge` cleanup, `bee worktree prune`, or `bee worktree unregister`, and the record carries `paseo_workspace_id`, bee runs `paseo workspace archive <id> --json` first. A failure prints one warning line on stderr and never blocks the teardown. This step does not depend on D1: the record decides. | The record is the only place the id lives; once the record is gone the id is lost. |
| D4 | `bee herding run` on a Paseo agent passes `--workspace <id>` to `paseo run`. The id is the `paseo_workspace_id` of the bee workspace record whose root holds the run cwd. When no record carries one and bee is inside Paseo, the run makes sure of a workspace for the cwd as in D2 (and records it when a bee record matches). When that fails inside Paseo, the run refuses before spawn with a FIX line — it never falls back to the caller's workspace. Outside Paseo with no recorded id, the argv does not change. | `paseo run` from inside an agent ignores `--cwd` without `--workspace` and joins the caller's workspace. |
| D5 | Every paseo process bee starts (the `PaseoCli` runner, the `wait` source, the prepare probe) runs with `PASEO_AGENT_ID` and `PASEO_AGENT_CWD` removed from its environment. The `--env` pairs bee gives a Paseo worker do not carry these two variables. | An agent created from inside an agent gets `ParentAgentId` = the caller. |
| D6 | bee reads paseo JSON output as the first JSON document in stdout, skipping log lines before it. | The AppImage wrapper prints startup log lines, some multi-line, before the JSON. |

### Environment facts

- Installed npm CLI `@getpaseo/cli` 0.10.3 against a Paseo 0.11.2 daemon; `herding.paseo.command` points at it.
- `paseo workspace create --path <worktree> --isolation local --project prj_602f0e2c856da05a --json` printed `{"workspaceId":"wks_69fcacfc975748a7","project":"beehive","name":"wt/paseo-worktree-workspace","isolation":"worktree","cwd":"/mnt/data/Projects/goglbe/beehive--wt--paseo-worktree-workspace"}` — Paseo labels a git worktree path `worktree` even when asked for `local`.
- `paseo project ls --json` rows carry `projectId`, `name`, `kind`, `path`.
- `paseo run --help`: `--workspace <id>  Run in an existing workspace (defaults to the caller workspace when agent-scoped)`.
- `paseo workspace archive <workspace-id> [--json]` exists.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Inside Paseo | `PASEO_AGENT_ID` or `PASEO_CLI` is set and not empty (D1). |
| Paseo workspace | A Paseo `wks_…` record that groups agents under one directory. |
| bee workspace record | `.bee/runtime/workspaces/<worktree id>.json` in the main checkout. |

## Out of scope

- Paseo-owned worktrees (`--isolation worktree` with a Paseo-made checkout): bee keeps making its own worktrees.
- Herdr and tmux transports.
- Back-filling workspaces for worktrees made before this feature, except through D4's ensure step when a run starts in one.
