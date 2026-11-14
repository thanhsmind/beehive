---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: skillstation-onboard-aliases

## Summary

When bee is installed or refreshed in a project, it now writes the `/<id>`
alias skills for the hub skills bee relies on, if the machine has
skillstation and the hub has them active. A user's own skill folder is never
touched. Adding missing skills to the hub stays behind `--skillstation`, on
both `scripts/install.sh` and `bee onboard`. Then a release ships it.

Mode: `standard` — 2 risk flags: external systems (the `skillstation` CLI), public contracts (onboard plan output and an install flag)
Why this is the least workflow that protects the work: one code cell and one writer cell, plan check inline (4 product files, no hard-gate flag).

Playbook: `skills/bee-planning/playbooks/feature.md` (class `feature`).

## Requirements (from CONTEXT.md)

- D1: onboard plans a `skillstation_alias` item per active, user-started `RELATED_SKILLS` row whose alias is missing or stale; default on for the Claude runtime.
- D2: a non-alias folder gets no item and one `skipped_skills` entry; never `--force`.
- D3: no binary / bad exit / bad JSON = one `skipped_skills` entry `{target: "skillstation-alias", reason}`.
- D4: `--skillstation` also adds missing rows to the hub as drafts.
- D5: `scripts/install.sh --skillstation` passes through to onboard.
- D6: reuse `skillstation_group.rs`; injected runner; onboard tests deterministic.
- D7: Claude Code only.
- D8: release with `scripts/release.sh` at the next minor version after merge.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Onboard already has a flag-gated skillstation branch in the plan to extend | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:1438` | `if opts.skillstation && opts.sync_skills && skill_sync.blocked.is_none() {` |
| 2 | install.sh forwards onboard flags through one array | read | `scripts/install.sh:501-502` | `if [ "$GLOBAL_SKILLS" -eq 1 ]; then /   ONBOARD_FLAGS+=("--global-skills")` |
| 3 | install.sh rechecks onboard after apply, so the plan must be idempotent | read | `scripts/install.sh:820` | `RECHECK="$( cd "$BEE_SRC" && "$HOST_BEE" onboard --repo-root "$TARGET_DIR" --jso` |
| 4 | The alias render to reuse exists | read | `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs:88` | `fn render_alias(id: &str, description: &str, disable_model: bool) -> String {` |
| 5 | Onboard apply already reports loud skips in `skipped_skills` | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:530` | `let mut skipped_skills: Vec<Value> = Vec::new();` |
| 6 | The three related skills are active with `disable-model-invocation: true` on this hub | ran | `bee skillstation install --apply` (main, after skillstation-aliases merge) | `statechart: present-active [alias /mnt/data/Projects/goglbe/beehive/.claude/skills/statechart/SKILL.md] - next: use /statechart` |

## Discovery

Read the CONTEXT read set. `scripts/install.sh` runs `bee onboard` three
times (plan at :664, apply at :788, recheck at :820) with the same
`ONBOARD_FLAGS`, so a step inside onboard reaches every install with no other
install.sh change than the flag.

## Approach

Recommended path: expose `pub(crate)` helpers from `skillstation_group.rs`
(hub read, wants-alias, render, ownership) and call them from onboard plan
with an injected runner (D6). Plan emits `skillstation_alias` items (D1) and
skip entries (D2, D3); apply writes the rendered bytes the plan carried. Under
`--skillstation`, plan adds `skillstation_add` items for missing rows (D4).
`install.sh` adds the flag (D5).

Rejected: call `bee skillstation install --apply` from install.sh (onboard
recheck and `bee onboard --apply` refreshes would not carry it); write aliases
for every hub skill (only the listed ones are bee's to manage).

Risk map:
- onboard tests turning red on a machine with skillstation / MEDIUM / Lands in sooa-1 / default test runner reports absent; the whole onboard test module runs green.
- recheck not `up_to_date` after apply / MEDIUM / Lands in sooa-1 / a test plans, applies, re-plans and sees zero alias items.

Waves: sooa-1, then sooa-2.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"sooa-1 changes onboard and install.sh with tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride sooa-1; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"The writer cell owns all docs."},
    {"stage":"knowledge-consolidation","classification":"required","role":"writer","reason":"sooa-2 updates the concepts, README and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The leader runs scripts/release.sh per LEADER.md."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Plan check runs inline at this size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Inline check at this size."},
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
| 1 | onboard and install.sh write the aliases | Owner chose option A | `bee onboard --repo-root <scratch> --apply` writes `.claude/skills/statechart/SKILL.md`; a second plan is `up_to_date` | Every project gets `/<id>` on install |
| 2 | Docs, then release | Owner chose release | README and concept name the onboard step | Other projects get it on their next install |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| sooa-1 | Write skillstation aliases during onboard and install | `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs`, `packages/bee-rs/crates/bee/src/onboard/plan.rs`, `packages/bee-rs/crates/bee/src/onboard/apply.rs`, `scripts/install.sh` | — | an install in a project on a machine with skillstation leaves `/statechart` ready | `cargo test … --bin bee -- skillstation onboard` green |
| sooa-2 | Document the onboard alias step | `README.md`, `docs/knowledge/**`, `docs/history/skillstation-onboard-aliases/delivery.md` | sooa-1 | README says install writes the aliases | `.bee/bin/bee knowledge check` green |

```json
[
  {
    "id": "sooa-1",
    "feature": "skillstation-onboard-aliases",
    "title": "Write skillstation aliases during onboard and install",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6", "D7"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "scripts/install.sh"
    ],
    "read_first": [
      "docs/history/skillstation-onboard-aliases/CONTEXT.md",
      "docs/history/skillstation-onboard-aliases/plan.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "action": "Make bee onboard write the /<id> alias skills (D1-D3) and add hub drafts under --skillstation (D4), reusing skillstation_group.rs helpers made pub(crate) with an injected runner (D6). Plan: when the Claude runtime is in scope, for each RELATED_SKILLS row read the hub with `skillstation skill show <id> --json` (cwd = repo root); emit one `skillstation_alias` item {action, skill, path: .claude/skills/<id>/SKILL.md, content} only when the row is active with `disable-model-invocation: true` and the file is missing or is a bee alias (marker `skillstation_alias: <id>`) whose bytes differ from render_alias. A folder without the marker: no item, one skipped_skills entry naming the folder; never --force (D2). No binary / non-zero / bad JSON: no item, ONE skipped_skills entry {target: \"skillstation-alias\", reason} (D3). Under --skillstation, also emit a `skillstation_add` item per missing row that apply runs as `skillstation skill add <repo> --path <path> --id <id> --yes --json`, failures a loud skipped_skills entry (D4). Apply writes the item's content. The plan must be idempotent: plan, apply, plan again yields no alias item. Onboard tests must not see the real skillstation: inject the runner through the onboard options or a seam whose test default reports `skillstation is not on PATH` (D6). scripts/install.sh: add `--skillstation` to usage and arg parsing and push it onto ONBOARD_FLAGS (D5). Claude only (D7). Tests: alias planned and written for an active user-started row; no item for a model-invoked row; idempotent re-plan; foreign folder skip; absent binary gives one skip; --skillstation plans add for a missing row; without it no add.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard",
    "must_haves": {
      "truths": [
        "bee onboard --apply writes .claude/skills/<id>/SKILL.md for an active hub skill with disable-model-invocation: true",
        "a second onboard plan after apply carries no skillstation_alias item",
        "a .claude/skills/<id>/ without the skillstation_alias marker is left unchanged and named in skipped_skills",
        "without skillstation on PATH onboard reports one skipped_skills entry and its normal result",
        "scripts/install.sh --skillstation passes --skillstation to bee onboard"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "skillstation_alias and skillstation_add items"}
      ],
      "key_links": [
        "onboard plan calls skillstation_group helpers, not a copy",
        "install.sh ONBOARD_FLAGS carries --skillstation"
      ],
      "prohibitions": [
        "No code comments",
        "Never pass --force from onboard",
        "Never change the hub without --skillstation",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": [
        "How the runner is injected, as long as tests are deterministic"
      ]
    }
  },
  {
    "id": "sooa-2",
    "feature": "skillstation-onboard-aliases",
    "title": "Document the onboard alias step",
    "lane": "standard",
    "role": "writer",
    "deps": ["sooa-1"],
    "decisions": ["D1", "D4", "D5"],
    "files": [
      "README.md",
      "docs/knowledge/**",
      "docs/history/skillstation-onboard-aliases/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-onboard-aliases/CONTEXT.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md"
    ],
    "action": "Update README.md § Skillstation: install and onboard now write the aliases on their own; `--skillstation` on install.sh or onboard also adds missing skills to the hub as drafts. Extend docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md and the skillstation paragraph of host-project-artifacts.md and the flag list in installer-entrypoints-and-source-staging.md (one fact, one home; cite `skillstation-onboard-aliases D<n>`, never raw decision ids). Run `bee knowledge consolidate --feature skillstation-onboard-aliases`, write docs/history/skillstation-onboard-aliases/delivery.md, run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "README.md § Skillstation says install and onboard write the aliases",
        "the onboarding concept names the skillstation_alias plan item"
      ],
      "artifacts": [
        {"path": "docs/history/skillstation-onboard-aliases/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": ["the delivery lands in the same merge as the code"],
      "prohibitions": ["No restating of code in concepts", "No code comments"],
      "open_choices": []
    }
  }
]
```

## Test matrix

- Happy: fake hub, `statechart` active with `disable-model-invocation: true`. Pass when apply writes `.claude/skills/statechart/SKILL.md` and a re-plan has no `skillstation_alias` item.
- Edge: row active without the flag. Pass when no item is planned for it.
- Edge: `--skillstation` with one row missing. Pass when one `skillstation_add` item is planned; without the flag none is.
- Error: skillstation absent. Pass when `skipped_skills` has exactly one `skillstation-alias` entry and onboard still reports its usual result.
- Error: foreign `.claude/skills/statechart/SKILL.md`. Pass when it is unchanged and named in `skipped_skills`.

## Open Questions

(none)

## Out of scope

Codex and other runtimes (D7); activating or approving hub content.
