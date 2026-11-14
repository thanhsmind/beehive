# skillstation-principles — context

The owner asked which bee skills fit Skillstation. The 25 `bee-principle-*`
skills fit best: `bee orient` names them by route class, so no agent has to
choose them from a description, and each is one file with no references or
scripts. Their descriptions cost about 4.4 KB of context in every session.
This feature lets a project serve them from the Skillstation hub to Claude
Code, behind one config key.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | the hub read, alias render and ownership marker this feature reuses |
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | the onboard `--skillstation` publish and its loud-skip contract |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | "Principle homes": how orient names principles and the `bee-principle-` prefix law |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` | `hub_runner`, `hub_show`, the injected test hub |
| `packages/bee-rs/crates/bee/src/onboard/skills.rs` | `SkillSources`, `compute_skill_items`, the bee-* deletion domain, `compute_skill_sync` |
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | the `--skillstation` publish branch and skip rows |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | `apply_publish_skillstation`, the commit-then-activate path |
| `packages/bee-rs/crates/bee/src/onboard/agents.rs` | `render_claude_agent_file`, `compute_agent_file_plan` |
| `packages/bee-rs/crates/bee/src/principles.rs` | `principle_lines`, the header line orient and the preamble print |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | Config key `skillstation.principles` takes `local` (the default, also when absent) or `hub`. `hub` means the Claude runtime gets the `bee-principle-*` skills from the hub through the MCP tool `skill_get`, not from `.claude/skills`. Codex, opencode and the plugin trees keep their local principle copies. | Principles are named by `bee orient`, never chosen from a description, so a fetch by id fits them. Alias support is Claude Code only (skillstation-aliases D8). |
| D2 | In hub mode, `bee onboard` checks each `bee-principle-*` source skill on the hub. When every one is active, the Claude target sync leaves the principles out of its source list, so the existing deletion domain removes the local copies. When one is missing or draft, or the hub check skips, the local principles stay and onboard adds one `skipped_skills` row `{target: "skillstation-principles", reason}`. Onboard tests use the injected hub runner. | A project never ends with no principles; the fallback is the old behavior, and one row names why. |
| D3 | `bee onboard --skillstation` in hub mode publishes every `bee-principle-*` skill to the hub through the commit-then-activate path that publishes `bee-deploying`, staged from the engine skill source rendered for the claude runtime, never from `.claude/skills`. Without `--skillstation`, onboard never changes the hub. The first hub-mode onboard publishes and keeps the local copies; the next onboard removes them. | Principles are first-party bee content like `bee-deploying`. In hub mode the local folder is gone, so the source must be the engine tree. |
| D4 | In hub mode, the rendered Claude agent files `.claude/agents/bee-*.md` add `mcp__skillstation__skill_get` to their `tools:` line. In local mode every rendered agent file stays byte-identical. | Every agent template lists only built-in tools, so a worker could not fetch a hub principle. |
| D5 | In hub mode, the principle header line that `bee orient` and the session preamble print tells the agent to load each named principle with the skillstation MCP tool `skill_get`, slug as `skill_id`. The principle rows stay unchanged; local mode output stays byte-identical. One helper in `skillstation_group.rs` reads the mode for onboard, agents and orient. | The agent must know where the body lives once the local skill is gone. One helper keeps one home for the mode read. |
| D7 | Onboard drops the local principles in hub mode only when the repo holds the principle index file that `bee orient` reads (`principles.rs` `PRINCIPLE_INDEX`). Without it, the local principles stay and onboard adds one `skipped_skills` row with target `skillstation-principles` naming the missing index. The hub publish under `--skillstation` does not change. | Host repos carry no principle index, so orient never names a principle there. Removing the local copies would leave the agent no way to find them. Found in the live check after the first merge. |
| D8 | The bee binary carries the principle index. `principle_lines` reads the repo's own index file when it exists, else a copy compiled into the binary. So `bee orient` names principles in every project, and the D7 guard goes away. Supersedes D7. | The owner wants `install.sh --skillstation` to remove local principles in every project. That is safe only when orient names the principles there. |
| D9 | `bee onboard --skillstation` turns on hub mode and finishes the switch in one apply: it writes `skillstation.principles: "hub"` into the tracked `.bee/config.json`, publishes every principle, then removes the `.claude/skills` principle folders in the same run. The plan marks those removals `after_publish`; apply runs them only when every principle publish succeeded, else keeps the local copies and reports one `skipped_skills` row. | Owner: install with `--skillstation` deploys through Skillstation, and a project that already has bee drops its local skills. |
| D10 | `scripts/install.sh --skillstation` passes the flag to the onboard plan and apply. The post-apply recheck runs without it, so it checks local state. Without the flag, install does not change the mode. | Owner: without the flag install behaves as today. A recheck that carries `--skillstation` always plans a publish and never reports `up_to_date`. |
| D6 | No plan-step hat wave: the scope is clear and not high-risk. The bee repo's own dev skill trees and the release are out of scope. | Hat-wave threshold: a clear ask keeps the fast path. |
