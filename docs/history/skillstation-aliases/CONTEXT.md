# skillstation-aliases — context

Skillstation serves hub skills to agents over MCP (`skill_resolve`,
`skill_get`). A hub skill is not in the host's own skill list, so the user
cannot start it with `/<id>`. A skill that agents may not trigger on their own
(`disable-model-invocation: true`) is then unreachable. `skillstation skill
checkout` writes a `.claude/skills/<id>/` pointer, but it copies the whole skill
for editing and is not an alias. bee also relies on three hub skills
(`statechart`, `create-formal-verification-skill`,
`maintain-formal-verification-skill`) and has no way to check or add them.
Source: PBI p-d7191e92 and the owner's request in this session.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | the onboard `--skillstation` publish and its loud-skip contract |
| docs/knowledge/areas/rust-runtime/command-surface.md | suspect | how a verb enters the registry |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | `run_skillstation`, the reusable CLI runner |
| `packages/bee-rs/crates/bee/src/verbs/tmp_group.rs` | the verb-group wiring pattern |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | Add a verb group `bee skillstation` with two verbs: `install [--apply] [--json]` and `alias <id> [--force] [--json]`. | One home for the hub link. `install` touches the hub, so it previews first like `bee onboard`. `alias` is an explicit local write. |
| D2 | One Rust constant lists the hub skills bee relies on: id, GitHub repository and path. The first rows are `statechart`, `create-formal-verification-skill` and `maintain-formal-verification-skill`, all from `https://github.com/cursor/plugins` under `pstack/skills/<id>`. `bee-deploying` stays with `bee onboard --skillstation` (`HUB_SKILLS`). | Single source of truth for the list. The upstreams are the ones the hub already tracks (`skillstation skill upstream <id>`). |
| D3 | `install` runs `skillstation skill show <id> --json` for each row. A missing row is added with `skillstation skill add <repo> --path <path> --id <id> --yes` only under `--apply`. The report names `skillstation skill review <id>` as the owner's next step for each added skill. | `--yes` never approves third-party content; content approval and activation stay with the hub owner. bee never writes a hub file itself. |
| D4 | `install --apply` writes an alias for every listed skill that is active on the hub and whose SKILL.md frontmatter has `disable-model-invocation: true`. `alias <id>` writes one alias for any active hub skill. | These are the skills that only `/<id>` can start. The frontmatter comes from the `content` field of `skillstation skill show --json`. |
| D5 | An alias is `.claude/skills/<id>/SKILL.md` only: frontmatter `name: <id>`, the hub description on one line, `disable-model-invocation: true` when the hub skill has it, and `metadata.skillstation_alias: <id>`. The body tells the agent to call the skillstation MCP tool `skill_get` with `skill_id: <id>` and follow the returned content, and to tell the user to run `skillstation connect -g` when the tool is missing. | A few lines cost few tokens; the hub stays the one copy of the skill. |
| D6 | bee overwrites a `.claude/skills/<id>/` folder only when its SKILL.md carries `metadata.skillstation_alias: <id>`. Any other existing folder is a refusal that names the folder; `alias --force` overrides it. | Never destroy a user's own skill or a bee-rendered skill. |
| D7 | No `skillstation` binary, a non-zero exit or unreadable JSON is a loud skip: one row per skill with the reason, exit 0. `alias <id>` for a skill that is not active on the hub exits non-zero with the reason. | Same contract as the onboard publish skip. An explicit alias request that cannot work must say so. |
| D8 | Claude Code only in this change. | `/<id>` is the Claude Code surface the owner named. Other runtimes are a later row. |
| D9 | Docs: one knowledge concept under `docs/knowledge/areas/onboarding/` and a short "Skillstation" section in `README.md`. | The owner asked for a Skillstation section in beehive. |
