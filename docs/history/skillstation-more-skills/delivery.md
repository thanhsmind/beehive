# Delivery: skillstation-more-skills

## sms-1: nine more engine skills from the hub

### What changed

- `onboard/skills.rs`: `HUB_SERVED_SKILLS` grows from five to fourteen with `bee-unslop`, `bee-researching`, `bee-knowledge-setup`, `bee-knowledge-upkeep`, `bee-verifying`, `bee-verify-upkeep`, `bee-wayfinding`, `bee-grooming` and `bee-deploying`. New `frontmatter_disables_model` reads `disable-model-invocation` from rendered frontmatter and is true only for the literal `true`.
- `onboard/plan.rs`: `hub_alias_items` passes that value to `render_alias`. `publish_skillstation_items` skips a `HUB_SKILLS` id that the hub-served publish of the same run covers (hub mode on), and skips one whose engine folder is absent. A `--skillstation` run without hub mode keeps its `HUB_SKILLS` publish.
- `scripts/install.sh` help, `README.md` and the onboarding concept name the larger set, the alias keeping `disable-model-invocation` and the single `bee-deploying` publish.
- `docs/history/codex-harness-hardening/release-manifest.json` regenerated, because `install.sh` is a release-manifest root.

### Existing tests changed

`bee-deploying` is now hub-served, so under the switch it has no local Claude copy, gets a removal and an alias, and its commit carries routing flags:

- `a_principle_commit_carries_the_routing_flags_and_bee_deploying_carries_none` is renamed `..._and_so_does_bee_deploying` and asserts `--not-for` and `--min-scope` on the `bee-deploying` commit.
- `publish_skip_plans_no_publish_for_skills_served_with_the_same_files` expects the removals `bee-deploying` plus the two principles.
- `publish_skip_keeps_every_item_when_the_hub_read_fails` expects two more `after_publish` items (the `bee-deploying` removal and alias).

### Tests

Red first (before the fix), new tests in `onboard/tests.rs`:

- `the_switch_plan_publishes_removes_and_aliases_the_added_engine_skills` failed (no `remove_skill` for the added skills).
- `an_alias_keeps_disable_model_invocation_only_for_a_skill_that_sets_it` failed (no alias planned for `bee-verifying`).
- `a_full_switch_aliases_the_added_skills_and_the_hub_replan_plans_nothing_for_them` failed (local copies stayed).
- `the_switch_plan_publishes_bee_deploying_exactly_once` and `a_run_without_the_claude_runtime_still_publishes_bee_deploying_once_and_plans_no_alias` passed before and after as guards.

Green after the fix: all five, with the rest of `-- skillstation onboard` (352 passed). `design_offer_flag_is_listed_in_onboard_help` needs a built release `bee` binary, so run `cargo build --release --bin bee` first.

### Proof

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard` and `bee dev release-manifest --check`.

## sms-2 — Serve the design core and the picked design packs from the hub

### What changed

- `onboard/plan.rs`: `principle_ids` returns each hub-served id with its source root: the engine skills root for engine skills, and `engine.design_root` for `bee-design` and each picked pack. Publish items, removals, aliases and the hub readiness check all follow it, and the publish item carries that root as `source`.
- `onboard/skills.rs`: new `design_picked` is the one pick rule behind `SkillSources::picks`. In hub mode `SkillSources::entries` drops the picked design skills as it drops engine ones, and the deletion-domain alias exception covers them. `compute_design_sync` takes the hub flag and applies it to the repo-claude target only.
- `onboard/apply.rs`: a publish item that carries a `source` root counts as hub-served for the routing flags on `skill commit` and for the failed-publish gate. The failed-design-publish test proved this was needed: without it the design failure did not stop the `after_publish` removals.
- `README.md` and the onboarding concept name the design core and picked packs as hub-served (skillstation-more-skills D4-D6) and state that sibling-pack links stay as written.

### Tests

Red first, new tests in `onboard/tests.rs` (`principles_on_the_hub`), with a design root of `bee-design`, `bee-design-web`, `bee-design-charts` and `bee-design-motion` and packs `web,charts`:

- `the_switch_plan_publishes_removes_and_aliases_the_design_core_and_picked_packs` failed (no publish for `bee-design`).
- `a_full_switch_aliases_the_design_skills_and_the_hub_replan_plans_nothing_for_them` failed (the local copy stayed a full skill).
- `a_failed_design_publish_keeps_every_local_copy_and_writes_no_alias` failed (the principles lost their local copies).
- `hub_mode_without_a_pack_pick_plans_no_design_hub_item` and `codex_and_agents_targets_keep_the_design_skills_through_the_switch` passed before and after as guards.

The failed-publish test failed a second time after the plan fix (the design alias was written) until the `apply.rs` change.

### Proof

`cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`.
