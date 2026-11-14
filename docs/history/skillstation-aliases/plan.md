---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: skillstation-aliases

## Summary

A new command, `bee skillstation`, links a project to the Skillstation hub.
`bee skillstation install` checks that the hub has the skills bee relies on,
adds the missing ones with `--apply`, and writes a small alias skill for each
hub skill that only the user may start. `bee skillstation alias <id>` writes one
alias for any active hub skill. After that, `/<id>` works in Claude Code. The
README gets a Skillstation section.

Mode: `standard` — 2 risk flags: external systems (the `skillstation` CLI), public contracts (a new verb)
Why this is the least workflow that protects the work: one code cell and one writer cell, with the session running the plan check inline (≤5 product files, no hard-gate flag).

Playbook: `skills/bee-planning/playbooks/feature.md` (class `feature`).

## Requirements (from CONTEXT.md)

- D1: `bee skillstation install [--apply] [--json]` and `bee skillstation alias <id> [--force] [--json]`.
- D2: one Rust constant lists `statechart`, `create-formal-verification-skill`, `maintain-formal-verification-skill` with repo `https://github.com/cursor/plugins` and path `pstack/skills/<id>`; `bee-deploying` stays in `HUB_SKILLS`.
- D3: check with `skillstation skill show <id> --json`; add with `skillstation skill add <repo> --path <path> --id <id> --yes` only under `--apply`; report `skillstation skill review <id>` as the owner's next step.
- D4: `install --apply` aliases listed active skills whose frontmatter has `disable-model-invocation: true`; `alias <id>` aliases any active hub skill.
- D5: alias = `.claude/skills/<id>/SKILL.md` only, with name, one-line description, copied `disable-model-invocation`, `metadata.skillstation_alias`, and a body that calls `skill_get`.
- D6: overwrite only a folder whose SKILL.md carries `metadata.skillstation_alias: <id>`; otherwise refuse and name the folder; `--force` overrides.
- D7: missing binary, non-zero exit or bad JSON is a loud skip row, exit 0; `alias <id>` for a non-active skill exits non-zero.
- D8: Claude Code only.
- D9: a knowledge concept under `docs/knowledge/areas/onboarding/` and a README "Skillstation" section.

## Load-bearing claims

Labels are `read` (file opened at that line) or `ran` (command run, output held); evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `skill show --json` carries the SKILL.md text, so bee can read its frontmatter | ran | `skillstation skill show create-formal-verification-skill --json` | `"content": "---\nname: create-formal-verification-skill` |
| 2 | `skill add --yes` never approves third-party content, so the owner must review | ran | `skillstation help skill add` | `--yes             Add without asking (never approves the content)` |
| 3 | The hub already tracks the three skills from cursor/plugins pstack | ran | `skillstation skill upstream statechart --json` | `"repository":"https://github.com/cursor/plugins.git","ref":"main","path":"pstack/skills/statechart"` |
| 4 | A verb group is wired by one `try_native` call in the verbs dispatcher | read | `packages/bee-rs/crates/bee/src/verbs/mod.rs:140` | `if let Some(code) = tmp_group::try_native(args, t0) {` |
| 5 | A skillstation runner with loud-skip error text already exists to reuse | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:309` | `fn run_skillstation(cwd: &Path, args: &[&str]) -> Result<Value, String> {` |
| 6 | `bee-deploying` is the only `HUB_SKILLS` row, so D2 does not overlap it | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:72` | `pub const HUB_SKILLS: [&str; 1] = ["bee-deploying"];` |
| 7 | The MCP `skill_get` tool needs only `skill_id`, so a few-line alias suffices | ran | `ToolSearch select:mcp__skillstation__skill_get` | `"required": ["skill_id"]` |

## Discovery

Read `docs/knowledge/areas/onboarding/host-project-artifacts.md` (the onboard
`--skillstation` publish and its loud-skip contract) and
`docs/knowledge/areas/rust-runtime/command-surface.md` (verb registry). Ran the
skillstation 0.14.1 CLI: `skill list`, `skill show`, `skill upstream`, `help
skill add`, `help skill activate`. Hub skills are absent from the Claude Code
skill list of this session, which is the defect the owner names.

## Approach

Recommended path: a new module `verbs/skillstation_group.rs` wired like
`tmp_group` (D1), reusing `run_skillstation` from `onboard/apply.rs` made
`pub(crate)` (D3, D7). A const table holds the related skills (D2). The alias
writer is a pure render function plus an ownership check (D5, D6). Docs follow
in one writer cell (D9).

Rejected: reuse `skillstation skill checkout` (copies the whole skill, D5);
fold into `bee onboard` (onboard must not touch the hub without a flag, and
the owner asked for its own command); write aliases for Codex too (D8).

Risk map:
- skillstation CLI shape drift / MEDIUM / Lands in ssa-1 / tests drive a fake `skillstation` script on PATH; claim rows 1–3 hold the real output.
- overwriting a user skill folder / MEDIUM / Lands in ssa-1 / a test proves the refusal and the `--force` path.

Waves: ssa-1, then ssa-2 (the docs quote the final flags and output).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ssa-1 adds the Rust verb group and its tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride ssa-1; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"The writer cell owns all docs."},
    {"stage":"knowledge-consolidation","classification":"required","role":"writer","reason":"ssa-2 writes the README section, the concept and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Plan check runs inline at this size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Standard with ≤5 product files and no hard-gate flag: inline check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Inline check."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Inline check."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline check."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline check."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI changes."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | `bee skillstation install` and `alias` work end to end | The owner asked for it now | `bee skillstation alias statechart` writes `.claude/skills/statechart/SKILL.md`; `/statechart` loads it | Hub skills that only the user may start |
| 2 | README section, knowledge concept, delivery | The owner asked for a Skillstation section | `README.md` § Skillstation | Other projects learn the command on refresh |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ssa-1 | Add the bee skillstation install and alias verbs | `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` (new), `packages/bee-rs/crates/bee/src/verbs/mod.rs`, `packages/bee-rs/crates/bee/src/onboard/apply.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | — | `bee skillstation install` lists each related skill as present, missing or skipped; `bee skillstation alias <id>` writes a small `.claude/skills/<id>/SKILL.md` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml skillstation` green |
| ssa-2 | Document Skillstation in the README and the knowledge bundle | `README.md`, `docs/knowledge/**`, `docs/history/skillstation-aliases/delivery.md` | ssa-1 | `README.md` has a Skillstation section naming both commands | `.bee/bin/bee knowledge check` green |

```json
[
  {
    "id": "ssa-1",
    "feature": "skillstation-aliases",
    "title": "Add the bee skillstation install and alias verbs",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/skillstation-aliases/CONTEXT.md",
      "docs/history/skillstation-aliases/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/tmp_group.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "docs/knowledge/areas/rust-runtime/command-surface.md"
    ],
    "action": "Add a verb group `bee skillstation` in a new module verbs/skillstation_group.rs, wired into verbs/mod.rs the way tmp_group is (D1). Verbs: `install [--apply] [--json]` and `alias <id> [--force] [--json]`. One const table RELATED_SKILLS holds id, repo `https://github.com/cursor/plugins` and path `pstack/skills/<id>` for statechart, create-formal-verification-skill, maintain-formal-verification-skill (D2). Reuse onboard/apply.rs run_skillstation (make it pub(crate)). install: for each row run `skillstation skill show <id> --json`; state present-active, present-draft, missing or skipped(reason). With --apply: add each missing row via `skillstation skill add <repo> --path <path> --id <id> --yes` and print `skillstation skill review <id>` as the owner's next step (D3); write an alias for each active row whose SKILL.md frontmatter (the `content` field) has `disable-model-invocation: true` (D4). Without --apply nothing is written. alias <id>: `skill show <id> --json`; not active → non-zero exit naming why (D7); else write the alias. Alias = `.claude/skills/<id>/SKILL.md` only, frontmatter name, one-line description from manifest.description, `disable-model-invocation: true` only when the hub skill has it, `metadata:\\n  skillstation_alias: <id>`; body of at most 6 lines telling the agent to call the skillstation MCP tool `skill_get` with skill_id `<id>` and follow the returned content as this skill, and to tell the user to run `skillstation connect -g` when the tool is missing (D5). Ownership: overwrite only a SKILL.md carrying `skillstation_alias: <id>`; any other existing `.claude/skills/<id>/` refuses naming the folder unless --force (D6). Missing binary, non-zero exit or bad JSON = one skip row with reason, exit 0 for install (D7). Claude Code only (D8). Add registry entries for both verbs in generated/registry_payload.json following command-surface.md so `bee skillstation install --help` prints its flags. Tests in the module: drive a fake `skillstation` shell script put first on PATH (cfg(unix)) for present/missing/skip/apply; pure tests for alias render and ownership refusal and --force.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml skillstation",
    "must_haves": {
      "truths": [
        "bee skillstation install without --apply writes nothing and lists every RELATED_SKILLS row",
        "bee skillstation alias <id> for an active hub skill writes .claude/skills/<id>/SKILL.md containing skill_get and skillstation_alias: <id>",
        "an existing non-alias .claude/skills/<id>/ is refused without --force",
        "a missing skillstation binary gives skip rows and exit 0 for install"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs", "substantive": "install and alias verbs, RELATED_SKILLS, alias render, tests"}
      ],
      "key_links": [
        "verbs/mod.rs calls skillstation_group::try_native",
        "skillstation_group uses onboard::apply::run_skillstation"
      ],
      "prohibitions": [
        "No code comments",
        "Never approve hub content or run skill activate",
        "Never write a hub file directly"
      ],
      "open_choices": [
        "Text report layout, as long as each row names id, state and next step"
      ]
    }
  },
  {
    "id": "ssa-2",
    "feature": "skillstation-aliases",
    "title": "Document Skillstation in the README and the knowledge bundle",
    "lane": "standard",
    "role": "writer",
    "deps": ["ssa-1"],
    "decisions": ["D9"],
    "files": [
      "README.md",
      "docs/knowledge/**",
      "docs/history/skillstation-aliases/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-aliases/CONTEXT.md",
      "docs/history/skillstation-aliases/plan.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs"
    ],
    "action": "Add a short `Skillstation` section to README.md: why hub skills cannot be started with /<id>, what `bee skillstation install [--apply]` and `bee skillstation alias <id> [--force]` do, and that content approval stays with the hub owner (`skillstation skill review <id>`). Write one knowledge concept under docs/knowledge/areas/onboarding/ for the verb group, citing history D-numbers as `skillstation-aliases D<n>` (never raw decision ids), with Pointers to skillstation_group.rs. Run `bee knowledge consolidate --feature skillstation-aliases`, write docs/history/skillstation-aliases/delivery.md listing each concept changed, one line each. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "README.md has a Skillstation section naming both commands",
        "a docs/knowledge/areas/onboarding concept describes the verb group anchored to skillstation_group.rs"
      ],
      "artifacts": [
        {"path": "docs/history/skillstation-aliases/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": ["the delivery lands in the same merge as the code"],
      "prohibitions": ["No restating of code in concepts", "No code comments"],
      "open_choices": []
    }
  }
]
```

## Test matrix

- Happy: fake hub reports all three active, one with `disable-model-invocation: true`; `install --apply` writes one alias. Pass when exactly `.claude/skills/<that-id>/SKILL.md` exists and contains `skill_get`.
- Happy: `alias statechart` with an active hub skill. Pass when the SKILL.md has `skillstation_alias: statechart` and no `disable-model-invocation` line.
- Edge: `install` without `--apply`, one row missing. Pass when no file is written and no `skill add` call is recorded by the fake.
- Edge: `install --apply`, one row missing. Pass when the fake records `skill add https://github.com/cursor/plugins --path pstack/skills/<id> --id <id> --yes` and the report names `skillstation skill review <id>`.
- Error: `skillstation` not on PATH. Pass when every row is a skip with the reason and the exit is 0.
- Error: `.claude/skills/<id>/SKILL.md` exists without the marker. Pass when `alias <id>` exits non-zero naming the folder and the file is unchanged; with `--force` the alias replaces it.
- Error: `alias <id>` for a draft skill. Pass when the exit is non-zero and the reason names the state.

## Open Questions

(none)

## Out of scope

Aliases for Codex and other runtimes (D8); activation or content approval of hub skills; `bee-deploying` publishing, which stays with `bee onboard --skillstation`.
