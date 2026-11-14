# skillstation-onboard-aliases — context

`bee skillstation install --apply` (feature `skillstation-aliases`) writes
`/<id>` aliases for hub skills, but nothing runs it when bee is installed in a
project: `scripts/install.sh` never mentions skillstation, and `bee onboard
--skillstation` only publishes `bee-deploying` to the hub. The owner chose
(option A, this session): every install and onboard writes the aliases, the
hub changes only behind a flag, then a release ships it.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | the verb group, alias shape and ownership marker this feature reuses |
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | the onboard `--skillstation` publish and its loud-skip contract |
| docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md | suspect | how `scripts/install.sh` runs onboard and rechecks it |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` | `RELATED_SKILLS`, `render_alias`, `write_alias`, `install_row` |
| `packages/bee-rs/crates/bee/src/onboard/plan.rs` | plan items, `publish_skillstation_items`, the `opts.skillstation` branch |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | the item arms and the `skipped_skills` loud skip |
| `scripts/install.sh` | `ONBOARD_FLAGS`, the onboard plan, apply and recheck calls |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | `bee onboard` plans one `skillstation_alias` item per `RELATED_SKILLS` row that is active on the hub, has `disable-model-invocation: true`, and whose `.claude/skills/<id>/SKILL.md` is missing or is a bee alias whose bytes differ from the fresh render. It runs by default, with no flag, when the Claude runtime is in scope. | Every install and onboard gives the project `/<id>` with no extra step. Planning only a missing or changed alias keeps the post-apply recheck `up_to_date`. |
| D2 | A `.claude/skills/<id>/` that is not a bee alias (no `skillstation_alias: <id>` marker) gets no item and one `skipped_skills` entry naming the folder. Onboard never passes `--force`. | Never overwrite a user's skill (skillstation-aliases D6). |
| D3 | No `skillstation` binary, a non-zero exit or unreadable JSON adds no item and one `skipped_skills` entry `{target: "skillstation-alias", reason}` for the whole check, not one per skill; the onboard still reports its normal result. | Most machines have no skillstation; one line is enough. |
| D4 | With `--skillstation`, onboard also adds each missing `RELATED_SKILLS` row to the hub as a draft (`skillstation skill add … --yes --json`), beside the existing `bee-deploying` publish. Without the flag onboard never changes the hub. | The hub is shared and outward; the owner keeps a switch. |
| D5 | `scripts/install.sh` gains `--skillstation`, passed through to `bee onboard`. Without it, install writes only the local aliases from D1. | Same switch on both entry points. |
| D6 | Onboard reuses the alias render, ownership check and hub read of `skillstation_group.rs`; no second copy. Onboard tests stay deterministic: the hub runner is injected and the default test runner reports skillstation as absent. | Single source of truth; a dev machine with skillstation on PATH must not change onboard test results. |
| D7 | Claude Code only, as in skillstation-aliases D8. | Same surface. |
| D8 | After merge, release with `scripts/release.sh` at the next minor version. | The owner chose release as part of option A. |
