# skillstation-human-skills — context

Request (verbatim): "Làm cả 5 nhớ là teong install.sh luôn để triển khai
nhiều project". The five are `bee-how`, `bee-teach`,
`bee-technical-writing`, `bee-why` and `bee-writing-skills`. The owner
wants the Skillstation hub to serve them like the `bee-principle-*`
skills, because a human starts most of them, and wants
`scripts/install.sh --skillstation` to roll the change out to every
project.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | the `--skillstation` switch, hub mode, `after_publish`, the alias marker |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | `principle_ids`, `principles_served_by_hub`, `principle_removal_items`, `publish_principle_items`, `skillstation_items` |
| `packages/bee-rs/crates/bee/src/onboard/skills.rs` | `SkillSources::entries` (the `without_principles` filter) and the `/^bee-/` deletion domain |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | the principle trigger flags on `skill commit`, `failed_principle`, the `skillstation_alias` arm |
| `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` | `render_alias`, `carries_alias_marker` |
| `scripts/install.sh` | `--skillstation` is already forwarded to onboard through `ONBOARD_FLAGS` |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | One constant lists the five skills: `bee-how`, `bee-teach`, `bee-technical-writing`, `bee-why`, `bee-writing-skills`. They join the `bee-principle-*` skills as one hub-served set and ride the same hub mode (`skillstation.principles: hub`) and the same `--skillstation` switch. No new config key and no new flag. | Owner, 2026-11-14. One switch keeps the rollout one command: `install.sh --skillstation` already forwards the flag. |
| D2 | Every principle rule applies to the five unchanged: publish from the engine source rendered for claude with the trigger flags, skip a publish when the hub copy is active with the same files (skillstation-publish-skip D1), drop the local `.claude/skills` copy only when every hub-served skill is active on the hub, and gate the `after_publish` removals and the config write on every hub-served publish succeeding. | The principle path is proven on the real hub; a second path would drift. |
| D3 | In place of each removed local copy of the five, onboard writes a `/<id>` alias (`render_alias`, with the alias marker) at `.claude/skills/<id>/SKILL.md`. Its description is the engine skill's own description, so the agent still sees it in the skill list and a human can still type `/<id>`. Principles get no alias. In the switch run the alias items are `after_publish` and come after the removals. | A human starts these skills by name; without the alias `/bee-how` stops working. |
| D4 | The repo-claude deletion domain leaves alone a `bee-*` directory whose `SKILL.md` carries the alias marker for its own name while that name is hub-served and hub mode drops it from the sources. | Without this, the next onboard deletes the alias, the one after writes it again, and the installer recheck never reaches `up_to_date`. |
| D5 | Codex, opencode, the plugin trees and the legacy global targets keep their local copies of the five (skillstation-principles D1). | Only Claude Code reads the hub. |

## Out of scope

- A release (the owner did not ask).
- Any change to which skills `bee skillstation install` adds from outside repos.
