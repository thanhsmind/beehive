---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: skillstation-more-skills

## Summary

The Skillstation hub serves more bee skills to Claude Code: eight engine
skills (`bee-unslop`, `bee-researching`, `bee-knowledge-setup`,
`bee-knowledge-upkeep`, `bee-verifying`, `bee-verify-upkeep`,
`bee-wayfinding`, `bee-grooming`), `bee-deploying`, and the design core with
every design pack the project picked. Each joins the hub-served set of
skillstation-human-skills: the `--skillstation` switch publishes it, removes
the local Claude copy and writes a `/<id>` alias, all gated on a clean
publish. An alias keeps the engine skill's `disable-model-invocation` value.
`product-description` goes to the hub by hand (D7).

Mode: `small`. Class feature, 1 flag (external-systems: the hub). Product
files: `skills.rs`, `plan.rs`, `apply.rs` and the `install.sh` help text,
as in skillstation-human-skills.
Why this is the least workflow that protects the work: two serial code cells,
each with a red test first, both extending the proven hub-served path.

Playbook: `skills/bee-planning/playbooks/feature.md` (class `feature`).

## Requirements (from CONTEXT.md)

- D1: eight engine skills join `HUB_SERVED_SKILLS` under the same rules.
- D2: an alias copies the engine skill's `disable-model-invocation` value.
- D3: `bee-deploying` joins the set; `HUB_SKILLS` publishes it only when the Claude runtime is out of scope.
- D4: the design core and each picked pack are hub-served from the design root; unpicked packs are untouched.
- D5: sibling-pack links stay as they are.
- D6: the doctor `design_skill` row is unchanged.
- D7: `product-description` is added to the hub by hand, with no code.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One constant names the human-started hub set | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:81` | `pub const HUB_SERVED_SKILLS: [&str; 5] =` |
| 2 | The separate publish list holds bee-deploying | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:78` | `pub const HUB_SKILLS: [&str; 1] = ["bee-deploying"];` |
| 3 | Hub-served ids come from the engine skills root only | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:119` | `fn principle_ids(engine: &Engine) -> Vec<String> {` |
| 4 | Every alias is written with model invocation allowed | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:134` | `let content = render_alias(&id, &frontmatter_description(&rendered).unwrap_or_default(), false);` |
| 5 | Design skills sync from their own root | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:836` | `pub fn compute_design_sync(engine: &Engine, repo_root: &Path, design_packs: &[String]) -> SkillSync {` |
| 6 | Apply gates on any failed hub-served publish | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:988` | `is_hub_served(skill) && failed_principle.is_none()` |
| 7 | The doctor design row checks a directory | read | `packages/bee-rs/crates/bee/src/doctor.rs:268` | `const DESIGN_SKILL: &str = "bee-design";` |

## Discovery

The design core and packs live in `packages/bee/design/`, synced by
`compute_design_sync` through `SkillSources::picks`. Four pack files point at
a sibling pack with `../bee-design-<pack>/`; the design core points at
`../bee-design-motion/`. This repo picks `web` and `charts`, so none of those
links targets a picked pack. Hub ids differ per project only in which packs
are published and aliased.

## Approach

Recommended path. sms-1 grows `HUB_SERVED_SKILLS` (D1, D3), makes the alias
render read `disable-model-invocation` from the engine frontmatter (D2), and
limits `HUB_SKILLS` to a run whose Claude runtime is out of scope (D3).
sms-2 makes the hub-served ids carry their source root, adds the design core
and picked packs from the design root (D4), and applies the same publish,
removal, alias and deletion-domain rules to them in `compute_design_sync`.

Rejected: rewriting sibling-pack links at publish (D5); a new config key or
flag (skillstation-human-skills D1).

Risk map:
- the design alias is deleted and rewritten on every run / HIGH / Lands in sms-2 / a re-plan test after apply asserts no design item.
- a `disable-model-invocation` skill alias lets the agent start it / MEDIUM / Lands in sms-1 / a test reads the alias frontmatter.
- `bee-deploying` publishes twice / MEDIUM / Lands in sms-1 / a switch plan test counts its publish items.

Waves: sms-1, then sms-2 (same files).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"sms-1 and sms-2 change onboard planning with a red test first."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each cell; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"Each cell updates the concept, README and install help."},
    {"stage":"knowledge-consolidation","classification":"not-applicable","role":"writer","reason":"Each cell owns its concept edit and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release unless the owner asks."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
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
| 1 | the hub serves eight more engine skills, bee-deploying and the picked design skills; `/<id>` aliases replace the Claude copies | Owner request 2026-11-14 | `bee onboard --skillstation` plan on a hub-mode project lists publishes, removals and aliases for them | Rollout to every project through `install.sh --skillstation` |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| sms-1 | Serve nine more engine skills from the hub, keeping each skill's model-invocation flag | `skills.rs`, `plan.rs`, `apply.rs`, `tests.rs`, `install.sh`, `README.md`, concept, delivery, release manifest | — | a hub-mode onboard aliases `bee-unslop` and `bee-verifying`, the latter with `disable-model-invocation: true` | `cargo test … --bin bee -- skillstation onboard` green, `bee dev release-manifest --check` |
| sms-2 | Serve the design core and the picked design packs from the hub | `skills.rs`, `plan.rs`, `tests.rs`, `README.md`, concept, delivery | sms-1 | a hub-mode onboard with packs web,charts aliases `bee-design`, `bee-design-web`, `bee-design-charts` and leaves other packs alone | `cargo test … --bin bee -- skillstation onboard design` green |

```json
[
  {
    "id": "sms-1",
    "feature": "skillstation-more-skills",
    "title": "Serve nine more engine skills from the hub, keeping each skill's model-invocation flag",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "3af71fed-6edb-438c-a688-fbef26d0e0ec"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "scripts/install.sh",
      "README.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/history/skillstation-more-skills/delivery.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/skillstation-more-skills/CONTEXT.md",
      "docs/history/skillstation-human-skills/CONTEXT.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs"
    ],
    "action": "RED FIRST in packages/bee-rs/crates/bee/src/onboard/tests.rs, next to the skillstation-human-skills switch tests (search for bee-how in tests.rs). Give the test engine skills root a `bee-unslop` skill (frontmatter description only) and a `bee-verifying` skill whose frontmatter carries `disable-model-invocation: true`, plus a `bee-deploying` skill. Tests: (a) the switch plan with --skillstation and runtime claude has a publish_skillstation item, an after_publish remove_skill (target repo-claude) and an after_publish skillstation_alias item for bee-unslop, bee-verifying and bee-deploying; (b) the bee-verifying alias content carries `disable-model-invocation: true` and the bee-unslop alias carries no disable-model-invocation line; (c) bee-deploying has exactly one publish_skillstation item in that plan; (d) a --skillstation plan with the Claude runtime out of scope still publishes bee-deploying once through HUB_SKILLS and plans no alias; (e) apply of the full switch and a re-plan in hub mode plan nothing for the three. Run them and record the red output. FIX: in skills.rs add bee-unslop, bee-researching, bee-knowledge-setup, bee-knowledge-upkeep, bee-verifying, bee-verify-upkeep, bee-wayfinding, bee-grooming and bee-deploying to HUB_SERVED_SKILLS. In plan.rs hub_alias_items, pass render_alias the engine skill's disable-model-invocation value read from the rendered frontmatter (true only for the literal value true). Make publish_skillstation_items skip any HUB_SKILLS id that the hub-served publish of the same run already covers, so a Claude-scope run publishes bee-deploying once and a run without the Claude runtime keeps its HUB_SKILLS publish. Fix any existing test that assumed bee-deploying keeps a local Claude copy under the switch, naming that change in delivery.md. DOCS: scripts/install.sh --skillstation help text names the hub-served set by group (principles, the human-started skills, and the newly added skills); README.md hub paragraph says the same; docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md Business Rules gain the larger set, the alias keeping disable-model-invocation and the single bee-deploying publish, citing skillstation-more-skills D1-D3 (never raw decision ids). Write docs/history/skillstation-more-skills/delivery.md (what changed, red and green test names, proof). One commit. REGEN: install.sh is a release-manifest root; finish with bee dev regen and commit the regenerated docs/history/codex-harness-hardening/release-manifest.json in the same commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the --skillstation switch publishes, removes and aliases the nine added engine skills for the Claude runtime",
        "an alias of a skill whose engine frontmatter says disable-model-invocation: true carries that line; any other alias does not",
        "a --skillstation run publishes bee-deploying exactly once, with or without the Claude runtime in scope",
        "codex, opencode, plugin and legacy global targets keep the added skills"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/skills.rs", "substantive": "the larger HUB_SERVED_SKILLS"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "the alias flag and the single bee-deploying publish"}
      ],
      "key_links": [
        "every hub-served test still goes through is_hub_served",
        "aliases are rendered with render_alias and recognised with carries_alias_marker"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation",
        "No new config key or CLI flag"
      ],
      "open_choices": [
        "How the disable-model-invocation value is read from frontmatter",
        "Helper names"
      ]
    }
  },
  {
    "id": "sms-2",
    "feature": "skillstation-more-skills",
    "title": "Serve the design core and the picked design packs from the hub",
    "lane": "small",
    "role": "code",
    "deps": ["sms-1"],
    "decisions": ["D4", "D5", "D6", "3af71fed-6edb-438c-a688-fbef26d0e0ec"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "README.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/history/skillstation-more-skills/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-more-skills/CONTEXT.md",
      "docs/history/skillstation-more-skills/delivery.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "action": "RED FIRST in packages/bee-rs/crates/bee/src/onboard/tests.rs, beside the sms-1 tests. Give the test engine a design root with bee-design, bee-design-web, bee-design-charts and bee-design-motion (each a SKILL.md with a frontmatter description) and pick packs web,charts. Tests: (a) the switch plan with --skillstation and runtime claude has a publish_skillstation item (source the design root), an after_publish remove_skill (target repo-claude) and an after_publish skillstation_alias item for bee-design, bee-design-web and bee-design-charts, and none for bee-design-motion; (b) a design publish that fails keeps every local copy, design and engine skills alike, and writes no alias; (c) apply of the full switch then a hub-mode re-plan plans nothing for the three design skills (no sync_skill from compute_design_sync, no remove_skill, no alias); (d) hub mode with no pack picked plans no design hub item; (e) codex and agents targets keep the design skills. Run them and record the red output. FIX: let the hub-served ids carry their source root, adding bee-design and each picked pack from engine.design_root when the project picked packs; publish them with that root as source; include them in principles_served_by_hub, the removals and the alias items. In compute_design_sync, drop the hub-served design skills from the repo-claude target in hub mode exactly as SkillSources::entries does for engine skills, and extend the deletion-domain alias exception to them. apply.rs needs no change when the publish item carries its source root; change it only if a test proves otherwise. DOCS: README.md hub paragraph and the concept's Business Rules name the design core and picked packs as hub-served, citing skillstation-more-skills D4-D6, and state that sibling-pack links stay as written. Append the sms-2 section to docs/history/skillstation-more-skills/delivery.md. One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard design",
    "must_haves": {
      "truths": [
        "the --skillstation switch publishes, removes and aliases bee-design and each picked design pack for the Claude runtime",
        "a design pack the project did not pick is neither published nor aliased",
        "after a full switch a hub-mode re-plan plans nothing for the hub-served design skills",
        "a failed publish of any hub-served skill keeps every local copy and writes no alias"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "hub-served ids with their source root, design included"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/skills.rs", "substantive": "compute_design_sync drops hub-served design skills in hub mode"}
      ],
      "key_links": [
        "design hub ids follow the design.packs pick that SkillSources::picks reads",
        "aliases are rendered with render_alias and recognised with carries_alias_marker"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation",
        "No rewrite of sibling-pack links"
      ],
      "open_choices": [
        "How an id carries its source root",
        "Helper names"
      ]
    }
  }
]
```
