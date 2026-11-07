# Project-aware workspace protocol init — Context

**Feature slug:** protocol-init-from-project
**Date:** 2026-11-07
**Shaping session:** complete (gate bypass full; owner chose "cách 1": binary facts plus an agent judgment step)
**Scope:** Small
**Domain types:** CALL | READ

## Feature Boundary

`bee protocol init` writes one fixed template today (workspace-protocol D4).
The owner wants the file written from each project. The binary fills the
facts it can read for sure. The agent then reads the project's own rule files
and adds what only judgment can find: the project's own human decision
boundaries, protected areas, anti-patterns and criticality. The hand-written
beehive file (`WORKSPACE_PROTOCOL.md`, commit fbf212d65) is the reference
result.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `bee protocol init` renders from the control root: `PROJECT_ID` = the root directory name; `DEFAULT_BRANCH` = the short name of `refs/remotes/origin/HEAD` without `origin/`, else the current branch, else the key is left out; `REPOSITORY_REMOTE` = the `origin` fetch URL with any `user[:pass]@` part removed, left out when there is no origin; `PROJECT_CRITICALITY: medium`. | Facts git holds; no guess. |
| D2 | `MERGE_OWNER` from `.bee/config.json` `uat_stop`: `"merge"` gives `human approves uat, then the agent runs bee worktree merge`; any other value or absent gives `agent, through bee worktree merge after a green bee test; the owner accepts at the uat door`. | The config already decides who merges. |
| D3 | `DEPLOY_OWNER`: the first release entry found, in this order: `scripts/release.sh`, `scripts/deploy.sh`, `docs/DEPLOY.md`, then a `.github/workflows/` file whose name holds `release` or `deploy` (path-sorted). Found gives `human; the agent runs <path> only when the owner asks`. None gives `human`. | |
| D4 | `HUMAN_DECISION_BOUNDARIES` starts from bee's base set: a release, deploy, tag or push to a shared remote; the uat acceptance of a standard or high-risk feature; a change to gate_bypass or another gate rule; reading or printing a secret-shaped file; a P1 review finding; a change that removes or weakens a guard or a pinned test. `PROTECTED_AREAS` always holds `.bee/*.json and .bee/*.jsonl change only through the bee CLI`, plus one item `<dir> is vendored by bee; never hand-edit it` for each of `.claude/skills`, `.agents/skills`, `.codex/skills`, `.opencode/skills`, `.claude-plugin/skills`, `.codex-plugin/skills` that exists. The rendered file holds no placeholder and checks `valid`. It carries no test command (workspace-protocol D3). | A valid starting point the agent extends. |
| D5 | The prose above the fenced block gains a `## Sources` list: one line per derived key naming where it came from (git, `.bee/config.json uat_stop`, the release entry path, bee's base set, the vendored tree paths). `--json` output gains `facts: [{key, value, source}]`. | The owner sees why each line is there. |
| D6 | Init names the judgment step. `--json` gains `review: [<path>]`: those of `LEADER.md`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `docs/DEPLOY.md` and the `.github/workflows/*` files that exist. The text output ends with one next line: read those files, add the project's own boundaries, protected areas and anti-patterns, set PROJECT_CRITICALITY, never add a test command, then run `bee protocol check`. | The binary cannot read prose; the agent can. |
| D7 | New flag `bee protocol init --print`: render to stdout and write nothing, also when a file exists. Without `--print`, an existing file is still the `protocol_exists` refusal. | Lets the agent compare a fresh render with an existing file without an overwrite path. |
| D8 | The agent procedure lives in bee-hive `references/onboarding.md`, section "Workspace protocol": when to run it (the owner asks, or `bee protocol check` is not valid), run init, read every `review` path, edit the file (cite the source of each added line in `## Sources`; no placeholder; no test command), run `bee protocol check` to valid, commit, show the owner the table of keys and sources. `skills/bee-hive/SKILL.md` Onboarding gains one pointer line. The workspace-protocol concept records the change. | Same pattern as the orientation entry: written by reading the tree, never by asking the owner. |
