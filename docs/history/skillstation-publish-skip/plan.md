---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: skillstation-publish-skip

## Summary

`bee onboard --skillstation` stops re-publishing a skill that the hub already
holds active with the same files. It reads each skill from the hub once at plan
time, compares per-file sha256 digests with the engine skill rendered for
claude, and plans a publish only for a missing, inactive or changed skill. The
switch's `after_publish` contract does not change.

Mode: `small` — class bugfix, 3 product files, no hard-gate flag.
Why this is the least workflow that protects the work: one code cell with its
red test first; the plan check runs inline.

Playbook: `skills/bee-planning/playbooks/bugfix.md` (class `bugfix`).

## Requirements (from CONTEXT.md)

- D1: plan a publish only for a skill that is missing, not active, or whose (relative path, sha256) set differs from the hub `manifest.resources`.
- D2: a hub read that skips keeps the publish item.
- D3: skillstation-principles D9 stays; zero planned publishes counts as success.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The principle publish items are planned with no hub read | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:111` | `fn publish_principle_items(engine: &Engine) -> Vec<Value> {` |
| 2 | The bee-deploying publish item is planned with no hub read | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:1602` | `plan.extend(publish_skillstation_items());` |
| 3 | The hub read to reuse exists | read | `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs:94` | `pub(crate) fn hub_show(run: Run, id: &str) -> Result<Hub, HubErr> {` |
| 4 | Apply gates `after_publish` items on a failed publish only | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:603` | `if item["after_publish"] == true {` |
| 5 | The hub digest is the raw sha256 of the file | ran | `skillstation skill show bee-principle-red-before-green --json` | `sha256:91da4c5a27cf31a64ca00418db3de1e3f103004206d3b750cf9aa63b17723400` |

## Discovery

The real hub prints `manifest.resources[]` with `path`
(`skills/default/<id>/<rel>`) and `digest` (`sha256:<hex>`). `sha256sum` of
`.claude/skills/bee-principle-red-before-green/SKILL.md` printed the same hex;
the three `bee-deploying` files matched too.

## Approach

Recommended path: extend `Hub` in `skillstation_group.rs` with the resource
set; in `plan.rs`, filter both publish lists through one helper that walks the
engine skill with the claude render and compares sets. A `HubErr::Skip` keeps
the item (D2). Apply does not change (D3).

Rejected: compare only the `content` field (misses `bee-deploying`
references); compare the current `.claude/skills` folder (stale before the
same run syncs it).

Risk map:
- a hub read per skill slows the plan / LOW / Lands in ssps-1 / one `skill show` per skill replaces a commit and an activate.
- a re-run of the switch skips the removals / MEDIUM / Lands in ssps-1 / a test plans the switch with every skill active and equal and sees the removals and `set_principles_hub`.

Waves: ssps-1.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ssps-1 changes onboard planning with a red test first."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride ssps-1; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"ssps-1 updates the one concept paragraph."},
    {"stage":"knowledge-consolidation","classification":"not-applicable","role":"writer","reason":"ssps-1 owns the concept edit and delivery.md."},
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
| 1 | onboard plans a publish only for a missing, inactive or changed skill | Owner report 2026-11-14 | `scripts/install.sh --source . -d . -y --skillstation` on beehive plans no `publish_skillstation` item | Fast re-runs of the switch |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ssps-1 | Skip the hub publish of a skill already active with the same files | `packages/bee-rs/crates/bee/src/onboard/plan.rs`, `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs`, `docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md`, `docs/history/skillstation-publish-skip/delivery.md` | — | a re-run of `--skillstation` on a served hub plans no publish | `cargo test … --bin bee -- skillstation onboard` green |

```json
[
  {
    "id": "ssps-1",
    "feature": "skillstation-publish-skip",
    "title": "Skip the hub publish of a skill already active with the same files",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "30aa7a86-a74c-4c42-b157-c9cb74fab022"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/history/skillstation-publish-skip/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-publish-skip/CONTEXT.md",
      "docs/history/skillstation-publish-skip/plan.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/onboard/render.rs"
    ],
    "action": "RED FIRST: in the onboard plan tests (find the existing --skillstation switch tests that call set_test_hub), add a test where the injected hub answers `skill show <id> --json` for bee-deploying and for every bee-principle-* of the test engine with manifest.status active and manifest.resources whose path is `skills/default/<id>/<rel>` and digest `sha256:<hex of the engine file rendered for claude>`. Plan with skillstation on and the claude runtime: assert zero publish_skillstation items, AND that the after_publish remove_skill items for the principles and the set_principles_hub item are still planned (D3). Add cases: one principle whose hub digest differs -> exactly that principle gets a publish item; one skill missing (runner error containing `skill not found`) -> it gets an item; one skill draft -> it gets an item; a hub runner that fails with any other reason -> every item kept (D2). Run the new test and watch it fail because 25-style items are planned today; record the red output. FIX: in skillstation_group.rs add the resource set to Hub (Vec of (rel path after the `<id>/` segment, hex digest without the `sha256:` prefix), parsed from manifest.resources; missing resources gives an empty set). In plan.rs add one helper that decides whether a skill needs a publish: hub_show via hub_runner(repo_root); Missing -> publish; Skip -> publish; status != active -> publish; else compare the sorted set from walk_skill_tree(engine.skills_root.join(id), Some(render claude)).files with the hub set; equal -> no item. Use it to filter publish_skillstation_items (bee-deploying, compared against the engine source rendered for claude) and publish_principle_items. Do not change apply.rs or the after_publish ordering. DOCS: in docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md Business Rules, state the publish filter in two or three sentences citing skillstation-publish-skip D1-D3 (never raw decision ids). Write docs/history/skillstation-publish-skip/delivery.md (what changed, the red and green test names, proof). One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard",
    "must_haves": {
      "truths": [
        "onboard --skillstation plans no publish_skillstation item for a skill active on the hub with the same per-file sha256 set",
        "a missing, non-active or changed skill still gets a publish_skillstation item",
        "a hub read that skips keeps every publish item",
        "with zero publish items the switch still plans the after_publish principle removals and set_principles_hub"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "the publish filter"}
      ],
      "key_links": [
        "plan.rs reads the hub through hub_show, not a copy",
        "the staged content is hashed with walk_skill_tree and the claude render"
      ],
      "prohibitions": [
        "No code comments",
        "No change to apply.rs",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": [
        "The helper name and where the hub set lives inside Hub"
      ]
    }
  }
]
```

## Test matrix

- Happy: every skill active with equal digests. Pass when no `publish_skillstation` item is planned and the removals and `set_principles_hub` are.
- Edge: one principle digest differs. Pass when exactly that principle is planned.
- Edge: one skill missing or draft. Pass when it is planned.
- Error: hub read fails. Pass when every publish item is kept.

## Open Questions

(none)

## Out of scope

Apply changes; a release (owner did not ask).
