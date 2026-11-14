# skillstation-publish-skip — context

`scripts/install.sh --source . -d . -y --skillstation` on beehive (2026-11-14)
planned 25 `publish_skillstation` items although all 24 `bee-principle-*`
skills and `bee-deploying` were already active on the hub with the same files.
Each publish ran `skill commit` and `skill activate`, about 25 s to 1 min per
skill. The owner wants a publish only for a skill that is missing, not active,
or whose staged content differs from the hub copy.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | the `--skillstation` switch, the `after_publish` contract and the hub read |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | `publish_skillstation_items`, `publish_principle_items` and the `opts.skillstation` branch |
| `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` | `hub_runner`, `hub_show` and the `Hub` read |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | the `after_publish` gate on a failed principle publish |
| `packages/bee-rs/crates/bee/src/onboard/render.rs` | `walk_skill_tree` and the per-file sha256 of a rendered skill |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | `bee onboard --skillstation` plans a `publish_skillstation` item for a skill only when the hub has no such skill, the hub copy is not `active`, or its files differ from the staged content. Same content means the set of (path relative to the skill folder, sha256 of the file bytes) of the engine skill rendered for claude equals the set read from `skillstation skill show <id> --json` `manifest.resources` (path after the `<id>/` segment, digest without the `sha256:` prefix). This holds for `bee-deploying` and for every principle. | Owner, 2026-11-14. The hub digest is the raw sha256 of each file, so a set compare needs no download and no new hub verb. The engine render is what the same run syncs and stages. |
| D2 | A hub read that skips (no binary, non-zero exit, unreadable JSON, no manifest) keeps the publish item, so apply reports the failure as today. | Never hide a hub that cannot be reached. |
| D3 | skillstation-principles D9 stays: the `after_publish` removals and `set_principles_hub` run only when every planned publish succeeded, and zero planned publish items counts as success. A re-run of the switch on a hub that already serves everything plans only the config, the agent files and the local removals. | Owner keeps the D9 contract; apply already gates on a failed principle only. |
