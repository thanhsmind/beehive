---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: bee reaches waggledance through its CLI first, MCP second

## Summary

When bee tells an agent to reach waggledance, it now names the
`waggledance` command first (`waggledance state <project>`, `waggledance
dispatch --project <P> --leader ...`) and the MCP tool second, as the
fallback. The fallback fires when the binary is missing, does not know the
verb, or exits non-zero. The cross-project guard refusal says the same.
Nothing in bee names a `wg` command.

Mode: `standard` — 3 risk flags: external-systems (the waggledance CLI), public-contracts (the guard refusal text), changes behavior an existing test asserts (the byte-identical refusal test).
Why this is the least workflow that protects the work: the change is text, but the guard text has tests and the doctrine has rendered copies, so each gets its own proof.

## Requirements (from CONTEXT.md)

- D1: CLI verb first, MCP tool second, in every home that names waggledance.
- D2 (revised): fallback on missing binary, unknown verb (`unrecognized subcommand`, exit 2), a guard-refused shell call, or a tool error; one request id shared by the CLI dispatch and its MCP fallback.
- D3: no `wg` command anywhere.
- D4: guard refusal names the CLI first and the MCP tools as fallback; the rest stays.
- D5: registry read keeps `waggledance list --json`.
- D6: no bee reader of `waggledance_runs.task` or the rollup `sessions`/`recent_decisions`; prove by search.
- D7: one home for the mapping and fallback — rule `agents-other-project-to-its-leader`; handoff homes cite it.
- D8: `<project>` is the registry id, never a path; a task file stays in this project; task through `--task`, `--task-file` or stdin; `--follow-up-of` run from `leaders[].runs` of `waggledance state`.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The refusal text names only the MCP tools today | read | packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs:246 | `FIX: call waggledance_ask_state for that project, then waggledance_dispatch with leader: true` |
| 2 | A test pins the refusal text byte for byte | read | packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:7113 | `FIX: call waggledance_ask_state for that project, then waggledance_dispatch with leader: true` |
| 3 | The shared refusal assert checks the MCP tool names | read | packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:6611 | `assert!(e.stderr.contains("waggledance_ask_state"), "{}", e.stderr);` |
| 4 | The registry read uses the list verb | read | packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs:308 | `.args(["list", "--json"])` |
| 5 | The rule home names only the MCP tools | read | packages/bee/LEADER.md:235 | `through waggledance. Call` |
| 6 | The handoff playbook names only the MCP dispatch | read | packages/bee/playbooks/handoff.md:22 | `door when one exists (waggledance:` |
| 7 | The installed waggledance binary predates the agent-cli verbs | ran | waggledance projects | `error: unrecognized subcommand 'projects'` |
| 8 | The installed binary still serves the registry list | ran | waggledance list --json | `{"projects":[{"id":"openmausbot","name":"OpenMausBot","root_path":"/mnt/data/Projects/refs/OpenMausBot"}` |
| 9 | The installed binary (0.5.5) does not know state | ran | waggledance state --help | `error: unrecognized subcommand 'state'` |
| 10 | The installed binary does not know dispatch | ran | waggledance dispatch --help | `error: unrecognized subcommand 'dispatch'` |
| 11 | An unknown verb exits 2 | ran | waggledance projects; echo exit | `projects exit=2` |

## Discovery

`rg 'waggledance_'` over `skills/`, `packages/` and `docs/` (outside
`docs/history/`) finds the homes: `packages/bee/LEADER.md`,
`packages/bee/playbooks/handoff.md`, `skills/bee-hive/references/go-mode.md`,
`skills/bee-swarming/SKILL.md`,
`skills/bee-swarming/references/swarming-reference.md`,
`docs/handbook/stages/swarming.md`, `docs/config-reference.md`, two
knowledge concepts, `cross_project.rs` and its tests. No Rust code calls a
waggledance MCP tool or parses a `waggledance_runs` row (D6). No home names
`wg`. The rendered copies under `.bee/`, `.claude/`, `.agents/`,
`.claude-plugin/`, `.codex-plugin/` and `.opencode/` follow from
`bee dev regen`.

## Approach

Recommended path: write the full mapping and fallback once in rule
`agents-other-project-to-its-leader` (D1, D2, D7). The handoff homes name
the CLI handoff call `waggledance dispatch --project <P> --leader
--follow-up-of <run> --handoff --task-file <file>` and cite the rule for the
fallback. The guard refusal gets the CLI-first remedy (D4); its asserts gain
the CLI verbs. The registry read stays (D5).

Rejected: a bee verb that wraps waggledance (bee would own a second copy of
waggledance's contract); restating the fallback in every handoff home (five
copies of one fact, D7); switching the registry read to `projects` (the
installed binary refuses it, claim 7).

Risk map:
- Agents on an older waggledance follow the CLI and stall — MEDIUM — Lands in: wcli-2 — proof: the rule text names the unknown-verb fallback; claim 7 shows the case is real.
- Rendered copies drift from the source — MEDIUM — Lands in: wcli-2 — proof: `bee dev release-manifest --check` clean after `bee dev regen`.
- The refusal loses a part another test or agent relies on — LOW — Lands in: wcli-1 — proof: `cross_project` tests green, MCP names still present.

Waves: wcli-1 and wcli-2 run in parallel (no shared files); wcli-3 waits for both.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wcli-1 changes the guard refusal and its tests."},
    {"stage":"docs","classification":"required","role":"docs","reason":"wcli-2 rewrites the doctrine, skill and handbook homes and runs regen."},
    {"stage":"writing","classification":"required","role":"writer","reason":"wcli-3 consolidates knowledge and writes delivery.md."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard: the hat wave is the plan check."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Standard hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Standard hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard hat wave; cites the SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard hat wave."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release runs through scripts/release.sh by the leader."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"wcli-1 writes its own red-first asserts."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader read every home during planning."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Phase plan, one slice. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

SMALLER PATH: change only the rule home and leave the handoff homes and the guard text on MCP — cheaper, but D1 says every home, and the guard text is the remedy agents read most. One cell for all text — cheaper in dispatches, but it mixes a Rust test change with regen output and loses the parallel wave. PASS.

Plan check (hat wave synthesis, five seats; reports under `.bee/mailbox/job-17944173198*` and `job-1794417477185-*`): no BLOCKER. Accepted: a dispatch fallback can start a second run (facts-gaps 1, user-impact F1, risks R1, value F3) — closed by the shared request id in D2; a guard-refused CLI call had no next step and `<project>` invited a path (risks R2, value F4, user-impact F3) — closed by D2 and D8; only `--task-file` named (user-impact F4, value F5) — closed by D8; the installed binary's `state`/`dispatch` unproven (facts-gaps 2, risks R3) — claims 9-11 ran: both unknown, exit 2, so this machine takes the fallback until waggledance is upgraded; doc verify ran no test that reads LEADER.md (risks R4) — wcli-2 and wcli-3 verify now run the integration tests. Noted, no change: the refusal text is longer (value F6); the hook binary carries the new text only after release reinstalls it (risks R5); the leader auto-handoff of a CLI dispatch is pre-existing behavior and out of scope (risks R3). Rejected: dropping wcli-3 into the other cells (alternatives A).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wcli-1 | Name the waggledance CLI first in the cross-project refusal | cross_project.rs, write_guard/tests.rs | — | A cross-project refusal tells the agent to run `waggledance state` and `waggledance dispatch --leader`, with the MCP tools as fallback | `cargo test … --bin bee -- cross_project` |
| wcli-2 | Name the waggledance CLI first in doctrine, skills and handbook | LEADER.md, handoff.md, 3 skill files, handbook, config-reference, rendered copies | — | The leader rule and every handoff home show the CLI call first and the MCP fallback | `bee dev release-manifest --check` + rule_index_parity, pointer_integrity, verification_contract_parity, knowledge_first_nudge |
| wcli-3 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md | wcli-1, wcli-2 | the rule index and guard concept show CLI first | `.bee/bin/bee knowledge check` + rule_index_parity |

```json
[
  {
    "id": "wcli-1",
    "feature": "waggledance-cli-first",
    "title": "Name the waggledance CLI first in the cross-project refusal",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["e1ca4356", "118414b3", "b469ea90", "7d181bf7", "76216feb", "e770bb53"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/waggledance-cli-first/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/waggledance-cli-first/CONTEXT.md D4 with D2 and D8, keeping D3 and D5. In fn refusal in cross_project.rs (anchor: `FIX: call waggledance_ask_state for that project, then waggledance_dispatch with leader: true so its own leader does the work.`) replace that one FIX sentence with exactly: `FIX: hand it to that project's own leader: run waggledance state <project-id>, then waggledance dispatch --project <project-id> --leader --request-id <id> --task \"<task>\" (the project id from waggledance list --json, never a path); if the waggledance CLI is missing, does not know the verb, is refused, or fails, call waggledance_ask_state for that project, then waggledance_dispatch with leader: true and the same request_id.` Every other sentence of the refusal, the supervisor sentences, the registry read (`waggledance list --json`) and every function signature stay unchanged. Red first in write_guard/tests.rs: in fn cp_refused add asserts that stderr contains `waggledance state`, `waggledance dispatch --project`, `--leader`, `--request-id` and `request_id` (keep the existing asserts for waggledance_ask_state, waggledance_dispatch, leader: true and guards.cross_project_allow), and update fn refusal_text_for_test to the new sentence; run the cross_project tests and watch the new asserts fail before you change cross_project.rs. The string `wg ` must not appear in the refusal. No code comments. One commit with the trailer `cell: wcli-1`.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cross_project",
    "must_haves": {
      "truths": [
        "The cross-project refusal names waggledance state and waggledance dispatch --project --leader before the MCP tools",
        "The cross-project refusal still names waggledance_ask_state and waggledance_dispatch with leader: true as the fallback",
        "The registry read still runs waggledance list --json"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs", "substantive": "the CLI-first refusal sentence"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "red-first CLI asserts and the updated byte-identical text"}
      ],
      "key_links": ["cp_refused asserts the CLI verbs and the MCP fallback in one refusal"],
      "prohibitions": ["Never name a wg command", "Never change the registry read verb", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "wcli-2",
    "feature": "waggledance-cli-first",
    "title": "Name the waggledance CLI first in doctrine, skills and handbook",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["bffdaf85", "76216feb", "7d181bf7", "a57bea25", "e770bb53"],
    "files": [
      "packages/bee/LEADER.md",
      "packages/bee/playbooks/handoff.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/handbook/stages/swarming.md",
      "docs/config-reference.md",
      ".bee/LEADER.md",
      ".bee/playbooks/handoff.md",
      ".claude/skills/bee-hive/references/go-mode.md",
      ".claude/skills/bee-swarming/SKILL.md",
      ".claude/skills/bee-swarming/references/swarming-reference.md",
      ".agents/skills/bee-hive/references/go-mode.md",
      ".agents/skills/bee-swarming/SKILL.md",
      ".agents/skills/bee-swarming/references/swarming-reference.md",
      ".claude-plugin/skills/bee-hive/references/go-mode.md",
      ".claude-plugin/skills/bee-swarming/SKILL.md",
      ".claude-plugin/skills/bee-swarming/references/swarming-reference.md",
      ".codex-plugin/skills/bee-hive/references/go-mode.md",
      ".codex-plugin/skills/bee-swarming/SKILL.md",
      ".codex-plugin/skills/bee-swarming/references/swarming-reference.md",
      ".opencode/skills/bee-hive/references/go-mode.md",
      ".opencode/skills/bee-swarming/SKILL.md",
      ".opencode/skills/bee-swarming/references/swarming-reference.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/waggledance-cli-first/CONTEXT.md",
      "docs/discovery/spec-drops/sup-20261112-1646.md",
      "packages/bee/LEADER.md",
      "packages/bee/playbooks/handoff.md"
    ],
    "affects_skills": ["skills/bee-hive/references/go-mode.md", "skills/bee-swarming/SKILL.md", "skills/bee-swarming/references/swarming-reference.md"],
    "affects_specs": [],
    "action": "Implement docs/history/waggledance-cli-first/CONTEXT.md D1, D2, D3, D7 and D8 in the hand-authored homes, ASD-STE100 style, short sentences. (1) packages/bee/LEADER.md, rule block agents-other-project-to-its-leader: keep the marker lines and the never-read and cross_project_allow sentences; say the agent uses the waggledance CLI first: `waggledance state <project-id>`, then `waggledance dispatch --project <project-id> --leader --request-id <id>` with the task through `--task`, `--task-file` or stdin (add `--follow-up-of <run>` when a `leaders[].runs` row of the state output shows its leader already runs the same work; add `--handoff` for a handoff); `<project-id>` is the `id` field of `waggledance list --json`; output is JSON. The MCP tools are the fallback: `waggledance_ask_state`, then `waggledance_dispatch` with `leader: true` (`follow_up_of`, `handoff: true`) and the SAME `request_id`, called when the `waggledance` binary is not on PATH, does not know the verb (`unrecognized subcommand`, exit 2), a guard or permission refuses the shell call, or it fails with a tool error; the shared request id means a fallback never starts a second run; with `--task-file` the fallback passes the file text as `task`. `<project-id>` is never a path, and a task file stays inside this project. Never call `wg`. This rule block is the one home of the mapping and the fallback (D7). (2) packages/bee/playbooks/handoff.md, skills/bee-hive/references/go-mode.md, skills/bee-swarming/SKILL.md, skills/bee-swarming/references/swarming-reference.md and docs/handbook/stages/swarming.md: where each says `waggledance: waggledance_dispatch with leader: true and follow_up_of ... plus handoff: true`, name the CLI call first — `waggledance dispatch --project <this project id> --leader --follow-up-of <current run> --handoff --request-id <id>` with the task through `--task`, `--task-file` or stdin — and say the MCP fallback follows rule agents-other-project-to-its-leader; use the same CLI string byte for byte in all five homes; keep every other sentence. (3) docs/config-reference.md, section guards.cross_project_allow: the remedy names `waggledance state` then `waggledance dispatch --leader` first, with `waggledance_ask_state` / `waggledance_dispatch` as the fallback. (4) Run `.bee/bin/bee dev regen` and keep every file it rewrites; list each in the cap. If regen rewrites a file outside this cell's list, name it as a scope deviation. Do not edit docs/knowledge (wcli-3 owns it) or any Rust file (wcli-1 owns them). No `wg` anywhere. One commit with the trailer `cell: wcli-2`.",
    "verify": ".bee/bin/bee dev release-manifest --check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test verification_contract_parity --test knowledge_first_nudge",
    "must_haves": {
      "truths": [
        "Rule agents-other-project-to-its-leader names waggledance state and waggledance dispatch --leader first and the MCP tools as the fallback",
        "The rule names the four fallback triggers: binary missing, unknown verb, refused shell call, tool error, and one request id shared by the CLI call and its MCP fallback",
        "Every handoff home names waggledance dispatch --leader --follow-up-of --handoff first and cites the rule for the fallback",
        "The rendered copies match the sources after bee dev regen"
      ],
      "artifacts": [
        {"path": "packages/bee/LEADER.md", "substantive": "the CLI-first rule block"},
        {"path": "packages/bee/playbooks/handoff.md", "substantive": "CLI-first handoff door"}
      ],
      "key_links": ["the handoff homes cite rule agents-other-project-to-its-leader for the fallback"],
      "prohibitions": ["Never name a wg command", "No edit under docs/knowledge", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "wcli-3",
    "feature": "waggledance-cli-first",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": ["wcli-1", "wcli-2"],
    "decisions": ["bffdaf85", "76216feb", "e1ca4356", "b469ea90", "3738deac", "a57bea25", "e770bb53"],
    "files": [
      "docs/knowledge/**",
      "docs/history/waggledance-cli-first/delivery.md"
    ],
    "read_first": [
      "docs/history/waggledance-cli-first/CONTEXT.md",
      "docs/history/waggledance-cli-first/plan.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step: `bee knowledge consolidate --feature waggledance-cli-first`. In docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md rewrite the spoken lines of rule agents-other-project-to-its-leader and agents-context-handoff-65 so they name the waggledance CLI first and the MCP tools as fallback, citing docs/history/waggledance-cli-first/CONTEXT.md D1, D2, D7. In docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md, cross-project section, update the quoted remedy to the CLI-first sentence wcli-1 wrote (D4) and note that the registry read keeps `waggledance list --json` (D5). Record D6 (no bee reader of the runs task field or the dropped rollup keys) where the waggledance contract is documented, one line. Write docs/history/waggledance-cli-first/delivery.md listing each concept changed and why, one line each. Cite CONTEXT.md D-numbers, never raw decision ids. Never name a wg command. Run `.bee/bin/bee knowledge check` green. One commit with the trailer `cell: wcli-3`.",
    "verify": ".bee/bin/bee knowledge check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity",
    "must_haves": {
      "truths": [
        "area and pattern concepts describe the feature's changes, anchored to code",
        "docs/history/waggledance-cli-first/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {"path": "docs/history/waggledance-cli-first/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": ["the delivery lands in the same merge as the code"],
      "prohibitions": ["No restating of code in area concepts", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

| Case | Probe | Pass when |
|---|---|---|
| Happy path | Read a file in a sibling repo | exit 2; stderr contains `waggledance state`, `waggledance dispatch --project`, `--leader`, `--request-id`, `waggledance_ask_state`, `waggledance_dispatch`, `request_id` |
| Byte-identical opt-out | supervisor fixture with no key | refusal equals the new full text |
| Doctrine copies | `bee dev release-manifest --check` | no drift |
| Main vs head | wcli-1 red-first: the new asserts run against the old text, then the new | red before the edit, green after |

## Open Questions

(none)

## Out of scope

- A bee verb that wraps waggledance.
- Changing the guard registry read verb (D5).
- Changes in the waggledance repo.
