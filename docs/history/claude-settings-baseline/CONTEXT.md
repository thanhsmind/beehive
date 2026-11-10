# Claude settings baseline at install — Context

**Feature slug:** claude-settings-baseline
**Date:** 2026-11-10
**Shaping session:** complete (the owner gave the exact settings block in chat)
**Scope:** High-risk (it sets the permission mode of every host project)
**Domain types:** RUN | ORGANIZE

## Feature Boundary

When a project installs or onboards bee, its `.claude/settings.json` gains the
owner's Claude settings baseline: bypass-permissions mode, the worktree tools
allowed, thirteen tools denied, and six top-level switches. The write only
adds what is missing and never changes a value the project already has. A
`--no-claude-settings` flag skips the baseline and keeps today's worktree-tool
allow. It ends at the project's own settings file: `~/.claude/settings.json`
is never touched.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Onboarding projects the baseline of decision bd6ebab1 into the host's `.claude/settings.json`; both installers get it through `bee onboard`. | One Rust projection serves `install.sh` and `install.ps1`. |
| D2 | Add-only: list names appended when missing, a scalar key written only when absent, a present value never overwritten, an unparsable or odd-shaped file left unchanged, re-runs converge. (bd6ebab1, extends 5fd24d1b) | The installer promises never to overwrite a present settings entry. |
| D3 | On by default; `bee onboard --no-claude-settings` (passed through by both installers) skips everything except the existing EnterWorktree/ExitWorktree allow. (bd6ebab1) | |
| D4 | The contract of decision 282a650f is what the tests check. | |
| D5 | The opt-out is durable: `.bee/config.json` `"claude_settings": false`, read like `"statusline": false` and never written by onboarding, has the same effect as the flag. (118c4686) | A bare re-run must not re-add a declined baseline. |
| D6 | Every run that plans or applies the full baseline emits one notice: permission prompts off, tools denied, keys kept at the project's own value, baseline denies the project's allow list already carried, and how to opt out. (a9daf9d2) | install.sh shows only notices. |
| D7 | Named guess: the baseline is written whether or not bee's guard hooks are wired; the Safety text and the notice say so. (9f22ec8d) | The owner asked for the block without a guard condition. |

### Agent's Discretion

- The flag name `--no-claude-settings` and the plan item name `merge_claude_settings`.

### Known costs the owner accepted

- `bypassPermissions` removes Claude's permission prompts in host sessions; bee's own guards stay the safety net, except on an install run with `--no-hooks` (D7).
- Denying `AskUserQuestion` makes gate questions plain chat text in hosts; this repo already runs that way.
- Denying `SendMessage` removes the leader's way to steer a Claude worker (LEADER.md); this repo already runs that way.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | The worktree-tool allow projection (5fd24d1b) this feature widens. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs` | `WORKTREE_TOOL_ALLOW`, `worktree_tool_allow_missing`, `merge_worktree_tool_allow`. |
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | The `merge_worktree_tool_allow` plan item. |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | The apply arm for that item. |
| `packages/bee-rs/crates/bee/src/onboard/mod.rs` | The onboard flags. |
| `scripts/install.sh`, `scripts/install.ps1` | Flag pass-through to `bee onboard`. |

<!-- bee:not-a-deferral: template section names with no open question under them, not a promise to act later -->
## Outstanding Questions

### Resolve Before Planning

- (none)

### Deferred To Planning

- (none)
<!-- /bee:not-a-deferral -->
