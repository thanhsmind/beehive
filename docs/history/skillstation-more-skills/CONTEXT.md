# skillstation-more-skills — context

Request (verbatim): "Làm a và B". Group A and group B come from the
evaluation given the same day. Group A: `bee-unslop`, `bee-researching`,
`bee-knowledge-setup`, `bee-knowledge-upkeep`, `bee-verifying`,
`bee-verify-upkeep`, `product-description`. Group B: `bee-design`,
`bee-design-charts`, `bee-design-web` (with the rest of the picked design
packs), `bee-wayfinding`, `bee-grooming`, `bee-deploying`. The owner wants the
Skillstation hub to serve them to Claude Code like the five human-started
skills of skillstation-human-skills, rolled out through
`scripts/install.sh --skillstation`.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | the hub-served set, the `--skillstation` switch, `after_publish`, the alias marker |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/skills.rs` | `HUB_SERVED_SKILLS`, `is_hub_served`, `SkillSources::entries`, the alias exception in the deletion domain, `compute_design_sync` |
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | `HUB_SKILLS`, `principle_ids`, `hub_alias_items`, `publish_principle_items`, `principle_removal_items`, `principles_served_by_hub` |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | the trigger flags on `skill commit` for hub-served skills, the `after_publish` gate |
| `packages/bee-rs/crates/bee/src/doctor.rs` | `design_skill_row` checks that `.claude/skills/bee-design` is a directory |
| `packages/bee/design/` | the design core and its packs; four pack files point at a sibling pack with `../bee-design-<pack>/` |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | `HUB_SERVED_SKILLS` grows by eight engine skills: `bee-unslop`, `bee-researching`, `bee-knowledge-setup`, `bee-knowledge-upkeep`, `bee-verifying`, `bee-verify-upkeep`, `bee-wayfinding`, `bee-grooming`. Every rule of skillstation-human-skills D2-D5 applies to them unchanged. | Owner, 2026-11-14. One set, one switch, one rollout command. |
| D2 | The alias of a hub-served skill copies the engine skill's `disable-model-invocation` value. `bee-knowledge-setup`, `bee-knowledge-upkeep`, `bee-verifying` and `bee-verify-upkeep` keep it on, so the agent never starts them on its own and a human still types `/<id>`. | Hub serving must not change who may start a skill. |
| D3 | `bee-deploying` joins the hub-served set, so in hub mode its Claude copy becomes an alias like the others. The separate `HUB_SKILLS` publish stays only for a `--skillstation` run whose Claude runtime is not in scope, so no run publishes `bee-deploying` twice. | One path for every Claude hub skill; a Codex-only project keeps today's publish. |
| D4 | The design core `bee-design` and each design pack the project picked (`design.packs`) are hub-served. They publish from the design root rendered for claude, and their removals and aliases follow the same rules as D1. A pack the project did not pick is neither published nor aliased. | Design is the largest copy in every project; per-project picks stay per project. |
| D5 | Pack files that point at a sibling pack with `../bee-design-<pack>/` stay as they are. Every such link targets a pack the reader may not have locally today too, so hub serving adds no new broken link for a picked pack. | No content rewrite: the hub copy stays byte-equal to the engine render, so the publish skip keeps working. |
| D6 | `bee doctor`'s `design_skill` row stays as it is: an alias folder `.claude/skills/bee-design/` is a directory, so the row stays ok. | Existing behavior already covers the alias. |
| D7 | `product-description` is not a skill bee ships, so it gets no code. The leader adds it to the hub once from `.claude/skills/product-description` and writes its `/<id>` alias with `bee skillstation alias product-description` when the hub owner has made it active. | It lives only in this repo; onboard never manages it. |

## Out of scope

- A release (the owner did not ask).
- Moving `bee-hive`, `bee-swarming`, `bee-planning`, `bee-shaping`, `bee-capturing`, `bee-reviewing`, `bee-herding`, `bee-herdr`, `bee-evolving`, `verify-app` or `formal-cell-lifecycle`.
