---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: skillstation-human-skills

## Summary

The Skillstation hub serves five more bee skills to Claude Code:
`bee-how`, `bee-teach`, `bee-technical-writing`, `bee-why` and
`bee-writing-skills`. They join the principles as one hub-served set, so the
existing `--skillstation` switch publishes them, removes the local copies and
gates both on a clean publish. Each removed copy becomes a `/<id>` alias so a
human can still start it by name. `scripts/install.sh --skillstation` rolls it
out to any project with no new flag.

Mode: `small` — class feature, 1 flag (external system: the hub). Product
files: `plan.rs`, `skills.rs`, `apply.rs` and the `install.sh` help text. The
fourth file is help prose only, so the lane stays small (named deviation,
recorded on the route).
Why this is the least workflow that protects the work: one code cell with a
red test first; every rule is an extension of the proven principle path.

Playbook: `skills/bee-planning/playbooks/feature.md` (class `feature`).

## Requirements (from CONTEXT.md)

- D1: one constant lists the five; they join the principles as one hub-served set under the same hub mode and `--skillstation` switch.
- D2: every principle rule applies to the five: engine-rendered publish with trigger flags, publish skip, drop-only-when-all-active, `after_publish` gate on any failed hub-served publish.
- D3: each removed local copy of the five becomes a `/<id>` alias with the engine description; principles get none; in the switch the alias items are `after_publish` and come after the removals.
- D4: the repo-claude deletion domain leaves a hub-served alias directory alone.
- D5: other runtimes keep their local copies.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The hub-served set is chosen by one predicate (after shs-1) | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:122` | `is_hub_served(&e.name)` |
| 2 | The switch alias items come from one helper (after shs-1) | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:127` | `fn hub_alias_items(engine: &Engine, repo_root: &Path, after_publish: bool) -> Vec<Value> {` |
| 3 | Any other `bee-*` directory not in the sources is removed | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:332` | `items.push(item("remove_skill", name, name, "installed"));` |
| 4 | Apply gates on any failed hub-served publish (after shs-1) | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:988` | `is_hub_served(skill) && failed_principle.is_none()` |
| 5 | The alias render and marker exist | read | `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs:137` | `pub(crate) fn render_alias(id: &str, description: &str, disable_model: bool) -> String {` |
| 6 | install.sh already forwards the switch | read | `scripts/install.sh:518` | `ONBOARD_FLAGS+=("--skillstation")` |

## Discovery

The alias apply arm (`apply.rs` `"skillstation_alias"`) refuses a target whose
`SKILL.md` has no marker, so the removal must run before the alias write in the
same apply. Section 7 of `compute_plan` appends the alias items after the skill
sync items and after the switch sort, so plan order already puts them last.

## Approach

Recommended path: one predicate `hub_served(name)` beside `PRINCIPLE_PREFIX` in
`skills.rs` (prefix or one of the five) and a constant for the five. Every
prefix test in claims 1, 2 and 4 calls it. The five that exist in the engine
skills root are the only ones added, so test engines without them behave as
today. Alias items for the five are planned when the repo-claude target drops
them (served) or during the switch (`after_publish`). The deletion domain skips
a `bee-*` directory that is hub-served, absent from the sources and carries its
own alias marker.

Rejected: a second config key or flag (D1); deleting without an alias (D3).

Risk map:
- the alias is deleted and rewritten on every run / HIGH / Lands in shs-1 / a re-plan test after apply asserts no item for the five.
- a failed publish of one of the five still removes local copies / MEDIUM / Lands in shs-1 / a test fails `bee-how` publish and sees the removals and aliases skipped.

Waves: shs-1, then shs-2 (fix found at the leader check of shs-1: a re-run of the switch deleted the alias).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"shs-1 changes onboard planning and apply with a red test first."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride shs-1; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"shs-1 updates the concept, README and install help."},
    {"stage":"knowledge-consolidation","classification":"not-applicable","role":"writer","reason":"shs-1 owns the concept edit and delivery.md."},
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
| 1 | the hub serves the five skills; `/<id>` aliases replace the Claude copies | Owner request 2026-11-14 | `bee onboard --skillstation` plan on a hub-mode project lists publishes, removals and aliases for the five | Rollout to every project through `install.sh --skillstation` |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| shs-1 | Serve five human-started bee skills from the hub with /<id> aliases | `packages/bee-rs/crates/bee/src/onboard/plan.rs`, `packages/bee-rs/crates/bee/src/onboard/skills.rs`, `packages/bee-rs/crates/bee/src/onboard/apply.rs`, `packages/bee-rs/crates/bee/src/onboard/tests.rs`, `scripts/install.sh`, `README.md`, `docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md`, `docs/history/skillstation-human-skills/delivery.md`, `docs/history/codex-harness-hardening/release-manifest.json` | — | a hub-mode onboard replaces `.claude/skills/bee-how` with an alias and the re-plan is clean | `cargo test … --bin bee -- skillstation onboard` green, `bee dev release-manifest --check` |
| shs-2 | Keep the /<id> alias when the --skillstation switch runs again | `packages/bee-rs/crates/bee/src/onboard/plan.rs`, `packages/bee-rs/crates/bee/src/onboard/tests.rs`, `docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md`, `docs/history/skillstation-human-skills/delivery.md` | shs-1 | a second `install.sh --skillstation` keeps `/bee-how` | `cargo test … --bin bee -- skillstation onboard` green |

```json
[
  {
    "id": "shs-1",
    "feature": "skillstation-human-skills",
    "title": "Serve five human-started bee skills from the hub with /<id> aliases",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D4",
      "D5",
      "3eadc5cf-685e-4f43-b312-41d8489c4e9c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "scripts/install.sh",
      "README.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/history/skillstation-human-skills/delivery.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/skillstation-human-skills/CONTEXT.md",
      "docs/history/skillstation-human-skills/plan.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs"
    ],
    "action": "RED FIRST in packages/bee-rs/crates/bee/src/onboard/tests.rs, next to the existing switch tests (switching_hub, switch_fixture, the_skillstation_flag_on_a_local_project_plans_the_switch, a_fully_published_switch_leaves_no_claude_principle_folder). Give the test engine skills root a `bee-how` skill dir (SKILL.md with a frontmatter description) beside the principles. Tests: (a) the switch plan with --skillstation and runtime claude has a publish_skillstation item for bee-how, an after_publish remove_skill for bee-how (target repo-claude) and an after_publish skillstation_alias item for bee-how at .claude/skills/bee-how/SKILL.md placed after that removal, whose content carries `skillstation_alias: bee-how` and the engine description and no disable-model-invocation line; (b) apply of the full switch with every publish succeeding leaves .claude/skills/bee-how/SKILL.md as the alias, and a re-plan (hub mode, every hub-served skill active) plans no item at all for bee-how (no remove_skill, no sync, no alias); (c) a switch where the bee-how publish fails keeps the local bee-how copy and the local principles, writes no alias, and reports the skillstation-principles skipped row; (d) hub mode with the hub read failing keeps the local bee-how copy and plans no alias; (e) the codex/agents targets keep bee-how. Run them and record the red output. FIX: in skills.rs add a constant HUB_SERVED_SKILLS = [bee-how, bee-teach, bee-technical-writing, bee-why, bee-writing-skills] and a predicate (prefix PRINCIPLE_PREFIX or in the constant); use it in SkillSources::entries where without_principles filters today. In the repo-claude deletion domain loop, skip a bee-* directory when without_principles is on, the name is hub-served and its SKILL.md carries the alias marker for its own name (carries_alias_marker). In plan.rs make principle_ids return every hub-served id present in the engine skills root (principles plus the five that exist), so publish_principle_items, principles_served_by_hub and principle_removal_items cover the five. Add alias planning for the five that exist: render_alias(id, engine SKILL.md description rendered for claude, false); plan a skillstation_alias item when the repo-claude target drops them (principles_from_hub) or during the switch (marked after_publish, appended after the removals); skip the item when the current file already equals the content. In apply.rs use the predicate for the trigger flags on skill commit and for failed_principle. Principles get no alias. DOCS: scripts/install.sh --skillstation help text names the five skills and their aliases; README.md hub paragraph (the one about the 24 principles) and the --skillstation flag line say the same; docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md Business Rules gain the hub-served set, the alias replacement and the deletion-domain exception citing skillstation-human-skills D1-D5 (never raw decision ids). Write docs/history/skillstation-human-skills/delivery.md (what changed, red and green test names, proof). One commit. REGEN: install.sh is a release-manifest root; finish with bee dev regen (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write) and commit the regenerated docs/history/codex-harness-hardening/release-manifest.json in the same commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the --skillstation switch publishes, removes and aliases bee-how, bee-teach, bee-technical-writing, bee-why and bee-writing-skills for the Claude runtime",
        "after a full switch, .claude/skills/<id>/SKILL.md for each of the five is an alias with the engine description and the re-plan plans nothing for them",
        "a failed publish of any hub-served skill keeps every local copy and writes no alias",
        "codex, opencode, plugin and legacy global targets keep the five"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/skills.rs",
          "substantive": "the hub-served predicate and the deletion-domain alias exception"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "the hub-served ids and the alias items"
        }
      ],
      "key_links": [
        "every former PRINCIPLE_PREFIX test that picks the hub set calls the one predicate",
        "aliases are rendered with render_alias and recognised with carries_alias_marker"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation",
        "No new config key or CLI flag"
      ],
      "open_choices": [
        "How the engine description is read from SKILL.md frontmatter",
        "Helper names"
      ]
    }
  },
  {
    "id": "shs-2",
    "feature": "skillstation-human-skills",
    "title": "Keep the /<id> alias when the --skillstation switch runs again",
    "lane": "small",
    "role": "code",
    "deps": [
      "shs-1"
    ],
    "decisions": [
      "D2",
      "D3",
      "D4",
      "3eadc5cf-685e-4f43-b312-41d8489c4e9c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/history/skillstation-human-skills/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-human-skills/CONTEXT.md",
      "docs/history/skillstation-human-skills/delivery.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "action": "DEFECT: in a project that already holds the bee-how alias at .claude/skills/bee-how/SKILL.md, a second `bee onboard --skillstation --apply` (what a re-run of scripts/install.sh --skillstation does) plans the after_publish remove_skill for bee-how from principle_removal_items, but hub_alias_items returns no item because the current file already equals the alias render. Apply deletes the alias and never writes it back; the installer recheck then plans the alias again. RED FIRST in packages/bee-rs/crates/bee/src/onboard/tests.rs beside the shs-1 tests: run the full switch apply with every publish succeeding, then run a second switch apply with the hub now reporting every hub-served skill active with the same files; assert .claude/skills/bee-how/SKILL.md still exists and carries `skillstation_alias: bee-how`, and that a plan without --skillstation (hub mode, all served) plans no item naming bee-how. Watch it fail for the reported reason and record the red output. FIX in plan.rs, smallest change: during the switch (after_publish true) hub_alias_items always plans the alias item, because the removal before it deletes the folder; the equal-content skip stays only for the non-switch path. Add one sentence to the Business Rules of docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md that a re-run of the switch rewrites the alias after the removal, citing skillstation-human-skills D3-D4. Add the red and green test names to docs/history/skillstation-human-skills/delivery.md. One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard",
    "must_haves": {
      "truths": [
        "a second --skillstation apply on a project that already holds the five aliases leaves every alias in place",
        "after that second apply, a plan without --skillstation plans no item for the five"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "the switch alias items are planned even when the file is current"
        }
      ],
      "key_links": [
        "the alias item still sorts after the remove_skill item for the same id"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation"
      ]
    }
  }
]
```

## Test matrix

- Happy: the full switch. Pass when the five are published, removed and aliased, and the re-plan is clean.
- Edge: a project already in hub mode with real copies of the five. Pass when the copies are removed and the aliases written in one apply.
- Error: a bee-how publish fails. Pass when every local copy stays and no alias is written.
- Error: the hub read fails in hub mode. Pass when the local copies stay.

## Open Questions

(none)

## Out of scope

A release (owner did not ask); `bee skillstation install`'s related-skill list.
