# Workspace Protocol — beehive

The project owner owns this file. The leader reads it in full before it
routes work. Workers never read it: the leader copies the lines a cell
touches into that cell's brief. `bee protocol check` reports its state.

Test commands are not here. They live in `.bee/config.json`
`commands.test`, and `bee test` runs them.

```text
WORKSPACE_PROTOCOL_VERSION: 1

PROJECT_ID: beehive
PROJECT_CRITICALITY: high
DEFAULT_BRANCH: main
REPOSITORY_REMOTE: https://github.com/thanhsmind/beehive.git

MERGE_OWNER: agent, through bee worktree merge after a green bee test; the owner accepts at the uat door
DEPLOY_OWNER: human; the agent runs scripts/release.sh only when the owner asks for a release

HUMAN_DECISION_BOUNDARIES:
- a release, a tag or a push to origin
- the uat acceptance of a standard or high-risk feature
- a change to gate_bypass or any other gate rule
- reading or printing a secret-shaped file
- a P1 review finding
- a change that removes or weakens a guard, a door or a pinned test
- a public issue or comment that names a real person

PROTECTED_AREAS:
- .bee/*.json and .bee/*.jsonl change only through the bee CLI
- the vendored skill trees (.claude/skills, .agents/skills, .claude-plugin/skills, .codex-plugin/skills, .opencode/skills) change only through bee dev regen from skills/
- .claude-plugin/plugin.json and .codex-plugin/plugin.json change only through scripts/release.sh
- docs/history/codex-harness-hardening/release-manifest.json changes only through bee dev release-manifest --write
- docs/knowledge/index.md and the area index files change only through bee knowledge index
- packages/bee-rs/crates/bee/src/generated/registry_payload.json is hand-edited, and tests/registry_contracts.rs pins its shape

FEEDBACK_RULE: owner feedback about how agents work is a change to bee itself; it goes through the bee chain and a release, never into private memory alone

ANTI_PATTERNS:
- running the full test suite per cell; it runs once per feature through bee test
- writing code comments; the why goes to docs/knowledge or bee decisions log
- hand-walking the release steps instead of scripts/release.sh
- writing through a reservation or hold conflict
```
