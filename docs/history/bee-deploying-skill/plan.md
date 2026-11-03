---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: ship bee-deploying with bee, with a skillstation install option

## Summary

bee gains one more skill, `bee-deploying`: learn how a project deploys
once, write it down as `docs/DEPLOY.md` plus `scripts/deploy.sh` plus a
pointer in `CLAUDE.md`, and replay it on the next deploy or release
request. It ships like every other bee skill, so every onboarded project
gets it. `bee onboard --skillstation` additionally publishes that one
skill into the user's skillstation hub through the skillstation CLI; a
missing CLI is a loud skip, never a red onboard.

Mode: `standard` — 1 risk flag (external-systems: the skillstation CLI),
9 product files across `skills/` and `onboard/`.
Why this is the least workflow that protects the work: three disjoint
cells (skill, code, docs) each with its own proof; the only new runtime
behaviour is one opt-in flag whose failure path is a skip.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

Named deviation: the plan-step hat wave is not run. The shape is the
user's own two-option sentence turned into four decisions; every claim
below was opened at its line, and the one open design question (which
skills reach the hub) is locked in D3. Recorded here so the skip is
visible, not silent.

## Requirements (from CONTEXT.md)

- D1 (24175c6f): `skills/bee-deploying/` ships with bee; per-project copy is the default.
- D2 (6855564f): `bee onboard --skillstation` publishes `bee-deploying` to the hub via the skillstation CLI; one plan item `publish_skillstation`, only with the flag.
- D3 (464c2cca): missing binary or non-zero exit is a loud `skipped_skills` entry with target `skillstation`; onboard still `applied`; `HUB_SKILLS = ["bee-deploying"]`.
- D4 (7d06a37d): two branches, Replay and Learn; `LIVE` steps wait for a per-command yes; secrets never read; existing deploy paths are pointed at.
- Contract (f4db9d7b): the four onboard truths the tests pin.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Onboard flags are parsed by hand in one `else if` chain, where `--skillstation` joins | read | packages/bee-rs/crates/bee/src/onboard/mod.rs:210 | `} else if arg == "--global-skills" {` |
| 2 | Parsed args are copied field by field into plan `Options` | read | packages/bee-rs/crates/bee/src/onboard/mod.rs:418 | `global_skills: args.global_skills,` |
| 3 | `Options` is a plain struct of booleans; a new field is additive | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:44 | `pub global_skills: bool,` |
| 4 | The skill-sync items are computed in section 7 of `compute_plan`; the publish item appends after them | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:1244 | `let skill_sync = if opts.sync_skills {` |
| 5 | Apply keeps a loud skip list for skills it could not sync, which D3 reuses | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:335 | `let mut skipped_skills: Vec<Value> = Vec::new();` |
| 6 | Apply dispatches on `item.action`; the new arm sits beside `sync_skill` | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:664 | `"sync_skill" => {` |
| 7 | The arg-parse test lists every flag; `--skillstation` joins it | read | packages/bee-rs/crates/bee/src/onboard/tests.rs:163 | `let a = ok(&["--repo-root=/y", "--runtime=codex", "--no-claude-md", "--global-skills"]);` |
| 8 | The router's usage line names every onboard flag by hand | read | packages/bee-rs/crates/bee/src/router.rs:102 | `[--runtime R] [--claude-md|--no-claude-md] [--global-skills] [--force-downgrade] \` |
| 9 | `bee onboard --help` reads its flag table from the hand-declared registry payload | read | packages/bee-rs/crates/bee/src/generated/registry_payload.json:1 | `Install the skills into the user-level skill directory instead of per-project.` |
| 10 | The per-project skill roots come from one target list; the rendered `bee-deploying` lives under `.claude/skills` | read | packages/bee-rs/crates/bee/src/onboard/source.rs:184 | `pub fn skill_sync_targets(repo_root: &Path, global_skills: bool) -> Vec<(String, PathBuf)> {` |
| 11 | Every `skills/bee-*` skill needs 3 opening briefs and 2 near misses in the trigger fixture | read | packages/bee-rs/crates/bee/tests/skill_triggers.rs:202 | `if opened < MIN_EXPECT {` |
| 12 | `skillstation skill commit <folder> --yes --json` from a non-hub folder commits a new skill and prints a JSON status | ran | skillstation skill commit /mnt/data/Projects/goglbe/beehive/.bee/tmp/deploy-runbook --yes --json | `"status":"applied"` |

## Discovery

Claims 1-11 were opened in the main checkout at the lines named. Claim 12
is the real CLI run that put the hand-authored `deploy-runbook` into the
user's hub on 2026-10-30; the activate step printed a bound `cli` confirm
line (`skillstation skill confirm --proposal ... --yes --workspace ... --json`)
which the apply arm re-runs verbatim.

## Approach

Three disjoint cells, one wave:

- dep-1 writes the skill and its two reference templates, and adds its
  trigger-fixture cases.
- dep-2 adds the flag end to end: parse, `Options`, plan item, apply arm
  with the loud skip, usage line, registry entry, tests.
- dep-3 names the flag in every doc home that names `--global-skills`,
  and the onboarding knowledge concept.

After the wave the leader runs `bee dev regen` once in the worktree so the
plugin skill trees and the release manifest carry the new skill, then the
declared test command.

Rejected alternatives:
- Publish every bee skill to the hub: workflow skills outside an onboarded project misroute (D3).
- Make skillstation the default: most hosts have no hub (D2).
- Write hub files by hand: the hub is owned by the skillstation binary and its journal (D2).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Apply arm spawning an external binary | MEDIUM — a hang or a non-zero exit must not red the onboard | dep-2 | tests: binary absent → `skipped_skills` entry, status `applied`; no flag → no item |
| Skill routing | LOW — a bad description misroutes deploy asks | dep-1 | `skill_triggers` fixture green |
| Docs parity | LOW | dep-3 | every home that names `--global-skills` also names `--skillstation` |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust changes in onboard"},
    {"stage": "tests", "classification": "conditional", "role": "test", "condition": "a code cell's worker asks for a test repair", "reason": "red-first tests ride the code cell"},
    {"stage": "doctrine", "classification": "required", "role": "docs", "reason": "the skill itself, its templates, and the doc homes of the flag"},
    {"stage": "lookup", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of a file outside its cell", "reason": "read-only gathers"},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "a narrow fact lookup during execution", "reason": "cheap reader"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "no free-form generation stage"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes bee-reviewing", "reason": "independent review stays user-invoked"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "standard lane; hat wave skipped with a recorded reason"},
    {"stage": "plan-hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "hat wave skipped, see Summary"},
    {"stage": "plan-hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "hat wave skipped, see Summary"},
    {"stage": "plan-hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "hat wave skipped, see Summary"},
    {"stage": "plan-hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "the leader plans in-session"},
    {"stage": "supervise", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes: precedent for every part"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "no rendered UI in this feature"}
  ]
}
```

## Shape

One slice, three cells, all in one wave (disjoint files).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dep-1 | Add the bee-deploying skill with its two templates and trigger cases | `skills/bee-deploying/SKILL.md`, `skills/bee-deploying/references/DEPLOY.template.md`, `skills/bee-deploying/references/deploy.sh.template`, `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` | — | "deploy this project" routes to bee-deploying; the skill walks Learn or Replay | `cargo test -p bee --test skill_triggers` |
| dep-2 | Add bee onboard --skillstation that publishes bee-deploying to the hub with a loud skip | `packages/bee-rs/crates/bee/src/onboard/mod.rs`, `packages/bee-rs/crates/bee/src/onboard/plan.rs`, `packages/bee-rs/crates/bee/src/onboard/apply.rs`, `packages/bee-rs/crates/bee/src/onboard/tests.rs`, `packages/bee-rs/crates/bee/src/router.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | — | `bee onboard --skillstation --json` plans one `publish_skillstation` item; without the CLI the apply reports `applied` with one skipped entry | `cargo test -p bee onboard::` |
| dep-3 | Name --skillstation in every doc home of the onboard flags | `INSTALL.md`, `README.md`, `skills/bee-hive/references/onboarding.md`, `docs/product-description/maintenance/onboarding.md`, `docs/knowledge/areas/onboarding/host-project-artifacts.md`, `.bee/verify/verify-app/features/onboard.md` | — | the install docs show the two ways to install the deploy skill | `rg -c -e --skillstation` over the six homes |

```json
[
  {
    "id": "dep-1",
    "feature": "bee-deploying-skill",
    "title": "Add the bee-deploying skill with its two templates and trigger cases",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["24175c6f-5b13-4e03-b1e1-595e65d5f63f", "7d06a37d-92a1-4765-9e94-9ae65f3b48a6"],
    "files": ["skills/bee-deploying/SKILL.md", "skills/bee-deploying/references/DEPLOY.template.md", "skills/bee-deploying/references/deploy.sh.template", "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json"],
    "read_first": ["docs/history/bee-deploying-skill/CONTEXT.md", "docs/history/bee-deploying-skill/plan.md", "skills/bee-grooming/SKILL.md", "packages/bee-rs/crates/bee/tests/skill_triggers.rs"],
    "affects_skills": ["skills/bee-deploying/SKILL.md", "skills/bee-deploying/references/DEPLOY.template.md", "skills/bee-deploying/references/deploy.sh.template"],
    "affects_specs": [],
    "action": "Per D1 (store 24175c6f) and D4 (store 7d06a37d): create skills/bee-deploying/SKILL.md with front matter `name: bee-deploying` and a quoted one-line `description:` (the routing surface: learn, record and replay how one project deploys or releases; use when the user says deploy, release, ship, or asks to record how this project deploys; not for the bee repo's own release, which scripts/release.sh owns, and not for a server a project-specific deploy skill already owns). Body, in this order: Step 0 Find the record (docs/DEPLOY.md; a `## Deploy` or `## Release` section in CLAUDE.md or AGENTS.md naming a script; scripts/deploy.sh or scripts/release.sh) — found → Branch A Replay, none → Branch B Learn; a project-specific deploy skill or release script beats this skill and is pointed at, never duplicated. Branch A: read the record, note its date, flag drift against CI/container/package/platform files, run Preconditions (a red one stops), run Steps in order with every `LIVE` step shown and waiting for the user's yes for that exact command, run Verify, and fix the record in the same turn when reality disagreed. Branch B: B1 read before you ask (a table of files: CI workflows, Dockerfile and compose, package scripts and Makefile, platform files such as fly.toml or vercel.json, .env.example names only, README and docs, scripts/, top git tags; never open .env, *.pem, *.key, credentials*, secrets*); B2 ask one round for only the gaps (target, environments, trigger, secrets by name and place, manual steps, rollback, done check); B3 walk one deploy with the user, LIVE steps per-command yes, record each step as it succeeds, or write the record UNTESTED when the user wants no deploy now; B4 write docs/DEPLOY.md from references/DEPLOY.template.md, scripts/deploy.sh from references/deploy.sh.template for the automatable steps (set -euo pipefail; env and version as args; refuse a dirty tree unless --allow-dirty; LIVE commands print and ask Continue? unless --yes), and a short `## Deploy` section appended to CLAUDE.md or AGENTS.md pointing at the record; report the three paths; do not commit unless asked. Hard rules: LIVE never without a fresh yes; secret values never read or printed; never deploy a dirty or unpushed tree unless told for that run; the record is the single source of truth; report what ran, what was skipped, what is UNTESTED; close with a capture line (`bee decisions log` for a settled deploy fact when the project runs bee, otherwise a Gotchas line in the record). Keep the whole SKILL.md under 140 lines, plain English, no code comments in the templates beyond the `#!` line. references/DEPLOY.template.md: header with Recorded date, Status TESTED|UNTESTED, Last verified by; sections Target, Preconditions (commands plus required inputs by name), Steps table (# | Kind SAFE|LIVE | Command or action | Expected result), Verify, Rollback, Gotchas. references/deploy.sh.template: the bash skeleton described in B4 with a `live` function, the dirty-tree refusal, and == section echoes for preconditions, build, test, deploy, verify. Then add to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json a block for bee-deploying: 3 briefs with expect [\"bee-deploying\"] (one in Vietnamese, e.g. deploy dự án này lên production / ghi lại cách deploy để lần sau khỏi mò lại / how does this service get shipped? write it down and run it), and 2 near-miss briefs with expect [] or another skill and near bee-deploying (e.g. a request to release the bee repo itself, and a request to set up a CI pipeline from scratch). Mirror the existing block shape exactly.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
    "must_haves": {
      "truths": ["skills/bee-deploying/SKILL.md has a one-line description and the Replay and Learn branches with the LIVE rule", "both reference templates exist and the SKILL.md names them", "the trigger fixture holds 3 opening briefs and 2 near misses for bee-deploying and the skill_triggers test is green"],
      "artifacts": [{"path": "skills/bee-deploying/SKILL.md", "substantive": "Step 0, Branch A, Branch B with B1-B4, Hard rules"}, {"path": "skills/bee-deploying/references/DEPLOY.template.md", "substantive": "the record template"}, {"path": "skills/bee-deploying/references/deploy.sh.template", "substantive": "the script skeleton"}],
      "key_links": ["SKILL.md B4 names references/DEPLOY.template.md and references/deploy.sh.template", "skill-triggers.json names bee-deploying in expect and near"],
      "prohibitions": ["No edit to any other skill", "No edit to skill_triggers.rs", "No code comments in the templates beyond the #! line", "No hub or skillstation logic inside the skill text"]
    },
    "regen_obligation_ack": "wave-barrier",
    "trace": {"behavior_change": false}
  },
  {
    "id": "dep-2",
    "feature": "bee-deploying-skill",
    "title": "Add bee onboard --skillstation that publishes bee-deploying to the hub with a loud skip",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["6855564f-afea-4fd4-ae6d-9a24faec51c4", "464c2cca-c7a9-4893-aa3f-8fc94cfd2fd6", "f4db9d7b-dbbb-466f-b32f-a2a65051ba6a"],
    "files": ["packages/bee-rs/crates/bee/src/onboard/mod.rs", "packages/bee-rs/crates/bee/src/onboard/plan.rs", "packages/bee-rs/crates/bee/src/onboard/apply.rs", "packages/bee-rs/crates/bee/src/onboard/tests.rs", "packages/bee-rs/crates/bee/src/router.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json"],
    "read_first": ["docs/history/bee-deploying-skill/CONTEXT.md", "docs/history/bee-deploying-skill/plan.md", "packages/bee-rs/crates/bee/src/onboard/mod.rs", "packages/bee-rs/crates/bee/src/onboard/plan.rs", "packages/bee-rs/crates/bee/src/onboard/apply.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D2 (store 6855564f), D3 (store 464c2cca) and contract onboard-skillstation (store f4db9d7b). mod.rs: add `skillstation: bool` to Args (default false), parse `--skillstation` in the else-if chain beside `} else if arg == \"--global-skills\" {`, and copy it into Options beside `global_skills: args.global_skills,`. plan.rs: add `pub skillstation: bool` to Options; add `pub const HUB_SKILLS: [&str; 1] = [\"bee-deploying\"];`; in compute_plan right after the section-7 skill-sync items are extended (after `let skill_sync = if opts.sync_skills {` block), when `opts.skillstation && opts.sync_skills && skill_sync.blocked.is_none()`, push one item per HUB_SKILLS entry: {action: \"publish_skillstation\", skill: <name>, path: \".claude/skills/<name>\", target: \"skillstation\"}; the path is the repo-claude rendered root joined with the skill (take the segments from the `repo-claude` entry of skill_sync_targets, never a hard-coded string twice). apply.rs: add an arm `\"publish_skillstation\"` beside `\"sync_skill\" => {`: resolve the absolute folder (repo_root joined with item.path); if the folder is not a directory push a skipped_skills entry {skill, target: \"skillstation\", reason: \"rendered skill folder is absent — run the skill sync first\"} and continue; run `skillstation skill commit <folder> --yes --json` with std::process::Command; a spawn error (binary not on PATH) or a non-zero exit pushes {skill, target: \"skillstation\", reason: <\"skillstation is not on PATH\" or the first 200 chars of stderr/stdout>} and continues; on success parse stdout as JSON, and unless it reports `\"activated\": true` run `skillstation skill activate <skill> --json`, read its top-level `cli` string, split it on whitespace and run it (first token is the program); any failure there is the same loud skip with the step named; on full success push the item into applied with an added `hub_commit` field holding the commit hash when present. Never read, print or write any file under the hub; never pass --approve-content or --force. Keep every refusal reason one line. router.rs: add `[--skillstation]` to the onboard usage string beside `[--global-skills]`. generated/registry_payload.json: add a `skillstation` boolean property to the onboard command's parameters beside `global-skills`, description: \"Also publish the bee-deploying skill into the user's skillstation hub through the skillstation CLI (skill commit, then activate). Absent CLI or a failed call is a loud skipped entry, never a red onboard.\" (declared by hand in the same change, like the other onboard flags). tests.rs: extend parse_args_mirrors_the_engine_flag_set with `--skillstation` → true and default false; add a plan test: with skillstation true the plan holds exactly one publish_skillstation item for bee-deploying with target skillstation, and with it false none; add an apply test with the flag set and PATH pointed at an empty temp dir so the binary is absent: status applied, skipped_skills carries one entry with target skillstation and a reason naming PATH. Run the onboard tests. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee onboard::",
    "must_haves": {
      "truths": ["parse_args accepts --skillstation and defaults it to false", "compute_plan emits exactly one publish_skillstation item for bee-deploying only when the flag is set and sync_skills is on", "apply with the flag and no skillstation binary on PATH reports applied with one skipped_skills entry whose target is skillstation", "without the flag no item and no skip appear"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/onboard/apply.rs", "substantive": "the publish_skillstation arm with commit, activate, bound confirm, and the loud skip"}, {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "HUB_SKILLS and the publish item"}, {"path": "packages/bee-rs/crates/bee/src/onboard/tests.rs", "substantive": "the four contract truths as tests"}],
      "key_links": ["mod.rs copies args.skillstation into Options.skillstation", "router.rs usage names --skillstation", "registry_payload.json declares skillstation on onboard"],
      "prohibitions": ["No change to the sync_skill, refresh_legacy_global_skill or remove_skill arms", "No write under the skillstation hub by bee itself", "No --approve-content or --force on any skillstation call", "No code comments"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "dep-3",
    "feature": "bee-deploying-skill",
    "title": "Name --skillstation in every doc home of the onboard flags",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["24175c6f-5b13-4e03-b1e1-595e65d5f63f", "6855564f-afea-4fd4-ae6d-9a24faec51c4"],
    "files": ["INSTALL.md", "README.md", "skills/bee-hive/references/onboarding.md", "docs/product-description/maintenance/onboarding.md", "docs/knowledge/areas/onboarding/host-project-artifacts.md", ".bee/verify/verify-app/features/onboard.md"],
    "read_first": ["docs/history/bee-deploying-skill/CONTEXT.md", "docs/history/bee-deploying-skill/plan.md", "INSTALL.md", "docs/knowledge/areas/onboarding/host-project-artifacts.md"],
    "affects_skills": ["skills/bee-hive/references/onboarding.md"],
    "affects_specs": ["docs/knowledge/areas/onboarding/host-project-artifacts.md"],
    "action": "Per D1 (store 24175c6f) and D2 (store 6855564f). In each file, wherever `--global-skills` is listed as an onboard flag, add `--skillstation` beside it with one sentence: it also publishes the bee-deploying skill into the user's skillstation hub through the skillstation CLI (commit, then activate); without the CLI the onboard still succeeds and reports one skipped entry with target skillstation. INSTALL.md: a row in the flags table at the `--global-skills` row (PowerShell column: `-Skillstation` marked as not yet wired in install.ps1), and one short paragraph under the two-layers section naming the two ways to install the deploy skill: the per-project copy every onboard writes, or the hub with `--skillstation`. README.md: the `bee onboard` usage line gains `[--skillstation]`, and the 'Useful flags' sentence names it. skills/bee-hive/references/onboarding.md: one sentence after the `--global-skills` sentence. docs/product-description/maintenance/onboarding.md: add the flag to the argv list and one line to the 'reach outside the repo' paragraph. docs/knowledge/areas/onboarding/host-project-artifacts.md: extend the existing rendered-skills entry with the hub publish fact, its plan item name publish_skillstation, the HUB_SKILLS constant, and the loud-skip shape; Pointers name onboard/plan.rs and onboard/apply.rs. .bee/verify/verify-app/features/onboard.md: add a sub-feature line `onboard-skillstation` and a short 'How to get to it' bullet. Plain English, short sentences, no new headings beyond what each file already uses.",
    "verify": "rg -c -e --skillstation INSTALL.md README.md skills/bee-hive/references/onboarding.md docs/product-description/maintenance/onboarding.md docs/knowledge/areas/onboarding/host-project-artifacts.md .bee/verify/verify-app/features/onboard.md",
    "must_haves": {
      "truths": ["every file that lists --global-skills as an onboard flag also lists --skillstation", "the install docs state the two ways to install the deploy skill", "the onboarding knowledge concept records the publish item and the loud skip"],
      "artifacts": [{"path": "INSTALL.md", "substantive": "flag row plus the two-ways paragraph"}, {"path": "docs/knowledge/areas/onboarding/host-project-artifacts.md", "substantive": "hub publish fact with pointers"}],
      "key_links": ["README.md usage line names --skillstation", "onboard.md feature map lists onboard-skillstation"],
      "prohibitions": ["No edit to scripts/install.sh or install.ps1", "No change to any line that does not concern onboard flags or the deploy skill"]
    },
    "regen_obligation_ack": "wave-barrier",
    "trace": {"behavior_change": false}
  }
]
```
