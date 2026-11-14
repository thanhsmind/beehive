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
