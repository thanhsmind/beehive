---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: A set-up ask for lint gets steps, not a refusal

## Summary

bee's notices about standard commands and static checks name the verb that
writes them, `bee config set --key commands.test --value '<JSON list>'`. The
bee-hive skill gains a short procedure an agent follows when the user asks to
set up lint, type check or tests. An agent then sets the check up instead of
saying it has no permission.

Playbook: `skills/bee-planning/playbooks/bugfix.md` (situation bug-fix).

Mode: `standard` — 2 risk flags: public contract (notice text every host
reads), multi-domain (Rust code, skills, knowledge).

## Requirements (from CONTEXT.md)

- D1: every commands and static-check notice names `bee config set --key commands.`.
- D2: a set-up ask is the confirmation; never reply that permission is missing.
- D3: onboarding.md "Set up the project's checks" procedure, loaded from SKILL.md on a set-up ask.
- D4: tests pin the notice text.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The how-to notice names no write verb | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:373 | `Record the user's answer for each check as a decision. Never write an unconfirmed value.` |
| 2 | The empty-commands notice says write the JSON file | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:404 | `write them to .bee/config.json` |
| 3 | The status warning still names the retired verify key | read | packages/bee-rs/crates/bee/src/verbs/status_full/build.rs:149 | `setup/start/test/verify into .bee/config.json` |
| 4 | bee config set writes commands.test with no refusal | ran | bee config set --key commands.test --value '["npm run lint","npm test"]' | `config set: commands.test = ["npm run lint","npm test"]` |
| 5 | The write guard lets an Edit of .bee/config.json commands through | ran | bee hook write-guard with an Edit payload | exit 0 |
| 6 | The onboarding reference loads only for onboarding itself | read | skills/bee-hive/SKILL.md:128 | `ONLY when onboarding is in question` |

## Approach

Three cells. lsg-1 changes the notice text. lsg-3 is the separate tester for
lsg-1: it adds the tests that pin D4. lsg-2 writes the skill procedure and the
knowledge concept; its files do not overlap lsg-1, so it runs beside it.

Rejected: a new `bee setup lint` verb (detecting and installing a linter is
judgment per stack, and `bee config set` already writes the value); dropping
the AGENTS.md CLI-only rule (it guards the state files, and naming the verb
removes the conflict).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "lsg-1 rewrites three Rust notice texts."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "lsg-3 is the separate tester for lsg-1 and pins D4."},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "lsg-2 edits the bee-hive skill, one concept, and runs the regen chain."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Notice text and docs only; the tester cell is the independent check. Independent review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Every claim row was opened or ran this session."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The Approach names the rejected paths; naming an existing verb is the smallest change."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Text change; no behavior of a verb changes."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "No door, guard or state write changes."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The leader read the anchors directly."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every cell has a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "lsg-2 runs as docs."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Standard lane; no high-risk advisor consult."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| An existing test pins the old notice wording | lsg-1 runs the notices and status_full tests and updates a pinned string |
| The skill procedure tells the agent to edit a governed key | lsg-2 names only `commands.*` and lists the governed keys as the user's |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| lsg-1 | Name bee config set in every standard-commands notice | templates.rs, notices.rs, status_full build.rs | — | `bee onboard --json` and `bee status` notices name `bee config set --key commands.test` | notices and status_full tests |
| lsg-3 | Pin the write verb in the standard-commands notices | notices.rs tests, status_full tests | lsg-1 | tests fail if a notice drops the verb | notices and status_full tests |
| lsg-2 | Give the agent a set-up procedure for project checks | bee-hive SKILL.md, onboarding.md, host-project-artifacts concept, rendered copies | — | the skill says what to do on "set up lint" | knowledge check, release-manifest check, skill_triggers |

```json
[
{"id":"lsg-1","feature":"lint-setup-guidance","title":"Name bee config set in every standard-commands notice","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- notices status_full",
"must_haves":{"truths":["STATIC_CHECK_HOW_TO_OFFER names bee config set --key commands.test and says a user request to set up the checks is the confirmation","both No standard commands recorded notices in commands_notices name bee config set --key commands. instead of telling the agent to write .bee/config.json","the status_full empty-commands warning names bee config set --key commands. and no longer names the retired verify key"]},
"decisions":["d509d105-c499-4b39-8469-99f12ff77b5b","1bc79e0e-7688-49db-9fe0-160981b4f7cd"],
"files":["packages/bee-rs/crates/bee/src/onboard/templates.rs","packages/bee-rs/crates/bee/src/onboard/notices.rs","packages/bee-rs/crates/bee/src/verbs/status_full/build.rs","packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"],
"read_first":["docs/history/lint-setup-guidance/CONTEXT.md","packages/bee-rs/crates/bee/src/onboard/templates.rs","packages/bee-rs/crates/bee/src/onboard/notices.rs"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/onboarding/host-project-artifacts.md"],
"action":"Implement CONTEXT.md D1 and D2 in notice text only; no logic change. (1) templates.rs STATIC_CHECK_HOW_TO_OFFER: keep its current sentences and add that when the user confirms, the agent writes the list itself with `bee config set --key commands.test --value '<JSON list>'` (checks first, then the existing test entries), that a user request to set up these checks is that confirmation, and that `commands` is the agent's to write, never a missing permission. (2) notices.rs, both No standard commands recorded strings: replace 'write ... to .bee/config.json `commands`' with writing each confirmed value through `bee config set --key commands.<key> --value '<value>'` (commands.test takes a JSON list). Keep the never-write-unconfirmed rule and the D3 citation. (3) status_full/build.rs empty-commands warning: name setup/start/test (drop the retired verify key) and `bee config set --key commands.<key>`. Keep the warning's first words 'No standard commands recorded'. Update any existing test that pins an old full string. No new tests (lsg-3 owns them). No code comments. Proof: the cell verify; quote the counts."},
{"id":"lsg-3","feature":"lint-setup-guidance","title":"Pin the write verb in the standard-commands notices","lane":"standard","role":"test","deps":["lsg-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- notices status_full",
"must_haves":{"truths":["a test asserts every commands_notices notice that asks for commands or offers a static check contains bee config set --key commands.","a test asserts no such notice contains write them to .bee/config.json","a test asserts the status_full empty-commands warning contains bee config set --key commands. and not setup/start/test/verify"]},
"decisions":["af69659c-64cd-40d2-8bbc-d8fec0e53d65","d509d105-c499-4b39-8469-99f12ff77b5b"],
"files":["packages/bee-rs/crates/bee/src/onboard/notices.rs","packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"],
"read_first":["docs/history/lint-setup-guidance/CONTEXT.md","packages/bee-rs/crates/bee/src/onboard/notices.rs"],
"affects_skills":[],"affects_specs":[],
"action":"Independently check lsg-1 and pin CONTEXT.md D4 with behavior tests. In the notices.rs test module, add tests over commands_notices on temp repos for three cases: no commands and no detected candidates; no commands with detected candidates and a static check (package.json with a lint script); a commands.test list missing a detected static check. Each returned notice must contain `bee config set --key commands.` and none may contain `to .bee/config.json`. In status_full/tests.rs, assert the empty-commands warning contains `bee config set --key commands.` and does not contain `verify`. Prove each new test fails against the pre-lsg-1 text (state how: e.g. git stash of lsg-1's hunks or reasoning from the old string) and passes now. No code comments. Proof: the cell verify; quote the counts and the new test names."},
{"id":"lsg-2","feature":"lint-setup-guidance","title":"Give the agent a set-up procedure for project checks","lane":"standard","role":"docs","deps":[],
"verify":".bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test skill_triggers",
"must_haves":{"truths":["skills/bee-hive/references/onboarding.md has a Set up the project's checks section that tells the agent to write commands.test with bee config set and never to reply that permission is missing","skills/bee-hive/SKILL.md loads that section when the user asks to set up lint, type check, tests or verify commands","docs/knowledge/areas/onboarding/host-project-artifacts.md says the static-check notices name bee config set and cites lint-setup-guidance D1 and D2","the rendered skill trees match skills/ (release-manifest --check clean)"]},
"decisions":["d509d105-c499-4b39-8469-99f12ff77b5b","1bc79e0e-7688-49db-9fe0-160981b4f7cd","72a0f753-00ef-4e04-9e06-c12c98e119f2"],
"files":["skills/bee-hive/SKILL.md","skills/bee-hive/references/onboarding.md","docs/knowledge/areas/onboarding/host-project-artifacts.md",".claude/skills/*",".agents/skills/*",".opencode/skills/*",".claude-plugin/skills/*",".codex-plugin/skills/*",".bee/onboarding.json","skills-lock.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/lint-setup-guidance/CONTEXT.md","skills/bee-hive/references/onboarding.md","skills/bee-hive/SKILL.md","docs/knowledge/areas/onboarding/host-project-artifacts.md"],
"affects_skills":["skills/bee-hive/SKILL.md","skills/bee-hive/references/onboarding.md"],"affects_specs":["docs/knowledge/areas/onboarding/host-project-artifacts.md"],
"action":"Implement CONTEXT.md D2 and D3 in docs. (1) skills/bee-hive/references/onboarding.md: add a section `## Set up the project's checks` after Workspace protocol. When: the user asks to set up lint, type check, tests, or the verify or CI commands, at any time. Steps: 1. Run `.bee/bin/bee onboard --repo-root . --json` and read its notices for detected checks and candidates; read `bee config explain --key commands.test` for what is recorded. 2. With a check found, run it once and tell the user green or the finding count. 3. With no linter in the project, name the stack's usual linter (eslint, ruff, clippy, golangci-lint, phpcs and so on), and after the user agrees add it to the project as a dev dependency with its config through the normal bee chain for a code change. 4. Write the list yourself: `.bee/bin/bee config set --key commands.test --value '[\"<lint>\",\"<typecheck>\",\"<tests>\"]'` — checks first, then the existing test entries, keeping every entry already there. The user's set-up request is the confirmation (lint-setup-guidance D2). 5. Log the answer with `bee decisions log`. Rules: `commands` is the agent's to write; never answer that permission is missing; never hand-edit `.bee/config.json`; only team, herding.agents, herding.agent_command and hooks are the user's to edit. (2) skills/bee-hive/SKILL.md: in the reference table row for references/onboarding.md, add that it also loads when the user asks to set up lint, type check, tests or verify commands (section Set up the project's checks); keep the line short. (3) host-project-artifacts.md section 'Static checks are offered as members of `commands.test`': add that the notices name `bee config set --key commands.test` and that a user set-up request is the confirmation, citing lint-setup-guidance D1 and D2 (history D-numbers, never raw ids), and point to the new onboarding.md section. Run `bee knowledge check` after. (4) Run `bee dev regen` so the rendered skill trees, onboarding ledger, skills lock and release manifest follow; restore any hub-served skill under .claude/skills that regen re-rendered and that this cell does not own. Plain English, no code comments. Proof: the cell verify; quote the counts."}
]
```
