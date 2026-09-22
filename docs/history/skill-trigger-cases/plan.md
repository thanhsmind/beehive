---
artifact_contract: bee-plan/v2
mode: small
---

# skill-trigger-cases — plan

Route: class `feature` · lane `small` · flags none · product files 3.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

A small lane writes no plan by default; this page exists because `bee gate --preview` parses the cell packet from `plan.md`, so it carries the merged-gate evidence and the two cells, nothing more.

## Summary

Every bee skill gets a set of test briefs: at least three requests that should open it and two look-alikes that must not. A test in the normal suite fails when a skill has too few, when a brief cheats by naming its skill, or when a case names a skill that does not exist. A separate, opt-in run asks a real agent the same briefs and reports how often it picks right. The skill-writing checklist tells authors to add cases for a new skill.

Mode: `small` — 0 risk flags.
Why this is the least workflow that protects the work: two files of test and fixture plus one checklist row; nothing runtime changes.

## Requirements (from CONTEXT.md)

- D1: scope is every `skills/bee-*` skill that is not `bee-principle-*` (20 today: 34 skills minus 14 principles).
- D2: one fixture, `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`.
- D3: a case is `brief` + `expect` (skills that must open, `[]` for none) + `near` (one skill that must not); per skill ≥3 expect cases and ≥2 near cases.
- D4: a brief never contains the slug or `/slash` form of a skill it expects or nears.
- D5: the model-free test runs in `commands.test` and fails by name.
- D6: the real-agent eval is opt-in, never in `commands.test` or CI; 3 runs, pass at 2 of 3; command from one env var; skipped with a printed reason when unset.
- D7: `skills/bee-writing-skills/SKILL.md` checklist gains one row.

## Load-bearing claims

Labels: `read` = opened at that line, `ran` = executed and output held. Match rule: verbatim byte substring of the anchored line(s).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Host packaging copies only `bee-` prefixed directories, so a fixture under `tests/` never ships | read | `packages/bee-rs/crates/bee/src/onboard/render.rs:381` | `read_dir_sorted(root).into_iter().filter(\|e\| e.name.starts_with("bee-")).collect()` |
| 2 | Integration tests of this crate import nothing from it and read the tree from disk | read | `packages/bee-rs/crates/bee/tests/principle_index_parity.rs:32` | `// Shape, deliberately: pure filesystem, std only, and NOTHING imported from the` |
| 3 | `serde_json` is a crate dependency, available to integration tests | read | `packages/bee-rs/crates/bee/Cargo.toml:11` | `serde_json.workspace = true` |
| 4 | The declared suite runs every integration test under `tests/` | read | `.bee/config.json:11` | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| 5 | The checklist row D7 extends sits in the SKILL.md checklist | read | `skills/bee-writing-skills/SKILL.md:38` | `one purpose clause, then "Use when..." triggers` |
| 6 | 34 skill directories, 14 of them principles | ran | `ls -d skills/*/ \| wc -l; ls -d skills/bee-principle-*/ \| wc -l` | `34` / `14` |
| 7 | A skill description is either a quoted one-liner or a `>-` folded block | read | `skills/bee-how/SKILL.md:3` | `description: >-` |

## Discovery

Read `principle_index_parity.rs:1-50` and `pointer_integrity.rs:1-40` (the fence shape: std only, `CARGO_MANIFEST_DIR` ancestors to the repo root, findings by name), `onboard/render.rs:377-382` (skill enumeration), `bee-writing-skills/SKILL.md:35-49` (the checklist). Ran the two `ls` counts. Finding: a fixture under `tests/fixtures/` is invisible to packaging; the crate has no lib target, so the test parses front matter itself.

## Approach

Recommended path: one integration test file `tests/skill_triggers.rs` holding the structural test (D3–D5) and one `#[ignore]` eval test (D6) that reads `BEE_SKILL_TRIGGER_EVAL` as the agent command; one fixture with the first case set for all 20 skills; one checklist row (D7). SMALLER PATH: a fixture without the eval would leave descriptions unmeasured (D6 is locked); a per-skill fixture ships into hosts (claim 1). PASS.

Waves: skt-1 and skt-2 run in parallel — no shared file.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"test-and-fixture","classification":"required","role":"test","reason":"skt-1 authors the fence, the ignored eval and the first case set."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"skt-2 adds the checklist row and runs the regen chain for the rendered skill copies."},
    {"stage":"implementation","classification":"not-applicable","role":"code","reason":"No runtime code changes."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"This page is the plan; the leader wrote it."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery was targeted reads by the leader."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left open."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked and was not requested."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Small lane, no consult owed."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane: no hat wave."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."}
  ]
}
```

## Shape

Two cells, one slice.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| skt-1 | Fence every skill description with opening briefs and near misses | `packages/bee-rs/crates/bee/tests/skill_triggers.rs` (new), `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` (new) | — | the suite fails by name when a skill has too few cases or a brief names its skill; `BEE_SKILL_TRIGGER_EVAL="claude -p" cargo test … -- --ignored` prints a per-skill pass count | `cargo test --test skill_triggers` green, red-first on a deliberately short fixture |
| skt-2 | Tell skill authors to add trigger cases | `skills/bee-writing-skills/SKILL.md`, `docs/history/codex-harness-hardening/release-manifest.json` | — | the SKILL.md checklist has one row pointing at the fixture and the suite rule | `bee dev release-manifest --check` green |

```json
[
  {
    "id": "skt-1",
    "feature": "skill-trigger-cases",
    "title": "Fence every skill description with opening briefs and near misses",
    "lane": "small",
    "role": "test",
    "deps": [],
    "decisions": ["21fb8d2d-b8d9-4b11-b390-2b42e0e13627", "39b02fdb-70c5-44a6-b154-4695940e233a", "516b603e-6110-4b08-adb4-5d9662d7e452", "464b09b9-7318-4dd1-8413-5ccad5a0672d"],
    "files": [
      "packages/bee-rs/crates/bee/tests/skill_triggers.rs",
      "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json"
    ],
    "read_first": [
      "docs/history/skill-trigger-cases/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/principle_index_parity.rs",
      "packages/bee-rs/crates/bee/tests/pointer_integrity.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Write packages/bee-rs/crates/bee/tests/skill_triggers.rs in the shape of principle_index_parity.rs (std + serde_json only, repo root from CARGO_MANIFEST_DIR ancestors, findings collected then reported by name). It reads every skills/bee-*/SKILL.md, parses the front matter between the first two `---` lines by hand, and takes `description:` in both spellings — a quoted one-liner and a `>-` folded block (join the indented continuation lines with one space). Scope = every skills/bee-* directory whose name does not start with bee-principle- (per D1). Fixture packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json is a JSON array of objects `{\"brief\": string, \"expect\": [string], \"near\": string}`; `near` may be absent on a case that only expects (per D3). Test `skill_triggers_fixture_is_complete` fails, naming each offender, when: a skill in scope has fewer than 3 cases whose expect contains it, or fewer than 2 whose near equals it; a case names a skill (in expect or near) with no skills/<name>/SKILL.md; a brief contains, case-insensitively, the slug `bee-xyz` or `/bee-xyz` of any skill it expects or nears (per D4); two cases share a brief after trimming. RED FIRST: write the fixture with only one skill's cases, run the test, watch it fail naming the other 19, then fill it. Fill the fixture for all 20 skills: at least 3 briefs each in the user's own words (mix English and short Vietnamese asks, since this repo's users write both), and at least 2 near misses each whose expect names the skill that should open instead (or [] for a plain question that opens nothing) — the near misses must be real confusions: bee-how vs bee-why vs bee-teach; bee-researching vs bee-wayfinding; bee-verifying vs bee-verify-upkeep; bee-reviewing vs a plain code question; bee-shaping vs bee-planning; bee-capturing vs bee-evolving; bee-grooming vs a bug report; bee-herding vs bee-swarming vs bee-herdr; bee-unslop vs bee-technical-writing. Then the eval: `#[ignore] fn skill_triggers_real_agent_eval` reads env BEE_SKILL_TRIGGER_EVAL; unset → print `skipped: BEE_SKILL_TRIGGER_EVAL is unset (set it to an agent command such as `claude -p` to run the paid eval)` and return; set → for each case build the prompt `You route user requests to skills. Skills (name: description):\\n<list of in-scope name: description>\\n\\nRequest: <brief>\\n\\nAnswer with only JSON: {\"skills\": [...]} naming the skills that should open, or [] for none.`, run the command via `sh -c \"$CMD\"` with the prompt on stdin 3 times, parse the first JSON object in stdout, and score a run right when no near skill appears, an expect [] case returns an empty list, and otherwise at least one expected skill appears; a case passes at 2 of 3; print one line per skill `<skill>: <passed>/<cases>` and a final total, and fail the test only when a case passed 0 of 3 (per D6). NO COMMENTS of any form in the code (no_code_comments is on; the ratchet reds the suite if the count rises). One commit, subject in imperative mood, trailer `cell: skt-1`.",
    "must_haves": {
      "truths": [
        "cargo test --test skill_triggers is green on the filled fixture",
        "removing all cases for one skill makes the test fail naming that skill",
        "a brief containing its own skill slug makes the test fail naming the brief",
        "the ignored eval prints its skip reason when BEE_SKILL_TRIGGER_EVAL is unset"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/skill_triggers.rs", "substantive": "the structural fence and the ignored eval"},
        {"path": "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json", "substantive": "at least 100 cases covering all 20 in-scope skills with real near misses"}
      ],
      "key_links": [
        "docs/history/skill-trigger-cases/CONTEXT.md D1-D6 are the locked rules"
      ],
      "prohibitions": [
        "No file under skills/",
        "No model call inside the non-ignored test",
        "No code comment of any form"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers"
  },
  {
    "id": "skt-2",
    "feature": "skill-trigger-cases",
    "title": "Tell skill authors to add trigger cases",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["516b603e-6110-4b08-adb4-5d9662d7e452"],
    "files": [
      "skills/bee-writing-skills/SKILL.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/skill-trigger-cases/CONTEXT.md",
      "skills/bee-writing-skills/SKILL.md"
    ],
    "affects_skills": ["skills/bee-writing-skills/SKILL.md"],
    "affects_specs": [],
    "action": "In skills/bee-writing-skills/SKILL.md, in the `## SKILL.md checklist (bee conventions)` list, directly after the `description:` row (the line starting `- [ ] \\`description\\`: one purpose clause, then \"Use when...\" triggers`), add one row: `- [ ] Trigger cases: a new or renamed skill adds at least 3 briefs that open it and 2 near misses that must not to \\`packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json\\` (\\`brief\\`/\\`expect\\`/\\`near\\`; a brief never names its skill) — the suite's \\`skill_triggers\\` test refuses a skill with none; \\`BEE_SKILL_TRIGGER_EVAL=\"claude -p\" cargo test --test skill_triggers -- --ignored\\` asks a real agent (per skill-trigger-cases D7)`. Change nothing else in the file. Then run `.bee/bin/bee dev regen` so the rendered copies under .claude/, .agents/ and .opencode/ match, and commit the refreshed copies plus docs/history/codex-harness-hardening/release-manifest.json with the rest. Prove: `.bee/bin/bee dev release-manifest --check` green. One commit, subject in imperative mood, trailer `cell: skt-2`.",
    "must_haves": {
      "truths": [
        "the checklist has one trigger-cases row naming the fixture path and the eval command",
        "bee dev release-manifest --check is green after regen"
      ],
      "artifacts": [
        {"path": "skills/bee-writing-skills/SKILL.md", "substantive": "the trigger-cases checklist row"}
      ],
      "key_links": [
        "skt-1's fixture path is the path the row names"
      ],
      "prohibitions": [
        "No hand edit under .claude/, .agents/ or .opencode/",
        "No other change to the skill"
      ]
    },
    "verify": ".bee/bin/bee dev release-manifest --check"
  }
]
```

## Test matrix

| case | probe | pass when |
|---|---|---|
| happy | filled fixture, all 20 skills | `skill_triggers_fixture_is_complete` green |
| edge | a skill with exactly 3 expect and 2 near | green |
| edge | `>-` folded description and quoted description both parsed | both skills listed in scope with non-empty descriptions |
| error | one skill's cases deleted | red, naming that skill |
| error | brief `use /bee-how to explain the hook` expecting bee-how | red, naming the brief |
| error | a case naming `bee-nope` | red, naming `bee-nope` |
| eval | env unset | ignored test prints the skip line and passes |

## Open Questions

(none)

## Out of scope

- Cases for principle skills; description rewrites based on eval data (CONTEXT Deferred Ideas).
