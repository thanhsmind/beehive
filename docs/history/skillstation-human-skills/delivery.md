# Delivery: skillstation-human-skills

## What changed

- `onboard/skills.rs`: `HUB_SERVED_SKILLS` (the five) and `is_hub_served` (the principle prefix or one of the five) are the one hub-set test. `SkillSources::entries` uses it where `without_principles` filters. The repo-claude deletion domain skips a hub-served `bee-*` directory whose `SKILL.md` carries its own alias marker (`carries_alias_marker`) while hub mode drops it from the sources. `frontmatter_description` reads a description from frontmatter, including folded `>-` blocks.
- `onboard/plan.rs`: `principle_ids` returns every hub-served id in the engine skills root, so publish, removal and the hub-active check cover the five. `hub_alias_items` renders each alias with `render_alias` (engine description rendered for claude, model invocation left on) and skips one whose file already matches. It is planned `after_publish` after the removals in the switch, and without the flag when the repo-claude target drops the skills in hub mode.
- `onboard/apply.rs`: the trigger flags on `skill commit` and `failed_principle` use `is_hub_served`. The routing flags read the description through `frontmatter_description`, because the real five use folded descriptions.
- `scripts/install.sh` help, `README.md` and the onboarding concept name the five and their aliases.

## Tests

Red first (before the fix):

- `the_switch_plan_publishes_removes_and_aliases_bee_how_in_that_order` failed (no remove_skill for bee-how: left 0, right 1).
- `a_full_switch_leaves_bee_how_as_an_alias_and_the_replan_plans_nothing_for_it` failed (local copy still in place).
- `a_failed_bee_how_publish_keeps_every_local_copy_and_writes_no_alias` failed (principle copy removed).
- `hub_mode_with_an_unreadable_hub_keeps_bee_how_and_plans_no_alias` and `the_switch_keeps_bee_how_for_the_agents_target` passed before and after as guards.

Green after the fix: all five, with the rest of `-- skillstation onboard`.

## shs-2: second switch apply keeps the alias

- Red: `a_second_switch_apply_keeps_the_bee_how_alias_and_the_hub_replan_plans_nothing_for_it` failed with the alias file gone after the second `--skillstation` apply.
- Green after the `hub_alias_items` fix in `plan.rs` (switch path always plans the alias item): the same test, with the rest of `-- skillstation onboard`.

## Proof

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard` and `bee dev release-manifest --check`.
