# skillstation-publish-skip — delivery

## What changed

- `Hub` in `verbs/skillstation_group.rs` carries `files`: (path after the `<id>/` segment, hex digest without `sha256:`) parsed from `manifest.resources`; absent resources give an empty set.
- `onboard/plan.rs` gains `hub_needs_publish`: it reads the skill through `hub_show`, and publishes when the read skips or the skill is missing, the status is not active, or the sorted file set of the engine skill (`walk_skill_tree` with the claude render) differs from the hub set.
- `publish_skillstation_items` (bee-deploying) and `publish_principle_items` filter through it. `apply.rs` and the `after_publish` ordering are untouched, so the removals and `set_principles_hub` stay planned with zero publishes.
- The concept `skillstation-hub-skills-and-aliases.md` states the filter under Business Rules.

## Tests (in `onboard/tests.rs`, module `principles_on_the_hub`)

- `publish_skip_plans_no_publish_for_skills_served_with_the_same_files`
- `publish_skip_keeps_only_the_principle_whose_digest_differs`
- `publish_skip_keeps_a_skill_with_an_extra_or_missing_hub_file`
- `publish_skip_keeps_a_skill_missing_from_the_hub`
- `publish_skip_keeps_a_draft_skill`
- `publish_skip_keeps_every_item_when_the_hub_read_fails`

## Proof

Red before the fix: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- publish_skip` gave `1 passed; 5 failed` (the hub-failure case already held). The same-files case planned `publish_skillstation` for bee-deploying and both principles.

Green after the fix: `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` gave `5118 passed; 0 failed` in the bee unit target and ok in every other target.
