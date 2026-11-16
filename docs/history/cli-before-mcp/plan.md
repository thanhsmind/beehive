---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Agents reach Skillstation and mcp-gateway through the CLI first

## Summary

bee tells agents to use the `skillstation` and `mcp-gateway` CLIs first and
the MCP tools as the fallback. One new doctrine rule holds the mapping. The
alias SKILL.md and the hub-mode principle header name
`skillstation skill show <id> --json` before `skill_get`.

Playbook: `skills/bee-planning/playbooks/feature/feature.md` (situation feature).

Mode: `standard` — 2 risk flags: public contract (doctrine and alias text
every host reads), multi-domain (Rust code, doctrine, knowledge).

Feature playbook steps: step 1 mapped inline (four render sites, one rg);
architect skipped: text change on the waggledance-cli-first shape; step 3
n/a: no event-order flow; step 8 n/a: no formal model covers these texts.
Throughput: blocking first steps n/a (gate only); independent workstreams:
Rust texts (cbm-1) and doctrine and knowledge (cbm-3) share no file; shared
mutable state n/a; smallest safe decomposition: two writers, two checkers.

## Requirements (from CONTEXT.md)

- D1: rule agents-cli-before-mcp in packages/bee/LEADER.md holds the mapping.
- D2: mcp-gateway tools, describe, call; fallback triggers; no re-send on outcome_unknown.
- D3: skillstation skill show --json for active skills; skill_get for relative files, other states, remote hub, CLI failure.
- D4: alias render and principle header name the CLI first; playbook pointer and agent tools line keep skill_get.
- D5: tests pin the order.
- D6: the mcp-gateway skill text went to its own leader; aliases re-render from main after merge.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The alias names only skill_get | read | packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs:146 | `Call the skillstation MCP tool` |
| 2 | The principle header names only skill_get | read | packages/bee-rs/crates/bee/src/principles.rs:139 | `Load each one with the skillstation MCP tool skill_get, the slug as skill_id.` |
| 3 | The playbook pointer opens a relative file | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:761 | `open this file under its local.path` |
| 4 | skill show returns content and manifest | ran | skillstation skill show bee-how --json | `schema_version status summary items suggested_actions warnings error manifest content` |
| 5 | The cross-project guard refuses a hub folder read | ran | ls of the hub folder skills/default/bee-how | `bee cross-project guard` |
| 6 | mcp-gateway serves tools from the shell | ran | mcp-gateway tools skill | exit 0, one scored line |

## Approach

cbm-1 changes two Rust texts. cbm-2 is its separate tester and pins D5.
cbm-3 writes the doctrine rule, its index row and the concept text, and runs
the regen chain; its files do not overlap cbm-1. cbm-4 checks cbm-3.

Rejected: opening the hub folder to reads (opens a whole foreign repo);
`skill checkout` for relative files (skillstation-aliases D5); dropping
`mcp__skillstation__skill_get` from agent files (reader roles have no shell).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "cbm-1 rewrites two Rust texts."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "cbm-2 and cbm-4 are the separate checkers."},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "cbm-3 edits the doctrine, the rule index, one concept, and runs regen."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Text change; the tester cells are the independent check. Review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Every claim row was opened or ran this session."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The Approach names the rejected paths; the shape copies waggledance-cli-first."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Text change; MCP stays as the fallback."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "No door, guard or state write changes."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The leader read the anchors directly."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every cell has a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "cbm-3 runs as docs."},
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
| A test pins the old alias or header text | cbm-1 runs the principles, skillstation and onboard tests |
| The rule index parity test reds on a new marker | cbm-3 adds the index row and runs rule_index_parity |
| An agent reads hub files and hits the guard | D3 sends relative files to skill_get |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cbm-1 | Name the skillstation CLI first in the alias and principle header | skillstation_group.rs, principles.rs | — | alias and orient header name `skillstation skill show <id> --json` first | principles, skillstation, onboard tests |
| cbm-2 | Pin the CLI-first order in the hub texts | skillstation_group.rs tests, principles.rs tests | cbm-1 | tests fail if the CLI path drops | same tests |
| cbm-3 | Add the CLI-before-MCP doctrine rule | LEADER.md, rule index concept, skillstation concept, rendered copies | — | the leader doctrine names both CLIs first | knowledge check, release-manifest check, rule_index_parity |
| cbm-4 | Check the doctrine rule and its rendered copies | cbm-4-check.md | cbm-3 | a second worker confirms the rule text | knowledge check, release-manifest check |

```json
[
{"id":"cbm-1","feature":"cli-before-mcp","title":"Name the skillstation CLI first in the alias and principle header","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- principles skillstation onboard",
"must_haves":{"truths":["render_alias tells the agent to run skillstation skill show <id> --json and follow its content field when manifest.status is active, and names the MCP tool skill_get as the fallback for relative files, other states, a remote hub, or a CLI failure","the hub-mode principle header names skillstation skill show <slug> --json first and skill_get as the fallback; the local-mode header is byte-identical to before","the alias success message no longer says the agent loads the skill only through skill_get"]},
"decisions":["cede1997-79d1-42f8-b083-f51262c5c561","17412392-890b-4f1c-95ff-26d518ead2c8","069eed40-e88d-40fe-8c0a-8ffc24847dd6"],
"files":["packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs","packages/bee-rs/crates/bee/src/principles.rs","packages/bee-rs/crates/bee/src/onboard/tests.rs"],
"read_first":["docs/history/cli-before-mcp/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs","packages/bee-rs/crates/bee/src/principles.rs"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md"],
"action":"Implement CONTEXT.md D3 and D4 in text only; no logic change. (1) skillstation_group.rs render_alias body after the frontmatter becomes these lines: `This skill lives in the Skillstation hub.` / `Load it with the CLI first: run skillstation skill show {id} --json and follow its content field as this skill when manifest.status is active.` / `Call the skillstation MCP tool skill_get with skill_id {id} instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.` / `If neither works, tell the user to run skillstation connect -g.` Keep backticks around commands and ids as the current text does. (2) The alias success message near line 337: say the agent loads the skill with skillstation skill show, skill_get as the fallback. (3) principles.rs hub header: replace the appended sentence with ` Load each one with skillstation skill show <slug> --json (its content field); the skillstation MCP tool skill_get, the slug as skill_id, is the fallback.` and update the test at the matching expected string. Do not touch status_full/orient.rs or onboard/agents.rs (D4). Update any existing test that pins an old full string. No new tests (cbm-2 owns them). No code comments. Proof: the cell verify; quote the counts."},
{"id":"cbm-2","feature":"cli-before-mcp","title":"Pin the CLI-first order in the hub texts","lane":"standard","role":"test","deps":["cbm-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- principles skillstation",
"must_haves":{"truths":["a test asserts render_alias output contains skillstation skill show and skill_get, with skillstation skill show at a lower byte offset","a test asserts the hub-mode principle header contains skillstation skill show and skill_get in that order, and the local-mode header contains neither"]},
"decisions":["069eed40-e88d-40fe-8c0a-8ffc24847dd6","cede1997-79d1-42f8-b083-f51262c5c561"],
"files":["packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs","packages/bee-rs/crates/bee/src/principles.rs"],
"read_first":["docs/history/cli-before-mcp/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs","packages/bee-rs/crates/bee/src/principles.rs"],
"affects_skills":[],"affects_specs":[],
"action":"Independently check cbm-1 and pin CONTEXT.md D5 with behavior tests in the existing test modules of skillstation_group.rs and principles.rs. Assert on rendered output: alias render for an id contains `skillstation skill show <id> --json` and `skill_get`, CLI first; hub-mode principle header (principle_lines on a temp repo with skillstation.principles set to hub) contains `skillstation skill show` before `skill_get`; local mode contains neither. State how each new test would fail against the pre-cbm-1 text. Only add tests; report any defect in cbm-1 instead of rewriting its text. No code comments. Proof: the cell verify; quote the counts and the new test names."},
{"id":"cbm-3","feature":"cli-before-mcp","title":"Add the CLI-before-MCP doctrine rule","lane":"standard","role":"docs","deps":[],
"verify":".bee/bin/bee knowledge check && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity",
"must_haves":{"truths":["packages/bee/LEADER.md carries a rule agents-cli-before-mcp marker block that maps mcp-gateway tools, describe and call to search_tools, get_tool_definition and call_tool, and skillstation skill show <id> --json to skill_get","the rule names the fallback triggers, the outcome_unknown no-re-send rule, the relative-file and non-active cases that take skill_get, and that skill_resolve and skill_feedback stay on MCP","the rule index in router-triage-and-the-agents-md-duplication-boundary.md has an agents-cli-before-mcp row citing cli-before-mcp D1 to D3","skillstation-hub-skills-and-aliases.md says the alias and the hub principle header name skillstation skill show first and skill_get as the fallback, citing cli-before-mcp D3 and D4","the rendered copies match the sources (release-manifest --check clean)"]},
"decisions":["bf107c86-4cdd-416d-a6e0-ebd5760f7d4c","27015a53-57c2-4564-84ac-074e976e142b","17412392-890b-4f1c-95ff-26d518ead2c8"],
"files":["packages/bee/LEADER.md",".bee/LEADER.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md","docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md","packages/bee-rs/crates/bee/src/generated/registry_payload.json",".bee/onboarding.json","skills-lock.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/cli-before-mcp/CONTEXT.md","packages/bee/LEADER.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md","docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
"action":"Implement CONTEXT.md D1 to D3 in docs. (1) packages/bee/LEADER.md, section Work in parallel, coordinate through the store: directly after the agents-other-project-to-its-leader block, add a block marked like the others with rule id agents-cli-before-mcp. Content, as bullets in the doctrine's plain style: a tool that ships a CLI and MCP tools is reached through its CLI first and the MCP tool is the fallback, because the CLI runs the same handler and puts far less text in context. mcp-gateway: `mcp-gateway tools <keywords>` (MCP search_tools), `mcp-gateway describe <upstream.tool>` (get_tool_definition), `mcp-gateway call [--text] <upstream.tool> '<json>'` (call_tool); a tool that is not read-only still needs the user's yes. Skillstation: `skillstation skill show <id> --json` loads a hub skill (MCP skill_get) — follow its content field, only when manifest.status is active; call skill_get when the content sends you to a relative file of the skill (the write guard refuses reads inside the hub folder, and skill_get copies the skill to a readable cache), when the status is not active, or when the hub is on another machine; skill_resolve and skill_feedback have no CLI verb and stay on MCP. Fallback: the binary is not on PATH, the verb is unknown (exit 2), a guard or permission refuses the shell call, or the call fails with a tool error. After `mcp-gateway call` exits 1 with code outcome_unknown, never repeat it through call_tool: read the system state first. A subagent with no shell uses the MCP tool. waggledance has its own mapping in rule agents-other-project-to-its-leader. (2) Rule index in router-triage-and-the-agents-md-duplication-boundary.md: add the agents-cli-before-mcp row after the agents-other-project-to-its-leader row, same shape (home, spoken line, applied_at), citing cli-before-mcp D1, D2, D3 (history D-numbers, never raw ids). (3) skillstation-hub-skills-and-aliases.md: in Write an alias and in the skillstation.principles business rule, say the alias and the hub principle header name skillstation skill show <id> --json first and skill_get as the fallback, cite cli-before-mcp D3 and D4, and add that orient's playbook pointer and the agent tools line keep skill_get (D4). (4) Run `bee dev regen` so .bee/LEADER.md and the generated outputs follow; restore any hub-served skill under .claude/skills that regen re-rendered. Plain English, ASD-STE100 style, no code comments. Proof: the cell verify; quote the counts."},
{"id":"cbm-4","feature":"cli-before-mcp","title":"Check the doctrine rule and its rendered copies","lane":"standard","role":"test","deps":["cbm-3"],
"verify":".bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check",
"must_haves":{"truths":["the agents-cli-before-mcp block in packages/bee/LEADER.md matches CONTEXT.md D1 to D3 and contradicts no other LEADER.md rule","the rule index row and the skillstation concept cite cli-before-mcp D-numbers, never raw decision ids","every rendered copy matches its source (release-manifest --check clean)"]},
"decisions":["069eed40-e88d-40fe-8c0a-8ffc24847dd6","bf107c86-4cdd-416d-a6e0-ebd5760f7d4c"],
"files":["docs/history/cli-before-mcp/cbm-4-check.md"],
"read_first":["docs/history/cli-before-mcp/CONTEXT.md","packages/bee/LEADER.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md","docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md"],
"affects_skills":[],"affects_specs":[],
"action":"Independently check cell cbm-3 against CONTEXT.md D1 to D3. Read the agents-cli-before-mcp block, the index row and the skillstation concept. Check each must_have truth and report pass or fail with a file:line anchor. Run the cell verify. Write the result to docs/history/cli-before-mcp/cbm-4-check.md (one line per truth, then the verify counts) and commit only that file. Do not edit the doctrine or the concepts; report a failure instead. No code comments."}
]
```
